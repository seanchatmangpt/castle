//! Authority skeptic lane. Tests assert the SECURE behavior; a test marked
//! `#[ignore = "FINDING: ..."]` fails today and demonstrates a real defect
//! (run with `--ignored`). Real collaborators only, no mocks.
mod common;
use common::payments::*;

use castle::payments::*;

/// FINDING-1: `LedgerPort::post` is `pub` and takes only `&PaymentAdmission` plus a
/// caller-chosen `construct_digest` string, so any holder of an admission can write a
/// ledger entry with NO CONSTRUCT admission, NO BRCE journal, and a forged construct digest.
#[test]
#[ignore = "FINDING: ledger.post reachable directly with admission + forged construct digest"]
fn direct_ledger_post_without_construct_chain_is_refused() {
    let fx = Fixture::new("atk-direct-post");
    let adm = fx.admit(fx.effect("100", "inv-direct"), "n-1").unwrap();
    let r = fx.ledger.post(&adm, "forged-construct-digest");
    assert!(r.is_err(), "value moved with no CONSTRUCT/BRCE chain: {r:?}");
    assert!(fx.ledger.entries().unwrap().is_empty());
}

/// FINDING-2: `execute_payment` never checks the claim is still `Reserved` BEFORE DO.
/// A cloned admission whose claim was Refused (budget released) still actuates when the
/// ledger later has funds; the claim transition then fails AFTER value has moved.
#[tokio::test]
#[ignore = "FINDING: stale cloned admission actuates after claim Refused/budget released"]
async fn stale_cloned_admission_cannot_actuate_after_claim_refused() {
    let fx = Fixture::with("atk-stale", 50, 500_000, 1_000);
    let adm = fx.admit(fx.effect("100", "inv-stale"), "n-1").unwrap();
    let clone = adm.clone();
    let first = execute_payment(adm, &fx.exec_ctx()).await.unwrap();
    assert_eq!(first.standing, PaymentStanding::Refused);
    assert_eq!(fx.claims.get(first.effect_digest.as_str()).unwrap().unwrap().state, ClaimState::Refused);

    // Ledger later funded (a different, funded ledger instance stands in for top-up).
    let funded = FileJournalLedger::open(fx.dir.join("ledger2"), &[(PAYER, Currency::USD, 10_000), (PAYEE, Currency::USD, 0)]).unwrap();
    let mut ctx = fx.exec_ctx();
    ctx.ledger = &funded;
    let _ = execute_payment(clone, &ctx).await;
    assert!(funded.entries().unwrap().is_empty(), "value moved under a Refused claim (budget already released)");
}

/// FINDING-2b: same root cause; claim store on the execute side is caller-supplied and
/// not tied to the admission, so DO runs before the claim transition can fail.
#[tokio::test]
#[ignore = "FINDING: execute with foreign ClaimStore moves value then errors"]
async fn execute_with_foreign_claim_store_moves_no_value() {
    let fx = Fixture::new("atk-foreign");
    let adm = fx.admit(fx.effect("100", "inv-foreign"), "n-1").unwrap();
    let other = ClaimStore::open(fx.dir.join("other-claims")).unwrap();
    let mut ctx = fx.exec_ctx();
    ctx.claims = &other;
    let r = execute_payment(adm, &ctx).await;
    assert!(r.is_err());
    assert!(fx.ledger.entries().unwrap().is_empty(), "ledger moved value although claim transition failed: {r:?}");
}

/// FINDING-3: reversal cap check (`check_reversal`) runs outside the ClaimStore lock, so
/// concurrent reversals can each pass and jointly exceed the original amount.
#[test]
#[ignore = "FINDING: check_reversal TOCTOU; concurrent reversals exceed original"]
fn concurrent_reversals_cannot_exceed_original() {
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    for i in 0..40 {
        let fx = Fixture::new(&format!("atk-rev-{i}"));
        let orig = fx.admit(fx.effect("100", "inv-orig"), "n-0").unwrap();
        let od = orig.effect().digest().to_string();
        assert_eq!(rt.block_on(execute_payment(orig, &fx.exec_ctx())).unwrap().standing, PaymentStanding::Settled);
        let mk = |n: &str, ob: &str| {
            let e = PaymentEffect::prepare(PRINCIPAL, PAYEE, PAYER, "60", Currency::USD, ob, "refund", Some(&od)).unwrap();
            let c = fx.cert(&e, n, &["mac", "phone"]);
            (e, c)
        };
        let (e1, c1) = mk("r1", "rev-a");
        let (e2, c2) = mk("r2", "rev-b");
        let barrier = std::sync::Barrier::new(2);
        let (r1, r2) = std::thread::scope(|s| {
            let h1 = s.spawn(|| { barrier.wait(); admit_payment(e1, &c1, &fx.admission_ctx()) });
            let h2 = s.spawn(|| { barrier.wait(); admit_payment(e2, &c2, &fx.admission_ctx()) });
            (h1.join().unwrap(), h2.join().unwrap())
        });
        assert!(!(r1.is_ok() && r2.is_ok()), "iteration {i}: both 60-unit reversals admitted against a 100 original");
    }
}

/// Evidence (passes): a Reserved claim that never executes is not stuck forever;
/// `reconcile` proves absence and releases the budget.
#[test]
fn reserved_never_executed_claim_is_releasable_by_reconcile() {
    let fx = Fixture::with("atk-leak", 1_000_000, 500_000, 100);
    let adm = fx.admit(fx.effect("100", "inv-leak"), "n-1").unwrap();
    let d = adm.effect().digest().to_string();
    assert_eq!(fx.admit(fx.effect("1", "inv-x"), "n-2").unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");
    assert_eq!(reconcile(&d, &fx.claims, &fx.ledger).unwrap(), ReconcileResolution::ProvenAbsent);
    assert!(fx.admit(fx.effect("1", "inv-x"), "n-3").is_ok());
}

/// Evidence (passes): cloned admission of an already-settled effect cannot double-post.
#[tokio::test]
async fn cloned_admission_after_settlement_cannot_double_post() {
    let fx = Fixture::new("atk-clone-settled");
    let adm = fx.admit(fx.effect("100", "inv-c"), "n-1").unwrap();
    let clone = adm.clone();
    assert_eq!(execute_payment(adm, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);
    let _ = execute_payment(clone, &fx.exec_ctx()).await;
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100);
}

/// Evidence (passes): certificate principal must match effect principal; a cert replayed on a
/// different effect is refused.
#[test]
fn certificate_reuse_across_effects_is_refused() {
    let fx = Fixture::new("atk-certreuse");
    let e1 = fx.effect("100", "inv-1");
    let cert = fx.cert(&e1, "n-1", &["mac", "phone"]);
    let e2 = fx.effect("200", "inv-2");
    assert!(admit_payment(e2, &cert, &fx.admission_ctx()).is_err());
}
