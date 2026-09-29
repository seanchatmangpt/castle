//! Reference COMPLIANCE controls (court step 3).
//!
//! These are deterministic REFERENCE controls. They are NOT a claim of legal AML, KYC or
//! sanctions compliance: the list contents are supplied by the operator, matching is exact
//! on normalized strings (no fuzzy matching, no transliteration), and a `CLEAR` outcome means
//! only "no entry of the supplied list matched". An empty control set is refused
//! (default-deny: no controls != clear).

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::sa2a_security::encoding::{push_field, sha256_tagged};

use super::effect::PaymentEffect;
use super::refusal::{refuse, PayResult};

pub const SANCTIONS_HIT: &str = "REFUSED:PAYMENT_SANCTIONS_HIT";
pub const SANCTIONS_LIST_INVALID: &str = "REFUSED:SANCTIONS_LIST_INVALID";
pub const NO_COMPLIANCE_CONTROLS: &str = "REFUSED:NO_COMPLIANCE_CONTROLS";
pub const COMPLIANCE_CONTROL_DUPLICATE: &str = "REFUSED:COMPLIANCE_CONTROL_DUPLICATE";

const LIST_DOMAIN: &[u8] = b"CASTLE-SANCTIONS-LIST-V1";
const EVIDENCE_DOMAIN: &[u8] = b"CASTLE-CONTROL-EVIDENCE-V1";
const BUNDLE_DOMAIN: &[u8] = b"CASTLE-COMPLIANCE-BUNDLE-V1";

/// Trim, collapse internal whitespace runs to one space, uppercase.
#[must_use]
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEvidence {
    pub control_id: String,
    pub version: String,
    pub outcome: String,
    pub evidence_digest: String,
}

pub trait ComplianceControl: Send + Sync {
    fn id(&self) -> &str;
    /// `Ok` only when the effect is clear; any failure is a typed refusal.
    fn screen(&self, effect: &PaymentEffect) -> PayResult<ControlEvidence>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanctionsList {
    pub list_id: String,
    pub version: String,
    pub entries: BTreeSet<String>,
    pub digest: String,
    /// normalized account id -> normalized registered legal names
    aliases: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListJson {
    list_id: String,
    version: String,
    entries: Vec<String>,
}

fn list_digest(list_id: &str, version: &str, entries: &BTreeSet<String>) -> String {
    let mut body = Vec::new();
    push_field(&mut body, list_id.as_bytes());
    push_field(&mut body, version.as_bytes());
    for e in entries {
        push_field(&mut body, e.as_bytes());
    }
    sha256_tagged(LIST_DOMAIN, &body)
}

impl SanctionsList {
    pub fn from_json(bytes: &[u8]) -> PayResult<Self> {
        let parsed: ListJson = serde_json::from_slice(bytes).map_err(|_| SANCTIONS_LIST_INVALID.to_string())?;
        if parsed.list_id.trim().is_empty() || parsed.version.trim().is_empty() || parsed.entries.is_empty() {
            return refuse(SANCTIONS_LIST_INVALID);
        }
        let mut entries = BTreeSet::new();
        for raw in &parsed.entries {
            let n = normalize(raw);
            if n.is_empty() || !entries.insert(n) {
                return refuse(SANCTIONS_LIST_INVALID);
            }
        }
        let digest = list_digest(&parsed.list_id, &parsed.version, &entries);
        Ok(Self { list_id: parsed.list_id, version: parsed.version, entries, digest, aliases: BTreeMap::new() })
    }

    /// Register legal names per account id: `(account_id, legal_name)`. Names are screened
    /// whenever their account is payer or payee.
    #[must_use]
    pub fn with_aliases(mut self, aliases: &[(&str, &str)]) -> Self {
        for (account, name) in aliases {
            self.aliases.entry(normalize(account)).or_default().insert(normalize(name));
        }
        self
    }

    fn hit(&self, account: &str) -> bool {
        let key = normalize(account);
        self.entries.contains(&key)
            || self.aliases.get(&key).is_some_and(|names| names.iter().any(|n| self.entries.contains(n)))
    }

    fn alias_digest_body(&self, out: &mut Vec<u8>) {
        for (account, names) in &self.aliases {
            push_field(out, account.as_bytes());
            for n in names {
                push_field(out, n.as_bytes());
            }
        }
    }
}

impl ComplianceControl for SanctionsList {
    fn id(&self) -> &str {
        &self.list_id
    }

    fn screen(&self, effect: &PaymentEffect) -> PayResult<ControlEvidence> {
        if self.hit(effect.payer()) || self.hit(effect.payee()) {
            return refuse(SANCTIONS_HIT);
        }
        let mut body = Vec::new();
        push_field(&mut body, self.digest.as_bytes());
        push_field(&mut body, normalize(effect.payer()).as_bytes());
        push_field(&mut body, normalize(effect.payee()).as_bytes());
        self.alias_digest_body(&mut body);
        Ok(ControlEvidence {
            control_id: self.list_id.clone(),
            version: self.version.clone(),
            outcome: "CLEAR".to_string(),
            evidence_digest: sha256_tagged(EVIDENCE_DOMAIN, &body),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplianceBundle {
    pub evidence: Vec<ControlEvidence>,
    pub bundle_digest: String,
}

/// Run ALL controls in id order; the first failure refuses. Deterministic and independent of
/// the order in which the caller listed the controls.
pub fn run_controls(controls: &[&dyn ComplianceControl], effect: &PaymentEffect) -> PayResult<ComplianceBundle> {
    if controls.is_empty() {
        return refuse(NO_COMPLIANCE_CONTROLS);
    }
    let mut ordered: Vec<&dyn ComplianceControl> = controls.to_vec();
    ordered.sort_by(|a, b| a.id().cmp(b.id()));
    if ordered.windows(2).any(|w| w[0].id() == w[1].id()) {
        return refuse(COMPLIANCE_CONTROL_DUPLICATE);
    }
    let mut evidence = Vec::with_capacity(ordered.len());
    for control in ordered {
        evidence.push(control.screen(effect)?);
    }
    let mut body = Vec::new();
    for e in &evidence {
        push_field(&mut body, e.control_id.as_bytes());
        push_field(&mut body, e.version.as_bytes());
        push_field(&mut body, e.outcome.as_bytes());
        push_field(&mut body, e.evidence_digest.as_bytes());
    }
    Ok(ComplianceBundle { bundle_digest: sha256_tagged(BUNDLE_DOMAIN, &body), evidence })
}
