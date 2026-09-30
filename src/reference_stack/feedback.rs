use crate::castle::ReceiptedOcelLog;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beam4PmFeedback {
    pub subject: String,
    pub replay_identity: String,
    pub construct_digest: String,
    pub outcome_receipt_digest: String,
    pub ocel_event_count: usize,
}

pub fn project_beam4pm_feedback(
    log: &ReceiptedOcelLog,
    subject: &str,
    replay_identity: &str,
) -> Result<Beam4PmFeedback, String> {
    if subject.is_empty() || replay_identity.is_empty() {
        return Err("REFUSED:FEEDBACK_IDENTITY_REQUIRED".to_string());
    }
    if log.receipt.subject != subject {
        return Err("REFUSED:POSTCONDITION_SUBJECT_MISMATCH".to_string());
    }
    if log.log.events.is_empty() {
        return Err("REFUSED:POSTCONDITION_NOT_OBSERVED".to_string());
    }

    Ok(Beam4PmFeedback {
        subject: subject.to_string(),
        replay_identity: replay_identity.to_string(),
        construct_digest: log.construct_digest.clone(),
        outcome_receipt_digest: log.receipt.receipt_digest.clone(),
        ocel_event_count: log.log.events.len(),
    })
}
