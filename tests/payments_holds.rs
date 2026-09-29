mod common;
use common::payments::*;

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
    let ev = castle::payments::settlement::FinalityEvidence {
        effect_digest: digest.clone(),
        correlation_id: "c-1".into(),
        evidence_digest: "ev-rej".into(),
        kind: castle::payments::settlement::FinalityKind::Rejected,
        reason: "AC04".into(),
    };
    castle::payments::settlement::apply_finality(&fx.claims, &fx.ledger, &ev).unwrap();
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
