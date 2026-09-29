use super::SecurityRefusal;

pub fn preserve_principal(expected: &str, observed: &str) -> Result<(), SecurityRefusal> {
    if expected.is_empty() || expected != observed {
        return Err(SecurityRefusal::PrincipalMismatch);
    }
    Ok(())
}
