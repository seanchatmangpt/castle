use castle::sa2a_security::algorithm::SignatureAlgorithm;
use castle::sa2a_security::crypto::verify_signature;
use castle::sa2a_security::SecurityRefusal;

#[test]
fn malformed_ed25519_key_refuses() {
    assert_eq!(
        verify_signature(SignatureAlgorithm::Ed25519, &[0; 31], b"m", &[0; 64]),
        Err(SecurityRefusal::InvalidKey)
    );
}
