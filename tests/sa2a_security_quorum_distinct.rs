use castle::sa2a_security::quorum::admit_distinct_quorum;
use castle::sa2a_security::{CertificateSignature, KeyRecord, KeyState, SignatureAlgorithm};

fn pair(key: &str, custodian: &str) -> (CertificateSignature, KeyRecord) {
    (
        CertificateSignature { key_id: key.into(), algorithm: SignatureAlgorithm::Ed25519, signature: vec![0; 64] },
        KeyRecord {
            key_id: key.into(),
            custodian_id: custodian.into(),
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
fn two_distinct_custodians_satisfy_two_of_n() {
    let admitted = admit_distinct_quorum(2, &[pair("k1", "device:a"), pair("k2", "device:b")]).unwrap();
    assert_eq!(admitted.len(), 2);
}
