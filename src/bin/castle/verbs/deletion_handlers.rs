//! Handlers for the `deletion` noun — first-party 45-day CPRA/SB 923 tracker.
//! Thin CLI seam only; all statutory logic lives in `castle::deletion`.

use std::fs;

use castle::deletion::{DeletionCase, DeletionIntake, CITATION_DELETE, CITATION_RESPONSE_CLOCK,
    CITATION_SB923, EXTENDED_RESPONSE_DAYS, PRIMARY_RESPONSE_DAYS, SB923_EFFECTIVE};
use clap_noun_verb::NounVerbError;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError {
        message: message.into(),
    }
}

/// `deletion info` — statutory constants and citations (no I/O).
pub fn deletion_info_handler() -> Result<Value> {
    Ok(json!({
        "primary_response_days": PRIMARY_RESPONSE_DAYS,
        "extended_response_days": EXTENDED_RESPONSE_DAYS,
        "sb923_effective": SB923_EFFECTIVE.iso(),
        "citations": {
            "delete": CITATION_DELETE,
            "response_clock": CITATION_RESPONSE_CLOCK,
            "sb923": CITATION_SB923,
        }
    }))
}

/// `deletion open` — open a case from an intake JSON file; emit sealed case JSON.
pub fn deletion_open_handler(intake_path: String) -> Result<Value> {
    let raw = fs::read_to_string(&intake_path)
        .map_err(|e| exec_err(format!("REFUSED:DELETION_INTAKE_UNREADABLE:{e}")))?;
    let intake: DeletionIntake = serde_json::from_str(&raw)
        .map_err(|e| exec_err(format!("REFUSED:DELETION_INTAKE_INVALID:{e}")))?;
    let case = DeletionCase::open(&intake).map_err(|e| exec_err(e.to_string()))?;
    serde_json::to_value(case)
        .map_err(|e| exec_err(format!("REFUSED:DELETION_CASE_SERIALIZE:{e}")))
}
