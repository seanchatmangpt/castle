use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{dual_artifact_identity, ArtifactIdentity, ReleaseStanding};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableEvidenceRecord {
    pub cell_id: String,
    pub subject: String,
    pub construct_digest: String,
    pub ocel_receipt_digest: String,
    pub brce_prepare_receipt_digests: Vec<String>,
    pub brce_outcome_receipt_digests: Vec<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceCommit {
    pub standing: ReleaseStanding,
    pub record_identity: ArtifactIdentity,
    pub path: String,
    pub reason: String,
}

fn safe_digest(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn target_path(root: &Path, digest: &str) -> PathBuf {
    root.join(format!("{digest}.json"))
}

/// Persist one post-BRCE/OCEL evidence record using content-addressed create-new
/// semantics. An exact replay is idempotently ALIVE; an existing path with
/// different bytes is a typed collision refusal. File and directory metadata
/// are synced before ALIVE is returned.
pub fn persist_evidence(root: impl AsRef<Path>, record: &DurableEvidenceRecord) -> Result<EvidenceCommit, String> {
    if record.cell_id.is_empty() || record.subject.is_empty()
        || !safe_digest(&record.construct_digest) || !safe_digest(&record.ocel_receipt_digest)
        || record.brce_prepare_receipt_digests.iter().any(|d| !safe_digest(d))
        || record.brce_outcome_receipt_digests.iter().any(|d| !safe_digest(d))
    {
        return Err("REFUSED:INVALID_DURABLE_EVIDENCE_RECORD".to_string());
    }
    if record.brce_prepare_receipt_digests.len() != record.event_count
        || record.brce_outcome_receipt_digests.len() != record.event_count
    {
        return Err("BLOCKED:INCOMPLETE_DURABLE_BRCE_EVIDENCE".to_string());
    }

    let bytes = serde_json::to_vec(record).map_err(|e| format!("BLOCKED:EVIDENCE_SERIALIZATION_FAILED:{e}"))?;
    let identity = dual_artifact_identity(&bytes);
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|e| format!("BLOCKED:EVIDENCE_DIRECTORY_FAILED:{e}"))?;
    let path = target_path(root, &identity.blake3_256);

    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(&bytes).map_err(|e| format!("BLOCKED:EVIDENCE_WRITE_FAILED:{e}"))?;
            file.sync_all().map_err(|e| format!("BLOCKED:EVIDENCE_FILE_SYNC_FAILED:{e}"))?;
            if let Ok(dir) = OpenOptions::new().read(true).open(root) {
                dir.sync_all().map_err(|e| format!("BLOCKED:EVIDENCE_DIRECTORY_SYNC_FAILED:{e}"))?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut existing = Vec::new();
            OpenOptions::new().read(true).open(&path)
                .and_then(|mut file| file.read_to_end(&mut existing))
                .map_err(|e| format!("BLOCKED:EVIDENCE_REPLAY_READ_FAILED:{e}"))?;
            if existing != bytes {
                return Err("REFUSED:EVIDENCE_CONTENT_ADDRESS_COLLISION".to_string());
            }
        }
        Err(error) => return Err(format!("BLOCKED:EVIDENCE_CREATE_FAILED:{error}")),
    }

    Ok(EvidenceCommit {
        standing: ReleaseStanding::Alive,
        record_identity: identity,
        path: path.to_string_lossy().into_owned(),
        reason: "ALIVE:DURABLE_EVIDENCE_COMMITTED".to_string(),
    })
}
