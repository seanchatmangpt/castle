use std::collections::BTreeSet;

use super::SecurityRefusal;

#[derive(Debug, Clone, Default)]
pub struct NonceFence {
    seen: BTreeSet<(String, String)>,
}

impl NonceFence {
    pub fn claim(&mut self, key_id: &str, nonce: &str) -> Result<(), SecurityRefusal> {
        if key_id.is_empty() || nonce.is_empty() {
            return Err(SecurityRefusal::NonceReplay);
        }
        if !self.seen.insert((key_id.to_string(), nonce.to_string())) {
            return Err(SecurityRefusal::NonceReplay);
        }
        Ok(())
    }
}
