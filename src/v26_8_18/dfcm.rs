use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::{
    dispatch_interface_intent, qualify_crypto_profile, qualify_deployment, qualify_pqc_runtime,
    AdapterBinding, CommandAdapterPolicy, CryptoProfile, DeploymentManifest, InterfaceIntent,
    InterfaceOrigin, IntentMode, PqcRuntimeQualification, ProtocolDispatch, ReleaseStanding,
};

fn digest_ok(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

fn bounded_program(program: &str) -> bool {
    !program.is_empty() && !program.contains(char::is_whitespace)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbePurpose {
    Version,
    AuthorityContext,
    SelfTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadOnlyProbeKind {
    AwsCliVersion,
    AwsAuthorityContext,
    AzureCliVersion,
    AzureAuthorityContext,
    GcpCliVersion,
    GcpAuthorityContext,
    KubernetesCliVersion,
    KubernetesAuthorityContext,
    GitHubCliVersion,
    GitHubAuthorityContext,
    LocalSelfTest,
}

impl ReadOnlyProbeKind {
    #[must_use]
    pub fn purpose(self) -> ProbePurpose {
        match self {
            Self::AwsCliVersion
            | Self::AzureCliVersion
            | Self::GcpCliVersion
            | Self::KubernetesCliVersion
            | Self::GitHubCliVersion => ProbePurpose::Version,
            Self::AwsAuthorityContext
            | Self::AzureAuthorityContext
            | Self::GcpAuthorityContext
            | Self::KubernetesAuthorityContext
            | Self::GitHubAuthorityContext => ProbePurpose::AuthorityContext,
            Self::LocalSelfTest => ProbePurpose::SelfTest,
        }
    }

    #[must_use]
    pub fn provider_kind(self) -> &'static str {
        match self {
            Self::AwsCliVersion | Self::AwsAuthorityContext => "aws",
            Self::AzureCliVersion | Self::AzureAuthorityContext => "azure",
            Self::GcpCliVersion | Self::GcpAuthorityContext => "gcp",
            Self::KubernetesCliVersion | Self::KubernetesAuthorityContext => "kubernetes",
            Self::GitHubCliVersion | Self::GitHubAuthorityContext => "github",
            Self::LocalSelfTest => "local",
        }
    }

    #[must_use]
    pub fn default_program(self) -> &'static str {
        match self {
            Self::AwsCliVersion | Self::AwsAuthorityContext => "aws",
            Self::AzureCliVersion | Self::AzureAuthorityContext => "az",
            Self::GcpCliVersion | Self::GcpAuthorityContext => "gcloud",
            Self::KubernetesCliVersion | Self::KubernetesAuthorityContext => "kubectl",
            Self::GitHubCliVersion | Self::GitHubAuthorityContext => "gh",
            Self::LocalSelfTest => "/bin/echo",
        }
    }

    #[must_use]
    pub fn args(self) -> Vec<&'static str> {
        match self {
            Self::AwsCliVersion => vec!["--version"],
            Self::AwsAuthorityContext => vec!["sts", "get-caller-identity", "--output", "json"],
            Self::AzureCliVersion => vec!["version", "--output", "json"],
            Self::AzureAuthorityContext => vec!["account", "show", "--output", "json"],
            Self::GcpCliVersion => vec!["version", "--format=json"],
            Self::GcpAuthorityContext => {
                vec!["auth", "list", "--filter=status:ACTIVE", "--format=json"]
            }
            Self::KubernetesCliVersion => vec!["version", "--client", "-o", "json"],
            Self::KubernetesAuthorityContext => vec!["auth", "whoami", "-o", "json"],
            Self::GitHubCliVersion => vec!["--version"],
            Self::GitHubAuthorityContext => vec!["api", "user"],
            Self::LocalSelfTest => vec!["CASTLE_DFCM_READ_ONLY_SELF_TEST"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadOnlyProbeSpec {
    pub cell_id: String,
    pub adapter_id: String,
    pub workload_identity: String,
    pub provider_semantics_version: String,
    pub kind: ReadOnlyProbeKind,
    /// Optional absolute path to the exact executable. Basename must still match
    /// the typed probe program, except for the explicit LocalSelfTest probe.
    pub program_override: Option<String>,
    /// Non-secret marker expected in bounded authority-context stdout.
    /// Its presence demonstrates only the configured identity/context binding;
    /// the resulting observation still has no DO authority.
    pub expected_identity_marker: Option<String>,
    pub max_output_bytes: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderProbeObservation {
    pub cell_id: String,
    pub adapter_id: String,
    pub provider_kind: String,
    pub workload_identity: String,
    pub provider_semantics_version: String,
    pub kind: ReadOnlyProbeKind,
    pub purpose: ProbePurpose,
    pub observed_at_epoch_ms: i64,
    pub exit_code: i32,
    pub stdout_blake3: String,
    pub stderr_blake3: String,
    pub identity_binding_verified: bool,
    pub observation_digest: String,
    pub standing: ReleaseStanding,
    pub reason: String,
}

fn resolve_probe_program(spec: &ReadOnlyProbeSpec) -> Result<String, String> {
    let expected = spec.kind.default_program();
    let program = spec.program_override.as_deref().unwrap_or(expected);
    if !bounded_program(program) {
        return Err("REFUSED:INVALID_PROBE_PROGRAM".to_string());
    }
    if let Some(override_program) = &spec.program_override {
        if spec.kind != ReadOnlyProbeKind::LocalSelfTest {
            if !Path::new(override_program).is_absolute() {
                return Err("REFUSED:PROBE_OVERRIDE_MUST_BE_ABSOLUTE".to_string());
            }
            let basename = Path::new(override_program)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| "REFUSED:INVALID_PROBE_OVERRIDE".to_string())?;
            if basename != expected {
                return Err("REFUSED:PROBE_PROGRAM_KIND_MISMATCH".to_string());
            }
        }
    }
    Ok(program.to_string())
}

fn validate_probe_spec(spec: &ReadOnlyProbeSpec) -> Result<String, String> {
    if spec.cell_id.is_empty()
        || spec.adapter_id.is_empty()
        || spec.workload_identity.is_empty()
        || spec.provider_semantics_version.is_empty()
    {
        return Err("REFUSED:INCOMPLETE_PROBE_SPEC".to_string());
    }
    if spec.max_output_bytes == 0 || spec.timeout_ms == 0 {
        return Err("REFUSED:UNBOUNDED_PROBE".to_string());
    }
    if spec.kind.purpose() == ProbePurpose::AuthorityContext
        && spec
            .expected_identity_marker
            .as_deref()
            .is_some_and(str::is_empty)
    {
        return Err("REFUSED:EMPTY_IDENTITY_MARKER".to_string());
    }
    resolve_probe_program(spec)
}

/// Execute one fixed, typed read-only provider observation. No shell is used,
/// inherited environment is cleared, output/time are bounded, and arbitrary
/// argument vectors are impossible because arguments are derived from the enum.
/// The returned object is observation evidence only and carries no DO authority.
pub fn run_read_only_probe(
    spec: &ReadOnlyProbeSpec,
    observed_at_epoch_ms: i64,
) -> Result<ProviderProbeObservation, String> {
    let program = validate_probe_spec(spec)?;
    let child = Command::new(&program)
        .args(spec.kind.args())
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let Ok(mut child) = child else {
        return Ok(probe_failure(
            spec,
            observed_at_epoch_ms,
            -1,
            "BLOCKED:PROBE_PROGRAM_UNAVAILABLE",
        ));
    };

    let started = Instant::now();
    let timed_out = loop {
        match child.try_wait() {
            Ok(Some(_)) => break false,
            Ok(None) if started.elapsed() < Duration::from_millis(spec.timeout_ms) => {
                thread::sleep(Duration::from_millis(5));
            }
            Ok(None) => {
                let _ = child.kill();
                break true;
            }
            Err(_) => {
                let _ = child.kill();
                break true;
            }
        }
    };
    let output = child
        .wait_with_output()
        .map_err(|_| "BLOCKED:PROBE_WAIT_FAILED".to_string())?;
    if timed_out {
        return Ok(probe_failure(
            spec,
            observed_at_epoch_ms,
            -1,
            "BLOCKED:PROBE_TIMEOUT",
        ));
    }

    let code = output.status.code().unwrap_or(-1);
    let stdout = &output.stdout[..output.stdout.len().min(spec.max_output_bytes)];
    let stderr = &output.stderr[..output.stderr.len().min(spec.max_output_bytes)];
    let stdout_blake3 = blake3::hash(stdout).to_hex().to_string();
    let stderr_blake3 = blake3::hash(stderr).to_hex().to_string();
    let stdout_text = String::from_utf8_lossy(stdout);
    let identity_binding_verified = spec.kind.purpose() != ProbePurpose::AuthorityContext
        || spec
            .expected_identity_marker
            .as_deref()
            .is_some_and(|marker| stdout_text.contains(marker));
    let reason = if code != 0 {
        "BLOCKED:PROBE_NONZERO_EXIT"
    } else if spec.kind.purpose() == ProbePurpose::AuthorityContext
        && spec.expected_identity_marker.is_none()
    {
        "UNKNOWN:AUTHORITY_CONTEXT_NOT_BOUND"
    } else if spec.kind.purpose() == ProbePurpose::AuthorityContext && !identity_binding_verified {
        "REFUSED:AUTHORITY_CONTEXT_MISMATCH"
    } else {
        "ALIVE:READ_ONLY_PROBE_OBSERVED"
    };
    let standing = standing_from_reason(reason);
    let observation_digest = probe_digest(
        spec,
        observed_at_epoch_ms,
        code,
        &stdout_blake3,
        &stderr_blake3,
        identity_binding_verified,
    );
    Ok(ProviderProbeObservation {
        cell_id: spec.cell_id.clone(),
        adapter_id: spec.adapter_id.clone(),
        provider_kind: spec.kind.provider_kind().to_string(),
        workload_identity: spec.workload_identity.clone(),
        provider_semantics_version: spec.provider_semantics_version.clone(),
        kind: spec.kind,
        purpose: spec.kind.purpose(),
        observed_at_epoch_ms,
        exit_code: code,
        stdout_blake3,
        stderr_blake3,
        identity_binding_verified,
        observation_digest,
        standing,
        reason: reason.to_string(),
    })
}

fn probe_failure(
    spec: &ReadOnlyProbeSpec,
    observed_at_epoch_ms: i64,
    exit_code: i32,
    reason: &str,
) -> ProviderProbeObservation {
    let zero = "0".repeat(64);
    ProviderProbeObservation {
        cell_id: spec.cell_id.clone(),
        adapter_id: spec.adapter_id.clone(),
        provider_kind: spec.kind.provider_kind().to_string(),
        workload_identity: spec.workload_identity.clone(),
        provider_semantics_version: spec.provider_semantics_version.clone(),
        kind: spec.kind,
        purpose: spec.kind.purpose(),
        observed_at_epoch_ms,
        exit_code,
        stdout_blake3: zero.clone(),
        stderr_blake3: zero.clone(),
        identity_binding_verified: false,
        observation_digest: probe_digest(
            spec,
            observed_at_epoch_ms,
            exit_code,
            &zero,
            &zero,
            false,
        ),
        standing: standing_from_reason(reason),
        reason: reason.to_string(),
    }
}

fn probe_digest(
    spec: &ReadOnlyProbeSpec,
    observed_at_epoch_ms: i64,
    exit_code: i32,
    stdout_blake3: &str,
    stderr_blake3: &str,
    identity_binding_verified: bool,
) -> String {
    let payload = serde_json::json!({
        "cell_id": spec.cell_id,
        "adapter_id": spec.adapter_id,
        "workload_identity": spec.workload_identity,
        "provider_semantics_version": spec.provider_semantics_version,
        "kind": spec.kind,
        "observed_at_epoch_ms": observed_at_epoch_ms,
        "exit_code": exit_code,
        "stdout_blake3": stdout_blake3,
        "stderr_blake3": stderr_blake3,
        "identity_binding_verified": identity_binding_verified,
    });
    serde_json::to_vec(&payload)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .unwrap_or_else(|_| "0".repeat(64))
}

fn standing_from_reason(reason: &str) -> ReleaseStanding {
    if reason.starts_with("REFUSED:") {
        ReleaseStanding::Refused
    } else if reason.starts_with("BLOCKED:") {
        ReleaseStanding::Blocked
    } else if reason.starts_with("UNKNOWN:") {
        ReleaseStanding::Unknown
    } else if reason.starts_with("PARTIAL_ALIVE:") {
        ReleaseStanding::PartialAlive
    } else {
        ReleaseStanding::Alive
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveDeploymentQualification {
    pub standing: ReleaseStanding,
    pub static_manifest_digest: String,
    pub cells: usize,
    pub adapters_expected: usize,
    pub adapters_alive: usize,
    pub findings: Vec<String>,
}

fn observation_valid_shape(observation: &ProviderProbeObservation) -> bool {
    digest_ok(&observation.stdout_blake3)
        && digest_ok(&observation.stderr_blake3)
        && digest_ok(&observation.observation_digest)
}

/// Upgrade a statically valid deployment graph into a live-subject claim only
/// from fresh version + authority-context observations for every adapter.
/// Missing evidence is UNKNOWN; unavailable live tooling is BLOCKED; mismatched
/// semantics/identity is REFUSED. Static configuration alone can never crown a
/// live deployment ALIVE.
#[must_use]
pub fn qualify_live_deployment(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    now_epoch_ms: i64,
    max_age_ms: i64,
) -> LiveDeploymentQualification {
    let static_q = qualify_deployment(manifest, now_epoch_ms);
    if static_q.standing != ReleaseStanding::Alive {
        return LiveDeploymentQualification {
            standing: ReleaseStanding::Refused,
            static_manifest_digest: static_q.manifest_digest,
            cells: manifest.cells.len(),
            adapters_expected: manifest.cells.iter().map(|cell| cell.adapters.len()).sum(),
            adapters_alive: 0,
            findings: static_q.findings,
        };
    }

    let mut findings = Vec::new();
    let mut adapters_alive = 0usize;
    let adapters_expected = manifest.cells.iter().map(|cell| cell.adapters.len()).sum();
    for cell in &manifest.cells {
        for adapter in &cell.adapters {
            let rows: Vec<&ProviderProbeObservation> = observations
                .iter()
                .filter(|row| row.cell_id == cell.cell_id && row.adapter_id == adapter.adapter_id)
                .collect();
            if rows.is_empty() {
                findings.push(format!(
                    "UNKNOWN:MISSING_LIVE_ADAPTER_EVIDENCE:{}:{}",
                    cell.cell_id, adapter.adapter_id
                ));
                continue;
            }
            let mut adapter_failed = false;
            for row in &rows {
                if !observation_valid_shape(row) {
                    findings.push(format!(
                        "REFUSED:INVALID_LIVE_OBSERVATION:{}:{}",
                        cell.cell_id, adapter.adapter_id
                    ));
                    adapter_failed = true;
                }
                if row.provider_kind != adapter.kind
                    || row.provider_semantics_version != adapter.provider_semantics_version
                {
                    findings.push(format!(
                        "REFUSED:LIVE_PROVIDER_SEMANTICS_MISMATCH:{}:{}",
                        cell.cell_id, adapter.adapter_id
                    ));
                    adapter_failed = true;
                }
                if row.workload_identity != adapter.workload_identity {
                    findings.push(format!(
                        "REFUSED:LIVE_WORKLOAD_IDENTITY_MISMATCH:{}:{}",
                        cell.cell_id, adapter.adapter_id
                    ));
                    adapter_failed = true;
                }
                if row.observed_at_epoch_ms > now_epoch_ms {
                    findings.push(format!(
                        "REFUSED:LIVE_OBSERVATION_FROM_FUTURE:{}:{}",
                        cell.cell_id, adapter.adapter_id
                    ));
                    adapter_failed = true;
                } else if now_epoch_ms.saturating_sub(row.observed_at_epoch_ms) > max_age_ms {
                    findings.push(format!(
                        "UNKNOWN:STALE_LIVE_ADAPTER_EVIDENCE:{}:{}",
                        cell.cell_id, adapter.adapter_id
                    ));
                    adapter_failed = true;
                }
                if row.standing == ReleaseStanding::Blocked {
                    findings.push(format!(
                        "BLOCKED:LIVE_ADAPTER_PROBE:{}:{}:{}",
                        cell.cell_id, adapter.adapter_id, row.reason
                    ));
                    adapter_failed = true;
                } else if row.standing == ReleaseStanding::Refused {
                    findings.push(format!(
                        "REFUSED:LIVE_ADAPTER_PROBE:{}:{}:{}",
                        cell.cell_id, adapter.adapter_id, row.reason
                    ));
                    adapter_failed = true;
                }
            }
            let version_alive = rows.iter().any(|row| {
                row.purpose == ProbePurpose::Version && row.standing == ReleaseStanding::Alive
            });
            let context_alive = rows.iter().any(|row| {
                row.purpose == ProbePurpose::AuthorityContext
                    && row.standing == ReleaseStanding::Alive
                    && row.identity_binding_verified
            });
            if !version_alive {
                findings.push(format!(
                    "UNKNOWN:MISSING_LIVE_VERSION_PROOF:{}:{}",
                    cell.cell_id, adapter.adapter_id
                ));
                adapter_failed = true;
            }
            if !context_alive {
                findings.push(format!(
                    "UNKNOWN:MISSING_LIVE_AUTHORITY_PROOF:{}:{}",
                    cell.cell_id, adapter.adapter_id
                ));
                adapter_failed = true;
            }
            if !adapter_failed {
                adapters_alive += 1;
            }
        }
    }

    let standing = if findings.iter().any(|finding| finding.starts_with("REFUSED:")) {
        ReleaseStanding::Refused
    } else if findings.iter().any(|finding| finding.starts_with("BLOCKED:")) {
        ReleaseStanding::Blocked
    } else if findings.iter().any(|finding| finding.starts_with("UNKNOWN:")) {
        ReleaseStanding::Unknown
    } else {
        ReleaseStanding::Alive
    };
    LiveDeploymentQualification {
        standing,
        static_manifest_digest: static_q.manifest_digest,
        cells: manifest.cells.len(),
        adapters_expected,
        adapters_alive,
        findings,
    }
}

fn canonical_provider_program(kind: &str) -> Option<&'static str> {
    match kind {
        "aws" => Some("aws"),
        "azure" => Some("az"),
        "gcp" => Some("gcloud"),
        "kubernetes" => Some("kubectl"),
        "github" => Some("gh"),
        _ => None,
    }
}

fn program_basename(program: &str) -> Option<&str> {
    Path::new(program).file_name().and_then(|value| value.to_str())
}

/// Admit a concrete command policy against its ontology/topology adapter
/// binding before it is included in a runtime CONSTRUCT. This does not execute
/// the policy. Every command transition must already be bounded by the binding,
/// and the provider executable/workload identity must match the admitted family.
pub fn bind_command_policy(
    binding: &AdapterBinding,
    policy: &CommandAdapterPolicy,
) -> Result<CommandAdapterPolicy, String> {
    if binding.ambient_credentials {
        return Err("REFUSED:AMBIENT_CREDENTIAL_BINDING".to_string());
    }
    if binding.adapter_id != policy.adapter_id {
        return Err("REFUSED:ADAPTER_POLICY_ID_MISMATCH".to_string());
    }
    if binding.kind != policy.provider {
        return Err("REFUSED:ADAPTER_POLICY_PROVIDER_MISMATCH".to_string());
    }
    if binding.workload_identity != policy.workload_identity {
        return Err("REFUSED:ADAPTER_POLICY_WORKLOAD_IDENTITY_MISMATCH".to_string());
    }
    let expected_program = canonical_provider_program(&binding.kind)
        .ok_or_else(|| "UNSUPPORTED:UNKNOWN_PROVIDER_ADAPTER_KIND".to_string())?;
    if policy.commands.is_empty() {
        return Err("REFUSED:EMPTY_BOUND_PROVIDER_POLICY".to_string());
    }
    for (transition, command) in &policy.commands {
        if !binding.allowed_transition_ids.contains(transition)
            || command.transition_id != *transition
        {
            return Err(format!("REFUSED:UNADMITTED_BOUND_TRANSITION:{transition}"));
        }
        if !bounded_program(&command.program)
            || program_basename(&command.program) != Some(expected_program)
        {
            return Err(format!("REFUSED:PROVIDER_PROGRAM_MISMATCH:{transition}"));
        }
    }
    Ok(policy.clone())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolFenceQualification {
    pub standing: ReleaseStanding,
    pub dispatches: Vec<ProtocolDispatchSummary>,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolDispatchSummary {
    pub origin: InterfaceOrigin,
    pub select_standing: ReleaseStanding,
    pub construct_standing: ReleaseStanding,
    pub do_standing: ReleaseStanding,
}

fn self_test_intent(origin: InterfaceOrigin, mode: IntentMode) -> InterfaceIntent {
    InterfaceIntent {
        request_id: format!("dfcm:{origin:?}:{mode:?}"),
        origin,
        mode,
        subject: "subject:dfcm:self-test".to_string(),
        operation: "fence-self-test".to_string(),
        payload: serde_json::json!({"authority": "none"}),
        construct_admission_digest: (mode == IntentMode::Do).then(|| "a".repeat(64)),
        prepare_receipt_digest: (mode == IntentMode::Do).then(|| "b".repeat(64)),
    }
}

fn dispatch_standing(origin: InterfaceOrigin, mode: IntentMode) -> ReleaseStanding {
    dispatch_interface_intent(&self_test_intent(origin, mode)).standing
}

/// Execute the transport standing law across every externally advertised
/// origin. The fence is ALIVE only if SELECT/CONSTRUCT remain inert-ALIVE and a
/// fully shaped DO request remains PARTIAL_ALIVE rather than gaining authority.
#[must_use]
pub fn qualify_protocol_fence() -> ProtocolFenceQualification {
    let origins = [
        InterfaceOrigin::Cli,
        InterfaceOrigin::Api,
        InterfaceOrigin::Mcp,
        InterfaceOrigin::A2a,
        InterfaceOrigin::Human,
        InterfaceOrigin::Planner,
        InterfaceOrigin::Replay,
    ];
    let mut dispatches = Vec::new();
    let mut findings = Vec::new();
    for origin in origins {
        let select_standing = dispatch_standing(origin, IntentMode::Select);
        let construct_standing = dispatch_standing(origin, IntentMode::Construct);
        let do_standing = dispatch_standing(origin, IntentMode::Do);
        if select_standing != ReleaseStanding::Alive
            || construct_standing != ReleaseStanding::Alive
        {
            findings.push(format!("BUILD_BROKEN:INERT_PROTOCOL_ROUTE:{origin:?}"));
        }
        if do_standing != ReleaseStanding::PartialAlive {
            findings.push(format!("REFUSED:TRANSPORT_DO_STANDING_LEAK:{origin:?}"));
        }
        dispatches.push(ProtocolDispatchSummary {
            origin,
            select_standing,
            construct_standing,
            do_standing,
        });
    }
    ProtocolFenceQualification {
        standing: if findings.is_empty() {
            ReleaseStanding::Alive
        } else if findings.iter().any(|finding| finding.starts_with("REFUSED:")) {
            ReleaseStanding::Refused
        } else {
            ReleaseStanding::BuildBroken
        },
        dispatches,
        findings,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfcmClosureQualification {
    pub standing: ReleaseStanding,
    pub static_deployment: ReleaseStanding,
    pub live_deployment: ReleaseStanding,
    pub crypto_profile: ReleaseStanding,
    pub pqc_runtime: PqcRuntimeQualification,
    pub protocol_fence: ProtocolFenceQualification,
    pub findings: Vec<String>,
}

/// Final DfCM closure court. This is deliberately stricter than release build
/// standing: a globally deployed subject is ALIVE only when the static graph,
/// exact live adapter evidence, accepted crypto profile, real PQC self-tests and
/// transport authority fence all stand simultaneously.
#[must_use]
pub fn qualify_dfcm_closure(
    manifest: &DeploymentManifest,
    observations: &[ProviderProbeObservation],
    crypto_profile: &CryptoProfile,
    now_epoch_ms: i64,
    max_live_evidence_age_ms: i64,
) -> DfcmClosureQualification {
    let static_q = qualify_deployment(manifest, now_epoch_ms);
    let live_q = qualify_live_deployment(
        manifest,
        observations,
        now_epoch_ms,
        max_live_evidence_age_ms,
    );
    let crypto_q = qualify_crypto_profile(crypto_profile);
    let pqc_runtime = qualify_pqc_runtime();
    let protocol_fence = qualify_protocol_fence();
    let mut findings = Vec::new();
    for (name, standing) in [
        ("static_deployment", static_q.standing),
        ("live_deployment", live_q.standing),
        ("crypto_profile", crypto_q.standing),
        ("pqc_runtime", pqc_runtime.standing),
        ("protocol_fence", protocol_fence.standing),
    ] {
        if standing != ReleaseStanding::Alive {
            findings.push(format!("{}:{name}", standing.as_str()));
        }
    }
    findings.extend(live_q.findings.iter().cloned());
    findings.extend(crypto_q.reasons.iter().cloned());
    findings.extend(pqc_runtime.reasons.iter().cloned());
    findings.extend(protocol_fence.findings.iter().cloned());

    let standings = [
        static_q.standing,
        live_q.standing,
        crypto_q.standing,
        pqc_runtime.standing,
        protocol_fence.standing,
    ];
    let standing = if standings.contains(&ReleaseStanding::Refused) {
        ReleaseStanding::Refused
    } else if standings.contains(&ReleaseStanding::BuildBroken) {
        ReleaseStanding::BuildBroken
    } else if standings.contains(&ReleaseStanding::Blocked) {
        ReleaseStanding::Blocked
    } else if standings.contains(&ReleaseStanding::Unsupported) {
        ReleaseStanding::Unsupported
    } else if standings.contains(&ReleaseStanding::Unknown) {
        ReleaseStanding::Unknown
    } else if standings.contains(&ReleaseStanding::PartialAlive) {
        ReleaseStanding::PartialAlive
    } else {
        ReleaseStanding::Alive
    };

    DfcmClosureQualification {
        standing,
        static_deployment: static_q.standing,
        live_deployment: live_q.standing,
        crypto_profile: crypto_q.standing,
        pqc_runtime,
        protocol_fence,
        findings,
    }
}

/// Build the two required live-observation classes per adapter without
/// executing them. This is SELECT/CONSTRUCT output: deployment tooling may fan
/// these specs to authorized runners, but manufacturing specs creates no DO.
#[must_use]
pub fn manufacture_live_probe_plan(manifest: &DeploymentManifest) -> Vec<ReadOnlyProbeSpec> {
    let mut plan = Vec::new();
    for cell in &manifest.cells {
        for adapter in &cell.adapters {
            let kinds = match adapter.kind.as_str() {
                "aws" => [ReadOnlyProbeKind::AwsCliVersion, ReadOnlyProbeKind::AwsAuthorityContext],
                "azure" => [ReadOnlyProbeKind::AzureCliVersion, ReadOnlyProbeKind::AzureAuthorityContext],
                "gcp" => [ReadOnlyProbeKind::GcpCliVersion, ReadOnlyProbeKind::GcpAuthorityContext],
                "kubernetes" => [ReadOnlyProbeKind::KubernetesCliVersion, ReadOnlyProbeKind::KubernetesAuthorityContext],
                "github" => [ReadOnlyProbeKind::GitHubCliVersion, ReadOnlyProbeKind::GitHubAuthorityContext],
                _ => continue,
            };
            for kind in kinds {
                plan.push(ReadOnlyProbeSpec {
                    cell_id: cell.cell_id.clone(),
                    adapter_id: adapter.adapter_id.clone(),
                    workload_identity: adapter.workload_identity.clone(),
                    provider_semantics_version: adapter.provider_semantics_version.clone(),
                    kind,
                    program_override: None,
                    expected_identity_marker: None,
                    max_output_bytes: 64 * 1024,
                    timeout_ms: 30_000,
                });
            }
        }
    }
    plan
}

/// Summarize observations by cell without collapsing evidence classes.
#[must_use]
pub fn live_observation_index(
    observations: &[ProviderProbeObservation],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for observation in observations {
        index
            .entry(observation.cell_id.clone())
            .or_default()
            .insert(observation.adapter_id.clone());
    }
    index
}
