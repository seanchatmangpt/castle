use castle::sa2a_security::{KeyRecord, KeyRegistry, KeyState, SecurityRefusal, SignatureAlgorithm};

#[test]
fn revoked_key_refuses() {
    let registry = KeyRegistry::from_records([KeyRecord {
        key_id: "k1".into(),
        custodian_id: "device:a".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        public_key: vec![1; 32],
        state: KeyState::Revoked,
        not_before_ms: 0,
        expires_at_ms: 100,
        revocation_epoch: 1,
    }]);
    assert_eq!(registry.resolve("k1", 10, 1), Err(SecurityRefusal::KeyRevoked));
}
