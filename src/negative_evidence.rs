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
/// Guard refusal: something that is not a typed refusal was offered as one.
pub const REASON_NOT_A_REFUSAL: &str = "REFUSED:NOT_A_TYPED_REFUSAL";
/// Guard refusal: a stored ledger file could not be parsed.
pub const REASON_LEDGER_MALFORMED: &str = "REFUSED:NEGATION_LEDGER_MALFORMED";

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

/// A witness adapter's typed refusal mapped into the negative-evidence
/// channel: a refusal that a required witness for a theorem could not be
/// produced is itself negative evidence about that theorem.
///
/// Mapping source: the typed refusal constants already emitted by the witness
/// adapters (`lean_witness::REFUSED:LEAN_*`, `insa_witness` refusals). The
/// predicate carries the refusal reason identity verbatim; the subject is the
/// theorem the witness line names (`mfact@<sha>:<theorem>` shape when the
/// line parsed) or a fallback subject when the line was malformed.
pub fn negation_from_witness_refusal(
    witness_line: &str,
    refusal_reason: &str,
    searched_at: &str,
    searched_by: &str,
) -> Result<NegationRecord, String> {
    if !refusal_reason.starts_with("REFUSED:") {
        return Err(crate::negative_evidence::REASON_NOT_A_REFUSAL.to_string());
    }
    let subject = serde_json::from_str::<serde_json::Value>(witness_line)
        .ok()
        .and_then(|value| {
            value
                .get("theorem")
                .and_then(|t| t.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "witness:unparsed-line".to_string());
    Ok(NegationRecord {
        predicate: refusal_reason.to_string(),
        subject,
        searched_at: searched_at.to_string(),
        searched_by: searched_by.to_string(),
    })
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

/// Serializable twin of [`NegativeEvidenceStatus`] for durable ledger files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StoredStatus {
    /// `NegativeEvidenceStatus::Absent(_)` — the inner Standing is carried as
    /// its stable string identity (`ALIVE`/`REFUSED`/`UNKNOWN`).
    Absent { standing: String },
    /// `NegativeEvidenceStatus::Superseded { by }`.
    Superseded { by: String },
    /// `NegativeEvidenceStatus::ContradictionRefused`.
    ContradictionRefused,
}

impl From<&NegativeEvidenceStatus> for StoredStatus {
    fn from(status: &NegativeEvidenceStatus) -> Self {
        match status {
            NegativeEvidenceStatus::Absent(standing) => StoredStatus::Absent {
                standing: standing.as_str().to_string(),
            },
            NegativeEvidenceStatus::Superseded { by } => StoredStatus::Superseded { by: by.clone() },
            NegativeEvidenceStatus::ContradictionRefused => StoredStatus::ContradictionRefused,
        }
    }
}

impl From<&StoredStatus> for NegativeEvidenceStatus {
    fn from(stored: &StoredStatus) -> Self {
        match stored {
            StoredStatus::Absent { standing } => {
                let standing = match standing.as_str() {
                    "ALIVE" => Standing::Alive,
                    "REFUSED" => Standing::Refused,
                    _ => Standing::Unknown,
                };
                NegativeEvidenceStatus::Absent(standing)
            }
            StoredStatus::Superseded { by } => NegativeEvidenceStatus::Superseded { by: by.clone() },
            StoredStatus::ContradictionRefused => NegativeEvidenceStatus::ContradictionRefused,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredEntry {
    record: NegationRecord,
    status: StoredStatus,
}

#[derive(Serialize, Deserialize)]
struct LedgerFile {
    records: Vec<StoredEntry>,
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

    /// Rebuild a ledger from a previously stored record/status sequence.
    #[must_use]
    pub fn with_records(records: Vec<(NegationRecord, NegativeEvidenceStatus)>) -> Self {
        Self { records }
    }

    /// Borrow the admitted record/status sequence (ledger file projection).
    #[must_use]
    pub fn records(&self) -> &[(NegationRecord, NegativeEvidenceStatus)] {
        &self.records
    }

    /// Project the ledger to its durable JSON form (the `negation-ingest` /
    /// `negation-status` CLI verbs persist and reload through this).
    ///
    /// # Errors
    /// Serialization failure (never expected for owned data).
    pub fn ledger_to_json(&self) -> Result<String, String> {
        let file = LedgerFile {
            records: self
                .records
                .iter()
                .map(|(record, status)| StoredEntry {
                    record: record.clone(),
                    status: StoredStatus::from(status),
                })
                .collect(),
        };
        serde_json::to_string(&file).map_err(|error| format!("{REASON_LEDGER_MALFORMED}:{error}"))
    }

    /// Reload a ledger from [`Self::ledger_to_json`] output.
    ///
    /// # Errors
    /// `REFUSED:NEGATION_LEDGER_MALFORMED` — unparsable ledger file bytes.
    pub fn ledger_from_json(bytes: &str) -> Result<Self, String> {
        let file: LedgerFile = serde_json::from_str(bytes)
            .map_err(|error| format!("{REASON_LEDGER_MALFORMED}:{error}"))?;
        Ok(Self::with_records(
            file.records
                .into_iter()
                .map(|entry| (entry.record, NegativeEvidenceStatus::from(&entry.status)))
                .collect(),
        ))
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

/// Current UTC time as RFC 3339 (`YYYY-MM-DDTHH:MM:SSZ`), computed from the
/// system clock with no external time dependency. Leap seconds truncate.
#[must_use]
pub fn rfc3339_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    rfc3339_from_epoch_secs(secs)
}

/// Convert Unix epoch seconds to RFC 3339 UTC (`YYYY-MM-DDTHH:MM:SSZ`) via
/// the civil-from-days algorithm (Hinnant). Pure so it is court-testable.
#[must_use]
pub fn rfc3339_from_epoch_secs(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        sod / 3_600,
        (sod % 3_600) / 60,
        sod % 3_600 % 60
    )
}

/// Howard Hinnant's `civil_from_days`: days since 1970-01-01 → (y, m, d).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (year + i64::from(month <= 2), month as u32, day as u32)
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

    // ---- refusal→negation mapping + durable store + clock ----

    fn sorry_ax_line() -> String {
        r#"{"schema":"lean-witness/v1","theorem":"chatman_equation_typed","axioms":["propext","sorryAx"],"source_sha256":"a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8","toolchain":"leanprover/lean4:v4.21.0"}"#
            .to_string()
    }

    #[test]
    fn typed_witness_refusal_maps_to_negation_record() {
        let record = negation_from_witness_refusal(
            &sorry_ax_line(),
            "REFUSED:LEAN_AXIOMS_NONSTANDARD",
            AT,
            "castle-lane",
        )
        .expect("maps");
        assert_eq!(record.predicate, "REFUSED:LEAN_AXIOMS_NONSTANDARD");
        assert_eq!(record.subject, "chatman_equation_typed");
        assert_eq!(record.searched_at, AT);
        assert_eq!(record.searched_by, "castle-lane");
    }

    #[test]
    fn non_refusal_reason_is_typed_refused() {
        assert_eq!(
            negation_from_witness_refusal(&sorry_ax_line(), "ALIVE:LOOKED", AT, "lane"),
            Err(REASON_NOT_A_REFUSAL.to_string())
        );
    }

    #[test]
    fn malformed_witness_line_still_maps_to_fallback_subject() {
        let record = negation_from_witness_refusal(
            "not json at all",
            "REFUSED:LEAN_WITNESS_MALFORMED_JSON",
            AT,
            "lane",
        )
        .expect("maps");
        assert_eq!(record.predicate, "REFUSED:LEAN_WITNESS_MALFORMED_JSON");
        assert_eq!(record.subject, "witness:unparsed-line");
    }

    #[test]
    fn ledger_json_round_trip_preserves_records_and_statuses() {
        let mut ledger = NegativeEvidenceLedger::default();
        ledger.admit_negation(negation(), &[]).expect("admits");
        // Retained contradiction record too.
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let ctx = issue_ctx();
        let bundle = positive_bundle(SUBJECT, PREDICATE, &ctx, &signing);
        let admitted = vec![bundle.observation.clone()];
        let other = NegationRecord {
            predicate: PREDICATE.to_string(),
            subject: SUBJECT.to_string(),
            searched_at: AT.to_string(),
            searched_by: "lane".to_string(),
        };
        let _ = ledger.admit_negation(other, &admitted);

        let json = ledger.ledger_to_json().expect("serializes");
        let reloaded = NegativeEvidenceLedger::ledger_from_json(&json).expect("reloads");
        assert_eq!(reloaded.records().len(), 2);
        assert_eq!(
            reloaded.status_of(SUBJECT, PREDICATE),
            Some(NegativeEvidenceStatus::ContradictionRefused)
        );
    }

    #[test]
    fn malformed_ledger_bytes_are_typed_refused() {
        assert!(NegativeEvidenceLedger::ledger_from_json("{not json")
            .err()
            .map_or(false, |e| e.starts_with(REASON_LEDGER_MALFORMED)));
    }

    #[test]
    fn rfc3339_known_epochs_render_exactly() {
        assert_eq!(rfc3339_from_epoch_secs(0), "1970-01-01T00:00:00Z");
        assert_eq!(
            rfc3339_from_epoch_secs(1_000_000_000),
            "2001-09-09T01:46:40Z"
        );
        // Leap day.
        assert_eq!(rfc3339_from_epoch_secs(1_709_164_800), "2024-02-29T00:00:00Z");
        // End of day.
        assert_eq!(rfc3339_from_epoch_secs(86_399), "1970-01-01T23:59:59Z");
        // now() is well-formed RFC 3339 UTC.
        let now = rfc3339_now();
        assert_eq!(now.len(), 20);
        assert!(now.ends_with('Z'));
        assert_eq!(&now[4..5], "-");
        assert_eq!(&now[10..11], "T");
    }
}
