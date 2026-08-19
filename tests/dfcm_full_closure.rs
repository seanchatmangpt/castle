use std::collections::{BTreeMap, BTreeSet};

use castle::v26_8_18::*;
use serde_json::json;

fn manifest() -> DeploymentManifest {
    DeploymentManifest {
        kind: RELEASE_KIND.to_string(),
        release: RELEASE_VERSION.to_string(),
        constitution: GlobalConstitution {
            constitution_id: "constitution:dfcm:test".to_string(),
            ontology_version: "castle-pack-v26.8.18+dfcm.1".to_string(),
            invariant_set_digest: "a".repeat(64),
            trust_root_ids: BTreeSet::from(["root:dfcm".to_string()]),
            provider_semantics: BTreeMap::from([("aws".to_string(), "2026-08-18".to_string())]),
            issued_at_epoch_ms: 1,
            expires_at_epoch_ms: 20_000,
        },
        cells: vec![CastleCellManifest {
            cell_id: "aws-test".to_string(),
            region: "us-east-1".to_string(),
            provider: CloudProvider::Aws,
            authority_domain: "test-authority".to_string(),
            residency: "US".to_string(),
            subject_prefixes: vec!["aws:test:".to_string()],
            local_receipt_store: "receipt://aws-test".to_string(),
            local_ocel_store: "ocel://aws-test".to_string(),
            max_parallel_do: 1,
            adapters: vec![AdapterBinding {
                adapter_id: "aws-test-adapter".to_string(),
                kind: "aws".to_string(),
                provider_semantics_key: "aws".to_string(),
                provider_semantics_version: "2026-08-18".to_string(),
                allowed_transition_ids: BTreeSet::from(["aws:iam:quarantine-role".to_string()]),
                workload_identity: "aws:iam-role/castle-test".to_string(),
                ambient_credentials: false,
            }],
        }],
        protocol_surfaces: [
            InterfaceOrigin::Cli,
            InterfaceOrigin::Api,
            InterfaceOrigin::Mcp,
            InterfaceOrigin::A2a,
        ]
        .into_iter()
        .map(|surface| ProtocolSurface {
            surface,
            endpoint: format!("{}://castle", surface.as_str()),
            modes: BTreeSet::from([IntentMode::Select, IntentMode::Construct, IntentMode::Do]),
        })
        .collect(),
        required_providers: BTreeSet::from([CloudProvider::Aws]),
        required_adapter_kinds: BTreeSet::from(["aws".to_string()]),
    }
}

fn observation(purpose: ProbePurpose, kind: ReadOnlyProbeKind) -> ProviderProbeObservation {
    ProviderProbeObservation {
        cell_id: "aws-test".to_string(),
        adapter_id: "aws-test-adapter".to_string(),
        provider_kind: "aws".to_string(),
        workload_identity: "aws:iam-role/castle-test".to_string(),
        provider_semantics_version: "2026-08-18".to_string(),
        kind,
        purpose,
        observed_at_epoch_ms: 9_900,
        exit_code: 0,
        stdout_blake3: "b".repeat(64),
        stderr_blake3: "c".repeat(64),
        identity_binding_verified: purpose == ProbePurpose::AuthorityContext,
        observation_digest: "d".repeat(64),
        standing: ReleaseStanding::Alive,
        reason: "ALIVE:READ_ONLY_PROBE_OBSERVED".to_string(),
    }
}

fn live_evidence() -> Vec<ProviderProbeObservation> {
    vec![
        observation(ProbePurpose::Version, ReadOnlyProbeKind::AwsCliVersion),
        observation(ProbePurpose::AuthorityContext, ReadOnlyProbeKind::AwsAuthorityContext),
    ]
}

#[test]
fn static_manifest_never_substitutes_for_live_subject_evidence() {
    let q = qualify_live_deployment(&manifest(), &[], 10_000, 1_000);
    assert_eq!(q.standing, ReleaseStanding::Unknown);
    assert!(q.findings.iter().any(|finding| finding.starts_with("UNKNOWN:MISSING_LIVE_ADAPTER_EVIDENCE")));
}

#[test]
fn exact_fresh_version_and_authority_context_can_admit_live_cell() {
    let q = qualify_live_deployment(&manifest(), &live_evidence(), 10_000, 1_000);
    assert_eq!(q.standing, ReleaseStanding::Alive, "{:?}", q.findings);
    assert_eq!(q.adapters_alive, 1);
}

#[test]
fn live_semantic_or_identity_drift_is_refused() {
    let mut evidence = live_evidence();
    evidence[1].workload_identity = "aws:iam-role/someone-else".to_string();
    let q = qualify_live_deployment(&manifest(), &evidence, 10_000, 1_000);
    assert_eq!(q.standing, ReleaseStanding::Refused);
    assert!(q.findings.iter().any(|finding| finding.starts_with("REFUSED:LIVE_WORKLOAD_IDENTITY_MISMATCH")));
}

#[test]
fn read_only_probe_arguments_are_typed_not_caller_supplied() {
    let spec = ReadOnlyProbeSpec {
        cell_id: "local".to_string(),
        adapter_id: "local".to_string(),
        workload_identity: "local:self-test".to_string(),
        provider_semantics_version: RELEASE_VERSION.to_string(),
        kind: ReadOnlyProbeKind::LocalSelfTest,
        program_override: Some("/bin/echo".to_string()),
        expected_identity_marker: None,
        max_output_bytes: 4096,
        timeout_ms: 2_000,
    };
    let observation = run_read_only_probe(&spec, 1).unwrap();
    assert_eq!(observation.standing, ReleaseStanding::Alive, "{}", observation.reason);
    assert_eq!(observation.purpose, ProbePurpose::SelfTest);
    assert_eq!(observation.exit_code, 0);
}

#[test]
fn provider_policy_must_match_manifest_binding_and_canonical_binary_family() {
    let binding = manifest().cells[0].adapters[0].clone();
    let good = CommandAdapterPolicy {
        adapter_id: binding.adapter_id.clone(),
        provider: "aws".to_string(),
        workload_identity: binding.workload_identity.clone(),
        commands: BTreeMap::from([(
            "aws:iam:quarantine-role".to_string(),
            CommandSpec {
                transition_id: "aws:iam:quarantine-role".to_string(),
                program: "/usr/bin/aws".to_string(),
                args: vec!["iam".to_string(), "update-assume-role-policy".to_string()],
                allowed_exit_codes: BTreeSet::from([0]),
                max_output_bytes: 4096,
                timeout_ms: 2_000,
            },
        )]),
    };
    assert!(bind_command_policy(&binding, &good).is_ok());

    let mut bad = good;
    bad.commands.get_mut("aws:iam:quarantine-role").unwrap().program = "/bin/sh".to_string();
    assert_eq!(bind_command_policy(&binding, &bad).unwrap_err(), "REFUSED:PROVIDER_PROGRAM_MISMATCH:aws:iam:quarantine-role");
}

#[test]
fn transport_do_with_receipt_names_is_partial_not_alive() {
    let dispatch = dispatch_interface_intent(&InterfaceIntent {
        request_id: "request:do".to_string(),
        origin: InterfaceOrigin::Api,
        mode: IntentMode::Do,
        subject: "aws:test:subject".to_string(),
        operation: "quarantine".to_string(),
        payload: json!({}),
        construct_admission_digest: Some("a".repeat(64)),
        prepare_receipt_digest: Some("b".repeat(64)),
    });
    assert_eq!(dispatch.standing, ReleaseStanding::PartialAlive);
    assert_eq!(dispatch.reason, "PARTIAL_ALIVE:DO_INTENT_REQUIRES_IN_PROCESS_ADMISSION");
    assert_eq!(dispatch.result["authority"], "NONE_FROM_TRANSPORT");
}

#[test]
fn protocol_fence_executes_all_origins_and_detects_no_do_leak() {
    let q = qualify_protocol_fence();
    assert_eq!(q.standing, ReleaseStanding::Alive, "{:?}", q.findings);
    assert_eq!(q.dispatches.len(), 7);
    assert!(q.dispatches.iter().all(|row| row.do_standing == ReleaseStanding::PartialAlive));
}

#[test]
fn complete_dfcm_closure_can_be_alive_for_exact_observed_subject() {
    let profile = CryptoProfile {
        required_identity_hashes: BTreeSet::from(["blake3-256".to_string(), "sha256".to_string()]),
        accepted_signature_suites: BTreeSet::from([SignatureSuite::MlDsa, SignatureSuite::SlhDsa]),
        require_post_quantum: true,
    };
    let q = qualify_dfcm_closure(&manifest(), &live_evidence(), &profile, 10_000, 1_000);
    assert_eq!(q.standing, ReleaseStanding::Alive, "{:?}", q.findings);
}
