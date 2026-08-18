use std::collections::{BTreeMap, BTreeSet};

use castle::castle::{
    admit_construct_for_do, manufacture_construct_capability, Blake3Provider, ConstructRequest,
    ConstructTrustPolicy, PowlActivity, PowlProcess, ReceiptSigner, ReceiptVerifier, TestEnvelope,
    WorldState,
};
use castle::v26_8_18::*;
use serde_json::json;

struct RealBlake3;
impl Blake3Provider for RealBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

struct TestSigner;
impl ReceiptSigner for TestSigner {
    fn key_id(&self) -> &str { "v26-root" }
    fn sign_digest(&self, digest_hex: &str) -> String { digest_hex.to_string() }
}

struct TestVerifier;
impl ReceiptVerifier for TestVerifier {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool {
        key_id == "v26-root" && signature == digest_hex
    }
}

fn constitution() -> GlobalConstitution {
    GlobalConstitution {
        constitution_id: "constitution:fortune5:v26.8.18".to_string(),
        ontology_version: "castle-pack-v26.8.18".to_string(),
        invariant_set_digest: "a".repeat(64),
        trust_root_ids: BTreeSet::from(["v26-root".to_string()]),
        provider_semantics: BTreeMap::from([
            ("aws".to_string(), "2026-08-18".to_string()),
            ("azure".to_string(), "2026-08-18".to_string()),
            ("gcp".to_string(), "2026-08-18".to_string()),
            ("kubernetes".to_string(), "1.35".to_string()),
            ("github".to_string(), "2026-08-18".to_string()),
        ]),
        issued_at_epoch_ms: 1,
        expires_at_epoch_ms: 10_000,
    }
}

fn adapter(kind: &str, transition: &str) -> AdapterBinding {
    let semantics = if kind == "kubernetes" { "1.35" } else { "2026-08-18" };
    AdapterBinding {
        adapter_id: format!("{kind}-adapter"),
        kind: kind.to_string(),
        provider_semantics_key: kind.to_string(),
        provider_semantics_version: semantics.to_string(),
        allowed_transition_ids: BTreeSet::from([transition.to_string()]),
        workload_identity: format!("workload:{kind}:bounded"),
        ambient_credentials: false,
    }
}

fn cell(id: &str, provider: CloudProvider, region: &str, adapter_kind: &str) -> CastleCellManifest {
    CastleCellManifest {
        cell_id: id.to_string(),
        region: region.to_string(),
        provider,
        authority_domain: format!("authority:{id}"),
        residency: region.to_string(),
        subject_prefixes: vec![format!("subject:{id}:")],
        local_receipt_store: format!("receipt://{id}"),
        local_ocel_store: format!("ocel://{id}"),
        max_parallel_do: 32,
        adapters: vec![adapter(adapter_kind, &format!("transition:{adapter_kind}:bounded"))],
    }
}

fn full_manifest() -> DeploymentManifest {
    let mut aws = cell("aws-us", CloudProvider::Aws, "us-east-1", "aws");
    aws.adapters.push(adapter("kubernetes", "transition:kubernetes:bounded"));
    aws.adapters.push(adapter("github", "transition:github:bounded"));
    DeploymentManifest {
        kind: RELEASE_KIND.to_string(),
        release: RELEASE_VERSION.to_string(),
        constitution: constitution(),
        cells: vec![
            aws,
            cell("azure-eu", CloudProvider::Azure, "westeurope", "azure"),
            cell("gcp-apac", CloudProvider::Gcp, "asia-northeast1", "gcp"),
        ],
        protocol_surfaces: vec![
            ProtocolSurface { surface: InterfaceOrigin::Cli, endpoint: "castle://cli".to_string(), modes: BTreeSet::from([IntentMode::Select, IntentMode::Construct, IntentMode::Do]) },
            ProtocolSurface { surface: InterfaceOrigin::Api, endpoint: "https://castle.internal/v26.8.18".to_string(), modes: BTreeSet::from([IntentMode::Select, IntentMode::Construct, IntentMode::Do]) },
            ProtocolSurface { surface: InterfaceOrigin::Mcp, endpoint: "mcp://castle".to_string(), modes: BTreeSet::from([IntentMode::Select, IntentMode::Construct, IntentMode::Do]) },
            ProtocolSurface { surface: InterfaceOrigin::A2a, endpoint: "a2a://castle".to_string(), modes: BTreeSet::from([IntentMode::Select, IntentMode::Construct, IntentMode::Do]) },
        ],
        required_providers: BTreeSet::from([CloudProvider::Aws, CloudProvider::Azure, CloudProvider::Gcp]),
        required_adapter_kinds: BTreeSet::from(["aws".to_string(), "azure".to_string(), "gcp".to_string(), "kubernetes".to_string(), "github".to_string()]),
    }
}

#[test]
fn full_fortune5_topology_qualifies_alive() {
    let q = qualify_deployment(&full_manifest(), 5_000);
    assert_eq!(q.standing, ReleaseStanding::Alive, "{:?}", q.findings);
    assert_eq!(q.cells, 3);
    assert_eq!(q.manifest_digest.len(), 64);
}

#[test]
fn ambient_credentials_are_a_typed_refusal() {
    let mut manifest = full_manifest();
    manifest.cells[0].adapters[0].ambient_credentials = true;
    let q = qualify_deployment(&manifest, 5_000);
    assert_eq!(q.standing, ReleaseStanding::Refused);
    assert!(q.findings.iter().any(|f| f.contains("REFUSED:AMBIENT_CREDENTIALS")));
}

#[test]
fn provider_semantic_drift_invalidates_the_cell() {
    let mut manifest = full_manifest();
    manifest.cells[1].adapters[0].provider_semantics_version = "stale".to_string();
    let q = qualify_deployment(&manifest, 5_000);
    assert_eq!(q.standing, ReleaseStanding::Refused);
    assert!(q.findings.iter().any(|f| f.contains("REFUSED:PROVIDER_SEMANTICS_DRIFT")));
}

#[test]
fn every_required_protocol_surface_is_structural_not_documentary() {
    let mut manifest = full_manifest();
    manifest.protocol_surfaces.retain(|s| s.surface != InterfaceOrigin::Mcp);
    let q = qualify_deployment(&manifest, 5_000);
    assert!(q.findings.iter().any(|f| f == "REFUSED:MISSING_PROTOCOL_SURFACE:mcp"));
}

#[test]
fn select_and_construct_have_no_do_requirement() {
    for mode in [IntentMode::Select, IntentMode::Construct] {
        let admitted = admit_interface_intent(&InterfaceIntent {
            request_id: format!("request:{mode:?}"),
            origin: InterfaceOrigin::Mcp,
            mode,
            subject: "subject:payments".to_string(),
            operation: "analyze".to_string(),
            payload: json!({"goal": "protect"}),
            construct_admission_digest: None,
            prepare_receipt_digest: None,
        });
        assert_eq!(admitted.standing, ReleaseStanding::Alive);
    }
}

#[test]
fn all_transport_origins_refuse_unreceipted_do() {
    for origin in [InterfaceOrigin::Cli, InterfaceOrigin::Api, InterfaceOrigin::Mcp, InterfaceOrigin::A2a, InterfaceOrigin::Human, InterfaceOrigin::Planner, InterfaceOrigin::Replay] {
        let admitted = admit_interface_intent(&InterfaceIntent {
            request_id: format!("request:{origin:?}"),
            origin,
            mode: IntentMode::Do,
            subject: "subject:payments".to_string(),
            operation: "quarantine".to_string(),
            payload: json!({}),
            construct_admission_digest: None,
            prepare_receipt_digest: None,
        });
        assert_eq!(admitted.standing, ReleaseStanding::Refused);
        assert_eq!(admitted.reason, "REFUSED:DO_WITHOUT_CONSTRUCT_ADMISSION");
    }
}

#[test]
fn observation_admission_manufactures_o_star_only_from_fresh_allowed_sources() {
    let allowed = BTreeSet::from(["cloudtrail".to_string()]);
    let admitted = admit_observation(
        &ObservationEnvelope {
            observation_id: "obs:1".to_string(), source: "cloudtrail".to_string(), subject: "subject:payments".to_string(),
            observed_at_epoch_ms: 90, epistemic_class: "OBSERVED".to_string(), payload: json!({"event": "AssumeRole"}),
        },
        &allowed, 100, 20,
    );
    assert_eq!(admitted.standing, ReleaseStanding::Alive);
    assert_eq!(admitted.payload_digest.len(), 64);

    let stale = admit_observation(
        &ObservationEnvelope {
            observation_id: "obs:2".to_string(), source: "cloudtrail".to_string(), subject: "subject:payments".to_string(),
            observed_at_epoch_ms: 1, epistemic_class: "OBSERVED".to_string(), payload: json!({}),
        },
        &allowed, 100, 20,
    );
    assert_eq!(stale.reason, "REFUSED:STALE_OBSERVATION");
}

#[test]
fn airgap_construct_bundle_has_zero_network_and_secret_dependencies() {
    let bundle = manufacture_airgap_bundle(
        "bundle:1".to_string(), &constitution(), json!({"subject": "payments"}), json!({"construct": true}), json!(["goal:no-exfiltration"]),
    ).unwrap();
    assert!(bundle.network_dependencies.is_empty());
    assert!(bundle.secret_dependencies.is_empty());
    let result = AirgapResult {
        bundle_id: bundle.bundle_id.clone(), input_bundle_digest: bundle.bundle_digest.clone(), result_digest: "b".repeat(64),
        network_used: false, secret_material_used: false,
    };
    assert_eq!(admit_airgap_result(&bundle, &result).standing, ReleaseStanding::Alive);
}

#[test]
fn airgap_result_fails_closed_if_network_was_used() {
    let bundle = manufacture_airgap_bundle("bundle:network".to_string(), &constitution(), json!({}), json!({}), json!([])).unwrap();
    let result = AirgapResult {
        bundle_id: bundle.bundle_id.clone(), input_bundle_digest: bundle.bundle_digest.clone(), result_digest: "b".repeat(64),
        network_used: true, secret_material_used: false,
    };
    assert_eq!(admit_airgap_result(&bundle, &result).reason, "REFUSED:AIRGAP_NETWORK_USED");
}

#[test]
fn provider_catalog_has_no_ambient_credentials() {
    let catalog = fortune5_adapter_catalog();
    assert_eq!(catalog.len(), 5);
    assert!(catalog.iter().all(|a| !a.ambient_credentials_allowed));
}

#[test]
fn mcp_and_a2a_surfaces_preserve_construct_only_default_authority() {
    let tools = mcp_tool_catalog();
    assert_eq!(tools.len(), 3);
    assert_eq!(tools.iter().filter(|t| t.consequential).count(), 1);
    let card = a2a_agent_card();
    assert_eq!(card.version, RELEASE_VERSION);
    assert_eq!(card.default_authority, "CONSTRUCT_ONLY");
}

#[test]
fn global_standing_is_a_lattice_not_a_risk_score() {
    let alive = CellStandingRow {
        cell_id: "cell:a".to_string(), observation: ReleaseStanding::Alive, construct: ReleaseStanding::Alive,
        do_standing: ReleaseStanding::Alive, replay: ReleaseStanding::Alive,
    };
    assert_eq!(aggregate_global_standing(vec![alive.clone()]).standing, ReleaseStanding::Alive);
    let mut partial = alive;
    partial.replay = ReleaseStanding::Unknown;
    assert_eq!(aggregate_global_standing(vec![partial]).standing, ReleaseStanding::PartialAlive);
}

#[test]
fn command_policy_rejects_shell_like_programs_and_accepts_exact_binaries() {
    let bad = CommandAdapterPolicy {
        adapter_id: "bad".to_string(), provider: "local".to_string(), workload_identity: "workload:test".to_string(),
        commands: BTreeMap::from([("t".to_string(), CommandSpec {
            transition_id: "t".to_string(), program: "sh -c".to_string(), args: vec!["echo no".to_string()],
            allowed_exit_codes: BTreeSet::from([0]), max_output_bytes: 1024, timeout_ms: 2_000,
        })]),
    };
    assert_eq!(validate_command_adapter_policy(&bad), ReleaseStanding::Refused);

    let good = CommandAdapterPolicy {
        adapter_id: "good".to_string(), provider: "local".to_string(), workload_identity: "workload:test".to_string(),
        commands: BTreeMap::from([("t".to_string(), CommandSpec {
            transition_id: "t".to_string(), program: "/bin/echo".to_string(), args: vec!["castle".to_string()],
            allowed_exit_codes: BTreeSet::from([0]), max_output_bytes: 1024, timeout_ms: 2_000,
        })]),
    };
    assert_eq!(validate_command_adapter_policy(&good), ReleaseStanding::Alive);
}

#[tokio::test]
async fn real_process_do_runs_only_through_construct_admission_and_brce() {
    let blake3 = RealBlake3;
    let signer = TestSigner;
    let verifier = TestVerifier;
    let process = PowlProcess {
        id: "powl:provider-read".to_string(), goal_id: "goal:validate-provider".to_string(),
        activities: vec![PowlActivity { id: "activity:provider-read".to_string(), transition_id: "provider-read".to_string(), predecessors: vec![] }],
    };
    let envelope = TestEnvelope {
        system_id: "system:fortune5:test".to_string(), allowed_transition_ids: BTreeSet::from(["provider-read".to_string()]),
        max_steps: 1, expires_at_epoch_ms: 10_000,
    };
    let capability = manufacture_construct_capability(
        ConstructRequest {
            subject: envelope.system_id.clone(), authority: "fortune5-bounded-do".to_string(), o_star: json!({"subject": envelope.system_id}),
            config_graph: json!({"zeroUnreceiptedActuation": true}), ontology: json!({"version": RELEASE_VERSION}),
            process: process.clone(), envelope: envelope.clone(),
        },
        &blake3, &signer,
    ).unwrap();
    let admission = admit_construct_for_do(
        &capability, &process, &envelope, &blake3, &verifier,
        &ConstructTrustPolicy {
            trusted_origin_key_ids: BTreeSet::from(["v26-root".to_string()]),
            allowed_authorities: BTreeSet::from(["fortune5-bounded-do".to_string()]),
        },
        || 1,
    ).unwrap();

    let policy = CommandAdapterPolicy {
        adapter_id: "local-proof".to_string(), provider: "local".to_string(), workload_identity: "workload:test".to_string(),
        commands: BTreeMap::from([("provider-read".to_string(), CommandSpec {
            transition_id: "provider-read".to_string(), program: "/bin/echo".to_string(), args: vec!["castle-v26.8.18".to_string()],
            allowed_exit_codes: BTreeSet::from([0]), max_output_bytes: 4096, timeout_ms: 2_000,
        })]),
    };
    let state = WorldState { system_id: envelope.system_id.clone(), facts: BTreeSet::new() };
    let (log, journal) = execute_command_process(&process, &state, &envelope, &admission, policy, &blake3, &signer, || 2).await.unwrap();

    assert_eq!(log.log.events.len(), 1);
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].standing, ReleaseStanding::Alive);
    assert!(journal[0].outcome_receipt.is_some());
    assert!(log.log.events[0].attributes.contains_key("brce_prepare_receipt_digest"));
    assert!(log.log.events[0].attributes.contains_key("brce_outcome_receipt_digest"));
}
