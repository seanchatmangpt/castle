use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::money::Currency;
use super::refusal::{self, refuse, PayResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimState {
    /// Admitted and budget-reserved; DO not yet observed.
    Reserved,
    /// Settlement observed in the ledger.
    Executed,
    /// Definitely refused / proven absent; budget released, re-admission allowed.
    Refused,
    /// DO ambiguous. Never retried blindly; only `reconcile` resolves it.
    UnknownOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub effect_digest: String,
    pub principal: String,
    pub payer: String,
    pub payee: String,
    pub amount_minor: u64,
    pub currency: Currency,
    pub generation: u64,
    pub state: ClaimState,
    pub reverses: Option<String>,
    pub construct_digest: Option<String>,
    pub detail: String,
    /// Persisted so "why did money move" is answerable from state alone (F10).
    #[serde(default)]
    pub obligation_id: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub verified_custodian_ids: Vec<String>,
}

/// Durable economic-identity store: one file per effect digest. It is the
/// single source of truth for both uniqueness and spend budget (budget is
/// derived by summing live claims, so it cannot drift from identity).
///
/// Uniqueness across processes rests on fsynced `create_new`; the budget sum
/// and state transitions are serialized by an in-process lock only.
#[derive(Debug)]
pub struct ClaimStore {
    root: PathBuf,
    lock: Mutex<()>,
}

fn io<T>(r: std::io::Result<T>) -> PayResult<T> {
    r.map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))
}

impl ClaimStore {
    pub fn open(root: impl Into<PathBuf>) -> PayResult<Self> {
        let root = root.into();
        io(fs::create_dir_all(&root))?;
        Ok(Self { root, lock: Mutex::new(()) })
    }

    fn path(&self, digest: &str) -> PathBuf {
        self.root.join(format!("{}.claim.json", digest.trim_start_matches("sha256:")))
    }

    fn write_new(&self, claim: &Claim) -> PayResult<()> {
        let bytes = serde_json::to_vec(claim).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
        let mut file = io(OpenOptions::new().write(true).create_new(true).open(self.path(&claim.effect_digest)))?;
        io(file.write_all(&bytes))?;
        io(file.sync_all())?;
        if let Ok(dir) = OpenOptions::new().read(true).open(&self.root) {
            let _ = dir.sync_all();
        }
        Ok(())
    }

    fn write_replace(&self, claim: &Claim) -> PayResult<()> {
        let bytes = serde_json::to_vec(claim).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?;
        let tmp = self.root.join(format!("{}.tmp", claim.effect_digest.trim_start_matches("sha256:")));
        {
            let mut file = io(OpenOptions::new().write(true).create(true).truncate(true).open(&tmp))?;
            io(file.write_all(&bytes))?;
            io(file.sync_all())?;
        }
        io(fs::rename(&tmp, self.path(&claim.effect_digest)))?;
        if let Ok(dir) = OpenOptions::new().read(true).open(&self.root) {
            let _ = dir.sync_all();
        }
        Ok(())
    }

    pub fn get(&self, digest: &str) -> PayResult<Option<Claim>> {
        match fs::read(self.path(digest)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}:{e}", refusal::CLAIM_STORE_FAILED)),
        }
    }

    pub fn list(&self) -> PayResult<Vec<Claim>> {
        let mut out = Vec::new();
        for entry in io(fs::read_dir(&self.root))? {
            let path = io(entry)?.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json")
                && path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".claim.json"))
            {
                let bytes = io(fs::read(&path))?;
                out.push(serde_json::from_slice(&bytes).map_err(|e| format!("{}:{e}", refusal::CLAIM_STORE_FAILED))?);
            }
        }
        out.sort_by(|a: &Claim, b| a.effect_digest.cmp(&b.effect_digest));
        Ok(out)
    }

    /// Sum of minor units already committed against `original` by live
    /// (non-refused) reversal claims.
    pub fn reversed_total(&self, original: &str) -> PayResult<u64> {
        Ok(self
            .list()?
            .iter()
            .filter(|c| c.reverses.as_deref() == Some(original) && c.state != ClaimState::Refused)
            .map(|c| c.amount_minor)
            .sum())
    }

    /// Atomically claim economic identity and reserve budget.
    ///
    /// `epoch_cap` is enforced against live non-reversal claims of the same
    /// principal+currency. An existing `Refused` claim (definite absence) may be
    /// re-reserved; every other existing state refuses with its own reason.
    pub fn reserve(&self, claim: &Claim, epoch_cap: Option<u64>) -> PayResult<()> {
        let _guard = self.lock.lock().map_err(|_| refusal::CLAIM_STORE_FAILED.to_string())?;
        if let Some(existing) = self.get(&claim.effect_digest)? {
            return match existing.state {
                ClaimState::Executed => refuse(refusal::ALREADY_SETTLED),
                ClaimState::UnknownOutcome => refuse(refusal::OUTCOME_UNKNOWN),
                ClaimState::Reserved => refuse(refusal::IN_FLIGHT),
                ClaimState::Refused => {
                    self.check_budget(claim, epoch_cap)?;
                    self.write_replace(claim)
                }
            };
        }
        self.check_budget(claim, epoch_cap)?;
        self.write_new(claim)
    }

    fn check_budget(&self, claim: &Claim, epoch_cap: Option<u64>) -> PayResult<()> {
        if claim.reverses.is_none()
            && !claim.obligation_id.is_empty()
            && self.list()?.iter().any(|c| {
                c.principal == claim.principal
                    && c.obligation_id == claim.obligation_id
                    && c.reverses.is_none()
                    && c.state != ClaimState::Refused
                    && c.effect_digest != claim.effect_digest
            })
        {
            return refuse(refusal::OBLIGATION_ALREADY_CLAIMED);
        }
        let Some(cap) = epoch_cap else { return Ok(()) };
        let used: u128 = self
            .list()?
            .iter()
            .filter(|c| {
                c.principal == claim.principal
                    && c.currency == claim.currency
                    && c.reverses.is_none()
                    && c.state != ClaimState::Refused
                    && c.effect_digest != claim.effect_digest
            })
            .map(|c| u128::from(c.amount_minor))
            .sum();
        if used + u128::from(claim.amount_minor) > u128::from(cap) {
            return refuse(refusal::BUDGET_EXCEEDED);
        }
        Ok(())
    }

    /// Move a claim between states; refuses if the observed state is not `from`.
    pub fn transition(
        &self,
        digest: &str,
        from: &[ClaimState],
        to: ClaimState,
        construct_digest: Option<&str>,
        detail: &str,
    ) -> PayResult<Claim> {
        let _guard = self.lock.lock().map_err(|_| refusal::CLAIM_STORE_FAILED.to_string())?;
        let Some(mut claim) = self.get(digest)? else {
            return Err(format!("{}:MISSING_CLAIM", refusal::CLAIM_STORE_FAILED));
        };
        if !from.contains(&claim.state) {
            return Err(format!("{}:UNEXPECTED_STATE_{:?}", refusal::CLAIM_STORE_FAILED, claim.state));
        }
        claim.state = to;
        if let Some(c) = construct_digest {
            claim.construct_digest = Some(c.to_string());
        }
        claim.detail = detail.to_string();
        self.write_replace(&claim)?;
        Ok(claim)
    }
}
