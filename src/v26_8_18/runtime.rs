use std::collections::BTreeSet;

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::castle::{
    admit_construct_for_do, manufacture_construct_capability, Blake3Provider, ConstructCapability,
    ConstructRequest, ConstructTrustPolicy, PowlActivity, PowlProcess, ReceiptSigner, ReceiptVerifier,
    TestEnvelope, WorldState,
};

use super::{
    execute_command_process, BrceTransitionRecord, CommandAdapterPolicy, ReleaseStanding,
};

pub struct NativeBlake3;
impl Blake3Provider for NativeBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

pub struct Ed25519RuntimeSigner {
    key_id: String,
    signing_key: SigningKey,
}

impl Ed25519RuntimeSigner {
    pub fn from_seed(key_id: String, seed: [u8; 32]) -> Result<Self, String> {
        if key_id.is_empty() {
            return Err("REFUSED:EMPTY_RUNTIME_SIGNING_KEY_ID".to_string());
        }
        Ok(Self { key_id, signing_key: SigningKey::from_bytes(&seed) })
    }

    #[must_use]
    pub fn verifier(&self) -> Ed25519RuntimeVerifier {
        Ed25519RuntimeVerifier { key_id: self.key_id.clone(), verifying_key: self.signing_key.verifying_key() }
    }
}

impl ReceiptSigner for Ed25519RuntimeSigner {
    fn key_id(&self) -> &str { &self.key_id }

    fn sign_digest(&self, digest_hex: &str) -> String {
        let Ok(bytes) = decode_hex(digest_hex) else { return String::new(); };
        self.signing_key.sign(&bytes).to_bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}

pub struct Ed25519RuntimeVerifier {
    key_id: String,
    verifying_key: VerifyingKey,
}

impl ReceiptVerifier for Ed25519RuntimeVerifier {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool {
        if key_id != self.key_id { return false; }
        let Ok(digest) = decode_hex(digest_hex) else { return false; };
        let Ok(sig_bytes) = decode_hex(signature) else { return false; };
        let Ok(sig_array) = <[u8; 64]>::try_from(sig_bytes.as_slice()) else { return false; };
        let signature = ed25519_dalek::Signature::from_bytes(&sig_array);
        self.verifying_key.verify(&digest, &signature).is_ok()
    }
}

pub fn decode_seed_hex(value: &str) -> Result<[u8; 32], String> {
    let bytes = decode_hex(value.trim())?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| "REFUSED:RUNTIME_SIGNING_SEED_MUST_BE_32_BYTES".to_string())
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("REFUSED:INVALID_HEX".to_string());
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).map_err(|_| "REFUSED:INVALID_HEX".to_string()))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableActivity {
    pub id: String,
    pub transition_id: String,
    pub predecessors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableProcess {
    pub id: String,
    pub goal_id: String,
    pub activities: Vec<PortableActivity>,
}

impl PortableProcess {
    #[must_use]
    pub fn to_powl(&self) -> PowlProcess {
        PowlProcess {
            id: self.id.clone(),
            goal_id: self.goal_id.clone(),
            activities: self.activities.iter().map(|a| PowlActivity {
                id: a.id.clone(), transition_id: a.transition_id.clone(), predecessors: a.predecessors.clone(),
            }).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortableEnvelope {
    pub system_id: String,
    pub allowed_transition_ids: BTreeSet<String>,
    pub max_steps: u32,
    pub expires_at_epoch_ms: i64,
}

impl PortableEnvelope {
    #[must_use]
    pub fn to_envelope(&self) -> TestEnvelope {
        TestEnvelope {
            system_id: self.system_id.clone(),
            allowed_transition_ids: self.allowed_transition_ids.clone(),
            max_steps: self.max_steps,
            expires_at_epoch_ms: self.expires_at_epoch_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeExecutionRequest {
    pub subject: String,
    pub authority: String,
    pub o_star: Value,
    pub config_graph: Value,
    pub ontology: Value,
    pub process: PortableProcess,
    pub envelope: PortableEnvelope,
    pub allowed_authorities: BTreeSet<String>,
    pub adapter_policy: CommandAdapterPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructManufactureSummary {
    pub standing: ReleaseStanding,
    pub construct_digest: String,
    pub construct_receipt_digest: String,
    pub process_digest: String,
    pub replay_identity_digest: String,
    pub subject: String,
    pub authority: String,
}

struct ManufacturedRuntimeConstruct {
    capability: ConstructCapability,
    process: PowlProcess,
    envelope: TestEnvelope,
    signer: Ed25519RuntimeSigner,
    verifier: Ed25519RuntimeVerifier,
}

fn manufacture_internal(request: &RuntimeExecutionRequest, key_id: String, seed: [u8; 32]) -> Result<ManufacturedRuntimeConstruct, String> {
    if request.subject != request.envelope.system_id {
        return Err("REFUSED:RUNTIME_SUBJECT_MISMATCH".to_string());
    }
    if !request.allowed_authorities.contains(&request.authority) {
        return Err("REFUSED:RUNTIME_AUTHORITY_NOT_ALLOWED".to_string());
    }
    let process = request.process.to_powl();
    let envelope = request.envelope.to_envelope();
    let signer = Ed25519RuntimeSigner::from_seed(key_id, seed)?;
    let verifier = signer.verifier();
    let capability = manufacture_construct_capability(
        ConstructRequest {
            subject: request.subject.clone(),
            authority: request.authority.clone(),
            o_star: request.o_star.clone(),
            config_graph: request.config_graph.clone(),
            ontology: request.ontology.clone(),
            process: process.clone(),
            envelope: envelope.clone(),
        },
        &NativeBlake3,
        &signer,
    )?;
    Ok(ManufacturedRuntimeConstruct { capability, process, envelope, signer, verifier })
}

pub fn manufacture_runtime_construct(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
) -> Result<ConstructManufactureSummary, String> {
    let manufactured = manufacture_internal(request, key_id, seed)?;
    Ok(summary(&manufactured))
}

fn summary(manufactured: &ManufacturedRuntimeConstruct) -> ConstructManufactureSummary {
    ConstructManufactureSummary {
        standing: ReleaseStanding::Alive,
        construct_digest: manufactured.capability.receipt.artifact_digest.clone(),
        construct_receipt_digest: manufactured.capability.receipt.receipt_digest.clone(),
        process_digest: manufactured.capability.artifact.process_digest.clone(),
        replay_identity_digest: manufactured.capability.artifact.replay_identity_digest.clone(),
        subject: manufactured.capability.artifact.subject.clone(),
        authority: manufactured.capability.artifact.authority.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeDoSummary {
    pub standing: ReleaseStanding,
    pub construct: ConstructManufactureSummary,
    pub ocel_receipt_digest: String,
    pub event_count: usize,
    pub brce_prepare_receipt_digests: Vec<String>,
    pub brce_outcome_receipt_digests: Vec<String>,
}

/// CLI/API callers cannot submit a raw DO. They must first manufacture the
/// exact deterministic CONSTRUCT and carry its digest into this second stage.
/// This function recomputes that CONSTRUCT, compares identity, performs the
/// normal opaque ConstructAdmission, then enters the private real-provider
/// execution rail. The expected digest is a checkpoint, not authority.
pub async fn execute_runtime_request(
    request: &RuntimeExecutionRequest,
    key_id: String,
    seed: [u8; 32],
    expected_construct_digest: &str,
    now_epoch_ms: i64,
) -> Result<RuntimeDoSummary, String> {
    if expected_construct_digest.len() != 64 {
        return Err("REFUSED:INVALID_EXPECTED_CONSTRUCT_DIGEST".to_string());
    }
    let manufactured = manufacture_internal(request, key_id, seed)?;
    let construct = summary(&manufactured);
    if construct.construct_digest != expected_construct_digest {
        return Err("REFUSED:CONSTRUCT_CHECKPOINT_MISMATCH".to_string());
    }
    let policy = ConstructTrustPolicy {
        trusted_origin_key_ids: BTreeSet::from([manufactured.signer.key_id().to_string()]),
        allowed_authorities: request.allowed_authorities.clone(),
    };
    let admission = admit_construct_for_do(
        &manufactured.capability,
        &manufactured.process,
        &manufactured.envelope,
        &NativeBlake3,
        &manufactured.verifier,
        &policy,
        || now_epoch_ms,
    )?;
    let state = WorldState { system_id: request.subject.clone(), facts: BTreeSet::new() };
    let (log, journal): (_, Vec<BrceTransitionRecord>) = execute_command_process(
        &manufactured.process,
        &state,
        &manufactured.envelope,
        &admission,
        request.adapter_policy.clone(),
        &NativeBlake3,
        &manufactured.signer,
        || now_epoch_ms,
    ).await?;

    Ok(RuntimeDoSummary {
        standing: ReleaseStanding::Alive,
        construct,
        ocel_receipt_digest: log.receipt.receipt_digest,
        event_count: log.log.events.len(),
        brce_prepare_receipt_digests: journal.iter().map(|r| r.prepare_receipt.receipt_digest.clone()).collect(),
        brce_outcome_receipt_digests: journal.iter().filter_map(|r| r.outcome_receipt.as_ref().map(|receipt| receipt.receipt_digest.clone())).collect(),
    })
}
