use serde::{Deserialize, Serialize};

use super::encoding::{push_field, sha256_tagged, valid_sha256_tag, PREPARED_EFFECT_DOMAIN};
use super::SecurityRefusal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedEffect {
    pub version: u32,
    pub principal: String,
    pub effect_kind: String,
    pub resource: String,
    pub operation: String,
    pub payload_digest: String,
    pub idempotency_key: String,
}

impl PreparedEffect {
    pub fn digest(&self) -> Result<String, SecurityRefusal> {
        if self.version != 1
            || self.principal.is_empty()
            || self.effect_kind.is_empty()
            || self.resource.is_empty()
            || self.operation.is_empty()
            || self.idempotency_key.is_empty()
            || !valid_sha256_tag(&self.payload_digest)
        {
            return Err(SecurityRefusal::InvalidDigest);
        }
        let mut body = Vec::new();
        push_field(&mut body, &self.version.to_be_bytes());
        push_field(&mut body, self.principal.as_bytes());
        push_field(&mut body, self.effect_kind.as_bytes());
        push_field(&mut body, self.resource.as_bytes());
        push_field(&mut body, self.operation.as_bytes());
        push_field(&mut body, self.payload_digest.as_bytes());
        push_field(&mut body, self.idempotency_key.as_bytes());
        Ok(sha256_tagged(PREPARED_EFFECT_DOMAIN, &body))
    }
}
