use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::contract::{CapError, CapResult, Capability, Facts, FailureClass};
use super::envelope::{EffectClass, OperationEnvelope};
use super::registry::Registry;
use super::{URI_ACTUATION, URI_EVIDENCE, URI_LAW, URI_PROCESS, URI_PROJECTION};

/// Distinct, separately reported states (PRD section 24). None implies the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Requested,
    Qualified,
    Constructed,
    Executed,
    Observed,
    Receipted,
    Reconciled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Receipt present and verified, process conforms.
    Alive,
    /// Executed or partly proven, but standing cannot be asserted.
    Unknown,
    Refused,
}

pub struct Outcome {
    pub stages: Vec<Stage>,
    pub standing: Standing,
    pub refusal: Option<String>,
    /// Number of times any DO-class capability was executed. Must be 0 or 1.
    pub do_crossings: u32,
    pub receipt: Option<Value>,
    pub conformance: Option<Value>,
    /// uri -> realization that actually served it.
    pub served_by: BTreeMap<String, String>,
    pub facts: Facts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayVerdict {
    pub same_decision: bool,
    pub receipt_valid: bool,
    pub detail: String,
}

/// Drives the BRCE-shaped sequence over a registry. It names URIs only, never realizations.
pub struct Orchestrator<'r, 'a> {
    registry: &'r Registry<'a>,
}

impl<'r, 'a> Orchestrator<'r, 'a> {
    pub fn new(registry: &'r Registry<'a>) -> Self {
        Self { registry }
    }

    /// Call `f` on the first realization that serves; a falls-over failure closes that edge and
    /// the next qualified realization is tried. DO never falls over (no double actuation).
    fn drive<T>(
        &self,
        uri: &str,
        op: EffectClass,
        served_by: &mut BTreeMap<String, String>,
        mut f: impl FnMut(&dyn Capability) -> CapResult<T>,
    ) -> CapResult<T> {
        let candidates = self.registry.resolve(uri);
        if candidates.is_empty() {
            return Err(CapError::new(FailureClass::CapabilityUnavailable, uri));
        }
        let mut last = CapError::new(FailureClass::CapabilityUnavailable, uri);
        for cap in candidates {
            let contract = cap.describe();
            Registry::permit(&contract, op)?;
            match f(cap) {
                Ok(v) => {
                    served_by.insert(uri.to_string(), contract.realization);
                    return Ok(v);
                }
                Err(e) if e.class.falls_over() && op != EffectClass::Do => last = e,
                Err(e) => return Err(e),
            }
        }
        Err(last)
    }

    pub fn run(&self, env: &OperationEnvelope) -> Outcome {
        let mut out = Outcome {
            stages: vec![Stage::Requested],
            standing: Standing::Unknown,
            refusal: None,
            do_crossings: 0,
            receipt: None,
            conformance: None,
            served_by: BTreeMap::new(),
            facts: Facts::default(),
        };
        out.facts.json.insert("envelope.digest".into(), json!(env.digest()));
        if let Err(e) = self.pipeline(env, &mut out) {
            out.refusal = Some(e.refusal());
            if matches!(e.class, FailureClass::SemanticRefusal | FailureClass::AuthorityRefusal | FailureClass::ProcessInvalid) {
                out.standing = Standing::Refused;
            }
        }
        out
    }

    fn event(&self, env: &OperationEnvelope, out: &mut Outcome, kind: &str) -> CapResult<()> {
        out.facts.json.insert("process.event".into(), json!(kind));
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_PROCESS, EffectClass::Observe, &mut out.served_by, |c| c.observe(env, &mut facts));
        out.facts = facts;
        r
    }

    fn pipeline(&self, env: &OperationEnvelope, out: &mut Outcome) -> CapResult<()> {
        // Every required plane must resolve before anything is attempted.
        for uri in [URI_PROJECTION, URI_PROCESS, URI_LAW, URI_EVIDENCE, URI_ACTUATION] {
            if self.registry.resolve(uri).is_empty() {
                return Err(CapError::new(FailureClass::CapabilityUnavailable, uri));
            }
        }

        // OBSERVE: project external data, establish process position.
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_PROJECTION, EffectClass::Observe, &mut out.served_by, |c| c.observe(env, &mut facts));
        out.facts = facts;
        r?;
        self.event(env, out, "requested")?;

        // Qualify + CONSTRUCT (law). A refusal here means no action object exists.
        self.drive(URI_LAW, EffectClass::Select, &mut out.served_by, |c| c.qualify(env))?;
        out.stages.push(Stage::Qualified);
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_LAW, EffectClass::Construct, &mut out.served_by, |c| c.construct(env, &mut facts));
        out.facts = facts;
        if let Err(e) = r {
            self.event(env, out, "refused")?;
            return Err(e);
        }
        out.stages.push(Stage::Constructed);
        self.event(env, out, "constructed")?;

        // Pre-actuation evidence binding, then actor selection.
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_EVIDENCE, EffectClass::Observe, &mut out.served_by, |c| c.observe(env, &mut facts));
        out.facts = facts;
        r?;
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_ACTUATION, EffectClass::Select, &mut out.served_by, |c| c.select(env, &mut facts));
        out.facts = facts;
        r?;

        // The one DO boundary.
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_ACTUATION, EffectClass::Do, &mut out.served_by, |c| c.execute(env, &mut facts));
        out.facts = facts;
        out.do_crossings += 1;
        r?;
        out.stages.push(Stage::Executed);

        // Independent observation of the effect.
        let mut facts = std::mem::take(&mut out.facts);
        let r = self.drive(URI_ACTUATION, EffectClass::Observe, &mut out.served_by, |c| c.observe(env, &mut facts));
        out.facts = facts;
        r?;
        out.stages.push(Stage::Observed);
        self.event(env, out, "executed")?;

        // Receipt: absence leaves standing Unknown (execution != standing).
        let mut facts = std::mem::take(&mut out.facts);
        let receipt = self.drive(URI_EVIDENCE, EffectClass::Observe, &mut out.served_by, |c| c.receipt(env, &mut facts));
        out.facts = facts;
        let receipt = receipt?;
        out.stages.push(Stage::Receipted);
        self.event(env, out, "receipted")?;

        let mut facts = std::mem::take(&mut out.facts);
        let conformance = self.drive(URI_PROCESS, EffectClass::Observe, &mut out.served_by, |c| c.receipt(env, &mut facts));
        out.facts = facts;
        let conformance = conformance?;
        let conforms = conformance.get("conforms").and_then(Value::as_bool) == Some(true);
        out.conformance = Some(conformance);
        if conforms {
            out.stages.push(Stage::Reconciled);
        }

        let mut served = BTreeMap::new();
        let valid = self.drive(URI_EVIDENCE, EffectClass::Observe, &mut served, |c| c.replay(env, &receipt)).is_ok();
        out.receipt = Some(receipt);
        if valid && conforms {
            out.standing = Standing::Alive;
        }
        Ok(())
    }

    /// Semantic replay: reconstruct the decision and re-verify the evidence from the receipt.
    /// Never re-actuates.
    pub fn replay(&self, env: &OperationEnvelope, receipt: &Value) -> ReplayVerdict {
        let mut served = BTreeMap::new();
        let law = self.drive(URI_LAW, EffectClass::Observe, &mut served, |c| c.replay(env, receipt));
        let ev = self.drive(URI_EVIDENCE, EffectClass::Observe, &mut served, |c| c.replay(env, receipt));
        ReplayVerdict {
            same_decision: law.is_ok(),
            receipt_valid: ev.is_ok(),
            detail: format!(
                "law={} evidence={}",
                law.err().map_or("ok".into(), |e| e.refusal()),
                ev.err().map_or("ok".into(), |e| e.refusal())
            ),
        }
    }
}
