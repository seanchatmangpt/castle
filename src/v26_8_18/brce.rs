use std::collections::{BTreeMap, BTreeSet};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::castle::{
    create_receipt, execute_powl_with_gym_act, ActuationPermit, Blake3Provider, ConstructAdmission,
    DoAuthorizationContext, EpistemicClass, GymActAdapter, GymActResult, GymActStatus, OcelObject,
    PowlActivity, PowlProcess, Receipt, ReceiptSigner, ReceiptedOcelLog, TestEnvelope, WorldState,
};

use super::{ReleaseStanding, RELEASE_VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrceTransitionRecord {
    pub transition_id: String,
    pub prepare_receipt: Receipt,
    pub outcome_receipt: Option<Receipt>,
    pub standing: ReleaseStanding,
    pub reason: String,
}

/// BRCE wrapper around any GymAct adapter. Receipt manufacture happens before
/// the inner adapter can execute. REFUSED provider outcomes are receipted too.
pub struct BrceGymActAdapter<'a> {
    inner: &'a dyn GymActAdapter,
    blake3: &'a dyn Blake3Provider,
    signer: &'a dyn ReceiptSigner,
    journal: Mutex<Vec<BrceTransitionRecord>>,
}

impl<'a> BrceGymActAdapter<'a> {
    #[must_use]
    pub fn new(inner: &'a dyn GymActAdapter, blake3: &'a dyn Blake3Provider, signer: &'a dyn ReceiptSigner) -> Self {
        Self { inner, blake3, signer, journal: Mutex::new(Vec::new()) }
    }

    #[must_use]
    pub fn journal(&self) -> Vec<BrceTransitionRecord> {
        self.journal.lock().map(|journal| journal.clone()).unwrap_or_default()
    }
}

fn permit_payload(activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> Value {
    json!({
        "kind": "CASTLE_BRCE_PREPARE_V1",
        "release": RELEASE_VERSION,
        "activity_id": activity.id,
        "transition_id": permit.transition_id,
        "subject": permit.subject,
        "state_system_id": state.system_id,
        "authority": permit.authority,
        "construct_digest": permit.construct_digest,
        "process_digest": permit.process_digest,
        "expires_at_epoch_ms": permit.expires_at_epoch_ms,
    })
}

fn outcome_payload(result: &GymActResult, prepare_receipt_digest: &str) -> Value {
    json!({
        "kind": "CASTLE_BRCE_OUTCOME_V1",
        "release": RELEASE_VERSION,
        "transition_id": result.transition_id,
        "status": match result.status { GymActStatus::Observed => "OBSERVED", GymActStatus::Refused => "REFUSED" },
        "prepare_receipt_digest": prepare_receipt_digest,
        "objects": result.objects.iter().map(|o| json!({"id": o.id, "kind": o.kind})).collect::<Vec<_>>(),
        "attributes": result.attributes,
    })
}

#[async_trait]
impl GymActAdapter for BrceGymActAdapter<'_> {
    async fn execute(&self, activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        let prepare = match create_receipt(
            &permit_payload(activity, state, permit),
            EpistemicClass::Constructed,
            &permit.subject,
            &[permit.construct_digest.clone()],
            self.blake3,
            self.signer,
        ) {
            Ok(receipt) => receipt,
            Err(error) => {
                return GymActResult {
                    transition_id: activity.transition_id.clone(),
                    status: GymActStatus::Refused,
                    objects: Vec::new(),
                    attributes: BTreeMap::from([
                        ("reason".to_string(), json!("REFUSED:BRCE_PREPARE_FAILED")),
                        ("detail".to_string(), json!(error)),
                    ]),
                };
            }
        };

        let Ok(mut journal) = self.journal.lock() else {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([("reason".to_string(), json!("BLOCKED:BRCE_JOURNAL_UNAVAILABLE"))]),
            };
        };
        journal.push(BrceTransitionRecord {
            transition_id: activity.transition_id.clone(),
            prepare_receipt: prepare.clone(),
            outcome_receipt: None,
            standing: ReleaseStanding::PartialAlive,
            reason: "PARTIAL_ALIVE:PREPARED".to_string(),
        });
        drop(journal);

        let mut result = self.inner.execute(activity, state, permit).await;
        let outcome = create_receipt(
            &outcome_payload(&result, &prepare.receipt_digest),
            EpistemicClass::Observed,
            &permit.subject,
            &[prepare.artifact_digest.clone()],
            self.blake3,
            self.signer,
        );

        match outcome {
            Ok(outcome_receipt) => {
                result.attributes.insert("brce_prepare_receipt_digest".to_string(), json!(prepare.receipt_digest));
                result.attributes.insert("brce_outcome_receipt_digest".to_string(), json!(outcome_receipt.receipt_digest));
                let standing = if result.status == GymActStatus::Observed { ReleaseStanding::Alive } else { ReleaseStanding::Refused };
                let reason = if result.status == GymActStatus::Observed { "ALIVE:BRCE_COMMITTED" } else { "REFUSED:GYMACT_REFUSED" };
                if let Ok(mut journal) = self.journal.lock() {
                    if let Some(record) = journal.last_mut() {
                        record.outcome_receipt = Some(outcome_receipt);
                        record.standing = standing;
                        record.reason = reason.to_string();
                    }
                }
            }
            Err(error) => {
                result.status = GymActStatus::Refused;
                result.attributes.insert("reason".to_string(), json!("BLOCKED:OUTCOME_RECEIPT_FAILED"));
                result.attributes.insert("detail".to_string(), json!(error));
                if let Ok(mut journal) = self.journal.lock() {
                    if let Some(record) = journal.last_mut() {
                        record.standing = ReleaseStanding::Blocked;
                        record.reason = "BLOCKED:OUTCOME_RECEIPT_FAILED".to_string();
                    }
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandSpec {
    pub transition_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub allowed_exit_codes: BTreeSet<i32>,
    pub max_output_bytes: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandAdapterPolicy {
    pub adapter_id: String,
    pub provider: String,
    pub workload_identity: String,
    pub commands: BTreeMap<String, CommandSpec>,
}

/// Concrete subprocess provider adapter. The type is private, so callers
/// cannot obtain the official real adapter and invoke GymAct directly.
struct CommandGymActAdapter {
    policy: CommandAdapterPolicy,
}

impl CommandGymActAdapter {
    fn new(policy: CommandAdapterPolicy) -> Result<Self, String> {
        if policy.adapter_id.is_empty() || policy.workload_identity.is_empty() {
            return Err("REFUSED:INVALID_COMMAND_ADAPTER_POLICY".to_string());
        }
        if policy.commands.is_empty() {
            return Err("REFUSED:EMPTY_COMMAND_ADAPTER".to_string());
        }
        for (transition, spec) in &policy.commands {
            if transition != &spec.transition_id
                || spec.program.is_empty()
                || spec.allowed_exit_codes.is_empty()
                || spec.max_output_bytes == 0
                || spec.timeout_ms == 0
            {
                return Err(format!("REFUSED:INVALID_COMMAND_SPEC:{transition}"));
            }
            if spec.program.contains(char::is_whitespace) {
                return Err(format!("REFUSED:SHELL_LIKE_PROGRAM:{transition}"));
            }
        }
        Ok(Self { policy })
    }
}

#[must_use]
pub fn validate_command_adapter_policy(policy: &CommandAdapterPolicy) -> ReleaseStanding {
    match CommandGymActAdapter::new(policy.clone()) {
        Ok(_) => ReleaseStanding::Alive,
        Err(_) => ReleaseStanding::Refused,
    }
}

/// Official real-provider actuation entrypoint. A genuine opaque
/// ConstructAdmission is mandatory and execution goes back through the
/// existing exclusive POWL DO function with BRCE wrapped around every
/// transition. The private command adapter never escapes this call.
#[allow(clippy::too_many_arguments)]
pub async fn execute_command_process(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    admission: &ConstructAdmission,
    policy: CommandAdapterPolicy,
    blake3: &dyn Blake3Provider,
    signer: &dyn ReceiptSigner,
    now: impl Fn() -> i64,
) -> Result<(ReceiptedOcelLog, Vec<BrceTransitionRecord>), String> {
    let adapter = CommandGymActAdapter::new(policy)?;
    let brce = BrceGymActAdapter::new(&adapter, blake3, signer);
    let log = execute_powl_with_gym_act(
        process,
        state,
        envelope,
        &brce,
        DoAuthorizationContext { admission, blake3, receipt_signer: signer, now: Box::new(now) },
    )
    .await?;
    let journal = brce.journal();
    if journal.len() != log.log.events.len()
        || journal.iter().any(|record| record.standing != ReleaseStanding::Alive || record.outcome_receipt.is_none())
    {
        return Err("BLOCKED:BRCE_JOURNAL_INCOMPLETE".to_string());
    }
    Ok((log, journal))
}

#[async_trait]
impl GymActAdapter for CommandGymActAdapter {
    async fn execute(&self, activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        let Some(spec) = self.policy.commands.get(&activity.transition_id) else {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([("reason".to_string(), json!("REFUSED:TRANSITION_NOT_MAPPED"))]),
            };
        };
        if permit.transition_id != activity.transition_id || permit.subject != state.system_id {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([("reason".to_string(), json!("REFUSED:PERMIT_SUBJECT_OR_TRANSITION_MISMATCH"))]),
            };
        }

        let child = Command::new(&spec.program)
            .args(&spec.args)
            .env_clear()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let Ok(mut child) = child else {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([("reason".to_string(), json!("REFUSED:COMMAND_SPAWN_FAILED"))]),
            };
        };

        let started = Instant::now();
        let timed_out = loop {
            match child.try_wait() {
                Ok(Some(_)) => break false,
                Ok(None) if started.elapsed() < Duration::from_millis(spec.timeout_ms) => thread::sleep(Duration::from_millis(5)),
                Ok(None) => { let _ = child.kill(); break true; }
                Err(_) => { let _ = child.kill(); break true; }
            }
        };
        let output = child.wait_with_output();
        let Ok(output) = output else {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([("reason".to_string(), json!("REFUSED:COMMAND_WAIT_FAILED"))]),
            };
        };
        if timed_out {
            return GymActResult {
                transition_id: activity.transition_id.clone(),
                status: GymActStatus::Refused,
                objects: Vec::new(),
                attributes: BTreeMap::from([
                    ("reason".to_string(), json!("REFUSED:COMMAND_TIMEOUT")),
                    ("timeout_ms".to_string(), json!(spec.timeout_ms)),
                ]),
            };
        }

        let code = output.status.code().unwrap_or(-1);
        let stdout = &output.stdout[..output.stdout.len().min(spec.max_output_bytes)];
        let stderr = &output.stderr[..output.stderr.len().min(spec.max_output_bytes)];
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: if spec.allowed_exit_codes.contains(&code) { GymActStatus::Observed } else { GymActStatus::Refused },
            objects: vec![OcelObject {
                id: format!("command:{}:{}", self.policy.adapter_id, activity.transition_id),
                kind: "ProviderTransition".to_string(),
            }],
            attributes: BTreeMap::from([
                ("provider".to_string(), json!(self.policy.provider)),
                ("workload_identity".to_string(), json!(self.policy.workload_identity)),
                ("exit_code".to_string(), json!(code)),
                ("stdout_blake3".to_string(), json!(blake3::hash(stdout).to_hex().to_string())),
                ("stderr_blake3".to_string(), json!(blake3::hash(stderr).to_hex().to_string())),
                ("stdout_truncated_bytes".to_string(), json!(stdout.len())),
                ("stderr_truncated_bytes".to_string(), json!(stderr.len())),
            ]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAdapterDescriptor {
    pub kind: String,
    pub program: String,
    pub authority_model: String,
    pub ambient_credentials_allowed: bool,
}

#[must_use]
pub fn fortune5_adapter_catalog() -> Vec<ProviderAdapterDescriptor> {
    vec![
        ProviderAdapterDescriptor { kind: "aws".to_string(), program: "aws".to_string(), authority_model: "workload-identity / assumed-role per admitted transition".to_string(), ambient_credentials_allowed: false },
        ProviderAdapterDescriptor { kind: "azure".to_string(), program: "az".to_string(), authority_model: "managed-identity / federated workload identity".to_string(), ambient_credentials_allowed: false },
        ProviderAdapterDescriptor { kind: "gcp".to_string(), program: "gcloud".to_string(), authority_model: "workload identity federation / service account impersonation".to_string(), ambient_credentials_allowed: false },
        ProviderAdapterDescriptor { kind: "kubernetes".to_string(), program: "kubectl".to_string(), authority_model: "namespace-scoped service account / projected identity".to_string(), ambient_credentials_allowed: false },
        ProviderAdapterDescriptor { kind: "github".to_string(), program: "github-app".to_string(), authority_model: "installation token scoped to repository/operation".to_string(), ambient_credentials_allowed: false },
    ]
}
