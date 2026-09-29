use crate::sa2a_security::epoch::SecurityEpochs;
use crate::sa2a_security::verifier::VerificationReceipt;
use crate::sa2a_security::{ActuationCertificate, CertificateVerifier, KeyRegistry, PreparedEffect};

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
#[derive(Debug, Clone)]
pub struct PaymentAdmission {
    effect: PaymentEffect,
    receipt: VerificationReceipt,
    generation: u64,
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
    let effect = PaymentEffect::from_prepared(prepared)?;

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

    Ok(PaymentAdmission { effect, receipt, generation: certificate.generation, _seal: () })
}

fn check_reversal(effect: &PaymentEffect, original: &str, claims: &ClaimStore) -> PayResult<()> {
    let Some(orig) = claims.get(original)? else {
        return refuse(refusal::REVERSAL_ORIGINAL_NOT_EXECUTED);
    };
    if orig.state != ClaimState::Executed {
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
