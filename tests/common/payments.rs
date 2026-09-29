//! Shared real-collaborator fixture for payments tests: real BLAKE3, real
//! Ed25519 receipt keys, real Ed25519 authority custodians, real fsynced
//! stores in unique temp dirs. No mocks.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use castle::castle::{Blake3Provider, ReceiptSigner, ReceiptVerifier};
use castle::payments::*;
use castle::sa2a_security::epoch::SecurityEpochs;
use castle::sa2a_security::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyRegistry, KeyState, PreparedEffect, SignatureAlgorithm,
};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

pub struct RealBlake3;
impl Blake3Provider for RealBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

pub struct ReceiptKey {
    id: String,
    key: SigningKey,
}
impl ReceiptSigner for ReceiptKey {
    fn key_id(&self) -> &str {
        &self.id
    }
    fn sign_digest(&self, digest_hex: &str) -> String {
        self.key.sign(&unhex(digest_hex)).to_bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}
pub struct ReceiptCheck {
    id: String,
    key: VerifyingKey,
}
impl ReceiptVerifier for ReceiptCheck {
    fn verify_digest(&self, key_id: &str, digest_hex: &str, signature: &str) -> bool {
        if key_id != self.id {
            return false;
        }
        let sig = unhex(signature);
        let Ok(arr): Result<[u8; 64], _> = sig.try_into() else { return false };
        self.key.verify(&unhex(digest_hex), &ed25519_dalek::Signature::from_bytes(&arr)).is_ok()
    }
}

pub const AUDIENCE: &str = "actuator:payments";
pub const PRINCIPAL: &str = "principal:procurement-agent";
pub const PAYER: &str = "acct:treasury";
pub const PAYEE: &str = "acct:supplier-9821";
pub const NOW_MS: u64 = 1_000;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn unique_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("castle-pay-{}-{}-{n}-{tag}", std::process::id(), NOW_MS));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub struct Fixture {
    pub dir: PathBuf,
    pub registry: KeyRegistry,
    pub epochs: SecurityEpochs,
    pub policy: SpendPolicy,
    pub claims: ClaimStore,
    pub nonces: DurableNonceFence,
    pub ledger: FileJournalLedger,
    pub signer: ReceiptKey,
    pub verifier: ReceiptCheck,
    custodians: Vec<(String, SigningKey)>,
}

impl Fixture {
    pub fn new(tag: &str) -> Self {
        Self::with(tag, 1_000_000, 500_000, 2_000_000)
    }

    pub fn with(tag: &str, treasury_minor: u64, per_effect_cap: u64, epoch_cap: u64) -> Self {
        let dir = unique_dir(tag);
        let custodians: Vec<(String, SigningKey)> = ["mac", "phone", "hsm"]
            .iter()
            .enumerate()
            .map(|(i, n)| ((*n).to_string(), SigningKey::from_bytes(&[(i as u8) + 11; 32])))
            .collect();
        let registry = KeyRegistry::from_records(custodians.iter().map(|(n, k)| KeyRecord {
            key_id: format!("key-{n}"),
            custodian_id: format!("device:{n}"),
            algorithm: SignatureAlgorithm::Ed25519,
            public_key: k.verifying_key().to_bytes().to_vec(),
            state: KeyState::Active,
            not_before_ms: 0,
            expires_at_ms: 10_000_000,
            revocation_epoch: 0,
        }));
        let mut policy = SpendPolicy::default();
        policy.principals.insert(
            PRINCIPAL.to_string(),
            PrincipalPolicy {
                allowed_payers: BTreeSet::from([PAYER.to_string(), PAYEE.to_string()]),
                allowed_payees: BTreeSet::from([PAYEE.to_string(), PAYER.to_string()]),
                per_effect_cap: [(Currency::USD, per_effect_cap)].into(),
                epoch_cap: [(Currency::USD, epoch_cap)].into(),
            },
        );
        let rk = SigningKey::from_bytes(&[99; 32]);
        Self {
            registry,
            epochs: SecurityEpochs { policy: 7, revocation: 0, generation: 1 },
            policy,
            claims: ClaimStore::open(dir.join("claims")).unwrap(),
            nonces: DurableNonceFence::open(dir.join("nonces")).unwrap(),
            ledger: FileJournalLedger::open(
                dir.join("ledger"),
                &[(PAYER, Currency::USD, treasury_minor), (PAYEE, Currency::USD, 0)],
            )
            .unwrap(),
            verifier: ReceiptCheck { id: "receipt-key".into(), key: rk.verifying_key() },
            signer: ReceiptKey { id: "receipt-key".into(), key: rk },
            custodians,
            dir,
        }
    }

    pub fn effect(&self, amount: &str, obligation: &str) -> PreparedEffect {
        PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, amount, Currency::USD, obligation, "invoice-payment", None).unwrap()
    }

    /// Certificate signed by `signers` custodians (threshold 2 of the three).
    pub fn cert(&self, effect: &PreparedEffect, nonce: &str, signers: &[&str]) -> ActuationCertificate {
        let mut cert = ActuationCertificate {
            version: 1,
            effect_digest: effect.digest().unwrap(),
            principal: effect.principal.clone(),
            policy_epoch: 7,
            revocation_epoch: 0,
            generation: 1,
            nonce: nonce.to_string(),
            not_before_ms: 0,
            expires_at_ms: 100_000,
            audience: AUDIENCE.to_string(),
            threshold: 2,
            signatures: vec![],
        };
        let msg = cert.signing_message().unwrap();
        for name in signers {
            let (_, key) = self.custodians.iter().find(|(n, _)| n == name).unwrap();
            cert.signatures.push(CertificateSignature {
                key_id: format!("key-{name}"),
                algorithm: SignatureAlgorithm::Ed25519,
                signature: key.sign(&msg).to_bytes().to_vec(),
            });
        }
        cert
    }

    pub fn admit(&self, effect: PreparedEffect, nonce: &str) -> Result<PaymentAdmission, String> {
        let cert = self.cert(&effect, nonce, &["mac", "phone"]);
        admit_payment(effect, &cert, &self.admission_ctx())
    }

    pub fn admission_ctx(&self) -> AdmissionContext<'_> {
        AdmissionContext {
            registry: &self.registry,
            epochs: self.epochs,
            audience: AUDIENCE,
            now_ms: NOW_MS,
            policy: &self.policy,
            claims: &self.claims,
            nonces: &self.nonces,
        }
    }

    /// Inert CONSTRUCT step only (no DO): yields the token a ledger `post` requires.
    pub fn token_for(&self, admission: &PaymentAdmission) -> ActuationToken {
        let (construct, _, _) = build_construct(admission, &self.exec_ctx()).expect("construct admits");
        ActuationToken::from_construct(&construct)
    }

    pub fn exec_ctx(&self) -> ExecutionContext<'_> {
        ExecutionContext {
            blake3: &RealBlake3,
            signer: &self.signer,
            verifier: &self.verifier,
            allowed_authorities: BTreeSet::from([AUDIENCE.to_string()]),
            journal_root: self.dir.join("brce"),
            now_epoch_ms: 1_000,
            ledger: &self.ledger,
            claims: &self.claims,
            policy: &self.policy,
        }
    }
}
