mod common;
use common::payments::*;

use castle::payments::pee::*;
use castle::payments::{Currency, PaymentEffect};

fn bindings() -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: "sha256:law".into(),
        counterparty_evidence_digest: "sha256:cp".into(),
        funding_source_id: "fund:treasury".into(),
        resource_reservation_id: "resv:1".into(),
        rail_profile_id: "rail:sim".into(),
        message_profile_version: "pain.001.001.11".into(),
        parent_receipt: "receipt:parent".into(),
        created_at_ms: 500,
        expires_at_ms: 5_000,
    }
}

fn sealed(tag: &str, nonce: &str) -> PreparedEconomicEffect {
    let f = Fixture::new(tag);
    let adm = f.admit(f.effect("100", "inv-1"), nonce).expect("admits");
    PreparedEconomicEffect::seal(&adm, &bindings()).expect("seals")
}

#[test]
fn seal_verifies_and_is_self_identifying() {
    let p = sealed("pee-basic", "n1");
    assert_eq!(p.version, 1);
    assert_eq!(p.effect_id, p.idempotency_key);
    assert_eq!(p.nonce, "n1");
    assert_eq!(p.amount_minor, "100");
    assert!(p.verify_identity().is_ok());
}

#[test]
fn identical_inputs_seal_byte_identically() {
    let f = Fixture::new("pee-det");
    let adm = f.admit(f.effect("100", "inv-1"), "n1").unwrap();
    let a = PreparedEconomicEffect::seal(&adm, &bindings()).unwrap();
    let b = PreparedEconomicEffect::seal(&adm, &bindings()).unwrap();
    assert_eq!(a, b);
    assert_eq!(serde_json::to_vec(&a).unwrap(), serde_json::to_vec(&b).unwrap());
    // a different nonce yields a different identity
    let c = sealed("pee-det2", "n2");
    assert_ne!(a.effect_id, c.effect_id);
}

#[test]
fn one_field_mutation_table_breaks_identity_and_reseal_differs() {
    let base = sealed("pee-mut", "n1");
    type Mutator = fn(&mut PreparedEconomicEffect);
    let table: Vec<(&str, Mutator)> = vec![
        ("beneficiary", |p| p.beneficiary_account = "acct:evil".into()),
        ("payer", |p| p.payer_account = "acct:other".into()),
        ("amount", |p| p.amount_minor = "101".into()),
        ("currency", |p| p.currency = Currency::EUR),
        ("purpose", |p| p.purpose = "other".into()),
        ("obligation", |p| p.obligation_id = "inv-2".into()),
        ("authority_digest", |p| p.authority_digest = "sha256:x".into()),
        ("policy_profile", |p| p.policy_profile_id = "policy:v2".into()),
        ("law_state", |p| p.law_state_digest = "sha256:law2".into()),
        ("counterparty_evidence", |p| p.counterparty_evidence_digest = "sha256:cp2".into()),
        ("funding_source", |p| p.funding_source_id = "fund:other".into()),
        ("reservation", |p| p.resource_reservation_id = "resv:2".into()),
        ("rail_profile", |p| p.rail_profile_id = "rail:other".into()),
        ("message_profile", |p| p.message_profile_version = "pain.001.001.12".into()),
        ("nonce", |p| p.nonce = "n2".into()),
        ("expiry", |p| p.expires_at_ms += 1),
        ("principal", |p| p.principal_id = "principal:x".into()),
        ("grant", |p| p.authority_grant_id = "grant:x".into()),
        ("parent_receipt", |p| p.parent_receipt = "receipt:other".into()),
        ("created_at", |p| p.created_at_ms += 1),
    ];
    for (name, m) in table {
        let mut t = base.clone();
        m(&mut t);
        assert_eq!(t.verify_identity(), Err(PEE_IDENTITY_MISMATCH.to_string()), "{name}");
        t.reseal_identity().unwrap();
        assert_ne!(t.effect_id, base.effect_id, "{name}");
        assert!(t.verify_identity().is_ok(), "{name}");
    }
    let mut t = base.clone();
    t.idempotency_key = "other".into();
    assert_eq!(t.verify_identity(), Err(PEE_IDENTITY_MISMATCH.to_string()));
    let mut t = base.clone();
    t.effect_id = "sha256:forged".into();
    assert_eq!(t.verify_identity(), Err(PEE_IDENTITY_MISMATCH.to_string()));
    let mut t = base;
    t.canonical_payload_digest = "sha256:forged".into();
    assert_eq!(t.verify_identity(), Err(PEE_IDENTITY_MISMATCH.to_string()));
}

#[test]
fn f2_authorization_not_transferable_to_other_payee() {
    let f = Fixture::new("pee-f2");
    let a = f.effect("100", "inv-a");
    let cert_a = f.cert(&a, "nA", &["mac", "phone"]);
    let b = PaymentEffect::prepare(PRINCIPAL, PAYEE, PAYER, "100", Currency::USD, "inv-a", "invoice-payment", None).unwrap();
    assert!(castle::payments::admit_payment(b, &cert_a, &f.admission_ctx()).is_err());

    let adm = f.admit(a, "nA").unwrap();
    let mut pee = PreparedEconomicEffect::seal(&adm, &bindings()).unwrap();
    assert_eq!(pee.beneficiary_account, PAYEE);
    pee.beneficiary_account = PAYER.into();
    assert_eq!(pee.verify_identity(), Err(PEE_IDENTITY_MISMATCH.to_string()));
}

#[test]
fn f4_obligation_effect_idempotency_join_survives_serde() {
    let p = sealed("pee-f4", "n1");
    let back: PreparedEconomicEffect = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(back, p);
    assert_eq!(back.obligation_id, "inv-1");
    assert_eq!(back.effect_id, back.idempotency_key);
    assert!(back.verify_identity().is_ok());
}

#[test]
fn freshness_window() {
    let p = sealed("pee-fresh", "n1");
    assert_eq!(p.check_fresh(499), Err(PEE_NOT_YET_VALID.to_string()));
    assert!(p.check_fresh(500).is_ok());
    assert!(p.check_fresh(4_999).is_ok());
    assert_eq!(p.check_fresh(5_000), Err(PEE_EXPIRED.to_string()));
    assert_eq!(p.check_fresh(u64::MAX), Err(PEE_EXPIRED.to_string()));
}

#[test]
fn incomplete_bindings_refuse() {
    let f = Fixture::new("pee-inc");
    let adm = f.admit(f.effect("100", "inv-1"), "n1").unwrap();
    let fields: Vec<fn(&mut EffectBindings)> = vec![
        |b| b.policy_profile_id.clear(),
        |b| b.law_state_digest.clear(),
        |b| b.counterparty_evidence_digest.clear(),
        |b| b.funding_source_id.clear(),
        |b| b.resource_reservation_id.clear(),
        |b| b.rail_profile_id.clear(),
        |b| b.message_profile_version.clear(),
        |b| b.parent_receipt.clear(),
    ];
    for clear in fields {
        let mut b = bindings();
        clear(&mut b);
        assert_eq!(PreparedEconomicEffect::seal(&adm, &b), Err(PEE_BINDING_INCOMPLETE.to_string()));
    }
}
