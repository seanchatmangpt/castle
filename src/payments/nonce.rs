use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::sa2a_security::encoding::sha256_tagged;
use crate::sa2a_security::SecurityRefusal;

use super::refusal::{self, PayResult};

/// Durable replacement for the in-memory `NonceFence`: a claim is an fsynced
/// `create_new` file keyed by hash(principal, nonce), so replay is refused
/// across process restarts and racing threads.
#[derive(Debug, Clone)]
pub struct DurableNonceFence {
    root: PathBuf,
}

impl DurableNonceFence {
    pub fn open(root: impl Into<PathBuf>) -> PayResult<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
        Ok(Self { root })
    }

    pub fn claim(&self, principal: &str, nonce: &str) -> PayResult<()> {
        if principal.is_empty() || nonce.is_empty() {
            return Err(SecurityRefusal::NonceReplay.to_string());
        }
        let mut body = Vec::new();
        body.extend_from_slice(principal.as_bytes());
        body.push(0);
        body.extend_from_slice(nonce.as_bytes());
        let name = sha256_tagged(b"CASTLE-PAYMENT-NONCE-V1", &body);
        let path = self.root.join(format!("{}.nonce", &name[7..]));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(nonce.as_bytes())
                    .and_then(|()| file.sync_all())
                    .map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
                super::dirlock::sync_dir(&self.root)?;
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(SecurityRefusal::NonceReplay.to_string())
            }
            Err(e) => Err(format!("{}:{e}", refusal::CLAIM_STORE_FAILED)),
        }
    }
}
