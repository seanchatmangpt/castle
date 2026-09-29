use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::castle::ConstructAdmission;

use super::admission::PaymentAdmission;
use super::dirlock::{publish_new, DirLock};
use super::money::Currency;
use super::settlement::{RejectionObservation, ReturnObservation, SettlementObservation};

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

/// Funds reserved against an account for an in-flight effect. Holds do not move
/// balances; they reduce `available`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerHold {
    pub effect_digest: String,
    pub account: String,
    pub currency: Currency,
    pub amount_minor: u64,
    pub construct_digest: String,
}

/// Proof that a real CONSTRUCT admission exists for the DO in progress. It can
/// only be minted from a genuine `ConstructAdmission`, so a ledger `post` cannot
/// be reached with a forged or absent CONSTRUCT chain.
#[derive(Debug, Clone)]
pub struct ActuationToken {
    construct_digest: String,
}

impl ActuationToken {
    #[must_use]
    pub fn from_construct(admission: &ConstructAdmission) -> Self {
        Self { construct_digest: admission.construct_digest.clone() }
    }
    #[must_use]
    pub fn construct_digest(&self) -> &str {
        &self.construct_digest
    }
}

/// Settlement surface. `post` demands a sealed `PaymentAdmission`, so a ledger
/// entry cannot be written without an admitted effect. Implementations must
/// make `post` atomic and idempotent per `effect_digest`.
pub trait LedgerPort: Send + Sync {
    fn balance(&self, account: &str, currency: Currency) -> Result<u64, LedgerError>;
    fn lookup(&self, effect_digest: &str) -> Result<Option<LedgerEntry>, LedgerError>;
    fn post(&self, admission: &PaymentAdmission, token: &ActuationToken) -> Result<LedgerEntry, LedgerError>;

    /// Reserve funds for an in-flight effect (idempotent per digest).
    fn hold(&self, _admission: &PaymentAdmission, _token: &ActuationToken) -> Result<LedgerHold, LedgerError> {
        Err(LedgerError::Unavailable("HOLD_UNSUPPORTED".into()))
    }
    /// Free a hold after an observed rail rejection.
    fn release_hold(&self, _obs: &RejectionObservation) -> Result<(), LedgerError> {
        Err(LedgerError::Unavailable("HOLD_UNSUPPORTED".into()))
    }
    /// Convert a hold into the settled entry, from an observed finality.
    fn settle_hold(&self, _obs: &SettlementObservation) -> Result<LedgerEntry, LedgerError> {
        Err(LedgerError::Unavailable("HOLD_UNSUPPORTED".into()))
    }
    /// Post the inverse entry for an observed return of settled funds.
    fn settle_return(&self, _obs: &ReturnObservation) -> Result<LedgerEntry, LedgerError> {
        Err(LedgerError::Unavailable("HOLD_UNSUPPORTED".into()))
    }
    /// Balance minus live holds (default: no holds exist).
    fn available(&self, account: &str, currency: Currency) -> Result<u64, LedgerError> {
        self.balance(account, currency)
    }
    /// The live hold for a digest, if any.
    fn hold_of(&self, _effect_digest: &str) -> Result<Option<LedgerHold>, LedgerError> {
        Ok(None)
    }
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

    fn return_path(&self, digest: &str) -> PathBuf {
        self.root.join(format!("{}.return.json", digest.trim_start_matches("sha256:")))
    }
    fn hold_path(&self, digest: &str) -> PathBuf {
        self.root.join(format!("{}.hold.json", digest.trim_start_matches("sha256:")))
    }
    fn released_path(&self, digest: &str) -> PathBuf {
        self.root.join(format!("{}.hold.released.json", digest.trim_start_matches("sha256:")))
    }

    fn read_suffix(&self, suffix: &str) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut out: Vec<LedgerEntry> = Vec::new();
        for e in fs::read_dir(&self.root).map_err(unavailable)? {
            let path = e.map_err(unavailable)?.path();
            if path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(suffix) && !n.starts_with('.')) {
                let bytes = fs::read(&path).map_err(unavailable)?;
                out.push(serde_json::from_slice(&bytes).map_err(unavailable)?);
            }
        }
        out.sort_by_key(|e| e.seq);
        Ok(out)
    }

    /// Forward settlement entries (`<digest>.entry.json`).
    pub fn entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        self.read_suffix(".entry.json")
    }

    /// Inverse entries written by `settle_return` (`<digest>.return.json`).
    pub fn returns(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        self.read_suffix(".return.json")
    }

    fn all_entries(&self) -> Result<Vec<LedgerEntry>, LedgerError> {
        let mut v = self.entries()?;
        v.extend(self.returns()?);
        v.sort_by_key(|e| e.seq);
        Ok(v)
    }

    fn next_seq(&self) -> Result<u64, LedgerError> {
        Ok(self.all_entries()?.iter().map(|e| e.seq).max().unwrap_or(0) + 1)
    }

    /// Live holds: a hold file with neither a settled entry nor a release marker.
    fn live_holds(&self) -> Result<Vec<LedgerHold>, LedgerError> {
        let mut out = Vec::new();
        for e in fs::read_dir(&self.root).map_err(unavailable)? {
            let path = e.map_err(unavailable)?.path();
            if !path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(".hold.json") && !n.starts_with('.')) {
                continue;
            }
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(unavailable(e)),
            };
            let h: LedgerHold = serde_json::from_slice(&bytes).map_err(unavailable)?;
            if self.entry_path(&h.effect_digest).exists() || self.released_path(&h.effect_digest).exists() {
                continue;
            }
            out.push(h);
        }
        out.sort_by(|a, b| a.effect_digest.cmp(&b.effect_digest));
        Ok(out)
    }

    fn held_excluding(&self, account: &str, currency: Currency, except: Option<&str>) -> Result<u128, LedgerError> {
        Ok(self
            .live_holds()?
            .iter()
            .filter(|h| h.account == account && h.currency == currency && Some(h.effect_digest.as_str()) != except)
            .map(|h| u128::from(h.amount_minor))
            .sum())
    }

    fn available_excluding(&self, account: &str, currency: Currency, except: Option<&str>) -> Result<u64, LedgerError> {
        let bal = u128::from(self.balance(account, currency)?);
        Ok(u64::try_from(bal.saturating_sub(self.held_excluding(account, currency, except)?)).unwrap_or(u64::MAX))
    }

    /// Refuse a credit that would push `account` past `u64::MAX` (the books would brick).
    fn check_credit(&self, account: &str, currency: Currency, amount: u64) -> Result<(), LedgerError> {
        let bal = u128::from(self.balance(account, currency)?);
        if bal + u128::from(amount) > u128::from(u64::MAX) {
            return Err(LedgerError::Unavailable("CREDIT_OVERFLOW".into()));
        }
        Ok(())
    }

    /// Conservation: sum of all balances equals sum of openings, per currency.
    pub fn conserves(&self, currency: Currency) -> Result<bool, LedgerError> {
        let openings = self.openings()?;
        let entries = self.all_entries()?;
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
        for e in self.all_entries()?.iter().filter(|e| e.currency == currency) {
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

    fn post(&self, admission: &PaymentAdmission, token: &ActuationToken) -> Result<LedgerEntry, LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let _dir = DirLock::acquire(&self.root).map_err(unavailable)?;
        let effect = admission.effect();
        if let Some(existing) = self.lookup(effect.digest())? {
            return Ok(existing); // idempotent replay: at most one entry per digest
        }
        let money = effect.money();
        if self.available_excluding(effect.payer(), money.currency, Some(effect.digest()))? < money.minor {
            return Err(LedgerError::InsufficientFunds);
        }
        self.check_credit(effect.payee(), money.currency, money.minor)?;
        let entry = LedgerEntry {
            seq: self.next_seq()?,
            effect_digest: effect.digest().to_string(),
            debit_account: effect.payer().to_string(),
            credit_account: effect.payee().to_string(),
            amount_minor: money.minor,
            currency: money.currency,
            construct_digest: token.construct_digest().to_string(),
        };
        let bytes = serde_json::to_vec(&entry).map_err(unavailable)?;
        if !publish_new(&self.root, &self.entry_path(effect.digest()), &bytes).map_err(unavailable)? {
            return self.lookup(effect.digest())?.ok_or_else(|| unavailable("ENTRY_VANISHED"));
        }
        let _ = fs::remove_file(self.hold_path(effect.digest()));
        Ok(entry)
    }

    fn hold(&self, admission: &PaymentAdmission, token: &ActuationToken) -> Result<LedgerHold, LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let _dir = DirLock::acquire(&self.root).map_err(unavailable)?;
        let effect = admission.effect();
        let digest = effect.digest();
        if self.lookup(digest)?.is_some() {
            return Err(unavailable("HOLD_AFTER_SETTLEMENT"));
        }
        if let Some(existing) = self.hold_of(digest)? {
            return Ok(existing); // idempotent per digest
        }
        if self.released_path(digest).exists() {
            // Fresh admission after a rejection: retire the stale release marker.
            let _ = fs::remove_file(self.hold_path(digest));
            fs::remove_file(self.released_path(digest)).map_err(unavailable)?;
        }
        let money = effect.money();
        if self.available_excluding(effect.payer(), money.currency, Some(digest))? < money.minor {
            return Err(LedgerError::InsufficientFunds);
        }
        self.check_credit(effect.payee(), money.currency, money.minor)?;
        let hold = LedgerHold {
            effect_digest: digest.to_string(),
            account: effect.payer().to_string(),
            currency: money.currency,
            amount_minor: money.minor,
            construct_digest: token.construct_digest().to_string(),
        };
        let bytes = serde_json::to_vec(&hold).map_err(unavailable)?;
        if !publish_new(&self.root, &self.hold_path(digest), &bytes).map_err(unavailable)? {
            // A stale non-live file (should not happen); replace it under the lock.
            super::dirlock::publish_replace(&self.root, &self.hold_path(digest), &bytes).map_err(unavailable)?;
        }
        Ok(hold)
    }

    fn hold_of(&self, effect_digest: &str) -> Result<Option<LedgerHold>, LedgerError> {
        Ok(self.live_holds()?.into_iter().find(|h| h.effect_digest == effect_digest))
    }

    fn available(&self, account: &str, currency: Currency) -> Result<u64, LedgerError> {
        self.available_excluding(account, currency, None)
    }

    fn release_hold(&self, obs: &RejectionObservation) -> Result<(), LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let _dir = DirLock::acquire(&self.root).map_err(unavailable)?;
        let digest = obs.effect_digest();
        if self.lookup(digest)?.is_some() {
            return Err(unavailable("RELEASE_AFTER_SETTLEMENT"));
        }
        let Some(hold) = self.hold_of(digest)? else {
            return if self.released_path(digest).exists() { Ok(()) } else { Err(unavailable("HOLD_MISSING")) };
        };
        let bytes = serde_json::to_vec(&hold).map_err(unavailable)?;
        super::dirlock::publish_replace(&self.root, &self.released_path(digest), &bytes).map_err(unavailable)?;
        let _ = fs::remove_file(self.hold_path(digest));
        Ok(())
    }

    fn settle_hold(&self, obs: &SettlementObservation) -> Result<LedgerEntry, LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let _dir = DirLock::acquire(&self.root).map_err(unavailable)?;
        let digest = obs.effect_digest();
        if let Some(existing) = self.lookup(digest)? {
            return Ok(existing); // same observation twice => same single entry
        }
        let Some(hold) = self.hold_of(digest)? else {
            return Err(unavailable("HOLD_MISSING"));
        };
        let payee = obs.payee();
        if self.balance(&hold.account, hold.currency)? < hold.amount_minor {
            return Err(LedgerError::InsufficientFunds);
        }
        self.check_credit(payee, hold.currency, hold.amount_minor)?;
        let entry = LedgerEntry {
            seq: self.next_seq()?,
            effect_digest: hold.effect_digest.clone(),
            debit_account: hold.account.clone(),
            credit_account: payee.to_string(),
            amount_minor: hold.amount_minor,
            currency: hold.currency,
            construct_digest: hold.construct_digest.clone(),
        };
        let bytes = serde_json::to_vec(&entry).map_err(unavailable)?;
        if !publish_new(&self.root, &self.entry_path(digest), &bytes).map_err(unavailable)? {
            return self.lookup(digest)?.ok_or_else(|| unavailable("ENTRY_VANISHED"));
        }
        let _ = fs::remove_file(self.hold_path(digest));
        Ok(entry)
    }

    fn settle_return(&self, obs: &ReturnObservation) -> Result<LedgerEntry, LedgerError> {
        let _guard = self.lock.lock().map_err(|_| unavailable("LOCK_POISONED"))?;
        let _dir = DirLock::acquire(&self.root).map_err(unavailable)?;
        let digest = obs.effect_digest();
        match fs::read(self.return_path(digest)) {
            Ok(bytes) => return serde_json::from_slice(&bytes).map_err(unavailable),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(unavailable(e)),
        }
        let Some(orig) = self.lookup(digest)? else {
            return Err(unavailable("ENTRY_MISSING"));
        };
        // The payee may only give back funds it has not itself reserved for in-flight effects.
        if self.available_excluding(&orig.credit_account, orig.currency, None)? < orig.amount_minor {
            return Err(LedgerError::InsufficientFunds);
        }
        self.check_credit(&orig.debit_account, orig.currency, orig.amount_minor)?;
        let entry = LedgerEntry {
            seq: self.next_seq()?,
            effect_digest: orig.effect_digest.clone(),
            debit_account: orig.credit_account.clone(),
            credit_account: orig.debit_account.clone(),
            amount_minor: orig.amount_minor,
            currency: orig.currency,
            construct_digest: orig.construct_digest.clone(),
        };
        let bytes = serde_json::to_vec(&entry).map_err(unavailable)?;
        if !publish_new(&self.root, &self.return_path(digest), &bytes).map_err(unavailable)? {
            return Err(unavailable("RETURN_RACE"));
        }
        Ok(entry)
    }
}
