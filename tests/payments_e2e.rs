mod common;
use common::payments::*;

use castle::payments::*;

#[tokio::test]
async fn admitted_payment_settles_end_to_end_with_durable_brce_and_receipts() {
    let fx = Fixture::new("e2e");
    let effect = fx.effect("470000", "invoice-9821");
    let admission = fx.admit(effect, "n-1").expect("admits");
    let digest = admission.effect().digest().to_string();

    let exec = execute_payment(admission, &fx.exec_ctx()).await.expect("executes");

    assert_eq!(exec.standing, PaymentStanding::Settled, "{}", exec.detail);
    assert_eq!(exec.brce_prepare_receipt_digests.len(), 2);
    assert_eq!(exec.brce_outcome_receipt_digests.len(), 2);
    assert!(exec.ocel_receipt_digest.is_some());
    let entry = exec.ledger_entry.expect("ledger entry");
    assert_eq!((entry.amount_minor, entry.debit_account.as_str(), entry.credit_account.as_str()), (470_000, PAYER, PAYEE));

    // Final state, not interactions: balances, claim, conservation.
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 530_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 470_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    let claim = fx.claims.get(&digest).unwrap().unwrap();
    assert_eq!(claim.state, ClaimState::Executed);
    assert_eq!(claim.construct_digest, exec.construct_digest);
}

#[tokio::test]
async fn resubmitting_a_settled_effect_with_a_fresh_certificate_cannot_settle_twice() {
    let fx = Fixture::new("dup");
    let a = fx.admit(fx.effect("100", "inv-1"), "n-1").unwrap();
    assert_eq!(execute_payment(a, &fx.exec_ctx()).await.unwrap().standing, PaymentStanding::Settled);

    let again = fx.admit(fx.effect("100", "inv-1"), "n-2");
    assert_eq!(again.unwrap_err(), "REFUSED:PAYMENT_ALREADY_SETTLED");
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100);
}

#[tokio::test]
async fn insufficient_funds_is_a_definite_refusal_that_releases_budget() {
    let fx = Fixture::with("nsf", 50, 500_000, 1_000);
    let a = fx.admit(fx.effect("100", "inv-nsf"), "n-1").unwrap();
    let exec = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Refused);
    assert!(exec.ledger_entry.is_none());
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 50);
    // Budget released: the full epoch cap is available again.
    assert!(fx.admit(fx.effect("1000", "inv-other"), "n-2").is_ok());
}
