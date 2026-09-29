use castle::sa2a_security::ActuationCertificate;

fn cert(effect: &str) -> ActuationCertificate {
    ActuationCertificate {
        version: 1,
        effect_digest: effect.into(),
        principal: "principal:alice".into(),
        policy_epoch: 7,
        revocation_epoch: 9,
        generation: 11,
        nonce: "nonce-1".into(),
        not_before_ms: 100,
        expires_at_ms: 200,
        audience: "actuator:payments".into(),
        threshold: 2,
        signatures: vec![],
    }
}

#[test]
fn signed_message_binds_exact_effect() {
    let a = cert("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let b = cert("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    assert_ne!(a.signing_message().unwrap(), b.signing_message().unwrap());
}
