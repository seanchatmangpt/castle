//! Proves chicago-tdd-tools is genuinely consumable as a cross-repo path dependency.
//!
//! Real dependency (see ../Cargo.toml: `chicago-tdd-tools = { path = "../chicago-tdd-tools", ... }`),
//! real Chicago-TDD harness (test!/fixture_test! + assert_* macros), driving REAL castle state —
//! castle's own `trust_chain` key-history chain — no mocking, no vacuous arithmetic.

use chicago_tdd_tools::prelude::*;

use castle::trust_chain::{check_key_history, KeyHistory};

test!(chicago_tdd_tools_harness_drives_real_castle_trust_chain, {
    // Arrange: real castle KeyHistory, no test doubles
    let mut history = KeyHistory::new();

    // Act: real chain transitions exercised through the cross-repo harness
    history
        .enroll("key-smoke", 10, "assure:build")
        .expect("first enrollment must succeed");
    let refused_dup = history
        .enroll("key-smoke", 20, "assure:build")
        .expect_err("duplicate enrollment must be refused");

    // Assert: state-based assertions via chicago-tdd-tools macros over real outcomes
    assert_eq_msg!(refused_dup, "REFUSED:KEY_ALREADY_ENROLLED", "duplicate enroll must carry the typed refusal");
    assert_ok!(check_key_history(&history, "key-smoke", 15), "epoch inside validity window must verify");
    assert_eq_msg!(
        check_key_history(&history, "key-smoke", 5).unwrap_err(),
        "REFUSED:KEY_NOT_ENROLLED_AT_EPOCH",
        "epoch before valid_from must be refused"
    );
});

fixture_test!(chicago_tdd_tools_fixture_drives_real_castle_chain_fold, fixture, {
    // Arrange: real fixture counter (unique per run), real castle chain
    let counter = fixture.test_counter();
    let mut history = KeyHistory::new();
    let key_id = format!("key-fixture-{counter}");

    // Act: enroll + revoke through the real BLAKE3 chain fold
    history.enroll(&key_id, 1, "assure:deploy").expect("enroll must succeed");
    history.revoke(&key_id, 5).expect("revoke must succeed");

    // Assert: real replayed state — revoked at epoch 5, valid in [1, 5)
    assert_ok!(check_key_history(&history, &key_id, 3), "epoch in validity window must verify");
    assert_eq_msg!(
        check_key_history(&history, &key_id, 5).unwrap_err(),
        "REFUSED:REVOKED_KEY_AT_EPOCH",
        "epoch at revocation boundary must be refused"
    );
});
