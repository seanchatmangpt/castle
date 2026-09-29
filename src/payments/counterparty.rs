//! Counterparty identity (court step 5): LEI (ISO 17442) validation and an account ->
//! counterparty registry. Reference identity check only: a syntactically valid LEI proves
//! the check digits, not that the entity exists in GLEIF.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::sa2a_security::encoding::{push_field, sha256_tagged};

use super::fibo::{CASTLE_NS, IRI_LEGAL_ENTITY_IDENTIFIER};
use super::refusal::{refuse, PayResult};

pub const LEI_INVALID: &str = "REFUSED:COUNTERPARTY_LEI_INVALID";
pub const COUNTERPARTY_UNVERIFIED: &str = "REFUSED:COUNTERPARTY_UNVERIFIED";
pub const ACCOUNT_AMBIGUOUS: &str = "REFUSED:COUNTERPARTY_ACCOUNT_AMBIGUOUS";
pub const REGISTRY_INVALID: &str = "REFUSED:COUNTERPARTY_REGISTRY_INVALID";

const EVIDENCE_DOMAIN: &[u8] = b"CASTLE-COUNTERPARTY-EVIDENCE-V1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lei(String);

/// Remainder mod 97 of the decimal expansion (A=10..Z=35) of `text`, by digit-wise chunking.
fn mod97(text: &str) -> Option<u32> {
    let mut r: u32 = 0;
    for c in text.chars() {
        let v = c.to_digit(36)?;
        if !c.is_ascii_digit() && !c.is_ascii_uppercase() {
            return None;
        }
        if v >= 10 {
            r = (r * 100 + v) % 97;
        } else {
            r = (r * 10 + v) % 97;
        }
    }
    Some(r)
}

impl Lei {
    pub fn parse(text: &str) -> PayResult<Self> {
        if text.len() != 20 || !text.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_uppercase()) {
            return refuse(LEI_INVALID);
        }
        // ISO 17442: characters 19-20 are numeric check digits.
        if !text.as_bytes()[18..].iter().all(u8::is_ascii_digit) || mod97(text) != Some(1) {
            return refuse(LEI_INVALID);
        }
        Ok(Self(text.to_string()))
    }

    /// Compute the two ISO 7064 MOD 97-10 check digits for an 18-char `[0-9A-Z]` prefix.
    pub fn with_check_digits(prefix18: &str) -> PayResult<Self> {
        if prefix18.len() != 18 || !prefix18.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_uppercase()) {
            return refuse(LEI_INVALID);
        }
        let r = mod97(&format!("{prefix18}00")).ok_or_else(|| LEI_INVALID.to_string())?;
        Self::parse(&format!("{prefix18}{:02}", 98 - r))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counterparty {
    pub party_id: String,
    pub legal_name: String,
    pub lei: Option<Lei>,
    pub accounts: BTreeSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartyJson {
    party_id: String,
    legal_name: String,
    #[serde(default)]
    lei: Option<String>,
    accounts: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryJson {
    #[serde(default)]
    require_lei: bool,
    parties: Vec<PartyJson>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterpartyRegistry {
    parties: Vec<Counterparty>,
    by_account: BTreeMap<String, usize>,
    require_lei: bool,
}

impl CounterpartyRegistry {
    pub fn from_json(bytes: &[u8]) -> PayResult<Self> {
        let parsed: RegistryJson = serde_json::from_slice(bytes).map_err(|_| REGISTRY_INVALID.to_string())?;
        let mut parties = Vec::new();
        let mut by_account = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for p in parsed.parties {
            if p.party_id.trim().is_empty() || p.legal_name.trim().is_empty() || p.accounts.is_empty() {
                return refuse(REGISTRY_INVALID);
            }
            if !ids.insert(p.party_id.clone()) {
                return refuse(REGISTRY_INVALID);
            }
            let lei = p.lei.as_deref().map(Lei::parse).transpose()?;
            let mut accounts = BTreeSet::new();
            for a in p.accounts {
                if a.is_empty() || !accounts.insert(a.clone()) {
                    return refuse(REGISTRY_INVALID);
                }
                if by_account.insert(a, parties.len()).is_some() {
                    return refuse(ACCOUNT_AMBIGUOUS);
                }
            }
            parties.push(Counterparty { party_id: p.party_id, legal_name: p.legal_name, lei, accounts });
        }
        Ok(Self { parties, by_account, require_lei: parsed.require_lei })
    }

    #[must_use]
    pub const fn with_require_lei(mut self, require: bool) -> Self {
        self.require_lei = require;
        self
    }

    pub fn resolve(&self, account: &str) -> PayResult<&Counterparty> {
        let party = self
            .by_account
            .get(account)
            .and_then(|i| self.parties.get(*i))
            .ok_or_else(|| COUNTERPARTY_UNVERIFIED.to_string())?;
        if self.require_lei && party.lei.is_none() {
            return refuse(COUNTERPARTY_UNVERIFIED);
        }
        Ok(party)
    }

    /// Deterministic digest binding both parties' identity facts and the LEI policy.
    #[must_use]
    pub fn evidence_digest(&self, payer: &Counterparty, payee: &Counterparty) -> String {
        let mut body = Vec::new();
        push_field(&mut body, if self.require_lei { b"require_lei" } else { b"lei_optional" });
        for (role, p) in [("payer", payer), ("payee", payee)] {
            push_field(&mut body, role.as_bytes());
            push_field(&mut body, p.party_id.as_bytes());
            push_field(&mut body, p.legal_name.as_bytes());
            push_field(&mut body, p.lei.as_ref().map_or("", Lei::as_str).as_bytes());
            for a in &p.accounts {
                push_field(&mut body, a.as_bytes());
            }
        }
        sha256_tagged(EVIDENCE_DOMAIN, &body)
    }
}

/// FIBO-typed JSON-LD view of a counterparty; the LEI is a literal typed as the FIBO
/// `LegalEntityIdentifier` class. Vocabulary only; grants nothing.
#[must_use]
pub fn fibo_party_json(cp: &Counterparty) -> Value {
    let mut node = json!({
        "@id": format!("urn:castle:party:{}", cp.party_id),
        "@type": format!("{CASTLE_NS}Counterparty"),
        "http://www.w3.org/2000/01/rdf-schema#label": cp.legal_name,
        format!("{CASTLE_NS}account"): cp.accounts.iter().collect::<Vec<_>>(),
    });
    if let (Some(lei), Some(obj)) = (&cp.lei, node.as_object_mut()) {
        obj.insert(
            format!("{CASTLE_NS}legalEntityIdentifier"),
            json!({ "@type": IRI_LEGAL_ENTITY_IDENTIFIER, "@value": lei.as_str() }),
        );
    }
    node
}
