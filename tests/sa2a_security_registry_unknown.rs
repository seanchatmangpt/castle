use castle::sa2a_security::{KeyRegistry, SecurityRefusal};

#[test]
fn unknown_key_refuses() {
    let registry = KeyRegistry::default();
    assert_eq!(registry.resolve("missing", 10, 1), Err(SecurityRefusal::UnknownKey));
}
