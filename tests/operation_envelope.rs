//! CASTLE owns product intent only: the envelope. Runtime composition lives in XaaS
//! (`~/xaas` `Xaas.Fabric`, closure test `test/xaas/fabric/castle_alive_test.exs`).
use castle::operation_envelope::{EffectClass, OperationEnvelope};
use serde_json::json;

fn envelope() -> OperationEnvelope {
    OperationEnvelope {
        operation_id: "op-1".into(),
        subject: "payment:op-1".into(),
        intent: "discharge obligation by transfer".into(),
        capability: "capability://agent/actuate".into(),
        context: json!({"epoch": 7}),
        ontology_refs: vec!["fibo:FND/Accounting".into()],
        input: json!({"amount_minor": "470000", "payer": "acct:a", "payee": "acct:b"}),
        actor: "principal:agent".into(),
        authority: "bounded-do".into(),
        effect_class: EffectClass::Do,
        standing_requirements: vec!["evidence.receipt".into()],
        prior_receipts: vec![],
        correlation: "corr-1".into(),
    }
}

#[test]
fn digest_is_stable_and_sensitive_to_every_field() {
    let a = envelope();
    assert_eq!(a.digest(), envelope().digest());
    let mut b = envelope();
    b.authority = "other".into();
    assert_ne!(a.digest(), b.digest());
    let mut c = envelope();
    c.effect_class = EffectClass::Construct;
    assert_ne!(a.digest(), c.digest());
}

#[test]
fn json_carries_exactly_the_schema_required_fields() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/castle-fabric/operation-envelope-v1.schema.json")).unwrap();
    let mut required: Vec<String> = schema["required"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    required.sort();
    let value = envelope().to_json();
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, required);
    assert_eq!(value["effect_class"], "DO");
}

#[test]
fn effect_classes_are_ordered_observe_to_do() {
    assert!(EffectClass::Observe < EffectClass::Select && EffectClass::Select < EffectClass::Construct && EffectClass::Construct < EffectClass::Do);
}

#[test]
fn envelope_source_names_no_runtime_or_realization() {
    let src = include_str!("../src/operation_envelope.rs");
    for banned in ["payments::", "Fixture", "ash_", "Registry", "Orchestrator"] {
        assert!(!src.contains(banned), "operation_envelope.rs mentions `{banned}`");
    }
    assert!(!std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/fabric")).exists());
}
