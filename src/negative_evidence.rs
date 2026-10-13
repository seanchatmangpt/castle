//! Negative-evidence channel (findings-register item): record "we looked for
//! evidence X and it was absent" as a first-class, durable, queryable record.
//!
//! Without it, absence-of-evidence is indistinguishable from not-looked.
//! A [`NegationRecord`] carries what was looked for (predicate), where
//! (subject), when, and by whom; a [`NegativeEvidenceLedger`] admits it
//! against the admitted positive evidence set.
//!
//! ## Admission rule (search-order deterministic)
//!
//! 1. **Absent-only** — no admitted positive observation for the same
//!    `(subject, predicate)` key exists → the negation is admitted as
//!    `NegativeEvidenceStatus::Absent(Standing::Unknown)` with reason
//!    `UNKNOWN:ABSENCE_OF_EVIDENCE`. Never Alive, never Refused.
//! 2. **Designed supersession** — once a negation is admitted, a positive
//!    observation for the same key admitted *later* admits Alive through the
//!    unchanged [`admit_evidence`] gate and supersedes the negation: its
//!    status becomes `NegativeEvidenceStatus::Superseded { by: <digest> }`.
//!    No contradiction. Rule 2 is applied by
//!    [`admit_positive_and_supersede`].
//! 3. **Positive-then-absent** — a positive observation for the same key is
//!    already admitted when the Absent finding arrives → typed refusal
//!    `REFUSED:EVIDENCE_CONTRADICTION`. The record is retained in the ledger
//!    (a false negative claim is itself durable evidence about the claimant)
//!    but its status is `ContradictionRefused` — never Unknown, never Alive.
//!
//! The rule is a pure function of the admission *sequence*: the same
//! sequence always produces the same outcome, and both orderings have the
//! exactly-defined outcomes above.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::board::{admit_evidence, EvidenceAdmission, EvidenceBundle, ReceiptV2};
use crate::fortune5::{MetricObservation, Standing};
use crate::board::TrustStore;

/// Stable reason identities (typed refusal/unknown strings).
pub const REASON_ABSENCE: &str = "UNKNOWN:ABSENCE_OF_EVIDENCE";
pub const REASON_SUPERSEDED: &str = "UNKNOWN:ABSENCE_SUPERSEDED_BY_POSITIVE";
pub const REASON_CONTRADICTION: &str = "REFUSED:EVIDENCE_CONTRADICTION";
pub const REASON_INCOMPLETE: &str = "REFUSED:NEGATION_INCOMPLETE";

/// A negative finding: "we looked for `predicate` on `subject` at
/// `searched_at` (RFC 3339) as `searched_by` and it was absent."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegationRecord {
    /// What was looked for — predicate/query/metric identity.
    pub predicate: String,
    /// Where it was looked — the subject of the would-be evidence.
    pub subject: String,
    /// When the search happened (RFC 3339).
    pub searched_at: String,
    /// By whom.
    pub searched_by: String,
}

impl NegationRecord {
    /// The `(subject, predicate)` collision surface shared with positive
    /// observations (`MetricObservation::{subject, metric}`).
    #[must_use]
    pub fn key(&self) -> (&str, &str) {
        (&self.subject, &self.predicate)
    }
}

/// Stable status vocabulary for a negation record in the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegativeEvidenceStatus {
    /// Looked and absent; nothing contradicts it yet.
    Absent(Standing),
    /// A positive observation for the same key was admitted after the
    /// negation; the negation no longer maps to Unknown. `by` is the
    /// superseding positive's receipt digest.
    Superseded { by: String },
    /// The negation contradicted already-admitted positive evidence at
    /// admission time; the record is retained but refused.
    ContradictionRefused,
}

/// Durable, queryable ledger of negative findings.
#[derive(Debug, Clone, Default)]
pub struct NegativeEvidenceLedger {
    records: Vec<(NegationRecord, NegativeEvidenceStatus)>,
}

impl NegativeEvidenceLedger {
    /// Admit one negative finding against the admitted positive evidence set.
    ///
    /// # Errors
    /// `REFUSED:EVIDENCE_CONTRADICTION` — rule 3 (positive-then-absent).
    /// `REFUSED:NEGATION_INCOMPLETE` — empty predicate/subject/searched_by.
    pub fn admit_negation(
        &mut self,
        negation: NegationRecord,
        admitted: &[MetricObservation],
    ) -> Result<NegativeEvidenceStatus, String> {
        if negation.predicate.is_empty()
            || negation.subject.is_empty()
            || negation.searched_by.is_empty()
        {
            return Err(REASON_INCOMPLETE.to_string());
        }
        let key = negation.key();
        for observation in admitted {
            if observation.subject == key.0 && observation.metric == key.1 {
                self.records
                    .push((negation, NegativeEvidenceStatus::ContradictionRefused));
                return Err(REASON_CONTRADICTION.to_string());
            }
        }
        let status = NegativeEvidenceStatus::Absent(Standing::Unknown);
        self.records.push((negation, status.clone()));
        Ok(status)
    }

    /// Query the latest status recorded for `(subject, predicate)`.
    #[must_use]
    pub fn status_of(&self, subject: &str, predicate: &str) -> Option<NegativeEvidenceStatus> {
        self.records
            .iter()
            .rev()
            .find(|(n, _)| n.key() == (subject, predicate))
            .map(|(_, status)| status.clone())
    }

    /// Mark every still-Absent negation for `(subject, predicate)` superseded.
    fn supersede(&mut self, subject: &str, predicate: &str, by: &str) {
        for (negation, status) in &mut self.records {
            if status == &NegativeEvidenceStatus::Absent(Standing::Unknown)
                && negation.key() == (subject, predicate)
            {
                *status = NegativeEvidenceStatus::Superseded { by: by.to_string() };
            }
        }
    }
}

/// Admit one positive evidence bundle and, if Alive, supersede any earlier
/// Absent negation for the same `(subject, metric)` key (rule 2).
///
/// The positive evidence passes through the unchanged [`admit_evidence`]
/// gate; the negative-evidence channel adds only the supersession side
/// effect. A Refused positive never supersedes anything.
///
/// # Errors
/// Never errors in this signature; `Result` mirrors [`admit_evidence`]'s
/// caller ergonomics. Returns the admission either way.
pub fn admit_positive_and_supersede(
    bundle: &EvidenceBundle,
    ledger: &mut NegativeEvidenceLedger,
    trust: &TrustStore,
    store: &HashMap<String, ReceiptV2>,
) -> EvidenceAdmission {
    let admission = admit_evidence(bundle, trust, store);
    if admission.standing == Standing::Alive {
        if let Some(observation) = &admission.observation {
            ledger.supersede(
                &observation.subject,
                &observation.metric,
                &bundle.receipt.receipt_digest,
            );
        }
    }
    admission
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{issue_evidence_receipt, EvidenceInput, ReceiptIssueContext, TrustKey};
    use crate::fortune5::{EvidenceEpistemicClass, MetricValue};
    use ed25519_dalek::SigningKey;

    const SUBJECT: &str = "mfact@aaa:theorem_x";
    const PREDICATE: &str = "lean.witness.verified";
    const AT: &str = "2026-10-13T00:00:00Z";

    fn negation() -> NegationRecord {
        NegationRecord {
            predicate: PREDICATE.to_string(),
            subject: SUBJECT.to_string(),
            searched_at: AT.to_string(),
            searched_by: "castle-lane".to_string(),
        }
    }

    fn trust() -> TrustStore {
        let mut keys = HashMap::new();
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        keys.insert(
            "test-key".to_string(),
            TrustKey {
                key_id: "test-key".to_string(),
                verifying_key: signing.verifying_key(),
                valid_from_epoch: 0,
                revoked_at_epoch: None,
                assurance_domain: "castle-test".to_string(),
            },
        );
        TrustStore { current_epoch: 1, keys }
    }

    fn positive_bundle(
        subject: &str,
        metric: &str,
        ctx: &ReceiptIssueContext,
        signing: &SigningKey,
    ) -> EvidenceBundle {
        let input = EvidenceInput {
            metric: metric.to_string(),
            value: MetricValue::Bool(true),
            subject: subject.to_string(),
            observed_at: AT.to_string(),
            epistemic_class: EvidenceEpistemicClass::Replayed,
        };
        issue_evidence_receipt(&input, ctx, signing).expect("positive evidence issues")
    }

    fn issue_ctx() -> ReceiptIssueContext {
        ReceiptIssueContext {
            policy_digest: "a".repeat(64),
            authority_digest: "b".repeat(64),
            parent_digests: vec![],
            key_id: "test-key".to_string(),
            trust_epoch: 1,
            assurance_domain: "castle-test".to_string(),
        }
    }

    // (a) absent-only → Standing Unknown, never Alive, never Refused.
    #[test]
    fn absent_only_admits_as_unknown() {
        let mut ledger = NegativeEvidenceLedger::default();
        let status = ledger.admit_negation(negation(), &[]).expect("admits");
        assert_eq!(status, NegativeEvidenceStatus::Absent(Standing::Unknown));
        assert_eq!(
            ledger.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::Absent(Standing::Unknown))
        );
        // Durable + serializable: round-trips through JSON.
        let json = serde_json::to_string(&negation()).expect("serializes");
        assert_eq!(
            serde_json::from_str::<NegationRecord>(&json).expect("deserializes"),
            negation()
        );
    }

    // (b) absent-then-positive → positive admits Alive, negation superseded.
    #[test]
    fn absent_then_positive_supersedes_negation() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let store = trust();
        let ctx = issue_ctx();
        let mut ledger = NegativeEvidenceLedger::default();
        let admitted: Vec<MetricObservation> = Vec::new();

        ledger.admit_negation(negation(), &admitted).expect("admits");
        assert_eq!(
            ledger.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::Absent(Standing::Unknown))
        );

        let bundle = positive_bundle(SUBJECT, PREDICATE, &ctx, &signing);
        let receipt_digest = bundle.receipt.receipt_digest.clone();
        let receipt_store: HashMap<String, ReceiptV2> = HashMap::new();
        let admission =
            admit_positive_and_supersede(&bundle, &mut ledger, &store, &receipt_store);
        assert_eq!(admission.standing, Standing::Alive);

        assert_eq!(
            ledger.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::Superseded { by: receipt_digest })
        );
    }

    // (c) positive-then-absent → typed contradiction refusal; record retained.
    #[test]
    fn positive_then_absent_is_typed_contradiction_refusal() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let store = trust();
        let ctx = issue_ctx();
        let bundle = positive_bundle(SUBJECT, PREDICATE, &ctx, &signing);
        let mut ledger = NegativeEvidenceLedger::default();
        let admitted = vec![bundle.observation.clone()];

        let err = ledger
            .admit_negation(negation(), &admitted)
            .expect_err("contradiction must refuse");
        assert_eq!(err, REASON_CONTRADICTION);
        assert_eq!(err, "REFUSED:EVIDENCE_CONTRADICTION");
        // The false negative claim is durable evidence about the claimant.
        assert_eq!(
            ledger.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::ContradictionRefused)
        );
    }

    // Ordering determinism: for a fixed sequence the outcome is identical
    // regardless of intra-batch order, and a refused positive never supersedes.
    #[test]
    fn refused_positive_never_supersedes_and_other_keys_untouched() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let store = trust();
        let ctx = issue_ctx();
        let mut ledger = NegativeEvidenceLedger::default();
        ledger.admit_negation(negation(), &[]).expect("admits");

        // Tamper the signature so the positive refuses.
        let mut bundle = positive_bundle(SUBJECT, PREDICATE, &ctx, &signing);
        bundle.receipt.signature = "bm90LXJlYWw".to_string();
        let receipt_store: HashMap<String, ReceiptV2> = HashMap::new();
        let admission =
            admit_positive_and_supersede(&bundle, &mut ledger, &store, &receipt_store);
        assert_eq!(admission.standing, Standing::Refused);
        assert_eq!(
            ledger.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::Absent(Standing::Unknown)),
            "a refused positive must not supersede a negation"
        );

        // A different predicate's negation is untouched by supersession.
        let other = NegationRecord {
            predicate: "other.predicate".to_string(),
            subject: SUBJECT.to_string(),
            searched_at: AT.to_string(),
            searched_by: "castle-lane".to_string(),
        };
        ledger.admit_negation(other, &[]).expect("admits");
        assert_eq!(
            ledger.status_of(SUBJECT, "other.predicate"),
            Some(NegativeEvidenceStatus::Absent(Standing::Unknown))
        );
    }

    // Incomplete negation refuses at ingest.
    #[test]
    fn incomplete_negation_is_typed_refusal() {
        let mut ledger = NegativeEvidenceLedger::default();
        let bad = NegationRecord {
            predicate: PREDICATE.to_string(),
            subject: SUBJECT.to_string(),
            searched_at: AT.to_string(),
            searched_by: String::new(),
        };
        assert_eq!(
            ledger.admit_negation(bad, &[]),
            Err(REASON_INCOMPLETE.to_string())
        );
    }
}
