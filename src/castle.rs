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
