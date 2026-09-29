mod common;
use common::payments::*;

use castle::payments::experience::*;
use castle::payments::*;

async fn run(fx: &Fixture, amount: &str, obl: &str, nonce: &str) -> (PaymentEffect, PaymentExecution) {
    let prepared = fx.effect(amount, obl);
    let effect = PaymentEffect::from_prepared(prepared.clone()).unwrap();
    let admission = fx.admit(prepared, nonce).expect("admits");
    let exec = execute_payment(admission, &fx.exec_ctx()).await.expect("executes");
    (effect, exec)
}

fn eff(fx: &Fixture, amount: &str, obl: &str) -> PaymentEffect {
    PaymentEffect::from_prepared(fx.effect(amount, obl)).unwrap()
}

#[tokio::test]
async fn known_class_settles_again_with_no_intelligence_step() {
    let fx = Fixture::new("exp-known");
    let store = ExperienceStore::open(fx.dir.join("experience")).unwrap();

    let first = eff(&fx, "1000", "inv-a");
    let k1 = store.classify(&first).unwrap();
    assert_eq!(k1, Knowledge::Unknown);
    assert!(requires_intelligence(&k1));
    let (e1, x1) = run(&fx, "1000", "inv-a", "n-1").await;
    assert_eq!(x1.standing, PaymentStanding::Settled, "{}", x1.detail);
    store.record_settled(&e1, &x1).unwrap();

    let second = eff(&fx, "800", "inv-b");
    let k2 = store.classify(&second).unwrap();
    match &k2 {
        Knowledge::Known(r) => {
            assert_eq!(r.settled_count, 1);
            assert_eq!(r.per_effect_cap_minor, 1000);
            assert_eq!(r.class_key, PaymentClass::of(&second).key());
        }
        Knowledge::Unknown => panic!("expected Known"),
    }
    assert!(!requires_intelligence(&k2));
    let (e2, x2) = run(&fx, "800", "inv-b", "n-2").await;
    assert_eq!(x2.standing, PaymentStanding::Settled, "{}", x2.detail);
    store.record_settled(&e2, &x2).unwrap();
    assert_eq!(fx.ledger.entries().unwrap().len(), 2);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());

    match store.classify(&eff(&fx, "1", "inv-c")).unwrap() {
        Knowledge::Known(r) => {
            assert_eq!(r.settled_count, 2);
            assert_eq!(r.per_effect_cap_minor, 1000, "cap is max ever settled");
        }
        Knowledge::Unknown => panic!("expected Known"),
    }
}

#[tokio::test]
async fn larger_amount_than_ever_settled_is_unknown_again() {
    let fx = Fixture::new("exp-larger");
    let store = ExperienceStore::open(fx.dir.join("experience")).unwrap();
    let (e, x) = run(&fx, "1000", "inv-a", "n-1").await;
    store.record_settled(&e, &x).unwrap();
    let big = eff(&fx, "1001", "inv-big");
    let k = store.classify(&big).unwrap();
    assert_eq!(k, Knowledge::Unknown);
    assert!(requires_intelligence(&k));
    assert!(!requires_intelligence(&store.classify(&eff(&fx, "1000", "inv-eq")).unwrap()));
}

#[tokio::test]
async fn refused_execution_cannot_teach() {
    let fx = Fixture::with("exp-nsf", 50, 500_000, 1_000);
    let store = ExperienceStore::open(fx.dir.join("experience")).unwrap();
    let (e, x) = run(&fx, "100", "inv-nsf", "n-1").await;
    assert_eq!(x.standing, PaymentStanding::Refused);
    assert_eq!(store.record_settled(&e, &x).unwrap_err(), "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT");
    assert_eq!(store.classify(&e).unwrap(), Knowledge::Unknown);

    // A tampered Settled standing without receipt digests is also refused.
    let mut forged = x.clone();
    forged.standing = PaymentStanding::Settled;
    forged.construct_digest = None;
    forged.ocel_receipt_digest = None;
    assert_eq!(store.record_settled(&e, &forged).unwrap_err(), "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT");

    // UnknownOutcome standing is refused even with digests present.
    let (e2, x2) = {
        let fx2 = Fixture::new("exp-unk");
        let (e, mut x) = run(&fx2, "100", "inv-u", "n-1").await;
        x.standing = PaymentStanding::UnknownOutcome;
        (e, x)
    };
    assert_eq!(store.record_settled(&e2, &x2).unwrap_err(), "REFUSED:EXPERIENCE_REQUIRES_SETTLED_RECEIPT");
    assert_eq!(store.classify(&e2).unwrap(), Knowledge::Unknown);
}

#[tokio::test]
async fn store_survives_reopen() {
    let fx = Fixture::new("exp-reopen");
    let dir = fx.dir.join("experience");
    let store = ExperienceStore::open(&dir).unwrap();
    let (e, x) = run(&fx, "500", "inv-a", "n-1").await;
    store.record_settled(&e, &x).unwrap();
    drop(store);
    let reopened = ExperienceStore::open(&dir).unwrap();
    match reopened.classify(&eff(&fx, "500", "inv-z")).unwrap() {
        Knowledge::Known(r) => {
            assert_eq!(r.settled_count, 1);
            assert_eq!(Some(&r.last_construct_digest), x.construct_digest.as_ref());
            assert_eq!(Some(&r.last_ocel_receipt_digest), x.ocel_receipt_digest.as_ref());
        }
        Knowledge::Unknown => panic!("expected Known after reopen"),
    }
}

#[test]
fn experience_source_is_model_free() {
    let path = format!("{}/src/payments/experience.rs", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).unwrap().to_lowercase();
    for banned in ["anthropic", "openai", "llm(", "model("] {
        assert!(!text.contains(banned), "experience.rs contains {banned}");
    }
}
