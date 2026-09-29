use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::admission::PaymentAdmission;
use super::money::Currency;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    InsufficientFunds,
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub seq: u64,
    pub effect_digest: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount_minor: u64,
    pub currency: Currency,
    /// castle CONSTRUCT digest under which the entry was actuated.
    pub construct_digest: String,
}

/// Settlement surface. `post` demands a sealed `PaymentAdmission`, so a ledger
/// entry cannot be written without an admitted effect. Implementations must
/// make `post` atomic and idempotent per `effect_digest`.
pub trait LedgerPort: Send + Sync {
    fn balance(&self, account: &str, currency: Currency) -> Result<u64, LedgerError>;
    fn lookup(&self, effect_digest: &str) -> Result<Option<LedgerEntry>, LedgerError>;
    fn post(&self, admission: &PaymentAdmission, construct_digest: &str) -> Result<LedgerEntry, LedgerError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Opening {
    account: String,
    currency: Currency,
    amount_minor: u64,
}

/// Reference/test ledger: opening balances plus one fsynced `create_new` entry
/// file per effect digest. Production ledger authority is xaas Postgres; the
/// CLI refuses this implementation unless `--reference-ledger` is passed.
#[derive(Debug)]
pub struct FileJournalLedger {
    root: PathBuf,
    lock: Mutex<()>,
}

fn unavailable<E: std::fmt::Display>(e: E) -> LedgerError {
    LedgerError::Unavailable(e.to_string())
}

impl FileJournalLedger {
    pub fn open(root: impl Into<PathBuf>, opening: &[(&str, Currency, u64)]) -> Result<Self, LedgerError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(unavailable)?;
        let path = root.join("opening.json");
        if !path.exists() {
            let rows: Vec<Opening> = opening
                .iter()
                .map(|(a, c, m)| Opening { account: (*a).to_string(), currency: *c, amount_minor: *m })
                .collect();
            let bytes = serde_json::to_vec(&rows).map_err(unavailable)?;
            let mut f = OpenOptions::new().write(true).create_new(true).open(&path).map_err(unavailable)?;
            f.write_all(&bytes).and_then(|()| f.sync_all()).map_err(unavailable)?;
        }
        Ok(Self { root, lock: Mutex::new(()) })
    }

    fn entry_path(&self, digest: &str) -> PathBuf {
        self.root.join(format!("{}.entry.json", digest.trim_start_matches("sha256:")))
    }

    fn openings(&self) -> Result<Vec<Opening>, LedgerError> {
        let bytes = fs::read(self.root.join("opening.json")).map_err(unavailable)?;
        serde_json::from_slice(&bytes).map_err(unavailable)
    }

    pub fn entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut out: Vec<LedgerEntry> = Vec::new();
        for e in fs::read_dir(&self.root).map_err(unavailable)? {
            let path = e.map_err(unavailable)?.path();
            if path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".entry.json")) {
                let bytes = fs::read(&path).map_err(unavailable)?;
                out.push(serde_json::from_slice(&bytes).map_err(unavailable)?);
            }
        }
        out.sort_by_key(|e| e.seq);
        Ok(out)
    }

    /// Conservation: sum of all balances equals sum of openings, per currency.
    pub fn conserves(&self, currency: Currency) -> Result<bool, LedgerError> {
        let openings = self.openings()?;
        let entries = self.entries()?;
        let opening_total: u128 = openings.iter().filter(|o| o.currency == currency).map(|o| u128::from(o.amount_minor)).sum();
        let mut accounts: BTreeMap<String, u128> = BTreeMap::new();
        for o in openings.iter().filter(|o| o.currency == currency) {
            *accounts.entry(o.account.clone()).or_default() += u128::from(o.amount_minor);
        }
        for e in entries.iter().filter(|e| e.currency == currency) {
            let d = accounts.entry(e.debit_account.clone()).or_default();
            *d = d.checked_sub(u128::from(e.amount_minor)).ok_or(LedgerError::InsufficientFunds)?;
            *accounts.entry(e.credit_account.clone()).or_default() += u128::from(e.amount_minor);
        }
        Ok(accounts.values().sum::<u128>() == opening_total)
    }
}

impl LedgerPort for FileJournalLedger {
    fn balance(&self, account: &str, currency: Currency) -> Result<u64, LedgerError> {
        let mut bal: i128 = self
            .openings()?
            .iter()
            .filter(|o| o.account == account && o.currency == currency)
            .map(|o| i128::from(o.amount_minor))
            .sum();
        for e in self.entries()?.iter().filter(|e| e.currency == currency) {
            if e.debit_account == account {
                bal -= i128::from(e.amount_minor);
            }
            if e.credit_account == account {
                bal += i128::from(e.amount_minor);
            }
        }
        u64::try_from(bal).map_err(|_| LedgerError::Unavailable("NEGATIVE_BALANCE_INVARIANT".into()))
    }

    fn lookup(&self, effect_digest: &str) -> Result<Option<LedgerEntry>, LedgerError> {
        match fs::read(self.entry_path(effect_digest)) {
            Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(unavailable),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(unavailable(e)),
        }
    }

    fn post(&self, admission: &PaymentAdmission, construct_digest: &str) -> Result<LedgerEntry, LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let effect = admission.effect();
        if let Some(existing) = self.lookup(effect.digest())? {
            return Ok(existing); // idempotent replay: at most one entry per digest
        }
        let money = effect.money();
        if self.balance(effect.payer(), money.currency)? < money.minor {
            return Err(LedgerError::InsufficientFunds);
        }
        let entry = LedgerEntry {
            seq: self.entries()?.len() as u64 + 1,
            effect_digest: effect.digest().to_string(),
            debit_account: effect.payer().to_string(),
            credit_account: effect.payee().to_string(),
            amount_minor: money.minor,
            currency: money.currency,
            construct_digest: construct_digest.to_string(),
        };
        let bytes = serde_json::to_vec(&entry).map_err(unavailable)?;
        let mut f = OpenOptions::new().write(true).create_new(true).open(self.entry_path(effect.digest())).map_err(unavailable)?;
        f.write_all(&bytes).and_then(|()| f.sync_all()).map_err(unavailable)?;
        if let Ok(dir) = OpenOptions::new().read(true).open(&self.root) {
            let _ = dir.sync_all();
        }
        Ok(entry)
    }
}
