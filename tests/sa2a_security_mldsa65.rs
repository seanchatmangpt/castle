use castle::sa2a_security::algorithm::SignatureAlgorithm;
use castle::sa2a_security::crypto::verify_signature;
use castle::sa2a_security::SecurityRefusal;

#[test]
fn malformed_ml_dsa_65_key_refuses() {
    assert_eq!(
        verify_signature(SignatureAlgorithm::MlDsa65, &[0; 16], b"m", &[0; 16]),
        Err(SecurityRefusal::InvalidKey)
    );
}
