use serde::{Deserialize, Serialize};

use super::quorum::VerifiedSigner;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReceipt {
    pub effect_digest: String,
    pub principal: String,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u64,
    pub audience: String,
    pub verified_key_ids: Vec<String>,
    pub verified_custodian_ids: Vec<String>,
}

impl VerificationReceipt {
    #[must_use]
    pub fn from_signers(
        effect_digest: String,
        principal: String,
        policy_epoch: u64,
        revocation_epoch: u64,
        generation: u64,
        audience: String,
        signers: &[VerifiedSigner],
    ) -> Self {
        Self {
            effect_digest,
            principal,
            policy_epoch,
            revocation_epoch,
            generation,
            audience,
            verified_key_ids: signers.iter().map(|s| s.key_id.clone()).collect(),
            verified_custodian_ids: signers.iter().map(|s| s.custodian_id.clone()).collect(),
        }
    }
}
