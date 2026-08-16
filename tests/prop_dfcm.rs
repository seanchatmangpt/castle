//! Property tests for `castle::derive_vulnerabilities` (DfCM inverse construction).
//!
//! Additive coverage only — does not touch `tests/castle.rs` or `tests/fortune5.rs`.
//! `derive_vulnerabilities` is `pub`, so this exercises it as an external integration
//! test through the crate's real public API (no mocked collaborators): arbitrary,
//! proptest-generated goal/rule sets are fed straight into the real DfCM algorithm.

use std::collections::HashSet;

use castle::{derive_vulnerabilities, AdversarialGoal, TransitionRule};
use proptest::prelude::*;

/// Small, fixed predicate alphabet keeps the generated transition graphs bounded
/// while still exercising arbitrary precondition/effect combinations.
const ALPHABET: [&str; 6] = ["p0", "p1", "p2", "p3", "p4", "p5"];

fn predicate_strategy() -> impl Strategy<Value = String> {
    (0..ALPHABET.len()).prop_map(|i| ALPHABET[i].to_string())
}

fn predicate_set_strategy(max_len: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(predicate_strategy(), 0..=max_len).prop_map(|mut v| {
        v.sort();
        v.dedup();
        v
    })
}

fn rules_strategy(max_rules: usize) -> impl Strategy<Value = Vec<TransitionRule>> {
    prop::collection::vec((predicate_set_strategy(2), predicate_set_strategy(2)), 0..=max_rules).prop_map(|pairs| {
        pairs
            .into_iter()
            .enumerate()
            .map(|(id, (preconditions, effects))| TransitionRule {
                id: format!("rule:{id}"),
                preconditions,
                effects,
                cost: None,
                planner_hint: None,
            })
            .collect()
    })
}

proptest! {
    /// The minimal vulnerability conditions `derive_vulnerabilities` returns must
    /// actually be subset-minimal: no returned predicate set may be a (non-strict)
    /// superset of another returned predicate set, for arbitrary generated goals,
    /// transition rules, and depth bounds. This is the real invariant the existing
    /// `is_subset`-filtering pass in `derive_vulnerabilities` is supposed to
    /// guarantee end-to-end, not merely re-assert what the code already does line
    /// by line.
    #[test]
    fn derived_vulnerabilities_are_subset_minimal(
        goal_predicate in predicate_strategy(),
        rules in rules_strategy(5),
        max_depth in 0u32..6,
    ) {
        let goal = AdversarialGoal {
            id: "goal:prop".to_string(),
            predicate: goal_predicate,
            consequence: 1,
        };
        let vulnerabilities = derive_vulnerabilities(&goal, &rules, max_depth);

        for (i, a) in vulnerabilities.iter().enumerate() {
            for (j, b) in vulnerabilities.iter().enumerate() {
                if i == j {
                    continue;
                }
                let a_set: HashSet<&String> = a.predicates.iter().collect();
                let b_set: HashSet<&String> = b.predicates.iter().collect();
                let b_subset_of_a = b_set.iter().all(|p| a_set.contains(p));
                prop_assert!(
                    !(b_subset_of_a && a_set.len() >= b_set.len() && a_set != b_set),
                    "condition {:?} is a non-minimal superset of {:?}",
                    a.predicates,
                    b.predicates
                );
            }
        }
    }

    /// `derive_vulnerabilities` is a pure function of its inputs: calling it twice
    /// with the same (goal, rules, max_depth) must yield identical output, regardless
    /// of any internal hashing/iteration order.
    #[test]
    fn derive_vulnerabilities_is_deterministic(
        goal_predicate in predicate_strategy(),
        rules in rules_strategy(5),
        max_depth in 0u32..6,
    ) {
        let goal = AdversarialGoal {
            id: "goal:prop".to_string(),
            predicate: goal_predicate,
            consequence: 1,
        };
        let first = derive_vulnerabilities(&goal, &rules, max_depth);
        let second = derive_vulnerabilities(&goal, &rules, max_depth);
        prop_assert_eq!(first, second);
    }
}
