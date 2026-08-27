//! Independent deterministic planner for the CASTLE planner ensemble.
//!
//! `WitnessPlanner` compiles a witness already present on a vulnerability. This
//! planner instead reconstructs a shortest transition witness from the admitted
//! transition calculus and the vulnerability's base predicates. It is SELECT /
//! CONSTRUCT only and has no path to admission or DO.

use std::collections::BTreeSet;

use async_trait::async_trait;

use crate::castle::{
    compile_witness_to_powl, PlanCandidate, Planner, PlanningProblem, TransitionRule,
    VulnerabilityCondition,
};

pub struct MinimalActionPlanner {
    pub id: String,
}

impl Default for MinimalActionPlanner {
    fn default() -> Self {
        Self {
            id: "minimal-action".to_string(),
        }
    }
}

fn producer_rules<'a>(predicate: &str, rules: &'a [TransitionRule]) -> Vec<&'a TransitionRule> {
    let mut producers: Vec<&TransitionRule> = rules
        .iter()
        .filter(|rule| rule.effects.iter().any(|effect| effect == predicate))
        .collect();
    producers.sort_by(|left, right| left.id.cmp(&right.id));
    producers
}

fn shortest_witness_for(
    predicate: &str,
    base: &BTreeSet<&str>,
    rules: &[TransitionRule],
    visiting: &mut BTreeSet<String>,
) -> Option<Vec<String>> {
    if base.contains(predicate) {
        return Some(Vec::new());
    }
    if !visiting.insert(predicate.to_string()) {
        return None;
    }

    let mut candidates: Vec<Vec<String>> = Vec::new();
    for rule in producer_rules(predicate, rules) {
        let mut witness = Vec::new();
        let mut feasible = true;
        for precondition in &rule.preconditions {
            match shortest_witness_for(precondition, base, rules, visiting) {
                Some(mut prefix) => witness.append(&mut prefix),
                None => {
                    feasible = false;
                    break;
                }
            }
        }
        if feasible {
            witness.push(rule.id.clone());
            let mut seen = BTreeSet::new();
            witness.retain(|id| seen.insert(id.clone()));
            candidates.push(witness);
        }
    }
    visiting.remove(predicate);

    candidates.sort_by(|left, right| {
        left.len()
            .cmp(&right.len())
            .then_with(|| left.join("\u{0}").cmp(&right.join("\u{0}")))
    });
    candidates.into_iter().next()
}

#[async_trait]
impl Planner for MinimalActionPlanner {
    fn id(&self) -> &str {
        &self.id
    }

    fn applicable(&self, problem: &PlanningProblem<'_>) -> bool {
        !problem.rules.is_empty()
            && producer_rules(problem.goal.predicate.as_str(), problem.rules)
                .into_iter()
                .next()
                .is_some()
    }

    async fn plan(&self, problem: &PlanningProblem<'_>) -> Vec<PlanCandidate> {
        if !self.applicable(problem) {
            return Vec::new();
        }
        let base: BTreeSet<&str> = problem
            .vulnerability
            .predicates
            .iter()
            .map(String::as_str)
            .collect();
        let Some(witness_transitions) = shortest_witness_for(
            problem.goal.predicate.as_str(),
            &base,
            problem.rules,
            &mut BTreeSet::new(),
        ) else {
            return Vec::new();
        };
        if witness_transitions.is_empty() {
            return Vec::new();
        }

        let reconstructed = VulnerabilityCondition {
            goal_id: problem.vulnerability.goal_id.clone(),
            predicates: problem.vulnerability.predicates.clone(),
            witness_transitions,
        };
        let process = compile_witness_to_powl(
            &format!("powl:minimal:{}", problem.goal.id),
            &reconstructed,
            problem.rules,
        );
        let score = process.activities.len() as i64 + problem.vulnerability.predicates.len() as i64;
        vec![PlanCandidate {
            planner_id: self.id.clone(),
            process,
            score,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::castle::{AdversarialGoal, WitnessPlanner, run_planner_ensemble};

    fn problem<'a>(
        goal: &'a AdversarialGoal,
        vulnerability: &'a VulnerabilityCondition,
        rules: &'a [TransitionRule],
    ) -> PlanningProblem<'a> {
        PlanningProblem {
            goal,
            vulnerability,
            rules,
        }
    }

    #[tokio::test]
    async fn independently_finds_a_shorter_admitted_witness() {
        let goal = AdversarialGoal {
            id: "goal:root".into(),
            predicate: "root".into(),
            consequence: 100,
        };
        let vulnerability = VulnerabilityCondition {
            goal_id: goal.id.clone(),
            predicates: vec!["foothold".into()],
            witness_transitions: vec!["slow-a".into(), "slow-b".into(), "slow-c".into()],
        };
        let rules = vec![
            TransitionRule { id: "slow-a".into(), preconditions: vec!["foothold".into()], effects: vec!["mid-a".into()], cost: None, planner_hint: None },
            TransitionRule { id: "slow-b".into(), preconditions: vec!["mid-a".into()], effects: vec!["mid-b".into()], cost: None, planner_hint: None },
            TransitionRule { id: "slow-c".into(), preconditions: vec!["mid-b".into()], effects: vec!["root".into()], cost: None, planner_hint: None },
            TransitionRule { id: "fast".into(), preconditions: vec!["foothold".into()], effects: vec!["root".into()], cost: None, planner_hint: None },
        ];
        let problem = problem(&goal, &vulnerability, &rules);
        let candidates = MinimalActionPlanner::default().plan(&problem).await;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].process.activities.len(), 1);
        assert_eq!(candidates[0].process.activities[0].transition_id, "fast");
    }

    #[tokio::test]
    async fn ensemble_ranks_independent_planners_by_common_score() {
        let goal = AdversarialGoal { id: "goal:root".into(), predicate: "root".into(), consequence: 100 };
        let vulnerability = VulnerabilityCondition {
            goal_id: goal.id.clone(),
            predicates: vec!["foothold".into()],
            witness_transitions: vec!["slow-a".into(), "slow-b".into()],
        };
        let rules = vec![
            TransitionRule { id: "slow-a".into(), preconditions: vec!["foothold".into()], effects: vec!["mid".into()], cost: None, planner_hint: None },
            TransitionRule { id: "slow-b".into(), preconditions: vec!["mid".into()], effects: vec!["root".into()], cost: None, planner_hint: None },
            TransitionRule { id: "fast".into(), preconditions: vec!["foothold".into()], effects: vec!["root".into()], cost: None, planner_hint: None },
        ];
        let problem = problem(&goal, &vulnerability, &rules);
        let planners: Vec<Box<dyn Planner>> = vec![
            Box::new(WitnessPlanner::default()),
            Box::new(MinimalActionPlanner::default()),
        ];
        let candidates = run_planner_ensemble(&problem, &planners).await;
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].planner_id, "minimal-action");
        assert!(candidates[0].score < candidates[1].score);
    }

    #[tokio::test]
    async fn cycles_do_not_manufacture_a_plan() {
        let goal = AdversarialGoal { id: "goal:root".into(), predicate: "root".into(), consequence: 100 };
        let vulnerability = VulnerabilityCondition { goal_id: goal.id.clone(), predicates: vec!["foothold".into()], witness_transitions: vec![] };
        let rules = vec![
            TransitionRule { id: "a".into(), preconditions: vec!["b".into()], effects: vec!["root".into()], cost: None, planner_hint: None },
            TransitionRule { id: "b".into(), preconditions: vec!["root".into()], effects: vec!["b".into()], cost: None, planner_hint: None },
        ];
        let problem = problem(&goal, &vulnerability, &rules);
        assert!(MinimalActionPlanner::default().plan(&problem).await.is_empty());
    }
}
