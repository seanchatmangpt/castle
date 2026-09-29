use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::encoding::sha256_tagged;
use super::SecurityRefusal;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedEffect {
    pub version: u32,
    pub principal: String,
    pub capability: String,
    pub subject: Value,
    pub payload: Value,
}

impl PreparedEffect {
    /// Exact cross-runtime identity shared with AshA2A:
    /// SHA-256(RFC8785/JCS({version,principal,capability,subject,payload})).
    pub fn digest(&self) -> Result<String, SecurityRefusal> {
        if self.version != 1 || self.principal.is_empty() || self.capability.is_empty() {
            return Err(SecurityRefusal::InvalidDigest);
        }

        let bytes =
            serde_json_canonicalizer::to_vec(self).map_err(|_| SecurityRefusal::InvalidDigest)?;
        Ok(sha256_tagged(b"", &bytes))
    }
}
