use serde::{Deserialize, Serialize};

use crate::sa2a_security::PreparedEffect;

use super::money::{Currency, Money};
use super::refusal::{self, refuse, PayResult};

pub const PAYMENT_CAPABILITY: &str = "payments.transfer.v1";
const SUBJECT_KIND: &str = "account_transfer";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Subject {
    kind: String,
    payer: String,
    payee: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    amount: String,
    currency: Currency,
    obligation_id: String,
    purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reverses: Option<String>,
}

/// A strictly parsed payment view of an SA2A `PreparedEffect`. Holding one
/// confers no authority; it is an inert candidate until admitted.
#[derive(Debug, Clone, PartialEq)]
pub struct PaymentEffect {
    prepared: PreparedEffect,
    digest: String,
    payer: String,
    payee: String,
    money: Money,
    obligation_id: String,
    purpose: String,
    reverses: Option<String>,
}

impl PaymentEffect {
    /// Build the portable effect for a transfer. Pure; grants nothing.
    pub fn prepare(
        principal: &str,
        payer: &str,
        payee: &str,
        amount_minor: &str,
        currency: Currency,
        obligation_id: &str,
        purpose: &str,
        reverses: Option<&str>,
    ) -> PayResult<PreparedEffect> {
        let subject = serde_json::to_value(Subject {
            kind: SUBJECT_KIND.into(),
            payer: payer.into(),
            payee: payee.into(),
        })
        .map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
        let payload = serde_json::to_value(Payload {
            amount: amount_minor.into(),
            currency,
            obligation_id: obligation_id.into(),
            purpose: purpose.into(),
            reverses: reverses.map(str::to_string),
        })
        .map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
        Ok(PreparedEffect {
            version: 1,
            principal: principal.into(),
            capability: PAYMENT_CAPABILITY.into(),
            subject,
            payload,
        })
    }

    pub fn from_prepared(prepared: PreparedEffect) -> PayResult<Self> {
        if prepared.capability != PAYMENT_CAPABILITY {
            return refuse(refusal::CAPABILITY_NOT_ADMITTED);
        }
        let subject: Subject = serde_json::from_value(prepared.subject.clone())
            .map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
        let payload: Payload = serde_json::from_value(prepared.payload.clone())
            .map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
        if subject.kind != SUBJECT_KIND
            || subject.payer.is_empty()
            || subject.payee.is_empty()
            || payload.obligation_id.is_empty()
            || payload.purpose.is_empty()
        {
            return refuse(refusal::PAYLOAD_INVALID);
        }
        if subject.payer == subject.payee {
            return refuse(refusal::SAME_ACCOUNT);
        }
        let money = Money::parse_minor(&payload.amount, payload.currency)?;
        let digest = prepared.digest().map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
        Ok(Self {
            prepared,
            digest,
            payer: subject.payer,
            payee: subject.payee,
            money,
            obligation_id: payload.obligation_id,
            purpose: payload.purpose,
            reverses: payload.reverses,
        })
    }

    #[must_use]
    pub const fn prepared(&self) -> &PreparedEffect {
        &self.prepared
    }
    /// SA2A effect digest; also the economic idempotency key.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    #[must_use]
    pub fn principal(&self) -> &str {
        &self.prepared.principal
    }
    #[must_use]
    pub fn payer(&self) -> &str {
        &self.payer
    }
    #[must_use]
    pub fn payee(&self) -> &str {
        &self.payee
    }
    #[must_use]
    pub const fn money(&self) -> Money {
        self.money
    }
    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }
    #[must_use]
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    #[must_use]
    pub fn reverses(&self) -> Option<&str> {
        self.reverses.as_deref()
    }
}
