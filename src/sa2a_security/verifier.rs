use super::crypto::verify_signature;
use super::epoch::{admit_epochs, SecurityEpochs};
use super::principal::preserve_principal;
use super::quorum::admit_distinct_quorum;
use super::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyRegistry, PreparedEffect, SecurityRefusal,
};
pub use super::receipt::VerificationReceipt;

pub struct CertificateVerifier<'a> {
    pub registry: &'a KeyRegistry,
    pub expected_epochs: SecurityEpochs,
    pub expected_audience: &'a str,
    pub now_ms: u64,
}

impl<'a> CertificateVerifier<'a> {
    pub fn verify(
        &self,
        effect: &PreparedEffect,
        certificate: &ActuationCertificate,
    ) -> Result<VerificationReceipt, SecurityRefusal> {
        let effect_digest = effect.digest()?;
        if certificate.effect_digest != effect_digest {
            return Err(SecurityRefusal::EffectDigestMismatch);
        }
        preserve_principal(&effect.principal, &certificate.principal)?;
        if certificate.audience != self.expected_audience {
            return Err(SecurityRefusal::AudienceMismatch);
        }
        if self.now_ms < certificate.not_before_ms {
            return Err(SecurityRefusal::CertificateNotYetValid);
        }
        if self.now_ms >= certificate.expires_at_ms {
            return Err(SecurityRefusal::CertificateExpired);
        }
        admit_epochs(
            self.expected_epochs,
            SecurityEpochs {
                policy: certificate.policy_epoch,
                revocation: certificate.revocation_epoch,
                generation: certificate.generation,
            },
        )?;

        let message = certificate.signing_message()?;
        let mut verified: Vec<(CertificateSignature, KeyRecord)> = Vec::new();

        for signature in &certificate.signatures {
            let key = self
                .registry
                .resolve(&signature.key_id, self.now_ms, certificate.revocation_epoch)?;
            if key.algorithm != signature.algorithm {
                return Err(SecurityRefusal::AlgorithmMismatch);
            }
            verify_signature(key.algorithm, &key.public_key, &message, &signature.signature)?;
            verified.push((signature.clone(), key.clone()));
        }

        if verified.len() < usize::from(certificate.threshold) {
            return Err(SecurityRefusal::InsufficientQuorum);
        }
        let signers = admit_distinct_quorum(certificate.threshold, &verified)?;
        Ok(VerificationReceipt::from_signers(
            certificate.effect_digest.clone(),
            certificate.principal.clone(),
            certificate.policy_epoch,
            certificate.revocation_epoch,
            certificate.generation,
            certificate.audience.clone(),
            &signers,
        ))
    }
}
