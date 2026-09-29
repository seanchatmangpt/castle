mod common;
use common::payments::*;

use castle::payments::reconcile::{reconcile, ReconcileResolution};
use castle::payments::*;

/// Genuine failing LedgerPort delegating to the real FileJournalLedger.
/// `write_first`: post really lands in the ledger, then reports Unavailable.
/// otherwise: post fails before writing anything.
struct FlakyPost<'a> {
    inner: &'a FileJournalLedger,
    write_first: bool,
}
impl LedgerPort for FlakyPost<'_> {
    fn balance(&self, a: &str, c: Currency) -> Result<u64, LedgerError> {
        self.inner.balance(a, c)
    }
    fn lookup(&self, d: &str) -> Result<Option<LedgerEntry>, LedgerError> {
        self.inner.lookup(d)
    }
    fn post(&self, adm: &PaymentAdmission, cd: &str) -> Result<LedgerEntry, LedgerError> {
        if self.write_first {
            self.inner.post(adm, cd)?;
        }
        Err(LedgerError::Unavailable("connection reset after commit".into()))
    }
}

/// Ledger that is entirely unreachable for lookup.
struct Down;
impl LedgerPort for Down {
    fn balance(&self, _: &str, _: Currency) -> Result<u64, LedgerError> {
        Err(LedgerError::Unavailable("down".into()))
    }
    fn lookup(&self, _: &str) -> Result<Option<LedgerEntry>, LedgerError> {
        Err(LedgerError::Unavailable("down".into()))
    }
    fn post(&self, _: &PaymentAdmission, _: &str) -> Result<LedgerEntry, LedgerError> {
        Err(LedgerError::Unavailable("down".into()))
    }
}

async fn run_flaky(fx: &Fixture, write_first: bool, nonce: &str) -> (String, PaymentStanding) {
    let admission = fx.admit(fx.effect("100", "inv-r"), nonce).unwrap();
    let digest = admission.effect().digest().to_string();
    let flaky = FlakyPost { inner: &fx.ledger, write_first };
    let mut ctx = fx.exec_ctx();
    ctx.ledger = &flaky;
    let exec = execute_payment(admission, &ctx).await.unwrap();
    (digest, exec.standing)
}

#[tokio::test]
async fn crash_after_post_is_unknown_then_reconciles_to_settled_once() {
    let fx = Fixture::new("rec-post");
    let (digest, standing) = run_flaky(&fx, true, "n-1").await;
    assert_eq!(standing, PaymentStanding::UnknownOutcome);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::UnknownOutcome);

    // No blind retry.
    assert_eq!(fx.admit(fx.effect("100", "inv-r"), "n-2").unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");

    // Unreachable ledger: still unknown, nothing changes.
    assert_eq!(reconcile(&digest, &fx.claims, &Down).unwrap(), ReconcileResolution::StillUnknown);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::UnknownOutcome);

    match reconcile(&digest, &fx.claims, &fx.ledger).unwrap() {
        ReconcileResolution::Settled(e) => assert_eq!(e.amount_minor, 100),
        other => panic!("expected Settled, got {other:?}"),
    }
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::Executed);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn proven_absent_refuses_claim_and_fresh_admission_settles_exactly_once() {
    let fx = Fixture::new("rec-absent");
    let (digest, standing) = run_flaky(&fx, false, "n-1").await;
    assert_eq!(standing, PaymentStanding::UnknownOutcome);
    assert_eq!(fx.ledger.entries().unwrap().len(), 0);

    assert_eq!(reconcile(&digest, &fx.claims, &fx.ledger).unwrap(), ReconcileResolution::ProvenAbsent);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::Refused);

    let a = fx.admit(fx.effect("100", "inv-r"), "n-2").expect("re-admission allowed after proven absence");
    let exec = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled, "{}", exec.detail);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::Executed);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100);
}

#[tokio::test]
async fn tampered_ledger_entry_is_reconciliation_drift() {
    let fx = Fixture::new("rec-drift");
    let (digest, _) = run_flaky(&fx, true, "n-1").await;
    let path = fx.dir.join("ledger").join(format!("{}.entry.json", digest.trim_start_matches("sha256:")));
    let mut v: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    v["amount_minor"] = serde_json::json!(999);
    std::fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();

    assert_eq!(reconcile(&digest, &fx.claims, &fx.ledger).unwrap_err(), "REFUSED:RECONCILIATION_DRIFT");
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::UnknownOutcome);
}

#[tokio::test]
async fn executed_claim_is_not_reconcilable() {
    let fx = Fixture::new("rec-exec");
    let a = fx.admit(fx.effect("100", "inv-x"), "n-1").unwrap();
    let digest = a.effect().digest().to_string();
    assert_eq!(execute_payment(a, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);
    assert_eq!(reconcile(&digest, &fx.claims, &fx.ledger).unwrap_err(), "REFUSED:PAYMENT_NOT_RECONCILABLE");
    assert_eq!(reconcile("sha256:nope", &fx.claims, &fx.ledger).unwrap_err(), "REFUSED:PAYMENT_NOT_RECONCILABLE");
}
