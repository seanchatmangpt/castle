mod common;
use common::payments::*;

use castle::payments::*;
use castle::sa2a_security::PreparedEffect;

const OPENING: u64 = 1_000_000;

async fn settle(fx: &Fixture, amount: &str, obl: &str, nonce: &str) -> String {
    let a = fx.admit(fx.effect(amount, obl), nonce).expect("admits");
    let d = a.effect().digest().to_string();
    let ex = execute_payment(a, &fx.exec_ctx()).await.expect("executes");
    assert_eq!(ex.standing, PaymentStanding::Settled, "{}", ex.detail);
    d
}

fn reversal(payer: &str, payee: &str, amount: &str, obl: &str, original: &str) -> PreparedEffect {
    PaymentEffect::prepare(PRINCIPAL, payer, payee, amount, Currency::USD, obl, "reversal", Some(original)).unwrap()
}

/// Correct-direction reversal of a PAYER->PAYEE original.
fn rev(amount: &str, obl: &str, original: &str) -> PreparedEffect {
    reversal(PAYEE, PAYER, amount, obl, original)
}

fn bal(fx: &Fixture, acct: &str) -> u64 {
    fx.ledger.balance(acct, Currency::USD).unwrap()
}

#[tokio::test]
async fn full_reversal_returns_balances_to_opening_and_ledger_conserves() {
    let fx = Fixture::new("rev-full");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING - 1000, 1000));

    let a = fx.admit(rev("1000", "rev-1", &orig), "n2").expect("reversal admits");
    let rd = a.effect().digest().to_string();
    let ex = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(ex.standing, PaymentStanding::Settled, "{}", ex.detail);

    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING, 0));
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(fx.ledger.entries().unwrap().len(), 2);
    let claim = fx.claims.get(&rd).unwrap().unwrap();
    assert_eq!((claim.state, claim.reverses.as_deref()), (ClaimState::Executed, Some(orig.as_str())));
    assert_eq!(fx.claims.reversed_total(&orig).unwrap(), 1000);
}

#[tokio::test]
async fn reversal_does_not_consume_epoch_budget() {
    // epoch cap exactly fits the original; a reversal must still admit and a
    // fresh payment must still be refused by the (unchanged) budget.
    let fx = Fixture::with("rev-budget", OPENING, 500_000, 1000);
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    assert!(fx.admit(rev("1000", "rev-1", &orig), "n2").is_ok());
    assert_eq!(fx.admit(fx.effect("1", "inv-2"), "n3").unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");
}

#[tokio::test]
async fn reversal_of_unknown_digest_is_original_not_executed() {
    let fx = Fixture::new("rev-ghost");
    let ghost = format!("sha256:{}", "ab".repeat(32));
    let e = rev("100", "rev-1", &ghost);
    let d = e.digest().unwrap();
    assert_eq!(fx.admit(e, "n1").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED");
    assert!(fx.claims.get(&d).unwrap().is_none());
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn reversal_of_reserved_but_unexecuted_original_is_refused() {
    let fx = Fixture::new("rev-reserved");
    let a = fx.admit(fx.effect("1000", "inv-1"), "n1").unwrap();
    let orig = a.effect().digest().to_string();
    assert_eq!(fx.admit(rev("1000", "rev-1", &orig), "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED");
    assert_eq!(fx.claims.reversed_total(&orig).unwrap(), 0);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn reversal_of_unknown_outcome_original_is_refused() {
    let fx = Fixture::new("rev-unknown");
    let a = fx.admit(fx.effect("1000", "inv-1"), "n1").unwrap();
    let orig = a.effect().digest().to_string();
    fx.claims
        .transition(&orig, &[ClaimState::Reserved], ClaimState::UnknownOutcome, None, "test: ambiguous DO")
        .unwrap();
    assert_eq!(fx.admit(rev("1000", "rev-1", &orig), "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED");
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn reversal_of_refused_original_is_refused() {
    let fx = Fixture::with("rev-nsf", 50, 500_000, 1_000_000);
    let a = fx.admit(fx.effect("100", "inv-nsf"), "n1").unwrap();
    let orig = a.effect().digest().to_string();
    let ex = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(ex.standing, PaymentStanding::Refused);
    assert_eq!(fx.admit(rev("100", "rev-1", &orig), "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_ORIGINAL_NOT_EXECUTED");
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[tokio::test]
async fn reversal_larger_than_original_is_refused() {
    let fx = Fixture::new("rev-over");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let e = rev("1001", "rev-1", &orig);
    let d = e.digest().unwrap();
    assert_eq!(fx.admit(e, "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL");
    assert!(fx.claims.get(&d).unwrap().is_none());
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING - 1000, 1000));
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
}

#[tokio::test]
async fn two_partial_reversals_summing_over_original_second_is_refused() {
    let fx = Fixture::new("rev-partial");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;

    let a = fx.admit(rev("600", "rev-a", &orig), "n2").expect("first partial admits");
    let ex = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(ex.standing, PaymentStanding::Settled, "{}", ex.detail);
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING - 400, 400));

    let b = rev("500", "rev-b", &orig);
    let bd = b.digest().unwrap();
    assert_eq!(fx.admit(b, "n3").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL");
    assert!(fx.claims.get(&bd).unwrap().is_none());
    assert_eq!(fx.claims.reversed_total(&orig).unwrap(), 600);

    // The remaining 400 is still reversible, exactly.
    let c = fx.admit(rev("400", "rev-c", &orig), "n4").expect("remainder admits");
    assert_eq!(execute_payment(c, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING, 0));
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(fx.admit(rev("1", "rev-d", &orig), "n5").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL");
}

#[tokio::test]
async fn in_flight_reversal_reserves_original_capacity() {
    let fx = Fixture::new("rev-inflight");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let _held = fx.admit(rev("700", "rev-a", &orig), "n2").expect("admitted, not executed");
    assert_eq!(fx.admit(rev("400", "rev-b", &orig), "n3").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_EXCEEDS_ORIGINAL");
}

#[tokio::test]
async fn reversal_in_the_original_direction_is_a_mismatch() {
    let fx = Fixture::new("rev-dir");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let e = reversal(PAYER, PAYEE, "500", "rev-1", &orig);
    let d = e.digest().unwrap();
    assert_eq!(fx.admit(e, "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_MISMATCH");
    assert!(fx.claims.get(&d).unwrap().is_none());
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING - 1000, 1000));
}

#[tokio::test]
async fn reversal_in_uncapped_currency_is_refused_by_policy_first() {
    let fx = Fixture::new("rev-cur");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let e = PaymentEffect::prepare(PRINCIPAL, PAYEE, PAYER, "100", Currency::EUR, "rev-1", "reversal", Some(&orig)).unwrap();
    // EUR has no cap in the fixture policy, so static policy refuses first.
    assert_eq!(fx.admit(e, "n2").unwrap_err(), "REFUSED:PAYMENT_CURRENCY_UNSUPPORTED");
}

#[tokio::test]
async fn reversing_a_reversal_is_a_mismatch() {
    let fx = Fixture::new("rev-rev");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let a = fx.admit(rev("1000", "rev-1", &orig), "n2").unwrap();
    let rdig = a.effect().digest().to_string();
    assert_eq!(execute_payment(a, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);

    // Swap back to the original direction, claiming to reverse the reversal.
    let e = reversal(PAYER, PAYEE, "1000", "rev-2", &rdig);
    let d = e.digest().unwrap();
    assert_eq!(fx.admit(e, "n3").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_MISMATCH");
    assert!(fx.claims.get(&d).unwrap().is_none());
    assert_eq!((bal(&fx, PAYER), bal(&fx, PAYEE)), (OPENING, 0));
    assert_eq!(fx.ledger.entries().unwrap().len(), 2);
}

#[tokio::test]
async fn reversal_by_a_different_principal_is_a_mismatch() {
    let mut fx = Fixture::new("rev-princ");
    let orig = settle(&fx, "1000", "inv-1", "n1").await;
    let mut other = fx.policy.principals[PRINCIPAL].clone();
    other.epoch_cap.insert(Currency::USD, 2_000_000);
    fx.policy.principals.insert("principal:other".into(), other);
    let e = PaymentEffect::prepare("principal:other", PAYEE, PAYER, "100", Currency::USD, "rev-1", "reversal", Some(&orig)).unwrap();
    assert_eq!(fx.admit(e, "n2").unwrap_err(), "REFUSED:PAYMENT_REVERSAL_MISMATCH");
}
