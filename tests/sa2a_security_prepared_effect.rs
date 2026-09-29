use castle::sa2a_security::PreparedEffect;
use serde_json::json;

fn effect(subject: serde_json::Value, payload: serde_json::Value) -> PreparedEffect {
    PreparedEffect {
        version: 1,
        principal: "principal:alice".into(),
        capability: "payments".into(),
        subject,
        payload,
    }
}

#[test]
fn portable_digest_is_stable_and_content_bound() {
    let a = effect(json!({"kind":"order","id":42}), json!({"amount":100,"currency":"USD"}));
    let b = effect(json!({"id":42,"kind":"order"}), json!({"currency":"USD","amount":100}));
    assert_eq!(a.digest().unwrap(), b.digest().unwrap());

    let changed = effect(json!({"kind":"order","id":43}), json!({"amount":100,"currency":"USD"}));
    assert_ne!(a.digest().unwrap(), changed.digest().unwrap());
}
