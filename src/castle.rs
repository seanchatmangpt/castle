//! Port of `castle.ts`: DfCM goal-to-vulnerability derivation, dependency CONSTRUCT,
//! POWL compilation, the receipted CONSTRUCT admission chain, and the exclusive
//! `execute_powl_with_gym_act` DO path.
//!
//! `CONSTRUCT != DO`: everything up to `admit_construct_for_do` manufactures inert
//! candidates. Only `execute_powl_with_gym_act` may actuate, and it refuses unless
//! given a `ConstructAdmission` manufactured exclusively by `admit_construct_for_do`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::Instrument;

pub type Predicate = String;

fn digest_re_ok(digest: &str) -> bool {
    digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn set_key<'a, I: IntoIterator<Item = &'a str>>(values: I) -> String {
    let mut v: Vec<&str> = values.into_iter().collect();
    v.sort();
    v.dedup();
    v.join("\u{0}")
}

fn is_subset(a: &[String], b: &[String]) -> bool {
    let bs: HashSet<&String> = b.iter().collect();
    a.iter().all(|x| bs.contains(x))
}

fn same_strings(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap(),
        Value::Array(arr) => format!("[{}]", arr.iter().map(canonical_json).collect::<Vec<_>>().join(",")),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_json(&map[k])))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn digest_canonical(value: &Value, blake3: &dyn Blake3Provider) -> Result<String, String> {
    let digest = blake3.digest_utf8(&canonical_json(value));
    if !digest_re_ok(&digest) {
        return Err("invalid BLAKE3-256 digest provider output".to_string());
    }
    Ok(digest)
}

// ---------------------------------------------------------------------------
// DfCM goal inversion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub id: String,
    pub preconditions: Vec<Predicate>,
    pub effects: Vec<Predicate>,
    pub cost: Option<f64>,
    pub planner_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdversarialGoal {
    pub id: String,
    pub predicate: Predicate,
    pub consequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VulnerabilityCondition {
    pub goal_id: String,
    pub predicates: Vec<Predicate>,
    pub witness_transitions: Vec<String>,
}

/// DfCM inverse construction: derive minimal base conditions from which a goal
/// is constructible under the admitted transition calculus.
pub fn derive_vulnerabilities(goal: &AdversarialGoal, rules: &[TransitionRule], max_depth: u32) -> Vec<VulnerabilityCondition> {
    let _span = tracing::info_span!(
        "derive_vulnerabilities",
        goal_id = %goal.id,
        rule_count = rules.len(),
        max_depth,
        vulnerability_count = tracing::field::Empty,
    )
    .entered();
    let mut producers: HashMap<&str, Vec<&TransitionRule>> = HashMap::new();
    for rule in rules {
        for effect in &rule.effects {
            producers.entry(effect.as_str()).or_default().push(rule);
        }
    }
    for bucket in producers.values_mut() {
        bucket.sort_by(|a, b| a.id.cmp(&b.id));
    }

    struct Candidate {
        required: BTreeSet<Predicate>,
        witness: Vec<String>,
        expanded: HashSet<String>,
    }

    let mut terminals: Vec<Candidate> = Vec::new();
    let mut stack: Vec<(Candidate, u32)> = vec![(
        Candidate {
            required: BTreeSet::from([goal.predicate.clone()]),
            witness: Vec::new(),
            expanded: HashSet::new(),
        },
        0,
    )];
    let mut visited: HashSet<String> = HashSet::new();

    while let Some((candidate, depth)) = stack.pop() {
        let required_refs: Vec<&str> = candidate.required.iter().map(|s| s.as_str()).collect();
        let witness_refs: Vec<&str> = candidate.witness.iter().map(|s| s.as_str()).collect();
        let visit_key = format!("{}|{}", set_key(required_refs), set_key(witness_refs));
        if visited.contains(&visit_key) {
            continue;
        }
        visited.insert(visit_key);

        let mut required_sorted: Vec<&Predicate> = candidate.required.iter().collect();
        required_sorted.sort();
        let expandable = required_sorted.into_iter().find(|predicate| {
            producers
                .get(predicate.as_str())
                .map(|options| options.iter().any(|r| !candidate.expanded.contains(&format!("{}|{}", predicate, r.id))))
                .unwrap_or(false)
        });

        let expandable = match (expandable, depth >= max_depth) {
            (Some(p), false) => p.clone(),
            _ => {
                terminals.push(candidate);
                continue;
            }
        };

        let options: Vec<&TransitionRule> = producers.get(expandable.as_str()).cloned().unwrap_or_default();
        let mut emitted = false;
        for rule in options {
            let expansion_key = format!("{}|{}", expandable, rule.id);
            if candidate.expanded.contains(&expansion_key) {
                continue;
            }
            emitted = true;
            let mut required = candidate.required.clone();
            required.remove(&expandable);
            for p in &rule.preconditions {
                required.insert(p.clone());
            }
            let mut expanded = candidate.expanded.clone();
            expanded.insert(expansion_key);
            let mut witness = candidate.witness.clone();
            witness.push(rule.id.clone());
            stack.push((Candidate { required, witness, expanded }, depth + 1));
        }
        if !emitted {
            terminals.push(candidate);
        }
    }

    let mut normalized: Vec<VulnerabilityCondition> = terminals
        .into_iter()
        .map(|c| {
            let mut predicates: Vec<Predicate> = c.required.into_iter().collect();
            predicates.sort();
            let mut seen = BTreeSet::new();
            let mut witness_transitions: Vec<String> = Vec::new();
            for w in c.witness.into_iter().rev() {
                if seen.insert(w.clone()) {
                    witness_transitions.push(w);
                }
            }
            VulnerabilityCondition {
                goal_id: goal.id.clone(),
                predicates,
                witness_transitions,
            }
        })
        .collect();

    normalized.sort_by(|a, b| {
        a.predicates
            .len()
            .cmp(&b.predicates.len())
            .then_with(|| {
                let a_refs: Vec<&str> = a.predicates.iter().map(|s| s.as_str()).collect();
                let b_refs: Vec<&str> = b.predicates.iter().map(|s| s.as_str()).collect();
                set_key(a_refs).cmp(&set_key(b_refs))
            })
            .then_with(|| {
                let a_refs: Vec<&str> = a.witness_transitions.iter().map(|s| s.as_str()).collect();
                let b_refs: Vec<&str> = b.witness_transitions.iter().map(|s| s.as_str()).collect();
                set_key(a_refs).cmp(&set_key(b_refs))
            })
    });

    let mut minimal: Vec<VulnerabilityCondition> = Vec::new();
    for candidate in normalized {
        if minimal.iter().any(|m| is_subset(&m.predicates, &candidate.predicates)) {
            continue;
        }
        minimal.push(candidate);
    }
    _span.record("vulnerability_count", minimal.len());
    minimal
}

// ---------------------------------------------------------------------------
// Dependency CONSTRUCT
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone)]
pub struct ConstructedCompromise {
    pub dependency_id: String,
    pub capability: String,
    pub facts: Vec<Predicate>,
    pub impacted: Vec<String>,
    pub epistemic_class: &'static str, // always "COUNTERFACTUAL"
}

pub struct DependencyGraph {
    pub nodes: BTreeMap<String, DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    dependents: BTreeMap<String, BTreeSet<String>>,
}

impl DependencyGraph {
    pub fn new(nodes: Vec<DependencyNode>, edges: Vec<DependencyEdge>) -> Result<Self, String> {
        let node_map: BTreeMap<String, DependencyNode> = nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
        let mut sorted_edges = edges;
        sorted_edges.sort_by(|a, b| format!("{}|{}|{}", a.from, a.to, a.relation).cmp(&format!("{}|{}|{}", b.from, b.to, b.relation)));
        let mut dependents: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in &sorted_edges {
            if !node_map.contains_key(&edge.from) || !node_map.contains_key(&edge.to) {
                return Err(format!("dependency edge references unknown node: {} -> {}", edge.from, edge.to));
            }
            dependents.entry(edge.from.clone()).or_default().insert(edge.to.clone());
        }
        Ok(Self { nodes: node_map, edges: sorted_edges, dependents })
    }

    pub fn impacted_closure<'a, I: IntoIterator<Item = &'a str>>(&self, changed_ids: I) -> Vec<String> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut queue: Vec<String> = changed_ids.into_iter().map(|s| s.to_string()).collect();
        queue.sort();
        let mut i = 0;
        while i < queue.len() {
            let id = queue[i].clone();
            i += 1;
            if seen.contains(&id) {
                continue;
            }
            seen.insert(id.clone());
            if let Some(deps) = self.dependents.get(&id) {
                for dependent in deps {
                    if !seen.contains(dependent) {
                        queue.push(dependent.clone());
                    }
                }
            }
        }
        seen.into_iter().collect()
    }

    pub fn construct_compromise(&self, dependency_id: &str, capability: &str) -> Result<ConstructedCompromise, String> {
        if !self.nodes.contains_key(dependency_id) {
            return Err(format!("unknown dependency: {dependency_id}"));
        }
        Ok(ConstructedCompromise {
            dependency_id: dependency_id.to_string(),
            capability: capability.to_string(),
            facts: vec![format!("compromised:{}", dependency_id), format!("capability:{}:{}", dependency_id, capability)],
            impacted: self.impacted_closure([dependency_id]),
            epistemic_class: "COUNTERFACTUAL",
        })
    }
}

// ---------------------------------------------------------------------------
// POWL compilation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlActivity {
    pub id: String,
    pub transition_id: String,
    pub predecessors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlProcess {
    pub id: String,
    pub goal_id: String,
    pub activities: Vec<PowlActivity>,
}

impl PowlProcess {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "goal_id": self.goal_id,
            "activities": self.activities.iter().map(|a| json!({
                "id": a.id,
                "transition_id": a.transition_id,
                "predecessors": a.predecessors,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Compile a witness into a partial order based on data dependencies between transitions.
pub fn compile_witness_to_powl(id: &str, vulnerability: &VulnerabilityCondition, rules: &[TransitionRule]) -> PowlProcess {
    let _span = tracing::info_span!(
        "compile_witness_to_powl",
        process_id = %id,
        goal_id = %vulnerability.goal_id,
        witness_len = vulnerability.witness_transitions.len(),
    )
    .entered();
    let by_id: HashMap<&str, &TransitionRule> = rules.iter().map(|r| (r.id.as_str(), r)).collect();
    let witness: Vec<&str> = vulnerability.witness_transitions.iter().map(|s| s.as_str()).filter(|id| by_id.contains_key(id)).collect();
    let mut activities: Vec<PowlActivity> = witness
        .iter()
        .map(|transition_id| {
            let rule = by_id[transition_id];
            let mut predecessors: Vec<String> = witness
                .iter()
                .filter(|other_id| *other_id != transition_id)
                .filter(|other_id| {
                    let other = by_id[*other_id];
                    other.effects.iter().any(|effect| rule.preconditions.contains(effect))
                })
                .map(|p| format!("activity:{p}"))
                .collect();
            predecessors.sort();
            PowlActivity {
                id: format!("activity:{transition_id}"),
                transition_id: transition_id.to_string(),
                predecessors,
            }
        })
        .collect();
    activities.sort_by(|a, b| a.id.cmp(&b.id));
    PowlProcess {
        id: id.to_string(),
        goal_id: vulnerability.goal_id.clone(),
        activities,
    }
}

#[must_use]
pub fn enabled_activities(process: &PowlProcess, completed: &BTreeSet<String>) -> Vec<PowlActivity> {
    let mut enabled: Vec<PowlActivity> = process
        .activities
        .iter()
        .filter(|a| !completed.contains(&a.id))
        .filter(|a| a.predecessors.iter().all(|p| completed.contains(p)))
        .cloned()
        .collect();
    enabled.sort_by(|a, b| a.id.cmp(&b.id));
    enabled
}

// ---------------------------------------------------------------------------
// Planner ensemble
// ---------------------------------------------------------------------------

pub struct PlanningProblem<'a> {
    pub goal: &'a AdversarialGoal,
    pub vulnerability: &'a VulnerabilityCondition,
    pub rules: &'a [TransitionRule],
}

#[derive(Debug, Clone)]
pub struct PlanCandidate {
    pub planner_id: String,
    pub process: PowlProcess,
    pub score: i64,
}

#[async_trait]
pub trait Planner: Send + Sync {
    fn id(&self) -> &str;
    fn applicable(&self, problem: &PlanningProblem<'_>) -> bool;
    async fn plan(&self, problem: &PlanningProblem<'_>) -> Vec<PlanCandidate>;
}

pub struct WitnessPlanner {
    pub id: String,
}

impl Default for WitnessPlanner {
    fn default() -> Self {
        Self { id: "witness-partial-order".to_string() }
    }
}

#[async_trait]
impl Planner for WitnessPlanner {
    fn id(&self) -> &str {
        &self.id
    }

    fn applicable(&self, problem: &PlanningProblem<'_>) -> bool {
        !problem.vulnerability.witness_transitions.is_empty()
    }

    async fn plan(&self, problem: &PlanningProblem<'_>) -> Vec<PlanCandidate> {
        if !self.applicable(problem) {
            return Vec::new();
        }
        let predicate_refs: Vec<&str> = problem.vulnerability.predicates.iter().map(|s| s.as_str()).collect();
        let process = compile_witness_to_powl(&format!("powl:{}:{}", problem.goal.id, set_key(predicate_refs)), problem.vulnerability, problem.rules);
        let score = process.activities.len() as i64 + problem.vulnerability.predicates.len() as i64;
        vec![PlanCandidate { planner_id: self.id.clone(), process, score }]
    }
}

pub async fn run_planner_ensemble(problem: &PlanningProblem<'_>, planners: &[Box<dyn Planner>]) -> Vec<PlanCandidate> {
    let span = tracing::info_span!(
        "run_planner_ensemble",
        goal_id = %problem.goal.id,
        planner_count = planners.len(),
        candidate_count = tracing::field::Empty,
    );
    async move {
        let mut applicable: Vec<&Box<dyn Planner>> = planners.iter().filter(|p| p.applicable(problem)).collect();
        applicable.sort_by(|a, b| a.id().cmp(b.id()));
        let mut batches: Vec<PlanCandidate> = Vec::new();
        for planner in applicable {
            batches.extend(planner.plan(problem).await);
        }
        batches.sort_by(|a, b| a.score.cmp(&b.score).then_with(|| a.planner_id.cmp(&b.planner_id)).then_with(|| a.process.id.cmp(&b.process.id)));
        tracing::Span::current().record("candidate_count", batches.len());
        batches
    }
    .instrument(span)
    .await
}

/// Second real `Planner` implementor. Shells out to a real Python subprocess
/// (`scripts/castle_bridge/plan_astar.py` in the autofde-lab checkout) that
/// runs a genuine A* forward search over the problem's transition rules,
/// serializing/deserializing the problem and candidates over stdin/stdout.
///
/// This is a candidate source only, exactly like `WitnessPlanner` -- it
/// produces `PlanCandidate`s for `run_planner_ensemble` to sort and rank. It
/// has no actuation authority: `CONSTRUCT != DO` is unaffected because this
/// planner, like `WitnessPlanner`, never touches `admit_construct_for_do` or
/// `execute_powl_with_gym_act`.
pub struct AutofdeLabPlanner {
    pub id: String,
    /// Path to the bridge script. Defaults to the sibling `autofde-lab`
    /// checkout's `scripts/castle_bridge/plan_astar.py`.
    pub script_path: std::path::PathBuf,
    /// Python interpreter to invoke. Defaults to `python3` on PATH -- the
    /// bridge script is pure stdlib so no autofde-lab venv is required.
    pub python_bin: String,
}

impl Default for AutofdeLabPlanner {
    fn default() -> Self {
        Self {
            id: "autofde-lab-astar".to_string(),
            script_path: std::path::PathBuf::from("/Users/sac/autofde-lab/scripts/castle_bridge/plan_astar.py"),
            python_bin: "python3".to_string(),
        }
    }
}

fn planning_problem_to_json(problem: &PlanningProblem<'_>) -> Value {
    json!({
        "goal": {
            "id": problem.goal.id,
            "predicate": problem.goal.predicate,
            "consequence": problem.goal.consequence,
        },
        "vulnerability": {
            "goal_id": problem.vulnerability.goal_id,
            "predicates": problem.vulnerability.predicates,
            "witness_transitions": problem.vulnerability.witness_transitions,
        },
        "rules": problem.rules.iter().map(|r| json!({
            "id": r.id,
            "preconditions": r.preconditions,
            "effects": r.effects,
            "cost": r.cost,
            "planner_hint": r.planner_hint,
        })).collect::<Vec<_>>(),
    })
}

fn json_to_powl_activity(v: &Value) -> Option<PowlActivity> {
    Some(PowlActivity {
        id: v.get("id")?.as_str()?.to_string(),
        transition_id: v.get("transition_id")?.as_str()?.to_string(),
        predecessors: v.get("predecessors")?.as_array()?.iter().filter_map(|p| p.as_str().map(str::to_string)).collect(),
    })
}

fn json_to_plan_candidate(v: &Value) -> Option<PlanCandidate> {
    let planner_id = v.get("planner_id")?.as_str()?.to_string();
    let process_v = v.get("process")?;
    let process = PowlProcess {
        id: process_v.get("id")?.as_str()?.to_string(),
        goal_id: process_v.get("goal_id")?.as_str()?.to_string(),
        activities: process_v.get("activities")?.as_array()?.iter().filter_map(json_to_powl_activity).collect(),
    };
    let score = v.get("score")?.as_i64()?;
    Some(PlanCandidate { planner_id, process, score })
}

#[async_trait]
impl Planner for AutofdeLabPlanner {
    fn id(&self) -> &str {
        &self.id
    }

    fn applicable(&self, problem: &PlanningProblem<'_>) -> bool {
        !problem.rules.is_empty() && !problem.goal.predicate.is_empty()
    }

    async fn plan(&self, problem: &PlanningProblem<'_>) -> Vec<PlanCandidate> {
        if !self.applicable(problem) {
            return Vec::new();
        }
        let _span = tracing::info_span!(
            "autofde_lab_planner_plan",
            goal_id = %problem.goal.id,
            script = %self.script_path.display(),
        )
        .entered();

        let stdin_payload = planning_problem_to_json(problem).to_string();

        let output = match std::process::Command::new(&self.python_bin)
            .arg(&self.script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(stdin_payload.as_bytes()).is_err() {
                        tracing::warn!("autofde-lab bridge: failed to write stdin");
                        return Vec::new();
                    }
                }
                match child.wait_with_output() {
                    Ok(out) => out,
                    Err(err) => {
                        tracing::warn!(error = %err, "autofde-lab bridge: subprocess wait failed");
                        return Vec::new();
                    }
                }
            }
            Err(err) => {
                tracing::warn!(error = %err, "autofde-lab bridge: failed to spawn subprocess");
                return Vec::new();
            }
        };

        if !output.status.success() {
            tracing::warn!(status = ?output.status, stderr = %String::from_utf8_lossy(&output.stderr), "autofde-lab bridge: subprocess exited non-zero");
            return Vec::new();
        }

        let stdout_text = String::from_utf8_lossy(&output.stdout);
        let parsed: Value = match serde_json::from_str(&stdout_text) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "autofde-lab bridge: invalid JSON on stdout");
                return Vec::new();
            }
        };
        parsed.as_array().map(|arr| arr.iter().filter_map(json_to_plan_candidate).collect()).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Receipts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicClass {
    Constructed,
    Counterfactual,
    Replayed,
    Observed,
    Inferred,
}

impl EpistemicClass {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EpistemicClass::Constructed => "CONSTRUCTED",
            EpistemicClass::Counterfactual => "COUNTERFACTUAL",
            EpistemicClass::Replayed => "REPLAYED",
            EpistemicClass::Observed => "OBSERVED",
            EpistemicClass::Inferred => "INFERRED",
        }
    }
}

pub trait Blake3Provider: Send + Sync {
    /// Return a lowercase 64-hex-character BLAKE3-256 digest.
    fn digest_utf8(&self, input: &str) -> String;
}

pub trait ReceiptSigner: Send + Sync {
    fn key_id(&self) -> &str;
    fn sign_digest(&self, digest_hex: &str) -> String;
}

pub trait ReceiptVerifier: Send + Sync {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub algorithm: &'static str, // "BLAKE3-256"
    pub artifact_digest: String,
    pub receipt_digest: String,
    pub epistemic_class: EpistemicClass,
    pub subject: String,
    pub parent_digests: Vec<String>,
    pub origin_key_id: String,
    pub origin_signature: String,
}

fn unsigned_receipt_payload(algorithm: &str, artifact_digest: &str, epistemic_class: EpistemicClass, subject: &str, parent_digests: &[String], origin_key_id: &str) -> Value {
    let mut parents = parent_digests.to_vec();
    parents.sort();
    json!({
        "algorithm": algorithm,
        "artifact_digest": artifact_digest,
        "epistemic_class": epistemic_class.as_str(),
        "subject": subject,
        "parent_digests": parents,
        "origin_key_id": origin_key_id,
    })
}

pub fn create_receipt(
    artifact: &Value,
    epistemic_class: EpistemicClass,
    subject: &str,
    parent_digests: &[String],
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
) -> Result<Receipt, String> {
    let artifact_digest = digest_canonical(artifact, blake3)?;
    let mut normalized_parents = parent_digests.to_vec();
    normalized_parents.sort();
    if normalized_parents.iter().any(|d| !digest_re_ok(d)) {
        return Err("invalid parent BLAKE3-256 digest".to_string());
    }
    if signer.key_id().is_empty() {
        return Err("receipt signer keyId is required".to_string());
    }
    let unsigned = unsigned_receipt_payload("BLAKE3-256", &artifact_digest, epistemic_class, subject, &normalized_parents, signer.key_id());
    let receipt_digest = digest_canonical(&unsigned, blake3)?;
    let origin_signature = signer.sign_digest(&receipt_digest);
    if origin_signature.is_empty() {
        return Err("receipt signer returned an empty origin signature".to_string());
    }
    Ok(Receipt {
        algorithm: "BLAKE3-256",
        artifact_digest,
        receipt_digest,
        epistemic_class,
        subject: subject.to_string(),
        parent_digests: normalized_parents,
        origin_key_id: signer.key_id().to_string(),
        origin_signature,
    })
}

pub fn verify_receipt(artifact: &Value, receipt: &Receipt, blake3: &dyn Blake3Provider, verifier: &dyn ReceiptVerifier, trusted_origin_key_ids: &BTreeSet<String>) -> bool {
    if receipt.algorithm != "BLAKE3-256" {
        return false;
    }
    if !trusted_origin_key_ids.contains(&receipt.origin_key_id) {
        return false;
    }
    if !digest_re_ok(&receipt.artifact_digest) || !digest_re_ok(&receipt.receipt_digest) {
        return false;
    }
    if receipt.parent_digests.iter().any(|d| !digest_re_ok(d)) {
        return false;
    }
    match digest_canonical(artifact, blake3) {
        Ok(d) if d == receipt.artifact_digest => {}
        _ => return false,
    }
    let expected = unsigned_receipt_payload(receipt.algorithm, &receipt.artifact_digest, receipt.epistemic_class, &receipt.subject, &receipt.parent_digests, &receipt.origin_key_id);
    match digest_canonical(&expected, blake3) {
        Ok(d) if d == receipt.receipt_digest => {}
        _ => return false,
    }
    verifier.verify_digest(&receipt.origin_key_id, &receipt.receipt_digest, &receipt.origin_signature)
}

// ---------------------------------------------------------------------------
// GymAct / OCEL
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WorldState {
    pub system_id: String,
    pub facts: BTreeSet<Predicate>,
}

#[derive(Debug, Clone)]
pub struct TestEnvelope {
    pub system_id: String,
    pub allowed_transition_ids: BTreeSet<String>,
    pub max_steps: u32,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct ActuationPermit {
    pub construct_digest: String,
    pub process_digest: String,
    pub subject: String,
    pub authority: String,
    pub transition_id: String,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone)]
pub struct OcelObject {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct GymActResult {
    pub transition_id: String,
    pub status: GymActStatus,
    pub objects: Vec<OcelObject>,
    pub attributes: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GymActStatus {
    Observed,
    Refused,
}

#[async_trait]
pub trait GymActAdapter: Send + Sync {
    async fn execute(&self, activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> GymActResult;
}

/// Real `GymActAdapter` against a real, already-running, already-authorized
/// local target: the `kind-platform-eng-colima` kind cluster on this host
/// (`kubectl config get-contexts` shows it; `docker ps` shows its live
/// containers). This is the first non-test-double implementor of
/// `GymActAdapter` in the crate.
///
/// Scoped deliberately narrow to stay inside `CONSTRUCT != DO`'s "explicit
/// test envelope over owned or authorized systems, no unbounded actuation"
/// contract (`README.md:53`):
///
/// - Every `transition_id` this adapter will run is a **read-only**
///   `kubectl get` query (nodes / pods / namespaces), fixed at construction
///   time in `allowed_read_only_queries`. There is no code path from a
///   `PowlActivity`'s `transition_id` to an arbitrary shell command --
///   `transition_id` is looked up in the map, and anything not present is
///   refused (`GymActStatus::Refused`) rather than passed through.
/// - It shells out to the real `kubectl` binary against a fixed
///   `--context`, exactly the same "real subprocess, real collaborator"
///   pattern `AutofdeLabPlanner` above uses for its A* bridge -- no network
///   client crate, no mocked Kubernetes API.
/// - `execute_powl_with_gym_act`'s own admission/envelope/receipt chain
///   (digest match, envelope expiry, `allowed_transition_ids` containment)
///   still gates every call into this adapter; this adapter adds a second,
///   independent allowlist on top rather than relying solely on the
///   envelope.
/// - No cluster mutation is possible: the allowlisted commands are `get`
///   verbs only, and the adapter does not accept caller-supplied kubectl
///   arguments.
pub struct KindClusterReadOnlyGymAct {
    /// kubectl context to query, e.g. "kind-platform-eng-colima".
    pub kube_context: String,
    /// transition_id -> fixed, read-only kubectl argv (no user input is ever
    /// interpolated into these).
    pub allowed_read_only_queries: BTreeMap<String, Vec<String>>,
}

impl KindClusterReadOnlyGymAct {
    /// Real, safe default envelope for the `platform-eng-colima` kind
    /// cluster: three read-only `kubectl get` queries against cluster-scoped
    /// or `kube-system` resources only, nothing namespace-arbitrary and
    /// nothing that mutates state.
    #[must_use]
    pub fn platform_eng_colima_default() -> Self {
        let mut allowed_read_only_queries = BTreeMap::new();
        allowed_read_only_queries.insert("observe-cluster-nodes".to_string(), vec!["get".to_string(), "nodes".to_string(), "-o".to_string(), "json".to_string()]);
        allowed_read_only_queries.insert(
            "observe-kube-system-pods".to_string(),
            vec!["get".to_string(), "pods".to_string(), "-n".to_string(), "kube-system".to_string(), "-o".to_string(), "json".to_string()],
        );
        allowed_read_only_queries.insert("observe-namespaces".to_string(), vec!["get".to_string(), "namespaces".to_string(), "-o".to_string(), "json".to_string()]);
        Self { kube_context: "kind-platform-eng-colima".to_string(), allowed_read_only_queries }
    }
}

#[async_trait]
impl GymActAdapter for KindClusterReadOnlyGymAct {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        let _span = tracing::info_span!(
            "kind_cluster_read_only_gym_act_execute",
            transition_id = %activity.transition_id,
            kube_context = %self.kube_context,
        )
        .entered();

        let Some(argv) = self.allowed_read_only_queries.get(&activity.transition_id) else {
            tracing::warn!("kind cluster GymAct: transition_id not in the read-only allowlist, refusing");
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "RefusedObservation".to_string() }],
                attributes: Default::default(),
            };
        };

        let output = match std::process::Command::new("kubectl").arg("--context").arg(&self.kube_context).args(argv).output() {
            Ok(out) => out,
            Err(err) => {
                tracing::warn!(error = %err, "kind cluster GymAct: failed to spawn kubectl");
                return GymActResult {
                    transition_id: activity.transition_id.clone(),
                    status: GymActStatus::Refused,
                    objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "RefusedObservation".to_string() }],
                    attributes: Default::default(),
                };
            }
        };

        if !output.status.success() {
            tracing::warn!(status = ?output.status, stderr = %String::from_utf8_lossy(&output.stderr), "kind cluster GymAct: kubectl exited non-zero");
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "RefusedObservation".to_string() }],
                attributes: Default::default(),
            };
        }

        let stdout_text = String::from_utf8_lossy(&output.stdout);
        let parsed: Value = match serde_json::from_str(&stdout_text) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "kind cluster GymAct: kubectl returned non-JSON stdout");
                return GymActResult {
                    transition_id: activity.transition_id.clone(),
                    status: GymActStatus::Refused,
                    objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "RefusedObservation".to_string() }],
                    attributes: Default::default(),
                };
            }
        };

        // Real observation, from a real cluster response: turn each item's
        // metadata.name into an OcelObject so the resulting OCEL log names
        // the actual nodes/pods/namespaces observed, not a synthetic label.
        let kind = parsed.get("kind").and_then(Value::as_str).unwrap_or("KubernetesList").to_string();
        let mut objects = Vec::new();
        if let Some(items) = parsed.get("items").and_then(Value::as_array) {
            for item in items {
                let name = item.pointer("/metadata/name").and_then(Value::as_str).unwrap_or("unknown");
                objects.push(OcelObject { id: format!("k8s:{}:{}", self.kube_context, name), kind: kind.trim_end_matches("List").to_string() });
            }
        }
        if objects.is_empty() {
            objects.push(OcelObject { id: format!("k8s:{}:{}:empty", self.kube_context, activity.transition_id), kind });
        }

        let mut attributes = BTreeMap::new();
        attributes.insert("observed_item_count".to_string(), json!(objects.len()));
        attributes.insert("kube_context".to_string(), json!(self.kube_context));
        attributes.insert("permit_authority".to_string(), json!(permit.authority));

        GymActResult { transition_id: activity.transition_id.clone(), status: GymActStatus::Observed, objects, attributes }
    }
}

/// Production `GymActAdapter` against the real, already-running `gymact`
/// service (`platform-console/services/gymact/facts.json`), shelling out to
/// the real `gymact` Typer CLI's `verify` subcommand (subprocess+JSON) --
/// the same subprocess-bridge precedent used by `~/autofde-lab` -- rather
/// than gymact's HTTP/FastAPI surface, which avoids network/auth setup for
/// a same-host CLI that is already installed and already authorized against
/// the target cluster's kubeconfig context.
///
/// `VISION.md`'s gap #1 ("no production adapter exists yet") is closed by
/// this type: it is the crate's first `GymActAdapter` implementor backed by
/// a real external CLI process rather than a hand-rolled `kubectl` call.
///
/// Scoped narrow, matching `KindClusterReadOnlyGymAct`'s `CONSTRUCT != DO`
/// discipline (README.md:53 -- "explicit test envelope over owned or
/// authorized systems, no unbounded actuation"):
///
/// - Every `transition_id` this adapter will run is fixed at construction
///   time in `allowed_verifications`, mapping to a caller-supplied gymact
///   `provider` name and an `expected` postcondition object. There is no
///   code path from a `PowlActivity`'s `transition_id` to an arbitrary
///   provider/config -- anything not present in the map is refused
///   (`GymActStatus::Refused`) rather than passed through.
/// - It shells out to the real `gymact` binary's `verify <request.json>`
///   subcommand, which materializes the configured subject then
///   independently observes and checks its current state (READ-shaped:
///   `gymact.gyms.kubernetes_reconciliation`'s `get_status` capability path,
///   never `act`/`scale_restart`) -- see `~/gymact/src/gymact/cli.py`.
/// - `gymact verify`'s CLI path materializes a real subject per invocation
///   (e.g. a real `kubernetes-reconciliation` Pod) but has no CLI
///   `teardown` command (`~/gymact/src/gymact/cli.py` has no such
///   subcommand); this adapter best-effort tears the materialized subject
///   down itself afterward via a fixed, non-interpolated `kubectl delete
///   pod <observed-name> --now` when the verification response exposes a
///   `pod_name` field, so repeated runs do not accumulate live pods on the
///   real cluster. A teardown failure is logged, not surfaced as a GymAct
///   refusal -- the verification's own pass/fail is the receipted fact.
pub struct ProcessGymActAdapter {
    /// Path to the real `gymact` executable (e.g. the project venv's
    /// `.venv/bin/gymact`), invoked as a real subprocess.
    pub gymact_bin: String,
    /// kubectl context used for best-effort teardown of subjects gymact's
    /// CLI materialized but did not tear down itself.
    pub kube_context: String,
    /// transition_id -> (gymact provider name, expected postcondition
    /// object). Fixed at construction time; nothing from `PowlActivity` or
    /// `WorldState` is interpolated into the gymact request.
    pub allowed_verifications: BTreeMap<String, (String, Value)>,
}

impl ProcessGymActAdapter {
    /// Real default envelope for the live `kubernetes-reconciliation`
    /// provider on the `kind-platform-eng-colima` cluster: one transition,
    /// `verify-kubernetes-reconciliation-running`, checking the real
    /// cluster-observed postcondition `{"running": true}`.
    #[must_use]
    pub fn platform_eng_colima_default(gymact_bin: impl Into<String>) -> Self {
        let mut allowed_verifications = BTreeMap::new();
        allowed_verifications.insert(
            "verify-kubernetes-reconciliation-running".to_string(),
            ("kubernetes-reconciliation".to_string(), json!({"running": true})),
        );
        Self {
            gymact_bin: gymact_bin.into(),
            kube_context: "kind-platform-eng-colima".to_string(),
            allowed_verifications,
        }
    }

    fn refused(transition_id: &str) -> GymActResult {
        GymActResult {
            transition_id: transition_id.to_string(),
            status: GymActStatus::Refused,
            objects: vec![OcelObject { id: format!("object:{}", transition_id), kind: "RefusedObservation".to_string() }],
            attributes: Default::default(),
        }
    }

    fn best_effort_teardown_pod(&self, pod_name: &str, namespace: &str) {
        let outcome = std::process::Command::new("kubectl")
            .arg("--context")
            .arg(&self.kube_context)
            .args(["delete", "pod", pod_name, "-n", namespace, "--now"])
            .output();
        match outcome {
            Ok(out) if out.status.success() => {
                tracing::info!(pod_name, namespace, "ProcessGymActAdapter: tore down materialized pod");
            }
            Ok(out) => {
                tracing::warn!(pod_name, namespace, stderr = %String::from_utf8_lossy(&out.stderr), "ProcessGymActAdapter: teardown kubectl exited non-zero");
            }
            Err(err) => {
                tracing::warn!(pod_name, namespace, error = %err, "ProcessGymActAdapter: failed to spawn teardown kubectl");
            }
        }
    }
}

#[async_trait]
impl GymActAdapter for ProcessGymActAdapter {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        let _span = tracing::info_span!(
            "process_gym_act_adapter_execute",
            transition_id = %activity.transition_id,
            gymact_bin = %self.gymact_bin,
        )
        .entered();

        let Some((provider, expected)) = self.allowed_verifications.get(&activity.transition_id) else {
            tracing::warn!("ProcessGymActAdapter: transition_id not in the allowlist, refusing");
            return Self::refused(&activity.transition_id);
        };

        let request = json!({"provider": provider, "config": {}, "expected": expected});
        let request_path = std::env::temp_dir().join(format!("castle-gymact-request-{}-{}.json", activity.transition_id, uuid_like_suffix()));
        if let Err(err) = std::fs::write(&request_path, request.to_string()) {
            tracing::warn!(error = %err, "ProcessGymActAdapter: failed to write request file");
            return Self::refused(&activity.transition_id);
        }

        let output = std::process::Command::new(&self.gymact_bin).arg("verify").arg(&request_path).output();
        let _ = std::fs::remove_file(&request_path);

        let output = match output {
            Ok(out) => out,
            Err(err) => {
                tracing::warn!(error = %err, "ProcessGymActAdapter: failed to spawn gymact");
                return Self::refused(&activity.transition_id);
            }
        };

        if !output.status.success() {
            tracing::warn!(status = ?output.status, stderr = %String::from_utf8_lossy(&output.stderr), "ProcessGymActAdapter: gymact exited non-zero");
            return Self::refused(&activity.transition_id);
        }

        let stdout_text = String::from_utf8_lossy(&output.stdout);
        let parsed: Value = match serde_json::from_str(stdout_text.trim()) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "ProcessGymActAdapter: gymact returned non-JSON stdout");
                return Self::refused(&activity.transition_id);
            }
        };

        let verification = parsed.get("verification").cloned().unwrap_or(Value::Null);
        let passed = verification.get("passed").and_then(Value::as_bool).unwrap_or(false);
        let observed = verification.get("observed").cloned().unwrap_or(Value::Null);

        if let Some(pod_name) = observed.get("pod_name").and_then(Value::as_str) {
            let namespace = observed.get("namespace").and_then(Value::as_str).unwrap_or("default");
            self.best_effort_teardown_pod(pod_name, namespace);
        }

        if !passed {
            tracing::warn!(?verification, "ProcessGymActAdapter: gymact verification did not pass");
            return Self::refused(&activity.transition_id);
        }

        let episode_id = verification.get("episode_id").and_then(Value::as_str).unwrap_or("unknown-episode");
        let mut objects = vec![OcelObject { id: format!("gymact:{}:{}", provider, episode_id), kind: "GymActEpisode".to_string() }];
        if let Some(pod_name) = observed.get("pod_name").and_then(Value::as_str) {
            objects.push(OcelObject { id: format!("k8s:{}:{}", self.kube_context, pod_name), kind: "Pod".to_string() });
        }

        let mut attributes = BTreeMap::new();
        attributes.insert("gymact_provider".to_string(), json!(provider));
        attributes.insert("gymact_episode_id".to_string(), json!(episode_id));
        attributes.insert("gymact_observed".to_string(), observed);
        attributes.insert("permit_authority".to_string(), json!(permit.authority));

        GymActResult { transition_id: activity.transition_id.clone(), status: GymActStatus::Observed, objects, attributes }
    }
}

fn uuid_like_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("{:x}", nanos)
}

/// Real `GymActAdapter` against the local `docker` (or colima-backed docker)
/// daemon -- the 2030-horizon `ContainerGymActAdapter` described in
/// `VISION.md`, now real. It shells out to the real `docker` binary
/// (`std::process::Command`, same subprocess-bridge pattern as
/// `KindClusterReadOnlyGymAct`/`ProcessGymActAdapter`), never a Docker API
/// client crate, and never a mock.
///
/// Scoped narrow, matching this crate's other `GymActAdapter`s'
/// `CONSTRUCT != DO` discipline (`README.md:53` -- "explicit test envelope
/// over owned or authorized systems, no unbounded actuation"):
///
/// - Every `transition_id` this adapter will run is fixed at construction
///   time in `allowed_images`, mapping to a caller-approved, already-pulled
///   image reference. There is no code path from a `PowlActivity`'s
///   `transition_id` to an arbitrary image or command -- anything not
///   present in the map is refused (`GymActStatus::Refused`) before `docker`
///   is ever invoked.
/// - The container this adapter runs is **owned by the adapter itself**: it
///   names every container it creates `castle-gymact-<transition_id>-<nonce>`
///   and never operates on any container it did not create. There is no
///   caller-supplied container name or id anywhere in this type.
/// - The container is started network-isolated (`--network=none`), with no
///   host mounts, running only `sleep 5` -- long enough to observe, never
///   long enough to be a standing liability if teardown fails -- then
///   observed via a real `docker inspect` on the adapter's own container,
///   then torn down (`docker rm -f`) by this adapter in the same call. A
///   teardown failure is logged, not surfaced as a refusal of an otherwise
///   real, successful run; the observation is the receipted fact, but the
///   container is always at minimum a bounded, self-expiring `sleep 5`
///   process even if teardown is skipped.
/// - `permit.expires_at_epoch_ms` is honored explicitly: if the caller-tracked
///   wall clock is already past it when `execute` is invoked, this adapter
///   refuses without spawning `docker` at all, even though
///   `execute_powl_with_gym_act` already checked the envelope/admission
///   expiry upstream -- defense in depth, not reliance on the caller alone.
/// - `execute_powl_with_gym_act`'s own admission/envelope/receipt chain
///   still gates every call into this adapter; `CONSTRUCT != DO` is
///   unrelaxed -- this adapter adds no new actuation path and cannot
///   construct its own `ConstructAdmission` (module-private
///   `sealed::AdmissionBrand` makes that structurally impossible).
/// - On any spawn failure or nonzero `docker` exit, this returns
///   `GymActStatus::Refused` -- never a fabricated `Observed`.
/// - The real captured stdout of the observing `docker inspect` call is
///   BLAKE3-digested and attached as the `stdout_blake3` attribute, so the
///   receipted OCEL event carries a verifiable fingerprint of exactly what
///   was observed, not just a summary.
pub struct ContainerGymActAdapter {
    /// Path to the real `docker` binary (e.g. `"docker"`, resolved via
    /// `PATH`, or an absolute path).
    pub docker_bin: String,
    /// A monotonic wall-clock reader, e.g. `SystemTime::now()` in epoch ms.
    /// Injected (not `SystemTime::now()` called directly) so tests can pin
    /// time, matching `DoAuthorizationContext::now`'s pattern.
    pub now_ms: fn() -> i64,
    /// transition_id -> already-approved, already-pullable image reference.
    /// Fixed at construction time; nothing from `PowlActivity` or
    /// `WorldState` is interpolated into the image reference or command.
    pub allowed_images: BTreeMap<String, String>,
}

impl ContainerGymActAdapter {
    fn real_now_ms() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
    }

    /// Real default envelope: one transition, `observe-alpine-container`,
    /// against the small, widely-available `alpine:latest` image. Callers
    /// must have already pulled the image (this adapter never pulls on the
    /// caller's behalf) -- `docker pull alpine:latest` once is sufficient.
    #[must_use]
    pub fn local_docker_default(docker_bin: impl Into<String>) -> Self {
        let mut allowed_images = BTreeMap::new();
        allowed_images.insert("observe-alpine-container".to_string(), "alpine:latest".to_string());
        Self { docker_bin: docker_bin.into(), now_ms: Self::real_now_ms, allowed_images }
    }

    fn refused(transition_id: &str) -> GymActResult {
        GymActResult {
            transition_id: transition_id.to_string(),
            status: GymActStatus::Refused,
            objects: vec![OcelObject { id: format!("object:{}", transition_id), kind: "RefusedObservation".to_string() }],
            attributes: Default::default(),
        }
    }

    fn best_effort_remove_container(&self, container_name: &str) {
        let outcome = std::process::Command::new(&self.docker_bin).args(["rm", "-f", container_name]).output();
        match outcome {
            Ok(out) if out.status.success() => {
                tracing::info!(container_name, "ContainerGymActAdapter: tore down owned throwaway container");
            }
            Ok(out) => {
                tracing::warn!(container_name, stderr = %String::from_utf8_lossy(&out.stderr), "ContainerGymActAdapter: teardown docker rm exited non-zero");
            }
            Err(err) => {
                tracing::warn!(container_name, error = %err, "ContainerGymActAdapter: failed to spawn teardown docker rm");
            }
        }
    }
}

#[async_trait]
impl GymActAdapter for ContainerGymActAdapter {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        let _span = tracing::info_span!(
            "container_gym_act_adapter_execute",
            transition_id = %activity.transition_id,
            docker_bin = %self.docker_bin,
        )
        .entered();

        let Some(image) = self.allowed_images.get(&activity.transition_id) else {
            tracing::warn!("ContainerGymActAdapter: transition_id not in the fixed image allowlist, refusing");
            return Self::refused(&activity.transition_id);
        };

        if (self.now_ms)() > permit.expires_at_epoch_ms {
            tracing::warn!("ContainerGymActAdapter: permit already expired, refusing without spawning docker");
            return Self::refused(&activity.transition_id);
        }

        let container_name = format!("castle-gymact-{}-{}", activity.transition_id, uuid_like_suffix());

        // Detached (`-d`), network-isolated, no host mounts, running only a
        // fixed, non-interpolated `sleep 5` -- self-expiring even if the
        // adapter's own teardown below never runs. We tear it down ourselves
        // (rather than `--rm`) so we can `docker inspect` it first.
        let run_output = std::process::Command::new(&self.docker_bin).args(["run", "-d", "--network=none", "--name", &container_name, image, "sleep", "5"]).output();
        let run_output = match run_output {
            Ok(out) => out,
            Err(err) => {
                tracing::warn!(error = %err, "ContainerGymActAdapter: failed to spawn docker run");
                return Self::refused(&activity.transition_id);
            }
        };

        if !run_output.status.success() {
            tracing::warn!(status = ?run_output.status, stderr = %String::from_utf8_lossy(&run_output.stderr), "ContainerGymActAdapter: docker run exited non-zero");
            return Self::refused(&activity.transition_id);
        }

        let inspect_output = std::process::Command::new(&self.docker_bin).args(["inspect", &container_name]).output();

        // Always attempt teardown of the container this adapter itself
        // created, whether or not inspect succeeded, so a failed inspect
        // never leaves an owned container running.
        self.best_effort_remove_container(&container_name);

        let inspect_output = match inspect_output {
            Ok(out) => out,
            Err(err) => {
                tracing::warn!(error = %err, "ContainerGymActAdapter: failed to spawn docker inspect");
                return Self::refused(&activity.transition_id);
            }
        };

        if !inspect_output.status.success() {
            tracing::warn!(status = ?inspect_output.status, stderr = %String::from_utf8_lossy(&inspect_output.stderr), "ContainerGymActAdapter: docker inspect exited non-zero");
            return Self::refused(&activity.transition_id);
        }

        let parsed: Value = match serde_json::from_slice(&inspect_output.stdout) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "ContainerGymActAdapter: docker inspect returned non-JSON stdout");
                return Self::refused(&activity.transition_id);
            }
        };

        let container_id = parsed.get(0).and_then(|v| v.get("Id")).and_then(Value::as_str).unwrap_or("unknown").to_string();
        let state_status = parsed.get(0).and_then(|v| v.pointer("/State/Status")).and_then(Value::as_str).unwrap_or("unknown").to_string();

        let stdout_digest = blake3::hash(&inspect_output.stdout).to_hex().to_string();

        let objects = vec![OcelObject { id: format!("docker:{}:{}", container_name, container_id), kind: "Container".to_string() }];
        let mut attributes = BTreeMap::new();
        attributes.insert("image".to_string(), json!(image));
        attributes.insert("container_name".to_string(), json!(container_name));
        attributes.insert("container_state_status".to_string(), json!(state_status));
        attributes.insert("stdout_blake3".to_string(), json!(stdout_digest));
        attributes.insert("permit_authority".to_string(), json!(permit.authority));

        GymActResult { transition_id: activity.transition_id.clone(), status: GymActStatus::Observed, objects, attributes }
    }
}

#[derive(Debug, Clone)]
pub struct OcelEvent {
    pub id: String,
    pub kind: String,
    pub time: String,
    pub attributes: BTreeMap<String, Value>,
    pub object_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OcelLog {
    pub version: &'static str, // "2.0"
    pub objects: Vec<OcelObject>,
    pub events: Vec<OcelEvent>,
}

impl OcelLog {
    fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "objects": self.objects.iter().map(|o| json!({"id": o.id, "kind": o.kind})).collect::<Vec<_>>(),
            "events": self.events.iter().map(|e| json!({
                "id": e.id,
                "kind": e.kind,
                "time": e.time,
                "attributes": e.attributes,
                "object_ids": e.object_ids,
            })).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReceiptedOcelLog {
    pub log: OcelLog,
    pub construct_digest: String,
    pub receipt: Receipt,
}

// ---------------------------------------------------------------------------
// CONSTRUCT manufacture and admission
// ---------------------------------------------------------------------------

pub struct ConstructSources {
    pub o_star: Value,
    pub config_graph: Value,
    pub ontology: Value,
}

#[derive(Debug, Clone)]
pub struct ConstructArtifact {
    pub kind: &'static str, // "CASTLE_CONSTRUCT_V1"
    pub algorithm: &'static str, // "BLAKE3-256"
    pub subject: String,
    pub authority: String,
    pub o_star_digest: String,
    pub config_graph_digest: String,
    pub ontology_digest: String,
    pub process_digest: String,
    pub replay_identity_digest: String,
    pub allowed_transition_ids: Vec<String>,
    pub max_steps: u32,
    pub expires_at_epoch_ms: i64,
}

impl ConstructArtifact {
    fn to_json(&self) -> Value {
        json!({
            "kind": self.kind,
            "algorithm": self.algorithm,
            "subject": self.subject,
            "authority": self.authority,
            "o_star_digest": self.o_star_digest,
            "config_graph_digest": self.config_graph_digest,
            "ontology_digest": self.ontology_digest,
            "process_digest": self.process_digest,
            "replay_identity_digest": self.replay_identity_digest,
            "allowed_transition_ids": self.allowed_transition_ids,
            "max_steps": self.max_steps,
            "expires_at_epoch_ms": self.expires_at_epoch_ms,
        })
    }
}

pub struct ConstructSourceReceipts {
    pub o_star: Receipt,
    pub config_graph: Receipt,
    pub ontology: Receipt,
    pub process: Receipt,
}

impl ConstructSourceReceipts {
    fn parent_digests(&self) -> Vec<String> {
        let mut v = vec![
            self.o_star.artifact_digest.clone(),
            self.config_graph.artifact_digest.clone(),
            self.ontology.artifact_digest.clone(),
            self.process.artifact_digest.clone(),
        ];
        v.sort();
        v
    }
}

pub struct ConstructCapability {
    pub sources: ConstructSources,
    pub artifact: ConstructArtifact,
    pub source_receipts: ConstructSourceReceipts,
    pub receipt: Receipt,
}

pub struct ConstructRequest {
    pub subject: String,
    pub authority: String,
    pub o_star: Value,
    pub config_graph: Value,
    pub ontology: Value,
    pub process: PowlProcess,
    pub envelope: TestEnvelope,
}

pub struct ConstructTrustPolicy {
    pub trusted_origin_key_ids: BTreeSet<String>,
    pub allowed_authorities: BTreeSet<String>,
}

mod sealed {
    /// Zero-sized, module-private brand. Rust's privacy is the equivalent of the TS
    /// `Symbol`-branded admission: `ConstructAdmission` can only be constructed from
    /// inside this module, so `admit_construct_for_do` is the sole manufacturer.
    #[derive(Debug, Clone, Copy)]
    pub struct AdmissionBrand;
}

#[derive(Debug, Clone)]
pub struct ConstructAdmission {
    pub standing: &'static str, // "ALIVE"
    pub construct_digest: String,
    pub process_digest: String,
    pub o_star_digest: String,
    pub config_graph_digest: String,
    pub ontology_digest: String,
    pub replay_identity_digest: String,
    pub subject: String,
    pub authority: String,
    pub allowed_transition_ids: Vec<String>,
    pub max_steps: u32,
    pub expires_at_epoch_ms: i64,
    _brand: sealed::AdmissionBrand,
}

pub struct DoAuthorizationContext<'a> {
    pub admission: &'a ConstructAdmission,
    pub blake3: &'a dyn Blake3Provider,
    pub receipt_signer: &'a dyn ReceiptSigner,
    pub now: Box<dyn Fn() -> i64 + 'a>,
}

fn build_construct_artifact(request: &ConstructRequest, source_receipts: &ConstructSourceReceipts, blake3: &dyn Blake3Provider) -> Result<ConstructArtifact, String> {
    let mut allowed_transition_ids: Vec<String> = request.envelope.allowed_transition_ids.iter().cloned().collect();
    allowed_transition_ids.sort();
    let replay_payload = json!({
        "subject": request.subject,
        "authority": request.authority,
        "o_star_digest": source_receipts.o_star.artifact_digest,
        "config_graph_digest": source_receipts.config_graph.artifact_digest,
        "ontology_digest": source_receipts.ontology.artifact_digest,
        "process_digest": source_receipts.process.artifact_digest,
        "allowed_transition_ids": allowed_transition_ids,
        "max_steps": request.envelope.max_steps,
        "expires_at_epoch_ms": request.envelope.expires_at_epoch_ms,
    });
    let replay_identity_digest = digest_canonical(&replay_payload, blake3)?;
    Ok(ConstructArtifact {
        kind: "CASTLE_CONSTRUCT_V1",
        algorithm: "BLAKE3-256",
        subject: request.subject.clone(),
        authority: request.authority.clone(),
        o_star_digest: source_receipts.o_star.artifact_digest.clone(),
        config_graph_digest: source_receipts.config_graph.artifact_digest.clone(),
        ontology_digest: source_receipts.ontology.artifact_digest.clone(),
        process_digest: source_receipts.process.artifact_digest.clone(),
        replay_identity_digest,
        allowed_transition_ids,
        max_steps: request.envelope.max_steps,
        expires_at_epoch_ms: request.envelope.expires_at_epoch_ms,
    })
}

/// DfCM CONSTRUCT manufacture. This creates no execution authority by itself: only
/// `admit_construct_for_do` can turn a cryptographically valid construction into an
/// opaque DO capability.
pub fn manufacture_construct_capability(request: ConstructRequest, blake3: &dyn Blake3Provider, signer: &dyn ReceiptSigner) -> Result<ConstructCapability, String> {
    let _span = tracing::info_span!(
        "manufacture_construct_capability",
        subject = %request.subject,
        authority = %request.authority,
    )
    .entered();
    if request.subject.is_empty() || request.envelope.system_id != request.subject {
        return Err("REFUSED:CONSTRUCT_SUBJECT_MISMATCH".to_string());
    }
    if request.authority.is_empty() {
        return Err("REFUSED:MISSING_CONSTRUCT_AUTHORITY".to_string());
    }
    // max_steps is a u32, so "integer >= 0" is already guaranteed by the type.
    let process_transitions: BTreeSet<String> = request.process.activities.iter().map(|a| a.transition_id.clone()).collect();
    let mut allowed: Vec<String> = request.envelope.allowed_transition_ids.iter().cloned().collect();
    allowed.sort();
    if process_transitions.iter().any(|t| !request.envelope.allowed_transition_ids.contains(t)) {
        return Err("REFUSED:CONSTRUCT_PROCESS_EXCEEDS_BOUNDS".to_string());
    }
    if request.process.activities.len() as u32 > request.envelope.max_steps {
        return Err("REFUSED:CONSTRUCT_PROCESS_EXCEEDS_STEP_BOUND".to_string());
    }

    let source_receipts = ConstructSourceReceipts {
        o_star: create_receipt(&request.o_star, EpistemicClass::Constructed, &request.subject, &[], blake3, signer)?,
        config_graph: create_receipt(&request.config_graph, EpistemicClass::Constructed, &request.subject, &[], blake3, signer)?,
        ontology: create_receipt(&request.ontology, EpistemicClass::Constructed, &request.subject, &[], blake3, signer)?,
        process: create_receipt(&request.process.to_json(), EpistemicClass::Constructed, &request.subject, &[], blake3, signer)?,
    };
    let artifact = build_construct_artifact(&request, &source_receipts, blake3)?;
    if !same_strings(&artifact.allowed_transition_ids, &allowed) {
        return Err("REFUSED:NONDETERMINISTIC_CONSTRUCT_BOUNDS".to_string());
    }
    let parent_digests = source_receipts.parent_digests();
    let receipt = create_receipt(&artifact.to_json(), EpistemicClass::Constructed, &request.subject, &parent_digests, blake3, signer)?;
    Ok(ConstructCapability {
        sources: ConstructSources { o_star: request.o_star, config_graph: request.config_graph, ontology: request.ontology },
        artifact,
        source_receipts,
        receipt,
    })
}

pub fn admit_construct_for_do(
    capability: &ConstructCapability,
    process: &PowlProcess,
    envelope: &TestEnvelope,
    blake3: &dyn Blake3Provider,
    verifier: &dyn ReceiptVerifier,
    policy: &ConstructTrustPolicy,
    now: impl Fn() -> i64,
) -> Result<ConstructAdmission, String> {
    let _span = tracing::info_span!(
        "admit_construct_for_do",
        subject = %capability.artifact.subject,
        authority = %capability.artifact.authority,
    )
    .entered();
    let refuse = |reason: &str| -> Result<ConstructAdmission, String> {
        tracing::event!(tracing::Level::WARN, reason, "construct admission refused");
        Err(format!("REFUSED:{reason}"))
    };
    let artifact = &capability.artifact;
    if artifact.kind != "CASTLE_CONSTRUCT_V1" || artifact.algorithm != "BLAKE3-256" {
        return refuse("INVALID_CONSTRUCT_KIND");
    }
    if !policy.allowed_authorities.contains(&artifact.authority) {
        return refuse("CONSTRUCT_AUTHORITY_NOT_ADMITTED");
    }
    if artifact.subject != envelope.system_id || capability.receipt.subject != envelope.system_id {
        return refuse("CONSTRUCT_SUBJECT_MISMATCH");
    }
    let now_ms = now();
    if now_ms > artifact.expires_at_epoch_ms || now_ms > envelope.expires_at_epoch_ms {
        return refuse("CONSTRUCT_EXPIRED");
    }

    let process_json = process.to_json();
    let checks: [(&Value, &Receipt); 4] = [
        (&capability.sources.o_star, &capability.source_receipts.o_star),
        (&capability.sources.config_graph, &capability.source_receipts.config_graph),
        (&capability.sources.ontology, &capability.source_receipts.ontology),
        (&process_json, &capability.source_receipts.process),
    ];
    for (source, receipt) in checks.iter() {
        if receipt.subject != artifact.subject || receipt.epistemic_class != EpistemicClass::Constructed {
            return refuse("INVALID_CONSTRUCT_PARENT");
        }
        if !verify_receipt(source, receipt, blake3, verifier, &policy.trusted_origin_key_ids) {
            return refuse("UNVERIFIED_CONSTRUCT_PARENT");
        }
    }

    let expected_request = ConstructRequest {
        subject: artifact.subject.clone(),
        authority: artifact.authority.clone(),
        o_star: capability.sources.o_star.clone(),
        config_graph: capability.sources.config_graph.clone(),
        ontology: capability.sources.ontology.clone(),
        process: process.clone(),
        envelope: envelope.clone(),
    };
    let expected_artifact = match build_construct_artifact(&expected_request, &capability.source_receipts, blake3) {
        Ok(a) => a,
        Err(_) => return refuse("CONSTRUCT_BINDING_MISMATCH"),
    };
    if canonical_json(&expected_artifact.to_json()) != canonical_json(&artifact.to_json()) {
        return refuse("CONSTRUCT_BINDING_MISMATCH");
    }

    let parent_digests = capability.source_receipts.parent_digests();
    let mut receipt_parents = capability.receipt.parent_digests.clone();
    receipt_parents.sort();
    if !same_strings(&receipt_parents, &parent_digests) {
        return refuse("CONSTRUCT_PARENT_CHAIN_MISMATCH");
    }
    if capability.receipt.epistemic_class != EpistemicClass::Constructed {
        return refuse("INVALID_CONSTRUCT_RECEIPT_CLASS");
    }
    if !verify_receipt(&artifact.to_json(), &capability.receipt, blake3, verifier, &policy.trusted_origin_key_ids) {
        return refuse("UNVERIFIED_CONSTRUCT_RECEIPT");
    }

    let process_digest = match digest_canonical(&process.to_json(), blake3) {
        Ok(d) => d,
        Err(_) => return refuse("PROCESS_DIGEST_MISMATCH"),
    };
    if process_digest != artifact.process_digest {
        return refuse("PROCESS_DIGEST_MISMATCH");
    }
    let mut envelope_allowed: Vec<String> = envelope.allowed_transition_ids.iter().cloned().collect();
    envelope_allowed.sort();
    if !same_strings(&envelope_allowed, &artifact.allowed_transition_ids) {
        return refuse("CONSTRUCT_BOUND_MISMATCH");
    }
    if envelope.max_steps != artifact.max_steps || envelope.expires_at_epoch_ms != artifact.expires_at_epoch_ms {
        return refuse("CONSTRUCT_BOUND_MISMATCH");
    }
    if process.activities.iter().any(|a| !envelope.allowed_transition_ids.contains(&a.transition_id)) {
        return refuse("PROCESS_OUTSIDE_CONSTRUCT_BOUNDS");
    }

    Ok(ConstructAdmission {
        standing: "ALIVE",
        construct_digest: capability.receipt.artifact_digest.clone(),
        process_digest: artifact.process_digest.clone(),
        o_star_digest: artifact.o_star_digest.clone(),
        config_graph_digest: artifact.config_graph_digest.clone(),
        ontology_digest: artifact.ontology_digest.clone(),
        replay_identity_digest: artifact.replay_identity_digest.clone(),
        subject: artifact.subject.clone(),
        authority: artifact.authority.clone(),
        allowed_transition_ids: artifact.allowed_transition_ids.clone(),
        max_steps: artifact.max_steps,
        expires_at_epoch_ms: artifact.expires_at_epoch_ms,
        _brand: sealed::AdmissionBrand,
    })
}

/// Exclusive DO path. There is intentionally no unreceipted GymAct execution path.
/// The `ConstructAdmission` must have been manufactured by `admit_construct_for_do`
/// (enforced structurally: its private brand field cannot be constructed elsewhere).
pub async fn execute_powl_with_gym_act(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    gymact: &dyn GymActAdapter,
    authorization: DoAuthorizationContext<'_>,
) -> Result<ReceiptedOcelLog, String> {
    let span = tracing::info_span!(
        "execute_powl_with_gym_act",
        subject = %state.system_id,
        process_id = %process.id,
        event_count = tracing::field::Empty,
    );
    async move {
    let admission = authorization.admission;
    let now = authorization.now;
    if admission.standing != "ALIVE" {
        return Err("REFUSED:CONSTRUCT_NOT_ALIVE".to_string());
    }
    if envelope.system_id != state.system_id || admission.subject != state.system_id {
        return Err("REFUSED: envelope subject mismatch".to_string());
    }
    let now_ms = now();
    if now_ms > envelope.expires_at_epoch_ms || now_ms > admission.expires_at_epoch_ms {
        return Err("REFUSED: envelope expired".to_string());
    }
    let process_digest = digest_canonical(&process.to_json(), authorization.blake3)?;
    if process_digest != admission.process_digest {
        return Err("REFUSED:PROCESS_DIGEST_MISMATCH".to_string());
    }
    let mut envelope_allowed: Vec<String> = envelope.allowed_transition_ids.iter().cloned().collect();
    envelope_allowed.sort();
    if !same_strings(&envelope_allowed, &admission.allowed_transition_ids) {
        return Err("REFUSED:CONSTRUCT_BOUND_MISMATCH".to_string());
    }
    if envelope.max_steps != admission.max_steps || envelope.expires_at_epoch_ms != admission.expires_at_epoch_ms {
        return Err("REFUSED:CONSTRUCT_BOUND_MISMATCH".to_string());
    }

    let mut completed: BTreeSet<String> = BTreeSet::new();
    let mut objects: BTreeMap<String, OcelObject> = BTreeMap::new();
    objects.insert(state.system_id.clone(), OcelObject { id: state.system_id.clone(), kind: "System".to_string() });
    let mut events: Vec<OcelEvent> = Vec::new();
    let mut steps: u32 = 0;

    while completed.len() < process.activities.len() {
        let enabled = enabled_activities(process, &completed);
        if enabled.is_empty() {
            return Err("REFUSED: POWL deadlock or cyclic precedence".to_string());
        }
        if steps + enabled.len() as u32 > envelope.max_steps {
            return Err("REFUSED: max step budget exceeded".to_string());
        }

        for activity in &enabled {
            if !envelope.allowed_transition_ids.contains(&activity.transition_id) || !admission.allowed_transition_ids.contains(&activity.transition_id) {
                return Err(format!("REFUSED: transition not admitted: {}", activity.transition_id));
            }
        }

        for activity in &enabled {
            let permit = ActuationPermit {
                construct_digest: admission.construct_digest.clone(),
                process_digest: admission.process_digest.clone(),
                subject: admission.subject.clone(),
                authority: admission.authority.clone(),
                transition_id: activity.transition_id.clone(),
                expires_at_epoch_ms: admission.expires_at_epoch_ms,
            };
            let result = gymact.execute(activity, state, &permit).await;
            if result.transition_id != activity.transition_id {
                return Err(format!("REFUSED: GymAct transition receipt mismatch {}", activity.transition_id));
            }
            if result.status != GymActStatus::Observed {
                return Err(format!("REFUSED: GymAct refused {}", activity.transition_id));
            }
            for object in &result.objects {
                objects.insert(object.id.clone(), object.clone());
            }
            let mut attributes = BTreeMap::new();
            attributes.insert("epistemic_class".to_string(), json!("OBSERVED"));
            attributes.insert("construct_digest".to_string(), json!(admission.construct_digest));
            attributes.insert("process_digest".to_string(), json!(admission.process_digest));
            attributes.insert("config_graph_digest".to_string(), json!(admission.config_graph_digest));
            attributes.insert("o_star_digest".to_string(), json!(admission.o_star_digest));
            attributes.insert("ontology_digest".to_string(), json!(admission.ontology_digest));
            attributes.insert("replay_identity_digest".to_string(), json!(admission.replay_identity_digest));
            attributes.insert("authority".to_string(), json!(admission.authority));
            for (k, v) in &result.attributes {
                attributes.insert(k.clone(), v.clone());
            }
            let mut object_ids: Vec<String> = vec![state.system_id.clone()];
            object_ids.extend(result.objects.iter().map(|o| o.id.clone()));
            object_ids.sort();
            events.push(OcelEvent {
                id: format!("event:{}:{}", events.len() + 1, activity.transition_id),
                kind: activity.transition_id.clone(),
                time: format!("{}", now()), // epoch-ms string stamp; callers needing RFC3339 wrap `now`
                attributes,
                object_ids,
            });
            completed.insert(activity.id.clone());
            steps += 1;
        }
    }

    let log = OcelLog {
        version: "2.0",
        objects: objects.into_values().collect(),
        events,
    };
    let receipt = create_receipt(&log.to_json(), EpistemicClass::Observed, &state.system_id, &[admission.construct_digest.clone()], authorization.blake3, authorization.receipt_signer)?;
    tracing::Span::current().record("event_count", log.events.len());
    Ok(ReceiptedOcelLog { log, construct_digest: admission.construct_digest.clone(), receipt })
    }
    .instrument(span)
    .await
}

// ---------------------------------------------------------------------------
// Compiled adversarial classes / matching
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CompiledAdversarialClass {
    pub key: String,
    pub goal: AdversarialGoal,
    pub vulnerability: VulnerabilityCondition,
    pub process: PowlProcess,
}

pub async fn compile_adversarial_classes(goals: &[AdversarialGoal], rules: &[TransitionRule], planners: &[Box<dyn Planner>]) -> Vec<CompiledAdversarialClass> {
    let mut compiled: Vec<CompiledAdversarialClass> = Vec::new();
    let mut ordered_goals: Vec<&AdversarialGoal> = goals.iter().collect();
    ordered_goals.sort_by(|a, b| b.consequence.cmp(&a.consequence).then_with(|| a.id.cmp(&b.id)));
    for goal in ordered_goals {
        let vulnerabilities = derive_vulnerabilities(goal, rules, 32);
        for vulnerability in vulnerabilities {
            let problem = PlanningProblem { goal, vulnerability: &vulnerability, rules };
            let candidates = run_planner_ensemble(&problem, planners).await;
            if let Some(selected) = candidates.into_iter().next() {
                let predicate_refs: Vec<&str> = vulnerability.predicates.iter().map(|s| s.as_str()).collect();
                compiled.push(CompiledAdversarialClass {
                    key: format!("{}|{}", goal.id, set_key(predicate_refs)),
                    goal: goal.clone(),
                    vulnerability,
                    process: selected.process,
                });
            }
        }
    }
    compiled.sort_by(|a, b| a.key.cmp(&b.key));
    compiled
}

#[must_use]
pub fn match_compiled_classes<'a>(classes: &'a [CompiledAdversarialClass], facts: &BTreeSet<Predicate>) -> Vec<&'a CompiledAdversarialClass> {
    let mut matched: Vec<&CompiledAdversarialClass> = classes.iter().filter(|c| c.vulnerability.predicates.iter().all(|p| facts.contains(p))).collect();
    matched.sort_by(|a, b| b.goal.consequence.cmp(&a.goal.consequence).then_with(|| a.key.cmp(&b.key)));
    matched
}

// ---------------------------------------------------------------------------
// Zero-day observations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ZeroDayObservation {
    pub dependency_id: String,
    pub capability: String,
}

#[derive(Debug, Clone)]
pub struct ZeroDayImpact {
    pub observation: ZeroDayObservation,
    pub impacted_dependencies: Vec<String>,
    pub newly_admitted_fact: Predicate,
}

pub fn apply_zero_day_observation(graph: &DependencyGraph, observation: ZeroDayObservation) -> Result<ZeroDayImpact, String> {
    if !graph.nodes.contains_key(&observation.dependency_id) {
        return Err(format!("unknown dependency: {}", observation.dependency_id));
    }
    let impacted_dependencies = graph.impacted_closure([observation.dependency_id.as_str()]);
    let newly_admitted_fact = format!("capability:{}:{}", observation.dependency_id, observation.capability);
    Ok(ZeroDayImpact { observation, impacted_dependencies, newly_admitted_fact })
}

// ---------------------------------------------------------------------------
// Property tests: canonical_json digest-stability (additive, private-fn coverage)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod planner_ensemble_tests {
    use super::*;

    /// Real second planner in the ensemble, VISION.md gap #2: proves
    /// `run_planner_ensemble`'s scoring/selection logic generalizes across
    /// TWO structurally different planners, not just `WitnessPlanner` alone.
    ///
    /// `WitnessPlanner` compiles the fixed `witness_transitions` list into a
    /// data-dependency partial order (score = activities.len() +
    /// predicates.len()). `AutofdeLabPlanner` shells out to a real Python
    /// subprocess (`plan_astar.py`) that runs a genuine, independent A*
    /// forward search over the same `TransitionRule` set (score =
    /// path length) -- a different algorithm producing a differently
    /// shaped/ordered `PowlProcess`. This is a real subprocess call against a
    /// real script on disk, not a mock of the `Planner` trait: per Chicago
    /// testing discipline, invoking the actual autofde-lab-side process is
    /// the real collaborator here, and is used directly rather than faked.
    #[tokio::test]
    async fn ensemble_selects_between_witness_and_autofde_lab_planners() {
        let goal = AdversarialGoal { id: "goal:priv-esc".to_string(), predicate: "p_root".to_string(), consequence: 9 };
        let vulnerability = VulnerabilityCondition {
            goal_id: goal.id.clone(),
            predicates: vec!["p_root".to_string()],
            witness_transitions: vec!["rule:gain-shell".to_string(), "rule:escalate".to_string()],
        };
        let rules = vec![
            TransitionRule {
                id: "rule:gain-shell".to_string(),
                preconditions: vec![],
                effects: vec!["p_shell".to_string()],
                cost: Some(1.0),
                planner_hint: None,
            },
            TransitionRule {
                id: "rule:escalate".to_string(),
                preconditions: vec!["p_shell".to_string()],
                effects: vec!["p_root".to_string()],
                cost: Some(1.0),
                planner_hint: None,
            },
        ];

        let problem = PlanningProblem { goal: &goal, vulnerability: &vulnerability, rules: &rules };

        let script_path = std::path::PathBuf::from("/Users/sac/autofde-lab/scripts/castle_bridge/plan_astar.py");
        assert!(script_path.exists(), "autofde-lab bridge script must exist on disk for a real subprocess call: {}", script_path.display());

        let planners: Vec<Box<dyn Planner>> = vec![
            Box::new(WitnessPlanner::default()),
            Box::new(AutofdeLabPlanner { script_path, ..AutofdeLabPlanner::default() }),
        ];

        let candidates = run_planner_ensemble(&problem, &planners).await;

        // Both planners are applicable and both must have genuinely contributed.
        let planner_ids: std::collections::BTreeSet<&str> = candidates.iter().map(|c| c.planner_id.as_str()).collect();
        assert!(planner_ids.contains("witness-partial-order"), "witness planner produced no candidates: {candidates:?}");
        assert!(planner_ids.contains("autofde-lab-astar"), "autofde-lab subprocess planner produced no candidates: {candidates:?}");
        assert_eq!(candidates.len(), 2, "expected exactly one candidate from each of the two structurally different planners: {candidates:?}");

        // Real selection: the combined list is sorted ascending by score, so
        // the ensemble's ranking (not planner registration order) determines
        // who is first -- this is the actual scoring/selection logic being
        // exercised, not an assumed outcome.
        assert!(candidates.windows(2).all(|w| w[0].score <= w[1].score), "ensemble output must be score-ascending: {candidates:?}");

        let witness_candidate = candidates.iter().find(|c| c.planner_id == "witness-partial-order").unwrap();
        let autofde_candidate = candidates.iter().find(|c| c.planner_id == "autofde-lab-astar").unwrap();

        // Structurally different processes proving these are two genuinely
        // different planners, not one planner's output relabeled: witness
        // compiles a fixed process id prefixed "powl:", autofde-lab's bridge
        // emits a distinctly-prefixed "powl:astar:" id from its own search.
        assert!(witness_candidate.process.id.starts_with("powl:goal:priv-esc"));
        assert!(autofde_candidate.process.id.starts_with("powl:astar:goal:priv-esc"));

        // Witness score = activities.len() (2) + predicates.len() (1) = 3.
        assert_eq!(witness_candidate.score, 3);
        // A* score = real shortest-path length found by search = 2 steps.
        assert_eq!(autofde_candidate.score, 2);

        // The ensemble's ascending-score ranking genuinely selects the
        // lower-scoring autofde-lab plan first over the witness plan --
        // this is the real cross-planner selection behavior under test.
        assert_eq!(candidates[0].planner_id, "autofde-lab-astar");
        assert_eq!(candidates[1].planner_id, "witness-partial-order");
    }
}

#[cfg(test)]
mod canonical_json_proptests {
    use super::canonical_json;
    use proptest::prelude::*;
    use serde_json::Value;

    /// Small fixed key alphabet keeps generated objects bounded while still
    /// covering multiple distinct keys per object.
    const KEYS: [&str; 5] = ["alpha", "beta", "gamma", "delta", "epsilon"];

    fn key_strategy() -> impl Strategy<Value = String> {
        (0..KEYS.len()).prop_map(|i| KEYS[i].to_string())
    }

    /// A small arbitrary set of distinct (key, value) pairs, generated once and
    /// then rendered into JSON text in two different key orders below — this is
    /// the actual "arbitrary key insertion order" input the digest-stability
    /// invariant must survive.
    fn pairs_strategy() -> impl Strategy<Value = Vec<(String, i64)>> {
        prop::collection::vec((key_strategy(), any::<i64>()), 0..=KEYS.len()).prop_map(|mut pairs| {
            let mut seen = std::collections::HashSet::new();
            pairs.retain(|(k, _)| seen.insert(k.clone()));
            pairs
        })
    }

    proptest! {
        /// `canonical_json` must produce byte-identical output for the same
        /// logical object regardless of the key order it was textually written
        /// in — this is the exact property the CONSTRUCT/receipt digest chains
        /// depend on for stability (see `CLAUDE.md`'s "two independent receipt
        /// systems" section: any new digestable artifact goes through this
        /// function precisely because it is NOT allowed to depend on serde's own
        /// key ordering).
        #[test]
        fn canonical_json_is_key_order_independent(pairs in pairs_strategy()) {
            let forward_json = {
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };
            let mut reversed = pairs.clone();
            reversed.reverse();
            let reversed_json = {
                let parts: Vec<String> = reversed.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };

            let forward_value: Value = serde_json::from_str(&forward_json).expect("forward JSON must parse");
            let reversed_value: Value = serde_json::from_str(&reversed_json).expect("reversed JSON must parse");

            prop_assert_eq!(canonical_json(&forward_value), canonical_json(&reversed_value));
        }

        /// `canonical_json` is a pure, deterministic function of its input value.
        #[test]
        fn canonical_json_is_deterministic(pairs in pairs_strategy()) {
            let json_text = {
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };
            let value: Value = serde_json::from_str(&json_text).expect("JSON must parse");
            prop_assert_eq!(canonical_json(&value), canonical_json(&value));
        }
    }
}
