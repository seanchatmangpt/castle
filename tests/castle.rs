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
