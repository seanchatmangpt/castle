use std::collections::BTreeMap;

use super::contract::{Capability, CapabilityContract, CapError, FailureClass};
use super::EffectClass;

/// Resolves `capability://` URIs to qualified realizations. No switch over repositories:
/// anything satisfying a contract may register.
#[derive(Default)]
pub struct Registry<'a> {
    by_uri: BTreeMap<String, Vec<Box<dyn Capability + 'a>>>,
}

impl<'a> Registry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// QRI at registration: a further realization of an already-registered URI must be
    /// semantically identical (same semantic id, effect class, invariants, evidence needs).
    pub fn register(&mut self, realization: Box<dyn Capability + 'a>) -> Result<(), String> {
        let contract = realization.describe();
        if !contract.uri.starts_with("capability://") {
            return Err("REFUSED:CONTRACT_URI_INVALID".into());
        }
        if let Some(existing) = self.by_uri.get(&contract.uri).and_then(|v| v.first()) {
            if existing.describe().qri_key() != contract.qri_key() {
                return Err("REFUSED:QRI_CONTRACT_MISMATCH".into());
            }
        }
        self.by_uri.entry(contract.uri.clone()).or_default().push(realization);
        Ok(())
    }

    pub fn contracts(&self, uri: &str) -> Vec<CapabilityContract> {
        self.by_uri.get(uri).map(|v| v.iter().map(|c| c.describe()).collect()).unwrap_or_default()
    }

    /// Healthy realizations in registration order.
    pub fn resolve(&self, uri: &str) -> Vec<&(dyn Capability + 'a)> {
        self.by_uri
            .get(uri)
            .map(|v| v.iter().filter(|c| c.health().is_ok()).map(|b| b.as_ref()).collect())
            .unwrap_or_default()
    }

    /// A capability may only be driven up to its declared effect class.
    pub fn permit(contract: &CapabilityContract, op: EffectClass) -> Result<(), CapError> {
        if op > contract.effect_class {
            return Err(CapError::new(
                FailureClass::AuthorityRefusal,
                format!("{}_EXCEEDS_{}", op.as_str(), contract.effect_class.as_str()),
            ));
        }
        Ok(())
    }
}
