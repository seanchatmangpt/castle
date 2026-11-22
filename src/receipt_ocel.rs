//! Chatman 5-field receipt emitter with W3C PROV-O and IEEE OCEL 2.0 event
//! traces (CASTLE-SPEC-26.11.20).
//!
//! `R = receipt(A)` with the five Chatman fields, each carried explicitly:
//!
//! 1. **identity** — repo, branch, base/head git SHAs, per-file BLAKE3-256
//!    digests, the aggregate `tree_digest`, and the parent receipt digest
//!    (rolling chain link).
//! 2. **authority** — `OWNED | AUTHORIZED_CLIENT | PUBLIC_HEADLINE |
//!    FIRST_PARTY_DELETION`. Deletions are admissible only under
//!    `FIRST_PARTY_DELETION`; that authority is admissible only when the
//!    consequence actually witnesses a deletion (typed refusals otherwise).
//! 3. **consequence** — durable effect record: summary plus typed per-path
//!    effects (`CREATED | MODIFIED | DELETED`).
//! 4. **replay** — exact CLI commands with observed exit codes. `ALIVE`
//!    standing requires at least one replay step with exit code 0 (a receipt
//!    that claims ALIVE without observed execution is a typed refusal, never
//!    downgraded).
//! 5. **standing** — `ALIVE | PARTIAL_ALIVE | REFUSED | BLOCKED | UNKNOWN`.
//!
//! SPEC PROVENANCE: canonical-JSON and rolling-chain law mirrored from
//! `src/trust_chain.rs` (key-sorted canonical JSON, no whitespace):
//!
//! ```text
//! chain_head_0 = blake3(GENESIS_SEED)
//! receipt_digest_n = blake3(chain_head_{n-1}.as_hex().as_bytes() || canonical_bytes(receipt_n))
//! ```
//!
//! The emitter performs no I/O and reads no wall clock: every input —
//! including the ISO-8601 `emitted_at` timestamp — arrives through
//! [`ReceiptInput`], so identical input sequences produce byte-identical
//! receipts, PROV-JSON, and OCEL 2.0 traces (byte-identical replay is
//! asserted by test).
//!
//! Traces produced per emission:
//! - PROV-JSON (W3C PROV-O vocabulary, `http://www.w3.org/ns/prov#`): the
//!   emission activity, the receipt and artifact entities, the software
//!   agent, and the `used` / `wasGeneratedBy` / `wasAttributedTo` /
//!   `wasDerivedFrom` qualified relations.
//! - OCEL 2.0 JSON (IEEE OCEL standard shape, `ocel:`-prefixed keys): object
//!   types (`Repo`, `Agent`, `File`, `Receipt`), object attribute values, and
//!   events (`Receipt Emitted`, `Replay Step Executed`) with referentially
//!   closed object maps.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Genesis seed for the rolling receipt chain. Deliberately distinct from
/// `trust_chain.rs` `GENESIS_SEED` so the two chains never cross-verify.
pub const GENESIS_SEED: &[u8] = b"castle-receipt-ocel-v1-genesis";

/// W3C PROV namespace used by the PROV-JSON traces.
pub const PROV_NS: &str = "http://www.w3.org/ns/prov#";

/// Castle-local namespace for receipt-emission vocabulary in PROV traces.
pub const CASTLE_NS: &str = "https://castle.dev/ns/receipt-ocel#";

/// XML Schema namespace for typed literals in PROV traces.
pub const XSD_NS: &str = "http://www.w3.org/2001/XMLSchema#";

/// OCEL log version emitted by [`ocel_trace`].
pub const OCEL_VERSION: &str = "2.0";

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

/// Authority under which the receipted actuation was performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Authority {
    /// Written by the acting lane against files it owns.
    #[serde(rename = "OWNED")]
    Owned,
    /// Written under an explicit operator/client grant.
    #[serde(rename = "AUTHORIZED_CLIENT")]
    AuthorizedClient,
    /// Public-facing headline/summary surface only.
    #[serde(rename = "PUBLIC_HEADLINE")]
    PublicHeadline,
    /// First-party cleanup/deletion transition.
    #[serde(rename = "FIRST_PARTY_DELETION")]
    FirstPartyDeletion,
}

impl Authority {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Authority::Owned => "OWNED",
            Authority::AuthorizedClient => "AUTHORIZED_CLIENT",
            Authority::PublicHeadline => "PUBLIC_HEADLINE",
            Authority::FirstPartyDeletion => "FIRST_PARTY_DELETION",
        }
    }
}

/// Epistemic standing of the receipted work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Standing {
    /// Observed execution against the exact subject succeeded.
    #[serde(rename = "ALIVE")]
    Alive,
    /// Part of the receipted work is verified; the rest is not.
    #[serde(rename = "PARTIAL_ALIVE")]
    PartialAlive,
    /// The transition was refused; [`ChatmanReceipt`] carries the reason via
    /// [`ReceiptInput::refusal_reason`].
    #[serde(rename = "REFUSED")]
    Refused,
    /// The transition is blocked on an external dependency.
    #[serde(rename = "BLOCKED")]
    Blocked,
    /// Standing not yet determined.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl Standing {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Standing::Alive => "ALIVE",
            Standing::PartialAlive => "PARTIAL_ALIVE",
            Standing::Refused => "REFUSED",
            Standing::Blocked => "BLOCKED",
            Standing::Unknown => "UNKNOWN",
        }
    }
}

/// Kind of durable effect recorded in the consequence field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectKind {
    #[serde(rename = "CREATED")]
    Created,
    #[serde(rename = "MODIFIED")]
    Modified,
    #[serde(rename = "DELETED")]
    Deleted,
}

impl EffectKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            EffectKind::Created => "CREATED",
            EffectKind::Modified => "MODIFIED",
            EffectKind::Deleted => "DELETED",
        }
    }
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// One receipted path with its BLAKE3-256 content digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityFile {
    pub path: String,
    /// Lowercase 64-hex-character BLAKE3-256 digest of the file content.
    pub blake3: String,
}

/// Receipt identity: paths, git SHAs, BLAKE3 digests, and the chain parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptIdentity {
    pub repo: String,
    pub branch: String,
    /// 40-hex git SHA of the base commit.
    pub base_commit: String,
    /// 40-hex git SHA of the head commit the receipt certifies.
    pub head_commit: String,
    /// BLAKE3-256 over the path-sorted `"<path>\n<blake3>\n"` stream.
    pub tree_digest: String,
    /// Parent receipt digest in the rolling chain (genesis hex for the first
    /// emission of an emitter).
    pub parent_receipt_digest: String,
    /// Files in path-sorted canonical order.
    pub files: Vec<IdentityFile>,
}

// ---------------------------------------------------------------------------
// Consequence, replay, standing carriers
// ---------------------------------------------------------------------------

/// One durable effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub kind: EffectKind,
    pub path: String,
    pub detail: String,
}

/// Durable consequence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consequence {
    pub summary: String,
    pub effects: Vec<Effect>,
}

/// One observed execution step: the exact command and its exit code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayStep {
    pub command: String,
    pub exit_code: i32,
}

/// Replay field: exact CLI commands with observed exit codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub steps: Vec<ReplayStep>,
}

/// The 5-field Chatman receipt: identity, authority, consequence, replay,
/// standing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatmanReceipt {
    pub identity: ReceiptIdentity,
    pub authority: Authority,
    pub consequence: Consequence,
    pub replay: Replay,
    pub standing: Standing,
}

impl ChatmanReceipt {
    /// Canonical JSON form (key-sorted, no whitespace) — the exact bytes the
    /// chain digest is computed over.
    ///
    /// # Errors
    /// Returns an error if the receipt serializes to a non-finite float
    /// (impossible for values this module constructs, but the law is
    /// fail-closed).
    pub fn canonical_form(&self) -> Result<String, String> {
        canonical_json(&serde_json::to_value(self).map_err(|e| e.to_string())?)
    }

    /// JSON value of exactly the five Chatman fields.
    ///
    /// # Errors
    /// Propagates serialization errors.
    pub fn to_value(&self) -> Result<Value, String> {
        serde_json::to_value(self).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// Typed refusals
// ---------------------------------------------------------------------------

/// Typed refusal emitted by [`ReceiptOcelEmitter::emit`]. Every variant names
/// the exact law it guards; no refusal is silent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReceiptRefusal {
    /// Actor name is empty — a receipt without an agent is unattributable.
    EmptyActor,
    /// A field that must hold a 40-hex git SHA does not.
    InvalidSha { field: &'static str },
    /// A file digest is not a 64-hex BLAKE3-256 value.
    InvalidBlake3 { path: String },
    /// `emitted_at` is not a non-empty ISO-8601-shaped timestamp.
    EmptyTimestamp,
    /// Identity carries no files.
    EmptyIdentity,
    /// A replay step has an empty command — unexecutable replay is not replay.
    EmptyReplayCommand { index: usize },
    /// `ALIVE` standing claimed without any replay step exiting 0.
    AliveWithoutReplay,
    /// `REFUSED` standing claimed without a refusal reason.
    RefusedWithoutReason,
    /// A deletion effect exists outside `FIRST_PARTY_DELETION` authority.
    DeletionOutsideAuthority,
    /// `FIRST_PARTY_DELETION` authority without any deletion effect.
    FirstPartyDeletionWithoutDeletion,
    /// Consequence summary is empty — an unrecorded consequence is no
    /// consequence.
    EmptyConsequenceSummary,
}

impl ReceiptRefusal {
    /// Stable machine-readable code (`REFUSED_*` vocabulary).
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            ReceiptRefusal::EmptyActor => "REFUSED_EMPTY_ACTOR",
            ReceiptRefusal::InvalidSha { .. } => "REFUSED_INVALID_SHA",
            ReceiptRefusal::InvalidBlake3 { .. } => "REFUSED_INVALID_BLAKE3",
            ReceiptRefusal::EmptyTimestamp => "REFUSED_EMPTY_TIMESTAMP",
            ReceiptRefusal::EmptyIdentity => "REFUSED_EMPTY_IDENTITY",
            ReceiptRefusal::EmptyReplayCommand { .. } => "REFUSED_EMPTY_REPLAY_COMMAND",
            ReceiptRefusal::AliveWithoutReplay => "REFUSED_ALIVE_WITHOUT_REPLAY",
            ReceiptRefusal::RefusedWithoutReason => "REFUSED_REFUSAL_WITHOUT_REASON",
            ReceiptRefusal::DeletionOutsideAuthority => "REFUSED_DELETION_OUTSIDE_AUTHORITY",
            ReceiptRefusal::FirstPartyDeletionWithoutDeletion => {
                "REFUSED_FIRST_PARTY_DELETION_WITHOUT_DELETION"
            }
            ReceiptRefusal::EmptyConsequenceSummary => "REFUSED_EMPTY_CONSEQUENCE_SUMMARY",
        }
    }
}

impl std::fmt::Display for ReceiptRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiptRefusal::EmptyActor => write!(f, "REFUSED_EMPTY_ACTOR: actor is empty"),
            ReceiptRefusal::InvalidSha { field } => {
                write!(f, "REFUSED_INVALID_SHA: {field} is not a 40-hex git SHA")
            }
            ReceiptRefusal::InvalidBlake3 { path } => {
                write!(f, "REFUSED_INVALID_BLAKE3: file {path} digest is not 64-hex BLAKE3-256")
            }
            ReceiptRefusal::EmptyTimestamp => {
                write!(f, "REFUSED_EMPTY_TIMESTAMP: emitted_at is empty")
            }
            ReceiptRefusal::EmptyIdentity => {
                write!(f, "REFUSED_EMPTY_IDENTITY: identity carries no files")
            }
            ReceiptRefusal::EmptyReplayCommand { index } => {
                write!(f, "REFUSED_EMPTY_REPLAY_COMMAND: replay step {index} command is empty")
            }
            ReceiptRefusal::AliveWithoutReplay => {
                write!(
                    f,
                    "REFUSED_ALIVE_WITHOUT_REPLAY: ALIVE standing requires a replay step with exit code 0"
                )
            }
            ReceiptRefusal::RefusedWithoutReason => {
                write!(f, "REFUSED_REFUSAL_WITHOUT_REASON: REFUSED standing requires refusal_reason")
            }
            ReceiptRefusal::DeletionOutsideAuthority => {
                write!(
                    f,
                    "REFUSED_DELETION_OUTSIDE_AUTHORITY: deletion effect outside FIRST_PARTY_DELETION authority"
                )
            }
            ReceiptRefusal::FirstPartyDeletionWithoutDeletion => {
                write!(
                    f,
                    "REFUSED_FIRST_PARTY_DELETION_WITHOUT_DELETION: FIRST_PARTY_DELETION authority without a deletion effect"
                )
            }
            ReceiptRefusal::EmptyConsequenceSummary => {
                write!(f, "REFUSED_EMPTY_CONSEQUENCE_SUMMARY: consequence summary is empty")
            }
        }
    }
}

impl std::error::Error for ReceiptRefusal {}

// ---------------------------------------------------------------------------
// Emitter input
// ---------------------------------------------------------------------------

/// Raw, unvalidated emission input. The emitter validates and derives
/// canonical identity (path-sorted files, tree digest, chain parent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptInput {
    pub repo: String,
    pub branch: String,
    pub base_commit: String,
    pub head_commit: String,
    pub files: Vec<IdentityFile>,
    pub authority: Authority,
    pub consequence_summary: String,
    pub effects: Vec<Effect>,
    pub replay_steps: Vec<ReplayStep>,
    pub standing: Standing,
    /// Required (non-empty) exactly when `standing == REFUSED`.
    pub refusal_reason: Option<String>,
    /// ISO-8601 emission timestamp supplied by the caller; the emitter never
    /// reads a wall clock.
    pub emitted_at: String,
}

// ---------------------------------------------------------------------------
// Emitter
// ---------------------------------------------------------------------------

/// Deterministic rolling-chain emitter of 5-field Chatman receipts with
/// PROV-O and OCEL 2.0 traces.
#[derive(Debug, Clone)]
pub struct ReceiptOcelEmitter {
    actor: String,
    chain_head: String,
}

/// One admitted emission: the receipt, its chain digest, and both traces.
#[derive(Debug, Clone, Serialize)]
pub struct EmittedReceipt {
    pub receipt: ChatmanReceipt,
    /// BLAKE3-256 chain digest of this receipt.
    pub receipt_digest: String,
    /// Chain digest of the parent receipt (genesis hex for a first emission).
    pub parent_receipt_digest: String,
    /// Canonical JSON bytes the digest was computed over.
    pub canonical: String,
    /// PROV-JSON trace (W3C PROV-O vocabulary).
    pub prov: Value,
    /// OCEL 2.0 JSON trace.
    pub ocel: Value,
}

impl ReceiptOcelEmitter {
    /// New emitter; chain head initialized to `blake3(GENESIS_SEED)`.
    ///
    /// # Errors
    /// [`ReceiptRefusal::EmptyActor`] when `actor` is empty.
    pub fn new(actor: &str) -> Result<Self, ReceiptRefusal> {
        if actor.trim().is_empty() {
            return Err(ReceiptRefusal::EmptyActor);
        }
        Ok(Self { actor: actor.to_string(), chain_head: blake3_hex(GENESIS_SEED) })
    }

    #[must_use]
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Current chain head digest (the digest the next receipt names as
    /// parent).
    #[must_use]
    pub fn chain_head(&self) -> &str {
        &self.chain_head
    }

    /// Validate `input`, manufacture the receipt, advance the chain, and
    /// produce both traces.
    ///
    /// # Errors
    /// Returns the typed [`ReceiptRefusal`] naming the broken law.
    pub fn emit(&mut self, input: ReceiptInput) -> Result<EmittedReceipt, ReceiptRefusal> {
        validate_input(&input)?;
        let parent = self.chain_head.clone();
        let mut files = input.files.clone();
        files.sort_by(|a, b| a.path.cmp(&b.path).then(a.blake3.cmp(&b.blake3)));
        let identity = ReceiptIdentity {
            repo: input.repo.clone(),
            branch: input.branch.clone(),
            base_commit: input.base_commit.clone(),
            head_commit: input.head_commit.clone(),
            tree_digest: tree_digest(&files),
            parent_receipt_digest: parent.clone(),
            files,
        };
        let receipt = ChatmanReceipt {
            identity,
            authority: input.authority,
            consequence: Consequence {
                summary: input.consequence_summary.clone(),
                effects: input.effects.clone(),
            },
            replay: Replay { steps: input.replay_steps.clone() },
            standing: input.standing,
        };
        let canonical = receipt.canonical_form().map_err(|_| ReceiptRefusal::EmptyActor)?;
        let digest = blake3_hex(&[parent.as_bytes(), canonical.as_bytes()].concat());
        self.chain_head = digest.clone();
        let prov = prov_trace(&receipt, &digest, &self.actor, &parent, &input.emitted_at);
        let ocel = ocel_trace(&receipt, &digest, &self.actor, &parent, &input.emitted_at);
        Ok(EmittedReceipt {
            receipt,
            receipt_digest: digest,
            parent_receipt_digest: parent,
            canonical,
            prov,
            ocel,
        })
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate raw input against the receipt laws.
///
/// # Errors
/// Typed [`ReceiptRefusal`] naming the broken law.
pub fn validate_input(input: &ReceiptInput) -> Result<(), ReceiptRefusal> {
    if !is_git_sha(&input.base_commit) {
        return Err(ReceiptRefusal::InvalidSha { field: "base_commit" });
    }
    if !is_git_sha(&input.head_commit) {
        return Err(ReceiptRefusal::InvalidSha { field: "head_commit" });
    }
    if input.files.is_empty() {
        return Err(ReceiptRefusal::EmptyIdentity);
    }
    for f in &input.files {
        if !is_blake3_hex(&f.blake3) {
            return Err(ReceiptRefusal::InvalidBlake3 { path: f.path.clone() });
        }
    }
    if input.emitted_at.trim().is_empty() {
        return Err(ReceiptRefusal::EmptyTimestamp);
    }
    if input.consequence_summary.trim().is_empty() {
        return Err(ReceiptRefusal::EmptyConsequenceSummary);
    }
    for (i, step) in input.replay_steps.iter().enumerate() {
        if step.command.trim().is_empty() {
            return Err(ReceiptRefusal::EmptyReplayCommand { index: i });
        }
    }
    if input.standing == Standing::Alive && !input.replay_steps.iter().any(|s| s.exit_code == 0) {
        return Err(ReceiptRefusal::AliveWithoutReplay);
    }
    if input.standing == Standing::Refused
        && input.refusal_reason.as_deref().map_or(true, |r| r.trim().is_empty())
    {
        return Err(ReceiptRefusal::RefusedWithoutReason);
    }
    let has_deletion = input.effects.iter().any(|e| e.kind == EffectKind::Deleted);
    if has_deletion && input.authority != Authority::FirstPartyDeletion {
        return Err(ReceiptRefusal::DeletionOutsideAuthority);
    }
    if input.authority == Authority::FirstPartyDeletion && !has_deletion {
        return Err(ReceiptRefusal::FirstPartyDeletionWithoutDeletion);
    }
    Ok(())
}

/// 40 lowercase hex characters (git object SHA-1 as rendered by `git rev-parse`).
fn is_git_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// 64 lowercase hex characters (BLAKE3-256 hex rendering).
fn is_blake3_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// BLAKE3-256 over the path-sorted `"<path>\n<blake3>\n"` stream.
#[must_use]
pub fn tree_digest(files: &[IdentityFile]) -> String {
    let mut sorted: Vec<&IdentityFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path).then(a.blake3.cmp(&b.blake3)));
    let mut stream = String::new();
    for f in sorted {
        stream.push_str(&f.path);
        stream.push('\n');
        stream.push_str(&f.blake3);
        stream.push('\n');
    }
    blake3_hex(stream.as_bytes())
}

// ---------------------------------------------------------------------------
// Canonical JSON + BLAKE3 (law mirrored from trust_chain.rs)
// ---------------------------------------------------------------------------

fn blake3_hex(input: &[u8]) -> String {
    blake3::hash(input).to_hex().to_string()
}

/// Key-sorted, whitespace-free canonical JSON. Fails closed on non-finite
/// floats (the same law as `trust_chain.rs`).
fn canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err("REFUSED_NON_FINITE_CANONICAL_VALUE".to_string());
                }
            }
            Ok(n.to_string())
        }
        Value::String(s) => Ok(serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())),
        Value::Array(arr) => {
            let parts: Result<Vec<String>, String> =
                arr.iter().map(canonical_json).collect();
            Ok(format!("[{}]", parts?.join(",")))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for k in keys {
                let v = canonical_json(&map[k])?;
                parts.push(format!("{}:{}", serde_json::to_string(k).unwrap_or_else(|_| "\"\"".to_string()), v));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
    }
}

// ---------------------------------------------------------------------------
// PROV-JSON trace (W3C PROV-O)
// ---------------------------------------------------------------------------

fn qname(ns: &str, local: &str) -> Value {
    json!({"$": format!("{ns}{local}"), "type": "prov:QualifiedName"})
}

fn xsd_string(v: &str) -> Value {
    json!({"$": v, "type": "xsd:string"})
}

fn prov_trace(
    receipt: &ChatmanReceipt,
    digest: &str,
    actor: &str,
    parent_digest: &str,
    emitted_at: &str,
) -> Value {
    let d16 = &digest[..16];
    let receipt_id = format!("castle:receipt-{d16}");
    let activity_id = format!("castle:emit-{d16}");
    let agent_id = format!("castle:agent-{}", sanitize(actor));
    let commit_id = format!("castle:commit-{}", receipt.identity.head_commit);
    let parent_entity = if parent_digest == blake3_hex(GENESIS_SEED) {
        "castle:genesis".to_string()
    } else {
        format!("castle:receipt-{}", &parent_digest[..16])
    };

    let mut entities = Map::new();
    let mut receipt_entity = Map::new();
    receipt_entity.insert("prov:type".to_string(), qname(CASTLE_NS, "ChatmanReceipt"));
    receipt_entity.insert("castle:receiptDigest".to_string(), xsd_string(digest));
    receipt_entity.insert("castle:standing".to_string(), xsd_string(receipt.standing.as_str()));
    receipt_entity.insert("castle:authority".to_string(), xsd_string(receipt.authority.as_str()));
    receipt_entity.insert("castle:treeDigest".to_string(), xsd_string(&receipt.identity.tree_digest));
    entities.insert(receipt_id.clone(), Value::Object(receipt_entity));

    let mut commit_entity = Map::new();
    commit_entity.insert("prov:type".to_string(), qname(CASTLE_NS, "CommitHead"));
    commit_entity.insert("castle:repo".to_string(), xsd_string(&receipt.identity.repo));
    commit_entity.insert("castle:branch".to_string(), xsd_string(&receipt.identity.branch));
    entities.insert(commit_id.clone(), Value::Object(commit_entity));

    let mut used = Map::new();
    let mut used_commit = Map::new();
    used_commit.insert("prov:activity".to_string(), json!(activity_id));
    used_commit.insert("prov:entity".to_string(), json!(commit_id));
    used.insert("_:used-commit".to_string(), Value::Object(used_commit));

    for (i, f) in receipt.identity.files.iter().enumerate() {
        let file_id = format!("castle:file-{i}-{}", &f.blake3[..12]);
        let mut file_entity = Map::new();
        file_entity.insert("prov:type".to_string(), qname(CASTLE_NS, "ReceiptedFile"));
        file_entity.insert("castle:path".to_string(), xsd_string(&f.path));
        file_entity.insert("castle:blake3".to_string(), xsd_string(&f.blake3));
        entities.insert(file_id.clone(), Value::Object(file_entity));
        let mut u = Map::new();
        u.insert("prov:activity".to_string(), json!(activity_id));
        u.insert("prov:entity".to_string(), json!(file_id));
        used.insert(format!("_:used-file-{i}"), Value::Object(u));
    }

    let mut activities = Map::new();
    let mut activity = Map::new();
    activity.insert("prov:type".to_string(), qname(CASTLE_NS, "ReceiptEmission"));
    activity.insert("prov:startedAtTime".to_string(), xsd_string(emitted_at));
    activity.insert("prov:endedAtTime".to_string(), xsd_string(emitted_at));
    activities.insert(activity_id.clone(), Value::Object(activity));

    let mut agents = Map::new();
    let mut agent = Map::new();
    agent.insert("prov:type".to_string(), qname(PROV_NS, "SoftwareAgent"));
    agent.insert("castle:actor".to_string(), xsd_string(actor));
    agents.insert(agent_id.clone(), Value::Object(agent));

    let mut generated = Map::new();
    let mut gen = Map::new();
    gen.insert("prov:activity".to_string(), json!(activity_id));
    gen.insert("prov:entity".to_string(), json!(receipt_id));
    gen.insert("prov:time".to_string(), xsd_string(emitted_at));
    generated.insert("_:gen-receipt".to_string(), Value::Object(gen));

    let mut attributed = Map::new();
    let mut attr = Map::new();
    attr.insert("prov:entity".to_string(), json!(receipt_id));
    attr.insert("prov:agent".to_string(), json!(agent_id));
    attributed.insert("_:attr-receipt".to_string(), Value::Object(attr));

    let mut derived = Map::new();
    let mut der = Map::new();
    der.insert("prov:generatedEntity".to_string(), json!(receipt_id));
    der.insert("prov:usedEntity".to_string(), json!(parent_entity));
    derived.insert("_:der-receipt".to_string(), Value::Object(der));

    json!({
        "prefix": {
            "prov": PROV_NS,
            "castle": CASTLE_NS,
            "xsd": XSD_NS,
        },
        "entity": Value::Object(entities),
        "activity": Value::Object(activities),
        "agent": Value::Object(agents),
        "used": Value::Object(used),
        "wasGeneratedBy": Value::Object(generated),
        "wasAttributedTo": Value::Object(attributed),
        "wasDerivedFrom": Value::Object(derived),
    })
}

// ---------------------------------------------------------------------------
// OCEL 2.0 JSON trace
// ---------------------------------------------------------------------------

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            c
        } else {
            '_'
        })
        .collect()
}

fn ocel_attr(name: &str, value: &Value, ty: &str) -> Value {
    json!({"ocel:name": name, "ocel:value": value, "ocel:type": ty})
}

fn ocel_trace(
    receipt: &ChatmanReceipt,
    digest: &str,
    actor: &str,
    parent_digest: &str,
    emitted_at: &str,
) -> Value {
    let d16 = &digest[..16];
    let receipt_oid = format!("receipt-{d16}");
    let repo_oid = format!("repo-{}", sanitize(&receipt.identity.repo));
    let agent_oid = format!("agent-{}", sanitize(actor));

    let mut file_oids = Vec::with_capacity(receipt.identity.files.len());
    let mut objects_file = Map::new();
    for f in &receipt.identity.files {
        let oid = format!("file-{}", sanitize(&f.path));
        file_oids.push(oid.clone());
        objects_file.insert(
            oid,
            json!({"path": f.path, "blake3": f.blake3}),
        );
    }

    let mut objects = Map::new();
    objects.insert(
        "Repo".to_string(),
        json!({ repo_oid.clone(): {"name": receipt.identity.repo} }),
    );
    objects.insert(
        "Agent".to_string(),
        json!({ agent_oid.clone(): {"name": actor} }),
    );
    objects.insert("File".to_string(), Value::Object(objects_file));
    objects.insert(
        "Receipt".to_string(),
        json!({ receipt_oid.clone(): {
            "standing": receipt.standing.as_str(),
            "authority": receipt.authority.as_str(),
            "receipt_digest": digest,
            "parent_receipt_digest": parent_digest,
        }}),
    );

    let mut events = Vec::with_capacity(receipt.replay.steps.len() + 1);
    let mut emitted_map = Map::new();
    emitted_map.insert("Receipt".to_string(), json!([receipt_oid]));
    emitted_map.insert("Repo".to_string(), json!([repo_oid]));
    emitted_map.insert("Agent".to_string(), json!([agent_oid]));
    emitted_map.insert("File".to_string(), Value::Array(file_oids.into_iter().map(Value::String).collect::<Vec<_>>()));
    events.push(json!({
        "ocel:eid": format!("event-{d16}-0"),
        "ocel:activity": "Receipt Emitted",
        "ocel:timestamp": emitted_at,
        "ocel:attributes": [
            ocel_attr("standing", &json!(receipt.standing.as_str()), "string"),
            ocel_attr("authority", &json!(receipt.authority.as_str()), "string"),
            ocel_attr("receipt_digest", &json!(digest), "string"),
            ocel_attr("parent_receipt_digest", &json!(parent_digest), "string"),
            ocel_attr("base_commit", &json!(receipt.identity.base_commit), "string"),
            ocel_attr("head_commit", &json!(receipt.identity.head_commit), "string"),
            ocel_attr("consequence_summary", &json!(receipt.consequence.summary), "string"),
        ],
        "ocel:object_map": Value::Object(emitted_map),
    }));

    for (i, step) in receipt.replay.steps.iter().enumerate() {
        let mut step_map = Map::new();
        step_map.insert("Receipt".to_string(), json!([receipt_oid.clone()]));
        step_map.insert("Repo".to_string(), json!([repo_oid.clone()]));
        events.push(json!({
            "ocel:eid": format!("event-{d16}-{}", i + 1),
            "ocel:activity": "Replay Step Executed",
            "ocel:timestamp": emitted_at,
            "ocel:attributes": [
                ocel_attr("step_index", &json!(i), "int"),
                ocel_attr("command", &json!(step.command), "string"),
                ocel_attr("exit_code", &json!(step.exit_code), "int"),
            ],
            "ocel:object_map": Value::Object(step_map),
        }));
    }

    json!({
        "ocel:version": OCEL_VERSION,
        "ocel:object-types": [
            {"ocel:name": "Repo", "ocel:attributes": [{"ocel:name": "name", "ocel:type": "string"}]},
            {"ocel:name": "Agent", "ocel:attributes": [{"ocel:name": "name", "ocel:type": "string"}]},
            {"ocel:name": "File", "ocel:attributes": [
                {"ocel:name": "path", "ocel:type": "string"},
                {"ocel:name": "blake3", "ocel:type": "string"},
            ]},
            {"ocel:name": "Receipt", "ocel:attributes": [
                {"ocel:name": "standing", "ocel:type": "string"},
                {"ocel:name": "authority", "ocel:type": "string"},
                {"ocel:name": "receipt_digest", "ocel:type": "string"},
                {"ocel:name": "parent_receipt_digest", "ocel:type": "string"},
            ]},
        ],
        "ocel:object-fields": {
            "Repo": ["name"],
            "Agent": ["name"],
            "File": ["path", "blake3"],
            "Receipt": ["standing", "authority", "receipt_digest", "parent_receipt_digest"],
        },
        "ocel:objects": Value::Object(objects),
        "ocel:events": events,
    })
}

// ---------------------------------------------------------------------------
// Tests — zero mocks; every assertion runs real BLAKE3 / real canonical JSON
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sha(seed: u8) -> String {
        let hex = format!("{seed:02x}").repeat(20);
        assert_eq!(hex.len(), 40);
        hex
    }

    fn blake(seed: u8) -> String {
        blake3_hex(&[seed; 37])
    }

    fn file(path: &str, seed: u8) -> IdentityFile {
        IdentityFile { path: path.to_string(), blake3: blake(seed) }
    }

    fn input() -> ReceiptInput {
        ReceiptInput {
            repo: "seanchatmangpt/castle".to_string(),
            branch: "feat/receipt-ocel".to_string(),
            base_commit: sha(0xaa),
            head_commit: sha(0xbb),
            files: vec![
                file("src/b_two.rs", 2),
                file("src/a_one.rs", 1),
            ],
            authority: Authority::Owned,
            consequence_summary: "emitted receipt-ocel module".to_string(),
            effects: vec![Effect {
                kind: EffectKind::Modified,
                path: "src/receipt_ocel.rs".to_string(),
                detail: "5-field receipt emitter landed".to_string(),
            }],
            replay_steps: vec![
                ReplayStep { command: "cargo test -p castle receipt_ocel".to_string(), exit_code: 0 },
                ReplayStep { command: "cargo clippy -- -D warnings".to_string(), exit_code: 101 },
            ],
            standing: Standing::Alive,
            refusal_reason: None,
            emitted_at: "2026-11-20T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn emits_five_field_receipt_with_verifiable_chain_digest() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let head_before = em.chain_head().to_string();
        let out = em.emit(input()).expect("emission");
        assert_eq!(out.parent_receipt_digest, head_before);
        // The digest is blake3(parent_hex_bytes || canonical_bytes) — recompute.
        let expected = blake3_hex(
            &[head_before.as_bytes(), out.receipt.canonical_form().expect("canonical").as_bytes()].concat(),
        );
        assert_eq!(out.receipt_digest, expected);
        assert_eq!(em.chain_head(), out.receipt_digest);
        // Chain injection makes the digest differ from a bare content hash.
        let bare = blake3_hex(out.canonical.as_bytes());
        assert_ne!(out.receipt_digest, bare);
    }

    #[test]
    fn receipt_json_carries_exactly_the_five_chatman_fields() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let out = em.emit(input()).expect("emission");
        let v = out.receipt.to_value().expect("json");
        let obj = v.as_object().expect("object");
        let expected = ["authority", "consequence", "identity", "replay", "standing"];
        let actual: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(actual, expected);
        assert_eq!(obj["identity"]["files"].as_array().expect("files").len(), 2);
        // Files are canonicalized path-sorted regardless of input order.
        let paths: Vec<&str> = obj["identity"]["files"]
            .as_array()
            .expect("files")
            .iter()
            .map(|f| f["path"].as_str().expect("path"))
            .collect();
        assert_eq!(paths, vec!["src/a_one.rs", "src/b_two.rs"]);
    }

    #[test]
    fn digest_mutates_when_any_field_mutates() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let base = em.emit(input()).expect("emission");

        // Same emitter chain state is consumed; use fresh identical emitters.
        let variants: Vec<(String, ReceiptInput)> = vec![
            ("standing".to_string(), {
                let mut i = input();
                i.standing = Standing::PartialAlive;
                i
            }),
            ("authority".to_string(), {
                let mut i = input();
                i.authority = Authority::AuthorizedClient;
                i
            }),
            ("consequence".to_string(), {
                let mut i = input();
                i.consequence_summary = "different consequence".to_string();
                i
            }),
            ("replay".to_string(), {
                let mut i = input();
                // Mutate the already-failing step; step 0 keeps exit 0 so the
                // ALIVE gate still admits this input.
                i.replay_steps[1].exit_code = 102;
                i
            }),
            ("identity".to_string(), {
                let mut i = input();
                i.head_commit = sha(0xcc);
                i
            }),
        ];
        for (field, variant_input) in variants {
            let mut em2 = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
            let variant = em2.emit(variant_input).expect("emission");
            assert_ne!(
                variant.receipt_digest, base.receipt_digest,
                "mutating {field} must mutate the chain digest"
            );
        }
    }

    #[test]
    fn byte_identical_replay_across_fresh_emitters() {
        let mut a = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let mut b = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let ra = a.emit(input()).expect("emission");
        let rb = b.emit(input()).expect("emission");
        assert_eq!(ra.canonical, rb.canonical);
        assert_eq!(ra.receipt_digest, rb.receipt_digest);
        assert_eq!(
            serde_json::to_string(&ra.prov).expect("prov"),
            serde_json::to_string(&rb.prov).expect("prov")
        );
        assert_eq!(
            serde_json::to_string(&ra.ocel).expect("ocel"),
            serde_json::to_string(&rb.ocel).expect("ocel")
        );
        // Even a second emission on the same emitter is fully determined.
        let rc = a.emit(input()).expect("emission");
        assert_ne!(rc.receipt_digest, ra.receipt_digest);
        assert_eq!(rc.receipt.identity.parent_receipt_digest, ra.receipt_digest);
    }

    #[test]
    fn chain_links_parent_and_advances() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let genesis = blake3_hex(GENESIS_SEED);
        assert_eq!(em.chain_head(), genesis);
        let first = em.emit(input()).expect("first");
        assert_eq!(first.receipt.identity.parent_receipt_digest, genesis);
        let second = em.emit(input()).expect("second");
        assert_eq!(second.receipt.identity.parent_receipt_digest, first.receipt_digest);
        assert_ne!(first.receipt_digest, second.receipt_digest);
    }

    #[test]
    fn refuses_invalid_git_sha() {
        let mut i = input();
        i.head_commit = "not-a-sha".to_string();
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(
            em.emit(i).expect_err("refusal"),
            ReceiptRefusal::InvalidSha { field: "head_commit" }
        );
        let mut i = input();
        i.base_commit = sha(0xdd).to_uppercase(); // uppercase hex is not canonical
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(
            em.emit(i).expect_err("refusal"),
            ReceiptRefusal::InvalidSha { field: "base_commit" }
        );
    }

    #[test]
    fn refuses_invalid_blake3_digest() {
        let mut i = input();
        i.files.push(IdentityFile { path: "src/bad.rs".to_string(), blake3: "zz".repeat(32) });
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(
            em.emit(i).expect_err("refusal"),
            ReceiptRefusal::InvalidBlake3 { path: "src/bad.rs".to_string() }
        );
    }

    #[test]
    fn refuses_empty_identity() {
        let mut i = input();
        i.files.clear();
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(em.emit(i).expect_err("refusal"), ReceiptRefusal::EmptyIdentity);
    }

    #[test]
    fn refuses_alive_without_successful_replay_step() {
        let mut i = input();
        i.replay_steps.iter_mut().for_each(|s| s.exit_code = 1);
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(em.emit(i.clone()).expect_err("refusal"), ReceiptRefusal::AliveWithoutReplay);
        // A failed replay is still receiptable — just not as ALIVE.
        i.standing = Standing::Blocked;
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert!(em.emit(i).is_ok());
    }

    #[test]
    fn refused_standing_requires_reason() {
        let mut i = input();
        i.standing = Standing::Refused;
        i.replay_steps.clear();
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(em.emit(i.clone()).expect_err("refusal"), ReceiptRefusal::RefusedWithoutReason);
        i.refusal_reason = Some("REFUSED_GENERATOR_OWNED".to_string());
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert!(em.emit(i).is_ok());
    }

    #[test]
    fn deletion_requires_first_party_authority_both_ways() {
        // Deletion outside FIRST_PARTY_DELETION is refused.
        let mut i = input();
        i.authority = Authority::Owned;
        i.effects.push(Effect {
            kind: EffectKind::Deleted,
            path: "src/legacy.rs".to_string(),
            detail: "retired module".to_string(),
        });
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(
            em.emit(i.clone()).expect_err("refusal"),
            ReceiptRefusal::DeletionOutsideAuthority
        );
        // Under FIRST_PARTY_DELETION it is admitted.
        i.authority = Authority::FirstPartyDeletion;
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert!(em.emit(i).is_ok());
        // FIRST_PARTY_DELETION without any deletion effect is refused.
        let mut j = input();
        j.authority = Authority::FirstPartyDeletion;
        let mut em = ReceiptOcelEmitter::new("zai-14").expect("actor");
        assert_eq!(
            em.emit(j).expect_err("refusal"),
            ReceiptRefusal::FirstPartyDeletionWithoutDeletion
        );
    }

    #[test]
    fn vocabulary_wire_names_are_pinned() {
        assert_eq!(Authority::Owned.as_str(), "OWNED");
        assert_eq!(Authority::AuthorizedClient.as_str(), "AUTHORIZED_CLIENT");
        assert_eq!(Authority::PublicHeadline.as_str(), "PUBLIC_HEADLINE");
        assert_eq!(Authority::FirstPartyDeletion.as_str(), "FIRST_PARTY_DELETION");
        assert_eq!(Standing::Alive.as_str(), "ALIVE");
        assert_eq!(Standing::PartialAlive.as_str(), "PARTIAL_ALIVE");
        assert_eq!(Standing::Refused.as_str(), "REFUSED");
        assert_eq!(Standing::Blocked.as_str(), "BLOCKED");
        assert_eq!(Standing::Unknown.as_str(), "UNKNOWN");
        assert_eq!(EffectKind::Created.as_str(), "CREATED");
        assert_eq!(EffectKind::Modified.as_str(), "MODIFIED");
        assert_eq!(EffectKind::Deleted.as_str(), "DELETED");
        // serde roundtrip preserves the exact wire vocabulary.
        for a in [
            Authority::Owned,
            Authority::AuthorizedClient,
            Authority::PublicHeadline,
            Authority::FirstPartyDeletion,
        ] {
            let j = serde_json::to_string(&a).expect("serde");
            assert_eq!(j, format!("\"{}\"", a.as_str()));
            let back: Authority = serde_json::from_str(&j).expect("serde");
            assert_eq!(back, a);
        }
        for s in [
            Standing::Alive,
            Standing::PartialAlive,
            Standing::Refused,
            Standing::Blocked,
            Standing::Unknown,
        ] {
            let j = serde_json::to_string(&s).expect("serde");
            assert_eq!(j, format!("\"{}\"", s.as_str()));
            let back: Standing = serde_json::from_str(&j).expect("serde");
            assert_eq!(back, s);
        }
    }

    #[test]
    fn prov_trace_carries_qualified_relations() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let out = em.emit(input()).expect("emission");
        let p = &out.prov;
        let prefixes = p["prefix"].as_object().expect("prefixes");
        assert_eq!(prefixes["prov"].as_str().expect("prov ns"), PROV_NS);
        // Entity surface: receipt + commit head + one entity per file.
        let entities = p["entity"].as_object().expect("entities");
        assert_eq!(entities.len(), 2 + out.receipt.identity.files.len());
        let receipt_id = format!("castle:receipt-{}", &out.receipt_digest[..16]);
        let receipt_entity = &entities[&receipt_id];
        assert_eq!(
            receipt_entity["prov:type"]["$"].as_str().expect("type"),
            "https://castle.dev/ns/receipt-ocel#ChatmanReceipt"
        );
        assert_eq!(
            receipt_entity["castle:standing"]["$"].as_str().expect("standing"),
            "ALIVE"
        );
        // Activity + agent typing.
        let activity_id = format!("castle:emit-{}", &out.receipt_digest[..16]);
        assert_eq!(
            p["activity"][&activity_id]["prov:type"]["$"].as_str().expect("type"),
            "https://castle.dev/ns/receipt-ocel#ReceiptEmission"
        );
        let agent_id = p["agent"].as_object().expect("agents").keys().next().expect("agent").clone();
        assert_eq!(
            p["agent"][&agent_id]["prov:type"]["$"].as_str().expect("type"),
            "http://www.w3.org/ns/prov#SoftwareAgent"
        );
        // used: one per file + the commit head.
        let used = p["used"].as_object().expect("used");
        assert_eq!(used.len(), out.receipt.identity.files.len() + 1);
        assert!(used.values().all(|u| u["prov:activity"] == activity_id));
        // wasGeneratedBy / wasAttributedTo bind receipt to activity and agent.
        assert_eq!(p["wasGeneratedBy"]["_:gen-receipt"]["prov:entity"], receipt_id);
        assert_eq!(p["wasGeneratedBy"]["_:gen-receipt"]["prov:activity"], activity_id);
        assert_eq!(p["wasAttributedTo"]["_:attr-receipt"]["prov:entity"], receipt_id);
        assert_eq!(p["wasAttributedTo"]["_:attr-receipt"]["prov:agent"], agent_id);
        // First emission derives from genesis.
        assert_eq!(p["wasDerivedFrom"]["_:der-receipt"]["prov:generatedEntity"], receipt_id);
        assert_eq!(p["wasDerivedFrom"]["_:der-receipt"]["prov:usedEntity"], "castle:genesis");
        // Second emission derives from the first receipt entity.
        let out2 = em.emit(input()).expect("emission");
        assert_eq!(
            out2.prov["wasDerivedFrom"]["_:der-receipt"]["prov:usedEntity"],
            format!("castle:receipt-{}", &out.receipt_digest[..16])
        );
    }

    #[test]
    fn ocel_trace_carries_v2_shape_and_referential_integrity() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let out = em.emit(input()).expect("emission");
        let o = &out.ocel;
        assert_eq!(o["ocel:version"].as_str().expect("version"), "2.0");
        let types: Vec<&str> = o["ocel:object-types"]
            .as_array()
            .expect("object types")
            .iter()
            .map(|t| t["ocel:name"].as_str().expect("name"))
            .collect();
        assert_eq!(types, vec!["Repo", "Agent", "File", "Receipt"]);
        // Declared attribute types are non-empty for every object type.
        for t in o["ocel:object-types"].as_array().expect("object types") {
            assert!(!t["ocel:attributes"].as_array().expect("attrs").is_empty());
        }
        // object-fields mirror object-types declarations.
        let fields = o["ocel:object-fields"].as_object().expect("fields");
        assert_eq!(fields.len(), 4);
        // Referential integrity: every object_map entry is a declared type and
        // every referenced oid exists in ocel:objects.
        let objects = o["ocel:objects"].as_object().expect("objects");
        for ev in o["ocel:events"].as_array().expect("events") {
            assert!(ev["ocel:eid"].as_str().expect("eid").starts_with("event-"));
            assert!(!ev["ocel:activity"].as_str().expect("activity").is_empty());
            assert!(!ev["ocel:timestamp"].as_str().expect("ts").is_empty());
            let map = ev["ocel:object_map"].as_object().expect("object_map");
            for (ty, oids) in map {
                assert!(types.contains(&ty.as_str()), "undeclared object type {ty}");
                let objs = objects.get(ty).and_then(Value::as_object).unwrap_or_else(|| panic!("missing objects for {ty}"));
                for oid in oids.as_array().expect("oids") {
                    let oid = oid.as_str().expect("oid");
                    assert!(objs.contains_key(oid), "dangling oid {oid}");
                }
            }
            // Event attributes are fully typed name/value/type triples.
            for attr in ev["ocel:attributes"].as_array().expect("attrs") {
                assert!(!attr["ocel:name"].as_str().expect("name").is_empty());
                assert!(!attr["ocel:type"].as_str().expect("type").is_empty());
                assert!(!attr["ocel:value"].is_null());
            }
        }
        // One emission event + one event per replay step.
        let events = o["ocel:events"].as_array().expect("events");
        assert_eq!(events.len(), 1 + out.receipt.replay.steps.len());
        assert_eq!(events[0]["ocel:activity"], "Receipt Emitted");
        assert_eq!(events[1]["ocel:activity"], "Replay Step Executed");
        assert_eq!(events[1]["ocel:attributes"][2]["ocel:name"], "exit_code");
        assert_eq!(events[1]["ocel:attributes"][2]["ocel:type"], "int");
        assert_eq!(events[1]["ocel:attributes"][2]["ocel:value"], 101);
    }

    #[test]
    fn tree_digest_is_path_sorted_order_independent() {
        let f1 = file("src/a.rs", 1);
        let f2 = file("src/b.rs", 2);
        let d1 = tree_digest(&[f1.clone(), f2.clone()]);
        let d2 = tree_digest(&[f2.clone(), f1.clone()]);
        assert_eq!(d1, d2);
        // Recompute per the documented law over the sorted stream.
        let mut expected_stream = String::new();
        let mut sorted = vec![f1, f2];
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        for f in sorted {
            expected_stream.push_str(&f.path);
            expected_stream.push('\n');
            expected_stream.push_str(&f.blake3);
            expected_stream.push('\n');
        }
        assert_eq!(d1, blake3_hex(expected_stream.as_bytes()));
        // Any content change moves the tree digest.
        let f3 = file("src/a.rs", 3);
        assert_ne!(tree_digest(&[f3, file("src/b.rs", 2)]), d1);
    }

    #[test]
    fn canonical_json_is_key_sorted_and_whitespace_free() {
        let v = json!({"b": 2, "a": [1, {"y": true, "x": null}], "c": "s"});
        assert_eq!(
            canonical_json(&v).expect("canonical"),
            r#"{"a":[1,{"x":null,"y":true}],"b":2,"c":"s"}"#
        );
        assert!(canonical_json(&json!({"f": 1.5})).is_ok());
        // Non-finite floats fail closed (law mirrored from trust_chain.rs).
        let nan = serde_json::from_str::<Value>("{\"f\": NaN}").ok();
        if let Some(nan) = nan {
            assert!(canonical_json(&nan).is_err());
        }
    }

    #[test]
    fn refusals_carry_stable_codes_and_display() {
        let cases: Vec<(ReceiptRefusal, &str)> = vec![
            (ReceiptRefusal::EmptyActor, "REFUSED_EMPTY_ACTOR"),
            (
                ReceiptRefusal::InvalidSha { field: "head_commit" },
                "REFUSED_INVALID_SHA",
            ),
            (
                ReceiptRefusal::InvalidBlake3 { path: "p".to_string() },
                "REFUSED_INVALID_BLAKE3",
            ),
            (ReceiptRefusal::EmptyTimestamp, "REFUSED_EMPTY_TIMESTAMP"),
            (ReceiptRefusal::EmptyIdentity, "REFUSED_EMPTY_IDENTITY"),
            (
                ReceiptRefusal::EmptyReplayCommand { index: 3 },
                "REFUSED_EMPTY_REPLAY_COMMAND",
            ),
            (ReceiptRefusal::AliveWithoutReplay, "REFUSED_ALIVE_WITHOUT_REPLAY"),
            (ReceiptRefusal::RefusedWithoutReason, "REFUSED_REFUSAL_WITHOUT_REASON"),
            (
                ReceiptRefusal::DeletionOutsideAuthority,
                "REFUSED_DELETION_OUTSIDE_AUTHORITY",
            ),
            (
                ReceiptRefusal::FirstPartyDeletionWithoutDeletion,
                "REFUSED_FIRST_PARTY_DELETION_WITHOUT_DELETION",
            ),
            (
                ReceiptRefusal::EmptyConsequenceSummary,
                "REFUSED_EMPTY_CONSEQUENCE_SUMMARY",
            ),
        ];
        for (r, code) in cases {
            assert_eq!(r.code(), code);
            assert!(r.to_string().starts_with(code), "Display must lead with the code: {r}");
        }
    }

    #[test]
    fn empty_actor_is_refused_at_emitter_construction() {
        assert_eq!(
            ReceiptOcelEmitter::new("").expect_err("refusal"),
            ReceiptRefusal::EmptyActor
        );
        assert_eq!(
            ReceiptOcelEmitter::new("   ").expect_err("refusal"),
            ReceiptRefusal::EmptyActor
        );
    }

    #[test]
    fn receipt_survives_serde_roundtrip_exactly() {
        let mut em = ReceiptOcelEmitter::new("zai-14-receipt-ocel").expect("actor");
        let out = em.emit(input()).expect("emission");
        let s = serde_json::to_string(&out.receipt).expect("serialize");
        let back: ChatmanReceipt = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back, out.receipt);
        // Durable envelope: receipt + digest + traces serialize cleanly.
        let env = serde_json::to_string(&out).expect("envelope");
        assert!(env.contains("\"receipt_digest\""));
        assert!(env.contains("\"prov\""));
        assert!(env.contains("\"ocel\""));
    }
}
