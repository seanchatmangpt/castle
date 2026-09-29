use super::SecurityRefusal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityEpochs {
    pub policy: u64,
    pub revocation: u64,
    pub generation: u64,
}

pub fn admit_epochs(expected: SecurityEpochs, certificate: SecurityEpochs) -> Result<(), SecurityRefusal> {
    if certificate.policy != expected.policy {
        return Err(SecurityRefusal::PolicyEpochMismatch);
    }
    if certificate.revocation != expected.revocation {
        return Err(SecurityRefusal::RevocationEpochMismatch);
    }
    if certificate.generation != expected.generation {
        return Err(SecurityRefusal::GenerationMismatch);
    }
    Ok(())
}
