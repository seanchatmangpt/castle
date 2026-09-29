//! SA2A external authority verification boundary: verification only; never DO.
use std::collections::{BTreeMap,BTreeSet};
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord)] pub enum SignatureAlgorithm{Ed25519,MlDsa65,SlhDsaShake128f}
#[derive(Debug,Clone,PartialEq,Eq)] pub struct KeyRecord{pub key_id:String,pub principal:String,pub algorithm:SignatureAlgorithm,pub public_key:Vec<u8>,pub valid_from_epoch:u64,pub revoked_at_epoch:Option<u64>,pub independence_domain:String}
#[derive(Debug,Clone,PartialEq,Eq)] pub struct CertificateSignature{pub key_id:String,pub algorithm:SignatureAlgorithm,pub signature:Vec<u8>}
#[derive(Debug,Clone,PartialEq,Eq)] pub struct ActuationCertificate{pub effect_digest:[u8;32],pub principal:String,pub policy_epoch:u64,pub revocation_epoch:u64,pub execution_generation:u64,pub signatures:Vec<CertificateSignature>}
#[derive(Debug,Clone,PartialEq,Eq)] pub struct VerificationPolicy{pub current_policy_epoch:u64,pub current_revocation_epoch:u64,pub minimum_execution_generation:u64,pub quorum:usize,pub allowed_algorithms:BTreeSet<SignatureAlgorithm>}
#[derive(Debug,Clone,PartialEq,Eq)] pub enum Refusal{EffectDigestMismatch,PrincipalMismatch,PolicyEpochMismatch,RevocationEpochStale,ExecutionGenerationStale,EmptyQuorum,DuplicateSigner,DuplicateIndependenceDomain,UnknownKey,KeyPrincipalMismatch,AlgorithmMismatch,AlgorithmNotAllowed,KeyNotYetValid,KeyRevoked,SignatureInvalid,QuorumNotMet}
pub trait CryptoProvider:Send+Sync{fn verify(&self,algorithm:SignatureAlgorithm,public_key:&[u8],message:&[u8],signature:&[u8])->bool;}
pub struct AuthorityVerifier<P>{provider:P,keys:BTreeMap<String,KeyRecord>}
impl<P:CryptoProvider> AuthorityVerifier<P>{
pub fn new(provider:P,keys:impl IntoIterator<Item=KeyRecord>)->Self{Self{provider,keys:keys.into_iter().map(|k|(k.key_id.clone(),k)).collect()}}
pub fn verify(&self,effect:[u8;32],principal:&str,c:&ActuationCertificate,p:&VerificationPolicy)->Result<(),Refusal>{
if c.effect_digest!=effect{return Err(Refusal::EffectDigestMismatch)} if c.principal!=principal{return Err(Refusal::PrincipalMismatch)}
if c.policy_epoch!=p.current_policy_epoch{return Err(Refusal::PolicyEpochMismatch)} if c.revocation_epoch<p.current_revocation_epoch{return Err(Refusal::RevocationEpochStale)}
if c.execution_generation<p.minimum_execution_generation{return Err(Refusal::ExecutionGenerationStale)} if p.quorum==0{return Err(Refusal::EmptyQuorum)}
let mut signers=BTreeSet::new();let mut domains=BTreeSet::new();let mut valid=0;let msg=certificate_message(c);
for s in &c.signatures{if !signers.insert(s.key_id.clone()){return Err(Refusal::DuplicateSigner)} let k=self.keys.get(&s.key_id).ok_or(Refusal::UnknownKey)?;
if k.principal!=c.principal{return Err(Refusal::KeyPrincipalMismatch)} if k.algorithm!=s.algorithm{return Err(Refusal::AlgorithmMismatch)}
if !p.allowed_algorithms.contains(&s.algorithm){return Err(Refusal::AlgorithmNotAllowed)} if c.policy_epoch<k.valid_from_epoch{return Err(Refusal::KeyNotYetValid)}
if k.revoked_at_epoch.is_some_and(|e|c.revocation_epoch>=e){return Err(Refusal::KeyRevoked)} if !domains.insert(k.independence_domain.clone()){return Err(Refusal::DuplicateIndependenceDomain)}
if !self.provider.verify(s.algorithm,&k.public_key,&msg,&s.signature){return Err(Refusal::SignatureInvalid)} valid+=1;} if valid<p.quorum{return Err(Refusal::QuorumNotMet)} Ok(())}}
pub fn certificate_message(c:&ActuationCertificate)->Vec<u8>{let mut o=Vec::new();o.extend_from_slice(&c.effect_digest);o.extend_from_slice(&(c.principal.len() as u64).to_be_bytes());o.extend_from_slice(c.principal.as_bytes());o.extend_from_slice(&c.policy_epoch.to_be_bytes());o.extend_from_slice(&c.revocation_epoch.to_be_bytes());o.extend_from_slice(&c.execution_generation.to_be_bytes());o}
