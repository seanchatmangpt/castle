use std::any::Any;
use std::collections::BTreeMap;

use serde_json::Value;

use super::envelope::{EffectClass, OperationEnvelope};

/// PRD section 17: failures are typed; only some close an edge and fall over to another realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    RealizationFailed,
    CapabilityUnavailable,
    SemanticRefusal,
    AuthorityRefusal,
    ProcessInvalid,
    TransportFailed,
    EvidenceInsufficient,
    Unsupported,
}

impl FailureClass {
    pub const fn code(self) -> &'static str {
        match self {
            Self::RealizationFailed => "REALIZATION_FAILED",
            Self::CapabilityUnavailable => "CAPABILITY_UNAVAILABLE",
            Self::SemanticRefusal => "SEMANTIC_REFUSAL",
            Self::AuthorityRefusal => "AUTHORITY_REFUSAL",
            Self::ProcessInvalid => "PROCESS_INVALID",
            Self::TransportFailed => "TRANSPORT_FAILED",
            Self::EvidenceInsufficient => "EVIDENCE_INSUFFICIENT",
            Self::Unsupported => "UNSUPPORTED",
        }
    }

    /// A realization-level failure closes one edge; the resolver may try another realization.
    pub const fn falls_over(self) -> bool {
        matches!(self, Self::RealizationFailed | Self::TransportFailed | Self::Unsupported)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapError {
    pub class: FailureClass,
    pub detail: String,
}

impl CapError {
    pub fn new(class: FailureClass, detail: impl Into<String>) -> Self {
        Self { class, detail: detail.into() }
    }

    pub fn unsupported(method: &str) -> Self {
        Self::new(FailureClass::Unsupported, method)
    }

    /// Crate-wide refusal string convention: `REFUSED:<CLASS>:<detail>`.
    pub fn refusal(&self) -> String {
        format!("REFUSED:{}:{}", self.class.code(), self.detail)
    }
}

pub type CapResult<T> = Result<T, CapError>;

/// Semantic declaration of a capability (PRD section 7). A realization is implementation;
/// this is semantics. Two realizations of one URI must carry equal `semantic_id` and `invariants`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityContract {
    pub uri: String,
    pub semantic_id: String,
    pub realization: String,
    pub effect_class: EffectClass,
    pub accepted_subjects: Vec<String>,
    pub invariants: Vec<String>,
    pub refusal_conditions: Vec<String>,
    pub evidence_requirements: Vec<String>,
}

impl CapabilityContract {
    /// Realization identity is excluded: interchangeability is about semantics.
    pub fn qri_key(&self) -> (String, String, EffectClass, Vec<String>, Vec<String>) {
        let mut inv = self.invariants.clone();
        inv.sort();
        let mut ev = self.evidence_requirements.clone();
        ev.sort();
        (self.uri.clone(), self.semantic_id.clone(), self.effect_class, inv, ev)
    }
}

/// Runtime blackboard owned by the orchestrator. Capabilities exchange state only through it,
/// never with each other: `json` for inspectable facts, `artifacts` for in-process objects
/// (e.g. sealed admissions) that cannot be serialized.
#[derive(Default)]
pub struct Facts {
    pub json: BTreeMap<String, Value>,
    artifacts: BTreeMap<String, Box<dyn Any>>,
}

impl Facts {
    pub fn put_artifact<T: Any>(&mut self, key: &str, value: T) {
        self.artifacts.insert(key.to_string(), Box::new(value));
    }

    pub fn take_artifact<T: Any>(&mut self, key: &str) -> Option<T> {
        let boxed = self.artifacts.remove(key)?;
        match boxed.downcast::<T>() {
            Ok(v) => Some(*v),
            Err(other) => {
                self.artifacts.insert(key.to_string(), other);
                None
            }
        }
    }

    pub fn has_artifact(&self, key: &str) -> bool {
        self.artifacts.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.json.get(key)
    }
}

/// Common capability bus (PRD section 15). Unsupported methods refuse with `UNSUPPORTED`.
pub trait Capability {
    fn describe(&self) -> CapabilityContract;

    fn qualify(&self, _env: &OperationEnvelope) -> CapResult<()> {
        Ok(())
    }
    fn observe(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<()> {
        Err(CapError::unsupported("observe"))
    }
    fn select(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<()> {
        Err(CapError::unsupported("select"))
    }
    fn construct(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<()> {
        Err(CapError::unsupported("construct"))
    }
    fn execute(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<()> {
        Err(CapError::unsupported("execute"))
    }
    fn receipt(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<Value> {
        Err(CapError::unsupported("receipt"))
    }
    fn replay(&self, _env: &OperationEnvelope, _receipt: &Value) -> CapResult<()> {
        Err(CapError::unsupported("replay"))
    }
    fn health(&self) -> CapResult<()> {
        Ok(())
    }
}
