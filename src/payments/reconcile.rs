//! Reconciliation: the only path that resolves an `UnknownOutcome` claim.
//! Resolution is strictly through the ledger's own record of the effect digest;
//! there is never a blind retry.

use std::fs;
use std::path::Path;

use crate::v26_8_18::{DurableBrceOutcomeRecord, DurableBrcePrepareRecord};

use super::claim_store::{ClaimState, ClaimStore};
use super::ledger::{LedgerEntry, LedgerPort};
use super::refusal::{self, refuse, PayResult};

pub const NOT_RECONCILABLE: &str = "REFUSED:PAYMENT_NOT_RECONCILABLE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileResolution {
    /// Ledger holds a matching entry; claim is now `Executed`.
    Settled(LedgerEntry),
    /// Reachable ledger proves no entry exists; claim is now `Refused`.
    ProvenAbsent,
    /// Ledger unreachable; claim unchanged.
    StillUnknown,
}

pub fn reconcile(effect_digest: &str, claims: &ClaimStore, ledger: &dyn LedgerPort) -> PayResult<ReconcileResolution> {
    let Some(claim) = claims.get(effect_digest)? else {
        return refuse(NOT_RECONCILABLE);
    };
    if !matches!(claim.state, ClaimState::Reserved | ClaimState::UnknownOutcome) {
        return refuse(NOT_RECONCILABLE);
    }
    // A live rail hold means the effect may be in flight at a rail: absence from the
    // ledger's entry table proves nothing. Only finalize_via_rail may resolve it.
    match ledger.hold_of(effect_digest) {
        Ok(Some(_)) => return refuse(NOT_RECONCILABLE),
        Ok(None) => {}
        Err(_) => return Ok(ReconcileResolution::StillUnknown),
    }
    match ledger.lookup(effect_digest) {
        Err(_) => Ok(ReconcileResolution::StillUnknown),
        Ok(Some(entry)) => {
            let matches = entry.effect_digest == claim.effect_digest
                && entry.amount_minor == claim.amount_minor
                && entry.currency == claim.currency
                && entry.debit_account == claim.payer
                && entry.credit_account == claim.payee;
            if !matches {
                return refuse(refusal::RECONCILIATION_DRIFT);
            }
            claims.transition(
                effect_digest,
                &[ClaimState::Reserved, ClaimState::UnknownOutcome],
                ClaimState::Executed,
                Some(&entry.construct_digest),
                "RECONCILED_SETTLED",
            )?;
            Ok(ReconcileResolution::Settled(entry))
        }
        Ok(None) => {
            claims.transition(
                effect_digest,
                &[ClaimState::Reserved, ClaimState::UnknownOutcome],
                ClaimState::Refused,
                None,
                "RECONCILED_PROVEN_ABSENT",
            )?;
            Ok(ReconcileResolution::ProvenAbsent)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JournalRecovery {
    /// Prepare receipt digests with no matching outcome: crash mid-DO, UNKNOWN.
    pub prepared_without_outcome: Vec<String>,
    /// Prepare receipt digests with a matching outcome.
    pub complete: Vec<String>,
}

fn journal_err<E: std::fmt::Display>(e: E) -> String {
    format!("BLOCKED:BRCE_JOURNAL_RECOVERY_FAILED:{e}")
}

pub fn recover_journal(journal_root: &Path) -> PayResult<JournalRecovery> {
    let mut prepares: Vec<String> = Vec::new();
    let mut outcomes: Vec<String> = Vec::new();
    let dir = match fs::read_dir(journal_root) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(JournalRecovery::default()),
        Err(e) => return Err(journal_err(e)),
    };
    for entry in dir {
        let path = entry.map_err(journal_err)?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if name.ends_with(".prepare.json") {
            let rec: DurableBrcePrepareRecord = serde_json::from_slice(&fs::read(&path).map_err(journal_err)?).map_err(journal_err)?;
            prepares.push(rec.prepare_receipt.receipt_digest);
        } else if name.ends_with(".outcome.json") {
            let rec: DurableBrceOutcomeRecord = serde_json::from_slice(&fs::read(&path).map_err(journal_err)?).map_err(journal_err)?;
            outcomes.push(rec.prepare_receipt_digest);
        }
    }
    prepares.sort();
    let mut out = JournalRecovery::default();
    for p in prepares {
        if outcomes.contains(&p) {
            out.complete.push(p);
        } else {
            out.prepared_without_outcome.push(p);
        }
    }
    Ok(out)
}
