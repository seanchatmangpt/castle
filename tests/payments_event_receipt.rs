mod common;
use common::payments::*;

use castle::payments::event_receipt::*;
use castle::payments::replay::*;
use castle::payments::*;

fn inputs() -> EventInputs {
    EventInputs {
        effect_id: "sha256:aa".into(),
        obligation_id: "obl:1".into(),
        admission_decision_digest: "sha256:bb".into(),
        pee_effect_id: Some("pee-1".into()),
        construct_digest: Some("c1".into()),
        brce_prepare_digests: vec!["p1".into(), "p2".into()],
        brce_outcome_digests: vec!["o1".into(), "o2".into()],
        rail_correlation_id: Some("corr-1".into()),
        rail_payload_digest: Some("rp".into()),
        finality_evidence_digest: Some("fin".into()),
        ledger_entry_digest: Some("led".into()),
        claim_state: "final".into(),
        compliance_bundle_digest: Some("cmp".into()),
        counterparty_evidence_digest: Some("cp".into()),
        settled_at_ms: Some(5_000),
    }
}

#[test]
fn seal_verify_and_determinism() {
    let r1 = seal_event_receipt(&inputs()).unwrap();
    let r2 = seal_event_receipt(&inputs()).unwrap();
    assert_eq!(r1, r2);
    assert!(r1.receipt_digest.starts_with("sha256:"));
    verify_event_receipt(&r1).unwrap();
}

#[test]
fn every_single_field_tamper_is_detected() {
    let good = seal_event_receipt(&inputs()).unwrap();
    let muts: Vec<(&str, Box<dyn Fn(&mut EventInputs)>)> = vec![
        ("effect_id", Box::new(|i| i.effect_id = "x".into())),
        ("obligation_id", Box::new(|i| i.obligation_id = "x".into())),
        ("admission", Box::new(|i| i.admission_decision_digest = "x".into())),
        ("pee", Box::new(|i| i.pee_effect_id = None)),
        ("construct", Box::new(|i| i.construct_digest = Some("x".into()))),
        ("prepare", Box::new(|i| i.brce_prepare_digests.push("x".into()))),
        ("outcome", Box::new(|i| i.brce_outcome_digests.clear())),
        ("corr", Box::new(|i| i.rail_correlation_id = Some("x".into()))),
        ("payload", Box::new(|i| i.rail_payload_digest = None)),
        ("finality", Box::new(|i| i.finality_evidence_digest = Some("x".into()))),
        ("ledger", Box::new(|i| i.ledger_entry_digest = None)),
        ("claim", Box::new(|i| i.claim_state = "returned".into())),
        ("compliance", Box::new(|i| i.compliance_bundle_digest = Some("x".into()))),
        ("counterparty", Box::new(|i| i.counterparty_evidence_digest = None)),
        ("settled", Box::new(|i| i.settled_at_ms = Some(1))),
    ];
    for (name, m) in muts {
        let mut r = good.clone();
        m(&mut r.inputs);
        assert_eq!(verify_event_receipt(&r).unwrap_err(), EVENT_RECEIPT_TAMPERED, "{name}");
    }
    let mut r = good;
    r.receipt_digest = "sha256:00".into();
    assert_eq!(verify_event_receipt(&r).unwrap_err(), EVENT_RECEIPT_TAMPERED);
}

#[test]
fn incomplete_inputs_are_refused() {
    for m in [
        (|i: &mut EventInputs| i.effect_id.clear()) as fn(&mut EventInputs),
        |i| i.obligation_id.clear(),
        |i| i.admission_decision_digest.clear(),
    ] {
        let mut i = inputs();
        m(&mut i);
        assert_eq!(seal_event_receipt(&i).unwrap_err(), EVENT_RECEIPT_INCOMPLETE);
    }
}

#[test]
fn explain_answers_and_nulls_with_reason() {
    let e = explain(&seal_event_receipt(&inputs()).unwrap());
    assert_eq!(e["why"]["obligation_id"], "obl:1");
    assert_eq!(e["under_whose_authority"]["admission_decision_digest"], "sha256:bb");
    assert_eq!(e["through_which_rail"]["correlation_id"], "corr-1");
    assert_eq!(e["when_final"]["settled_at_ms"], 5_000);
    assert_eq!(e["what_happened"]["claim_state"], "final");
    assert!(e["whose_money"]["value"].is_null());
    assert!(!e["whose_money"]["reason"].as_str().unwrap().is_empty());
    assert!(e["to_whom"]["value"].is_null());

    let mut i = inputs();
    i.rail_correlation_id = None;
    i.settled_at_ms = None;
    let e = explain(&seal_event_receipt(&i).unwrap());
    assert!(e["through_which_rail"]["value"].is_null());
    assert!(e["through_which_rail"]["reason"].as_str().unwrap().contains("no rail"));
    assert!(e["when_final"]["value"].is_null());
}

#[tokio::test]
async fn real_settled_payment_seals_and_verifies() {
    let fx = Fixture::new("evt-e2e");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let eff = fx.effect("470000", "invoice-9821");
    let cert = fx.cert(&eff, "n-1", &["mac", "phone"]);
    let adm = admit_payment_journaled(eff, &cert, &fx.admission_ctx(), &j).unwrap();
    let digest = adm.effect().digest().to_string();
    let decision = match replay_admission(&j, &digest).unwrap() {
        ReplayVerdict::Reproduced { decision_digest, .. } => decision_digest,
        o => panic!("{o:?}"),
    };
    let exec = execute_payment(adm, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled);
    let claim = fx.claims.get(&digest).unwrap().unwrap();
    let entry = exec.ledger_entry.clone().unwrap();
    assert!(fx.ledger.entries().unwrap().contains(&entry));

    let inp = EventInputs {
        effect_id: digest.clone(),
        obligation_id: claim.obligation_id.clone(),
        admission_decision_digest: decision.clone(),
        pee_effect_id: None,
        construct_digest: exec.construct_digest.clone(),
        brce_prepare_digests: exec.brce_prepare_receipt_digests.clone(),
        brce_outcome_digests: exec.brce_outcome_receipt_digests.clone(),
        rail_correlation_id: None,
        rail_payload_digest: None,
        finality_evidence_digest: None,
        ledger_entry_digest: Some(ledger_entry_digest(&entry)),
        claim_state: format!("{:?}", claim.state),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: None,
    };
    let r = seal_event_receipt(&inp).unwrap();
    verify_event_receipt(&r).unwrap();
    assert_eq!(r.inputs.obligation_id, "invoice-9821");
    assert_eq!(r.inputs.claim_state, "Executed");
    assert_eq!(r.inputs.brce_prepare_digests.len(), 2);
    let e = explain(&r);
    assert_eq!(e["under_whose_authority"]["admission_decision_digest"], decision.as_str());
    assert!(e["through_which_rail"]["value"].is_null());
}
