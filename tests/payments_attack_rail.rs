//! Lane `rail-crash` (skeptic): crash-consistency attacks on rail execution, finality,
//! ledger holds and the hostile SimRail. Real files, real BRCE, real SimRail, real
//! FileJournalLedger. "Crashes" are produced by hand-written REAL trait impls that
//! genuinely fail (a ledger/rail wrapper that makes the claim store unwritable at a
//! chosen instant), not by mocks. Tests marked #[ignore = "FINDING: ..."] FAIL on
//! current code; run with `--ignored` to see them fail.
mod common;
use common::payments::*;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Barrier, Mutex};

use castle::payments::execute_rail::*;
use castle::payments::rail::RailInstruction;
use castle::payments::settlement::{
    apply_finality, FinalityEvidence, FinalityKind, RejectionObservation, ReturnObservation, SettlementObservation,
};
use castle::payments::*;

// ---------------------------------------------------------------- helpers

fn bindings(expires_at_ms: u64) -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: "law:v1".into(),
        counterparty_evidence_digest: "cp:v1".into(),
        funding_source_id: "fund:treasury".into(),
        resource_reservation_id: "res:1".into(),
        rail_profile_id: "sim-rail".into(),
        message_profile_version: "pain.001.001.09".into(),
        parent_receipt: "receipt:parent".into(),
        created_at_ms: 0,
        expires_at_ms,
    }
}

fn params() -> RailExecutionParams {
    RailExecutionParams {
        bindings: bindings(100_000),
        created_at_iso: "2026-09-29T12:34:56Z".into(),
        debtor_name: "Treasury Ltd".into(),
        creditor_name: "Supplier 9821 GmbH".into(),
        debtor_agent_bic: "DEUTDEFF".into(),
        creditor_agent_bic: "BOFAUS3N".into(),
        rail_profile: "pain.001.001.09".into(),
    }
}

fn sim(fx: &Fixture, tag: &str, mode: SimMode) -> SimRail {
    SimRail::open(&fx.dir.join(format!("rail-{tag}")), mode).expect("sim rail")
}

async fn submit(fx: &Fixture, rail: &dyn RailActuator, amount: &str, obligation: &str, nonce: &str) -> RailSubmission {
    let a = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    submit_via_rail(a, &params(), &fx.exec_ctx(), rail).await.expect("submit_via_rail")
}

fn claim_state(claims: &ClaimStore, digest: &str) -> ClaimState {
    claims.get(digest).unwrap().unwrap().state
}

fn lock_dir(p: &Path) {
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o555)).unwrap();
}
fn unlock_dir(p: &Path) {
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Restores directory permissions even when an assertion unwinds.
struct Unlock(PathBuf);
impl Drop for Unlock {
    fn drop(&mut self) {
        unlock_dir(&self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    /// `settle_hold` fails once, before touching state (crash before the ledger step).
    FailSettleOnce,
    /// `settle_hold` completes, then the claim store becomes unwritable (crash before claim Final).
    LockAfterSettle,
    /// `release_hold` completes, then the claim store becomes unwritable.
    LockAfterRelease,
    /// `settle_return` completes, then the claim store becomes unwritable.
    LockAfterReturn,
}

/// A real LedgerPort that delegates to a real FileJournalLedger and genuinely
/// "crashes" (fails every subsequent claim write) right after a chosen step.
struct FaultLedger<'a> {
    inner: &'a FileJournalLedger,
    claims_dir: PathBuf,
    fault: Mutex<Fault>,
}
impl FaultLedger<'_> {
    fn arm(&self, f: Fault) {
        *self.fault.lock().unwrap() = f;
    }
    fn take(&self, want: Fault) -> bool {
        let mut g = self.fault.lock().unwrap();
        if *g == want {
            *g = Fault::None;
            true
        } else {
            false
        }
    }
}
impl LedgerPort for FaultLedger<'_> {
    fn balance(&self, a: &str, c: Currency) -> Result<u64, LedgerError> {
        self.inner.balance(a, c)
    }
    fn lookup(&self, d: &str) -> Result<Option<LedgerEntry>, LedgerError> {
        self.inner.lookup(d)
    }
    fn post(&self, a: &PaymentAdmission, t: &ActuationToken) -> Result<LedgerEntry, LedgerError> {
        self.inner.post(a, t)
    }
    fn hold(&self, a: &PaymentAdmission, t: &ActuationToken) -> Result<LedgerHold, LedgerError> {
        self.inner.hold(a, t)
    }
    fn release_hold(&self, o: &RejectionObservation) -> Result<(), LedgerError> {
        let r = self.inner.release_hold(o);
        if self.take(Fault::LockAfterRelease) {
            lock_dir(&self.claims_dir);
        }
        r
    }
    fn settle_hold(&self, o: &SettlementObservation) -> Result<LedgerEntry, LedgerError> {
        if self.take(Fault::FailSettleOnce) {
            return Err(LedgerError::Unavailable("CRASH_BEFORE_SETTLE".into()));
        }
        let r = self.inner.settle_hold(o);
        if self.take(Fault::LockAfterSettle) {
            lock_dir(&self.claims_dir);
        }
        r
    }
    fn settle_return(&self, o: &ReturnObservation) -> Result<LedgerEntry, LedgerError> {
        let r = self.inner.settle_return(o);
        if self.take(Fault::LockAfterReturn) {
            lock_dir(&self.claims_dir);
        }
        r
    }
    fn available(&self, a: &str, c: Currency) -> Result<u64, LedgerError> {
        self.inner.available(a, c)
    }
    fn hold_of(&self, d: &str) -> Result<Option<LedgerHold>, LedgerError> {
        self.inner.hold_of(d)
    }
}

/// A real RailActuator that delegates to SimRail and makes the claim store
/// unwritable right after the rail accepted (crash between rail ack and claim Submitted).
struct LockAfterSubmit<'a> {
    inner: &'a SimRail,
    claims_dir: PathBuf,
}
impl RailActuator for LockAfterSubmit<'_> {
    fn submit(&self, i: &RailInstruction) -> Result<RailAck, RailError> {
        let r = self.inner.submit(i);
        lock_dir(&self.claims_dir);
        r
    }
    fn status(&self, c: &str) -> Result<RailStatus, RailError> {
        self.inner.status(c)
    }
}

// ------------------------------------------------- crash / resume (converging)

#[tokio::test]
async fn crash_before_ledger_settle_hold_leaves_hold_and_retry_settles_once() {
    let fx = Fixture::new("ar-c-before");
    let fl = FaultLedger { inner: &fx.ledger, claims_dir: fx.dir.join("claims"), fault: Mutex::new(Fault::None) };
    let rail = sim(&fx, "a", SimMode::SettleAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    fl.arm(Fault::FailSettleOnce);
    let err = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap_err();
    assert!(err.contains("LEDGER_UNAVAILABLE") || err.contains("CRASH_BEFORE_SETTLE"), "{err}");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Submitted);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some());
    assert!(fx.ledger.entries().unwrap().is_empty());

    let r = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap();
    assert!(matches!(r, FinalizeResult::Applied(FinalityOutcome::Settled(_))), "{r:?}");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn crash_after_ledger_settle_before_claim_final_resumes_idempotently() {
    let fx = Fixture::new("ar-c-after-settle");
    let fl = FaultLedger { inner: &fx.ledger, claims_dir: fx.dir.join("claims"), fault: Mutex::new(Fault::None) };
    let rail = sim(&fx, "b", SimMode::SettleAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    let _guard = Unlock(fx.dir.join("claims"));
    fl.arm(Fault::LockAfterSettle);
    let err = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap_err();
    assert!(err.contains("CLAIM_STORE_FAILED"), "{err}");
    // Books moved, claim did not: the half-committed state that a crash leaves.
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Submitted);

    unlock_dir(&fx.dir.join("claims"));
    for _ in 0..3 {
        let r = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap();
        assert!(matches!(r, FinalizeResult::Applied(_)), "{r:?}");
    }
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1, "resume must not double-settle");
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 600_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn crash_after_release_hold_before_claim_refused_resumes() {
    let fx = Fixture::new("ar-c-after-release");
    let fl = FaultLedger { inner: &fx.ledger, claims_dir: fx.dir.join("claims"), fault: Mutex::new(Fault::None) };
    let rail = sim(&fx, "c", SimMode::RejectAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    let _guard = Unlock(fx.dir.join("claims"));
    fl.arm(Fault::LockAfterRelease);
    assert!(finalize_via_rail(&d, &fx.claims, &fl, &rail).is_err());
    assert!(fx.ledger.hold_of(&d).unwrap().is_none(), "hold released");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Submitted, "claim did not follow");

    unlock_dir(&fx.dir.join("claims"));
    let r = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap();
    assert_eq!(r, FinalizeResult::Applied(FinalityOutcome::Released));
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Refused);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn crash_after_settle_return_before_claim_returned_resumes() {
    let fx = Fixture::new("ar-c-return");
    let fl = FaultLedger { inner: &fx.ledger, claims_dir: fx.dir.join("claims"), fault: Mutex::new(Fault::None) };
    let rail = sim(&fx, "r", SimMode::ReturnAfterSettle);
    let s = submit(&fx, &rail, "250000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    for _ in 0..2 {
        finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap();
    }
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);

    let _guard = Unlock(fx.dir.join("claims"));
    fl.arm(Fault::LockAfterReturn);
    assert!(finalize_via_rail(&d, &fx.claims, &fl, &rail).is_err());
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);

    unlock_dir(&fx.dir.join("claims"));
    let r = finalize_via_rail(&d, &fx.claims, &fl, &rail).unwrap();
    assert!(matches!(r, FinalizeResult::Applied(FinalityOutcome::Returned(_))), "{r:?}");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Returned);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1, "one inverse entry only");
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn missed_settled_report_then_returned_settles_implied_then_returns() {
    let fx = Fixture::new("ar-missed-settled");
    let rail = sim(&fx, "m", SimMode::ReturnAfterSettle);
    let s = submit(&fx, &rail, "250000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    // The caller never observed Accepted/Settled: burn polls straight to Returned.
    for _ in 0..3 {
        let _ = rail.status(&s.correlation_id).unwrap();
    }
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Submitted);

    let r = finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap();
    assert!(matches!(r, FinalizeResult::Applied(FinalityOutcome::Returned(_))), "{r:?}");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Returned);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn dropack_then_rail_down_then_recovery_settles_exactly_once() {
    let fx = Fixture::new("ar-drop-down");
    let rail = sim(&fx, "dd", SimMode::DropAckAfterAccept);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome);

    rail.set_mode(SimMode::Down).unwrap();
    for _ in 0..5 {
        assert_eq!(finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap(), FinalizeResult::StillUnknown);
    }
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::UnknownOutcome);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some(), "Down must never release the hold");

    rail.set_mode(SimMode::DropAckAfterAccept).unwrap();
    let mut settled = false;
    for _ in 0..6 {
        if matches!(
            finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap(),
            FinalizeResult::Applied(FinalityOutcome::Settled(_))
        ) {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1, "never resubmitted");
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);
}

#[tokio::test]
async fn rejection_then_late_settlement_report_cannot_reopen_a_refused_claim() {
    let fx = Fixture::new("ar-reorder");
    let rail = sim(&fx, "ro", SimMode::RejectAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    assert_eq!(
        finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap(),
        FinalizeResult::Applied(FinalityOutcome::Released)
    );
    let late = FinalityEvidence {
        effect_digest: d.clone(),
        correlation_id: s.correlation_id.clone(),
        evidence_digest: "late-settle".into(),
        kind: FinalityKind::Final,
        reason: "LATE".into(),
    };
    assert_eq!(apply_finality(&fx.claims, &fx.ledger, &late).unwrap_err(), "REFUSED:PAYMENT_EVIDENCE_CONFLICT");
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Refused);
}

// ------------------------------------------------------------ concurrency

#[tokio::test]
async fn racing_finalize_across_separate_handles_yields_exactly_one_entry() {
    let fx = Fixture::new("ar-race-final");
    let rail = sim(&fx, "rf", SimMode::Honest);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    // Push the rail to its terminal report before the race so every thread sees Settled.
    for _ in 0..2 {
        rail.status(&s.correlation_id).unwrap();
    }
    let n = 8;
    let barrier = Barrier::new(n);
    let results: Vec<Result<FinalizeResult, String>> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..n)
            .map(|_| {
                let (fx, d, barrier) = (&fx, d.clone(), &barrier);
                sc.spawn(move || {
                    let claims = ClaimStore::open(fx.dir.join("claims")).unwrap();
                    let ledger = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap();
                    let rail = SimRail::open(&fx.dir.join("rail-rf"), SimMode::Honest).unwrap();
                    barrier.wait();
                    finalize_via_rail(&d, &claims, &ledger, &rail)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(fx.ledger.entries().unwrap().len(), 1, "results: {results:?}");
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Final);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);
    assert!(results.iter().any(|r| matches!(r, Ok(FinalizeResult::Applied(FinalityOutcome::Settled(_))))));
}

#[test]
fn concurrent_holds_across_separate_ledger_handles_never_overdraw_available() {
    let fx = Fixture::with("ar-race-hold", 1_000_000, 500_000, 100_000_000);
    let admissions: Vec<PaymentAdmission> =
        (0..8).map(|i| fx.admit(fx.effect("300000", &format!("inv-{i}")), &format!("n-{i}")).unwrap()).collect();
    let tokens: Vec<ActuationToken> = admissions.iter().map(|a| fx.token_for(a)).collect();
    let barrier = Barrier::new(8);
    let outcomes: Vec<Result<LedgerHold, LedgerError>> = std::thread::scope(|sc| {
        let hs: Vec<_> = admissions
            .iter()
            .zip(tokens.iter())
            .map(|(a, t)| {
                let (fx, barrier) = (&fx, &barrier);
                sc.spawn(move || {
                    let l = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap(); // OWN handle
                    barrier.wait();
                    l.hold(a, t)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let ok = outcomes.iter().filter(|o| o.is_ok()).count();
    let poor = outcomes.iter().filter(|o| matches!(o, Err(LedgerError::InsufficientFunds))).count();
    assert_eq!((ok, poor), (3, 5), "{outcomes:?}");
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[test]
fn concurrent_submit_via_rail_across_handles_holds_at_most_the_balance() {
    let fx = Fixture::with("ar-race-submit", 1_000_000, 500_000, 100_000_000);
    let admissions: Vec<PaymentAdmission> =
        (0..8).map(|i| fx.admit(fx.effect("300000", &format!("inv-{i}")), &format!("n-{i}")).unwrap()).collect();
    let barrier = Barrier::new(8);
    let standings: Vec<Result<RailStandingAfterSubmit, String>> = std::thread::scope(|sc| {
        let hs: Vec<_> = admissions
            .into_iter()
            .map(|a| {
                let (fx, barrier) = (&fx, &barrier);
                sc.spawn(move || {
                    let ledger = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap();
                    let claims = ClaimStore::open(fx.dir.join("claims")).unwrap();
                    let rail = SimRail::open(&fx.dir.join("rail-shared"), SimMode::Honest).unwrap();
                    let mut ctx = fx.exec_ctx();
                    ctx.ledger = &ledger;
                    ctx.claims = &claims;
                    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                    barrier.wait();
                    rt.block_on(submit_via_rail(a, &params(), &ctx, &rail)).map(|s| s.standing)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let submitted = standings.iter().filter(|s| matches!(s, Ok(RailStandingAfterSubmit::Submitted))).count();
    assert!(submitted <= 3, "more effects submitted than funds allow: {standings:?}");
    let held: u64 = fx
        .claims
        .list()
        .unwrap()
        .iter()
        .filter(|c| c.state == ClaimState::Submitted)
        .map(|c| fx.ledger.hold_of(&c.effect_digest).unwrap().map_or(0, |h| h.amount_minor))
        .sum();
    assert!(held <= 1_000_000, "held {held} > balance");
    assert_eq!(fx.ledger.entries().unwrap().len(), 0);
}

// ------------------------------------------------- torn / corrupt (fail closed)

#[tokio::test]
async fn torn_hold_file_fails_closed_and_never_reaches_the_rail() {
    let fx = Fixture::new("ar-torn-hold");
    let rail = sim(&fx, "th", SimMode::Honest);
    let ledger_dir = fx.dir.join("ledger");
    std::fs::write(ledger_dir.join("deadbeef.hold.json"), b"{\"effect_digest\":\"sha256:de").unwrap();

    assert!(fx.ledger.available(PAYER, Currency::USD).is_err());
    assert!(fx.ledger.hold_of("sha256:x").is_err());

    let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    let sub = submit_via_rail(a, &params(), &fx.exec_ctx(), &rail).await.unwrap();
    assert_ne!(sub.standing, RailStandingAfterSubmit::Submitted, "{}", sub.detail);
    assert_eq!(rail.submissions_seen(&sub.correlation_id), 0);
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_ne!(claim_state(&fx.claims, &d), ClaimState::Final);
    // Empty file is equally torn.
    std::fs::write(ledger_dir.join("deadbeef.hold.json"), b"").unwrap();
    assert!(fx.ledger.available(PAYER, Currency::USD).is_err());
}

#[tokio::test]
async fn torn_entry_file_makes_balances_unavailable_not_wrong() {
    let fx = Fixture::new("ar-torn-entry");
    let rail = sim(&fx, "te", SimMode::SettleAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    std::fs::write(fx.dir.join("ledger").join("cafe.entry.json"), b"{\"seq\":").unwrap();
    assert!(fx.ledger.balance(PAYER, Currency::USD).is_err());
    assert!(fx.ledger.entries().is_err());
    let r = finalize_via_rail(&s.effect_id, &fx.claims, &fx.ledger, &rail);
    assert!(r.is_err(), "finalize must refuse to settle on top of a torn journal: {r:?}");
    assert_eq!(claim_state(&fx.claims, &s.effect_id), ClaimState::Submitted);
}

#[tokio::test]
async fn corrupt_claim_file_fails_closed_on_finalize() {
    let fx = Fixture::new("ar-torn-claim");
    let rail = sim(&fx, "tc", SimMode::SettleAfterPolls(0));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let hex = s.effect_id.trim_start_matches("sha256:");
    std::fs::write(fx.dir.join("claims").join(format!("{hex}.claim.json")), b"{").unwrap();
    assert!(finalize_via_rail(&s.effect_id, &fx.claims, &fx.ledger, &rail).is_err());
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn stuck_submitted_claim_retains_its_hold_and_stays_pending() {
    // Documents fail-safe behaviour: a rail that stays Accepted never frees funds.
    // (No expiry/release path exists: see report gap G1.)
    let fx = Fixture::new("ar-stuck");
    let rail = sim(&fx, "st", SimMode::SettleAfterPolls(u32::MAX - 1));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    for _ in 0..25 {
        assert_eq!(finalize_via_rail(&s.effect_id, &fx.claims, &fx.ledger, &rail).unwrap(), FinalizeResult::Pending);
    }
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 600_000);
    assert_eq!(claim_state(&fx.claims, &s.effect_id), ClaimState::Submitted);
}

// ---------------------------------------------------------------- FINDINGS

/// FINDING G2: crash after the rail ACCEPTED but before `Reserved -> Submitted` is
/// persisted leaves claim=Reserved, hold live, rail settled. `apply_finality` only
/// treats Submitted|UnknownOutcome as in-flight, so the rail's Settled report is
/// refused PAYMENT_NOT_SUBMITTED forever: rail paid, ledger never settles, hold leaks.
#[ignore = "FINDING: Reserved claim with rail-accepted effect can never be finalized (rail settled, ledger not)"]
#[tokio::test]
async fn crash_between_rail_ack_and_claim_submitted_is_recoverable() {
    let fx = Fixture::new("ar-f-ack-crash");
    let inner = sim(&fx, "ac", SimMode::SettleAfterPolls(0));
    let rail = LockAfterSubmit { inner: &inner, claims_dir: fx.dir.join("claims") };
    let _guard = Unlock(fx.dir.join("claims"));
    let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    let r = submit_via_rail(a, &params(), &fx.exec_ctx(), &rail).await;
    assert!(r.is_err(), "claim store was unwritable at transition time");
    unlock_dir(&fx.dir.join("claims"));

    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Reserved);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some());
    let corr = RailInstruction::correlation_id_for(&d);
    assert_eq!(inner.submissions_seen(&corr), 1);

    let mut last = String::new();
    let mut settled = false;
    for _ in 0..4 {
        match finalize_via_rail(&d, &fx.claims, &fx.ledger, &inner) {
            Ok(FinalizeResult::Applied(FinalityOutcome::Settled(_))) => {
                settled = true;
                break;
            }
            Ok(other) => last = format!("{other:?}"),
            Err(e) => last = e,
        }
    }
    assert!(settled, "rail settled {corr} but ledger cannot follow; last finalize = {last}");
}

/// FINDING G3: crash between `ledger.hold` and rail submit (admission is consumed by
/// value, so it is lost) leaves claim=Reserved + live hold. `finalize_via_rail` with
/// the rail reporting Unknown returns StillUnknown for Reserved (only UnknownOutcome
/// is resolved), and re-admission is refused IN_FLIGHT: hold and budget leak forever.
#[ignore = "FINDING: orphan Reserved claim + hold (crash before rail submit) has no recovery path"]
#[test]
fn orphan_hold_after_crash_before_rail_submit_is_recoverable() {
    let fx = Fixture::new("ar-f-orphan");
    let rail = sim(&fx, "or", SimMode::Honest);
    let d = {
        let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
        let t = fx.token_for(&a);
        fx.ledger.hold(&a, &t).unwrap();
        a.effect().digest().to_string()
        // `a` dropped here: the process "died" before the rail call.
    };
    assert_eq!(claim_state(&fx.claims, &d), ClaimState::Reserved);
    let r = finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail);
    let recovered = fx.ledger.hold_of(&d).unwrap().is_none() && claim_state(&fx.claims, &d) != ClaimState::Reserved;
    assert!(recovered, "hold {:?} claim {:?} finalize {r:?}", fx.ledger.hold_of(&d).unwrap(), claim_state(&fx.claims, &d));
}

/// FINDING G4: `apply_finality` takes caller-fabricated `FinalityEvidence` (public
/// fields) and never checks `correlation_id` against the effect, so any caller can
/// settle a held payment with no rail observation at all.
#[ignore = "FINDING: apply_finality accepts forged evidence with a foreign correlation_id and settles the ledger"]
#[tokio::test]
async fn forged_finality_evidence_with_foreign_correlation_is_refused() {
    let fx = Fixture::new("ar-f-forge");
    let rail = sim(&fx, "fg", SimMode::SettleAfterPolls(100));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let forged = FinalityEvidence {
        effect_digest: s.effect_id.clone(),
        correlation_id: "E2E-not-this-effect".into(),
        evidence_digest: "made-up".into(),
        kind: FinalityKind::Final,
        reason: "FORGED".into(),
    };
    let r = apply_finality(&fx.claims, &fx.ledger, &forged);
    assert!(r.is_err(), "ledger settled on forged evidence: {r:?}");
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(rail.settlement_count(&s.correlation_id), 0, "the rail never settled this effect");
}

/// FINDING G5: `settle_return` checks the payee's balance, not its available funds,
/// so a return can claw back money the payee has HELD for its own in-flight payment.
/// Invariant broken: balance >= live holds. The payee's later settle_hold then hits
/// InsufficientFunds while the rail has already settled it.
#[ignore = "FINDING: settle_return ignores holds; balance drops below live holds, later settle_hold cannot book rail finality"]
#[tokio::test]
async fn return_cannot_claw_back_funds_held_for_another_effect() {
    let fx = Fixture::with("ar-f-return-hold", 1_000_000, 500_000, 100_000_000);
    let rail_a = sim(&fx, "ra", SimMode::ReturnAfterSettle);
    let rail_b = sim(&fx, "rb", SimMode::SettleAfterPolls(50));
    let a = submit(&fx, &rail_a, "400000", "inv-A", "n-A").await;
    for _ in 0..2 {
        finalize_via_rail(&a.effect_id, &fx.claims, &fx.ledger, &rail_a).unwrap(); // Settled: payee owns 400k
    }
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);

    // Payee pays PAYER back 400k: its whole balance is now held.
    let eff_b = PaymentEffect::prepare(PRINCIPAL, PAYEE, PAYER, "400000", Currency::USD, "inv-B", "invoice-payment", None).unwrap();
    let ab = fx.admit(eff_b, "n-B").unwrap();
    let b = submit_via_rail(ab, &params(), &fx.exec_ctx(), &rail_b).await.unwrap();
    assert_eq!(b.standing, RailStandingAfterSubmit::Submitted, "{}", b.detail);
    assert_eq!(fx.ledger.available(PAYEE, Currency::USD).unwrap(), 0);

    // The rail now reports A as returned.
    let _ = finalize_via_rail(&a.effect_id, &fx.claims, &fx.ledger, &rail_a);
    let held = fx.ledger.hold_of(&b.effect_id).unwrap().map_or(0, |h| h.amount_minor);
    let bal = fx.ledger.balance(PAYEE, Currency::USD).unwrap();
    assert!(bal >= held, "payee balance {bal} < its live hold {held}: hold invariant broken by settle_return");
}

/// FINDING G6: settle_hold/hold credit the payee without checking representability.
/// A payee at u64::MAX gets a legal-looking entry after which `balance(payee)` errors
/// NEGATIVE_BALANCE_INVARIANT for every later call: the books are bricked by a
/// settlement the ledger itself wrote.
#[ignore = "FINDING: settlement credit overflows payee balance past u64; entry written, balance() then errors forever"]
#[tokio::test]
async fn credit_that_overflows_payee_balance_is_refused_before_the_entry_is_written() {
    let fx = Fixture::new("ar-f-overflow");
    let big = FileJournalLedger::open(fx.dir.join("ledger-max"), &[(PAYER, Currency::USD, 1_000_000), (PAYEE, Currency::USD, u64::MAX)])
        .unwrap();
    let rail = sim(&fx, "ov", SimMode::SettleAfterPolls(0));
    let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
    let mut ctx = fx.exec_ctx();
    ctx.ledger = &big;
    let s = submit_via_rail(a, &params(), &ctx, &rail).await.unwrap();
    let _ = finalize_via_rail(&s.effect_id, &fx.claims, &big, &rail);
    assert!(
        big.balance(PAYEE, Currency::USD).is_ok(),
        "ledger wrote an entry that makes balance() permanently unavailable: {:?}",
        big.balance(PAYEE, Currency::USD)
    );
}

/// FINDING G7: when the ledger is unavailable at hold time the claim is moved to
/// UnknownOutcome (correct) but `RailSubmission.standing` reports `Refused`, so a
/// caller trusting `standing` believes re-admission is possible while the claim
/// blocks it as OUTCOME_UNKNOWN.
#[ignore = "FINDING: submit_via_rail reports standing=Refused while claim is UnknownOutcome (hold_of unavailable)"]
#[tokio::test]
async fn standing_matches_claim_state_when_ledger_is_unavailable_at_hold() {
    let fx = Fixture::new("ar-f-standing");
    let rail = sim(&fx, "sd", SimMode::Honest);
    std::fs::write(fx.dir.join("ledger").join("deadbeef.hold.json"), b"garbage").unwrap();
    let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    let s = submit_via_rail(a, &params(), &fx.exec_ctx(), &rail).await.unwrap();
    let st = claim_state(&fx.claims, &d);
    let consistent = matches!(
        (s.standing, st),
        (RailStandingAfterSubmit::Refused, ClaimState::Refused)
            | (RailStandingAfterSubmit::UnknownOutcome, ClaimState::UnknownOutcome)
            | (RailStandingAfterSubmit::Submitted, ClaimState::Submitted)
    );
    assert!(consistent, "standing {:?} but claim {:?}", s.standing, st);
}

/// FINDING G8 (low): benign race. Two finalizers on separate handles: the loser
/// completes the idempotent ledger step, then its claim transition fails with
/// UNEXPECTED_STATE_Final and the call returns Err although the effect is Final.
#[ignore = "FINDING: racing finalize returns spurious Err (CLAIM_STORE_FAILED:UNEXPECTED_STATE_Final) for an already-Final claim"]
#[tokio::test]
async fn racing_finalize_never_returns_an_error_for_an_effect_that_ends_final() {
    let mut worst = 0usize;
    for round in 0..12 {
        let fx = Fixture::new(&format!("ar-f-race-{round}"));
        let rail = sim(&fx, "rr", SimMode::Honest);
        let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
        for _ in 0..2 {
            rail.status(&s.correlation_id).unwrap();
        }
        let barrier = Barrier::new(8);
        let errs: usize = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..8)
                .map(|_| {
                    let (fx, d, barrier) = (&fx, s.effect_id.clone(), &barrier);
                    sc.spawn(move || {
                        let claims = ClaimStore::open(fx.dir.join("claims")).unwrap();
                        let ledger = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap();
                        let rail = SimRail::open(&fx.dir.join("rail-rr"), SimMode::Honest).unwrap();
                        barrier.wait();
                        usize::from(finalize_via_rail(&d, &claims, &ledger, &rail).is_err())
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).sum()
        });
        worst = worst.max(errs);
    }
    assert_eq!(worst, 0, "{worst} finalize calls returned Err on a claim that ended Final");
}
