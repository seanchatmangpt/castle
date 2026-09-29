use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::effect::PaymentEffect;
use super::money::Currency;
use super::refusal::{self, refuse, PayResult};

/// Per-principal spend limits. Default-deny: absent principal, account, or
/// currency cap means refusal.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrincipalPolicy {
    pub allowed_payers: BTreeSet<String>,
    pub allowed_payees: BTreeSet<String>,
    pub per_effect_cap: BTreeMap<Currency, u64>,
    pub epoch_cap: BTreeMap<Currency, u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendPolicy {
    pub principals: BTreeMap<String, PrincipalPolicy>,
    /// When set, obligation ids must be derived from (payer, payee, purpose,
    /// invoice_ref) via `obligation::derive_obligation_id`. Default false = v1.
    #[serde(default)]
    pub require_derived_obligation: bool,
}

impl SpendPolicy {
    /// Pure static checks. Budget (epoch cap) is enforced atomically at claim time.
    pub fn check_static(&self, effect: &PaymentEffect) -> PayResult<&PrincipalPolicy> {
        let Some(p) = self.principals.get(effect.principal()) else {
            return refuse(refusal::NO_SPEND_POLICY);
        };
        if !p.allowed_payers.contains(effect.payer()) || !p.allowed_payees.contains(effect.payee()) {
            return refuse(refusal::ACCOUNT_NOT_ALLOWED);
        }
        let currency = effect.money().currency;
        let Some(cap) = p.per_effect_cap.get(&currency) else {
            return refuse(refusal::CURRENCY_UNSUPPORTED);
        };
        if effect.money().minor > *cap {
            return refuse(refusal::AMOUNT_EXCEEDS_CAP);
        }
        if !p.epoch_cap.contains_key(&currency) {
            return refuse(refusal::CURRENCY_UNSUPPORTED);
        }
        Ok(p)
    }

    /// Deterministic JSON identity for binding into the CONSTRUCT config graph.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}
