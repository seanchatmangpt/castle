//! Machine experience for payments (falsifier F8): a KNOWN recurring payment
//! class executes with zero intelligence involvement. Pure deterministic
//! lookup over durable, receipt-derived rules; learning is monotone and only
//! ever driven by a `Settled` execution carrying real receipt digests.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::sa2a_security::encoding::{push_field, sha256_tagged};

use super::effect::PaymentEffect;
use super::execute::{PaymentExecution, PaymentStanding};
use super::money::Currency;
use super::refusal::PayResult;

const CLASS_DOMAIN: &[u8] = b"CASTLE-PAYMENT-CLASS-V1";
const REQUIRES_SETTLED: &str = "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT";
const STORE_FAILED: &str = "BLOCKED:EXPERIENCE_STORE_FAILED";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PaymentClass {
    principal: String,
    payer: String,
    payee: String,
    currency: Currency,
    purpose: String,
}

impl PaymentClass {
    #[must_use]
    pub fn of(effect: &PaymentEffect) -> Self {
        Self {
            principal: effect.principal().to_string(),
            payer: effect.payer().to_string(),
            payee: effect.payee().to_string(),
            currency: effect.money().currency,
            purpose: effect.purpose().to_string(),
        }
    }

    #[must_use]
    pub fn key(&self) -> String {
        let mut body = Vec::new();
        push_field(&mut body, self.principal.as_bytes());
        push_field(&mut body, self.payer.as_bytes());
        push_field(&mut body, self.payee.as_bytes());
        push_field(&mut body, format!("{:?}", self.currency).as_bytes());
        push_field(&mut body, self.purpose.as_bytes());
        sha256_tagged(CLASS_DOMAIN, &body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassRule {
    pub class_key: String,
    pub per_effect_cap_minor: u64,
    pub settled_count: u64,
    pub last_construct_digest: String,
    pub last_ocel_receipt_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Knowledge {
    Known(ClassRule),
    Unknown,
}

/// True only for `Unknown`: a Known class never needs intelligence.
#[must_use]
pub const fn requires_intelligence(k: &Knowledge) -> bool {
    matches!(k, Knowledge::Unknown)
}

pub struct ExperienceStore {
    root: PathBuf,
}

impl ExperienceStore {
    pub fn open(root: impl Into<PathBuf>) -> PayResult<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|_| STORE_FAILED.to_string())?;
        Ok(Self { root })
    }

    fn path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.json"))
    }

    fn load(&self, key: &str) -> PayResult<Option<ClassRule>> {
        match fs::read(self.path(key)) {
            Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|_| STORE_FAILED.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(STORE_FAILED.to_string()),
        }
    }

    fn persist(&self, rule: &ClassRule) -> PayResult<()> {
        let bytes = serde_json::to_vec(rule).map_err(|_| STORE_FAILED.to_string())?;
        let tmp = self.root.join(format!("{}.tmp", rule.class_key));
        let fin = self.path(&rule.class_key);
        let mut f = fs::File::create(&tmp).map_err(|_| STORE_FAILED.to_string())?;
        f.write_all(&bytes).map_err(|_| STORE_FAILED.to_string())?;
        f.sync_all().map_err(|_| STORE_FAILED.to_string())?;
        drop(f);
        fs::rename(&tmp, &fin).map_err(|_| STORE_FAILED.to_string())?;
        if let Ok(dir) = fs::File::open(&self.root) {
            let _ = dir.sync_all();
        }
        Ok(())
    }

    /// Learn from a settled execution only. Monotone: count increments, cap
    /// is the maximum amount ever settled in the class.
    pub fn record_settled(&self, effect: &PaymentEffect, exec: &PaymentExecution) -> PayResult<()> {
        let (Some(construct), Some(ocel)) = (&exec.construct_digest, &exec.ocel_receipt_digest) else {
            return Err(REQUIRES_SETTLED.to_string());
        };
        if exec.standing != PaymentStanding::Settled
            || construct.is_empty()
            || ocel.is_empty()
            || exec.effect_digest != effect.digest()
        {
            return Err(REQUIRES_SETTLED.to_string());
        }
        let key = PaymentClass::of(effect).key();
        let amount = effect.money().minor;
        let rule = match self.load(&key)? {
            Some(mut r) => {
                r.per_effect_cap_minor = r.per_effect_cap_minor.max(amount);
                r.settled_count = r.settled_count.saturating_add(1);
                r.last_construct_digest = construct.clone();
                r.last_ocel_receipt_digest = ocel.clone();
                r
            }
            None => ClassRule {
                class_key: key,
                per_effect_cap_minor: amount,
                settled_count: 1,
                last_construct_digest: construct.clone(),
                last_ocel_receipt_digest: ocel.clone(),
            },
        };
        self.persist(&rule)
    }

    pub fn classify(&self, effect: &PaymentEffect) -> PayResult<Knowledge> {
        let key = PaymentClass::of(effect).key();
        Ok(match self.load(&key)? {
            Some(rule) if effect.money().minor <= rule.per_effect_cap_minor => Knowledge::Known(rule),
            _ => Knowledge::Unknown,
        })
    }
}
