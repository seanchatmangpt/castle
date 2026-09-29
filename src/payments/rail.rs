//! Rail boundary: the only port through which a payment instruction leaves the kernel.
//! A rail may accept then lose its response; `submit` is therefore idempotent by
//! `correlation_id` and outcomes are observed through `status`, never assumed.

use serde::{Deserialize, Serialize};

use super::money::Currency;
use crate::sa2a_security::encoding::sha256_tagged;

/// Domain tag for payload digests.
pub const PAYLOAD_DOMAIN: &[u8] = b"CASTLE-RAIL-PAYLOAD-V1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RailInstruction {
    pub effect_id: String,
    pub correlation_id: String,
    pub message_profile: String,
    /// pain.001 XML.
    pub payload: String,
    pub amount_minor: u64,
    pub currency: Currency,
    pub payer: String,
    pub payee: String,
}

impl RailInstruction {
    /// Deterministic end-to-end id: "E2E-" + first 28 hex chars of the effect digest.
    #[must_use]
    pub fn correlation_id_for(effect_id: &str) -> String {
        let hex = effect_id.strip_prefix("sha256:").unwrap_or(effect_id);
        let head: String = hex.chars().take(28).collect();
        format!("E2E-{head}")
    }

    #[must_use]
    pub fn payload_digest(&self) -> String {
        sha256_tagged(PAYLOAD_DOMAIN, self.payload.as_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RailAck {
    Accepted { correlation_id: String },
    Rejected { correlation_id: String, reason_code: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RailError {
    Timeout,
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RailStatus {
    Unknown,
    Accepted,
    Settled { final_ref: String },
    Rejected { reason: String },
    Returned { reason: String, return_ref: String },
}

pub trait RailActuator: Send + Sync {
    fn submit(&self, i: &RailInstruction) -> Result<RailAck, RailError>;
    fn status(&self, correlation_id: &str) -> Result<RailStatus, RailError>;
    /// True only when this rail's durable store is THE authority for what it has seen, so
    /// `RailStatus::Unknown` is proof the effect was never accepted. Default false: an
    /// unknown answer from an amnesiac or proxying rail is not proof of absence.
    fn absence_is_authoritative(&self) -> bool {
        false
    }
}
