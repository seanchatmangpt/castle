//! CASTLE ALIVE: the first Chicago test of the capability fabric. One payment is driven through
//! the registry across all five planes; every assertion is on final, observable state (ledger
//! balances, receipts, process log), never on interaction counts of doubles. No mocks.
mod common;
use common::fabric_planes::*;
use common::payments::*;

use std::sync::atomic::Ordering;

use common::fabric::*;
use castle::payments::*;
use serde_json::json;

fn envelope(amount: &str, obligation: &str) -> OperationEnvelope {
    OperationEnvelope {
        operation_id: format!("op-{obligation}"),
        subject: format!("payment:{obligation}"),
        intent: "discharge obligation by transfer".into(),
        capability: "capability://payments/transfer".into(),
        context: json!({"epoch": 7}),
        ontology_refs: vec!["fibo:FND/Accounting".into()],
        input: json!({
            "principal": PRINCIPAL, "payer": PAYER, "payee": PAYEE,
            "amount_minor": amount, "obligation_id": obligation,
        }),
        actor: PRINCIPAL.into(),
        authority: AUDIENCE.into(),
        effect_class: EffectClass::Do,
        standing_requirements: vec!["evidence.receipt".into()],
        prior_receipts: vec![],
        correlation: format!("corr-{obligation}"),
    }
}

struct World<'a> {
    registry: Registry<'a>,
}

fn world<'a>(
    fx: &'a Fixture,
    process: &'a OcelProcess,
    actuator: &'a LedgerActuator<'a>,
    laws: Vec<Box<dyn Capability + 'a>>,
    withhold: bool,
) -> World<'a> {
    let mut registry = Registry::new();
    registry.register(Box::new(RowProjection)).unwrap();
    registry.register(Box::new(ProcessRef(process))).unwrap();
    for law in laws {
        registry.register(law).unwrap();
    }
    registry.register(Box::new(SignedEvidence::new(fx, withhold))).unwrap();
    registry.register(Box::new(ActuatorRef(actuator))).unwrap();
    World { registry }
}

// Borrowing wrappers so the test can inspect the process log / DO counter after the run.
struct ProcessRef<'a>(&'a OcelProcess);
impl Capability for ProcessRef<'_> {
    fn describe(&self) -> CapabilityContract {
        self.0.describe()
    }
    fn observe(&self, e: &OperationEnvelope, f: &mut Facts) -> CapResult<()> {
        self.0.observe(e, f)
    }
    fn receipt(&self, e: &OperationEnvelope, f: &mut Facts) -> CapResult<serde_json::Value> {
        self.0.receipt(e, f)
    }
}
struct ActuatorRef<'a>(&'a LedgerActuator<'a>);
impl Capability for ActuatorRef<'_> {
    fn describe(&self) -> CapabilityContract {
        self.0.describe()
    }
    fn select(&self, e: &OperationEnvelope, f: &mut Facts) -> CapResult<()> {
        self.0.select(e, f)
    }
    fn execute(&self, e: &OperationEnvelope, f: &mut Facts) -> CapResult<()> {
        self.0.execute(e, f)
    }
    fn observe(&self, e: &OperationEnvelope, f: &mut Facts) -> CapResult<()> {
        self.0.observe(e, f)
    }
}

fn balances(fx: &Fixture) -> (u64, u64) {
    (fx.ledger.balance(PAYER, Currency::USD).unwrap(), fx.ledger.balance(PAYEE, Currency::USD).unwrap())
}

#[test]
fn castle_owns_only_the_envelope_and_names_no_realization() {
    let root = env!("CARGO_MANIFEST_DIR");
    assert!(!std::path::Path::new(&format!("{root}/src/fabric")).exists(), "runtime composition belongs to XaaS");
    let src = std::fs::read_to_string(format!("{root}/src/operation_envelope.rs")).unwrap();
    for banned in ["payments::", "Fixture", "PaymentAdmission", "ash_", "SimRail", "Registry", "Orchestrator"] {
        assert!(!src.contains(banned), "src/operation_envelope.rs mentions `{banned}`");
    }
}

#[test]
fn valid_payment_is_alive_with_exactly_one_do_and_independent_observation() {
    let fx = Fixture::new("alive");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let w = world(&fx, &process, &actuator, vec![Box::new(JournalLaw::new(&fx))], false);
    let env = envelope("470000", "inv-1");
    let out = Orchestrator::new(&w.registry).run(&env);

    assert_eq!(out.refusal, None);
    assert_eq!(
        out.stages,
        vec![Stage::Requested, Stage::Qualified, Stage::Constructed, Stage::Executed, Stage::Observed, Stage::Receipted, Stage::Reconciled]
    );
    assert_eq!(out.standing, Standing::Alive);
    assert_eq!(out.do_crossings, 1);
    assert_eq!(actuator.executions.load(Ordering::SeqCst), 1);

    // External effect observed in the ledger itself, plus the actuator's observation agrees.
    assert_eq!(balances(&fx), (530_000, 470_000));
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(out.facts.get("actuation.observed").unwrap()["settled"], true);
    assert_eq!(out.facts.get("actuation.result").unwrap()["standing"], "Settled");

    // Process history reconstructs the run and conforms.
    assert_eq!(process.kinds(), ["requested", "constructed", "executed", "receipted"]);
    assert_eq!(out.conformance.as_ref().unwrap()["conforms"], true);

    // All five planes served through URIs.
    for uri in [URI_PROJECTION, URI_PROCESS, URI_LAW, URI_EVIDENCE, URI_ACTUATION] {
        assert!(out.served_by.contains_key(uri), "{uri} not served");
    }
    // Projection fed law: the law decision is bound to the projected graph.
    assert!(out.facts.get("graph").unwrap().as_array().unwrap().iter().all(|t| t["source"].as_str().unwrap().starts_with("row:")));

    // Replay reproduces the decision and re-verifies the receipt without re-actuating.
    let receipt = out.receipt.clone().expect("receipt");
    let verdict = Orchestrator::new(&w.registry).replay(&env, &receipt);
    assert_eq!(verdict, common::fabric::ReplayVerdict { same_decision: true, receipt_valid: true, detail: "law=ok evidence=ok".into() });
    assert_eq!(actuator.executions.load(Ordering::SeqCst), 1);
    assert_eq!(balances(&fx), (530_000, 470_000));
}

#[test]
fn over_limit_payment_is_nonconstructible_and_never_reaches_do() {
    let fx = Fixture::new("overlimit");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let w = world(&fx, &process, &actuator, vec![Box::new(JournalLaw::new(&fx))], false);
    let out = Orchestrator::new(&w.registry).run(&envelope("900000", "inv-big"));

    assert_eq!(out.standing, Standing::Refused);
    assert!(out.refusal.as_deref().unwrap().starts_with("REFUSED:SEMANTIC_REFUSAL"), "{:?}", out.refusal);
    assert!(!out.stages.contains(&Stage::Constructed));
    assert_eq!(out.do_crossings, 0);
    assert_eq!(actuator.executions.load(Ordering::SeqCst), 0);
    assert_eq!(balances(&fx), (1_000_000, 0));
    assert_eq!(process.kinds(), ["requested", "refused"]);
}

#[test]
fn insufficient_authority_is_an_authority_refusal_not_a_do() {
    let fx = Fixture::new("noauth");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let mut law = JournalLaw::new(&fx);
    law.signers = vec!["mac"]; // threshold is 2
    let w = world(&fx, &process, &actuator, vec![Box::new(law)], false);
    let out = Orchestrator::new(&w.registry).run(&envelope("1000", "inv-noauth"));

    assert_eq!(out.standing, Standing::Refused);
    assert!(out.refusal.as_deref().unwrap().starts_with("REFUSED:AUTHORITY_REFUSAL"), "{:?}", out.refusal);
    assert_eq!(out.do_crossings, 0);
    assert_eq!(balances(&fx), (1_000_000, 0));
}

#[test]
fn wrong_actuator_authority_is_refused_before_do() {
    let fx = Fixture::new("badactor");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let w = world(&fx, &process, &actuator, vec![Box::new(JournalLaw::new(&fx))], false);
    let mut env = envelope("1000", "inv-badactor");
    env.authority = "actuator:somebody-else".into();
    let out = Orchestrator::new(&w.registry).run(&env);

    assert_eq!(out.standing, Standing::Refused);
    assert!(out.refusal.as_deref().unwrap().contains("ACTOR_NOT_AUTHORIZED"));
    assert_eq!(out.do_crossings, 0);
    assert_eq!(actuator.executions.load(Ordering::SeqCst), 0);
    assert_eq!(balances(&fx), (1_000_000, 0));
}

#[test]
fn qri_equivalent_law_realization_replaces_the_original_with_the_same_standing() {
    let run = |tag: &str, direct: bool| {
        let fx = Fixture::new(tag);
        let process = OcelProcess::new();
        let actuator = LedgerActuator::new(&fx);
        let law: Box<dyn Capability> = if direct { Box::new(DirectLaw { fx: &fx }) } else { Box::new(JournalLaw::new(&fx)) };
        let w = world(&fx, &process, &actuator, vec![law], false);
        let env = envelope("250000", "inv-qri");
        let out = Orchestrator::new(&w.registry).run(&env);
        let verdict = Orchestrator::new(&w.registry).replay(&env, out.receipt.as_ref().unwrap());
        (out.standing, out.stages.clone(), balances(&fx), out.served_by[URI_LAW].clone(), verdict)
    };
    let (s1, st1, b1, by1, v1) = run("qri-a", false);
    let (s2, st2, b2, by2, v2) = run("qri-b", true);
    assert_eq!((s1, s2), (Standing::Alive, Standing::Alive));
    assert_eq!(st1, st2);
    assert_eq!(b1, b2);
    assert_eq!((by1.as_str(), by2.as_str()), ("journal-law", "direct-law"));
    assert!(v1.same_decision && v1.receipt_valid && v2.same_decision && v2.receipt_valid);
}

#[test]
fn registry_refuses_a_realization_that_breaks_the_contract() {
    struct Impostor;
    impl Capability for Impostor {
        fn describe(&self) -> CapabilityContract {
            let mut c = law_contract("impostor");
            c.invariants.clear();
            c
        }
    }
    let fx = Fixture::new("impostor");
    let mut registry = Registry::new();
    registry.register(Box::new(JournalLaw::new(&fx))).unwrap();
    assert_eq!(registry.register(Box::new(Impostor)).unwrap_err(), "REFUSED:QRI_CONTRACT_MISMATCH");
    assert_eq!(registry.contracts(URI_LAW).len(), 1);
}

#[test]
fn failed_realization_closes_one_edge_and_a_qualified_alternative_serves() {
    let fx = Fixture::new("flaky");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let laws: Vec<Box<dyn Capability>> = vec![Box::new(FlakyLaw), Box::new(JournalLaw::new(&fx))];
    let w = world(&fx, &process, &actuator, laws, false);
    let out = Orchestrator::new(&w.registry).run(&envelope("1000", "inv-flaky"));

    assert_eq!(out.standing, Standing::Alive);
    assert_eq!(out.served_by[URI_LAW], "journal-law");
    assert_eq!(balances(&fx), (999_000, 1_000));
}

#[test]
fn execution_without_a_receipt_has_no_standing() {
    let fx = Fixture::new("noreceipt");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let w = world(&fx, &process, &actuator, vec![Box::new(JournalLaw::new(&fx))], true);
    let out = Orchestrator::new(&w.registry).run(&envelope("5000", "inv-noreceipt"));

    assert!(out.stages.contains(&Stage::Executed) && out.stages.contains(&Stage::Observed));
    assert!(!out.stages.contains(&Stage::Receipted));
    assert_eq!(out.standing, Standing::Unknown);
    assert!(out.refusal.as_deref().unwrap().contains("EVIDENCE_INSUFFICIENT"));
    assert_eq!(balances(&fx), (995_000, 5_000)); // it happened; standing still cannot be asserted
}

#[test]
fn missing_capability_is_unavailable_not_a_private_fallback() {
    let fx = Fixture::new("missing");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let mut registry = Registry::new();
    registry.register(Box::new(RowProjection)).unwrap();
    registry.register(Box::new(ProcessRef(&process))).unwrap();
    registry.register(Box::new(ActuatorRef(&actuator))).unwrap();
    let out = Orchestrator::new(&registry).run(&envelope("1000", "inv-missing"));

    assert!(out.refusal.as_deref().unwrap().starts_with("REFUSED:CAPABILITY_UNAVAILABLE"));
    assert_eq!(out.do_crossings, 0);
    assert_eq!(balances(&fx), (1_000_000, 0));
}

#[test]
fn tampered_receipt_fails_replay() {
    let fx = Fixture::new("tamper");
    let process = OcelProcess::new();
    let actuator = LedgerActuator::new(&fx);
    let w = world(&fx, &process, &actuator, vec![Box::new(JournalLaw::new(&fx))], false);
    let env = envelope("1000", "inv-tamper");
    let out = Orchestrator::new(&w.registry).run(&env);
    let mut receipt = out.receipt.unwrap();
    receipt["artifact"]["observed"]["payee_balance"] = json!(999_999);
    let verdict = Orchestrator::new(&w.registry).replay(&env, &receipt);
    assert!(!verdict.receipt_valid, "{}", verdict.detail);
}

#[test]
fn a_capability_cannot_be_driven_beyond_its_declared_effect_class() {
    let law = law_contract("any");
    assert!(Registry::permit(&law, EffectClass::Construct).is_ok());
    assert_eq!(Registry::permit(&law, EffectClass::Do).unwrap_err().class, FailureClass::AuthorityRefusal);
}
