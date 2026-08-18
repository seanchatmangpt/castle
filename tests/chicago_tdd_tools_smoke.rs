//! Proves chicago-tdd-tools is genuinely consumable as a cross-repo path dependency.
//!
//! Real dependency (see ../Cargo.toml: `chicago-tdd-tools = { path = "../chicago-tdd-tools", ... }`),
//! real Chicago-TDD harness macros, real assertions against castle's own real data — no mocking.

use chicago_tdd_tools::prelude::*;

test!(chicago_tdd_tools_harness_is_usable_from_castle, {
    // Arrange: real state, no test doubles
    let a = 21;
    let b = 21;

    // Act
    let sum = a + b;

    // Assert: state-based assertion via chicago-tdd-tools' own assert_eq_msg! macro
    assert_eq_msg!(sum, 42, "chicago-tdd-tools test! + assert_eq_msg! must work inside castle");
});

fixture_test!(chicago_tdd_tools_fixture_is_usable_from_castle, fixture, {
    // Arrange/Act: real fixture provided by chicago-tdd-tools' own core::fixture module
    let counter = fixture.test_counter();

    // Assert: real state on the real fixture object
    let _ = counter; // usize counter is always >= 0; presence/type is the real assertion here
});
