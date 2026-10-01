use serde_json::{json, Value};

/// Effect classes are ordered: a capability may only be driven up to its declared class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectClass {
    Observe,
    Select,
    Construct,
    Do,
}

impl EffectClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "OBSERVE",
            Self::Select => "SELECT",
            Self::Construct => "CONSTRUCT",
            Self::Do => "DO",
        }
    }
}

/// The universal product request (PRD section 8). Persists unchanged through execution.
#[derive(Debug, Clone)]
pub struct OperationEnvelope {
    pub operation_id: String,
    pub subject: String,
    pub intent: String,
    pub capability: String,
    pub context: Value,
    pub ontology_refs: Vec<String>,
    /// Source data in its external representation; never canonical.
    pub input: Value,
    pub actor: String,
    /// Authority relevant to the requested effect (audience of the actuator).
    pub authority: String,
    pub effect_class: EffectClass,
    pub standing_requirements: Vec<String>,
    pub prior_receipts: Vec<String>,
    pub correlation: String,
}

impl OperationEnvelope {
    pub fn to_json(&self) -> Value {
        json!({
            "operation_id": self.operation_id,
            "subject": self.subject,
            "intent": self.intent,
            "capability": self.capability,
            "context": self.context,
            "ontology_refs": self.ontology_refs,
            "input": self.input,
            "actor": self.actor,
            "authority": self.authority,
            "effect_class": self.effect_class.as_str(),
            "standing_requirements": self.standing_requirements,
            "prior_receipts": self.prior_receipts,
            "correlation": self.correlation,
        })
    }

    pub fn digest(&self) -> String {
        let canon = crate::castle::canonical_json(&self.to_json());
        format!("blake3:{}", blake3::hash(canon.as_bytes()).to_hex())
    }
}
