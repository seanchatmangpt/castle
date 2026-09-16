//! VISION.md 2030-horizon item made real: "A second planner exists and the
//! ensemble's scoring/selection has been shown to matter." Two producers for
//! one effect predicate — one cheap-but-longer, one expensive-but-shorter —
//! run through `run_planner_ensemble`, asserting the winner's `score`,
//! `planner_id`, and `process.activities` differ from `WitnessPlanner`'s and
//! that the winning process differs structurally, not just in labeling.

use castle::castle::{
    compile_adversarial_classes, run_planner_ensemble, AdversarialGoal, CostMinimizingPlanner,
    PlanCandidate, Planner, PlanningProblem, TransitionRule, VulnerabilityCondition,
    WitnessPlanner,
};

fn rule(id: &str, preconditions: &[&str], effects: &[&str], cost: Option<f64>) -> TransitionRule {
    TransitionRule {
        id: id.to_string(),
        preconditions: preconditions.iter().map(|s| (*s).to_string()).collect(),
        effects: effects.iter().map(|s| (*s).to_string()).collect(),
        cost,
        planner_hint: None,
    }
}

fn goal() -> AdversarialGoal {
    AdversarialGoal {
        id: "data-exfiltration".to_string(),
        predicate: "goal:exfiltrate".to_string(),
        consequence: 100,
    }
}

/// Two producer chains for `goal:exfiltrate` from the same base condition:
/// the slow chain sums to cost 1.0 over three transitions; the direct chain
/// costs 10.0 in one transition. The witness (backward, subset-minimal) finds
/// the short one; the cost planner finds the cheap one.
fn rules() -> Vec<TransitionRule> {
    vec![
        rule("slow-hop-1", &["cap:base"], &["mid:one"], Some(0.3)),
        rule("slow-hop-2", &["mid:one"], &["mid:two"], Some(0.3)),
        rule("slow-hop-3", &["mid:two"], &["goal:exfiltrate"], Some(0.4)),
        rule(
            "fast-direct",
            &["cap:base"],
            &["goal:exfiltrate"],
            Some(10.0),
        ),
    ]
}

fn vulnerability() -> VulnerabilityCondition {
    VulnerabilityCondition {
        goal_id: goal().id,
        predicates: vec!["cap:base".to_string()],
        witness_transitions: vec!["fast-direct".to_string()],
    }
}

fn transition_ids(candidate: &PlanCandidate) -> Vec<String> {
    candidate
        .process
        .activities
        .iter()
        .map(|a| a.transition_id.clone())
        .collect()
}

#[tokio::test]
async fn cost_minimizing_planner_wins_ensemble_selection_over_the_witness_planner() {
    let g = goal();
    let rs = rules();
    let v = vulnerability();
    let problem = PlanningProblem {
        goal: &g,
        vulnerability: &v,
        rules: &rs,
    };
    let planners: Vec<Box<dyn Planner>> = vec![
        Box::new(WitnessPlanner::default()),
        Box::new(CostMinimizingPlanner::default()),
    ];
    let candidates = run_planner_ensemble(&problem, &planners).await;

    assert_eq!(
        candidates.len(),
        2,
        "both ensemble members contribute a candidate"
    );
    let winner = &candidates[0];
    let witness = &candidates[1];

    // The winning planner differs from WitnessPlanner, and selection order
    // (ascending score) put it first: summed cost 1.0 beats 1 activity + 1
    // predicate.
    assert_eq!(winner.planner_id, "cost-minimizing-forward-search");
    assert_eq!(winner.score, 1, "ceil(0.3 + 0.3 + 0.4)");
    assert_eq!(witness.planner_id, "witness-partial-order");
    assert_eq!(witness.score, 2, "1 witness activity + 1 base predicate");

    // The winning process differs structurally, not just in labeling: three
    // chained slow-hop activities instead of the single direct one.
    assert_eq!(
        transition_ids(winner),
        vec![
            "slow-hop-1".to_string(),
            "slow-hop-2".to_string(),
            "slow-hop-3".to_string(),
        ]
    );
    assert_eq!(transition_ids(witness), vec!["fast-direct".to_string()]);
    assert_ne!(winner.process.id, witness.process.id);
    // Causal ordering survived compilation: hop 2 depends on hop 1, hop 3 on hop 2.
    let hop2 = winner
        .process
        .activities
        .iter()
        .find(|a| a.transition_id == "slow-hop-2")
        .unwrap();
    let hop3 = winner
        .process
        .activities
        .iter()
        .find(|a| a.transition_id == "slow-hop-3")
        .unwrap();
    assert_eq!(hop2.predecessors.len(), 1);
    assert_eq!(hop3.predecessors, vec![hop2.id.clone()]);
}

#[tokio::test]
async fn cost_minimizing_search_is_deterministic_across_runs() {
    let g = goal();
    let rs = rules();
    let v = vulnerability();
    let problem = PlanningProblem {
        goal: &g,
        vulnerability: &v,
        rules: &rs,
    };
    let planners: Vec<Box<dyn Planner>> = vec![
        Box::new(WitnessPlanner::default()),
        Box::new(CostMinimizingPlanner::default()),
    ];
    let first = run_planner_ensemble(&problem, &planners).await;
    let second = run_planner_ensemble(&problem, &planners).await;
    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.planner_id, b.planner_id);
        assert_eq!(a.score, b.score);
        assert_eq!(a.process, b.process);
    }
}

#[tokio::test]
async fn cost_minimizing_planner_is_fail_closed_when_no_producer_reaches_the_goal() {
    // Nothing produces the goal predicate, so the planner is not applicable
    // and contributes no candidate — the ensemble is not padded with a
    // fabricated plan.
    let g = goal();
    let rs = vec![rule("irrelevant", &["cap:base"], &["mid:other"], Some(1.0))];
    let v = VulnerabilityCondition {
        goal_id: g.id.clone(),
        predicates: vec!["cap:base".to_string()],
        witness_transitions: vec!["irrelevant".to_string()],
    };
    let problem = PlanningProblem {
        goal: &g,
        vulnerability: &v,
        rules: &rs,
    };
    assert!(!CostMinimizingPlanner::default().applicable(&problem));

    let planners: Vec<Box<dyn Planner>> = vec![
        Box::new(CostMinimizingPlanner::default()),
        Box::new(WitnessPlanner::default()),
    ];
    let candidates = run_planner_ensemble(&problem, &planners).await;
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planner_id, "witness-partial-order");

    // A producer exists but is unreachable from the base predicates: still
    // no fabricated candidate.
    let rs_unreachable = vec![rule(
        "orphan",
        &["cap:elsewhere"],
        &["goal:exfiltrate"],
        Some(1.0),
    )];
    let problem_unreachable = PlanningProblem {
        goal: &g,
        vulnerability: &v,
        rules: &rs_unreachable,
    };
    assert!(CostMinimizingPlanner::default().applicable(&problem_unreachable));
    let alone = vec![Box::new(CostMinimizingPlanner::default()) as Box<dyn Planner>];
    assert!(run_planner_ensemble(&problem_unreachable, &alone)
        .await
        .is_empty());
}

#[tokio::test]
async fn ensemble_selection_flows_through_compile_adversarial_classes() {
    // The full rail: derive -> plan with both ensemble members -> the selected
    // class's process comes from the cost planner, proving ensemble scoring
    // changes what CASTLE would otherwise compile from the witness alone.
    let planners: Vec<Box<dyn Planner>> = vec![
        Box::new(WitnessPlanner::default()),
        Box::new(CostMinimizingPlanner::default()),
    ];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    assert!(!classes.is_empty());
    for class in &classes {
        let ids: Vec<&str> = class
            .process
            .activities
            .iter()
            .map(|a| a.transition_id.as_str())
            .collect();
        assert!(
            ids.contains(&"slow-hop-3"),
            "selected process should reach the goal through the cheap chain, got {ids:?}"
        );
    }
}
