//! PreparedEconomicEffect v1: a self-identifying, canonically hashed economic
//! effect. `effect_id` (== `idempotency_key`) is the digest of every
//! consequential field, so mutating any of them breaks identity.

use serde::{Deserialize, Serialize};

use crate::sa2a_security::encoding::{push_field, sha256_tagged};

use super::admission::PaymentAdmission;
use super::money::Currency;
use super::refusal::{refuse, PayResult};

pub const PEE_IDENTITY_MISMATCH: &str = "REFUSED:EFFECT_IDENTITY_MISMATCH";
pub const PEE_EXPIRED: &str = "REFUSED:EFFECT_EXPIRED";
pub const PEE_NOT_YET_VALID: &str = "REFUSED:EFFECT_NOT_YET_VALID";
pub const PEE_BINDING_INCOMPLETE: &str = "REFUSED:EFFECT_BINDING_INCOMPLETE";

const PEE_DOMAIN: &[u8] = b"CASTLE-PEE-V1";
const AUTHORITY_DOMAIN: &[u8] = b"CASTLE-PEE-AUTHORITY-V1";
const EXCLUDED: [&str; 3] = ["effect_id", "idempotency_key", "canonical_payload_digest"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedEconomicEffect {
    pub version: u32,
    pub effect_id: String,
    pub obligation_id: String,
    pub principal_id: String,
    pub payer_account: String,
    pub beneficiary_account: String,
    pub amount_minor: String,
    pub currency: Currency,
    pub purpose: String,
    pub authority_grant_id: String,
    pub authority_digest: String,
    pub policy_profile_id: String,
    pub law_state_digest: String,
    pub counterparty_evidence_digest: String,
    pub funding_source_id: String,
    pub resource_reservation_id: String,
    pub rail_profile_id: String,
    pub message_profile_version: String,
    pub idempotency_key: String,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub nonce: String,
    pub parent_receipt: String,
    pub canonical_payload_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBindings {
    pub policy_profile_id: String,
    pub law_state_digest: String,
    pub counterparty_evidence_digest: String,
    pub funding_source_id: String,
    pub resource_reservation_id: String,
    pub rail_profile_id: String,
    pub message_profile_version: String,
    pub parent_receipt: String,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
}

impl EffectBindings {
    fn check_complete(&self) -> PayResult<()> {
        let req = [
            &self.policy_profile_id,
            &self.law_state_digest,
            &self.counterparty_evidence_digest,
            &self.funding_source_id,
            &self.resource_reservation_id,
            &self.rail_profile_id,
            &self.message_profile_version,
            &self.parent_receipt,
        ];
        if req.iter().any(|s| s.is_empty()) {
            return refuse(PEE_BINDING_INCOMPLETE);
        }
        Ok(())
    }
}

fn authority_digest(admission: &PaymentAdmission) -> String {
    let r = admission.verification();
    let mut body = Vec::new();
    push_field(&mut body, &u64::try_from(r.verified_custodian_ids.len()).unwrap_or(u64::MAX).to_be_bytes());
    for c in &r.verified_custodian_ids {
        push_field(&mut body, c.as_bytes());
    }
    push_field(&mut body, &r.policy_epoch.to_be_bytes());
    push_field(&mut body, &r.revocation_epoch.to_be_bytes());
    push_field(&mut body, &r.generation.to_be_bytes());
    push_field(&mut body, r.audience.as_bytes());
    sha256_tagged(AUTHORITY_DOMAIN, &body)
}

impl PreparedEconomicEffect {
    /// JCS bytes of the struct without effect_id / idempotency_key / payload digest.
    fn identity_bytes(&self) -> PayResult<Vec<u8>> {
        let mut v = serde_json::to_value(self).map_err(|_| PEE_IDENTITY_MISMATCH.to_string())?;
        let Some(map) = v.as_object_mut() else {
            return refuse(PEE_IDENTITY_MISMATCH);
        };
        for k in EXCLUDED {
            map.remove(k);
        }
        serde_json_canonicalizer::to_vec(&v).map_err(|_| PEE_IDENTITY_MISMATCH.to_string())
    }

    pub fn seal(admission: &PaymentAdmission, b: &EffectBindings) -> PayResult<Self> {
        b.check_complete()?;
        // A screened admission fixes the screening outcome: the bindings must carry exactly it.
        if let Some(sc) = admission.screening() {
            if b.law_state_digest != sc.compliance_bundle_digest
                || b.counterparty_evidence_digest != sc.counterparty_evidence_digest
            {
                return refuse(PEE_BINDING_INCOMPLETE);
            }
        }
        let e = admission.effect();
        let r = admission.verification();
        let mut pee = Self {
            version: 1,
            effect_id: String::new(),
            obligation_id: e.obligation_id().to_string(),
            principal_id: e.principal().to_string(),
            payer_account: e.payer().to_string(),
            beneficiary_account: e.payee().to_string(),
            amount_minor: e.money().minor.to_string(),
            currency: e.money().currency,
            purpose: e.purpose().to_string(),
            authority_grant_id: format!("grant:{}#gen{}", r.audience, r.generation),
            authority_digest: authority_digest(admission),
            policy_profile_id: b.policy_profile_id.clone(),
            law_state_digest: b.law_state_digest.clone(),
            counterparty_evidence_digest: b.counterparty_evidence_digest.clone(),
            funding_source_id: b.funding_source_id.clone(),
            resource_reservation_id: b.resource_reservation_id.clone(),
            rail_profile_id: b.rail_profile_id.clone(),
            message_profile_version: b.message_profile_version.clone(),
            idempotency_key: String::new(),
            created_at_ms: b.created_at_ms,
            expires_at_ms: b.expires_at_ms,
            nonce: admission.nonce().to_string(),
            parent_receipt: b.parent_receipt.clone(),
            canonical_payload_digest: String::new(),
        };
        let bytes = pee.identity_bytes()?;
        pee.effect_id = sha256_tagged(PEE_DOMAIN, &bytes);
        pee.idempotency_key = pee.effect_id.clone();
        pee.canonical_payload_digest = sha256_tagged(b"", &bytes);
        Ok(pee)
    }

    /// Recompute `effect_id`, `idempotency_key` and the payload digest in place
    /// (used after a deliberate field change; the result is a *different* effect).
    ///
    /// WARNING: test/tooling hook. A resealed effect is self-consistent, so `verify_identity`
    /// passes; only `verify_against(admission)` detects a substituted field.
    #[doc(hidden)]
    pub fn reseal_identity(&mut self) -> PayResult<()> {
        let bytes = self.identity_bytes()?;
        self.effect_id = sha256_tagged(PEE_DOMAIN, &bytes);
        self.idempotency_key = self.effect_id.clone();
        self.canonical_payload_digest = sha256_tagged(b"", &bytes);
        Ok(())
    }

    pub fn verify_identity(&self) -> PayResult<()> {
        let bytes = self.identity_bytes()?;
        if self.version != 1
            || self.effect_id != sha256_tagged(PEE_DOMAIN, &bytes)
            || self.idempotency_key != self.effect_id
            || self.canonical_payload_digest != sha256_tagged(b"", &bytes)
        {
            return refuse(PEE_IDENTITY_MISMATCH);
        }
        Ok(())
    }

    /// Bind this effect to the admission it claims to come from: identity is self-consistent AND
    /// payer, beneficiary, amount, currency, purpose, obligation, principal, nonce and authority
    /// equal what the admission authorized. A resealed effect with a substituted field is refused.
    pub fn verify_against(&self, admission: &PaymentAdmission) -> PayResult<()> {
        self.verify_identity()?;
        let e = admission.effect();
        let r = admission.verification();
        let ok = self.payer_account == e.payer()
            && self.beneficiary_account == e.payee()
            && self.amount_minor == e.money().minor.to_string()
            && self.currency == e.money().currency
            && self.purpose == e.purpose()
            && self.obligation_id == e.obligation_id()
            && self.principal_id == e.principal()
            && self.nonce == admission.nonce()
            && self.authority_grant_id == format!("grant:{}#gen{}", r.audience, r.generation)
            && self.authority_digest == authority_digest(admission);
        if !ok {
            return refuse(PEE_IDENTITY_MISMATCH);
        }
        if let Some(sc) = admission.screening() {
            if self.law_state_digest != sc.compliance_bundle_digest
                || self.counterparty_evidence_digest != sc.counterparty_evidence_digest
            {
                return refuse(PEE_IDENTITY_MISMATCH);
            }
        }
        Ok(())
    }

    pub fn check_fresh(&self, now_ms: u64) -> PayResult<()> {
        if now_ms < self.created_at_ms {
            return refuse(PEE_NOT_YET_VALID);
        }
        if now_ms >= self.expires_at_ms {
            return refuse(PEE_EXPIRED);
        }
        Ok(())
    }
}
