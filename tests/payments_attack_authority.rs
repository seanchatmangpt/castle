//! Authority skeptic lane. Tests assert the SECURE behavior; a test marked
//! `#[ignore = "FINDING: ..."]` fails today and demonstrates a real defect
//! (run with `--ignored`). Real collaborators only, no mocks.
mod common;
use common::payments::*;

use castle::payments::*;

/// FIXED (was FINDING-1): `LedgerPort::post` now requires an `ActuationToken`, mintable only
/// from a genuine castle `ConstructAdmission`; the recorded construct digest is that token's.
#[test]
fn ledger_post_requires_a_real_construct_admission_token() {
    let fx = Fixture::new("atk-direct-post");
    let adm = fx.admit(fx.effect("100", "inv-direct"), "n-1").unwrap();
    let tok = fx.token_for(&adm);
    let entry = fx.ledger.post(&adm, &tok).unwrap();
    assert_eq!(entry.construct_digest, tok.construct_digest());
    assert_ne!(entry.construct_digest, "forged-construct-digest");
    // No struct-literal / alternate constructor for the token outside its module.
    let root = env!("CARGO_MANIFEST_DIR");
    for f in ["admission", "adapter", "execute", "claim_store", "reconcile", "experience", "iso20022"] {
        let src = std::fs::read_to_string(format!("{root}/src/payments/{f}.rs")).unwrap();
        assert!(!src.contains("ActuationToken {"), "{f}.rs must not fabricate a token");
    }
}

/// FIXED (was FINDING-2): `PaymentAdmission` is no longer `Clone`, and `execute_payment`
/// requires the admission's claim to still be `Reserved`. A claim moved out of `Reserved`
/// (here: refused/released) makes the admission inert: no value moves.
#[tokio::test]
async fn admission_whose_claim_left_reserved_cannot_actuate() {
    let fx = Fixture::new("atk-stale");
    let adm = fx.admit(fx.effect("100", "inv-stale"), "n-1").unwrap();
    let d = adm.effect().digest().to_string();
    fx.claims.transition(&d, &[ClaimState::Reserved], ClaimState::Refused, None, "released").unwrap();
    let r = execute_payment(adm, &fx.exec_ctx()).await;
    assert_eq!(r.unwrap_err(), "REFUSED:PAYMENT_CLAIM_NOT_RESERVED");
    assert!(fx.ledger.entries().unwrap().is_empty());
}

/// FINDING-2b: same root cause; claim store on the execute side is caller-supplied and
/// not tied to the admission, so DO runs before the claim transition can fail.
#[tokio::test]
async fn execute_with_foreign_claim_store_moves_no_value() {
    let fx = Fixture::new("atk-foreign");
    let adm = fx.admit(fx.effect("100", "inv-foreign"), "n-1").unwrap();
    let other = ClaimStore::open(fx.dir.join("other-claims")).unwrap();
    let mut ctx = fx.exec_ctx();
    ctx.claims = &other;
    let r = execute_payment(adm, &ctx).await;
    assert_eq!(r.as_ref().unwrap_err(), "REFUSED:PAYMENT_CLAIM_NOT_RESERVED");
    assert!(fx.ledger.entries().unwrap().is_empty(), "ledger moved value although claim transition failed: {r:?}");
}

/// FINDING-3: reversal cap check (`check_reversal`) runs outside the ClaimStore lock, so
/// concurrent reversals can each pass and jointly exceed the original amount.
#[test]
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

/// Evidence: an admission is single-use (moved into `execute_payment`, not `Clone`); after
/// settlement a fresh certificate for the same effect cannot post again.
#[tokio::test]
async fn settled_admission_is_consumed_and_cannot_repost() {
    let fx = Fixture::new("atk-clone-settled");
    let adm = fx.admit(fx.effect("100", "inv-c"), "n-1").unwrap();
    assert_eq!(execute_payment(adm, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);
    assert_eq!(fx.admit(fx.effect("100", "inv-c"), "n-2").unwrap_err(), "REFUSED:PAYMENT_ALREADY_SETTLED");
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
