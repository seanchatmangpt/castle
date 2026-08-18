use std::collections::{BTreeMap, BTreeSet};

use castle::v26_8_18::*;

#[test]
fn dual_identity_emits_both_canonical_and_enterprise_hashes() {
    let identity = dual_artifact_identity(b"castle-v26.8.18");
    assert_eq!(identity.blake3_256.len(), 64);
    assert_eq!(identity.sha256.len(), 64);
    assert_ne!(identity.blake3_256, identity.sha256);
}

#[test]
fn crypto_profile_is_alive_for_dual_identity_plus_ed25519() {
    let profile = CryptoProfile {
        required_identity_hashes: BTreeSet::from(["blake3-256".to_string(), "sha256".to_string()]),
        accepted_signature_suites: BTreeSet::from([SignatureSuite::Ed25519]),
        require_post_quantum: false,
    };
    let q = qualify_crypto_profile(&profile);
    assert_eq!(q.standing, ReleaseStanding::Alive, "{:?}", q.reasons);
}

#[test]
fn pqc_requirement_is_explicitly_unsupported_not_fabricated() {
    let profile = CryptoProfile {
        required_identity_hashes: BTreeSet::from(["blake3-256".to_string(), "sha256".to_string()]),
        accepted_signature_suites: BTreeSet::from([SignatureSuite::Ed25519, SignatureSuite::MlDsa]),
        require_post_quantum: true,
    };
    let q = qualify_crypto_profile(&profile);
    assert_eq!(q.standing, ReleaseStanding::Unsupported);
    assert!(q.reasons.iter().any(|r| r == "UNSUPPORTED:PQC_SIGNATURE_RUNTIME_NOT_SHIPPED_IN_V26_8_18"));
}

#[test]
fn receipt_replication_is_monotonic_idempotent_and_equivocation_safe() {
    let mut state = ReplicaState { receiver_id: "global-replica".to_string(), checkpoints: BTreeMap::new() };
    let checkpoint = ReceiptCheckpoint {
        cell_id: "cell:aws-us".to_string(), sequence: 7, head_digest: "a".repeat(64),
        constitution_id: "constitution:v26.8.18".to_string(), observed_at_epoch_ms: 100,
    };
    assert_eq!(admit_receipt_checkpoint(&mut state, checkpoint.clone()).standing, ReleaseStanding::Alive);
    assert_eq!(admit_receipt_checkpoint(&mut state, checkpoint.clone()).standing, ReleaseStanding::Alive);

    let rollback = ReceiptCheckpoint { sequence: 6, ..checkpoint.clone() };
    assert_eq!(admit_receipt_checkpoint(&mut state, rollback).reason, "REFUSED:REPLICA_ROLLBACK");

    let equivocation = ReceiptCheckpoint { head_digest: "b".repeat(64), ..checkpoint };
    assert_eq!(admit_receipt_checkpoint(&mut state, equivocation).reason, "REFUSED:REPLICA_EQUIVOCATION");
}

#[test]
fn uncertain_provider_consequence_is_unknown_and_never_blindly_replayed() {
    let decision = reconcile_transition(&ReconciliationEvidence {
        transition_id: "aws:iam:quarantine-role".to_string(),
        prepare_receipt_digest: "a".repeat(64),
        outcome_receipt_digest: None,
        provider_observed: None,
        retry_is_proven_idempotent: true,
    });
    assert_eq!(decision.standing, ReleaseStanding::Unknown);
    assert_eq!(decision.reason, "UNKNOWN:CONSEQUENCE_UNCONFIRMED");
    assert!(!decision.replay_allowed);
}

fn all_chaos(failed_closed: bool) -> Vec<ChaosEvidence> {
    required_chaos_scenarios().into_iter().map(|scenario| ChaosEvidence {
        scenario, exercised: true, failed_closed, receipt_digest: "c".repeat(64), detail: "observed".to_string(),
    }).collect()
}

#[test]
fn all_required_failure_domains_must_be_receipted_and_fail_closed() {
    assert_eq!(qualify_chaos(&all_chaos(true)).standing, ReleaseStanding::Alive);
    let mut incomplete = all_chaos(true);
    incomplete.pop();
    assert_eq!(qualify_chaos(&incomplete).standing, ReleaseStanding::Unknown);
    let mut fail_open = all_chaos(true);
    fail_open[0].failed_closed = false;
    assert_eq!(qualify_chaos(&fail_open).standing, ReleaseStanding::Refused);
}

#[test]
fn durable_evidence_is_content_addressed_and_idempotent() {
    let root = std::env::temp_dir().join(format!("castle-evidence-{}", std::process::id()));
    let record = DurableEvidenceRecord {
        cell_id: "cell:test".to_string(), subject: "system:test".to_string(),
        construct_digest: "a".repeat(64), ocel_receipt_digest: "b".repeat(64),
        brce_prepare_receipt_digests: vec!["c".repeat(64)],
        brce_outcome_receipt_digests: vec!["d".repeat(64)], event_count: 1,
    };
    let first = persist_evidence(&root, &record).unwrap();
    let second = persist_evidence(&root, &record).unwrap();
    assert_eq!(first.standing, ReleaseStanding::Alive);
    assert_eq!(first.record_identity, second.record_identity);
    assert_eq!(first.path, second.path);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn runtime_construct_digest_is_deterministic_and_checkpointed() {
    let request = RuntimeExecutionRequest {
        cell_id: "cell:test".to_string(), evidence_dir: "target/test-evidence".to_string(),
        subject: "system:test".to_string(), authority: "bounded-do".to_string(),
        o_star: serde_json::json!({"subject":"system:test"}),
        config_graph: serde_json::json!({"zeroUnreceiptedActuation":true}),
        ontology: serde_json::json!({"version":"26.8.18"}),
        process: PortableProcess {
            id: "powl:test".to_string(), goal_id: "goal:test".to_string(),
            activities: vec![PortableActivity { id: "activity:echo".to_string(), transition_id: "echo".to_string(), predecessors: vec![] }],
        },
        envelope: PortableEnvelope {
            system_id: "system:test".to_string(), allowed_transition_ids: BTreeSet::from(["echo".to_string()]), max_steps: 1, expires_at_epoch_ms: 10_000,
        },
        allowed_authorities: BTreeSet::from(["bounded-do".to_string()]),
        adapter_policy: CommandAdapterPolicy {
            adapter_id: "local".to_string(), provider: "local".to_string(), workload_identity: "workload:test".to_string(),
            commands: BTreeMap::from([("echo".to_string(), CommandSpec {
                transition_id: "echo".to_string(), program: "/bin/echo".to_string(), args: vec!["castle".to_string()],
                allowed_exit_codes: BTreeSet::from([0]), max_output_bytes: 1024, timeout_ms: 2_000,
            })]),
        },
    };
    let first = manufacture_runtime_construct(&request, "runtime-key".to_string(), [9u8; 32]).unwrap();
    let second = manufacture_runtime_construct(&request, "runtime-key".to_string(), [9u8; 32]).unwrap();
    assert_eq!(first.construct_digest, second.construct_digest);
    assert_eq!(first.construct_receipt_digest, second.construct_receipt_digest);
}
