//! REPLAY (court step 16): re-derive an admission decision from PERSISTED
//! state only.
//!
//! `admit_payment_journaled` snapshots every stateless input of an admission
//! (prepared effect, certificate, key registry, epochs, audience, clock,
//! spend policy) and durably writes it after a successful admission.
//! Unanchored `replay_admission` proves internal consistency only; use
//! `replay_admission_anchored` to bind the record to a trusted key registry.
//! `replay_admission` re-runs the *stateless* checks against that snapshot
//! and never touches the ClaimStore, ledger, rail, nonce fence or network,
//! and never actuates.
//!
//! Stateful checks (durable nonce, budget/epoch cap, claim uniqueness,
//! reversal-original lookup) are EXCLUDED BY DESIGN: they depend on history
//! (what else was admitted before or since), so they cannot be reproduced
//! from an admission-time snapshot. The verdict is therefore "the decision
//! was lawful given its inputs", not "it would be admitted again now".

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::sa2a_security::encoding::sha256_tagged;
use crate::sa2a_security::epoch::SecurityEpochs;
use crate::sa2a_security::{ActuationCertificate, CertificateVerifier, KeyRecord, KeyRegistry, PreparedEffect};

use super::admission::{
    admit_payment, admit_payment_screened, AdmissionContext, PaymentAdmission, Screening, ScreeningEvidence,
};
use super::dirlock::{publish_new, publish_replace};
use super::effect::PaymentEffect;
use super::obligation::{derive_obligation_id, OBLIGATION_ID_NOT_DERIVED};
use super::policy::SpendPolicy;
use super::refusal::{self, PayResult};

pub const REPLAY_RECORD_MISSING: &str = "REFUSED:REPLAY_RECORD_MISSING";
pub const REPLAY_RECORD_CORRUPT: &str = "REFUSED:REPLAY_RECORD_CORRUPT";
pub const REPLAY_REGISTRY_UNTRUSTED: &str = "REFUSED:REPLAY_REGISTRY_UNTRUSTED";
pub const REPLAY_DIGEST_MISMATCH: &str = "REFUSED:REPLAY_EFFECT_DIGEST_MISMATCH";

const DECISION_DOMAIN: &[u8] = b"CASTLE-REPLAY-DECISION-V1";

/// Everything the stateless part of an admission decision depended on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmissionRecord {
    pub prepared: PreparedEffect,
    pub certificate: ActuationCertificate,
    pub registry: Vec<KeyRecord>,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u64,
    pub audience: String,
    pub now_ms: u64,
    pub policy: SpendPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayVerdict {
    Reproduced { effect_digest: String, decision_digest: String },
    Diverged { reason: String },
}

#[derive(Debug, Clone)]
pub struct AdmissionJournal {
    root: PathBuf,
}

impl AdmissionJournal {
    pub fn open(root: impl Into<PathBuf>) -> PayResult<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
        Ok(Self { root })
    }

    fn path(&self, effect_digest: &str) -> PathBuf {
        self.root.join(format!("{}.admission.json", effect_digest.trim_start_matches("sha256:")))
    }

    /// Load the persisted record for an effect digest.
    pub fn load(&self, effect_digest: &str) -> PayResult<AdmissionRecord> {
        let bytes = fs::read(self.path(effect_digest)).map_err(|_| REPLAY_RECORD_MISSING.to_string())?;
        serde_json::from_slice(&bytes).map_err(|_| REPLAY_RECORD_CORRUPT.to_string())
    }

    fn store(&self, effect_digest: &str, record: &AdmissionRecord) -> PayResult<()> {
        let bytes = serde_json::to_vec(record).map_err(|_| REPLAY_RECORD_CORRUPT.to_string())?;
        let path = self.path(effect_digest);
        if !publish_new(&self.root, &path, &bytes)? {
            // A prior admission of this digest (e.g. after a Refused claim)
            // left a record; the latest lawful admission supersedes it.
            publish_replace(&self.root, &path, &bytes)?;
        }
        Ok(())
    }
}

/// `admit_payment` plus a durable snapshot of its stateless inputs. A refused
/// admission writes nothing.
pub fn admit_payment_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    journal: &AdmissionJournal,
) -> PayResult<PaymentAdmission> {
    let record = snapshot(&prepared, certificate, ctx);
    let admission = admit_payment(prepared, certificate, ctx)?;
    journal.store(admission.effect().digest(), &record)?;
    Ok(admission)
}

/// `admit_payment_screened` plus a durable snapshot of the stateless inputs (screened + journaled).
pub fn admit_payment_screened_journaled(
    prepared: PreparedEffect,
    certificate: &ActuationCertificate,
    ctx: &AdmissionContext<'_>,
    screening: &Screening<'_>,
    journal: &AdmissionJournal,
) -> PayResult<(PaymentAdmission, ScreeningEvidence)> {
    let record = snapshot(&prepared, certificate, ctx);
    let (admission, evidence) = admit_payment_screened(prepared, certificate, ctx, screening)?;
    journal.store(admission.effect().digest(), &record)?;
    Ok((admission, evidence))
}

fn snapshot(prepared: &PreparedEffect, certificate: &ActuationCertificate, ctx: &AdmissionContext<'_>) -> AdmissionRecord {
    AdmissionRecord {
        prepared: prepared.clone(),
        certificate: certificate.clone(),
        registry: ctx.registry.records(),
        policy_epoch: ctx.epochs.policy,
        revocation_epoch: ctx.epochs.revocation,
        generation: ctx.epochs.generation,
        audience: ctx.audience.to_string(),
        now_ms: ctx.now_ms,
        policy: ctx.policy.clone(),
    }
}

/// Like `replay_admission`, but the record's key-registry snapshot must equal `trusted_registry`.
/// Unanchored replay proves only INTERNAL CONSISTENCY (the record verifies against the registry
/// it carries); a record written by anyone holding their own keys reproduces. Only anchoring to a
/// registry you trust establishes AUTHORITY.
pub fn replay_admission_anchored(
    journal: &AdmissionJournal,
    effect_digest: &str,
    trusted_registry: &KeyRegistry,
) -> PayResult<ReplayVerdict> {
    let rec = journal.load(effect_digest)?;
    let canon = |rs: &[KeyRecord]| -> Vec<String> {
        let mut v: Vec<String> = rs.iter().map(|r| serde_json::to_string(r).unwrap_or_default()).collect();
        v.sort();
        v
    };
    if canon(&rec.registry) != canon(&trusted_registry.records()) {
        return diverged(REPLAY_REGISTRY_UNTRUSTED);
    }
    replay_admission(journal, effect_digest)
}

fn diverged(reason: impl Into<String>) -> PayResult<ReplayVerdict> {
    Ok(ReplayVerdict::Diverged { reason: reason.into() })
}

/// Re-derive the stateless admission decision from the persisted record ONLY.
/// `Err` means the record itself is missing/corrupt; `Diverged` means the
/// record is readable but a stateless check no longer holds.
pub fn replay_admission(journal: &AdmissionJournal, effect_digest: &str) -> PayResult<ReplayVerdict> {
    let rec = journal.load(effect_digest)?;
    let mut checks: Vec<(&str, String)> = Vec::new();

    let effect = match PaymentEffect::from_prepared(rec.prepared.clone()) {
        Ok(e) => e,
        Err(e) => return diverged(e),
    };
    checks.push(("strict_parse", "ok".into()));

    if effect.digest() != effect_digest {
        return diverged(REPLAY_DIGEST_MISMATCH);
    }
    checks.push(("effect_digest", effect.digest().to_string()));

    let registry = KeyRegistry::from_records(rec.registry.clone());
    let verifier = CertificateVerifier {
        registry: &registry,
        expected_epochs: SecurityEpochs {
            policy: rec.policy_epoch,
            revocation: rec.revocation_epoch,
            generation: rec.generation,
        },
        expected_audience: &rec.audience,
        now_ms: rec.now_ms,
    };
    let receipt = match verifier.verify(effect.prepared(), &rec.certificate) {
        Ok(r) => r,
        Err(r) => return diverged(r.to_string()),
    };
    checks.push(("certificate", format!("ok:{}", receipt.verified_custodian_ids.join(","))));

    match rec.policy.check_static(&effect) {
        Ok(p) => {
            if let Err(e) = p.check_quorum(rec.certificate.threshold) {
                return diverged(e);
            }
        }
        Err(e) => return diverged(e),
    }
    checks.push(("static_policy", "ok".into()));

    if rec.policy.require_derived_obligation {
        let derived = effect
            .invoice_ref()
            .map(|inv| derive_obligation_id(effect.payer(), effect.payee(), effect.purpose(), inv));
        if derived.as_deref() != Some(effect.obligation_id()) {
            return diverged(OBLIGATION_ID_NOT_DERIVED);
        }
        checks.push(("derived_obligation", "ok".into()));
    } else {
        checks.push(("derived_obligation", "not_required".into()));
    }

    let body = serde_json_canonicalizer::to_vec(&checks).map_err(|_| REPLAY_RECORD_CORRUPT.to_string())?;
    Ok(ReplayVerdict::Reproduced {
        effect_digest: effect.digest().to_string(),
        decision_digest: sha256_tagged(DECISION_DOMAIN, &body),
    })
}
