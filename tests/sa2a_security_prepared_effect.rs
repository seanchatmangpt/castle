use castle::sa2a_security::PreparedEffect;

fn effect(payload: &str) -> PreparedEffect {
    PreparedEffect {
        version: 1,
        principal: "principal:alice".into(),
        effect_kind: "http".into(),
        resource: "https://example.invalid/orders/42".into(),
        operation: "POST".into(),
        payload_digest: payload.into(),
        idempotency_key: "order-42".into(),
    }
}

#[test]
fn portable_digest_is_stable_and_content_bound() {
    let a = effect("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let b = a.clone();
    assert_eq!(a.digest().unwrap(), b.digest().unwrap());
    let c = effect("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    assert_ne!(a.digest().unwrap(), c.digest().unwrap());
}
