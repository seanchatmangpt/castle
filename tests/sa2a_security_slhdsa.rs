use castle::sa2a_security::algorithm::SignatureAlgorithm;
use castle::sa2a_security::crypto::verify_signature;
use castle::sa2a_security::SecurityRefusal;

#[test]
fn malformed_slh_dsa_key_refuses() {
    assert_eq!(
        verify_signature(SignatureAlgorithm::SlhDsaShake128f, &[0; 16], b"m", &[0; 16]),
        Err(SecurityRefusal::InvalidKey)
    );
}
