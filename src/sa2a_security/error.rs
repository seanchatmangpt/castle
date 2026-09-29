use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityRefusal {
    InvalidDigest,
    InvalidKey,
    InvalidSignature,
    AlgorithmMismatch,
    UnknownKey,
    KeyRevoked,
    KeyNotYetValid,
    KeyExpired,
    PolicyEpochMismatch,
    RevocationEpochMismatch,
    GenerationMismatch,
    PrincipalMismatch,
    AudienceMismatch,
    CertificateExpired,
    CertificateNotYetValid,
    EffectDigestMismatch,
    DuplicateSigner,
    InsufficientQuorum,
    InsufficientCustodianIndependence,
    NonceReplay,
    ResourceAmplification,
    InvalidResourceEnvelope,
}

impl fmt::Display for SecurityRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "REFUSED:{self:?}")
    }
}

impl std::error::Error for SecurityRefusal {}
