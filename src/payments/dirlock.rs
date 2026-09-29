use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::refusal::{self, PayResult};

const WAIT_STEP: Duration = Duration::from_millis(2);
const WAIT_MAX: Duration = Duration::from_secs(15);
const STALE_AFTER: Duration = Duration::from_secs(30);

/// Cross-process, cross-handle exclusive lock over a store directory: an
/// fsync-free `create_new` lock file, removed on drop. A lock file older than
/// `STALE_AFTER` (holder crashed) is broken. Guards read-check-write sequences
/// (budget sums, balance checks, sequence numbers) that must not interleave.
#[derive(Debug)]
pub struct DirLock {
    path: PathBuf,
}

impl DirLock {
    pub fn acquire(root: &Path) -> PayResult<Self> {
        let path = root.join(".lock");
        let mut waited = Duration::ZERO;
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut f) => {
                    let _ = writeln!(f, "{}", std::process::id());
                    return Ok(Self { path });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| SystemTime::now().duration_since(t).ok())
                        .is_some_and(|age| age > STALE_AFTER);
                    if stale {
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                    if waited >= WAIT_MAX {
                        return Err(format!("{}:LOCK_TIMEOUT", refusal::CLAIM_STORE_FAILED));
                    }
                    std::thread::sleep(WAIT_STEP);
                    waited += WAIT_STEP;
                }
                Err(e) => return Err(format!("{}:{e}", refusal::CLAIM_STORE_FAILED)),
            }
        }
    }
}

impl Drop for DirLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Publish `bytes` at `final_path` atomically with create-new semantics: the
/// file becomes visible only once complete and durable, and an existing file is
/// never replaced (returns `Ok(false)` if it already exists).
pub fn publish_new(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<bool> {
    let io = |e: std::io::Error| format!("{}:{e}", refusal::CLAIM_STORE_FAILED);
    let name = final_path.file_name().and_then(|n| n.to_str()).unwrap_or("x");
    let tmp = root.join(format!(".{name}.{}.tmp", std::process::id()));
    {
        let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(&tmp).map_err(io)?;
        f.write_all(bytes).map_err(io)?;
        f.sync_all().map_err(io)?;
    }
    let linked = match fs::hard_link(&tmp, final_path) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => false,
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            return Err(io(e));
        }
    };
    let _ = fs::remove_file(&tmp);
    if linked {
        sync_dir(root)?;
    }
    Ok(linked)
}

/// Atomically replace `final_path` (write tmp, fsync, rename, fsync dir).
pub fn publish_replace(root: &Path, final_path: &Path, bytes: &[u8]) -> PayResult<()> {
    let io = |e: std::io::Error| format!("{}:{e}", refusal::CLAIM_STORE_FAILED);
    let name = final_path.file_name().and_then(|n| n.to_str()).unwrap_or("x");
    let tmp = root.join(format!(".{name}.{}.tmp", std::process::id()));
    {
        let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(&tmp).map_err(io)?;
        f.write_all(bytes).map_err(io)?;
        f.sync_all().map_err(io)?;
    }
    fs::rename(&tmp, final_path).map_err(io)?;
    sync_dir(root)
}

pub fn sync_dir(root: &Path) -> PayResult<()> {
    let dir = OpenOptions::new().read(true).open(root).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
    dir.sync_all().map_err(|e| format!("{}:DIR_FSYNC:{e}", refusal::CLAIM_STORE_FAILED))
}
