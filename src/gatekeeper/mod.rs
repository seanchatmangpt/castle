//! Air-gapped Gatekeeper — witness envelope verifier & gatekeeper-pack packager
//! (CASTLE-SPEC-26.11.20, lane zai-13-gatekeeper-wasm).
//!
//! The Gatekeeper validates witness envelopes as a **pure function of bytes**:
//! zero network calls, zero filesystem reads, zero wall-clock reads, zero
//! ambient entropy in the verification path. The caller supplies every input;
//! the same envelope bytes always produce the same verdict (determinism twin
//! holds by construction and is pinned by test).
//!
//! # Envelope formats (defined by this module)
//!
//! Both formats are fixed-width, big-endian, and sealed with a trailing
//! blake3 digest over every preceding byte, so any bit flip anywhere in the
//! sealed region is refused. Fixed offsets mean an air-gapped reviewer can
//! verify a frame with nothing but this table — no parser, no network.
//!
//! ## CTRT v1 — Counter-signed Testimony Receipt Transcript (352 bytes)
//!
//! A single-witness attestation: one subject, one artifact, one issuer.
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0 | 4 | magic `CTRT` |
//! | 4 | 4 | format version, u32 BE, must be 1 |
//! | 8 | 64 | subject — 64 ASCII **lowercase** hex chars (sha256 hex) |
//! | 72 | 64 | artifact — 64 ASCII lowercase hex chars |
//! | 136 | 64 | issuer key id — 64 ASCII lowercase hex chars |
//! | 200 | 8 | epoch, u64 BE, must be > 0 (0 = unset, refused) |
//! | 208 | 8 | sequence, u64 BE |
//! | 216 | 104 | reserved — all zero bytes |
//! | 320 | 32 | seal = blake3(bytes 0..320) |
//!
//! ## PCCf v1 — Process Conformance Chain frame, fixed-width (736 bytes)
//!
//! A chained frame: each frame names its parent by the parent's 64-hex
//! identity (conventionally the parent's seal), so a run of frames forms a
//! tamper-evident chain. The all-zeros hex string is the documented genesis
//! parent.
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0 | 4 | magic `PCCf` |
//! | 4 | 4 | format version, u32 BE, must be 1 |
//! | 8 | 64 | subject — 64 ASCII lowercase hex chars |
//! | 72 | 64 | process id — 64 ASCII lowercase hex chars |
//! | 136 | 64 | parent frame id — 64 ASCII lowercase hex chars (all-`0` = genesis) |
//! | 200 | 64 | case id — 64 ASCII lowercase hex chars |
//! | 264 | 8 | activity count, u64 BE |
//! | 272 | 8 | epoch, u64 BE, must be > 0 |
//! | 280 | 8 | sequence, u64 BE |
//! | 288 | 4 | outcome, u32 BE (0 = [`OUTCOME_CONFORMANT`]) |
//! | 292 | 412 | reserved — all zero bytes |
//! | 704 | 32 | seal = blake3(bytes 0..704) |
//!
//! # Gate order (fail-closed, each gate typed)
//!
//! `verify_witness` and the per-kind verifiers run gates in this fixed order
//! and return the FIRST failure, so a diagnosis is never shadowed:
//!
//! 1. [`Refusal::Length`] — envelope length is not exactly 352/736;
//! 2. [`Refusal::KindUnknown`] — `verify_witness` only: length matches a
//!    known envelope but the magic is neither `CTRT` nor `PCCf`;
//! 3. [`Refusal::Magic`] — wrong magic for the kind being verified;
//! 4. [`Refusal::Version`] — format version ≠ 1;
//! 5. [`Refusal::FieldFormat`] — a hash field is not 64 lowercase hex chars,
//!    or epoch is 0 (unset);
//! 6. [`Refusal::Reserved`] — a reserved byte is nonzero (checked before the
//!    seal so the diagnosis names the padding violation, not just the
//!    resulting seal break);
//! 7. [`Refusal::Seal`] — the trailing blake3 seal does not match the sealed
//!    region (the load-bearing gate: every sealed byte is bound).
//!
//! # no_std surface boundary
//!
//! The verification core ([`verify_ctrt`], [`verify_pccf`], [`verify_witness`],
//! the seal check, and the builders) uses only `core` types — fixed arrays,
//! slices, integer byte codecs — and no heap allocation: blake3 is computed
//! into stack arrays. Heap use (`String`, `Vec`, `serde_json`) is confined to
//! the accessor methods, the pack-manifest builder ([`build_pack_manifest`],
//! [`package_witnesses`]), and `#[cfg(test)]` tests. There is no `std::process`,
//! no I/O trait, no clock anywhere in this module.
//!
//! # Packaging
//!
//! [`package_witnesses`] verifies every envelope and admits **only verified
//! witnesses** into a `gatekeeper-pack/v1` manifest (fail-closed: one bad
//! envelope refuses the whole pack, naming the offending index — the pack
//! never partial-admits). The manifest's `pack_digest` is blake3 over the
//! canonical entry lines, with entries sorted by their full field tuple, so
//! the same witness set produces a byte-identical manifest regardless of the
//! order envelopes were handed in.

use blake3::Hasher;

/// CTRT magic bytes.
pub const CTRT_MAGIC: [u8; 4] = *b"CTRT";
/// PCCf magic bytes.
pub const PCCF_MAGIC: [u8; 4] = *b"PCCf";
/// The only admitted envelope format version. A future version must be
/// admitted explicitly, never silently ingested.
pub const FORMAT_VERSION: u32 = 1;
/// CTRT envelope total length in bytes.
pub const CTRT_LEN: usize = 352;
/// PCCf envelope total length in bytes.
pub const PCCF_LEN: usize = 736;
/// Genesis parent identity for a PCCf chain root: 64 ASCII `0` chars.
pub const GENESIS_PARENT: [u8; 64] = [b'0'; 64];
/// PCCf outcome code for a conformant frame.
pub const OUTCOME_CONFORMANT: u32 = 0;
/// The only manifest schema this packager admits.
pub const PACK_SCHEMA: &str = "gatekeeper-pack/v1";
/// Maximum pack id length (ASCII `[A-Za-z0-9._-]` only).
pub const MAX_PACK_ID_LEN: usize = 128;

// CTRT field offsets (sizes in comments; sealed prefix ends at SEAL).
const CTRT_OFF_MAGIC: usize = 0; // 4
const CTRT_OFF_VERSION: usize = 4; // 4
const CTRT_OFF_SUBJECT: usize = 8; // 64
const CTRT_OFF_ARTIFACT: usize = 72; // 64
const CTRT_OFF_ISSUER: usize = 136; // 64
const CTRT_OFF_EPOCH: usize = 200; // 8
const CTRT_OFF_SEQUENCE: usize = 208; // 8
const CTRT_OFF_RESERVED: usize = 216; // 104, all zero
const CTRT_SEALED: usize = 320; // sealed prefix length; seal occupies 320..352

// PCCf field offsets.
const PCCF_OFF_MAGIC: usize = 0; // 4
const PCCF_OFF_VERSION: usize = 4; // 4
const PCCF_OFF_SUBJECT: usize = 8; // 64
const PCCF_OFF_PROCESS: usize = 72; // 64
const PCCF_OFF_PARENT: usize = 136; // 64
const PCCF_OFF_CASE: usize = 200; // 64
const PCCF_OFF_ACTIVITY: usize = 264; // 8
const PCCF_OFF_EPOCH: usize = 272; // 8
const PCCF_OFF_SEQUENCE: usize = 280; // 8
const PCCF_OFF_OUTCOME: usize = 288; // 4
const PCCF_OFF_RESERVED: usize = 292; // 412, all zero
const PCCF_SEALED: usize = 704; // sealed prefix length; seal occupies 704..736

/// A typed refusal from the Gatekeeper. Allocation-free by design: the code
/// string is a compile-time constant, so the whole verification core stays on
/// the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Refusal {
    /// Envelope length is not exactly the fixed width for any known kind.
    Length,
    /// Length matches a known envelope size but the magic is unknown.
    KindUnknown,
    /// Magic bytes do not match the kind being verified.
    Magic,
    /// Format version is not [`FORMAT_VERSION`].
    Version,
    /// A hash field is not 64 lowercase ASCII hex chars, or epoch is unset.
    FieldFormat,
    /// Epoch is 0 — the unset epoch can never be admitted.
    EpochUnset,
    /// A reserved byte is nonzero (padding is part of the transcript).
    Reserved,
    /// The trailing blake3 seal does not bind the sealed region.
    Seal,
    /// Pack id is empty, too long, or has characters outside `[A-Za-z0-9._-]`.
    PackId,
    /// A pack with zero entries admits nothing and is never minted.
    PackEmpty,
}

impl Refusal {
    /// The typed refusal code, house vocabulary (`REFUSED:GK_*`).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Refusal::Length => "REFUSED:GK_ENVELOPE_LENGTH",
            Refusal::KindUnknown => "REFUSED:GK_ENVELOPE_KIND_UNKNOWN",
            Refusal::Magic => "REFUSED:GK_ENVELOPE_MAGIC",
            Refusal::Version => "REFUSED:GK_ENVELOPE_VERSION",
            Refusal::FieldFormat => "REFUSED:GK_ENVELOPE_FIELD_FORMAT",
            Refusal::EpochUnset => "REFUSED:GK_ENVELOPE_EPOCH_UNSET",
            Refusal::Reserved => "REFUSED:GK_ENVELOPE_RESERVED_NOT_ZERO",
            Refusal::Seal => "REFUSED:GK_ENVELOPE_SEAL_MISMATCH",
            Refusal::PackId => "REFUSED:GK_PACK_ID_INVALID",
            Refusal::PackEmpty => "REFUSED:GK_PACK_EMPTY",
        }
    }
}

/// A verified CTRT (counter-signed testimony receipt transcript) envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtrtWitness {
    /// 64 lowercase hex chars identifying the attested subject.
    pub subject: [u8; 64],
    /// 64 lowercase hex chars identifying the attested artifact.
    pub artifact: [u8; 64],
    /// 64 lowercase hex chars identifying the issuer key.
    pub issuer: [u8; 64],
    /// Caller-supplied epoch (> 0); the verifier never reads a clock.
    pub epoch: u64,
    /// Issuer-local monotonic sequence number.
    pub sequence: u64,
    /// blake3 over bytes 0..320 of the envelope.
    pub seal: [u8; 32],
}

impl CtrtWitness {
    /// Subject as a hex string.
    #[must_use]
    pub fn subject_hex(&self) -> String {
        hex_string(&self.subject)
    }

    /// Artifact as a hex string.
    #[must_use]
    pub fn artifact_hex(&self) -> String {
        hex_string(&self.artifact)
    }

    /// Issuer as a hex string.
    #[must_use]
    pub fn issuer_hex(&self) -> String {
        hex_string(&self.issuer)
    }

    /// Seal as a hex string.
    #[must_use]
    pub fn seal_hex(&self) -> String {
        raw_seal_hex(&self.seal)
    }
}

/// A verified PCCf (process conformance chain frame) envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PccfWitness {
    /// 64 lowercase hex chars identifying the attested subject.
    pub subject: [u8; 64],
    /// 64 lowercase hex chars identifying the process definition.
    pub process: [u8; 64],
    /// 64 lowercase hex chars naming the parent frame ([`GENESIS_PARENT`] =
    /// chain root).
    pub parent: [u8; 64],
    /// 64 lowercase hex chars identifying the case.
    pub case: [u8; 64],
    /// Number of activities in the frame.
    pub activity_count: u64,
    /// Caller-supplied epoch (> 0); the verifier never reads a clock.
    pub epoch: u64,
    /// Frame sequence number.
    pub sequence: u64,
    /// Outcome code ([`OUTCOME_CONFORMANT`] = 0 = conformant).
    pub outcome: u32,
    /// blake3 over bytes 0..704 of the envelope.
    pub seal: [u8; 32],
}

impl PccfWitness {
    /// Subject as a hex string.
    #[must_use]
    pub fn subject_hex(&self) -> String {
        hex_string(&self.subject)
    }

    /// Process id as a hex string.
    #[must_use]
    pub fn process_hex(&self) -> String {
        hex_string(&self.process)
    }

    /// Parent as a hex string.
    #[must_use]
    pub fn parent_hex(&self) -> String {
        hex_string(&self.parent)
    }

    /// Case as a hex string.
    #[must_use]
    pub fn case_hex(&self) -> String {
        hex_string(&self.case)
    }

    /// Seal as a hex string.
    #[must_use]
    pub fn seal_hex(&self) -> String {
        raw_seal_hex(&self.seal)
    }

    /// True when the outcome code is [`OUTCOME_CONFORMANT`].
    #[must_use]
    pub fn outcome_is_conformant(&self) -> bool {
        self.outcome == OUTCOME_CONFORMANT
    }
}

/// The verdict of [`verify_witness`]: a fully verified envelope of one kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessVerdict {
    /// A verified CTRT testimony transcript.
    Ctrt(CtrtWitness),
    /// A verified PCCf conformance chain frame.
    Pccf(PccfWitness),
}

impl WitnessVerdict {
    /// The envelope kind, `"CTRT"` or `"PCCF"`.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            WitnessVerdict::Ctrt(_) => "CTRT",
            WitnessVerdict::Pccf(_) => "PCCF",
        }
    }

    /// The kind-specific witness, when this is a CTRT.
    #[must_use]
    pub const fn as_ctrt(&self) -> Option<&CtrtWitness> {
        match self {
            WitnessVerdict::Ctrt(w) => Some(w),
            WitnessVerdict::Pccf(_) => None,
        }
    }

    /// The kind-specific witness, when this is a PCCf.
    #[must_use]
    pub const fn as_pccf(&self) -> Option<&PccfWitness> {
        match self {
            WitnessVerdict::Ctrt(_) => None,
            WitnessVerdict::Pccf(w) => Some(w),
        }
    }

    /// Subject field bytes (validated 64 lowercase hex chars).
    #[must_use]
    pub const fn subject(&self) -> &[u8; 64] {
        match self {
            WitnessVerdict::Ctrt(w) => &w.subject,
            WitnessVerdict::Pccf(w) => &w.subject,
        }
    }

    /// Epoch carried by the envelope.
    #[must_use]
    pub const fn epoch(&self) -> u64 {
        match self {
            WitnessVerdict::Ctrt(w) => w.epoch,
            WitnessVerdict::Pccf(w) => w.epoch,
        }
    }

    /// Sequence number carried by the envelope.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        match self {
            WitnessVerdict::Ctrt(w) => w.sequence,
            WitnessVerdict::Pccf(w) => w.sequence,
        }
    }

    /// Seal bytes.
    #[must_use]
    pub const fn seal(&self) -> &[u8; 32] {
        match self {
            WitnessVerdict::Ctrt(w) => &w.seal,
            WitnessVerdict::Pccf(w) => &w.seal,
        }
    }

    /// Seal as a hex string.
    #[must_use]
    pub fn seal_hex(&self) -> String {
        raw_seal_hex(self.seal())
    }
}

/// Verify a CTRT witness envelope (352 bytes) with the fixed gate order.
///
/// # Errors
/// See the module-level gate order; returns the first failing gate's
/// [`Refusal`].
pub fn verify_ctrt(envelope: &[u8]) -> Result<CtrtWitness, Refusal> {
    if envelope.len() != CTRT_LEN {
        return Err(Refusal::Length);
    }
    if envelope[CTRT_OFF_MAGIC..CTRT_OFF_VERSION] != CTRT_MAGIC[..] {
        return Err(Refusal::Magic);
    }
    if read_u32_be(envelope, CTRT_OFF_VERSION) != FORMAT_VERSION {
        return Err(Refusal::Version);
    }
    let mut subject = [0u8; 64];
    let mut artifact = [0u8; 64];
    let mut issuer = [0u8; 64];
    subject.copy_from_slice(&envelope[CTRT_OFF_SUBJECT..CTRT_OFF_ARTIFACT]);
    artifact.copy_from_slice(&envelope[CTRT_OFF_ARTIFACT..CTRT_OFF_ISSUER]);
    issuer.copy_from_slice(&envelope[CTRT_OFF_ISSUER..CTRT_OFF_EPOCH]);
    if ![&subject, &artifact, &issuer].iter().all(|f| is_lower_hex64(f)) {
        return Err(Refusal::FieldFormat);
    }
    let epoch = read_u64_be(envelope, CTRT_OFF_EPOCH);
    if epoch == 0 {
        return Err(Refusal::EpochUnset);
    }
    let sequence = read_u64_be(envelope, CTRT_OFF_SEQUENCE);
    if envelope[CTRT_OFF_RESERVED..CTRT_SEALED].iter().any(|&b| b != 0) {
        return Err(Refusal::Reserved);
    }
    let mut seal = [0u8; 32];
    seal.copy_from_slice(&envelope[CTRT_SEALED..CTRT_LEN]);
    if !seal_matches(envelope, CTRT_SEALED) {
        return Err(Refusal::Seal);
    }
    Ok(CtrtWitness {
        subject,
        artifact,
        issuer,
        epoch,
        sequence,
        seal,
    })
}

/// Verify a PCCf witness envelope (736 bytes) with the fixed gate order.
///
/// # Errors
/// See the module-level gate order; returns the first failing gate's
/// [`Refusal`].
pub fn verify_pccf(envelope: &[u8]) -> Result<PccfWitness, Refusal> {
    if envelope.len() != PCCF_LEN {
        return Err(Refusal::Length);
    }
    if envelope[PCCF_OFF_MAGIC..PCCF_OFF_VERSION] != PCCF_MAGIC[..] {
        return Err(Refusal::Magic);
    }
    if read_u32_be(envelope, PCCF_OFF_VERSION) != FORMAT_VERSION {
        return Err(Refusal::Version);
    }
    let mut subject = [0u8; 64];
    let mut process = [0u8; 64];
    let mut parent = [0u8; 64];
    let mut case = [0u8; 64];
    subject.copy_from_slice(&envelope[PCCF_OFF_SUBJECT..PCCF_OFF_PROCESS]);
    process.copy_from_slice(&envelope[PCCF_OFF_PROCESS..PCCF_OFF_PARENT]);
    parent.copy_from_slice(&envelope[PCCF_OFF_PARENT..PCCF_OFF_CASE]);
    case.copy_from_slice(&envelope[PCCF_OFF_CASE..PCCF_OFF_ACTIVITY]);
    if ![&subject, &process, &parent, &case]
        .iter()
        .all(|f| is_lower_hex64(f))
    {
        return Err(Refusal::FieldFormat);
    }
    let activity_count = read_u64_be(envelope, PCCF_OFF_ACTIVITY);
    let epoch = read_u64_be(envelope, PCCF_OFF_EPOCH);
    if epoch == 0 {
        return Err(Refusal::EpochUnset);
    }
    let sequence = read_u64_be(envelope, PCCF_OFF_SEQUENCE);
    let outcome = read_u32_be(envelope, PCCF_OFF_OUTCOME);
    if envelope[PCCF_OFF_RESERVED..PCCF_SEALED].iter().any(|&b| b != 0) {
        return Err(Refusal::Reserved);
    }
    let mut seal = [0u8; 32];
    seal.copy_from_slice(&envelope[PCCF_SEALED..PCCF_LEN]);
    if !seal_matches(envelope, PCCF_SEALED) {
        return Err(Refusal::Seal);
    }
    Ok(PccfWitness {
        subject,
        process,
        parent,
        case,
        activity_count,
        epoch,
        sequence,
        outcome,
        seal,
    })
}

/// Verify a witness envelope of either kind, dispatching on size then magic:
///
/// 1. length is neither [`CTRT_LEN`] nor [`PCCF_LEN`] → [`Refusal::Length`]
///    (a truncated frame reports Length, never KindUnknown);
/// 2. magic `CTRT` → [`verify_ctrt`];
/// 3. magic `PCCf` → [`verify_pccf`];
/// 4. otherwise → [`Refusal::KindUnknown`].
///
/// Zero network calls, zero I/O, zero clock: a pure function of `envelope`.
///
/// # Errors
/// See the gate order above; returns the first failing gate's [`Refusal`].
pub fn verify_witness(envelope: &[u8]) -> Result<WitnessVerdict, Refusal> {
    if envelope.len() != CTRT_LEN && envelope.len() != PCCF_LEN {
        return Err(Refusal::Length);
    }
    // Magic occupies bytes 0..4 in both formats.
    if envelope[0..4] == CTRT_MAGIC[..] {
        return verify_ctrt(envelope).map(WitnessVerdict::Ctrt);
    }
    if envelope[0..4] == PCCF_MAGIC[..] {
        return verify_pccf(envelope).map(WitnessVerdict::Pccf);
    }
    Err(Refusal::KindUnknown)
}

/// Build a sealed CTRT envelope from validated fields. The builder and the
/// verifier are the two faces of one format: tests and packagers construct
/// envelopes only through this path, never by forging bytes.
///
/// # Errors
/// [`Refusal::FieldFormat`] on a non-lowercase-hex field;
/// [`Refusal::EpochUnset`] on epoch 0.
pub fn build_ctrt(
    subject: &[u8; 64],
    artifact: &[u8; 64],
    issuer: &[u8; 64],
    epoch: u64,
    sequence: u64,
) -> Result<[u8; CTRT_LEN], Refusal> {
    if ![subject, artifact, issuer]
        .iter()
        .all(|f| is_lower_hex64(f))
    {
        return Err(Refusal::FieldFormat);
    }
    if epoch == 0 {
        return Err(Refusal::EpochUnset);
    }
    let mut buf = [0u8; CTRT_LEN];
    buf[CTRT_OFF_MAGIC..CTRT_OFF_VERSION].copy_from_slice(&CTRT_MAGIC);
    buf[CTRT_OFF_VERSION..CTRT_OFF_SUBJECT].copy_from_slice(&FORMAT_VERSION.to_be_bytes());
    buf[CTRT_OFF_SUBJECT..CTRT_OFF_ARTIFACT].copy_from_slice(subject);
    buf[CTRT_OFF_ARTIFACT..CTRT_OFF_ISSUER].copy_from_slice(artifact);
    buf[CTRT_OFF_ISSUER..CTRT_OFF_EPOCH].copy_from_slice(issuer);
    buf[CTRT_OFF_EPOCH..CTRT_OFF_SEQUENCE].copy_from_slice(&epoch.to_be_bytes());
    buf[CTRT_OFF_SEQUENCE..CTRT_OFF_RESERVED].copy_from_slice(&sequence.to_be_bytes());
    // Reserved bytes stay zero by array initialization.
    seal_into(&mut buf, CTRT_SEALED);
    Ok(buf)
}

/// Build a sealed PCCf envelope from validated fields.
///
/// # Errors
/// [`Refusal::FieldFormat`] on a non-lowercase-hex field;
/// [`Refusal::EpochUnset`] on epoch 0.
#[allow(clippy::too_many_arguments)]
pub fn build_pccf(
    subject: &[u8; 64],
    process: &[u8; 64],
    parent: &[u8; 64],
    case: &[u8; 64],
    activity_count: u64,
    epoch: u64,
    sequence: u64,
    outcome: u32,
) -> Result<[u8; PCCF_LEN], Refusal> {
    if ![subject, process, parent, case]
        .iter()
        .all(|f| is_lower_hex64(f))
    {
        return Err(Refusal::FieldFormat);
    }
    if epoch == 0 {
        return Err(Refusal::EpochUnset);
    }
    let mut buf = [0u8; PCCF_LEN];
    buf[PCCF_OFF_MAGIC..PCCF_OFF_VERSION].copy_from_slice(&PCCF_MAGIC);
    buf[PCCF_OFF_VERSION..PCCF_OFF_SUBJECT].copy_from_slice(&FORMAT_VERSION.to_be_bytes());
    buf[PCCF_OFF_SUBJECT..PCCF_OFF_PROCESS].copy_from_slice(subject);
    buf[PCCF_OFF_PROCESS..PCCF_OFF_PARENT].copy_from_slice(process);
    buf[PCCF_OFF_PARENT..PCCF_OFF_CASE].copy_from_slice(parent);
    buf[PCCF_OFF_CASE..PCCF_OFF_ACTIVITY].copy_from_slice(case);
    buf[PCCF_OFF_ACTIVITY..PCCF_OFF_EPOCH].copy_from_slice(&activity_count.to_be_bytes());
    buf[PCCF_OFF_EPOCH..PCCF_OFF_SEQUENCE].copy_from_slice(&epoch.to_be_bytes());
    buf[PCCF_OFF_SEQUENCE..PCCF_OFF_OUTCOME].copy_from_slice(&sequence.to_be_bytes());
    buf[PCCF_OFF_OUTCOME..PCCF_OFF_RESERVED].copy_from_slice(&outcome.to_be_bytes());
    // Reserved bytes stay zero by array initialization.
    seal_into(&mut buf, PCCF_SEALED);
    Ok(buf)
}

/// One admitted entry of a gatekeeper pack — always derived from a
/// [`WitnessVerdict`], never authored by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackEntry {
    /// Envelope kind: `"CTRT"` or `"PCCF"`.
    pub kind: &'static str,
    /// Subject hex string.
    pub subject: String,
    /// Kind-specific secondary hash: the artifact (CTRT) or the case (PCCF).
    pub ref_hash: String,
    /// Envelope epoch.
    pub epoch: u64,
    /// Envelope sequence.
    pub sequence: u64,
    /// Seal hex string.
    pub seal: String,
}

impl PackEntry {
    /// Derive a pack entry from a verified witness.
    #[must_use]
    pub fn from_verdict(verdict: &WitnessVerdict) -> PackEntry {
        match verdict {
            WitnessVerdict::Ctrt(w) => PackEntry {
                kind: "CTRT",
                subject: w.subject_hex(),
                ref_hash: w.artifact_hex(),
                epoch: w.epoch,
                sequence: w.sequence,
                seal: w.seal_hex(),
            },
            WitnessVerdict::Pccf(w) => PackEntry {
                kind: "PCCF",
                subject: w.subject_hex(),
                ref_hash: w.case_hex(),
                epoch: w.epoch,
                sequence: w.sequence,
                seal: w.seal_hex(),
            },
        }
    }
}

/// A packager failure: either a pack-level gate or a bad envelope at a
/// specific index (fail-closed — the pack never partial-admits).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// A pack-level gate refused.
    Pack(Refusal),
    /// The envelope at `index` failed verification.
    Witness {
        /// Zero-based index of the refusing envelope.
        index: usize,
        /// The envelope's typed refusal.
        reason: Refusal,
    },
}

impl PackError {
    /// Fully qualified refusal code; envelope failures carry their index.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            PackError::Pack(r) => r.code().to_string(),
            PackError::Witness { index, reason } => format!(
                "REFUSED:GK_PACK_ENTRY_{}:{}",
                index,
                reason.code().trim_start_matches("REFUSED:GK_")
            ),
        }
    }
}

/// Build a `gatekeeper-pack/v1` manifest over verified entries.
///
/// Entries are sorted by their full field tuple and the `pack_digest` is
/// blake3 over each entry's canonical `kind|subject|ref_hash|epoch|sequence|seal`
/// line concatenated in sorted order, so the manifest is a deterministic
/// function of the witness set alone (no wall-clock, no ambient state).
///
/// # Errors
/// [`Refusal::PackId`] on an invalid pack id; [`Refusal::PackEmpty`] on zero
/// entries.
pub fn build_pack_manifest(pack_id: &str, entries: &[PackEntry]) -> Result<serde_json::Value, Refusal> {
    if !pack_id_valid(pack_id) {
        return Err(Refusal::PackId);
    }
    if entries.is_empty() {
        return Err(Refusal::PackEmpty);
    }
    let mut sorted: Vec<&PackEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        (
            a.kind,
            a.subject.as_str(),
            a.ref_hash.as_str(),
            a.epoch,
            a.sequence,
            a.seal.as_str(),
        )
            .cmp(&(
                b.kind,
                b.subject.as_str(),
                b.ref_hash.as_str(),
                b.epoch,
                b.sequence,
                b.seal.as_str(),
            ))
    });
    let mut hasher = Hasher::new();
    for entry in &sorted {
        hasher.update(canonical_entry_line(entry).as_bytes());
    }
    let pack_digest = hasher.finalize().to_hex().to_string();
    let entries_json: Vec<serde_json::Value> = sorted
        .iter()
        .map(|e| {
            serde_json::json!({
                "kind": e.kind,
                "subject": e.subject,
                "ref_hash": e.ref_hash,
                "epoch": e.epoch,
                "sequence": e.sequence,
                "seal": e.seal,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "schema": PACK_SCHEMA,
        "pack_id": pack_id,
        "entry_count": entries.len(),
        "pack_digest": pack_digest,
        "entries": entries_json,
    }))
}

/// Verify every envelope and package the verified witnesses into a
/// gatekeeper-pack manifest. Fail-closed: the first bad envelope refuses the
/// whole pack with its index; nothing partial is ever emitted.
///
/// # Errors
/// [`PackError::Witness`] when the envelope at an index fails verification;
/// [`PackError::Pack`] when a pack-level gate refuses.
pub fn package_witnesses(pack_id: &str, envelopes: &[&[u8]]) -> Result<serde_json::Value, PackError> {
    let mut entries = Vec::with_capacity(envelopes.len());
    for (index, envelope) in envelopes.iter().enumerate() {
        let verdict = verify_witness(envelope)
            .map_err(|reason| PackError::Witness { index, reason })?;
        entries.push(PackEntry::from_verdict(&verdict));
    }
    build_pack_manifest(pack_id, &entries).map_err(PackError::Pack)
}

// ---------------------------------------------------------------------------
// Core helpers — `core` types only, allocation-free.
// ---------------------------------------------------------------------------

/// blake3 seal over `buf[..sealed_len]`, written into `buf[sealed_len..sealed_len+32]`.
fn seal_into(buf: &mut [u8], sealed_len: usize) {
    let seal = blake3::hash(&buf[..sealed_len]);
    buf[sealed_len..sealed_len + 32].copy_from_slice(seal.as_bytes());
}

/// True when the trailing 32 bytes equal blake3 of the sealed prefix.
fn seal_matches(envelope: &[u8], sealed_len: usize) -> bool {
    let expected = blake3::hash(&envelope[..sealed_len]);
    envelope[sealed_len..] == expected.as_bytes()[..]
}

/// 64 lowercase ASCII hex chars only (canonical form; uppercase refuses).
fn is_lower_hex64(field: &[u8; 64]) -> bool {
    field
        .iter()
        .all(|&b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}

fn read_u64_be(buf: &[u8], offset: usize) -> u64 {
    let mut word = [0u8; 8];
    word.copy_from_slice(&buf[offset..offset + 8]);
    u64::from_be_bytes(word)
}

fn read_u32_be(buf: &[u8], offset: usize) -> u32 {
    let mut word = [0u8; 4];
    word.copy_from_slice(&buf[offset..offset + 4]);
    u32::from_be_bytes(word)
}

/// Validated hex field bytes to string (fields are ASCII by admission gate).
fn hex_string(field: &[u8]) -> String {
    field.iter().map(|&b| b as char).collect()
}

/// Seal bytes to lowercase hex.
fn raw_seal_hex(seal: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for byte in seal {
        s.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        s.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap_or('0'));
    }
    s
}

/// Pack id: non-empty, ≤ [`MAX_PACK_ID_LEN`], ASCII `[A-Za-z0-9._-]`.
fn pack_id_valid(pack_id: &str) -> bool {
    !pack_id.is_empty()
        && pack_id.len() <= MAX_PACK_ID_LEN
        && pack_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
}

/// Canonical digest line for one pack entry (documented manifest format).
fn canonical_entry_line(entry: &PackEntry) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}\n",
        entry.kind, entry.subject, entry.ref_hash, entry.epoch, entry.sequence, entry.seal
    )
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic valid 64-char lowercase hex field from a seed — the
    /// alphabet cycles with offset, so distinct seeds give distinct fields.
    fn hex64(seed: u8) -> [u8; 64] {
        let alphabet = b"0123456789abcdef";
        let mut field = [0u8; 64];
        for (i, slot) in field.iter_mut().enumerate() {
            *slot = alphabet[usize::from((seed + u8::try_from(i % 16).unwrap()) % 16)];
        }
        field
    }

    fn hex64_str(seed: u8) -> String {
        hex_string(&hex64(seed))
    }

    fn valid_ctrt(epoch: u64, sequence: u64) -> [u8; CTRT_LEN] {
        build_ctrt(&hex64(0), &hex64(1), &hex64(2), epoch, sequence).expect("valid fields build")
    }

    fn valid_pccf(epoch: u64, sequence: u64) -> [u8; PCCF_LEN] {
        build_pccf(
            &hex64(3),
            &hex64(4),
            &hex64(5),
            &hex64(6),
            7,
            epoch,
            sequence,
            OUTCOME_CONFORMANT,
        )
        .expect("valid fields build")
    }

    #[test]
    const fn format_lengths_are_pinned() {
        // The wire contract: 352 and 736, no drift.
        let _ = CTRT_LEN; // const-context use; values asserted below at runtime
    }

    #[test]
    fn envelope_lengths_are_352_and_736() {
        assert_eq!(CTRT_LEN, 352);
        assert_eq!(PCCF_LEN, 736);
    }

    #[test]
    fn valid_ctrt_roundtrips_through_the_verifier() {
        let envelope = valid_ctrt(1_786_824_000_000, 42);
        let witness = verify_ctrt(&envelope).expect("built envelope verifies");
        assert_eq!(witness.subject, hex64(0));
        assert_eq!(witness.artifact, hex64(1));
        assert_eq!(witness.issuer, hex64(2));
        assert_eq!(witness.epoch, 1_786_824_000_000);
        assert_eq!(witness.sequence, 42);
        assert_eq!(witness.seal_hex().len(), 64);
        // The seal independently re-derives from the sealed prefix.
        assert_eq!(witness.seal, *blake3::hash(&envelope[..CTRT_SEALED]).as_bytes());
    }

    #[test]
    fn valid_pccf_roundtrips_through_the_verifier() {
        let envelope = valid_pccf(1_786_824_000_000, 9);
        let witness = verify_pccf(&envelope).expect("built envelope verifies");
        assert_eq!(witness.subject, hex64(3));
        assert_eq!(witness.process, hex64(4));
        assert_eq!(witness.parent, hex64(5));
        assert_eq!(witness.case, hex64(6));
        assert_eq!(witness.activity_count, 7);
        assert_eq!(witness.epoch, 1_786_824_000_000);
        assert_eq!(witness.sequence, 9);
        assert_eq!(witness.outcome, OUTCOME_CONFORMANT);
        assert!(witness.outcome_is_conformant());
        assert_eq!(witness.seal, *blake3::hash(&envelope[..PCCF_SEALED]).as_bytes());
    }

    #[test]
    fn genesis_parent_is_a_valid_chain_root() {
        let envelope = build_pccf(
            &hex64(3),
            &hex64(4),
            &GENESIS_PARENT,
            &hex64(6),
            1,
            1000,
            0,
            OUTCOME_CONFORMANT,
        )
        .expect("genesis frame builds");
        let witness = verify_pccf(&envelope).expect("genesis frame verifies");
        assert_eq!(witness.parent, GENESIS_PARENT);
    }

    #[test]
    fn pccf_chain_links_parent_to_parent_seal() {
        // Chain convention: frame 1's parent identity is frame 0's seal hex,
        // so tampering with frame 0 (its seal changes) is detectable from
        // frame 1 alone.
        let frame0 = verify_pccf(&valid_pccf(1000, 0)).expect("frame0 verifies");
        let mut parent = [0u8; 64];
        let parent_hex = frame0.seal_hex();
        parent.copy_from_slice(parent_hex.as_bytes());
        let child_envelope = build_pccf(
            &hex64(3),
            &hex64(4),
            &parent,
            &hex64(6),
            2,
            1001,
            1,
            OUTCOME_CONFORMANT,
        )
        .expect("child builds");
        let frame1 = verify_pccf(&child_envelope).expect("child verifies");
        assert_eq!(frame1.parent_hex(), frame0.seal_hex());
        // A different valid frame0 (different case bytes) carries a different
        // seal, so the child's link no longer names it — chain drift is
        // detectable from frame 1 alone.
        let other_frame0 = verify_pccf(&build_pccf(
            &hex64(3),
            &hex64(4),
            &GENESIS_PARENT,
            &hex64(7), // different case than hex64(6)
            1,
            1000,
            0,
            OUTCOME_CONFORMANT,
        )
        .expect("other frame0 builds"))
        .expect("other frame0 verifies");
        assert_ne!(other_frame0.seal_hex(), frame1.parent_hex());
    }

    #[test]
    fn verify_witness_dispatches_both_kinds() {
        let ct = verify_witness(&valid_ctrt(1000, 1)).expect("CTRT dispatches");
        assert_eq!(ct.kind(), "CTRT");
        assert!(ct.as_ctrt().is_some());
        assert!(ct.as_pccf().is_none());
        assert_eq!(ct.subject(), &hex64(0));
        assert_eq!(ct.epoch(), 1000);

        let pc = verify_witness(&valid_pccf(1000, 2)).expect("PCCf dispatches");
        assert_eq!(pc.kind(), "PCCF");
        assert!(pc.as_pccf().is_some());
        assert!(pc.as_ctrt().is_none());
        assert_eq!(pc.sequence(), 2);
    }

    #[test]
    fn wrong_length_refuses_as_length_never_kind_unknown() {
        assert_eq!(verify_witness(&[]), Err(Refusal::Length));
        assert_eq!(verify_witness(&valid_ctrt(1000, 1)[..351]), Err(Refusal::Length));
        let mut extended = valid_ctrt(1000, 1).to_vec();
        extended.push(0);
        assert_eq!(verify_witness(&extended), Err(Refusal::Length));
        assert_eq!(verify_witness(&valid_pccf(1000, 1)[..735]), Err(Refusal::Length));
        assert_eq!(verify_ctrt(&[0u8; 100]), Err(Refusal::Length));
        assert_eq!(verify_pccf(&[0u8; 1000]), Err(Refusal::Length));
    }

    #[test]
    fn known_size_unknown_magic_refuses_as_kind_unknown() {
        assert_eq!(verify_witness(&[0u8; CTRT_LEN]), Err(Refusal::KindUnknown));
        assert_eq!(verify_witness(&[0u8; PCCF_LEN]), Err(Refusal::KindUnknown));
        assert_eq!(verify_witness(b"XXXX".repeat(88).as_slice()), Err(Refusal::KindUnknown));
    }

    #[test]
    fn wrong_magic_refuses_before_the_seal_gate() {
        let mut envelope = valid_ctrt(1000, 1);
        envelope[0] = b'X'; // magic corrupted (seal also broken; magic fires first)
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::Magic));
        let mut frame = valid_pccf(1000, 1);
        frame[3] = b'F'; // "PCCF" instead of "PCCf" — case matters
        assert_eq!(verify_pccf(&frame), Err(Refusal::Magic));
    }

    #[test]
    fn version_flip_refuses() {
        let mut envelope = valid_ctrt(1000, 1);
        envelope[4] = 2;
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::Version));
        let mut frame = valid_pccf(1000, 1);
        frame[7] = 9;
        assert_eq!(verify_pccf(&frame), Err(Refusal::Version));
    }

    #[test]
    fn non_lowercase_hex_field_refuses_as_field_format() {
        // Uppercase is not canonical — byte-identical replay discipline.
        let mut upper = hex64(0);
        upper[0] = b'A';
        assert_eq!(
            build_ctrt(&upper, &hex64(1), &hex64(2), 1000, 1),
            Err(Refusal::FieldFormat)
        );
        // Non-hex character in a field, mutated post-build: the field gate
        // fires before the seal gate.
        let mut envelope = valid_ctrt(1000, 1);
        envelope[CTRT_OFF_SUBJECT] = b'g';
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::FieldFormat));
        // Same on the PCCf parent field.
        let mut frame = valid_pccf(1000, 1);
        frame[PCCF_OFF_PARENT] = b'z';
        assert_eq!(verify_pccf(&frame), Err(Refusal::FieldFormat));
    }

    #[test]
    fn epoch_zero_is_the_unset_epoch_and_never_admitted() {
        assert_eq!(
            build_ctrt(&hex64(0), &hex64(1), &hex64(2), 0, 1),
            Err(Refusal::EpochUnset)
        );
        assert_eq!(
            build_pccf(&hex64(3), &hex64(4), &hex64(5), &hex64(6), 1, 0, 1, 0),
            Err(Refusal::EpochUnset)
        );
        let mut envelope = valid_ctrt(1000, 1);
        for slot in &mut envelope[CTRT_OFF_EPOCH..CTRT_OFF_SEQUENCE] {
            *slot = 0;
        }
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::EpochUnset));
    }

    #[test]
    fn nonzero_reserved_byte_refuses_before_the_seal_gate() {
        let mut envelope = valid_ctrt(1000, 1);
        envelope[CTRT_OFF_RESERVED] = 1;
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::Reserved));
        let mut frame = valid_pccf(1000, 1);
        frame[PCCF_SEALED - 1] = 0xAA;
        assert_eq!(verify_pccf(&frame), Err(Refusal::Reserved));
    }

    #[test]
    fn any_sealed_bit_flip_refuses_as_seal_mismatch() {
        // Flip in a hex field (still valid hex, so only the seal can refuse).
        let mut envelope = valid_ctrt(1000, 1);
        let first = envelope[CTRT_OFF_ISSUER];
        envelope[CTRT_OFF_ISSUER] = if first == b'0' { b'1' } else { b'0' };
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::Seal));
        // Flip the first seal byte itself.
        let mut envelope = valid_ctrt(1000, 1);
        envelope[CTRT_SEALED] ^= 0xFF;
        assert_eq!(verify_ctrt(&envelope), Err(Refusal::Seal));
        // Flip the very last byte of a PCCf frame.
        let mut frame = valid_pccf(1000, 1);
        let last = frame[PCCF_LEN - 1];
        frame[PCCF_LEN - 1] = if last == 0 { 1 } else { 0 };
        assert_eq!(verify_pccf(&frame), Err(Refusal::Seal));
    }

    #[test]
    fn seal_binds_every_field_including_epoch_and_sequence() {
        let a = verify_ctrt(&valid_ctrt(1000, 1)).expect("a verifies");
        let b = verify_ctrt(&valid_ctrt(1000, 2)).expect("b verifies");
        let c = verify_ctrt(&valid_ctrt(2000, 1)).expect("c verifies");
        assert_ne!(a.seal, b.seal, "sequence is sealed");
        assert_ne!(a.seal, c.seal, "epoch is sealed");
    }

    #[test]
    fn determinism_twin_identical_inputs_identical_outputs() {
        // Gate-3 doctrine: same bytes twice, identical verdict, no ambient
        // state (no clock, no entropy, no network) can leak into the verdict.
        let envelope = valid_ctrt(1000, 1);
        assert_eq!(verify_ctrt(&envelope), verify_ctrt(&envelope));
        assert_eq!(verify_witness(&envelope), verify_witness(&envelope));
        assert_eq!(valid_ctrt(1000, 1), valid_ctrt(1000, 1));
        let frame = valid_pccf(1000, 1);
        assert_eq!(verify_pccf(&frame), verify_pccf(&frame));
    }

    #[test]
    fn refusal_codes_use_the_house_vocabulary() {
        assert_eq!(Refusal::Length.code(), "REFUSED:GK_ENVELOPE_LENGTH");
        assert_eq!(Refusal::KindUnknown.code(), "REFUSED:GK_ENVELOPE_KIND_UNKNOWN");
        assert_eq!(Refusal::Seal.code(), "REFUSED:GK_ENVELOPE_SEAL_MISMATCH");
        assert_eq!(Refusal::PackEmpty.code(), "REFUSED:GK_PACK_EMPTY");
    }

    #[test]
    fn pack_manifest_carries_only_verified_entries_in_fixed_shape() {
        let ct = valid_ctrt(1000, 1);
        let pc = valid_pccf(1000, 2);
        let manifest = package_witnesses("audit-pack", &[&ct, &pc]).expect("pack builds");
        assert_eq!(manifest["schema"], PACK_SCHEMA);
        assert_eq!(manifest["pack_id"], "audit-pack");
        assert_eq!(manifest["entry_count"], 2);
        assert_eq!(manifest["entries"].as_array().map(Vec::len), Some(2));
        let entries = manifest["entries"].as_array().expect("entries array");
        // Sorted order: "CTRT" < "PCCF".
        assert_eq!(entries[0]["kind"], "CTRT");
        assert_eq!(
            entries[0]["subject"],
            verify_ctrt(&ct).expect("verifies").subject_hex()
        );
        assert_eq!(
            entries[0]["ref_hash"],
            verify_ctrt(&ct).expect("verifies").artifact_hex()
        );
        assert_eq!(entries[1]["kind"], "PCCF");
        assert_eq!(
            entries[1]["ref_hash"],
            verify_pccf(&pc).expect("verifies").case_hex()
        );
        // Exactly the documented key set — no fabricated fields.
        let keys: Vec<&str> = manifest
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["entries", "entry_count", "pack_digest", "pack_id", "schema"]);
    }

    #[test]
    fn pack_manifest_is_deterministic_under_input_permutation() {
        let ct = valid_ctrt(1000, 1);
        let pc = valid_pccf(1000, 2);
        let ct2 = valid_ctrt(1000, 3);
        let a = package_witnesses("p", &[&ct, &pc, &ct2]).expect("pack builds");
        let b = package_witnesses("p", &[&ct2, &pc, &ct]).expect("pack builds");
        assert_eq!(a, b, "same witness set, different input order, same manifest");
    }

    #[test]
    fn pack_digest_changes_when_a_witness_changes() {
        let ct = valid_ctrt(1000, 1);
        let mutated = valid_ctrt(1000, 2);
        let a = package_witnesses("p", &[&ct]).expect("pack builds");
        let b = package_witnesses("p", &[&mutated]).expect("pack builds");
        assert_ne!(a["pack_digest"], b["pack_digest"]);
    }

    #[test]
    fn pack_id_gates_refuse_invalid_ids() {
        let ct = valid_ctrt(1000, 1);
        for bad in ["", "has space", "slash/ed", "tab\tbed", &"x".repeat(MAX_PACK_ID_LEN + 1)] {
            assert_eq!(
                package_witnesses(bad, &[&ct]),
                Err(PackError::Pack(Refusal::PackId)),
                "pack id {bad:?} must refuse"
            );
        }
        // Boundary: exactly MAX_PACK_ID_LEN of legal characters admits.
        let ok_id = "a".repeat(MAX_PACK_ID_LEN);
        assert!(package_witnesses(&ok_id, &[&ct]).is_ok());
    }

    #[test]
    fn empty_pack_never_mints() {
        assert_eq!(
            package_witnesses("p", &[]),
            Err(PackError::Pack(Refusal::PackEmpty))
        );
        assert_eq!(
            build_pack_manifest("p", &[]),
            Err(Refusal::PackEmpty)
        );
    }

    #[test]
    fn bad_envelope_refuses_the_whole_pack_naming_the_index() {
        let good = valid_ctrt(1000, 1);
        let truncated = &valid_pccf(1000, 2)[..735];
        assert_eq!(
            package_witnesses("p", &[&good, truncated]),
            Err(PackError::Witness {
                index: 1,
                reason: Refusal::Length
            })
        );
        let err = package_witnesses("p", &[&good, truncated])
            .expect_err("refuses")
            .code();
        assert_eq!(err, "REFUSED:GK_PACK_ENTRY_1:ENVELOPE_LENGTH");
    }

    #[test]
    fn pack_entry_from_verdict_carries_the_witness_fields() {
        let ct = verify_witness(&valid_ctrt(1000, 1)).expect("verifies");
        let entry = PackEntry::from_verdict(&ct);
        assert_eq!(entry.kind, "CTRT");
        assert_eq!(entry.subject, hex64_str(0));
        assert_eq!(entry.ref_hash, hex64_str(1));
        assert_eq!(entry.epoch, 1000);
        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.seal, ct.seal_hex());

        let pc = verify_witness(&valid_pccf(2000, 2)).expect("verifies");
        let entry = PackEntry::from_verdict(&pc);
        assert_eq!(entry.kind, "PCCF");
        assert_eq!(entry.ref_hash, hex64_str(6), "PCCf ref_hash is the case id");
        assert_eq!(entry.epoch, 2000);
    }
}
