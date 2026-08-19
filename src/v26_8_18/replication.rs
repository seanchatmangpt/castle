use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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
struct DurableReplicaEnvelope {
    state: ReplicaState,
    state_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableReplicaCommit {
    pub standing: ReleaseStanding,
    pub reason: String,
    pub path: String,
    pub state_blake3: String,
    pub admission: ReplicationAdmission,
}

fn replica_state_digest(state: &ReplicaState) -> Result<String, String> {
    serde_json::to_vec(state)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|error| format!("REFUSED:REPLICA_STATE_SERIALIZATION:{error}"))
}

fn durable_envelope(state: ReplicaState) -> Result<DurableReplicaEnvelope, String> {
    let state_blake3 = replica_state_digest(&state)?;
    Ok(DurableReplicaEnvelope { state, state_blake3 })
}

pub fn load_durable_replica(path: impl AsRef<Path>) -> Result<ReplicaState, String> {
    let path = path.as_ref();
    if !path.exists() {
        return Err("UNKNOWN:REPLICA_STATE_NOT_FOUND".to_string());
    }
    let bytes = fs::read(path).map_err(|error| format!("BLOCKED:REPLICA_STATE_READ:{error}"))?;
    let envelope: DurableReplicaEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| format!("REFUSED:REPLICA_STATE_INVALID_JSON:{error}"))?;
    let actual = replica_state_digest(&envelope.state)?;
    if actual != envelope.state_blake3 || !digest_ok(&envelope.state_blake3) {
        return Err("REFUSED:REPLICA_STATE_DIGEST_MISMATCH".to_string());
    }
    Ok(envelope.state)
}

fn temp_path(path: &Path, state_blake3: &str) -> PathBuf {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("replica-state");
    let temp_name = format!(".{file_name}.{}.{}.tmp", std::process::id(), &state_blake3[..12]);
    path.parent().unwrap_or_else(|| Path::new(".")).join(temp_name)
}

fn atomic_persist_replica(path: &Path, state: ReplicaState) -> Result<String, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("BLOCKED:REPLICA_STATE_DIRECTORY:{error}"))?;
    }
    let envelope = durable_envelope(state)?;
    let bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| format!("REFUSED:REPLICA_ENVELOPE_SERIALIZATION:{error}"))?;
    let temp = temp_path(path, &envelope.state_blake3);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp)
        .map_err(|error| format!("BLOCKED:REPLICA_TEMP_OPEN:{error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("BLOCKED:REPLICA_TEMP_WRITE:{error}"))?;
    file.write_all(b"\n")
        .map_err(|error| format!("BLOCKED:REPLICA_TEMP_WRITE:{error}"))?;
    file.sync_all()
        .map_err(|error| format!("BLOCKED:REPLICA_TEMP_SYNC:{error}"))?;
    drop(file);
    fs::rename(&temp, path).map_err(|error| {
        let _ = fs::remove_file(&temp);
        format!("BLOCKED:REPLICA_ATOMIC_RENAME:{error}")
    })?;
    Ok(envelope.state_blake3)
}

/// Admit and durably commit one checkpoint. Existing state is verified before
/// use, so rollback/equivocation protection survives process restart. A refused
/// checkpoint never mutates the persisted ledger.
pub fn persist_receipt_checkpoint(
    path: impl AsRef<Path>,
    receiver_id: &str,
    checkpoint: ReceiptCheckpoint,
) -> Result<DurableReplicaCommit, String> {
    if receiver_id.is_empty() {
        return Err("REFUSED:EMPTY_REPLICA_RECEIVER".to_string());
    }
    let path = path.as_ref();
    let mut state = if path.exists() {
        load_durable_replica(path)?
    } else {
        ReplicaState { receiver_id: receiver_id.to_string(), checkpoints: BTreeMap::new() }
    };
    if state.receiver_id != receiver_id {
        return Err("REFUSED:REPLICA_RECEIVER_ID_MISMATCH".to_string());
    }
    let before = state.clone();
    let admission = admit_receipt_checkpoint(&mut state, checkpoint);
    if admission.standing != ReleaseStanding::Alive {
        return Ok(DurableReplicaCommit {
            standing: admission.standing,
            reason: admission.reason.clone(),
            path: path.display().to_string(),
            state_blake3: replica_state_digest(&before)?,
            admission,
        });
    }
    let state_blake3 = atomic_persist_replica(path, state)?;
    Ok(DurableReplicaCommit {
        standing: ReleaseStanding::Alive,
        reason: "ALIVE:DURABLE_REPLICA_COMMITTED".to_string(),
        path: path.display().to_string(),
        state_blake3,
        admission,
    })
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
