//! Port of `test/castle.test.ts`. Uses the real `blake3` crate and a real
//! Ed25519 keypair (via `ed25519-dalek`) as the receipt signer/verifier rather
//! than fakes — Chicago-style: real collaborators, state-based assertions.

use std::collections::BTreeSet;

use castle::castle::*;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::json;

struct RealBlake3;
impl Blake3Provider for RealBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

struct Ed25519Signer {
    key_id: String,
    signing_key: SigningKey,
}
impl ReceiptSigner for Ed25519Signer {
    fn key_id(&self) -> &str {
        &self.key_id
    }
    fn sign_digest(&self, digest_hex: &str) -> String {
        let bytes = (0..digest_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&digest_hex[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();
        let sig = self.signing_key.sign(&bytes);
        hex_encode(&sig.to_bytes())
    }
}

struct Ed25519Verifier {
    key_id: String,
    verifying_key: VerifyingKey,
}
impl ReceiptVerifier for Ed25519Verifier {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool {
        if key_id != self.key_id {
            return false;
        }
        let digest_bytes: Vec<u8> = match (0..digest_hex.len()).step_by(2).map(|i| u8::from_str_radix(&digest_hex[i..i + 2], 16)).collect() {
            Ok(v) => v,
            Err(_) => return false,
        };
        let sig_bytes: Vec<u8> = match (0..signature.len()).step_by(2).map(|i| u8::from_str_radix(&signature[i..i + 2], 16)).collect() {
            Ok(v) => v,
            Err(_) => return false,
        };
        if sig_bytes.len() != 64 {
            return false;
        }
        let sig_array: [u8; 64] = sig_bytes.try_into().unwrap();
        let sig = ed25519_dalek::Signature::from_bytes(&sig_array);
        self.verifying_key.verify(&digest_bytes, &sig).is_ok()
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn goal() -> AdversarialGoal {
    AdversarialGoal { id: "unauthorized-authority".to_string(), predicate: "goal:unauthorized-authority".to_string(), consequence: 100 }
}

fn rules() -> Vec<TransitionRule> {
    vec![
        TransitionRule {
            id: "assume-control-plane".to_string(),
            preconditions: vec!["dep:auth-service:execute".to_string(), "trust:service-account".to_string()],
            effects: vec!["goal:unauthorized-authority".to_string()],
            cost: None,
            planner_hint: None,
        },
        TransitionRule {
            id: "execute-auth-service".to_string(),
            preconditions: vec!["capability:auth-lib:execute".to_string()],
            effects: vec!["dep:auth-service:execute".to_string()],
            cost: None,
            planner_hint: None,
        },
    ]
}

struct Fixture {
    blake3: RealBlake3,
    signer: Ed25519Signer,
    verifier: Ed25519Verifier,
}

fn fixture() -> Fixture {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let verifying_key = signing_key.verifying_key();
    Fixture {
        blake3: RealBlake3,
        signer: Ed25519Signer { key_id: "construct-root".to_string(), signing_key },
        verifier: Ed25519Verifier { key_id: "construct-root".to_string(), verifying_key },
    }
}

fn construct_for(fx: &Fixture, process: PowlProcess, envelope: TestEnvelope) -> (ConstructCapability, ConstructAdmission) {
    let capability = manufacture_construct_capability(
        ConstructRequest {
            subject: envelope.system_id.clone(),
            authority: "defensive-test".to_string(),
            o_star: json!({ "admittedSubject": envelope.system_id }),
            config_graph: json!({ "root": "configs/CONSTRUCT", "zeroUnreceiptedActuation": true }),
            ontology: json!({ "version": "castle-pack-v1" }),
            process: process.clone(),
            envelope: envelope.clone(),
        },
        &fx.blake3,
        &fx.signer,
    )
    .expect("construct manufactures");
    let policy = ConstructTrustPolicy {
        trusted_origin_key_ids: BTreeSet::from(["construct-root".to_string()]),
        allowed_authorities: BTreeSet::from(["defensive-test".to_string()]),
    };
    let admission = admit_construct_for_do(&capability, &process, &envelope, &fx.blake3, &fx.verifier, &policy, || 1).expect("construct admits");
    (capability, admission)
}

#[test]
fn dfcm_derives_minimal_vulnerability_conditions_backward_from_the_goal() {
    let vulnerabilities = derive_vulnerabilities(&goal(), &rules(), 32);
    assert_eq!(vulnerabilities.len(), 1);
    assert_eq!(vulnerabilities[0].goal_id, "unauthorized-authority");
    assert_eq!(vulnerabilities[0].predicates, vec!["capability:auth-lib:execute".to_string(), "trust:service-account".to_string()]);
    assert_eq!(vulnerabilities[0].witness_transitions, vec!["execute-auth-service".to_string(), "assume-control-plane".to_string()]);
}

#[test]
fn dependency_construct_computes_downstream_impact_without_claiming_observation() {
    let graph = DependencyGraph::new(
        vec![
            DependencyNode { id: "auth-lib".to_string(), kind: "package".to_string() },
            DependencyNode { id: "auth-service".to_string(), kind: "service".to_string() },
            DependencyNode { id: "control-plane".to_string(), kind: "service".to_string() },
        ],
        vec![
            DependencyEdge { from: "auth-lib".to_string(), to: "auth-service".to_string(), relation: "dependsOn".to_string() },
            DependencyEdge { from: "auth-service".to_string(), to: "control-plane".to_string(), relation: "calls".to_string() },
        ],
    )
    .unwrap();
    let constructed = graph.construct_compromise("auth-lib", "execute").unwrap();
    assert_eq!(constructed.epistemic_class, "COUNTERFACTUAL");
    assert_eq!(constructed.impacted, vec!["auth-lib".to_string(), "auth-service".to_string(), "control-plane".to_string()]);
}

#[tokio::test]
async fn planner_ensemble_compiles_the_witness_into_a_powl_causal_partial_order() {
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    assert_eq!(classes.len(), 1);
    let process = &classes[0].process;
    let first = process.activities.iter().find(|a| a.transition_id == "execute-auth-service").unwrap();
    let second = process.activities.iter().find(|a| a.transition_id == "assume-control-plane").unwrap();
    assert!(first.predecessors.is_empty());
    assert_eq!(second.predecessors, vec![first.id.clone()]);
}

#[tokio::test]
async fn known_structural_vulnerability_selects_a_precompiled_adversarial_class() {
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    let facts = BTreeSet::from(["capability:auth-lib:execute".to_string(), "trust:service-account".to_string()]);
    let matches = match_compiled_classes(&classes, &facts);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].goal.id, goal().id);
}

struct RecordingGymAct;
#[async_trait::async_trait]
impl GymActAdapter for RecordingGymAct {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        assert_eq!(permit.transition_id, activity.transition_id);
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Observed,
            objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "TestObservation".to_string() }],
            attributes: Default::default(),
        }
    }
}

#[tokio::test]
async fn gymact_executes_only_through_admitted_construct_and_returns_receipted_ocel_v2_evidence() {
    let fx = fixture();
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    let process = classes[0].process.clone();
    let envelope = TestEnvelope {
        system_id: "system:self".to_string(),
        allowed_transition_ids: BTreeSet::from(["execute-auth-service".to_string(), "assume-control-plane".to_string()]),
        max_steps: 2,
        expires_at_epoch_ms: 10_000,
    };
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let state = WorldState { system_id: "system:self".to_string(), facts: BTreeSet::new() };
    let gymact = RecordingGymAct;
    let log = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await
    .expect("gymact executes");
    assert_eq!(log.log.version, "2.0");
    assert_eq!(log.log.events.len(), 2);
    assert_eq!(
        log.log.events.iter().map(|e| e.kind.clone()).collect::<Vec<_>>(),
        vec!["execute-auth-service".to_string(), "assume-control-plane".to_string()]
    );
    assert_eq!(log.receipt.epistemic_class, EpistemicClass::Observed);
    assert_eq!(log.receipt.parent_digests, vec![admission.construct_digest.clone()]);
    assert!(log.log.events.iter().all(|event| event.attributes.get("construct_digest") == Some(&json!(admission.construct_digest))));
}

#[tokio::test]
async fn do_refuses_envelope_mutation_after_construct() {
    let fx = fixture();
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    let process = classes[0].process.clone();
    let envelope = TestEnvelope {
        system_id: "system:self".to_string(),
        allowed_transition_ids: BTreeSet::from(["execute-auth-service".to_string(), "assume-control-plane".to_string()]),
        max_steps: 2,
        expires_at_epoch_ms: 10_000,
    };
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let state = WorldState { system_id: "system:self".to_string(), facts: BTreeSet::new() };
    let gymact = RecordingGymAct;
    let mutated_envelope = TestEnvelope { allowed_transition_ids: BTreeSet::from(["execute-auth-service".to_string()]), ..envelope.clone() };
    let err = execute_powl_with_gym_act(
        &process,
        &state,
        &mutated_envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 1) },
    )
    .await
    .unwrap_err();
    assert!(err.contains("REFUSED:CONSTRUCT_BOUND_MISMATCH"), "unexpected error: {err}");
}

#[tokio::test]
async fn construct_admission_refuses_config_mutation_process_substitution_and_untrusted_origin() {
    let fx = fixture();
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    let process = classes[0].process.clone();
    let envelope = TestEnvelope {
        system_id: "system:self".to_string(),
        allowed_transition_ids: BTreeSet::from(["execute-auth-service".to_string(), "assume-control-plane".to_string()]),
        max_steps: 2,
        expires_at_epoch_ms: 10_000,
    };
    let (mut capability, _admission) = construct_for(&fx, process.clone(), envelope.clone());

    capability.sources.o_star = json!({ "admittedSubject": "human-edited" });
    let policy = ConstructTrustPolicy {
        trusted_origin_key_ids: BTreeSet::from(["construct-root".to_string()]),
        allowed_authorities: BTreeSet::from(["defensive-test".to_string()]),
    };
    let err = admit_construct_for_do(&capability, &process, &envelope, &fx.blake3, &fx.verifier, &policy, || 1).unwrap_err();
    assert!(err.contains("REFUSED:UNVERIFIED_CONSTRUCT_PARENT"), "unexpected error: {err}");

    let (fresh_capability, _) = construct_for(&fx, process.clone(), envelope.clone());
    let substituted = PowlProcess { id: format!("{}:human-substitution", process.id), ..process.clone() };
    let err = admit_construct_for_do(&fresh_capability, &substituted, &envelope, &fx.blake3, &fx.verifier, &policy, || 1).unwrap_err();
    assert!(err.starts_with("REFUSED:"), "unexpected error: {err}");

    let untrusted_policy = ConstructTrustPolicy {
        trusted_origin_key_ids: BTreeSet::from(["other-root".to_string()]),
        allowed_authorities: BTreeSet::from(["defensive-test".to_string()]),
    };
    let err = admit_construct_for_do(&fresh_capability, &process, &envelope, &fx.blake3, &fx.verifier, &untrusted_policy, || 1).unwrap_err();
    assert!(err.contains("REFUSED:UNVERIFIED_CONSTRUCT_PARENT"), "unexpected error: {err}");
}

#[test]
fn receipt_contract_binds_artifact_subject_parents_origin_and_signature() {
    let fx = fixture();
    let receipt = create_receipt(
        &json!({ "x": 1 }),
        EpistemicClass::Constructed,
        "subject",
        &["b".repeat(64), "a".repeat(64)],
        &fx.blake3,
        &fx.signer,
    )
    .unwrap();
    assert_eq!(receipt.algorithm, "BLAKE3-256");
    assert_eq!(receipt.parent_digests, vec!["a".repeat(64), "b".repeat(64)]);
    assert_eq!(receipt.origin_key_id, "construct-root");
    assert!(fx.verifier.verify_digest(&receipt.origin_key_id, &receipt.receipt_digest, &receipt.origin_signature));
}

#[test]
fn zero_day_listener_turns_a_new_capability_fact_into_an_impacted_dependency_closure() {
    let graph = DependencyGraph::new(
        vec![
            DependencyNode { id: "auth-lib".to_string(), kind: "package".to_string() },
            DependencyNode { id: "auth-service".to_string(), kind: "service".to_string() },
            DependencyNode { id: "control-plane".to_string(), kind: "service".to_string() },
        ],
        vec![
            DependencyEdge { from: "auth-lib".to_string(), to: "auth-service".to_string(), relation: "dependsOn".to_string() },
            DependencyEdge { from: "auth-service".to_string(), to: "control-plane".to_string(), relation: "calls".to_string() },
        ],
    )
    .unwrap();
    let impact = apply_zero_day_observation(&graph, ZeroDayObservation { dependency_id: "auth-lib".to_string(), capability: "execute".to_string() }).unwrap();
    assert_eq!(impact.newly_admitted_fact, "capability:auth-lib:execute");
    assert_eq!(impact.impacted_dependencies, vec!["auth-lib".to_string(), "auth-service".to_string(), "control-plane".to_string()]);
}

#[test]
fn marketplace_generated_bindings_provide_architecture_components_and_default_goal_priorities() {
    assert_eq!(castle::generated_components().count(), 10);
    let goals = castle::default_adversarial_goals();
    assert_eq!(goals.len(), 5);
    assert_eq!(goals[0].id, "unauthorized-authority");
}

// ---------------------------------------------------------------------------
// Refusal-path adapters. These are REAL implementations of `GymActAdapter`
// with real (if simple) behavior — a refusing actuator and a misreporting
// actuator — not interaction-verifying mocks. Assertions are state-based:
// the returned Err payload and the absence of any receipted OCEL log.
// ---------------------------------------------------------------------------

/// A real actuator that declines to actuate: it returns a well-formed result
/// for the requested transition, but with `GymActStatus::Refused`.
struct RefusingGymAct;
#[async_trait::async_trait]
impl GymActAdapter for RefusingGymAct {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        assert_eq!(permit.transition_id, activity.transition_id);
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Refused,
            objects: vec![OcelObject { id: format!("object:{}", activity.transition_id), kind: "RefusedObservation".to_string() }],
            attributes: Default::default(),
        }
    }
}

/// A real actuator that reports a receipt for a DIFFERENT transition than the
/// one it was permitted to run (transition receipt substitution).
struct MisreportingGymAct;
#[async_trait::async_trait]
impl GymActAdapter for MisreportingGymAct {
    async fn execute(&self, activity: &PowlActivity, _state: &WorldState, _permit: &ActuationPermit) -> GymActResult {
        GymActResult {
            transition_id: format!("{}-substituted", activity.transition_id),
            status: GymActStatus::Observed,
            objects: vec![OcelObject { id: "object:substituted".to_string(), kind: "TestObservation".to_string() }],
            attributes: Default::default(),
        }
    }
}

async fn refusal_fixture() -> (Fixture, PowlProcess, TestEnvelope, WorldState) {
    let fx = fixture();
    let planners: Vec<Box<dyn Planner>> = vec![Box::new(WitnessPlanner::default())];
    let classes = compile_adversarial_classes(&[goal()], &rules(), &planners).await;
    let process = classes[0].process.clone();
    let envelope = TestEnvelope {
        system_id: "system:self".to_string(),
        allowed_transition_ids: BTreeSet::from(["execute-auth-service".to_string(), "assume-control-plane".to_string()]),
        max_steps: 2,
        expires_at_epoch_ms: 10_000,
    };
    let state = WorldState { system_id: "system:self".to_string(), facts: BTreeSet::new() };
    (fx, process, envelope, state)
}

#[tokio::test]
async fn do_refuses_when_the_actuator_genuinely_refuses_and_emits_no_receipted_log() {
    let (fx, process, envelope, state) = refusal_fixture().await;
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let gymact = RefusingGymAct;
    let outcome = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await;
    let err = outcome.err().expect("a refusing actuator must not yield a receipted OCEL log");
    assert_eq!(err, "REFUSED: GymAct refused execute-auth-service");
}

#[tokio::test]
async fn do_refuses_when_the_actuator_reports_a_receipt_for_a_different_transition() {
    let (fx, process, envelope, state) = refusal_fixture().await;
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let gymact = MisreportingGymAct;
    let outcome = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await;
    let err = outcome.err().expect("a substituted transition receipt must not yield a receipted OCEL log");
    assert_eq!(err, "REFUSED: GymAct transition receipt mismatch execute-auth-service");
}

// ---------------------------------------------------------------------------
// `KindClusterReadOnlyGymAct`: the crate's first non-test-double
// `GymActAdapter`, exercised against the real `kind-platform-eng-colima`
// kind cluster already running on this host. Chicago style: the real
// collaborator (a real `kubectl` binary talking to a real kind cluster) is
// used directly, not mocked. Per this repo's testing rule, a machine without
// that cluster running degrades to a named, visible skip rather than a
// silent mock substitution.
// ---------------------------------------------------------------------------

fn kind_platform_eng_colima_is_available() -> bool {
    std::process::Command::new("kubectl")
        .args(["--context", "kind-platform-eng-colima", "get", "nodes", "-o", "name"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn kind_cluster_gymact_observes_real_nodes_and_yields_a_receipted_ocel_log() {
    if !kind_platform_eng_colima_is_available() {
        eprintln!("SKIPPED: kind-platform-eng-colima context is not reachable on this host");
        return;
    }

    let fx = fixture();
    let process = PowlProcess {
        id: "process:observe-kind-cluster".to_string(),
        goal_id: "goal:observe-kind-cluster".to_string(),
        activities: vec![PowlActivity { id: "activity:observe-cluster-nodes".to_string(), transition_id: "observe-cluster-nodes".to_string(), predecessors: vec![] }],
    };
    let envelope = TestEnvelope {
        system_id: "system:kind-platform-eng-colima".to_string(),
        allowed_transition_ids: BTreeSet::from(["observe-cluster-nodes".to_string()]),
        max_steps: 1,
        expires_at_epoch_ms: 10_000,
    };
    let state = WorldState { system_id: "system:kind-platform-eng-colima".to_string(), facts: BTreeSet::new() };
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let gymact = KindClusterReadOnlyGymAct::platform_eng_colima_default();

    let outcome = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await
    .expect("a real, allowlisted, read-only kubectl query against a live kind cluster must yield a receipted OCEL log");

    // State-based assertions on the real OCEL log produced from the real
    // cluster response, not on "was kubectl called".
    assert_eq!(outcome.log.version, "2.0");
    assert!(outcome.log.objects.iter().any(|o| o.id.starts_with("k8s:kind-platform-eng-colima:")), "expected at least one real node object observed from the live cluster, got: {:?}", outcome.log.objects);
    let observe_event = outcome.log.events.iter().find(|e| e.kind == "observe-cluster-nodes").expect("expected an OCEL event for the observe-cluster-nodes transition");
    assert_eq!(observe_event.attributes.get("epistemic_class").and_then(|v| v.as_str()), Some("OBSERVED"));
}

#[tokio::test]
async fn kind_cluster_gymact_refuses_transitions_outside_its_read_only_allowlist() {
    let gymact = KindClusterReadOnlyGymAct::platform_eng_colima_default();
    let activity = PowlActivity { id: "activity:delete-everything".to_string(), transition_id: "delete-everything".to_string(), predecessors: vec![] };
    let state = WorldState { system_id: "system:kind-platform-eng-colima".to_string(), facts: BTreeSet::new() };
    let permit = ActuationPermit {
        construct_digest: "0".repeat(64),
        process_digest: "0".repeat(64),
        subject: "system:kind-platform-eng-colima".to_string(),
        authority: "defensive-test".to_string(),
        transition_id: "delete-everything".to_string(),
        expires_at_epoch_ms: 10_000,
    };
    let result = gymact.execute(&activity, &state, &permit).await;
    assert_eq!(result.status, GymActStatus::Refused, "a transition_id absent from the fixed read-only allowlist must be refused without ever invoking kubectl");
}

// ---------------------------------------------------------------------------
// `ProcessGymActAdapter`: the crate's first `GymActAdapter` backed by the
// real, already-running `gymact` service's Typer CLI (`gymact verify`),
// shelled out to as a real subprocess, exercised against the real
// `kubernetes-reconciliation` provider on the real `kind-platform-eng-colima`
// kind cluster. Chicago style: the real collaborator (the real `gymact`
// binary, which itself shells out to a real `kubectl` against a real
// cluster) is used directly, not mocked. Per this repo's testing rule, a
// host without a runnable `gymact` CLI or without the real cluster degrades
// to a named, visible skip rather than a silent mock substitution.
// ---------------------------------------------------------------------------

fn gymact_bin_path() -> String {
    std::env::var("GYMACT_BIN").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{home}/gymact/.venv/bin/gymact")
    })
}

fn process_gymact_is_available() -> bool {
    if !kind_platform_eng_colima_is_available() {
        return false;
    }
    std::process::Command::new(gymact_bin_path()).arg("version").output().map(|out| out.status.success()).unwrap_or(false)
}

#[tokio::test]
async fn process_gym_act_adapter_runs_a_real_powl_sequence_against_the_live_kubernetes_reconciliation_provider_and_yields_a_receipted_ocel_log() {
    if !process_gymact_is_available() {
        eprintln!("SKIPPED: real `gymact` CLI or kind-platform-eng-colima context is not reachable on this host");
        return;
    }

    let fx = fixture();
    let process = PowlProcess {
        id: "process:verify-kubernetes-reconciliation".to_string(),
        goal_id: "goal:verify-kubernetes-reconciliation".to_string(),
        activities: vec![PowlActivity {
            id: "activity:verify-kubernetes-reconciliation-running".to_string(),
            transition_id: "verify-kubernetes-reconciliation-running".to_string(),
            predecessors: vec![],
        }],
    };
    let envelope = TestEnvelope {
        system_id: "system:kubernetes-reconciliation".to_string(),
        allowed_transition_ids: BTreeSet::from(["verify-kubernetes-reconciliation-running".to_string()]),
        max_steps: 1,
        expires_at_epoch_ms: 60_000,
    };
    let state = WorldState { system_id: "system:kubernetes-reconciliation".to_string(), facts: BTreeSet::new() };
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let gymact = ProcessGymActAdapter::platform_eng_colima_default(gymact_bin_path());

    let outcome = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await
    .expect("a real, allowlisted `gymact verify` call against the live kubernetes-reconciliation provider must yield a receipted OCEL log");

    // State-based assertions on the real OCEL log produced from the real
    // `gymact` CLI response, not on "was gymact called".
    assert_eq!(outcome.log.version, "2.0");
    assert!(
        outcome.log.objects.iter().any(|o| o.id.starts_with("gymact:kubernetes-reconciliation:")),
        "expected a real gymact episode object, got: {:?}",
        outcome.log.objects
    );
    let verify_event = outcome.log.events.iter().find(|e| e.kind == "verify-kubernetes-reconciliation-running").expect("expected an OCEL event for the verify-kubernetes-reconciliation-running transition");
    assert_eq!(verify_event.attributes.get("epistemic_class").and_then(|v| v.as_str()), Some("OBSERVED"));
    let observed = verify_event.attributes.get("gymact_observed").expect("expected the real gymact-observed postcondition to be attached to the event");
    assert_eq!(observed.get("running").and_then(|v| v.as_bool()), Some(true), "expected the real cluster-observed postcondition to report running:true, got: {observed:?}");
    assert!(!outcome.receipt.receipt_digest.is_empty(), "expected a real, non-empty receipt digest");
}

#[tokio::test]
async fn process_gym_act_adapter_refuses_transitions_outside_its_fixed_allowlist() {
    let gymact = ProcessGymActAdapter::platform_eng_colima_default(gymact_bin_path());
    let activity = PowlActivity { id: "activity:arbitrary-provider".to_string(), transition_id: "arbitrary-provider".to_string(), predecessors: vec![] };
    let state = WorldState { system_id: "system:kubernetes-reconciliation".to_string(), facts: BTreeSet::new() };
    let permit = ActuationPermit {
        construct_digest: "0".repeat(64),
        process_digest: "0".repeat(64),
        subject: "system:kubernetes-reconciliation".to_string(),
        authority: "defensive-test".to_string(),
        transition_id: "arbitrary-provider".to_string(),
        expires_at_epoch_ms: 10_000,
    };
    let result = gymact.execute(&activity, &state, &permit).await;
    assert_eq!(result.status, GymActStatus::Refused, "a transition_id absent from the fixed verification allowlist must be refused without ever invoking gymact");
}

// ---------------------------------------------------------------------------
// `ContainerGymActAdapter`: the crate's third non-test-double `GymActAdapter`,
// and the first backed directly by a local container runtime (`docker`,
// colima-backed on this host) rather than Kubernetes. Chicago style: the
// real `docker` binary talking to the real local daemon is used directly,
// never mocked. A host without a reachable docker daemon degrades to a
// named, visible skip rather than a silent mock substitution.
// ---------------------------------------------------------------------------

fn docker_bin_path() -> String {
    std::env::var("DOCKER_BIN").unwrap_or_else(|_| "docker".to_string())
}

fn docker_daemon_is_available() -> bool {
    std::process::Command::new(docker_bin_path()).args(["info"]).output().map(|out| out.status.success()).unwrap_or(false)
}

#[tokio::test]
async fn container_gym_act_adapter_runs_a_real_powl_sequence_against_the_local_docker_daemon_and_yields_a_receipted_ocel_log() {
    if !docker_daemon_is_available() {
        eprintln!("SKIPPED: no reachable local docker daemon on this host");
        return;
    }

    let fx = fixture();
    let process = PowlProcess {
        id: "process:observe-alpine-container".to_string(),
        goal_id: "goal:observe-alpine-container".to_string(),
        activities: vec![PowlActivity { id: "activity:observe-alpine-container".to_string(), transition_id: "observe-alpine-container".to_string(), predecessors: vec![] }],
    };
    let envelope = TestEnvelope {
        system_id: "system:local-docker".to_string(),
        allowed_transition_ids: BTreeSet::from(["observe-alpine-container".to_string()]),
        max_steps: 1,
        expires_at_epoch_ms: 60_000,
    };
    let state = WorldState { system_id: "system:local-docker".to_string(), facts: BTreeSet::new() };
    let (_capability, admission) = construct_for(&fx, process.clone(), envelope.clone());
    let mut gymact = ContainerGymActAdapter::local_docker_default(docker_bin_path());
    // Match the fixture's injected test-time convention (`now: Box::new(|| 5)`
    // below) rather than the real wall clock, so the permit's
    // `expires_at_epoch_ms: 60_000` test-time budget isn't immediately
    // exceeded by `SystemTime::now()`'s real epoch-ms value.
    gymact.now_ms = || 5;

    let outcome = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &gymact,
        DoAuthorizationContext { admission: &admission, blake3: &fx.blake3, receipt_signer: &fx.signer, now: Box::new(|| 5) },
    )
    .await
    .expect("a real, allowlisted, network-isolated docker container run+inspect must yield a receipted OCEL log");

    // State-based assertions on the real OCEL log produced from the real
    // `docker inspect` response, not on "was docker called".
    assert_eq!(outcome.log.version, "2.0");
    assert!(outcome.log.objects.iter().any(|o| o.id.starts_with("docker:castle-gymact-observe-alpine-container-")), "expected a real owned throwaway container object, got: {:?}", outcome.log.objects);
    let observe_event = outcome.log.events.iter().find(|e| e.kind == "observe-alpine-container").expect("expected an OCEL event for the observe-alpine-container transition");
    assert_eq!(observe_event.attributes.get("epistemic_class").and_then(|v| v.as_str()), Some("OBSERVED"));
    let digest = observe_event.attributes.get("stdout_blake3").and_then(|v| v.as_str()).expect("expected a real BLAKE3 digest of the captured docker inspect stdout");
    assert_eq!(digest.len(), 64, "expected a real 32-byte BLAKE3 hex digest, got: {digest}");
    assert!(!outcome.receipt.receipt_digest.is_empty(), "expected a real, non-empty receipt digest");

    // Confirm the adapter's own teardown actually ran: its owned container
    // must not still be present on the real daemon after this call returns.
    let ps_output = std::process::Command::new(docker_bin_path()).args(["ps", "-a", "--filter", "name=castle-gymact-observe-alpine-container-", "--format", "{{.Names}}"]).output().expect("docker ps must run on an available daemon");
    let remaining = String::from_utf8_lossy(&ps_output.stdout);
    assert!(remaining.trim().is_empty(), "expected the adapter to have torn down its own owned container, but docker ps still shows: {remaining}");
}

#[tokio::test]
async fn container_gym_act_adapter_refuses_transitions_outside_its_fixed_image_allowlist() {
    let gymact = ContainerGymActAdapter::local_docker_default(docker_bin_path());
    let activity = PowlActivity { id: "activity:arbitrary-image".to_string(), transition_id: "arbitrary-image".to_string(), predecessors: vec![] };
    let state = WorldState { system_id: "system:local-docker".to_string(), facts: BTreeSet::new() };
    let permit = ActuationPermit {
        construct_digest: "0".repeat(64),
        process_digest: "0".repeat(64),
        subject: "system:local-docker".to_string(),
        authority: "defensive-test".to_string(),
        transition_id: "arbitrary-image".to_string(),
        expires_at_epoch_ms: 10_000,
    };
    let result = gymact.execute(&activity, &state, &permit).await;
    assert_eq!(result.status, GymActStatus::Refused, "a transition_id absent from the fixed image allowlist must be refused without ever invoking docker");
}

#[tokio::test]
async fn container_gym_act_adapter_refuses_an_already_expired_permit_without_spawning_docker() {
    let mut gymact = ContainerGymActAdapter::local_docker_default(docker_bin_path());
    gymact.now_ms = || 999_999;
    let activity = PowlActivity { id: "activity:observe-alpine-container".to_string(), transition_id: "observe-alpine-container".to_string(), predecessors: vec![] };
    let state = WorldState { system_id: "system:local-docker".to_string(), facts: BTreeSet::new() };
    let permit = ActuationPermit {
        construct_digest: "0".repeat(64),
        process_digest: "0".repeat(64),
        subject: "system:local-docker".to_string(),
        authority: "defensive-test".to_string(),
        transition_id: "observe-alpine-container".to_string(),
        expires_at_epoch_ms: 1_000,
    };
    let result = gymact.execute(&activity, &state, &permit).await;
    assert_eq!(result.status, GymActStatus::Refused, "an already-expired permit must be refused before docker is ever spawned, regardless of upstream envelope checks");
}
