mod common;
use common::payments::*;

use castle::payments::reconcile::recover_journal;
use castle::payments::*;

#[tokio::test]
async fn journal_recovery_classifies_missing_outcome_as_prepared_without_outcome() {
    let fx = Fixture::new("crash-journal");
    let a = fx.admit(fx.effect("100", "inv-j"), "n-1").unwrap();
    let exec = execute_payment(a, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled, "{}", exec.detail);

    let root = fx.dir.join("brce");
    let rec = recover_journal(&root).unwrap();
    assert_eq!(rec.complete.len(), 2);
    assert!(rec.prepared_without_outcome.is_empty());

    // Simulate crash between DO and OUTCOME persistence: drop one outcome file.
    let victim = exec.brce_outcome_receipt_digests[1].clone();
    std::fs::remove_file(root.join(format!("{victim}.outcome.json"))).unwrap();

    let rec = recover_journal(&root).unwrap();
    assert_eq!(rec.complete.len(), 1);
    assert_eq!(rec.prepared_without_outcome, vec![exec.brce_prepare_receipt_digests[1].clone()]);
}

#[test]
fn missing_journal_dir_recovers_empty() {
    let fx = Fixture::new("crash-empty");
    let rec = recover_journal(&fx.dir.join("no-such-journal")).unwrap();
    assert!(rec.complete.is_empty() && rec.prepared_without_outcome.is_empty());
}
