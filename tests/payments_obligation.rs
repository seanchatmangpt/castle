mod common;
use common::payments::*;

use std::collections::BTreeSet;

use castle::payments::obligation::*;
use castle::payments::{Currency, PaymentEffect};

#[test]
fn derivation_deterministic_and_collision_free() {
    assert_eq!(derive_obligation_id("a", "b", "p", "i"), derive_obligation_id("a", "b", "p", "i"));
    let rows = [
        ("a", "b", "p", "i"),
        ("b", "a", "p", "i"),
        ("a", "b", "q", "i"),
        ("a", "b", "p", "j"),
        ("ab", "c", "p", "i"),
        ("a", "bc", "p", "i"),
        ("a", "b", "", "pi"),
        ("a", "b", "pi", ""),
        ("", "ab", "p", "i"),
        ("ab", "", "p", "i"),
    ];
    let ids: BTreeSet<String> = rows.iter().map(|r| derive_obligation_id(r.0, r.1, r.2, r.3)).collect();
    assert_eq!(ids.len(), rows.len());
    assert!(ids.iter().all(|i| i.starts_with("obl:sha256:")));
}

fn strict() -> Fixture {
    let mut f = Fixture::new("obl-strict");
    f.policy.require_derived_obligation = true;
    f
}

#[test]
fn strict_policy_refuses_hand_typed_obligation() {
    let f = strict();
    let e = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "100", Currency::USD, "inv-1", "invoice-payment", None).unwrap();
    assert_eq!(f.admit(e, "n1").unwrap_err(), OBLIGATION_ID_NOT_DERIVED);
}

#[test]
fn strict_policy_refuses_wrong_invoice_ref() {
    let f = strict();
    let wrong = derive_obligation_id(PAYER, PAYEE, "invoice-payment", "INV-1");
    let e = PaymentEffect::prepare_with_invoice(
        PRINCIPAL, PAYER, PAYEE, "100", Currency::USD, &wrong, "invoice-payment", Some("INV-2"), None,
    )
    .unwrap();
    assert_eq!(f.admit(e, "n1").unwrap_err(), OBLIGATION_ID_NOT_DERIVED);
    // derived id but no invoice_ref at all
    let e = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "100", Currency::USD, &wrong, "invoice-payment", None).unwrap();
    assert_eq!(f.admit(e, "n2").unwrap_err(), OBLIGATION_ID_NOT_DERIVED);
}

#[test]
fn strict_policy_admits_prepare_for_invoice() {
    let f = strict();
    let e = prepare_for_invoice(PRINCIPAL, PAYER, PAYEE, "100", Currency::USD, "invoice-payment", "INV-1", None).unwrap();
    let adm = f.admit(e, "n1").expect("derived obligation admits");
    assert_eq!(adm.effect().invoice_ref(), Some("INV-1"));
    assert_eq!(adm.effect().obligation_id(), derive_obligation_id(PAYER, PAYEE, "invoice-payment", "INV-1"));
    assert_eq!(adm.nonce(), "n1");
}

#[test]
fn default_policy_admits_legacy_effects_with_unchanged_digest() {
    let f = Fixture::new("obl-legacy");
    assert!(!f.policy.require_derived_obligation);
    let e = f.effect("100", "inv-legacy");
    assert!(e.payload.get("invoice_ref").is_none());
    assert!(f.admit(e, "n1").is_ok());
    // default policy also admits derived-obligation effects
    let e2 = prepare_for_invoice(PRINCIPAL, PAYER, PAYEE, "100", Currency::USD, "invoice-payment", "INV-9", None).unwrap();
    assert!(f.admit(e2, "n2").is_ok());
}
