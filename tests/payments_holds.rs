mod common;
use common::payments::*;

#[path = "common/scripted_rail.rs"]
mod scripted_rail;
use scripted_rail::*;

use castle::payments::execute_rail::finalize_via_rail;
use castle::payments::ledger::LedgerHold;
use castle::payments::*;

/// Admit, move Reserved -> Submitted (as the rail lane would), return admission.
pub fn admit_submitted(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) -> PaymentAdmission {
    let admission = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    fx.claims
        .transition(admission.effect().digest(), &[ClaimState::Reserved], ClaimState::Submitted, None, "submitted")
        .expect("submitted");
    admission
}

#[test]
fn hold_reduces_available_not_balance() {
    let fx = Fixture::new("hold-avail");
    let a = admit_submitted(&fx, "400000", "inv-1", "n-1");
    let hold = fx.ledger.hold(&a, &fx.token_for(&a)).expect("hold");
    assert_eq!((hold.amount_minor, hold.account.as_str()), (400_000, PAYER));
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 600_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(fx.ledger.hold_of(a.effect().digest()).unwrap(), Some(hold));
}

#[test]
fn two_effects_cannot_both_hold_funds_covering_only_one() {
    let fx = Fixture::with("hold-two", 500_000, 500_000, 5_000_000);
    let a = admit_submitted(&fx, "400000", "inv-a", "n-a");
    let b = admit_submitted(&fx, "400000", "inv-b", "n-b");
    fx.ledger.hold(&a, &fx.token_for(&a)).expect("first hold");
    assert_eq!(fx.ledger.hold(&b, &fx.token_for(&b)).unwrap_err(), LedgerError::InsufficientFunds);
    assert_eq!(fx.ledger.hold_of(b.effect().digest()).unwrap(), None);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);
}

#[test]
fn hold_is_idempotent_per_digest() {
    let fx = Fixture::new("hold-idem");
    let a = admit_submitted(&fx, "300000", "inv-1", "n-1");
    let t = fx.token_for(&a);
    let h1 = fx.ledger.hold(&a, &t).unwrap();
    let h2 = fx.ledger.hold(&a, &t).unwrap();
    assert_eq!(h1, h2);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 700_000);
}

#[test]
fn release_frees_funds_and_allows_a_fresh_hold() {
    let fx = Fixture::with("hold-release", 500_000, 500_000, 5_000_000);
    let a = admit_submitted(&fx, "400000", "inv-a", "n-a");
    fx.ledger.hold(&a, &fx.token_for(&a)).unwrap();
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);

    let digest = a.effect().digest().to_string();
    finalize_via_rail(&digest, &fx.claims, &fx.ledger, &ScriptedRail::new(vec![rejected()])).unwrap();
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 500_000);
    assert_eq!(fx.ledger.hold_of(&digest).unwrap(), None);

    let b = admit_submitted(&fx, "400000", "inv-b", "n-b");
    fx.ledger.hold(&b, &fx.token_for(&b)).expect("funds were freed");
}

#[test]
fn post_respects_other_digests_holds_but_converts_its_own() {
    let fx = Fixture::with("hold-post", 500_000, 500_000, 5_000_000);
    let a = admit_submitted(&fx, "400000", "inv-a", "n-a");
    let b = admit_submitted(&fx, "300000", "inv-b", "n-b");
    fx.ledger.hold(&a, &fx.token_for(&a)).unwrap();
    // b's post would dip into a's held funds.
    assert_eq!(fx.ledger.post(&b, &fx.token_for(&b)).unwrap_err(), LedgerError::InsufficientFunds);
    // a's own post converts cleanly.
    let e = fx.ledger.post(&a, &fx.token_for(&a)).unwrap();
    assert_eq!(e.amount_minor, 400_000);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);
    assert_eq!(fx.ledger.hold_of(a.effect().digest()).unwrap(), None);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[test]
fn concurrent_holds_over_separate_handles_never_overdraw() {
    let fx = Fixture::with("hold-conc", 1_000_000, 500_000, 50_000_000);
    let admissions: Vec<PaymentAdmission> =
        (0..8).map(|i| admit_submitted(&fx, "300000", &format!("inv-{i}"), &format!("n-{i}"))).collect();
    let tokens: Vec<ActuationToken> = admissions.iter().map(|a| fx.token_for(a)).collect();
    let root = fx.dir.join("ledger");

    let results: Vec<Result<LedgerHold, LedgerError>> = std::thread::scope(|s| {
        let handles: Vec<_> = admissions
            .iter()
            .zip(tokens.iter())
            .map(|(a, t)| {
                let root = root.clone();
                s.spawn(move || {
                    let handle = FileJournalLedger::open(root, &[]).expect("open handle");
                    handle.hold(a, t)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("thread")).collect()
    });

    let ok = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(ok, 3, "1_000_000 covers exactly three 300_000 holds: {results:?}");
    assert!(results.iter().filter(|r| r.is_err()).all(|r| *r == Err(LedgerError::InsufficientFunds)));
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
}

#[test]
fn hold_that_would_overflow_the_payee_balance_is_refused_typed() {
    // G6: a credit past u64::MAX would brick balance() for the payee.
    let fx = Fixture::new("hold-overflow");
    let big = FileJournalLedger::open(
        fx.dir.join("ledger-max"),
        &[(PAYER, Currency::USD, 1_000_000), (PAYEE, Currency::USD, u64::MAX)],
    )
    .unwrap();
    let a = admit_submitted(&fx, "400000", "inv-ov", "n-ov");
    let err = big.hold(&a, &fx.token_for(&a)).unwrap_err();
    assert_eq!(err, LedgerError::Unavailable("CREDIT_OVERFLOW".into()));
    assert_eq!(big.balance(PAYEE, Currency::USD).unwrap(), u64::MAX);
    assert!(big.entries().unwrap().is_empty());
}

#[test]
fn return_cannot_claw_back_funds_the_payee_holds_and_stays_retryable() {
    // G5 at ledger level: settle_return refuses while the payee's available < amount.
    let fx = Fixture::with("hold-return", 1_000_000, 500_000, 100_000_000);
    let a = admit_submitted(&fx, "400000", "inv-A", "n-A");
    let d = a.effect().digest().to_string();
    fx.ledger.hold(&a, &fx.token_for(&a)).unwrap();
    let rail = ScriptedRail::new(vec![settled(), returned()]);
    finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap(); // payee owns 400k
    // Payee reserves its whole balance for its own outbound payment.
    let eff_b = PaymentEffect::prepare(PRINCIPAL, PAYEE, PAYER, "400000", Currency::USD, "inv-B", "invoice-payment", None).unwrap();
    let b = fx.admit(eff_b, "n-B").unwrap();
    fx.ledger.hold(&b, &fx.token_for(&b)).unwrap();
    assert_eq!(fx.ledger.available(PAYEE, Currency::USD).unwrap(), 0);

    let err = finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap_err();
    assert_eq!(err, "REFUSED:PAYMENT_INSUFFICIENT_FUNDS", "{err}");
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Final, "claim stays Final");
    assert!(fx.ledger.returns().unwrap().is_empty());
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);

    // Retryable: once the hold is gone the same return converges.
    finalize_via_rail(b.effect().digest(), &fx.claims, &fx.ledger, &ScriptedRail::new(vec![rejected()])).unwrap();
    finalize_via_rail(&d, &fx.claims, &fx.ledger, &rail).unwrap();
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Returned);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
}
