mod common;
use common::payments::*;

#[path = "common/scripted_rail.rs"]
mod scripted_rail;
use scripted_rail::*;

use castle::payments::execute_rail::{finalize_via_rail, FinalizeResult};
use castle::payments::settlement::*;
use castle::payments::*;

fn submitted_held(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) -> String {
    let a = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    let digest = a.effect().digest().to_string();
    fx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Submitted, None, "submitted").unwrap();
    fx.ledger.hold(&a, &fx.token_for(&a)).expect("hold");
    digest
}

/// Drive one rail report through the only public path: a real (scripted) rail polled by
/// `finalize_via_rail`. Returns the applied outcome.
fn fin(fx: &Fixture, rail: &ScriptedRail, d: &str) -> Result<FinalityOutcome, String> {
    match finalize_via_rail(d, &fx.claims, &fx.ledger, rail)? {
        FinalizeResult::Applied { outcome, .. } => Ok(outcome),
        other => Err(format!("not applied: {other:?}")),
    }
}

fn state(fx: &Fixture, d: &str) -> ClaimState {
    fx.claims.get(d).unwrap().unwrap().state
}

#[test]
fn final_evidence_settles_the_hold_into_exactly_one_entry() {
    let fx = Fixture::new("s-final");
    let d = submitted_held(&fx, "250000", "inv-1", "n-1");
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0, "no settlement before observation");

    let rail = ScriptedRail::new(vec![settled()]);
    let out = fin(&fx, &rail, &d).unwrap();
    let FinalityOutcome::Settled(entry) = out else { panic!("expected Settled") };
    assert_eq!((entry.amount_minor, entry.debit_account.as_str(), entry.credit_account.as_str()), (250_000, PAYER, PAYEE));
    assert_eq!(state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 750_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 250_000);
    assert_eq!(fx.ledger.hold_of(&d).unwrap(), None);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 750_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[test]
fn duplicate_final_evidence_is_already_final_with_one_entry() {
    let fx = Fixture::new("s-dup");
    let d = submitted_held(&fx, "100000", "inv-1", "n-1");
    let rail = ScriptedRail::new(vec![settled()]);
    fin(&fx, &rail, &d).unwrap();
    let again = fin(&fx, &rail, &d).unwrap();
    assert_eq!(again, FinalityOutcome::AlreadyFinal);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100_000);
    assert_eq!(state(&fx, &d), ClaimState::Final);
}

#[test]
fn unknown_outcome_claim_is_resolved_by_final_evidence() {
    let fx = Fixture::new("s-unk");
    let d = submitted_held(&fx, "100000", "inv-1", "n-1");
    fx.claims.transition(&d, &[ClaimState::Submitted], ClaimState::UnknownOutcome, None, "timeout").unwrap();
    let rail = ScriptedRail::new(vec![settled()]);
    assert!(matches!(fin(&fx, &rail, &d).unwrap(), FinalityOutcome::Settled(_)));
    assert_eq!(state(&fx, &d), ClaimState::Final);
}

#[test]
fn final_without_a_hold_is_refused_not_held() {
    let fx = Fixture::new("s-nohold");
    let a = fx.admit(fx.effect("100000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    fx.claims.transition(&d, &[ClaimState::Reserved], ClaimState::Submitted, None, "submitted").unwrap();
    let err = fin(&fx, &ScriptedRail::new(vec![settled()]), &d).unwrap_err();
    assert_eq!(err, PAYMENT_NOT_HELD);
    assert_eq!(state(&fx, &d), ClaimState::Submitted);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[test]
fn settled_report_on_a_reserved_claim_first_moves_it_through_submitted() {
    // G2: the rail has seen an effect whose claim is still Reserved (crash after rail
    // accept): finalize records "rail-observed" (Reserved->Submitted) and then converges.
    let fx = Fixture::new("s-reserved");
    let a = fx.admit(fx.effect("100000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    fx.ledger.hold(&a, &fx.token_for(&a)).unwrap();
    assert_eq!(state(&fx, &d), ClaimState::Reserved);
    let out = fin(&fx, &ScriptedRail::new(vec![settled()]), &d).unwrap();
    assert!(matches!(out, FinalityOutcome::Settled(_)));
    assert_eq!(state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
}

#[test]
fn apply_rail_report_refuses_unknown_claims_and_non_terminal_reports() {
    let fx = Fixture::new("s-report-guards");
    let phantom = "sha256:".to_string() + &"ab".repeat(32);
    let report = observe_rail(&ScriptedRail::new(vec![settled()]), &phantom).unwrap();
    assert_eq!(apply_rail_report(&fx.claims, &fx.ledger, &report).unwrap_err(), PAYMENT_NOT_SUBMITTED);
    let d = submitted_held(&fx, "100000", "inv-1", "n-1");
    let pending = observe_rail(&ScriptedRail::new(vec![RailStatus::Accepted]), &d).unwrap();
    assert_eq!(apply_rail_report(&fx.claims, &fx.ledger, &pending).unwrap_err(), REPORT_NOT_TERMINAL);
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(state(&fx, &d), ClaimState::Submitted);
}

#[test]
fn conflicting_and_reordered_reports_are_refused_with_no_state_change() {
    let fx = Fixture::new("s-conflict");
    // Rejected after Final.
    let d1 = submitted_held(&fx, "100000", "inv-1", "n-1");
    fin(&fx, &ScriptedRail::new(vec![settled()]), &d1).unwrap();
    assert_eq!(fin(&fx, &ScriptedRail::new(vec![rejected()]), &d1).unwrap_err(), EVIDENCE_CONFLICT);
    assert_eq!(state(&fx, &d1), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100_000);

    // Final after Rejected.
    let d2 = submitted_held(&fx, "200000", "inv-2", "n-2");
    fin(&fx, &ScriptedRail::new(vec![rejected()]), &d2).unwrap();
    assert_eq!(fin(&fx, &ScriptedRail::new(vec![settled()]), &d2).unwrap_err(), EVIDENCE_CONFLICT);
    assert_eq!(state(&fx, &d2), ClaimState::Refused);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[test]
fn return_report_on_a_submitted_claim_settles_implied_then_returns() {
    let fx = Fixture::new("s-return-implied");
    let d = submitted_held(&fx, "100000", "inv-1", "n-1");
    let out = fin(&fx, &ScriptedRail::new(vec![returned()]), &d).unwrap();
    assert!(matches!(out, FinalityOutcome::Returned(_)));
    assert_eq!(state(&fx, &d), ClaimState::Returned);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[test]
fn rejection_releases_hold_and_budget_and_allows_fresh_admission() {
    // epoch cap 500_000 fits exactly one 400_000 payment.
    let fx = Fixture::with("s-reject", 1_000_000, 500_000, 500_000);
    let d = submitted_held(&fx, "400000", "inv-1", "n-1");
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 600_000);
    // Budget is consumed while Submitted.
    assert_eq!(fx.admit(fx.effect("400000", "inv-2"), "n-2").unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");

    let out = fin(&fx, &ScriptedRail::new(vec![rejected()]), &d).unwrap();
    assert_eq!(out, FinalityOutcome::Released);
    let claim = fx.claims.get(&d).unwrap().unwrap();
    assert_eq!(claim.state, ClaimState::Refused);
    assert!(claim.detail.contains("rejected:"), "{}", claim.detail);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.entries().unwrap().is_empty());

    let fresh = fx.admit(fx.effect("400000", "inv-2"), "n-3").expect("budget released");
    fx.ledger.hold(&fresh, &fx.token_for(&fresh)).expect("funds free again");
}

#[test]
fn return_after_final_posts_inverse_entry_and_restores_balances() {
    let fx = Fixture::new("s-return");
    let d = submitted_held(&fx, "300000", "inv-1", "n-1");
    fin(&fx, &ScriptedRail::new(vec![settled()]), &d).unwrap();

    let rail = ScriptedRail::new(vec![returned()]);
    let out = fin(&fx, &rail, &d).unwrap();
    let FinalityOutcome::Returned(inv) = out else { panic!("expected Returned") };
    assert_eq!((inv.debit_account.as_str(), inv.credit_account.as_str(), inv.amount_minor), (PAYEE, PAYER, 300_000));
    assert_eq!(state(&fx, &d), ClaimState::Returned);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());

    // Idempotent: replaying the return adds nothing.
    fin(&fx, &rail, &d).unwrap();
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);

    // Returned counts as settled: same obligation cannot be re-paid.
    assert_eq!(fx.admit(fx.effect("300000", "inv-1"), "n-9").unwrap_err(), "REFUSED:PAYMENT_ALREADY_SETTLED");
}

#[test]
fn claim_store_reserve_maps_new_states() {
    let fx = Fixture::new("s-reserve");
    let d = submitted_held(&fx, "100000", "inv-1", "n-1");
    assert_eq!(fx.admit(fx.effect("100000", "inv-1"), "n-2").unwrap_err(), "REFUSED:PAYMENT_IN_FLIGHT");
    fin(&fx, &ScriptedRail::new(vec![settled()]), &d).unwrap();
    assert_eq!(fx.admit(fx.effect("100000", "inv-1"), "n-3").unwrap_err(), "REFUSED:PAYMENT_ALREADY_SETTLED");
}
