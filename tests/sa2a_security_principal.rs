use castle::sa2a_security::principal::preserve_principal;
use castle::sa2a_security::SecurityRefusal;

#[test]
fn confused_deputy_principal_substitution_refuses() {
    assert_eq!(
        preserve_principal("principal:alice", "principal:service"),
        Err(SecurityRefusal::PrincipalMismatch)
    );
}
