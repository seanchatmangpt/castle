mod common;
use common::payments::*;

use castle::payments::admission::{admit_payment_screened, Screening, PAYMENT_SCREENING_REQUIRED};
use castle::payments::compliance::{ComplianceControl, SanctionsList, SANCTIONS_HIT};
use castle::payments::counterparty::{CounterpartyRegistry, COUNTERPARTY_UNVERIFIED};
use castle::payments::*;

fn list(entries: &[&str], version: &str) -> SanctionsList {
    let j = serde_json::json!({ "list_id": "OFAC-REF", "version": version, "entries": entries });
    SanctionsList::from_json(j.to_string().as_bytes()).expect("list")
}

fn registry(with_payee: bool) -> CounterpartyRegistry {
    let mut parties = vec![serde_json::json!({ "party_id": "p-treasury", "legal_name": "Treasury Ltd", "accounts": [PAYER] })];
    if with_payee {
        parties.push(serde_json::json!({ "party_id": "p-supplier", "legal_name": "Supplier GmbH", "accounts": [PAYEE] }));
    }
    CounterpartyRegistry::from_json(serde_json::json!({ "parties": parties }).to_string().as_bytes()).expect("registry")
}

fn bindings(law: &str, cp: &str) -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: law.into(),
        counterparty_evidence_digest: cp.into(),
        funding_source_id: "fund:treasury".into(),
        resource_reservation_id: "res:1".into(),
        rail_profile_id: "sim-rail".into(),
        message_profile_version: "pain.001.001.09".into(),
        parent_receipt: "receipt:parent".into(),
        created_at_ms: 0,
        expires_at_ms: 100_000,
    }
}

#[test]
fn sanctions_hit_on_payee_refuses_before_nonce_or_budget_are_spent() {
    let fx = Fixture::new("scr-hit");
    let effect = fx.effect("1000", "inv-1");
    let cert = fx.cert(&effect, "n-1", &["mac", "phone"]);
    let digest = effect.digest().unwrap();
    let reg = registry(true);

    let bad = list(&["acct:supplier-9821"], "1");
    let controls: Vec<&dyn ComplianceControl> = vec![&bad];
    let err = admit_payment_screened(
        effect.clone(),
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
    )
    .unwrap_err();
    assert_eq!(err, SANCTIONS_HIT);
    assert!(fx.claims.get(&digest).unwrap().is_none(), "no budget reserved");

    // The SAME certificate (same nonce) still admits against a list without the hit.
    let clean = list(&["acct:someone-else"], "2");
    let controls: Vec<&dyn ComplianceControl> = vec![&clean];
    let (a, ev) = admit_payment_screened(
        effect,
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
    )
    .expect("nonce was not burned");
    assert_eq!(a.effect().digest(), digest);
    assert!(ev.compliance_bundle_digest.starts_with("sha256:"));
    assert!(ev.counterparty_evidence_digest.starts_with("sha256:"));
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::Reserved);
}

#[test]
fn unknown_counterparty_refuses_before_nonce_or_budget_are_spent() {
    let fx = Fixture::new("scr-unk");
    let effect = fx.effect("1000", "inv-1");
    let cert = fx.cert(&effect, "n-1", &["mac", "phone"]);
    let digest = effect.digest().unwrap();
    let clean = list(&["acct:nobody"], "1");
    let controls: Vec<&dyn ComplianceControl> = vec![&clean];

    let partial = registry(false);
    let err = admit_payment_screened(
        effect.clone(),
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &partial },
    )
    .unwrap_err();
    assert_eq!(err, COUNTERPARTY_UNVERIFIED);
    assert!(fx.claims.get(&digest).unwrap().is_none());

    let full = registry(true);
    admit_payment_screened(effect, &cert, &fx.admission_ctx(), &Screening { controls: &controls, counterparties: &full })
        .expect("same certificate admits once the payee is registered");
}

#[test]
fn require_screening_policy_refuses_unscreened_and_admits_screened() {
    let mut fx = Fixture::new("scr-req");
    fx.policy.require_screening = true;
    let effect = fx.effect("1000", "inv-1");
    let cert = fx.cert(&effect, "n-1", &["mac", "phone"]);
    let digest = effect.digest().unwrap();

    assert_eq!(fx.admit(effect.clone(), "n-1").unwrap_err(), PAYMENT_SCREENING_REQUIRED);
    assert!(fx.claims.get(&digest).unwrap().is_none());

    let clean = list(&["acct:nobody"], "1");
    let controls: Vec<&dyn ComplianceControl> = vec![&clean];
    let reg = registry(true);
    let (a, _) = admit_payment_screened(
        effect,
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
    )
    .expect("screened path admits under require_screening; nonce was not burned by the refusal");
    assert_eq!(a.effect().digest(), digest);
}

#[test]
fn unscreened_admission_is_unchanged_when_screening_is_not_required() {
    let fx = Fixture::new("scr-off");
    assert!(!fx.policy.require_screening);
    fx.admit(fx.effect("1000", "inv-1"), "n-1").expect("unscreened admits by default");
}

#[test]
fn empty_controls_refuse_screened_admission() {
    let fx = Fixture::new("scr-empty");
    let effect = fx.effect("1000", "inv-1");
    let cert = fx.cert(&effect, "n-1", &["mac", "phone"]);
    let reg = registry(true);
    let controls: Vec<&dyn ComplianceControl> = vec![];
    let err = admit_payment_screened(effect, &cert, &fx.admission_ctx(), &Screening { controls: &controls, counterparties: &reg })
        .unwrap_err();
    assert_eq!(err, "REFUSED:NO_COMPLIANCE_CONTROLS");
}

fn effect_id_for(tag: &str, list_version: &str, entries: &[&str]) -> String {
    let fx = Fixture::new(tag);
    let effect = fx.effect("1000", "inv-1");
    let cert = fx.cert(&effect, "n-1", &["mac", "phone"]);
    let l = list(entries, list_version);
    let controls: Vec<&dyn ComplianceControl> = vec![&l];
    let reg = registry(true);
    let (a, ev) =
        admit_payment_screened(effect, &cert, &fx.admission_ctx(), &Screening { controls: &controls, counterparties: &reg })
            .expect("admits");
    let pee = PreparedEconomicEffect::seal(
        &a,
        &bindings(&ev.compliance_bundle_digest, &ev.counterparty_evidence_digest),
    )
    .expect("seals");
    pee.verify_identity().expect("identity");
    assert_eq!(pee.counterparty_evidence_digest, ev.counterparty_evidence_digest);
    assert_eq!(pee.law_state_digest, ev.compliance_bundle_digest);
    pee.effect_id
}

#[test]
fn compliance_bundle_digest_flows_into_effect_identity_and_tracks_the_list() {
    let a1 = effect_id_for("scr-id-a", "1", &["acct:nobody"]);
    let a2 = effect_id_for("scr-id-b", "1", &["acct:nobody"]);
    let b = effect_id_for("scr-id-c", "2", &["acct:nobody", "acct:another"]);
    assert_eq!(a1, a2, "same list => same effect id");
    assert_ne!(a1, b, "changed list => different effect id");
}
