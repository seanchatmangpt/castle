//! mfact witness corpus byte-pin courts (wave-v1).
//!
//! Pins castle-side a byte-exact copy of the real mfact producer corpus
//! (`/Users/sac/mfact/witnesses.jsonl`, emitted by `lake env lean` +
//! `#print axioms` via `scripts/witness_receipt.py`) under
//! `tests/fixtures/mfact_witnesses.jsonl`, and runs it through the REAL
//! `castle::lean_witness` adapter gates.
//!
//! PIN LAW: the pin tracks the mfact wave-v1 corpus. A corpus change (new
//! witnesses from a landing wave) MUST update the fixture in the SAME commit
//! that lands the wave — the drift court below fails otherwise, naming the
//! delta, and that failure is by design. To re-pin: copy the live file over
//! the fixture and update `PINNED_SHA256`/`PINNED_LINE_COUNT`.
//!
//! As-of pin: 16 lines (PN64 x3, head_unique, StrataNet, StrataStore x5
//! incl. RollbackLaw, StrataLedger, StrataMerkle, StrataValve x4).
//! The pin was re-taken mid-run from the initial 8-line wave-v1 corpus
//! because sibling mfact lanes landed StrataStore.RollbackLaw and
//! StrataValve witnesses concurrently — disclosed per the pin law.

use castle::fortune5::EvidenceEpistemicClass;
use castle::lean_witness::{
    evidence_input_from_lean_witness, ADMITTED_AXIOMS, ADMITTED_SCHEMAS,
};

const FIXTURE: &str = "tests/fixtures/mfact_witnesses.jsonl";
const LIVE_MFACT: &str = "/Users/sac/mfact/witnesses.jsonl";

/// sha256 of the pinned fixture bytes (`shasum -a 256`).
const PINNED_SHA256: &str = "51ea11d2bff3d4c117aa762e1f60273496716673df7bbd4a1bbe529bb22a6518";
/// Pinned line count (wave-v1 corpus as-of this pin).
const PINNED_LINE_COUNT: usize = 16;

fn pinned_bytes() -> String {
    std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|e| panic!("pinned fixture {FIXTURE} unreadable: {e}"))
}

fn pinned_lines() -> Vec<String> {
    pinned_bytes()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(String::from)
        .collect()
}

/// Compute sha256 via the crate's own sha2 dependency.
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    hex(&h.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// Court (a): every pinned line parses under the real adapter gates.
// ---------------------------------------------------------------------------

#[test]
fn all_pinned_witnesses_admit_through_real_adapter() {
    let lines = pinned_lines();
    assert_eq!(
        lines.len(),
        PINNED_LINE_COUNT,
        "fixture line count must match the pin constant"
    );
    for line in &lines {
        let evidence = evidence_input_from_lean_witness(line, "2026-10-13T00:00:00Z")
            .unwrap_or_else(|e| panic!("pinned witness refused: {e}: {line}"));
        let input = evidence.input;
        assert_eq!(input.metric, "lean.witness.verified");
        assert_eq!(input.value, castle::fortune5::MetricValue::Bool(true));
        assert_eq!(
            input.epistemic_class,
            EvidenceEpistemicClass::Replayed,
            "admits as Replayed"
        );
        assert!(
            input.subject.starts_with("mfact@"),
            "subject must carry the mfact source digest: {}",
            input.subject
        );
    }
}

// ---------------------------------------------------------------------------
// Court (b): axiom families — every pinned axiom set ⊆ the Lean core trio.
// ---------------------------------------------------------------------------

#[test]
fn every_pinned_axiom_set_is_within_the_core_trio() {
    for line in &pinned_lines() {
        let value: serde_json::Value = serde_json::from_str(line).expect("fixture line is JSON");
        assert_eq!(
            value.get("schema").and_then(|s| s.as_str()),
            Some(ADMITTED_SCHEMAS[0]),
            "pinned schema identity"
        );
        let axioms = value
            .get("axioms")
            .and_then(|a| a.as_array())
            .expect("axioms array")
            .iter()
            .map(|a| a.as_str().expect("axiom is a string"))
            .collect::<Vec<_>>();
        for axiom in &axioms {
            assert!(
                ADMITTED_AXIOMS.contains(axiom),
                "axiom {axiom:?} outside the core trio in: {line}"
            );
        }
        // source_sha256 must be a non-empty 64-char hex digest.
        let digest = value
            .get("source_sha256")
            .and_then(|s| s.as_str())
            .expect("source_sha256 present");
        assert_eq!(digest.len(), 64, "sha256 hex length in: {line}");
        assert!(
            digest.bytes().all(|b| b.is_ascii_hexdigit()),
            "sha256 hex charset in: {line}"
        );
    }
}

// ---------------------------------------------------------------------------
// Court (c): drift meta-test against the live mfact corpus.
// ---------------------------------------------------------------------------

#[test]
fn fixture_pin_matches_live_mfact_corpus() {
    let Ok(live) = std::fs::read_to_string(LIVE_MFACT) else {
        // BLOCKED:-skip — the mfact producer checkout is not present on this
        // machine; the pin is still self-consistent and is not falsified.
        eprintln!("BLOCKED:-skip: {LIVE_MFACT} absent; drift court vacuous here");
        return;
    };
    let pinned = pinned_bytes();
    if live == pinned {
        return;
    }
    // Fail by design, naming the delta, per the pin law.
    let live_n = live.lines().filter(|l| !l.trim().is_empty()).count();
    panic!(
        "PIN DRIFT: fixture {FIXTURE} (sha256 {PINNED_SHA256}, {PINNED_LINE_COUNT} lines) \
         differs from live {LIVE_MFACT} ({live_n} lines). A corpus change MUST re-pin the \
         fixture in the same commit that lands the wave: copy the live file over the fixture \
         and update PINNED_SHA256/PINNED_LINE_COUNT."
    );
}

// ---------------------------------------------------------------------------
// Court (d): sabotage twin — one flipped axiom must be refused.
// ---------------------------------------------------------------------------

#[test]
fn flipped_axiom_sabotage_is_refused_by_the_adapter() {
    let lines = pinned_lines();
    let victim = lines
        .iter()
        .find(|l| l.contains("\"axioms\":[\"propext\",\"Quot.sound\"]"))
        .expect("corpus contains a propext+Quot.sound witness to sabotage");
    let sabotaged = victim.replace(
        "\"axioms\":[\"propext\",\"Quot.sound\"]",
        "\"axioms\":[\"propext\",\"myCustomAxiom\"]",
    );
    assert_ne!(sabotaged, *victim, "sabotage must mutate the line");
    assert_eq!(
        evidence_input_from_lean_witness(&sabotaged, "2026-10-13T00:00:00Z"),
        Err("REFUSED:LEAN_AXIOMS_NONSTANDARD".to_string()),
        "adapter must refuse the sabotaged axiom set"
    );
}

// ---------------------------------------------------------------------------
// Fixture self-integrity: pinned bytes hash to the pinned digest.
// ---------------------------------------------------------------------------

#[test]
fn fixture_bytes_hash_to_pinned_digest() {
    assert_eq!(
        sha256_hex(pinned_bytes().as_bytes()),
        PINNED_SHA256,
        "fixture bytes drifted from the pinned sha256 — restore or re-pin"
    );
}
