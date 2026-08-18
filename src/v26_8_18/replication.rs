use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ReleaseStanding;

fn digest_ok(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptCheckpoint {
    pub cell_id: String,
    pub sequence: u64,
    pub head_digest: String,
    pub constitution_id: String,
    pub observed_at_epoch_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicaState {
    pub receiver_id: String,
    pub checkpoints: BTreeMap<String, ReceiptCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationAdmission {
    pub standing: ReleaseStanding,
    pub reason: String,
    pub cell_id: String,
    pub sequence: u64,
}

/// Cross-region replication is monotonic per cell. An old sequence is a
/// rollback refusal; the same sequence with another digest is equivocation.
/// Replaying the same checkpoint is idempotently ALIVE.
pub fn admit_receipt_checkpoint(state: &mut ReplicaState, checkpoint: ReceiptCheckpoint) -> ReplicationAdmission {
    let (standing, reason) = if state.receiver_id.is_empty() || checkpoint.cell_id.is_empty() || checkpoint.constitution_id.is_empty() {
        (ReleaseStanding::Refused, "REFUSED:INCOMPLETE_RECEIPT_CHECKPOINT")
    } else if !digest_ok(&checkpoint.head_digest) {
        (ReleaseStanding::Refused, "REFUSED:INVALID_RECEIPT_HEAD_DIGEST")
    } else if let Some(current) = state.checkpoints.get(&checkpoint.cell_id) {
        if checkpoint.sequence < current.sequence {
            (ReleaseStanding::Refused, "REFUSED:REPLICA_ROLLBACK")
        } else if checkpoint.sequence == current.sequence && checkpoint.head_digest != current.head_digest {
            (ReleaseStanding::Refused, "REFUSED:REPLICA_EQUIVOCATION")
        } else {
            (ReleaseStanding::Alive, "ALIVE:REPLICA_CHECKPOINT_ADMITTED")
        }
    } else {
        (ReleaseStanding::Alive, "ALIVE:REPLICA_CHECKPOINT_ADMITTED")
    };

    if standing == ReleaseStanding::Alive {
        state.checkpoints.insert(checkpoint.cell_id.clone(), checkpoint.clone());
    }
    ReplicationAdmission {
        standing,
        reason: reason.to_string(),
        cell_id: checkpoint.cell_id,
        sequence: checkpoint.sequence,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationEvidence {
    pub transition_id: String,
    pub prepare_receipt_digest: String,
    pub outcome_receipt_digest: Option<String>,
    /// `None` means the provider consequence cannot currently be established.
    pub provider_observed: Option<bool>,
    pub retry_is_proven_idempotent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationDecision {
    pub standing: ReleaseStanding,
    pub reason: String,
    pub replay_allowed: bool,
}

/// A network loss after provider acceptance is never guessed away. Missing
/// consequence evidence is UNKNOWN and blocks replay unless a later outcome
/// receipt closes the transition. Even a declared idempotent operation is not
/// auto-replayed while its consequence remains unknown.
#[must_use]
pub fn reconcile_transition(evidence: &ReconciliationEvidence) -> ReconciliationDecision {
    if evidence.transition_id.is_empty() || !digest_ok(&evidence.prepare_receipt_digest) {
        return ReconciliationDecision {
            standing: ReleaseStanding::Refused,
            reason: "REFUSED:INVALID_RECONCILIATION_INPUT".to_string(),
            replay_allowed: false,
        };
    }
    if let Some(outcome) = &evidence.outcome_receipt_digest {
        if !digest_ok(outcome) {
            return ReconciliationDecision {
                standing: ReleaseStanding::Refused,
                reason: "REFUSED:INVALID_OUTCOME_RECEIPT_DIGEST".to_string(),
                replay_allowed: false,
            };
        }
        return ReconciliationDecision {
            standing: ReleaseStanding::Alive,
            reason: "ALIVE:OUTCOME_RECEIPT_CONFIRMED".to_string(),
            replay_allowed: evidence.retry_is_proven_idempotent,
        };
    }

    match evidence.provider_observed {
        Some(true) => ReconciliationDecision {
            standing: ReleaseStanding::Blocked,
            reason: "BLOCKED:CONSEQUENCE_OBSERVED_WITHOUT_OUTCOME_RECEIPT".to_string(),
            replay_allowed: false,
        },
        Some(false) => ReconciliationDecision {
            standing: ReleaseStanding::PartialAlive,
            reason: "PARTIAL_ALIVE:PREPARED_WITH_NO_OBSERVED_CONSEQUENCE".to_string(),
            replay_allowed: evidence.retry_is_proven_idempotent,
        },
        None => ReconciliationDecision {
            standing: ReleaseStanding::Unknown,
            reason: "UNKNOWN:CONSEQUENCE_UNCONFIRMED".to_string(),
            replay_allowed: false,
        },
    }
}
