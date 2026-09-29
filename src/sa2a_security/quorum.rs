use std::collections::BTreeSet;

use super::{CertificateSignature, KeyRecord, SecurityRefusal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSigner {
    pub key_id: String,
    pub custodian_id: String,
}

pub fn admit_distinct_quorum(
    threshold: u16,
    verified: &[(CertificateSignature, KeyRecord)],
) -> Result<Vec<VerifiedSigner>, SecurityRefusal> {
    let mut keys = BTreeSet::new();
    let mut custodians = BTreeSet::new();
    let mut admitted = Vec::new();

    for (signature, key) in verified {
        if !keys.insert(signature.key_id.clone()) {
            return Err(SecurityRefusal::DuplicateSigner);
        }
        if !custodians.insert(key.custodian_id.clone()) {
            continue;
        }
        admitted.push(VerifiedSigner {
            key_id: signature.key_id.clone(),
            custodian_id: key.custodian_id.clone(),
        });
    }

    if admitted.len() < usize::from(threshold) {
        return Err(SecurityRefusal::InsufficientCustodianIndependence);
    }
    Ok(admitted)
}
