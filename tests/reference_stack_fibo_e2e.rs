use std::collections::BTreeSet;

use castle::castle::{
    admit_construct_for_do, manufacture_construct_capability, Blake3Provider, ConstructRequest,
    ConstructTrustPolicy, PowlActivity, PowlProcess, ReceiptSigner, ReceiptVerifier, TestEnvelope,
    WorldState,
};
use castle::reference_stack::{
    project_beam4pm_feedback, project_fibo_payment, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY,
};
use castle::v26_8_18::{execute_command_process, CommandAdapterPolicy, CommandSpec, ReleaseStanding};
use serde_json::json;

struct RealBlake3;
impl Blake3Provider for RealBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

struct TestSigner;
impl ReceiptSigner for TestSigner {
    fn key_id(&self) -> &str { "reference-root" }
    fn sign_digest(&self, digest_hex: &str) -> String { digest_hex.to_string() }
}

struct TestVerifier;
impl ReceiptVerifier for TestVerifier {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool {
        key_id == "reference-root" && signature == digest_hex
    }
}

#[tokio::test]
async fn fibo_semantic_delta_reuses_brce_receipt_ocel_and_feedback() {
    let hook = project_fibo_payment(KnowledgeHookCandidate {
        version: 1,
        exact_subject: json!("system:fibo:payment:001"),
        source_digest: format!("sha256:{}", "c".repeat(64)),
        replay_identity: "replay:fibo:payment:001".to_string(),
        authority: "NONE".to_string(),
        capability: FIBO_PAYMENT_CAPABILITY.to_string(),
        principal: "principal:synthetic-treasury".to_string(),
        payload: json!({"amount":"1250.00","currency":"USD","beneficiary":"counterparty:42"}),
    }).unwrap();

    let subject = hook.prepared_effect.subject.as_str().unwrap().to_string();
    let transition = "synthetic-fibo-payment";
    let process = PowlProcess {
        id: "powl:fibo-payment".to_string(),
        goal_id: "goal:fibo-payment-postcondition".to_string(),
        activities: vec![PowlActivity {
            id: "activity:synthetic-fibo-payment".to_string(),
            transition_id: transition.to_string(),
            predecessors: vec![],
        }],
    };
    let envelope = TestEnvelope {
        system_id: subject.clone(),
        allowed_transition_ids: BTreeSet::from([transition.to_string()]),
        max_steps: 1,
        expires_at_epoch_ms: 10_000,
    };

    let blake3 = RealBlake3;
    let signer = TestSigner;
    let verifier = TestVerifier;
    let capability = manufacture_construct_capability(
        ConstructRequest {
            subject: subject.clone(),
            authority: "castle:bounded-fibo-do".to_string(),
            o_star: json!({
                "graphlaw_hook_contract": hook.hook_contract,
                "prepared_effect_digest": hook.effect_digest,
                "semantic_source_digest": hook.source_digest,
                "replay_identity": hook.replay_identity,
            }),
            config_graph: json!({
                "observation_membrane": "ash_r2rml",
                "consequence_thin_waist": "ash_a2a",
                "trust_plane": "affidavit",
                "runtime_composition": "xaas",
                "zeroUnreceiptedActuation": true,
            }),
            ontology: json!({"capability":FIBO_PAYMENT_CAPABILITY}),
            process: process.clone(),
            envelope: envelope.clone(),
        },
        &blake3,
        &signer,
    ).unwrap();

    let admission = admit_construct_for_do(
        &capability,
        &process,
        &envelope,
        &blake3,
        &verifier,
        &ConstructTrustPolicy {
            trusted_origin_key_ids: BTreeSet::from(["reference-root".to_string()]),
            allowed_authorities: BTreeSet::from(["castle:bounded-fibo-do".to_string()]),
        },
        || 1,
    ).unwrap();

    let policy = CommandAdapterPolicy {
        adapter_id: "synthetic-fibo".to_string(),
        provider: "synthetic".to_string(),
        workload_identity: "workload:fibo:reference".to_string(),
        commands: std::collections::BTreeMap::from([(
            transition.to_string(),
            CommandSpec {
                transition_id: transition.to_string(),
                program: "/bin/echo".to_string(),
                args: vec!["fibo-payment-observed".to_string()],
                allowed_exit_codes: BTreeSet::from([0]),
                max_output_bytes: 4096,
                timeout_ms: 2_000,
            },
        )]),
    };

    let state = WorldState { system_id: subject.clone(), facts: BTreeSet::new() };
    let (log, journal) = execute_command_process(
        &process,
        &state,
        &envelope,
        &admission,
        policy,
        &blake3,
        &signer,
        || 2,
    ).await.unwrap();

    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].standing, ReleaseStanding::Alive);
    assert!(journal[0].outcome_receipt.is_some());

    let feedback = project_beam4pm_feedback(&log, &subject, "replay:fibo:payment:001").unwrap();
    assert_eq!(feedback.ocel_event_count, 1);
    assert_eq!(feedback.subject, subject);
    assert!(!feedback.outcome_receipt_digest.is_empty());
}
