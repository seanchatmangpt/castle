//! Court: concurrency and idempotency of the payments kernel. Real collaborators only.
mod common;
use common::payments::*;

use std::sync::{Arc, Barrier};

use castle::payments::*;

const N: usize = 8;

fn run_async<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(f)
}

/// admit + execute; Err(reason) if refused at admission, Ok(standing) otherwise.
fn attempt(fx: &Fixture, effect: castle::sa2a_security::PreparedEffect, nonce: &str) -> Result<PaymentExecution, String> {
    let admission = fx.admit(effect, nonce)?;
    run_async(execute_payment(admission, &fx.exec_ctx()))
}

#[test]
fn eight_concurrent_attempts_at_one_effect_settle_exactly_once() {
    let fx = Arc::new(Fixture::new("idem-same"));
    let barrier = Arc::new(Barrier::new(N));
    let results: Vec<Result<PaymentExecution, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..N)
            .map(|i| {
                let fx = Arc::clone(&fx);
                let barrier = Arc::clone(&barrier);
                s.spawn(move || {
                    let effect = fx.effect("100000", "inv-race");
                    barrier.wait();
                    attempt(&fx, effect, &format!("nonce-{i}"))
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let settled = results
        .iter()
        .filter(|r| matches!(r, Ok(e) if e.standing == PaymentStanding::Settled))
        .count();
    assert_eq!(settled, 1, "results: {:?}", results.iter().map(|r| r.as_ref().map(|e| (e.standing, e.detail.clone()))).collect::<Vec<_>>());
    for r in &results {
        match r {
            Ok(e) if e.standing == PaymentStanding::Settled => {}
            Ok(e) => panic!("non-settled execution reached DO: {:?} {}", e.standing, e.detail),
            Err(reason) => assert!(
                [
                    "REFUSED:PAYMENT_IN_FLIGHT",
                    "REFUSED:PAYMENT_ALREADY_SETTLED",
                    "REFUSED:PAYMENT_OUTCOME_UNKNOWN"
                ]
                .contains(&reason.as_str()),
                "unexpected refusal {reason}"
            ),
        }
    }
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000 - 100_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 100_000);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    let claims = fx.claims.list().unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].state, ClaimState::Executed);
}

#[test]
fn concurrent_distinct_effects_cannot_leak_epoch_budget() {
    // epoch cap 2_000_000; 8 x 400_000 = 3_200_000 requested.
    let fx = Arc::new(Fixture::with("idem-budget", 10_000_000, 500_000, 2_000_000));
    let barrier = Arc::new(Barrier::new(N));
    let results: Vec<Result<PaymentExecution, String>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..N)
            .map(|i| {
                let fx = Arc::clone(&fx);
                let barrier = Arc::clone(&barrier);
                s.spawn(move || {
                    let effect = fx.effect("400000", &format!("inv-budget-{i}"));
                    barrier.wait();
                    attempt(&fx, effect, &format!("nonce-b-{i}"))
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let entries = fx.ledger.entries().unwrap();
    let total: u64 = entries.iter().map(|e| e.amount_minor).sum();
    assert!(total <= 2_000_000, "budget leaked under concurrency: settled {total} > cap 2000000");
    assert_eq!(entries.len(), 5, "cap allows exactly five 400000 effects");
    let settled = results.iter().filter(|r| matches!(r, Ok(e) if e.standing == PaymentStanding::Settled)).count();
    assert_eq!(settled, entries.len());
    for r in &results {
        if let Err(reason) = r {
            assert_eq!(reason, "REFUSED:PAYMENT_BUDGET_EXCEEDED");
        }
    }
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    let live: u64 = fx
        .claims
        .list()
        .unwrap()
        .iter()
        .filter(|c| c.state != ClaimState::Refused)
        .map(|c| c.amount_minor)
        .sum();
    assert!(live <= 2_000_000);
}

#[test]
fn nonce_race_on_one_durable_fence_admits_exactly_one_claimant() {
    let fx = Arc::new(Fixture::new("idem-nonce"));
    let barrier = Arc::new(Barrier::new(N));
    let results: Vec<Result<(), String>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..N)
            .map(|_| {
                let fx = Arc::clone(&fx);
                let barrier = Arc::clone(&barrier);
                s.spawn(move || {
                    barrier.wait();
                    fx.nonces.claim(PRINCIPAL, "shared-nonce")
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1, "{results:?}");
    assert_eq!(results.iter().filter(|r| r.is_err()).count(), N - 1);
    // A different principal may use the same nonce string.
    assert!(fx.nonces.claim("principal:other", "shared-nonce").is_ok());
}
