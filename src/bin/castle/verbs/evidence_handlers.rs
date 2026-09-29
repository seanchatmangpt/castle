//! Read-only durable-evidence verification handlers.
//!
//! This module deliberately contains no actuation path. It recomputes the identity of
//! an already committed CASTLE evidence file and returns its verified record for replay
//! and crash-recovery consumers such as XaaS.

use castle::v26_8_18::verify_evidence_file;
use clap_noun_verb::NounVerbError;
use serde_json::Value;

type Result<T> = std::result::Result<T, NounVerbError>;

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError { message: message.into() }
}

pub fn evidence_verify_handler(evidence_path: String) -> Result<Value> {
    let verification = verify_evidence_file(&evidence_path).map_err(exec_err)?;
    serde_json::to_value(verification)
        .map_err(|error| exec_err(format!("failed to serialize evidence verification: {error}")))
}
