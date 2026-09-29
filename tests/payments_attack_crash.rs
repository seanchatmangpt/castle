//! Lane `crash`: crash-consistency / money-safety attacks. Tests marked
//! #[ignore] FAIL on current code (FINDING); run with `--ignored` to prove it.
mod common;
use common::payments::*;

use std::sync::Barrier;

use castle::payments::*;

fn claim(digest: &str, obligation: &str, amount: u64) -> Claim {
    Claim {
        effect_digest: format!("sha256:{digest}"),
        principal: PRINCIPAL.into(),
        payer: PAYER.into(),
        payee: PAYEE.into(),
        amount_minor: amount,
        currency: Currency::USD,
        generation: 1,
        state: ClaimState::Reserved,
        reverses: None,
        construct_digest: None,
        detail: "ADMITTED".into(),
        obligation_id: obligation.into(),
        purpose: "p".into(),
        audience: AUDIENCE.into(),
        verified_custodian_ids: vec![],
    }
}

/// FINDING F1: ClaimStore's budget lock is an in-process Mutex. Two handles on
/// the same directory (== two processes) both pass check_budget then both
/// write_new different digests: epoch cap is double-spent.
#[test]
#[ignore = "FINDING F1: epoch budget double-spend across ClaimStore handles (in-process lock only)"]
fn two_handles_cannot_both_reserve_the_whole_epoch_budget() {
    let mut worst = 0;
    for round in 0..40 {
        let dir = unique_dir(&format!("f1-{round}"));
        let n = 8;
        let barrier = Barrier::new(n);
        let ok: usize = std::thread::scope(|s| {
            let hs: Vec<_> = (0..n)
                .map(|i| {
                    let (dir, barrier) = (dir.clone(), &barrier);
                    s.spawn(move || {
                        let store = ClaimStore::open(dir.join("claims")).unwrap(); // OWN handle
                        let c = claim(&format!("{i:064}"), &format!("obl-{i}"), 1_000);
                        barrier.wait();
                        store.reserve(&c, Some(1_000)).is_ok()
                    })
                })
                .collect();
            hs.into_iter().map(|h| usize::from(h.join().unwrap())).sum()
        });
        worst = worst.max(ok);
    }
    assert_eq!(worst, 1, "epoch cap 1000 admitted up to {worst} claims of 1000 across handles");
}

/// FINDING F2: FileJournalLedger's lock is in-process; two handles both pass
/// the balance check and overdraw the payer, after which balance() errors
/// NEGATIVE_BALANCE_INVARIANT forever (ledger bricked).
#[test]
#[ignore = "FINDING F2: ledger overdraft across handles; negative balance bricks balance()"]
fn two_ledger_handles_cannot_overdraw_the_payer() {
    let mut overdrawn = 0;
    for round in 0..40 {
        let fx = Fixture::with(&format!("f2-{round}"), 100, 100, 1_000_000);
        let a = fx.admit(fx.effect("100", "obl-a"), "na").unwrap();
        let b = fx.admit(fx.effect("100", "obl-b"), "nb").unwrap();
        let other = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap();
        let barrier = Barrier::new(2);
        let (ra, rb) = std::thread::scope(|s| {
            let ha = s.spawn(|| { barrier.wait(); fx.ledger.post(&a, "c") });
            let hb = s.spawn(|| { barrier.wait(); other.post(&b, "c") });
            (ha.join().unwrap(), hb.join().unwrap())
        });
        if ra.is_ok() && rb.is_ok() {
            overdrawn += 1;
        }
    }
    assert_eq!(overdrawn, 0, "{overdrawn}/40 rounds posted 200 from a balance of 100");
}

/// FINDING F3: seq = entries().len()+1 under a per-handle lock -> duplicate seq
/// across handles.
#[test]
#[ignore = "FINDING F3: duplicate ledger seq across handles"]
fn ledger_seq_is_unique_across_handles() {
    let mut dup = 0;
    for round in 0..40 {
        let fx = Fixture::new(&format!("f3-{round}"));
        let a = fx.admit(fx.effect("10", "obl-a"), "na").unwrap();
        let b = fx.admit(fx.effect("10", "obl-b"), "nb").unwrap();
        let other = FileJournalLedger::open(fx.dir.join("ledger"), &[]).unwrap();
        let barrier = Barrier::new(2);
        std::thread::scope(|s| {
            s.spawn(|| { barrier.wait(); fx.ledger.post(&a, "c").unwrap(); });
            s.spawn(|| { barrier.wait(); other.post(&b, "c").unwrap(); });
        });
        let e = fx.ledger.entries().unwrap();
        if e.len() == 2 && e[0].seq == e[1].seq {
            dup += 1;
        }
    }
    assert_eq!(dup, 0, "{dup}/40 rounds produced duplicate seq");
}

/// FINDING F4: one torn/corrupt claim file makes list() fail, so every capped
/// admission (all principals) is refused CLAIM_STORE_FAILED: single-file DoS.
#[test]
#[ignore = "FINDING F4: one corrupt claim file blocks ALL admissions"]
fn one_torn_claim_file_does_not_block_unrelated_admissions() {
    let fx = Fixture::new("f4");
    std::fs::write(fx.dir.join("claims").join("deadbeef.claim.json"), b"{\"effect_dig").unwrap();
    let r = fx.admit(fx.effect("100", "obl-unrelated"), "n1");
    assert!(r.is_ok(), "unrelated admission blocked by torn file: {:?}", r.err());
}

/// FINDING F5: a torn ledger entry file makes balance() (hence every post)
/// fail, including for unrelated accounts.
#[test]
#[ignore = "FINDING F5: one corrupt ledger entry file bricks all posts/balances"]
fn one_torn_ledger_entry_does_not_brick_balance() {
    let fx = Fixture::new("f5");
    std::fs::write(fx.dir.join("ledger").join("ffff.entry.json"), b"{\"seq\":").unwrap();
    assert!(fx.ledger.balance(PAYER, Currency::USD).is_ok());
}

/// FINDING F6: nonce is burned before budget reservation; a budget refusal
/// (retryable once budget frees) makes the same signed certificate unusable.
#[test]
#[ignore = "FINDING F6: nonce burned when reserve refuses (BUDGET_EXCEEDED); cert cannot be retried"]
fn budget_refusal_does_not_burn_the_certificate_nonce() {
    let fx = Fixture::with("f6", 1_000_000, 500_000, 1_000);
    let e1 = fx.effect("1000", "obl-1");
    let _a1 = fx.admit(e1, "n1").unwrap(); // reserves whole budget
    let e2 = fx.effect("500", "obl-2");
    let cert = fx.cert(&e2, "n2", &["mac", "phone"]);
    let first = admit_payment(e2.clone(), &cert, &fx.admission_ctx());
    assert_eq!(first.unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");
    // free budget
    let d1 = fx.claims.list().unwrap()[0].effect_digest.clone();
    fx.claims.transition(&d1, &[ClaimState::Reserved], ClaimState::Refused, None, "x").unwrap();
    let second = admit_payment(e2, &cert, &fx.admission_ctx());
    assert!(second.is_ok(), "same cert now refused: {:?}", second.err());
}

/// Regression (passes): crash after ledger post, before claim transition,
/// leaves Reserved; re-admission is IN_FLIGHT and reconcile recovers Executed.
#[test]
fn crash_after_post_before_transition_is_recoverable_by_reconcile() {
    let fx = Fixture::new("c1");
    let a = fx.admit(fx.effect("100", "obl-1"), "n1").unwrap();
    let d = a.effect().digest().to_string();
    fx.ledger.post(&a, "c").unwrap();
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Reserved);
    assert_eq!(fx.admit(fx.effect("100", "obl-1"), "n2").unwrap_err(), "REFUSED:PAYMENT_IN_FLIGHT");
    assert!(matches!(reconcile(&d, &fx.claims, &fx.ledger).unwrap(), ReconcileResolution::Settled(_)));
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Executed);
}

/// Regression (passes): crash before DO leaves Reserved forever unless
/// reconcile runs; reconcile proves absence and releases budget.
#[test]
fn crash_before_do_reserved_claim_is_released_by_reconcile() {
    let fx = Fixture::with("c2", 1_000_000, 500_000, 1_000);
    let a = fx.admit(fx.effect("1000", "obl-1"), "n1").unwrap();
    let d = a.effect().digest().to_string();
    assert_eq!(fx.admit(fx.effect("1", "obl-2"), "n2").unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");
    assert_eq!(reconcile(&d, &fx.claims, &fx.ledger).unwrap(), ReconcileResolution::ProvenAbsent);
    assert!(fx.admit(fx.effect("1", "obl-2"), "n3").is_ok());
}
