use serde::{Deserialize, Serialize};

use super::SignatureAlgorithm;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateSignature {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub signature: Vec<u8>,
}
