//! Real realizations of the five fabric planes over the real payments machinery. Each is a
//! hand-written implementation of `Capability` (not a mock): the law plane really admits, the
//! actuator really settles against the real ledger, the evidence plane really signs.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use serde_json::{json, Value};

use castle::castle::{create_receipt, verify_receipt, EpistemicClass, OcelEvent, OcelLog, Receipt};
use castle::fabric::*;
use castle::payments::replay::{replay_admission, AdmissionJournal, ReplayVerdict as PayReplay};
use castle::payments::*;

use crate::common::payments::*;

fn contract(
    uri: &str,
    semantic_id: &str,
    realization: &str,
    class: EffectClass,
    invariants: &[&str],
    evidence: &[&str],
) -> CapabilityContract {
    CapabilityContract {
        uri: uri.into(),
        semantic_id: semantic_id.into(),
        realization: realization.into(),
        effect_class: class,
        accepted_subjects: vec!["payment".into()],
        invariants: invariants.iter().map(|s| (*s).into()).collect(),
        refusal_conditions: vec!["REFUSED".into()],
        evidence_requirements: evidence.iter().map(|s| (*s).into()).collect(),
    }
}

pub fn law_contract(realization: &str) -> CapabilityContract {
    contract(
        URI_LAW,
        "law.admit.payment.v1",
        realization,
        EffectClass::Construct,
        &["INVALID_IMPLIES_NONCONSTRUCTIBLE", "REPLAYABLE_DECISION"],
        &["law.decision"],
    )
}

// ---------------------------------------------------------------- projection

/// Relational row -> graph triples with per-triple provenance. Carries no payment law.
pub struct RowProjection;
const COLUMNS: &[(&str, &str)] = &[
    ("principal", "pay:principal"),
    ("payer", "pay:payer"),
    ("payee", "pay:payee"),
    ("amount_minor", "pay:amountMinor"),
    ("obligation_id", "pay:dischargesObligation"),
];
impl Capability for RowProjection {
    fn describe(&self) -> CapabilityContract {
        contract(
            URI_PROJECTION,
            "projection.map.relational.v1",
            "row-projection",
            EffectClass::Observe,
            &["SOURCE_REPRESENTATION_NOT_CANONICAL", "PROVENANCE_PER_ASSERTION"],
            &["projection.provenance"],
        )
    }
    fn observe(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let mut triples = vec![];
        for (col, pred) in COLUMNS {
            let v = env.input.get(col).and_then(Value::as_str).ok_or_else(|| {
                CapError::new(FailureClass::SemanticRefusal, format!("MISSING_COLUMN:{col}"))
            })?;
            triples.push(json!({"s": format!("op:{}", env.operation_id), "p": pred, "o": v, "source": format!("row:{col}")}));
        }
        facts.json.insert("graph".into(), Value::Array(triples));
        Ok(())
    }
}

fn graph_value(facts: &Facts, pred: &str) -> Result<String, CapError> {
    facts
        .get("graph")
        .and_then(Value::as_array)
        .and_then(|a| a.iter().find(|t| t["p"] == pred))
        .and_then(|t| t["o"].as_str().map(str::to_string))
        .ok_or_else(|| CapError::new(FailureClass::ProcessInvalid, format!("GRAPH_MISSING:{pred}")))
}

fn effect_from_graph(facts: &Facts) -> Result<castle::sa2a_security::PreparedEffect, CapError> {
    effect_from_triples(facts.get("graph").cloned().unwrap_or(Value::Null))
}

fn effect_from_triples(graph: Value) -> Result<castle::sa2a_security::PreparedEffect, CapError> {
    let get = |pred: &str| {
        graph
            .as_array()
            .and_then(|a| a.iter().find(|t| t["p"] == pred))
            .and_then(|t| t["o"].as_str().map(str::to_string))
            .ok_or_else(|| CapError::new(FailureClass::ProcessInvalid, format!("GRAPH_MISSING:{pred}")))
    };
    PaymentEffect::prepare(
        &get("pay:principal")?,
        &get("pay:payer")?,
        &get("pay:payee")?,
        &get("pay:amountMinor")?,
        Currency::USD,
        &get("pay:dischargesObligation")?,
        "invoice-payment",
        None,
    )
    .map_err(|e| CapError::new(FailureClass::SemanticRefusal, e))
}

fn classify(refusal: String) -> CapError {
    let upper = refusal.to_uppercase();
    let class = if ["QUORUM", "CERT", "SIGNATURE", "AUDIENCE", "PRINCIPAL", "AUTHORITY"].iter().any(|k| upper.contains(k)) {
        FailureClass::AuthorityRefusal
    } else {
        FailureClass::SemanticRefusal
    };
    CapError::new(class, refusal)
}

// ----------------------------------------------------------------------- law

/// Law realization 1: journaled admission; replay re-evaluates the journaled record.
pub struct JournalLaw<'a> {
    pub fx: &'a Fixture,
    pub journal: AdmissionJournal,
    pub signers: Vec<&'static str>,
}
impl<'a> JournalLaw<'a> {
    pub fn new(fx: &'a Fixture) -> Self {
        Self { fx, journal: AdmissionJournal::open(fx.dir.join("journal")).unwrap(), signers: vec!["mac", "phone"] }
    }
}
impl Capability for JournalLaw<'_> {
    fn describe(&self) -> CapabilityContract {
        law_contract("journal-law")
    }
    fn construct(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let effect = effect_from_graph(facts)?;
        let cert = self.fx.cert(&effect, &env.operation_id, &self.signers);
        let admission =
            admit_payment_journaled(effect, &cert, &self.fx.admission_ctx(), &self.journal).map_err(classify)?;
        let digest = admission.effect().digest().to_string();
        let decision = match replay_admission(&self.journal, &digest)
            .map_err(|e| CapError::new(FailureClass::RealizationFailed, e))?
        {
            PayReplay::Reproduced { decision_digest, .. } => decision_digest,
            PayReplay::Diverged { reason } => return Err(CapError::new(FailureClass::SemanticRefusal, reason)),
        };
        facts.json.insert("law.effect_digest".into(), json!(digest));
        facts.json.insert("law.decision".into(), json!(decision));
        facts.put_artifact("admission", admission);
        Ok(())
    }
    fn replay(&self, _env: &OperationEnvelope, receipt: &Value) -> CapResult<()> {
        let d = receipt["law"]["effect_digest"].as_str().unwrap_or_default();
        match replay_admission(&self.journal, d).map_err(|e| CapError::new(FailureClass::RealizationFailed, e))? {
            PayReplay::Reproduced { decision_digest, .. } if Some(decision_digest.as_str()) == receipt["law"]["decision"].as_str() => Ok(()),
            other => Err(CapError::new(FailureClass::SemanticRefusal, format!("REPLAY_DIVERGED:{other:?}"))),
        }
    }
}

/// Law realization 2 (QRI): independently written, no journal; same contract.
pub struct DirectLaw<'a> {
    pub fx: &'a Fixture,
}
impl Capability for DirectLaw<'_> {
    fn describe(&self) -> CapabilityContract {
        law_contract("direct-law")
    }
    fn construct(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let effect = effect_from_graph(facts)?;
        let cert = self.fx.cert(&effect, &env.operation_id, &["mac", "phone"]);
        let admission = admit_payment(effect, &cert, &self.fx.admission_ctx()).map_err(classify)?;
        let digest = admission.effect().digest().to_string();
        let decision = format!("blake3:{}", blake3::hash(format!("ALIVE|{digest}").as_bytes()).to_hex());
        facts.json.insert("law.effect_digest".into(), json!(digest));
        facts.json.insert("law.decision".into(), json!(decision));
        facts.put_artifact("admission", admission);
        Ok(())
    }
    fn replay(&self, _env: &OperationEnvelope, receipt: &Value) -> CapResult<()> {
        let effect = effect_from_triples(receipt["graph"].clone())?;
        let digest = effect.digest().map_err(|e| CapError::new(FailureClass::RealizationFailed, format!("{e:?}")))?;
        let decision = format!("blake3:{}", blake3::hash(format!("ALIVE|{digest}").as_bytes()).to_hex());
        if Some(digest.as_str()) == receipt["law"]["effect_digest"].as_str() && Some(decision.as_str()) == receipt["law"]["decision"].as_str() {
            Ok(())
        } else {
            Err(CapError::new(FailureClass::SemanticRefusal, "REPLAY_DIVERGED"))
        }
    }
}

/// A healthy-looking realization whose construct fails at the realization level.
pub struct FlakyLaw;
impl Capability for FlakyLaw {
    fn describe(&self) -> CapabilityContract {
        law_contract("flaky-law")
    }
    fn construct(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<()> {
        Err(CapError::new(FailureClass::RealizationFailed, "crashed"))
    }
}

// ------------------------------------------------------------------- process

pub struct OcelProcess {
    log: Mutex<OcelLog>,
    expected: Vec<&'static str>,
}
impl OcelProcess {
    pub fn new() -> Self {
        Self {
            log: Mutex::new(OcelLog { version: "2.0", objects: vec![], events: vec![] }),
            expected: vec!["requested", "constructed", "executed", "receipted"],
        }
    }
    pub fn kinds(&self) -> Vec<String> {
        self.log.lock().unwrap().events.iter().map(|e| e.kind.clone()).collect()
    }
}
impl Capability for OcelProcess {
    fn describe(&self) -> CapabilityContract {
        contract(
            URI_PROCESS,
            "process.conformance.ocel.v1",
            "ocel-process",
            EffectClass::Observe,
            &["PROCESS_STATE_NOT_LAW_STATE"],
            &["process.events"],
        )
    }
    fn observe(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let kind = facts.get("process.event").and_then(Value::as_str).unwrap_or("unknown").to_string();
        let mut log = self.log.lock().unwrap();
        let id = format!("e{}", log.events.len());
        log.events.push(OcelEvent {
            id,
            kind,
            time: "2026-09-30T00:00:00.000Z".into(),
            attributes: Default::default(),
            object_ids: vec![env.subject.clone()],
        });
        let kinds: Vec<_> = log.events.iter().map(|e| e.kind.clone()).collect();
        facts.json.insert("process.state".into(), json!(kinds));
        Ok(())
    }
    fn receipt(&self, _env: &OperationEnvelope, _facts: &mut Facts) -> CapResult<Value> {
        let kinds = self.kinds();
        Ok(json!({"events": kinds, "conforms": kinds == self.expected}))
    }
}

// ------------------------------------------------------------------ evidence

pub struct SignedEvidence<'a> {
    pub fx: &'a Fixture,
    pub withhold: bool,
    store: Mutex<Vec<Receipt>>,
}
impl<'a> SignedEvidence<'a> {
    pub fn new(fx: &'a Fixture, withhold: bool) -> Self {
        Self { fx, withhold, store: Mutex::new(vec![]) }
    }
}
impl Capability for SignedEvidence<'_> {
    fn describe(&self) -> CapabilityContract {
        contract(
            URI_EVIDENCE,
            "evidence.attest.signed.v1",
            "signed-evidence",
            EffectClass::Observe,
            &["EVERY_DO_RECEIPTED", "RECEIPT_VERIFIES"],
            &["evidence.receipt"],
        )
    }
    fn observe(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        // Pre-actuation binding: envelope digest + law decision.
        let decision = facts.get("law.decision").cloned().unwrap_or(Value::Null);
        let bound = format!("{}|{}", env.digest(), decision);
        facts.json.insert("evidence.pre".into(), json!(format!("blake3:{}", blake3::hash(bound.as_bytes()).to_hex())));
        Ok(())
    }
    fn receipt(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<Value> {
        if self.withhold {
            return Err(CapError::new(FailureClass::EvidenceInsufficient, "withheld"));
        }
        let artifact = json!({
            "operation_id": env.operation_id,
            "envelope_digest": env.digest(),
            "graph": facts.get("graph"),
            "law": {"effect_digest": facts.get("law.effect_digest"), "decision": facts.get("law.decision")},
            "pre": facts.get("evidence.pre"),
            "actuation": facts.get("actuation.result"),
            "observed": facts.get("actuation.observed"),
        });
        let receipt = create_receipt(&artifact, EpistemicClass::Observed, &env.subject, &[], &RealBlake3, &self.fx.signer)
            .map_err(|e| CapError::new(FailureClass::RealizationFailed, e))?;
        let digest = receipt.receipt_digest.clone();
        self.store.lock().unwrap().push(receipt);
        let mut out = artifact.clone();
        out["artifact"] = artifact;
        out["receipt_digest"] = json!(digest);
        Ok(out)
    }
    fn replay(&self, _env: &OperationEnvelope, receipt: &Value) -> CapResult<()> {
        let digest = receipt["receipt_digest"].as_str().unwrap_or_default();
        let stored = self.store.lock().unwrap().iter().find(|r| r.receipt_digest == digest).cloned();
        let Some(stored) = stored else {
            return Err(CapError::new(FailureClass::EvidenceInsufficient, "UNKNOWN_RECEIPT"));
        };
        let trusted = BTreeSet::from(["receipt-key".to_string()]);
        let artifact = &receipt["artifact"];
        if verify_receipt(artifact, &stored, &RealBlake3, &self.fx.verifier, &trusted)
            && receipt["law"] == artifact["law"]
        {
            Ok(())
        } else {
            Err(CapError::new(FailureClass::EvidenceInsufficient, "RECEIPT_DOES_NOT_VERIFY"))
        }
    }
}

// ----------------------------------------------------------------- actuation

/// The only realization that crosses DO: settles through the real BRCE-guarded payment path.
pub struct LedgerActuator<'a> {
    pub fx: &'a Fixture,
    pub executions: AtomicU32,
}
impl<'a> LedgerActuator<'a> {
    pub fn new(fx: &'a Fixture) -> Self {
        Self { fx, executions: AtomicU32::new(0) }
    }
}
impl Capability for LedgerActuator<'_> {
    fn describe(&self) -> CapabilityContract {
        contract(
            URI_ACTUATION,
            "agent.actuate.payment.v1",
            "ledger-actuator",
            EffectClass::Do,
            &["SELECTED_AND_AUTHORIZED_AND_CAPABLE", "SINGLE_DO_BOUNDARY"],
            &["actuation.observed"],
        )
    }
    fn select(&self, env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        if env.authority != AUDIENCE {
            return Err(CapError::new(FailureClass::AuthorityRefusal, "ACTOR_NOT_AUTHORIZED"));
        }
        if !facts.has_artifact("admission") {
            return Err(CapError::new(FailureClass::ProcessInvalid, "NOTHING_CONSTRUCTED"));
        }
        Ok(())
    }
    fn execute(&self, _env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let admission: PaymentAdmission = facts
            .take_artifact("admission")
            .ok_or_else(|| CapError::new(FailureClass::ProcessInvalid, "NOTHING_CONSTRUCTED"))?;
        self.executions.fetch_add(1, Ordering::SeqCst);
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let exec = rt
            .block_on(execute_payment(admission, &self.fx.exec_ctx()))
            .map_err(|e| CapError::new(FailureClass::RealizationFailed, e))?;
        facts.json.insert(
            "actuation.result".into(),
            json!({"standing": format!("{:?}", exec.standing), "effect_digest": exec.effect_digest, "ocel_receipt": exec.ocel_receipt_digest}),
        );
        Ok(())
    }
    /// Independent observation: reads the ledger, not the actuator's own report.
    fn observe(&self, _env: &OperationEnvelope, facts: &mut Facts) -> CapResult<()> {
        let digest = facts.get("law.effect_digest").and_then(Value::as_str).unwrap_or_default().to_string();
        let entry = self.fx.ledger.lookup(&digest).map_err(|e| CapError::new(FailureClass::TransportFailed, format!("{e:?}")))?;
        let payee = self.fx.ledger.balance(PAYEE, Currency::USD).map_err(|e| CapError::new(FailureClass::TransportFailed, format!("{e:?}")))?;
        facts.json.insert("actuation.observed".into(), json!({"settled": entry.is_some(), "payee_balance": payee}));
        Ok(())
    }
}
