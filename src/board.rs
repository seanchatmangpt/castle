//! Port of `board.ts`: the independent `ReceiptV2` (BLAKE3 + real Ed25519) chain used
//! for Fortune-5 evidence admission and board-package qualification. Deliberately does
//! **not** share types with `castle.rs`'s `Receipt` — see the "two independent receipt
//! systems" note in this crate's CLAUDE.md.

use std::collections::{BTreeMap, HashMap};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::{json, Value};

use crate::fortune5::{
    qualify_fortune5, Fortune5Qualification, MetricObservation, QualificationContext, Standing,
};
use crate::fortune5_generated::{Fortune5Requirement, FORTUNE5_REQUIREMENTS};

fn digest_re_ok(digest: &str) -> bool {
    digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err("REFUSED:NON_FINITE_CANONICAL_VALUE".to_string());
                }
            }
            Ok(n.to_string())
        }
        Value::String(s) => Ok(serde_json::to_string(s).unwrap()),
        Value::Array(arr) => {
            let parts: Result<Vec<String>, String> = arr.iter().map(canonical_json).collect();
            Ok(format!("[{}]", parts?.join(",")))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for k in keys {
                parts.push(format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_json(&map[k])?));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
    }
}

fn blake3_hex(input: &str) -> String {
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

pub const SIGNATURE_ALGORITHM: &str = "Ed25519";

#[derive(Debug, Clone)]
pub struct ReceiptCoreV2 {
    pub version: &'static str, // "CASTLE-RECEIPT-V2"
    pub algorithm: &'static str, // "BLAKE3-256"
    pub signature_algorithm: &'static str, // "Ed25519"
    pub payload_digest: String,
    pub subject: String,
    pub metric: String,
    pub policy_digest: String,
    pub authority_digest: String,
    pub parent_digests: Vec<String>,
    pub key_id: String,
    pub trust_epoch: i64,
    pub issued_at: String,
    pub assurance_domain: String,
}

impl ReceiptCoreV2 {
    fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "algorithm": self.algorithm,
            "signature_algorithm": self.signature_algorithm,
            "payload_digest": self.payload_digest,
            "subject": self.subject,
            "metric": self.metric,
            "policy_digest": self.policy_digest,
            "authority_digest": self.authority_digest,
            "parent_digests": self.parent_digests,
            "key_id": self.key_id,
            "trust_epoch": self.trust_epoch,
            "issued_at": self.issued_at,
            "assurance_domain": self.assurance_domain,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ReceiptV2 {
    pub core: ReceiptCoreV2,
    pub receipt_digest: String,
    pub signature: String, // base64
}

#[derive(Debug, Clone)]
pub struct TrustKey {
    pub key_id: String,
    pub verifying_key: VerifyingKey,
    pub valid_from_epoch: i64,
    pub revoked_at_epoch: Option<i64>,
    pub assurance_domain: String,
}

pub struct TrustStore {
    pub current_epoch: i64,
    pub keys: HashMap<String, TrustKey>,
}

#[derive(Debug, Clone)]
pub struct EvidenceInput {
    pub metric: String,
    pub value: crate::fortune5::MetricValue,
    pub subject: String,
    pub observed_at: String,
    pub epistemic_class: crate::fortune5::EvidenceEpistemicClass,
}

#[derive(Debug, Clone)]
pub struct EvidenceBundle {
    pub observation: MetricObservation,
    pub receipt: ReceiptV2,
}

pub struct ReceiptIssueContext {
    pub policy_digest: String,
    pub authority_digest: String,
    pub parent_digests: Vec<String>,
    pub key_id: String,
    pub trust_epoch: i64,
    pub assurance_domain: String,
}

fn observation_payload_json(metric: &str, value: &crate::fortune5::MetricValue, subject: &str, observed_at: &str, epistemic_class: crate::fortune5::EvidenceEpistemicClass) -> Value {
    let value_json = match value {
        crate::fortune5::MetricValue::Number(n) => json!(n),
        crate::fortune5::MetricValue::Bool(b) => json!(b),
        crate::fortune5::MetricValue::Str(s) => json!(s),
    };
    let epistemic_str = match epistemic_class {
        crate::fortune5::EvidenceEpistemicClass::Observed => "OBSERVED",
        crate::fortune5::EvidenceEpistemicClass::Replayed => "REPLAYED",
        crate::fortune5::EvidenceEpistemicClass::Inferred => "INFERRED",
    };
    json!({
        "metric": metric,
        "value": value_json,
        "subject": subject,
        "observed_at": observed_at,
        "epistemic_class": epistemic_str,
    })
}

fn validate_digest(label: &str, digest: &str) -> Result<(), String> {
    if !digest_re_ok(digest) {
        return Err(format!("REFUSED:INVALID_{label}_DIGEST"));
    }
    Ok(())
}

pub fn issue_evidence_receipt(input: &EvidenceInput, context: &ReceiptIssueContext, signing_key: &SigningKey) -> Result<EvidenceBundle, String> {
    validate_digest("POLICY", &context.policy_digest)?;
    validate_digest("AUTHORITY", &context.authority_digest)?;
    if context.key_id.is_empty() || context.assurance_domain.is_empty() {
        return Err("REFUSED:INCOMPLETE_RECEIPT_AUTHORITY".to_string());
    }
    if context.trust_epoch < 0 {
        return Err("REFUSED:INVALID_TRUST_EPOCH".to_string());
    }
    let payload = observation_payload_json(&input.metric, &input.value, &input.subject, &input.observed_at, input.epistemic_class);
    let payload_digest = blake3_hex(&canonical_json(&payload)?);
    let mut parent_digests = context.parent_digests.clone();
    parent_digests.sort();
    parent_digests.dedup();
    for parent in &parent_digests {
        validate_digest("PARENT", parent)?;
    }
    let core = ReceiptCoreV2 {
        version: "CASTLE-RECEIPT-V2",
        algorithm: "BLAKE3-256",
        signature_algorithm: "Ed25519",
        payload_digest,
        subject: input.subject.clone(),
        metric: input.metric.clone(),
        policy_digest: context.policy_digest.clone(),
        authority_digest: context.authority_digest.clone(),
        parent_digests,
        key_id: context.key_id.clone(),
        trust_epoch: context.trust_epoch,
        issued_at: input.observed_at.clone(),
        assurance_domain: context.assurance_domain.clone(),
    };
    let receipt_digest = blake3_hex(&canonical_json(&core.to_json())?);
    let digest_bytes = hex::decode(&receipt_digest).map_err(|_| "REFUSED:INVALID_RECEIPT_DIGEST".to_string())?;
    let signature: Signature = signing_key.sign(&digest_bytes);
    let signature_b64 = base64_encode(signature.to_bytes().as_slice());
    let observation = MetricObservation {
        metric: input.metric.clone(),
        value: input.value.clone(),
        receipt_digest: receipt_digest.clone(),
        subject: input.subject.clone(),
        observed_at: input.observed_at.clone(),
        epistemic_class: input.epistemic_class,
    };
    Ok(EvidenceBundle {
        observation,
        receipt: ReceiptV2 { core, receipt_digest, signature: signature_b64 },
    })
}

// Minimal base64 encode/decode so this crate doesn't need to pull in a dedicated base64
// crate just for signature transport; only standard (no padding-agnostic tricks) alphabet.
mod b64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(input: &[u8]) -> String {
        let mut out = String::new();
        for chunk in input.chunks(3) {
            let b0 = chunk[0];
            let b1 = *chunk.get(1).unwrap_or(&0);
            let b2 = *chunk.get(2).unwrap_or(&0);
            let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
            out.push(ALPHABET[((n >> 18) & 0x3f) as usize] as char);
            out.push(ALPHABET[((n >> 12) & 0x3f) as usize] as char);
            out.push(if chunk.len() > 1 { ALPHABET[((n >> 6) & 0x3f) as usize] as char } else { '=' });
            out.push(if chunk.len() > 2 { ALPHABET[(n & 0x3f) as usize] as char } else { '=' });
        }
        out
    }

    pub fn decode(input: &str) -> Result<Vec<u8>, String> {
        let clean: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
        let mut out = Vec::new();
        let mut buf = 0u32;
        let mut bits = 0u32;
        for b in clean {
            let val = ALPHABET.iter().position(|&c| c == b).ok_or_else(|| "invalid base64".to_string())? as u32;
            buf = (buf << 6) | val;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push(((buf >> bits) & 0xff) as u8);
            }
        }
        Ok(out)
    }
}

fn base64_encode(input: &[u8]) -> String {
    b64::encode(input)
}

mod hex {
    pub fn decode(input: &str) -> Result<Vec<u8>, String> {
        if input.len() % 2 != 0 {
            return Err("odd length".to_string());
        }
        (0..input.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&input[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ReceiptVerification {
    pub standing: Standing,
    pub reasons: Vec<String>,
    pub verified_digests: Vec<String>,
}

fn verify_receipt_node(
    receipt: &ReceiptV2,
    trust: &TrustStore,
    store: &HashMap<String, ReceiptV2>,
    visiting: &mut std::collections::HashSet<String>,
    verified: &mut std::collections::HashSet<String>,
    reasons: &mut Vec<String>,
) {
    if verified.contains(&receipt.receipt_digest) {
        return;
    }
    if visiting.contains(&receipt.receipt_digest) {
        reasons.push("REFUSED:RECEIPT_DAG_CYCLE".to_string());
        return;
    }
    visiting.insert(receipt.receipt_digest.clone());

    let core = &receipt.core;
    if core.version != "CASTLE-RECEIPT-V2" {
        reasons.push("REFUSED:UNSUPPORTED_RECEIPT_VERSION".to_string());
    }
    if core.algorithm != "BLAKE3-256" {
        reasons.push("REFUSED:UNSUPPORTED_RECEIPT_DIGEST_ALGORITHM".to_string());
    }
    if core.signature_algorithm != "Ed25519" {
        reasons.push("REFUSED:UNSUPPORTED_SIGNATURE_ALGORITHM".to_string());
    }
    if !digest_re_ok(&receipt.receipt_digest) {
        reasons.push("REFUSED:INVALID_RECEIPT_DIGEST".to_string());
    }
    if !digest_re_ok(&core.payload_digest) {
        reasons.push("REFUSED:INVALID_PAYLOAD_DIGEST".to_string());
    }
    if !digest_re_ok(&core.policy_digest) {
        reasons.push("REFUSED:INVALID_POLICY_DIGEST".to_string());
    }
    if !digest_re_ok(&core.authority_digest) {
        reasons.push("REFUSED:INVALID_AUTHORITY_DIGEST".to_string());
    }
    if core.trust_epoch < 0 || core.trust_epoch > trust.current_epoch {
        reasons.push("REFUSED:INVALID_TRUST_EPOCH".to_string());
    }

    match canonical_json(&core.to_json()) {
        Ok(canon) => {
            let expected_digest = blake3_hex(&canon);
            if expected_digest != receipt.receipt_digest {
                reasons.push("REFUSED:RECEIPT_CONTENT_MISMATCH".to_string());
            }
        }
        Err(_) => reasons.push("REFUSED:RECEIPT_CONTENT_MISMATCH".to_string()),
    }

    match trust.keys.get(&core.key_id) {
        None => reasons.push("REFUSED:UNTRUSTED_RECEIPT_KEY".to_string()),
        Some(key) => {
            if core.assurance_domain != key.assurance_domain {
                reasons.push("REFUSED:ASSURANCE_DOMAIN_MISMATCH".to_string());
            }
            if core.trust_epoch < key.valid_from_epoch {
                reasons.push("REFUSED:KEY_NOT_YET_VALID".to_string());
            }
            if let Some(revoked) = key.revoked_at_epoch {
                if core.trust_epoch >= revoked {
                    reasons.push("REFUSED:REVOKED_RECEIPT_KEY".to_string());
                }
            }
            let valid = match (hex::decode(&receipt.receipt_digest), b64::decode(&receipt.signature)) {
                (Ok(digest_bytes), Ok(sig_bytes)) => {
                    if digest_bytes.len() == 32 && sig_bytes.len() == 64 {
                        let sig_array: [u8; 64] = sig_bytes.try_into().unwrap();
                        let sig = Signature::from_bytes(&sig_array);
                        key.verifying_key.verify(&digest_bytes, &sig).is_ok()
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !valid {
                reasons.push("REFUSED:INVALID_RECEIPT_SIGNATURE".to_string());
            }
        }
    }

    for parent_digest in &core.parent_digests {
        match store.get(parent_digest) {
            None => reasons.push("REFUSED:ORPHAN_RECEIPT_PARENT".to_string()),
            Some(parent) => verify_receipt_node(parent, trust, store, visiting, verified, reasons),
        }
    }

    visiting.remove(&receipt.receipt_digest);
    verified.insert(receipt.receipt_digest.clone());
}

#[must_use]
pub fn verify_receipt_dag(root: &ReceiptV2, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> ReceiptVerification {
    let mut reasons: Vec<String> = Vec::new();
    let mut verified = std::collections::HashSet::new();
    verify_receipt_node(root, trust, store, &mut std::collections::HashSet::new(), &mut verified, &mut reasons);
    reasons.sort();
    reasons.dedup();
    let standing = if reasons.is_empty() { Standing::Alive } else { Standing::Refused };
    let reasons = if reasons.is_empty() { vec!["ALIVE:RECEIPT_DAG_VERIFIED".to_string()] } else { reasons };
    let mut verified_digests: Vec<String> = verified.into_iter().collect();
    verified_digests.sort();
    ReceiptVerification { standing, reasons, verified_digests }
}

#[derive(Debug, Clone)]
pub struct EvidenceAdmission {
    pub standing: Standing,
    pub reasons: Vec<String>,
    pub observation: Option<MetricObservation>,
}

pub fn admit_evidence(bundle: &EvidenceBundle, trust: &TrustStore, store: &HashMap<String, ReceiptV2>) -> EvidenceAdmission {
    let _span = tracing::info_span!(
        "admit_evidence",
        subject = %bundle.observation.subject,
        metric = %bundle.observation.metric,
        standing = tracing::field::Empty,
    )
    .entered();
    let mut reasons: Vec<String> = Vec::new();
    let observation = &bundle.observation;
    let receipt = &bundle.receipt;
    if observation.receipt_digest != receipt.receipt_digest {
        reasons.push("REFUSED:EVIDENCE_RECEIPT_REFERENCE_MISMATCH".to_string());
    }
    if receipt.core.subject != observation.subject {
        reasons.push("REFUSED:EVIDENCE_SUBJECT_MISMATCH".to_string());
    }
    if receipt.core.metric != observation.metric {
        reasons.push("REFUSED:EVIDENCE_METRIC_MISMATCH".to_string());
    }
    if receipt.core.issued_at != observation.observed_at {
        reasons.push("REFUSED:EVIDENCE_TIME_BINDING_MISMATCH".to_string());
    }
    let payload = observation_payload_json(&observation.metric, &observation.value, &observation.subject, &observation.observed_at, observation.epistemic_class);
    match canonical_json(&payload) {
        Ok(canon) => {
            if blake3_hex(&canon) != receipt.core.payload_digest {
                reasons.push("REFUSED:EVIDENCE_PAYLOAD_MISMATCH".to_string());
            }
        }
        Err(_) => reasons.push("REFUSED:EVIDENCE_PAYLOAD_MISMATCH".to_string()),
    }

    let dag = verify_receipt_dag(receipt, trust, store);
    if dag.standing != Standing::Alive {
        reasons.extend(dag.reasons);
    }
    reasons.sort();
    reasons.dedup();
    let standing = if reasons.is_empty() { Standing::Alive } else { Standing::Refused };
    let out_reasons = if reasons.is_empty() { vec!["ALIVE:RECEIPTED_EVIDENCE_ADMITTED".to_string()] } else { reasons };
    _span.record("standing", tracing::field::debug(standing));
    EvidenceAdmission {
        standing,
        reasons: out_reasons,
        observation: if standing == Standing::Alive { Some(observation.clone()) } else { None },
    }
}

#[derive(Debug, Clone)]
pub struct VerifiedQualification {
    pub standing: Standing,
    pub qualification: Fortune5Qualification,
    pub evidence_refusals: Vec<String>,
}

#[must_use]
pub fn qualify_verified_fortune5(
    bundles: &[EvidenceBundle],
    context: &QualificationContext,
    trust: &TrustStore,
    receipt_store: &HashMap<String, ReceiptV2>,
    requirements: &[Fortune5Requirement],
) -> VerifiedQualification {
    let mut admitted: Vec<MetricObservation> = Vec::new();
    let mut refusals: Vec<String> = Vec::new();
    for bundle in bundles {
        let admission = admit_evidence(bundle, trust, receipt_store);
        if admission.standing == Standing::Alive {
            if let Some(o) = admission.observation {
                admitted.push(o);
            }
        } else {
            for reason in admission.reasons {
                refusals.push(format!("{}:{}", bundle.observation.metric, reason));
            }
        }
    }
    refusals.sort();
    refusals.dedup();
    let qualification = qualify_fortune5(&admitted, context, requirements);
    let standing = if !refusals.is_empty() { Standing::Refused } else { qualification.standing };
    VerifiedQualification { standing, qualification, evidence_refusals: refusals }
}

pub struct BoardAdmissionInput<'a> {
    pub enterprise_context: QualificationContext,
    pub enterprise_evidence: Vec<EvidenceBundle>,
    pub castle_context: QualificationContext,
    pub castle_evidence: Vec<EvidenceBundle>,
    pub trust: &'a TrustStore,
    pub receipt_store: HashMap<String, ReceiptV2>,
    pub castle_assurance_domain: String,
    pub independent_assurance_domain: String,
}

#[derive(Debug, Clone)]
pub struct BoardAdmission {
    pub standing: Standing,
    pub enterprise: VerifiedQualification,
    pub castle: VerifiedQualification,
    pub reasons: Vec<String>,
}

fn find_evidence_domain(evidence: &[EvidenceBundle], metric: &str) -> Option<String> {
    evidence.iter().find(|item| item.observation.metric == metric).map(|item| item.receipt.core.assurance_domain.clone())
}

pub fn qualify_fortune5_board(input: BoardAdmissionInput) -> BoardAdmission {
    let _span = tracing::info_span!(
        "qualify_fortune5_board",
        castle_assurance_domain = %input.castle_assurance_domain,
        standing = tracing::field::Empty,
    )
    .entered();
    let enterprise = qualify_verified_fortune5(&input.enterprise_evidence, &input.enterprise_context, input.trust, &input.receipt_store, FORTUNE5_REQUIREMENTS);
    let castle = qualify_verified_fortune5(&input.castle_evidence, &input.castle_context, input.trust, &input.receipt_store, FORTUNE5_REQUIREMENTS);
    let mut reasons: Vec<String> = Vec::new();
    if enterprise.standing != Standing::Alive {
        reasons.push(format!("REFUSED:ENTERPRISE_NOT_ALIVE:{}", enterprise.standing.as_str()));
    }
    if castle.standing != Standing::Alive {
        reasons.push(format!("REFUSED:CASTLE_NOT_ALIVE:{}", castle.standing.as_str()));
    }
    if input.castle_assurance_domain.is_empty() || input.independent_assurance_domain.is_empty() || input.castle_assurance_domain == input.independent_assurance_domain {
        reasons.push("REFUSED:ASSURANCE_NOT_INDEPENDENT".to_string());
    }
    for metric in ["independent_verifier_agreement_bps", "board_package_independent_assurance_passed"] {
        let enterprise_domain = find_evidence_domain(&input.enterprise_evidence, metric);
        let castle_domain = find_evidence_domain(&input.castle_evidence, metric);
        if enterprise_domain.as_deref() != Some(input.independent_assurance_domain.as_str()) || castle_domain.as_deref() != Some(input.independent_assurance_domain.as_str()) {
            reasons.push(format!("REFUSED:INDEPENDENT_ASSURANCE_EVIDENCE:{metric}"));
        }
    }
    let standing = if reasons.is_empty() { Standing::Alive } else { Standing::Refused };
    let reasons = if reasons.is_empty() { vec!["ALIVE:BOARD_ADMISSION_PROVED".to_string()] } else { reasons };
    _span.record("standing", tracing::field::debug(standing));
    BoardAdmission { standing, enterprise, castle, reasons }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMode {
    FailClosed,
    SafeDegrade,
    LocalCapability,
    Defer,
}

pub struct FailureSemanticsInput {
    pub mode: FailureMode,
    pub castle_available: bool,
    pub local_capability_verified: bool,
    pub receipt_channel_available: bool,
}

#[derive(Debug, Clone)]
pub struct FailureSemanticsDecision {
    pub standing: Standing,
    pub may_actuate: bool,
    pub reason: String,
}

#[must_use]
pub fn admit_failure_semantics(input: &FailureSemanticsInput) -> FailureSemanticsDecision {
    if input.castle_available {
        return FailureSemanticsDecision {
            standing: if input.receipt_channel_available { Standing::Alive } else { Standing::Refused },
            may_actuate: input.receipt_channel_available,
            reason: if input.receipt_channel_available { "ALIVE:RECEIPTED_PRIMARY_PATH".to_string() } else { "REFUSED:NO_RECEIPT_CHANNEL".to_string() },
        };
    }
    if input.mode == FailureMode::FailClosed {
        return FailureSemanticsDecision { standing: Standing::Alive, may_actuate: false, reason: "ALIVE:FAIL_CLOSED".to_string() };
    }
    if input.mode == FailureMode::Defer {
        return FailureSemanticsDecision { standing: Standing::Alive, may_actuate: false, reason: "ALIVE:DEFERRED".to_string() };
    }
    if !input.receipt_channel_available {
        return FailureSemanticsDecision { standing: Standing::Refused, may_actuate: false, reason: "REFUSED:UNRECEIPTABLE_DEGRADED_DO".to_string() };
    }
    if input.mode == FailureMode::LocalCapability && !input.local_capability_verified {
        return FailureSemanticsDecision { standing: Standing::Refused, may_actuate: false, reason: "REFUSED:LOCAL_CAPABILITY_NOT_VERIFIED".to_string() };
    }
    FailureSemanticsDecision {
        standing: Standing::Alive,
        may_actuate: true,
        reason: if input.mode == FailureMode::LocalCapability { "ALIVE:RECEIPTED_LOCAL_CAPABILITY".to_string() } else { "ALIVE:RECEIPTED_SAFE_DEGRADE".to_string() },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaterialityDimension {
    Financial,
    Operational,
    Customer,
    Legal,
    Regulatory,
    Reputational,
    Systemic,
}

pub struct MaterialityPolicy {
    pub policy_digest: String,
    pub authority_digest: String,
    pub per_dimension_threshold_bps: BTreeMap<MaterialityDimension, i64>,
    pub aggregate_threshold_bps: i64,
    pub escalation_within_ms: i64,
}

pub struct MaterialityEvent {
    pub id: String,
    pub subject: String,
    pub occurred_at_epoch_ms: i64,
    pub impact_bps: BTreeMap<MaterialityDimension, i64>,
}

#[derive(Debug, Clone)]
pub struct MaterialityAssessment {
    pub material: bool,
    pub score_bps: i64,
    pub triggering_dimensions: Vec<MaterialityDimension>,
    pub escalate_by_epoch_ms: Option<i64>,
    pub policy_digest: String,
    pub authority_digest: String,
}

pub fn assess_materiality(event: &MaterialityEvent, policy: &MaterialityPolicy) -> Result<MaterialityAssessment, String> {
    validate_digest("POLICY", &policy.policy_digest)?;
    validate_digest("AUTHORITY", &policy.authority_digest)?;
    if policy.aggregate_threshold_bps < 0 || policy.escalation_within_ms < 0 {
        return Err("REFUSED:INVALID_MATERIALITY_POLICY".to_string());
    }
    let mut score_bps = 0i64;
    let mut triggering_dimensions: Vec<MaterialityDimension> = Vec::new();
    for (dimension, impact) in &event.impact_bps {
        let threshold = *policy.per_dimension_threshold_bps.get(dimension).unwrap_or(&0);
        if !(0..=10000).contains(impact) || !(0..=10000).contains(&threshold) {
            return Err("REFUSED:INVALID_MATERIALITY_IMPACT".to_string());
        }
        score_bps += impact;
        if impact >= &threshold {
            triggering_dimensions.push(*dimension);
        }
    }
    let count = event.impact_bps.len().max(1) as i64;
    score_bps = (score_bps / count).min(10000);
    let material = !triggering_dimensions.is_empty() || score_bps >= policy.aggregate_threshold_bps;
    triggering_dimensions.sort();
    Ok(MaterialityAssessment {
        material,
        score_bps,
        triggering_dimensions,
        escalate_by_epoch_ms: if material { Some(event.occurred_at_epoch_ms + policy.escalation_within_ms) } else { None },
        policy_digest: policy.policy_digest.clone(),
        authority_digest: policy.authority_digest.clone(),
    })
}

pub struct IcfrSubject {
    pub subject: String,
    pub processes: Vec<String>,
    pub material_accounts: Vec<String>,
    pub affects_financial_reporting: bool,
}

pub struct IcfrClassification {
    pub subject: String,
    pub in_scope: bool,
    pub reasons: Vec<String>,
}

#[must_use]
pub fn classify_icfr_subject(input: &IcfrSubject) -> IcfrClassification {
    let mut reasons: Vec<String> = Vec::new();
    if input.affects_financial_reporting {
        reasons.push("affects-financial-reporting".to_string());
    }
    if !input.material_accounts.is_empty() {
        reasons.push("material-account-impact".to_string());
    }
    let key_processes = ["revenue", "payments", "payroll", "purchasing", "inventory", "general-ledger", "financial-reporting"];
    if input.processes.iter().any(|p| key_processes.contains(&p.as_str())) {
        reasons.push("key-financial-process".to_string());
    }
    reasons.sort();
    IcfrClassification { subject: input.subject.clone(), in_scope: !reasons.is_empty(), reasons }
}

pub struct RoleAssignment {
    pub principal: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct SodViolation {
    pub principal: String,
    pub roles: (String, String),
}

#[must_use]
pub fn detect_segregation_of_duty_violations(assignments: &[RoleAssignment], incompatible_role_pairs: &[(String, String)]) -> Vec<SodViolation> {
    let mut by_principal: BTreeMap<String, std::collections::BTreeSet<String>> = BTreeMap::new();
    for assignment in assignments {
        by_principal.entry(assignment.principal.clone()).or_default().insert(assignment.role.clone());
    }
    let mut violations: Vec<SodViolation> = Vec::new();
    for (principal, roles) in &by_principal {
        for pair in incompatible_role_pairs {
            if roles.contains(&pair.0) && roles.contains(&pair.1) {
                violations.push(SodViolation { principal: principal.clone(), roles: pair.clone() });
            }
        }
    }
    violations.sort_by(|a, b| format!("{}|{}|{}", a.principal, a.roles.0, a.roles.1).cmp(&format!("{}|{}|{}", b.principal, b.roles.0, b.roles.1)));
    violations
}

pub struct BoardPackage {
    pub profile: &'static str, // "CASTLE_FORTUNE5_BOARD_V1"
    pub enterprise_subject: String,
    pub castle_subject: String,
    pub generated_at: String,
    pub enterprise_standing: Standing,
    pub castle_standing: Standing,
    pub material_refused_subjects: u32,
    pub risk_appetite_breaches: u32,
    pub control_count: usize,
    pub evidence_digest: String,
}

pub fn build_board_package(admission: &BoardAdmission, generated_at: &str, material_refused_subjects: u32, risk_appetite_breaches: u32) -> Result<BoardPackage, String> {
    if admission.standing != Standing::Alive {
        return Err("REFUSED:BOARD_PACKAGE_WITHOUT_BOARD_ADMISSION".to_string());
    }
    let evidence_digest = blake3_hex(&canonical_json(&json!({
        "enterprise": admission.enterprise.qualification.controls.iter().map(|c| c.control_id.clone()).collect::<Vec<_>>(),
        "castle": admission.castle.qualification.controls.iter().map(|c| c.control_id.clone()).collect::<Vec<_>>(),
    }))?);
    Ok(BoardPackage {
        profile: "CASTLE_FORTUNE5_BOARD_V1",
        enterprise_subject: admission.enterprise.qualification.subject.clone(),
        castle_subject: admission.castle.qualification.subject.clone(),
        generated_at: generated_at.to_string(),
        enterprise_standing: admission.enterprise.standing,
        castle_standing: admission.castle.standing,
        material_refused_subjects,
        risk_appetite_breaches,
        control_count: FORTUNE5_REQUIREMENTS.len(),
        evidence_digest,
    })
}

#[must_use]
pub fn board_requirements() -> &'static [Fortune5Requirement] {
    FORTUNE5_REQUIREMENTS
}

// ---------------------------------------------------------------------------
// Property tests: canonical_json digest-stability (additive, private-fn coverage)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod canonical_json_proptests {
    use super::canonical_json;
    use proptest::prelude::*;
    use serde_json::Value;

    const KEYS: [&str; 5] = ["alpha", "beta", "gamma", "delta", "epsilon"];

    fn key_strategy() -> impl Strategy<Value = String> {
        (0..KEYS.len()).prop_map(|i| KEYS[i].to_string())
    }

    fn pairs_strategy() -> impl Strategy<Value = Vec<(String, i64)>> {
        prop::collection::vec((key_strategy(), any::<i64>()), 0..=KEYS.len()).prop_map(|mut pairs| {
            let mut seen = std::collections::HashSet::new();
            pairs.retain(|(k, _)| seen.insert(k.clone()));
            pairs
        })
    }

    proptest! {
        /// `board.rs`'s independent `canonical_json` copy (see `CLAUDE.md`'s "two
        /// independent receipt systems" note — this one additionally rejects
        /// non-finite numbers, which `castle.rs`'s copy does not need to) must be
        /// just as key-order independent as `castle.rs`'s copy, since it feeds the
        /// same class of digest-stability requirement for the `ReceiptV2` chain.
        #[test]
        fn canonical_json_is_key_order_independent(pairs in pairs_strategy()) {
            let forward_json = {
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };
            let mut reversed = pairs.clone();
            reversed.reverse();
            let reversed_json = {
                let parts: Vec<String> = reversed.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };

            let forward_value: Value = serde_json::from_str(&forward_json).expect("forward JSON must parse");
            let reversed_value: Value = serde_json::from_str(&reversed_json).expect("reversed JSON must parse");

            prop_assert_eq!(canonical_json(&forward_value), canonical_json(&reversed_value));
        }

        /// Deterministic and, for finite-number inputs, always `Ok`.
        #[test]
        fn canonical_json_is_deterministic(pairs in pairs_strategy()) {
            let json_text = {
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k:?}:{v}")).collect();
                format!("{{{}}}", parts.join(","))
            };
            let value: Value = serde_json::from_str(&json_text).expect("JSON must parse");
            prop_assert_eq!(canonical_json(&value), canonical_json(&value));
            prop_assert!(canonical_json(&value).is_ok());
        }
    }
}
