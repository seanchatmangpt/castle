use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ReleaseStanding, RELEASE_VERSION};

fn digest_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|e| format!("REFUSED:SERIALIZATION_FAILED:{e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceOrigin {
    Cli,
    Api,
    Mcp,
    A2a,
    Human,
    Planner,
    Replay,
}

impl InterfaceOrigin {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Api => "api",
            Self::Mcp => "mcp",
            Self::A2a => "a2a",
            Self::Human => "human",
            Self::Planner => "planner",
            Self::Replay => "replay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentMode {
    Select,
    Construct,
    Do,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceIntent {
    pub request_id: String,
    pub origin: InterfaceOrigin,
    pub mode: IntentMode,
    pub subject: String,
    pub operation: String,
    pub payload: Value,
    pub construct_admission_digest: Option<String>,
    pub prepare_receipt_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceAdmission {
    pub standing: ReleaseStanding,
    pub request_id: String,
    pub reason: String,
    pub normalized_intent_digest: String,
}

/// CLI/API/MCP/A2A/human/planner/replay transports normalize here. SELECT and
/// CONSTRUCT are inert. A DO-shaped request receives no standing merely from
/// transport authentication: it must name both the admitted CONSTRUCT and the
/// BRCE prepare receipt. Actual provider execution still requires the opaque
/// in-process ConstructAdmission and cannot be reconstructed from these hashes.
#[must_use]
pub fn admit_interface_intent(intent: &InterfaceIntent) -> InterfaceAdmission {
    let digest = digest_serializable(intent).unwrap_or_else(|_| "0".repeat(64));
    let reason = if intent.request_id.is_empty() || intent.subject.is_empty() || intent.operation.is_empty() {
        Some("REFUSED:INCOMPLETE_INTENT")
    } else if intent.mode == IntentMode::Do
        && intent.construct_admission_digest.as_deref().map_or(true, |d| d.len() != 64)
    {
        Some("REFUSED:DO_WITHOUT_CONSTRUCT_ADMISSION")
    } else if intent.mode == IntentMode::Do
        && intent.prepare_receipt_digest.as_deref().map_or(true, |d| d.len() != 64)
    {
        Some("REFUSED:DO_WITHOUT_PREPARE_RECEIPT")
    } else {
        None
    };
    InterfaceAdmission {
        standing: if reason.is_some() { ReleaseStanding::Refused } else { ReleaseStanding::Alive },
        request_id: intent.request_id.clone(),
        reason: reason.unwrap_or("ALIVE:INTENT_ADMITTED").to_string(),
        normalized_intent_digest: digest,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationEnvelope {
    pub observation_id: String,
    pub source: String,
    pub subject: String,
    pub observed_at_epoch_ms: i64,
    pub epistemic_class: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmittedObservation {
    pub standing: ReleaseStanding,
    pub observation_id: String,
    pub subject: String,
    pub source: String,
    pub payload_digest: String,
    pub reason: String,
}

/// Raw telemetry is not O*. Source identity, subject, freshness and epistemic
/// class must be admitted before DfCM or a planner may consume it.
#[must_use]
pub fn admit_observation(
    observation: &ObservationEnvelope,
    allowed_sources: &BTreeSet<String>,
    now_epoch_ms: i64,
    max_age_ms: i64,
) -> AdmittedObservation {
    let payload_digest = digest_serializable(&observation.payload).unwrap_or_else(|_| "0".repeat(64));
    let reason = if observation.observation_id.is_empty() || observation.subject.is_empty() {
        "REFUSED:INCOMPLETE_OBSERVATION"
    } else if !allowed_sources.contains(&observation.source) {
        "REFUSED:UNADMITTED_OBSERVATION_SOURCE"
    } else if observation.epistemic_class != "OBSERVED" {
        "REFUSED:OBSERVATION_NOT_OBSERVED"
    } else if observation.observed_at_epoch_ms > now_epoch_ms {
        "REFUSED:OBSERVATION_FROM_FUTURE"
    } else if now_epoch_ms.saturating_sub(observation.observed_at_epoch_ms) > max_age_ms {
        "REFUSED:STALE_OBSERVATION"
    } else {
        "ALIVE:OBSERVATION_ADMITTED"
    };
    AdmittedObservation {
        standing: if reason.starts_with("ALIVE:") { ReleaseStanding::Alive } else { ReleaseStanding::Refused },
        observation_id: observation.observation_id.clone(),
        subject: observation.subject.clone(),
        source: observation.source.clone(),
        payload_digest,
        reason: reason.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    pub mode: IntentMode,
    pub consequential: bool,
}

#[must_use]
pub fn mcp_tool_catalog() -> Vec<McpToolDescriptor> {
    vec![
        McpToolDescriptor { name: "castle.select".to_string(), mode: IntentMode::Select, consequential: false },
        McpToolDescriptor { name: "castle.construct".to_string(), mode: IntentMode::Construct, consequential: false },
        McpToolDescriptor { name: "castle.do".to_string(), mode: IntentMode::Do, consequential: true },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct A2aAgentCard {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub default_authority: String,
}

#[must_use]
pub fn a2a_agent_card() -> A2aAgentCard {
    A2aAgentCard {
        name: "CASTLE".to_string(),
        version: RELEASE_VERSION.to_string(),
        capabilities: vec![
            "select".to_string(),
            "construct".to_string(),
            "admission".to_string(),
            "receipted-do".to_string(),
            "replay".to_string(),
        ],
        default_authority: "CONSTRUCT_ONLY".to_string(),
    }
}
