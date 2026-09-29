//! SA2A C2/C3 cryptographic admission primitives.
//!
//! This module never signs certificates and never performs effects. It verifies
//! authority artifacts and resource bounds for callers that own those separate
//! trust domains.

pub mod algorithm;
pub mod certificate;
pub mod crypto;
pub mod encoding;
pub mod epoch;
pub mod error;
pub mod key_registry;
pub mod nonce;
pub mod prepared_effect;
pub mod principal;
pub mod quorum;
pub mod receipt;
pub mod resource;
pub mod signature;
pub mod verifier;

pub use algorithm::SignatureAlgorithm;
pub use certificate::ActuationCertificate;
pub use error::SecurityRefusal;
pub use key_registry::{KeyRecord, KeyRegistry, KeyState};
pub use prepared_effect::PreparedEffect;
pub use resource::{BudgetLedger, ResourceEnvelope};
pub use signature::CertificateSignature;
pub use verifier::{CertificateVerifier, VerificationReceipt};
