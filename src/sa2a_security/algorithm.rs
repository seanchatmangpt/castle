use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignatureAlgorithm {
    Ed25519,
    MlDsa65,
    SlhDsaShake128f,
}

impl SignatureAlgorithm {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::MlDsa65 => "ml-dsa-65",
            Self::SlhDsaShake128f => "slh-dsa-shake-128f",
        }
    }
}
