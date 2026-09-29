mod common;
use common::payments::*;

use castle::payments::execute_rail::*;
use castle::payments::rail::{RailActuator, RailInstruction};
#[path = "common/scripted_rail.rs"]
mod scripted_rail;

use castle::payments::settlement::EVIDENCE_CONFLICT;
use castle::payments::*;

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

fn params(expires_at_ms: u64) -> RailExecutionParams {
    RailExecutionParams {
        bindings: bindings(expires_at_ms),
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

async fn submit(fx: &Fixture, rail: &SimRail, amount: &str, obligation: &str, nonce: &str) -> RailSubmission {
    let a = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    submit_via_rail(a, &params(100_000), &fx.exec_ctx(), rail).await.expect("submit_via_rail")
}

fn finalize(fx: &Fixture, rail: &SimRail, digest: &str) -> FinalizeResult {
    finalize_via_rail(digest, &fx.claims, &fx.ledger, rail).expect("finalize")
}

/// Poll until the first non-Pending result.
fn poll_terminal(fx: &Fixture, rail: &SimRail, digest: &str) -> FinalizeResult {
    for _ in 0..12 {
        let r = finalize(fx, rail, digest);
        if r != FinalizeResult::Pending {
            return r;
        }
    }
    panic!("rail never left Pending");
}

fn claim_state(fx: &Fixture, digest: &str) -> ClaimState {
    fx.claims.get(digest).unwrap().unwrap().state
}

#[tokio::test]
async fn honest_submit_holds_only_and_finality_alone_writes_the_ledger() {
    let fx = Fixture::new("rail-honest");
    let rail = sim(&fx, "h", SimMode::SettleAfterPolls(2));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);
    assert_eq!(s.correlation_id, RailInstruction::correlation_id_for(&d));
    assert_eq!(s.brce_prepare_receipt_digests.len(), 2);
    assert_eq!(s.brce_outcome_receipt_digests.len(), 2);
    assert!(s.construct_digest.is_some());
    assert!(s.payload_digest.starts_with("sha256:"));
    assert!(s.pee.verify_identity().is_ok());
    assert_eq!(claim_state(&fx, &d), ClaimState::Submitted);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some());
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);

    // F: the ledger cannot show settled before observed finality.
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert!(fx.ledger.lookup(&d).unwrap().is_none());
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 600_000);

    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::Pending);
    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::Pending);
    assert!(fx.ledger.entries().unwrap().is_empty());

    let FinalizeResult::Applied { outcome: FinalityOutcome::Settled(entry), .. } = finalize(&fx, &rail, &d) else {
        panic!("expected settled after finality");
    };
    assert_eq!((entry.amount_minor, entry.debit_account.as_str(), entry.credit_account.as_str()), (400_000, PAYER, PAYEE));
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(claim_state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 600_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn court1_dropped_ack_is_unknown_never_resubmitted_and_settles_once() {
    let fx = Fixture::new("rail-drop");
    let rail = sim(&fx, "d", SimMode::DropAckAfterAccept);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s.detail);
    assert_eq!(claim_state(&fx, &d), ClaimState::UnknownOutcome);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some(), "hold retained");
    assert!(fx.ledger.entries().unwrap().is_empty());

    // No second effect can be manufactured for the same effect.
    let again = fx.admit(fx.effect("400000", "inv-1"), "n-2");
    assert_eq!(again.unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);

    let FinalizeResult::Applied { outcome: FinalityOutcome::Settled(_), .. } = poll_terminal(&fx, &rail, &d) else {
        panic!("status poll must resolve to Final");
    };
    assert_eq!(claim_state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn rail_rejection_releases_hold_and_funds_are_reusable() {
    let fx = Fixture::with("rail-reject", 500_000, 500_000, 5_000_000);
    let rail = sim(&fx, "r", SimMode::RejectAfterPolls(1));
    let s = submit(&fx, &rail, "400000", "inv-a", "n-a").await;
    let d = s.effect_id.clone();
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);

    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::Pending);
    assert!(matches!(finalize(&fx, &rail, &d), FinalizeResult::Applied { outcome: FinalityOutcome::Released, .. }));
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(claim_state(&fx, &d), ClaimState::Refused);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 500_000);
    assert!(fx.ledger.entries().unwrap().is_empty());

    // Funds are free again: a different obligation holds and submits.
    let s2 = submit(&fx, &rail, "400000", "inv-b", "n-b").await;
    assert_eq!(s2.standing, RailStandingAfterSubmit::Submitted, "{}", s2.detail);
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 100_000);
}

#[tokio::test]
async fn rail_down_during_submit_is_unknown_then_proven_absent_when_rail_never_saw_it() {
    let fx = Fixture::new("rail-down");
    let rail = sim(&fx, "x", SimMode::Down);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s.detail);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some());

    // Still down: uninformative, nothing changes.
    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::StillUnknown);
    assert_eq!(claim_state(&fx, &d), ClaimState::UnknownOutcome);
    assert!(fx.ledger.hold_of(&d).unwrap().is_some());

    // Rail recovers and authoritatively never saw the effect.
    rail.set_mode(SimMode::Honest).unwrap();
    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::ProvenAbsent);
    assert_eq!(claim_state(&fx, &d), ClaimState::Refused);
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(rail.submissions_seen(&s.correlation_id), 0);
}

#[tokio::test]
async fn return_after_final_posts_the_inverse_entry() {
    let fx = Fixture::new("rail-return");
    let rail = sim(&fx, "ret", SimMode::ReturnAfterSettle);
    let s = submit(&fx, &rail, "250000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    let FinalizeResult::Applied { outcome: FinalityOutcome::Settled(_), .. } = poll_terminal(&fx, &rail, &d) else {
        panic!("expected Final first");
    };
    assert_eq!(claim_state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 250_000);

    let FinalizeResult::Applied { outcome: FinalityOutcome::Returned(_), .. } = finalize(&fx, &rail, &d) else {
        panic!("expected Returned");
    };
    assert_eq!(claim_state(&fx, &d), ClaimState::Returned);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());

    // Repeating the return report changes nothing.
    let _ = finalize(&fx, &rail, &d);
    assert_eq!(fx.ledger.returns().unwrap().len(), 1);
}

#[tokio::test]
async fn duplicate_and_reordered_reports_change_nothing() {
    let fx = Fixture::new("rail-dup");
    let rail = sim(&fx, "dup", SimMode::DuplicateReports);
    let s = submit(&fx, &rail, "100000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::Pending);
    let FinalizeResult::Applied { outcome: FinalityOutcome::Settled(_), .. } = finalize(&fx, &rail, &d) else { panic!("settled") };
    for _ in 0..4 {
        assert!(matches!(finalize(&fx, &rail, &d), FinalizeResult::Applied { outcome: FinalityOutcome::AlreadyFinal, .. }));
    }
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);

    // A reordered late rejection cannot undo observed finality.
    let late_rail = scripted_rail::ScriptedRail::new(vec![scripted_rail::rejected()]);
    assert_eq!(finalize_via_rail(&d, &fx.claims, &fx.ledger, &late_rail).unwrap_err(), EVIDENCE_CONFLICT);
    assert_eq!(claim_state(&fx, &d), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

#[tokio::test]
async fn insufficient_funds_is_definite_refusal_and_the_rail_is_never_called() {
    let fx = Fixture::with("rail-poor", 100_000, 500_000, 2_000_000);
    let rail = sim(&fx, "poor", SimMode::Honest);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();

    assert_eq!(s.standing, RailStandingAfterSubmit::Refused);
    assert_eq!(s.detail, "REFUSED:PAYMENT_INSUFFICIENT_FUNDS");
    assert_eq!(claim_state(&fx, &d), ClaimState::Refused);
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(rail.submissions_seen(&s.correlation_id), 0);
    assert_eq!(rail.settlement_count(&s.correlation_id), 0);
    assert!(rail.accepted_correlations().is_empty());
}

#[tokio::test]
async fn expired_pee_is_refused_before_do_with_no_hold() {
    let fx = Fixture::new("rail-expired");
    let rail = sim(&fx, "exp", SimMode::Honest);
    let a = fx.admit(fx.effect("400000", "inv-1"), "n-1").unwrap();
    let d = a.effect().digest().to_string();

    let err = submit_via_rail(a, &params(900), &fx.exec_ctx(), &rail).await.unwrap_err();
    assert_eq!(err, "REFUSED:EFFECT_EXPIRED");
    assert_eq!(claim_state(&fx, &d), ClaimState::Refused);
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(rail.accepted_correlations().is_empty());
    assert!(!fx.dir.join("brce").join("rail").exists(), "no DO journal was opened");
}

#[tokio::test]
async fn admission_is_single_use_across_paths() {
    let fx = Fixture::new("rail-single");
    let rail = sim(&fx, "s", SimMode::Honest);
    let s = submit(&fx, &rail, "1000", "inv-1", "n-1").await;
    // Claim is now Submitted; a second admission of the same effect is in flight.
    assert_eq!(fx.admit(fx.effect("1000", "inv-1"), "n-2").unwrap_err(), "REFUSED:PAYMENT_IN_FLIGHT");
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);
}
