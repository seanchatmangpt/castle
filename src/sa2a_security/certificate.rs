use serde::{Deserialize, Serialize};

use super::encoding::{push_field, CERTIFICATE_DOMAIN};
use super::{CertificateSignature, SecurityRefusal};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActuationCertificate {
    pub version: u32,
    pub effect_digest: String,
    pub principal: String,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u64,
    pub nonce: String,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub audience: String,
    pub threshold: u16,
    pub signatures: Vec<CertificateSignature>,
}

impl ActuationCertificate {
    pub fn signing_message(&self) -> Result<Vec<u8>, SecurityRefusal> {
        if self.version != 1
            || self.effect_digest.is_empty()
            || self.principal.is_empty()
            || self.nonce.is_empty()
            || self.audience.is_empty()
            || self.threshold == 0
            || self.not_before_ms >= self.expires_at_ms
        {
            return Err(SecurityRefusal::InvalidSignature);
        }
        let mut out = Vec::new();
        out.extend_from_slice(CERTIFICATE_DOMAIN);
        push_field(&mut out, &self.version.to_be_bytes());
        push_field(&mut out, self.effect_digest.as_bytes());
        push_field(&mut out, self.principal.as_bytes());
        push_field(&mut out, &self.policy_epoch.to_be_bytes());
        push_field(&mut out, &self.revocation_epoch.to_be_bytes());
        push_field(&mut out, &self.generation.to_be_bytes());
        push_field(&mut out, self.nonce.as_bytes());
        push_field(&mut out, &self.not_before_ms.to_be_bytes());
        push_field(&mut out, &self.expires_at_ms.to_be_bytes());
        push_field(&mut out, self.audience.as_bytes());
        push_field(&mut out, &self.threshold.to_be_bytes());
        Ok(out)
    }
}
