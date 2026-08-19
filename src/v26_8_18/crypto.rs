use std::collections::BTreeSet;

use ml_dsa::Keypair;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PqcSignatureProof {
    pub suite: SignatureSuite,
    pub parameter_set: String,
    pub message_blake3: String,
    pub public_key_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PqcRuntimeQualification {
    pub standing: ReleaseStanding,
    pub ml_dsa_65: bool,
    pub slh_dsa_shake_128f: bool,
    pub reasons: Vec<String>,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hex_decode(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("REFUSED:INVALID_HEX".to_string());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| "REFUSED:INVALID_HEX".to_string())
        })
        .collect()
}

fn derive_slh_seed(seed: &[u8; 32], label: &[u8]) -> [u8; 16] {
    let mut input = Vec::with_capacity(seed.len() + label.len());
    input.extend_from_slice(seed);
    input.extend_from_slice(label);
    let digest = blake3::hash(&input);
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest.as_bytes()[..16]);
    out
}

/// Sign an exact byte sequence with the shipped FIPS-204 or FIPS-205 runtime.
/// The seed is caller-supplied so deterministic replay can bind the key source;
/// CASTLE never infers a production trust root from this helper.
pub fn sign_pqc_message(
    suite: SignatureSuite,
    seed: [u8; 32],
    message: &[u8],
) -> Result<PqcSignatureProof, String> {
    let message_blake3 = blake3::hash(message).to_hex().to_string();
    match suite {
        SignatureSuite::MlDsa => {
            use ml_dsa::{Keypair, MlDsa65, Signer, SigningKey};

            let mut ml_seed = ml_dsa::Seed::default();
            ml_seed.as_mut_slice().copy_from_slice(&seed);
            let signing_key = SigningKey::<MlDsa65>::from_seed(&ml_seed);
            let signature = signing_key.sign(message);
            let verifying_key = signing_key.verifying_key();
            Ok(PqcSignatureProof {
                suite,
                parameter_set: "ML-DSA-65".to_string(),
                message_blake3,
                public_key_hex: hex_encode(verifying_key.encode().as_slice()),
                signature_hex: hex_encode(signature.encode().as_slice()),
            })
        }
        SignatureSuite::SlhDsa => {
            use slh_dsa::signature::Signer;
            use slh_dsa::{Shake128f, SigningKey};

            let sk_seed = derive_slh_seed(&seed, b"castle:slh:sk-seed");
            let sk_prf = derive_slh_seed(&seed, b"castle:slh:sk-prf");
            let pk_seed = derive_slh_seed(&seed, b"castle:slh:pk-seed");
            let signing_key = SigningKey::<Shake128f>::slh_keygen_internal(&sk_seed, &sk_prf, &pk_seed);
            let signature = signing_key
                .try_sign(message)
                .map_err(|_| "REFUSED:SLH_DSA_SIGN_FAILED".to_string())?;
            Ok(PqcSignatureProof {
                suite,
                parameter_set: "SLH-DSA-SHAKE-128f".to_string(),
                message_blake3,
                public_key_hex: hex_encode(signing_key.verifying_key().to_bytes().as_slice()),
                signature_hex: hex_encode(signature.to_bytes().as_slice()),
            })
        }
        _ => Err(format!("UNSUPPORTED:PQC_SIGNER_NOT_REQUESTED:{}", suite.as_str())),
    }
}

#[must_use]
pub fn verify_pqc_message(proof: &PqcSignatureProof, message: &[u8]) -> bool {
    if proof.message_blake3 != blake3::hash(message).to_hex().to_string() {
        return false;
    }
    let Ok(public_key) = hex_decode(&proof.public_key_hex) else {
        return false;
    };
    let Ok(signature) = hex_decode(&proof.signature_hex) else {
        return false;
    };

    match proof.suite {
        SignatureSuite::MlDsa => {
            use ml_dsa::{MlDsa65, Verifier};
            let Ok(encoded_key) = ml_dsa::EncodedVerifyingKey::<MlDsa65>::try_from(public_key.as_slice()) else {
                return false;
            };
            let verifying_key = ml_dsa::VerifyingKey::<MlDsa65>::decode(&encoded_key);
            let Ok(signature) = ml_dsa::Signature::<MlDsa65>::try_from(signature.as_slice()) else {
                return false;
            };
            verifying_key.verify(message, &signature).is_ok()
        }
        SignatureSuite::SlhDsa => {
            use slh_dsa::signature::Verifier;
            use slh_dsa::{Shake128f, Signature, VerifyingKey};
            let Ok(verifying_key) = VerifyingKey::<Shake128f>::try_from(public_key.as_slice()) else {
                return false;
            };
            let Ok(signature) = Signature::<Shake128f>::try_from(signature.as_slice()) else {
                return false;
            };
            verifying_key.verify(message, &signature).is_ok()
        }
        _ => false,
    }
}

#[must_use]
pub fn qualify_pqc_runtime() -> PqcRuntimeQualification {
    let seed = [0x42u8; 32];
    let message = b"CASTLE:PQC:RUNTIME:SELF-TEST";
    let ml_dsa_65 = sign_pqc_message(SignatureSuite::MlDsa, seed, message)
        .map(|proof| verify_pqc_message(&proof, message))
        .unwrap_or(false);
    let slh_dsa_shake_128f = sign_pqc_message(SignatureSuite::SlhDsa, seed, message)
        .map(|proof| verify_pqc_message(&proof, message))
        .unwrap_or(false);
    let mut reasons = Vec::new();
    if !ml_dsa_65 {
        reasons.push("BUILD_BROKEN:ML_DSA_65_SELF_TEST_FAILED".to_string());
    }
    if !slh_dsa_shake_128f {
        reasons.push("BUILD_BROKEN:SLH_DSA_SHAKE_128F_SELF_TEST_FAILED".to_string());
    }
    PqcRuntimeQualification {
        standing: if reasons.is_empty() { ReleaseStanding::Alive } else { ReleaseStanding::BuildBroken },
        ml_dsa_65,
        slh_dsa_shake_128f,
        reasons,
    }
}

#[must_use]
pub fn implemented_signature_suites() -> BTreeSet<SignatureSuite> {
    BTreeSet::from([SignatureSuite::Ed25519, SignatureSuite::MlDsa, SignatureSuite::SlhDsa])
}

/// CASTLE identity remains BLAKE3-first while also emitting SHA-256 for
/// enterprise interoperability. Signature suites are capability-admitted;
/// naming a suite never implies implementation. PQC standing is granted only
/// when an accepted shipped suite exists; `qualify_pqc_runtime` separately
/// executes the real implementations as a self-test.
#[must_use]
pub fn qualify_crypto_profile(profile: &CryptoProfile) -> CryptoQualification {
    let implemented = implemented_signature_suites();
    let implemented_pqc = BTreeSet::from([SignatureSuite::MlDsa, SignatureSuite::SlhDsa]);
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
    if profile.require_post_quantum && profile.accepted_signature_suites.is_disjoint(&implemented_pqc) {
        reasons.push("REFUSED:PQC_REQUIRED_BUT_NO_IMPLEMENTED_PQC_SUITE_ACCEPTED".to_string());
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
