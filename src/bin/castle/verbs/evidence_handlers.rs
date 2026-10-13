//! Read-only durable-evidence verification handlers.
//!
//! This module deliberately contains no actuation path. It recomputes the identity of
//! an already committed CASTLE evidence file and returns its verified record for replay
//! and crash-recovery consumers such as XaaS.

use castle::fortune5::MetricObservation;
use castle::negative_evidence::{
    rfc3339_now, NegativeEvidenceLedger, NegativeEvidenceStatus, NegationRecord,
    REASON_CONTRADICTION, REASON_INCOMPLETE, REASON_NOT_A_REFUSAL,
};
use castle::v26_8_18::verify_evidence_file;
use clap_noun_verb::NounVerbError;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError { message: message.into() }
}

pub fn evidence_verify_handler(evidence_path: String) -> Result<Value> {
    let verification = verify_evidence_file(&evidence_path).map_err(exec_err)?;
    serde_json::to_value(verification)
        .map_err(|error| exec_err(format!("failed to serialize evidence verification: {error}")))
}

/// Load the durable negation ledger at `ledger_path` (empty when absent).
fn load_ledger(ledger_path: &str) -> Result<NegativeEvidenceLedger> {
    match std::fs::read_to_string(ledger_path) {
        Ok(bytes) => {
            NegativeEvidenceLedger::ledger_from_json(&bytes).map_err(|reason| exec_err(reason))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(NegativeEvidenceLedger::default())
        }
        Err(error) => Err(exec_err(format!(
            "REFUSED:NEGATION_LEDGER_UNREADABLE:{error}"
        ))),
    }
}

/// Persist the ledger to `ledger_path`.
fn save_ledger(ledger: &NegativeEvidenceLedger, ledger_path: &str) -> Result<()> {
    let bytes = ledger.ledger_to_json().map_err(exec_err)?;
    std::fs::write(ledger_path, bytes)
        .map_err(|error| exec_err(format!("REFUSED:NEGATION_LEDGER_UNWRITABLE:{error}")))
}

fn status_to_value(status: &NegativeEvidenceStatus) -> Value {
    match status {
        NegativeEvidenceStatus::Absent(standing) => {
            json!({"type": "absent", "standing": standing.as_str()})
        }
        NegativeEvidenceStatus::Superseded { by } => {
            json!({"type": "superseded", "by": by})
        }
        NegativeEvidenceStatus::ContradictionRefused => {
            json!({"type": "contradiction_refused"})
        }
    }
}

/// `evidence negation-ingest` — admit one negative finding into the durable
/// ledger. Either explicit `(subject, predicate)` or a typed witness refusal
/// (`--witness-json` + `--refusal-reason`) supplies the record; `searched_at`
/// defaults to now (RFC 3339 UTC). The CLI admits against no admitted
/// positive set (contradiction gating stays library-side); the typed refusal
/// surfaces as a nonzero exit with `REFUSED:*` in stderr, and a retained
/// contradiction record is still persisted.
pub fn negative_ingest_handler(
    ledger_path: String,
    subject: Option<String>,
    predicate: Option<String>,
    searched_by: String,
    searched_at: Option<String>,
    refusal_reason: Option<String>,
    witness_json: Option<String>,
) -> Result<Value> {
    // The CLI surface carries no admitted positive observations; contradiction
    // detection through this verb is exercised at the library court layer.
    let admitted: Vec<MetricObservation> = Vec::new();

    let record = match (&subject, &predicate, &witness_json, &refusal_reason) {
        (Some(subject), Some(predicate), _, _) => NegationRecord {
            predicate: predicate.clone(),
            subject: subject.clone(),
            searched_at: searched_at.clone().unwrap_or_else(rfc3339_now),
            searched_by: searched_by.clone(),
        },
        (None, None, Some(witness_json), Some(reason)) => {
            castle::negative_evidence::negation_from_witness_refusal(
                witness_json,
                reason,
                &searched_at.unwrap_or_else(rfc3339_now),
                &searched_by,
            )
            .map_err(exec_err)?
        }
        _ => {
            return Err(exec_err(
                "REFUSED:NEGATION_INCOMPLETE: provide --subject and --predicate, \
                 or --witness-json with --refusal-reason",
            ))
        }
    };

    let mut ledger = load_ledger(&ledger_path)?;
    let record_subject = record.subject.clone();
    let record_predicate = record.predicate.clone();
    let record_searched_at = record.searched_at.clone();
    let admission = ledger.admit_negation(record, &admitted);
    // Persist whatever the ledger now holds (a refused contradiction record is
    // retained durable evidence about the claimant).
    save_ledger(&ledger, &ledger_path)?;

    match admission {
        Ok(status) => {
            let status_value = status_to_value(&status);
            Ok(json!({
                "admitted": true,
                "subject": record_subject,
                "predicate": record_predicate,
                "searched_at": record_searched_at,
                "status": status_value,
                "ledger_path": ledger_path,
            }))
        }
        Err(reason) if reason == REASON_CONTRADICTION => Err(exec_err(format!(
            "{REASON_CONTRADICTION}: an admitted positive observation for this key \
             already exists; the negation record was retained in the ledger"
        ))),
        Err(reason)
            if reason == REASON_INCOMPLETE || reason == REASON_NOT_A_REFUSAL =>
        {
            Err(exec_err(reason))
        }
        Err(reason) => Err(exec_err(reason)),
    }
}

/// `evidence negation-status` — query the durable negation ledger. With
/// `(subject, predicate)` the latest status for that key; otherwise a summary
/// of every stored record.
pub fn negative_status_handler(
    ledger_path: String,
    subject: Option<String>,
    predicate: Option<String>,
) -> Result<Value> {
    let ledger = load_ledger(&ledger_path)?;
    match (&subject, &predicate) {
        (Some(subject), Some(predicate)) => {
            let status = ledger
                .status_of(subject, predicate)
                .map(|status| status_to_value(&status))
                .unwrap_or(json!({"type": "never_looked"}));
            Ok(json!({
                "subject": subject,
                "predicate": predicate,
                "status": status,
            }))
        }
        (None, None) => {
            let records: Vec<Value> = ledger
                .records()
                .iter()
                .map(|(record, status)| {
                    json!({
                        "subject": record.subject,
                        "predicate": record.predicate,
                        "searched_at": record.searched_at,
                        "searched_by": record.searched_by,
                        "status": status_to_value(status),
                    })
                })
                .collect();
            Ok(json!({"record_count": records.len(), "records": records}))
        }
        _ => Err(exec_err(
            "REFUSED:NEGATION_INCOMPLETE: provide both --subject and --predicate, or neither",
        )),
    }
}
