use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::ReleaseStanding;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub blake3_256: String,
    pub sha256: String,
}

#[must_use]
pub fn dual_artifact_identity(bytes: &[u8]) -> ArtifactIdentity {
    let blake3_256 = blake3::hash(bytes).to_hex().to_string();
    let mut sha = Sha256::new();
    sha.update(bytes);
    let sha256 = format!("{:x}", sha.finalize());
    ArtifactIdentity { blake3_256, sha256 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureSuite {
    Ed25519,
    EcdsaP256,
    RsaPssSha256,
    MlDsa,
    SlhDsa,
}

impl SignatureSuite {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::EcdsaP256 => "ecdsa-p256",
            Self::RsaPssSha256 => "rsa-pss-sha256",
            Self::MlDsa => "ml-dsa",
            Self::SlhDsa => "slh-dsa",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryptoProfile {
    pub required_identity_hashes: BTreeSet<String>,
    pub accepted_signature_suites: BTreeSet<SignatureSuite>,
    pub require_post_quantum: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CryptoQualification {
    pub standing: ReleaseStanding,
    pub reasons: Vec<String>,
    pub implemented_signature_suites: BTreeSet<SignatureSuite>,
}

#[must_use]
pub fn implemented_signature_suites() -> BTreeSet<SignatureSuite> {
    BTreeSet::from([SignatureSuite::Ed25519])
}

/// CASTLE identity remains BLAKE3-first while also emitting SHA-256 for
/// enterprise interoperability. Signature suites are capability-admitted;
/// naming a suite never implies implementation. v26.8.18 implements Ed25519
/// and explicitly reports PQC suites as UNSUPPORTED rather than fabricating
/// standing.
#[must_use]
pub fn qualify_crypto_profile(profile: &CryptoProfile) -> CryptoQualification {
    let implemented = implemented_signature_suites();
    let mut reasons = Vec::new();
    if !profile.required_identity_hashes.contains("blake3-256") {
        reasons.push("REFUSED:MISSING_CANONICAL_BLAKE3_IDENTITY".to_string());
    }
    if !profile.required_identity_hashes.contains("sha256") {
        reasons.push("REFUSED:MISSING_ENTERPRISE_SHA256_IDENTITY".to_string());
    }
    if profile.accepted_signature_suites.is_disjoint(&implemented) {
        reasons.push("UNSUPPORTED:NO_IMPLEMENTED_ACCEPTED_SIGNATURE_SUITE".to_string());
    }
    if profile.require_post_quantum
        && !profile.accepted_signature_suites.contains(&SignatureSuite::MlDsa)
        && !profile.accepted_signature_suites.contains(&SignatureSuite::SlhDsa)
    {
        reasons.push("REFUSED:PQC_REQUIRED_BUT_NOT_ACCEPTED".to_string());
    }
    if profile.require_post_quantum {
        reasons.push("UNSUPPORTED:PQC_SIGNATURE_RUNTIME_NOT_SHIPPED_IN_V26_8_18".to_string());
    }

    let standing = if reasons.iter().any(|reason| reason.starts_with("REFUSED:")) {
        ReleaseStanding::Refused
    } else if reasons.iter().any(|reason| reason.starts_with("UNSUPPORTED:")) {
        ReleaseStanding::Unsupported
    } else {
        ReleaseStanding::Alive
    };
    CryptoQualification { standing, reasons, implemented_signature_suites: implemented }
}
