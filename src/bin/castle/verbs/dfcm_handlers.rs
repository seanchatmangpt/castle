use std::collections::BTreeSet;
use std::fs;

use castle::v26_8_18::{
    bind_command_policy, dispatch_interface_intent, dual_artifact_identity,
    manufacture_live_probe_plan, persist_receipt_checkpoint, qualify_crypto_profile,
    qualify_dfcm_closure, qualify_live_deployment, qualify_pqc_runtime, qualify_protocol_fence,
    run_read_only_probe, AdapterBinding, CommandAdapterPolicy, CryptoProfile, DeploymentManifest,
    InterfaceIntent, ProviderProbeObservation, ReadOnlyProbeKind, ReadOnlyProbeSpec,
    ReceiptCheckpoint, ReleaseStanding, SignatureSuite, RELEASE_VERSION,
};
use clap_noun_verb::NounVerbError;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError { message: message.into() }
}

fn read_json<T: DeserializeOwned>(path: &str, kind: &str) -> Result<T> {
    let contents = fs::read_to_string(path)
        .map_err(|error| exec_err(format!("failed to read {kind} {path}: {error}")))?;
    serde_json::from_str(&contents)
        .map_err(|error| exec_err(format!("invalid {kind} JSON in {path}: {error}")))
}

fn enterprise_pqc_profile() -> CryptoProfile {
    CryptoProfile {
        required_identity_hashes: BTreeSet::from([
            "blake3-256".to_string(),
            "sha256".to_string(),
        ]),
        accepted_signature_suites: BTreeSet::from([
            SignatureSuite::Ed25519,
            SignatureSuite::MlDsa,
            SignatureSuite::SlhDsa,
        ]),
        require_post_quantum: true,
    }
}

pub fn crypto_capabilities_handler() -> Result<Value> {
    let profile = enterprise_pqc_profile();
    let qualification = qualify_crypto_profile(&profile);
    let pqc_runtime = qualify_pqc_runtime();
    Ok(json!({
        "release": RELEASE_VERSION,
        "identity": dual_artifact_identity(format!("CASTLE:{RELEASE_VERSION}").as_bytes()),
        "qualification": qualification,
        "pqcRuntime": pqc_runtime,
    }))
}

pub fn live_check_plan_handler(manifest_path: String) -> Result<Value> {
    let manifest: DeploymentManifest = read_json(&manifest_path, "deployment manifest")?;
    serde_json::to_value(manufacture_live_probe_plan(&manifest))
        .map_err(|error| exec_err(format!("failed to serialize live-check plan: {error}")))
}

pub fn live_check_run_handler(spec_path: String, observed_at_epoch_ms: i64) -> Result<Value> {
    let spec: ReadOnlyProbeSpec = read_json(&spec_path, "read-only probe spec")?;
    let observation = run_read_only_probe(&spec, observed_at_epoch_ms).map_err(exec_err)?;
    serde_json::to_value(observation)
        .map_err(|error| exec_err(format!("failed to serialize probe observation: {error}")))
}

pub fn live_check_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) -> Result<Value> {
    let manifest: DeploymentManifest = read_json(&manifest_path, "deployment manifest")?;
    let observations: Vec<ProviderProbeObservation> = read_json(&evidence_path, "live evidence")?;
    serde_json::to_value(qualify_live_deployment(
        &manifest,
        &observations,
        now_epoch_ms,
        max_evidence_age_ms,
    ))
    .map_err(|error| exec_err(format!("failed to serialize live qualification: {error}")))
}

pub fn deployment_bind_policy_handler(
    binding_path: String,
    policy_path: String,
) -> Result<Value> {
    let binding: AdapterBinding = read_json(&binding_path, "adapter binding")?;
    let policy: CommandAdapterPolicy = read_json(&policy_path, "command adapter policy")?;
    let admitted = bind_command_policy(&binding, &policy).map_err(exec_err)?;
    Ok(json!({
        "standing": ReleaseStanding::Alive,
        "reason": "ALIVE:BOUND_PROVIDER_POLICY_ADMITTED",
        "adapterPolicy": admitted,
    }))
}

pub fn protocol_dispatch_handler(intent_path: String) -> Result<Value> {
    let intent: InterfaceIntent = read_json(&intent_path, "interface intent")?;
    serde_json::to_value(dispatch_interface_intent(&intent))
        .map_err(|error| exec_err(format!("failed to serialize protocol dispatch: {error}")))
}

pub fn replication_admit_handler(
    state_path: String,
    checkpoint_path: String,
    receiver_id: String,
) -> Result<Value> {
    let checkpoint: ReceiptCheckpoint = read_json(&checkpoint_path, "receipt checkpoint")?;
    let commit = persist_receipt_checkpoint(&state_path, &receiver_id, checkpoint).map_err(exec_err)?;
    serde_json::to_value(commit)
        .map_err(|error| exec_err(format!("failed to serialize replica commit: {error}")))
}

pub fn dfcm_verify_handler() -> Result<Value> {
    let pqc = qualify_pqc_runtime();
    let protocol = qualify_protocol_fence();
    let probe = run_read_only_probe(
        &ReadOnlyProbeSpec {
            cell_id: "cell:dfcm-self-test".to_string(),
            adapter_id: "adapter:local-self-test".to_string(),
            workload_identity: "local:self-test".to_string(),
            provider_semantics_version: RELEASE_VERSION.to_string(),
            kind: ReadOnlyProbeKind::LocalSelfTest,
            program_override: None,
            expected_identity_marker: None,
            max_output_bytes: 4096,
            timeout_ms: 2_000,
        },
        1,
    )
    .map_err(exec_err)?;
    let standing = if pqc.standing == ReleaseStanding::Alive
        && protocol.standing == ReleaseStanding::Alive
        && probe.standing == ReleaseStanding::Alive
    {
        ReleaseStanding::Alive
    } else if [pqc.standing, protocol.standing, probe.standing]
        .contains(&ReleaseStanding::Refused)
    {
        ReleaseStanding::Refused
    } else {
        ReleaseStanding::BuildBroken
    };
    Ok(json!({
        "release": RELEASE_VERSION,
        "standing": standing,
        "pqcRuntime": pqc,
        "protocolFence": protocol,
        "readOnlyProbe": probe,
    }))
}

pub fn dfcm_qualify_handler(
    manifest_path: String,
    evidence_path: String,
    now_epoch_ms: i64,
    max_evidence_age_ms: i64,
) -> Result<Value> {
    let manifest: DeploymentManifest = read_json(&manifest_path, "deployment manifest")?;
    let observations: Vec<ProviderProbeObservation> = read_json(&evidence_path, "live evidence")?;
    let qualification = qualify_dfcm_closure(
        &manifest,
        &observations,
        &enterprise_pqc_profile(),
        now_epoch_ms,
        max_evidence_age_ms,
    );
    serde_json::to_value(qualification)
        .map_err(|error| exec_err(format!("failed to serialize DfCM closure: {error}")))
}
