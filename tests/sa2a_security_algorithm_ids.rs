use castle::sa2a_security::SignatureAlgorithm;

#[test]
fn algorithm_ids_are_wire_stable() {
    assert_eq!(SignatureAlgorithm::Ed25519.id(), "ed25519");
    assert_eq!(SignatureAlgorithm::MlDsa65.id(), "ml-dsa-65");
    assert_eq!(SignatureAlgorithm::SlhDsaShake128f.id(), "slh-dsa-shake-128f");
}
