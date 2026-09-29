use castle::sa2a_security::quorum::admit_distinct_quorum;
use castle::sa2a_security::{CertificateSignature, KeyRecord, KeyState, SecurityRefusal, SignatureAlgorithm};

fn pair(key: &str) -> (CertificateSignature, KeyRecord) {
    (
        CertificateSignature { key_id: key.into(), algorithm: SignatureAlgorithm::Ed25519, signature: vec![0; 64] },
        KeyRecord {
            key_id: key.into(),
            custodian_id: "same-device".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            public_key: vec![1; 32],
            state: KeyState::Active,
            not_before_ms: 0,
            expires_at_ms: 100,
            revocation_epoch: 0,
        },
    )
}

#[test]
fn labels_do_not_fake_independence() {
    assert_eq!(
        admit_distinct_quorum(2, &[pair("k1"), pair("k2")]),
        Err(SecurityRefusal::InsufficientCustodianIndependence)
    );
}
