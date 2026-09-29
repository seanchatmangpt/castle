mod common;
use common::payments::*;

use castle::payments::*;
use castle::sa2a_security::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyRegistry, KeyState, PreparedEffect, SignatureAlgorithm,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;

/// Custodian seeds mirror `Fixture::with` (mac=11, phone=12, hsm=13).
fn seed(name: &str) -> SigningKey {
    let i = ["mac", "phone", "hsm"].iter().position(|n| *n == name).unwrap();
    SigningKey::from_bytes(&[(i as u8) + 11; 32])
}

/// Build, mutate, then sign (so mutated fields are covered by valid signatures).
fn cert_with(
    effect: &PreparedEffect,
    nonce: &str,
    signers: &[&str],
    mutate: impl FnOnce(&mut ActuationCertificate),
) -> ActuationCertificate {
    let mut c = ActuationCertificate {
        version: 1,
        effect_digest: effect.digest().unwrap(),
        principal: effect.principal.clone(),
        policy_epoch: 7,
        revocation_epoch: 0,
        generation: 1,
        nonce: nonce.to_string(),
        not_before_ms: 0,
        expires_at_ms: 100_000,
        audience: AUDIENCE.to_string(),
        threshold: 2,
        signatures: vec![],
    };
    mutate(&mut c);
    let msg = c.signing_message().unwrap();
    for n in signers {
        c.signatures.push(CertificateSignature {
            key_id: format!("key-{n}"),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: seed(n).sign(&msg).to_bytes().to_vec(),
        });
    }
    c
}

fn admit_with(fx: &Fixture, e: PreparedEffect, c: &ActuationCertificate) -> Result<PaymentAdmission, String> {
    admit_payment(e, c, &fx.admission_ctx())
}

struct Case {
    name: &'static str,
    expect: &'static str,
    run: fn() -> (Fixture, Result<PaymentAdmission, String>, Vec<String>),
}

fn digest(e: &PreparedEffect) -> String {
    e.digest().unwrap()
}

fn prep(payer: &str, payee: &str, amount: &str, cur: Currency, obl: &str) -> PreparedEffect {
    PaymentEffect::prepare(PRINCIPAL, payer, payee, amount, cur, obl, "invoice-payment", None).unwrap()
}

fn cases() -> Vec<Case> {
    vec![
        Case { name: "tampered_amount_after_signing", expect: "REFUSED:EffectDigestMismatch", run: || {
            let fx = Fixture::new("tamper");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |_| {});
            let mut t = e.clone();
            t.payload["amount"] = json!("101");
            let r = admit_with(&fx, t.clone(), &c);
            (fx, r, vec![digest(&e), digest(&t)])
        }},
        Case { name: "one_signer_threshold_two", expect: "REFUSED:InsufficientQuorum", run: || {
            let fx = Fixture::new("quorum1");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac"], |_| {});
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "same_custodian_two_keys", expect: "REFUSED:InsufficientCustodianIndependence", run: || {
            let fx = Fixture::new("custodian");
            let reg = KeyRegistry::from_records(["mac", "phone"].iter().map(|n| KeyRecord {
                key_id: format!("key-{n}"),
                custodian_id: "device:shared".into(),
                algorithm: SignatureAlgorithm::Ed25519,
                public_key: seed(n).verifying_key().to_bytes().to_vec(),
                state: KeyState::Active,
                not_before_ms: 0,
                expires_at_ms: 10_000_000,
                revocation_epoch: 0,
            }));
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |_| {});
            let mut ctx = fx.admission_ctx();
            ctx.registry = &reg;
            let r = admit_payment(e.clone(), &c, &ctx);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "duplicate_signer", expect: "REFUSED:DuplicateSigner", run: || {
            let fx = Fixture::new("dup");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "mac"], |_| {});
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "unknown_signer_key", expect: "REFUSED:UnknownKey", run: || {
            let fx = Fixture::new("unkkey");
            let e = fx.effect("100", "o1");
            let mut c = cert_with(&e, "n1", &["mac"], |_| {});
            let rogue = SigningKey::from_bytes(&[77; 32]);
            c.signatures.push(CertificateSignature {
                key_id: "key-rogue".into(),
                algorithm: SignatureAlgorithm::Ed25519,
                signature: rogue.sign(&c.signing_message().unwrap()).to_bytes().to_vec(),
            });
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "wrong_audience", expect: "REFUSED:AudienceMismatch", run: || {
            let fx = Fixture::new("aud");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |c| c.audience = "actuator:other".into());
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "expired_certificate_validly_signed", expect: "REFUSED:CertificateExpired", run: || {
            let fx = Fixture::new("expired");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |c| c.expires_at_ms = NOW_MS);
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "not_yet_valid_certificate", expect: "REFUSED:CertificateNotYetValid", run: || {
            let fx = Fixture::new("nyv");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |c| c.not_before_ms = NOW_MS + 1);
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "nonce_replayed_for_different_effect", expect: "REFUSED:NonceReplay", run: || {
            let fx = Fixture::new("replay");
            let a = fx.effect("100", "o1");
            fx.admit(a, "n-shared").expect("first admits");
            let b = fx.effect("200", "o2");
            let r = fx.admit(b.clone(), "n-shared");
            (fx, r, vec![digest(&b)])
        }},
        Case { name: "no_policy_for_principal", expect: "REFUSED:NO_SPEND_POLICY_FOR_PRINCIPAL", run: || {
            let fx = Fixture::new("nopol");
            let e = PaymentEffect::prepare("principal:stranger", PAYER, PAYEE, "100", Currency::USD, "o1", "p", None).unwrap();
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "payee_not_allow_listed", expect: "REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED", run: || {
            let fx = Fixture::new("payee");
            let e = prep(PAYER, "acct:stranger", "100", Currency::USD, "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "payer_not_allow_listed", expect: "REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED", run: || {
            let fx = Fixture::new("payer");
            let e = prep("acct:stranger", PAYEE, "100", Currency::USD, "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "currency_without_cap", expect: "REFUSED:PAYMENT_CURRENCY_UNSUPPORTED", run: || {
            let fx = Fixture::new("cur");
            let e = prep(PAYER, PAYEE, "100", Currency::EUR, "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "amount_over_per_effect_cap", expect: "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP", run: || {
            let fx = Fixture::new("cap");
            let e = fx.effect("500001", "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "epoch_budget_exhausted_by_second_admit", expect: "REFUSED:PAYMENT_BUDGET_EXCEEDED", run: || {
            let fx = Fixture::with("budget", 1_000_000, 500_000, 600_000);
            fx.admit(fx.effect("400000", "o1"), "n1").expect("first fits");
            let b = fx.effect("300000", "o2");
            let r = fx.admit(b.clone(), "n2");
            (fx, r, vec![digest(&b)])
        }},
        Case { name: "payer_equals_payee", expect: "REFUSED:PAYMENT_SAME_ACCOUNT", run: || {
            let fx = Fixture::new("same");
            let e = prep(PAYER, PAYER, "100", Currency::USD, "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "wrong_capability", expect: "REFUSED:PAYMENT_CAPABILITY_NOT_ADMITTED", run: || {
            let fx = Fixture::new("capab");
            let mut e = fx.effect("100", "o1");
            e.capability = "payments.refund.v9".into();
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "unknown_payload_field", expect: "REFUSED:PAYMENT_PAYLOAD_INVALID", run: || {
            let fx = Fixture::new("plfield");
            let mut e = fx.effect("100", "o1");
            e.payload["sneaky"] = json!("route-to-me");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "unknown_subject_field", expect: "REFUSED:PAYMENT_PAYLOAD_INVALID", run: || {
            let fx = Fixture::new("subfield");
            let mut e = fx.effect("100", "o1");
            e.subject["also_pay"] = json!("acct:evil");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "zero_amount", expect: "REFUSED:PAYMENT_AMOUNT_ZERO", run: || {
            let fx = Fixture::new("zero");
            let e = fx.effect("0", "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "non_integer_amount", expect: "REFUSED:PAYMENT_AMOUNT_NOT_INTEGER", run: || {
            let fx = Fixture::new("float");
            let e = fx.effect("1.5", "o1");
            let r = fx.admit(e.clone(), "n1");
            (fx, r, vec![digest(&e)])
        }},
        Case { name: "alice_approval_cannot_authorize_bob", expect: "REFUSED:EffectDigestMismatch", run: || {
            let fx = Fixture::new("alicebob");
            let alice = prep(PAYER, PAYEE, "100", Currency::USD, "o1");
            let bob = prep(PAYEE, PAYER, "100", Currency::USD, "o1");
            let c = cert_with(&alice, "n1", &["mac", "phone"], |_| {});
            let r = admit_with(&fx, bob.clone(), &c);
            (fx, r, vec![digest(&alice), digest(&bob)])
        }},
        Case { name: "certificate_principal_differs_from_effect", expect: "REFUSED:PrincipalMismatch", run: || {
            let fx = Fixture::new("princ");
            let e = fx.effect("100", "o1");
            let c = cert_with(&e, "n1", &["mac", "phone"], |c| c.principal = "principal:other".into());
            let r = admit_with(&fx, e.clone(), &c);
            (fx, r, vec![digest(&e)])
        }},
    ]
}

#[test]
fn every_refusal_is_exact_and_leaves_no_ledger_entry_or_claim() {
    for case in cases() {
        let (fx, result, absent) = (case.run)();
        assert_eq!(result.as_ref().err().map(String::as_str), Some(case.expect), "case {}", case.name);
        assert!(fx.ledger.entries().unwrap().is_empty(), "case {}: ledger must be empty", case.name);
        for d in absent {
            assert!(fx.claims.get(&d).unwrap().is_none(), "case {}: refused digest {d} must have no claim", case.name);
        }
    }
}

#[test]
fn positive_control_valid_admission_reserves_a_claim_and_touches_no_ledger() {
    let fx = Fixture::new("control");
    let e = fx.effect("100", "o1");
    let a = fx.admit(e, "n1").expect("admits");
    let claim = fx.claims.get(a.effect().digest()).unwrap().expect("claim reserved");
    assert_eq!(claim.state, ClaimState::Reserved);
    assert_eq!(claim.amount_minor, 100);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[test]
fn budget_refusal_leaves_first_claim_intact_and_second_absent() {
    let fx = Fixture::with("budget2", 1_000_000, 500_000, 600_000);
    let a = fx.admit(fx.effect("400000", "o1"), "n1").unwrap();
    let b = fx.effect("300000", "o2");
    assert_eq!(fx.admit(b.clone(), "n2").unwrap_err(), "REFUSED:PAYMENT_BUDGET_EXCEEDED");
    assert_eq!(fx.claims.get(a.effect().digest()).unwrap().unwrap().state, ClaimState::Reserved);
    assert!(fx.claims.get(&digest(&b)).unwrap().is_none());
    assert_eq!(fx.claims.list().unwrap().len(), 1);
    // A smaller effect that still fits is admitted: the refusal did not poison the budget.
    assert!(fx.admit(fx.effect("200000", "o3"), "n3").is_ok());
}

#[test]
fn replayed_certificate_for_same_effect_is_refused_by_nonce_not_by_claim() {
    let fx = Fixture::new("replaysame");
    let e = fx.effect("100", "o1");
    let c = cert_with(&e, "n1", &["mac", "phone"], |_| {});
    admit_with(&fx, e.clone(), &c).expect("first");
    assert_eq!(admit_with(&fx, e, &c).unwrap_err(), "REFUSED:NonceReplay");
    assert_eq!(fx.claims.list().unwrap().len(), 1);
}
