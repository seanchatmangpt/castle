use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{SecurityRefusal, SignatureAlgorithm};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    Active,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRecord {
    pub key_id: String,
    pub custodian_id: String,
    pub algorithm: SignatureAlgorithm,
    pub public_key: Vec<u8>,
    pub state: KeyState,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub revocation_epoch: u64,
}

#[derive(Debug, Clone, Default)]
pub struct KeyRegistry {
    keys: BTreeMap<String, KeyRecord>,
}

impl KeyRegistry {
    #[must_use]
    pub fn from_records(records: impl IntoIterator<Item = KeyRecord>) -> Self {
        Self {
            keys: records.into_iter().map(|r| (r.key_id.clone(), r)).collect(),
        }
    }

    pub fn resolve(&self, key_id: &str, now_ms: u64, revocation_epoch: u64) -> Result<&KeyRecord, SecurityRefusal> {
        let record = self.keys.get(key_id).ok_or(SecurityRefusal::UnknownKey)?;
        if record.state == KeyState::Revoked || record.revocation_epoch > revocation_epoch {
            return Err(SecurityRefusal::KeyRevoked);
        }
        if now_ms < record.not_before_ms {
            return Err(SecurityRefusal::KeyNotYetValid);
        }
        if now_ms >= record.expires_at_ms {
            return Err(SecurityRefusal::KeyExpired);
        }
        if record.custodian_id.is_empty() || record.public_key.is_empty() {
            return Err(SecurityRefusal::InvalidKey);
        }
        Ok(record)
    }
}
