use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey as Ed25519VerifyingKey};
use ml_dsa::{MlDsa65, Signature as MlDsaSignature, Verifier as _, VerifyingKey as MlDsaVerifyingKey};
use slh_dsa::{Shake128f, Signature as SlhDsaSignature, VerifyingKey as SlhDsaVerifyingKey};
use slh_dsa::signature::Verifier as _;

use super::{SecurityRefusal, SignatureAlgorithm};

pub fn verify_signature(
    algorithm: SignatureAlgorithm,
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), SecurityRefusal> {
    match algorithm {
        SignatureAlgorithm::Ed25519 => {
            let key_bytes: &[u8; 32] = public_key.try_into().map_err(|_| SecurityRefusal::InvalidKey)?;
            let key = Ed25519VerifyingKey::from_bytes(key_bytes).map_err(|_| SecurityRefusal::InvalidKey)?;
            let signature = Ed25519Signature::from_slice(signature).map_err(|_| SecurityRefusal::InvalidSignature)?;
            key.verify_strict(message, &signature).map_err(|_| SecurityRefusal::InvalidSignature)
        }
        SignatureAlgorithm::MlDsa65 => {
            let encoded_key: &ml_dsa::EncodedVerifyingKey<MlDsa65> =
                public_key.try_into().map_err(|_| SecurityRefusal::InvalidKey)?;
            let encoded_signature: &ml_dsa::EncodedSignature<MlDsa65> =
                signature.try_into().map_err(|_| SecurityRefusal::InvalidSignature)?;
            let key = MlDsaVerifyingKey::<MlDsa65>::decode(encoded_key);
            let signature = MlDsaSignature::<MlDsa65>::decode(encoded_signature)
                .ok_or(SecurityRefusal::InvalidSignature)?;
            key.verify(message, &signature).map_err(|_| SecurityRefusal::InvalidSignature)
        }
        SignatureAlgorithm::SlhDsaShake128f => {
            let key = SlhDsaVerifyingKey::<Shake128f>::try_from(public_key)
                .map_err(|_| SecurityRefusal::InvalidKey)?;
            let signature = SlhDsaSignature::<Shake128f>::try_from(signature)
                .map_err(|_| SecurityRefusal::InvalidSignature)?;
            key.verify(message, &signature).map_err(|_| SecurityRefusal::InvalidSignature)
        }
    }
}
