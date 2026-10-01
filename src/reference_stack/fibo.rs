use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::sa2a_security::{PreparedEffect, VerificationReceipt};

pub const FIBO_PAYMENT_CAPABILITY: &str = "fibo:PaymentExecution";
pub const GRAPHLAW_HOOK_CONTRACT: &str = "https://graphlaw.dev/knowledge-hook#HookCandidate";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeHookCandidate {
    pub version: u32,
    pub exact_subject: Value,
    pub source_digest: String,
    pub replay_identity: String,
    pub authority: String,
    pub capability: String,
    pub principal: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FiboPaymentProjection {
    pub hook_contract: &'static str,
    pub source_digest: String,
    pub replay_identity: String,
    pub prepared_effect: PreparedEffect,
    pub effect_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityEvidence {
    pub effect_digest: String,
    pub principal: String,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u64,
    pub audience: String,
    pub signer_count: usize,
    pub custodian_count: usize,
}

fn valid_sha256_tag(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn project_fibo_payment(candidate: KnowledgeHookCandidate) -> Result<FiboPaymentProjection, String> {
    if candidate.version != 1 {
        return Err("REFUSED:KNOWLEDGE_HOOK_VERSION".to_string());
    }
    if candidate.authority != "NONE" {
        return Err("REFUSED:KNOWLEDGE_HOOK_AUTHORITY".to_string());
    }
    if candidate.capability != FIBO_PAYMENT_CAPABILITY {
        return Err("REFUSED:FIBO_CAPABILITY_MISMATCH".to_string());
    }
    if candidate.principal.is_empty() || candidate.replay_identity.is_empty() {
        return Err("REFUSED:IDENTITY_REQUIRED".to_string());
    }
    if candidate.exact_subject.is_null() {
        return Err("REFUSED:EXACT_SUBJECT_REQUIRED".to_string());
    }
    if !valid_sha256_tag(&candidate.source_digest) {
        return Err("REFUSED:SOURCE_DIGEST_REQUIRED".to_string());
    }

    let source_digest = candidate.source_digest.clone();
    let replay_identity = candidate.replay_identity.clone();
    let prepared_effect = PreparedEffect {
        version: 1,
        principal: candidate.principal,
        capability: candidate.capability,
        subject: candidate.exact_subject,
        payload: json!({
            "graphlaw_hook_contract": GRAPHLAW_HOOK_CONTRACT,
            "semantic_source_digest": source_digest.clone(),
            "replay_identity": replay_identity.clone(),
            "input": candidate.payload,
        }),
    };
    let effect_digest = prepared_effect
        .digest()
        .map_err(|_| "REFUSED:PREPARED_EFFECT_DIGEST".to_string())?;

    Ok(FiboPaymentProjection {
        hook_contract: GRAPHLAW_HOOK_CONTRACT,
        source_digest,
        replay_identity,
        prepared_effect,
        effect_digest,
    })
}

pub fn bind_verification_receipt(
    projection: &FiboPaymentProjection,
    receipt: &VerificationReceipt,
) -> Result<AuthorityEvidence, String> {
    if receipt.effect_digest != projection.effect_digest {
        return Err("REFUSED:AUTHORITY_EFFECT_DIGEST_MISMATCH".to_string());
    }
    if receipt.principal != projection.prepared_effect.principal {
        return Err("REFUSED:AUTHORITY_PRINCIPAL_MISMATCH".to_string());
    }
    if receipt.audience.is_empty()
        || receipt.verified_key_ids.is_empty()
        || receipt.verified_custodian_ids.is_empty()
    {
        return Err("REFUSED:INDEPENDENT_AUTHORITY_EVIDENCE_REQUIRED".to_string());
    }

    Ok(AuthorityEvidence {
        effect_digest: receipt.effect_digest.clone(),
        principal: receipt.principal.clone(),
        policy_epoch: receipt.policy_epoch,
        revocation_epoch: receipt.revocation_epoch,
        generation: receipt.generation,
        audience: receipt.audience.clone(),
        signer_count: receipt.verified_key_ids.len(),
        custodian_count: receipt.verified_custodian_ids.len(),
    })
}
