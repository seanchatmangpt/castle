use crate::sa2a_security::epoch::SecurityEpochs;
use crate::sa2a_security::verifier::VerificationReceipt;
use crate::sa2a_security::{ActuationCertificate, CertificateVerifier, KeyRegistry, PreparedEffect};

use super::compliance::{run_controls, ComplianceControl};
use super::counterparty::CounterpartyRegistry;
use super::claim_store::{Claim, ClaimState, ClaimStore};
use super::effect::PaymentEffect;
use super::nonce::DurableNonceFence;
use super::policy::SpendPolicy;
use super::refusal::{self, refuse, PayResult};

/// Sealed proof that a payment effect passed identity, authority, policy,
/// nonce, budget and reversal checks. The only constructor is `admit_payment`;
/// fields are private, so no other code (or agent) can fabricate one.
///
/// It is *not* DO authority: DO additionally requires the castle
/// `ConstructAdmission` chain (`execute_payment`).
#[derive(Debug)]
pub struct PaymentAdmission {
    effect: PaymentEffect,
    receipt: VerificationReceipt,
    generation: u64,
    nonce: String,
    screening: Option<ScreeningEvidence>,
    _seal: (),
}

impl PaymentAdmission {
    #[must_use]
    pub const fn effect(&self) -> &PaymentEffect {
        &self.effect
    }
    #[must_use]
    pub const fn verification(&self) -> &VerificationReceipt {
        &self.receipt
    }
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    /// Certificate nonce consumed by this admission.
    #[must_use]
    pub fn nonce(&self) -> &str {
        &self.nonce
    }
    /// Screening evidence; `Some` only when admitted via `admit_payment_screened`.
    #[must_use]
    pub const fn screening(&self) -> Option<&ScreeningEvidence> {
        self.screening.as_ref()
    }
}

pub struct AdmissionContext<'a> {
    pub registry: &'a KeyRegistry,
    pub epochs: SecurityEpochs,
    pub audience: &'a str,
    pub now_ms: u64,
    pub policy: &'a SpendPolicy,
    pub claims: &'a ClaimStore,
    pub nonces: &'a DurableNonceFence,
}

/// Admission order (each step fail-closed):
/// strict parse → certificate verification (SA2A C2/C3) → static policy →
/// reversal checks → durable nonce claim → atomic budget reservation + claim.
pub fn admit_payment(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
) -> PayResult<PaymentAdmission> {
    admit_inner(prepared, certificate, ctx, None)
}

/// Refusal when policy demands screening but the unscreened admission path was used.
pub const PAYMENT_SCREENING_REQUIRED: &str = "REFUSED:PAYMENT_SCREENING_REQUIRED";

/// Screening inputs: compliance controls plus the counterparty registry that must
/// resolve both payer and payee accounts.
pub struct Screening<'a> {
    pub controls: &'a [&'a dyn ComplianceControl],
    pub counterparties: &'a CounterpartyRegistry,
}

/// Digests of the screening evidence. Callers feed these into
/// `EffectBindings.counterparty_evidence_digest` (`counterparty_evidence_digest`)
/// and their law/compliance binding (`compliance_bundle_digest`) so the sealed
/// `PreparedEconomicEffect` identity depends on the exact screening outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreeningEvidence {
    pub compliance_bundle_digest: String,
    pub counterparty_evidence_digest: String,
}

/// strict parse -> screening -> the normal admission path. A sanctions hit or an
/// unresolved counterparty refuses BEFORE the nonce is burned or budget reserved.
pub fn admit_payment_screened(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
) -> PayResult<(PaymentAdmission, ScreeningEvidence)> {
    let effect = PaymentEffect::from_prepared(prepared.clone())?;
    let bundle = run_controls(screening.controls, &effect)?;
    let payer = screening.counterparties.resolve(effect.payer())?;
    let payee = screening.counterparties.resolve(effect.payee())?;
    let evidence = ScreeningEvidence {
        compliance_bundle_digest: bundle.bundle_digest,
        counterparty_evidence_digest: screening.counterparties.evidence_digest(payer, payee),
    };
    let admission = admit_inner(prepared, certificate, ctx, Some(evidence.clone()))?;
    Ok((admission, evidence))
}

fn admit_inner(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: Option<ScreeningEvidence>,
) -> PayResult<PaymentAdmission> {
    let effect = PaymentEffect::from_prepared(prepared)?;
    if ctx.policy.require_screening && screening.is_none() {
        return refuse(PAYMENT_SCREENING_REQUIRED);
    }

    let verifier = CertificateVerifier {
        registry: ctx.registry,
        expected_epochs: ctx.epochs,
        expected_audience: ctx.audience,
        now_ms: ctx.now_ms,
    };
    let receipt = verifier
        .verify(effect.prepared(), certificate)
        .map_err(|r| r.to_string())?;

    let policy = ctx.policy.check_static(&effect)?;
    // Quorum floor comes from policy, never from the certificate; refused before the nonce burns.
    policy.check_quorum(certificate.threshold)?;

    if ctx.policy.require_derived_obligation {
        let derived = effect.invoice_ref().map(|inv| {
            super::obligation::derive_obligation_id(effect.payer(), effect.payee(), effect.purpose(), inv)
        });
        if derived.as_deref() != Some(effect.obligation_id()) {
            return refuse(super::obligation::OBLIGATION_ID_NOT_DERIVED);
        }
    }

    if let Some(original) = effect.reverses() {
        check_reversal(&effect, original, ctx.claims)?;
    }

    ctx.nonces.claim(&certificate.principal, &certificate.nonce)?;

    let currency = effect.money().currency;
    let epoch_cap = if effect.reverses().is_some() {
        None
    } else {
        policy.epoch_cap.get(&currency).copied()
    };
    let claim = Claim {
        effect_digest: effect.digest().to_string(),
        principal: effect.principal().to_string(),
        payer: effect.payer().to_string(),
        payee: effect.payee().to_string(),
        amount_minor: effect.money().minor,
        currency,
        generation: certificate.generation,
        state: ClaimState::Reserved,
        reverses: effect.reverses().map(str::to_string),
        construct_digest: None,
        detail: "ADMITTED".to_string(),
        obligation_id: effect.obligation_id().to_string(),
        purpose: effect.purpose().to_string(),
        audience: receipt.audience.clone(),
        verified_custodian_ids: receipt.verified_custodian_ids.clone(),
    };
    ctx.claims.reserve(&claim, epoch_cap)?;

    Ok(PaymentAdmission { effect, receipt, generation: certificate.generation, nonce: certificate.nonce.clone(), screening, _seal: () })
}

fn check_reversal(effect: &PaymentEffect, original: &str, claims: &ClaimStore) -> PayResult<()> {
    let Some(orig) = claims.get(original)? else {
        return refuse(refusal::REVERSAL_ORIGINAL_NOT_EXECUTED);
    };
    if !matches!(orig.state, ClaimState::Executed | ClaimState::Final) {
        return refuse(refusal::REVERSAL_ORIGINAL_NOT_EXECUTED);
    }
    if orig.reverses.is_some()
        || orig.payer != effect.payee()
        || orig.payee != effect.payer()
        || orig.currency != effect.money().currency
        || orig.principal != effect.principal()
    {
        return refuse(refusal::REVERSAL_MISMATCH);
    }
    let already = claims.reversed_total(original)?;
    if u128::from(already) + u128::from(effect.money().minor) > u128::from(orig.amount_minor) {
        return refuse(refusal::REVERSAL_EXCEEDS_ORIGINAL);
    }
    Ok(())
}
