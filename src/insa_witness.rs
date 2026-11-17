//! insa sync witness → Replayed evidence adapter (L32 rank-1, L179).
//!
//! Ingests an insa-witness JSON line (schema `insa-witness/v1`, the L32
//! rank-1 format emitted by insa `cargo test` sync-roundtrip runs) and lifts
//! it into a castle `Replayed` evidence input.
//!
//! Family-aware (L196): insa's witnesses are Rust-compiled `cargo test`
//! replays, honestly declared with the axiom marker
//! `rust-compiled:no-lean-kernel` — a Rust producer runs no Lean kernel and
//! cannot emit the Lean axiom trio. Both execution families lift as
//! `Replayed` (a Rust cargo-test run is a deterministic replay too); the
//! family is recorded in the evidence subject (`#lean` / `#rust`). A
//! witness carrying `refused: true` is evidence OF a refusal and lifts to
//! the typed refusal `REFUSED:INSA_WITNESS_IS_REFUSAL`, never to evidence.
//!
//! Two load-bearing gates: the AXIOM gate (mirrors [`crate::lean_witness`] —
//! the theorem is sorry-free only if its axiom footprint is exactly the Lean
//! core trio) and the HEAD-EQUALITY gate — a sync roundtrip is only `Replayed`
//! if `sync_head == anchor_head`; divergence is a typed refusal, never
//! admitted evidence.
//!
//! Schema gate mirrors `crates/castle-ggen-rdf/src/strata.rs`
//! (`ADMITTED_SCHEMAS`, fail-closed on unknown/missing schema).

use crate::board::EvidenceInput;
use crate::fortune5::{EvidenceEpistemicClass, MetricValue};

/// Refusal prefix used by every typed refusal in this module.
pub const REFUSED: &str = "REFUSED";

/// The only schema identity this adapter admits. A future schema must be
/// admitted explicitly, never silently ingested.
pub const ADMITTED_SCHEMAS: [&str; 1] = ["insa-witness/v1"];

/// The Lean core axiom trio — the exact sorry-freedom footprint `#print
/// axioms` is allowed to report for an admitted theorem.
pub const ADMITTED_AXIOMS: [&str; 3] = ["propext", "Classical.choice", "Quot.sound"];

/// The marker axiom emitted by the Rust execution family. A Rust `cargo
/// test` replay runs no Lean kernel at all, so the honest producer declares
/// this single marker instead of fabricating Lean axiom names.
pub const RUST_FAMILY_MARKER: &str = "rust-compiled:no-lean-kernel";

/// Execution families an admitted witness may belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxiomFamily {
    /// A Lean-kernel proof; axiom footprint ⊆ [`ADMITTED_AXIOMS`].
    LeanTrio,
    /// A Rust `cargo test` deterministic replay; axioms are exactly the
    /// [`RUST_FAMILY_MARKER`] singleton.
    RustCompiled,
}

impl AxiomFamily {
    /// Subject tag recording the execution family (L196 adjudication: both
    /// families lift as `Replayed` — a Rust cargo-test run is a deterministic
    /// replay too — so the family is carried in the subject, not the
    /// epistemic class).
    pub fn tag(self) -> &'static str {
        match self {
            AxiomFamily::LeanTrio => "lean",
            AxiomFamily::RustCompiled => "rust",
        }
    }
}

/// Classify a witness axiom set into its execution family, or refuse.
///
/// Family-aware gate (L196): the axiom field must be EITHER a subset of the
/// Lean trio (Lean-kernel proof) OR exactly the Rust marker singleton.
/// Anything else — a mixed set (marker + Lean names), an empty set, or any
/// unknown axiom (`sorryAx` above all) — refuses.
///
/// Residual risk, documented honestly: a Rust producer cannot emit the trio
/// (it runs no Lean kernel), so a witness carrying the trio is trusted to
/// come from a Lean producer. The fabrication risk shifts to producer
/// trust — castle verifies the FAMILY CONSISTENCY of the claim, not the
/// identity of its author. The marker path is the honest declaration; the
/// trio path is unverifiable-at-the-adapter and is admitted on producer
/// trust alone.
pub fn axiom_family(axioms: &[String]) -> Result<AxiomFamily, String> {
    if axioms.len() == 1 && axioms[0] == RUST_FAMILY_MARKER {
        return Ok(AxiomFamily::RustCompiled);
    }
    if axioms.is_empty() {
        return Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string());
    }
    if axioms.iter().any(|a| a == RUST_FAMILY_MARKER) {
        // Mixed family: the marker is only ever a singleton.
        return Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string());
    }
    if axioms.iter().all(|a| ADMITTED_AXIOMS.contains(&a.as_str())) {
        return Ok(AxiomFamily::LeanTrio);
    }
    Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
}

/// A parsed insa-witness line.
#[derive(Debug, Clone, PartialEq)]
pub struct InsaWitness {
    pub theorem: String,
    pub axioms: Vec<String>,
    /// Hex sha256 of the insa source the theorem was proven against.
    pub source_sha256: String,
    /// Hex sha256 of the sync tip the roundtrip ran against.
    pub sync_head: String,
    /// Number of verified roundtrip entries.
    pub verified_entries: u64,
    /// Hex sha256 of the anchor head the sync must land on.
    pub anchor_head: String,
}

/// A verified `Replayed` evidence admission derived from an insa witness.
#[derive(Debug, Clone, PartialEq)]
pub struct InsaWitnessEvidence {
    pub input: EvidenceInput,
}

/// Parse an insa-witness JSON line and lift it into a `Replayed`
/// `EvidenceInput`, or a typed refusal.
///
/// Gates, in order:
/// 1. schema gate — `REFUSED:INSA_SCHEMA_UNKNOWN` on missing/unknown
///    `schema` (fail-closed, mirrors strata.rs);
/// 2. refusal-awareness gate — `REFUSED:INSA_WITNESS_IS_REFUSAL` when
///    `refused: true` (a refusal witness is evidence OF a refusal, never
///    `Replayed` success evidence; checked before all later gates so the
///    typed refusal is never shadowed);
/// 3. axiom gate (family-aware) — `REFUSED:INSA_AXIOMS_NONSTANDARD` unless
///    the axiom set is a subset of [`ADMITTED_AXIOMS`] (Lean family) or
///    exactly [`RUST_FAMILY_MARKER`] (Rust family); mixed sets, empty sets,
///    and anything outside both (`sorryAx` above all) refuse — see
///    [`axiom_family`];
/// 4. head-equality gate — `REFUSED:INSA_SYNC_DIVERGED` unless
///    `sync_head == anchor_head` (a roundtrip replayed against a diverged
///    head proves nothing about the anchor);
/// 5. required-field gate — `REFUSED:INSA_WITNESS_INCOMPLETE` on empty
///    theorem / source digest / malformed or non-64-hex heads / non-numeric
///    `verified_entries`.
///
/// # Errors
/// See the gate list above.
pub fn evidence_input_from_insa_witness(
    line: &str,
    observed_at: &str,
) -> Result<InsaWitnessEvidence, String> {
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|_| "REFUSED:INSA_WITNESS_MALFORMED_JSON".to_string())?;
    let schema = value
        .get("schema")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:INSA_SCHEMA_UNKNOWN".to_string())?;
    if !ADMITTED_SCHEMAS.contains(&schema) {
        return Err("REFUSED:INSA_SCHEMA_UNKNOWN".to_string());
    }
    let theorem = value
        .get("theorem")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    // Refusal-awareness gate (L196, checked BEFORE the axiom and divergence
    // gates — a refusal witness is evidence OF a refusal, never liftable as
    // `Replayed` success evidence, so the typed refusal must not be shadowed
    // by later gates firing first):
    // `refused: true` → `REFUSED:INSA_WITNESS_IS_REFUSAL`.
    // Adjudication: typed-refusal path, NOT a distinct metric. A refusal
    // witness records that the insa producer refused — the count is always 0
    // and carries no `insa.sync.verified` signal to chart; mapping it to a
    // metric would launder a refusal into an observability datum. The typed
    // refusal keeps it a court-visible event.
    let refused = value.get("refused").and_then(|r| r.as_bool());
    if refused == Some(true) {
        return Err("REFUSED:INSA_WITNESS_IS_REFUSAL".to_string());
    }
    let axioms: Vec<String> = value
        .get("axioms")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?
        .iter()
        .map(|a| a.as_str().unwrap_or_default().to_string())
        .collect();
    let source_sha256 = value
        .get("source_sha256")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    let sync_head = value
        .get("sync_head")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    let anchor_head = value
        .get("anchor_head")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    let verified_entries = value
        .get("verified_entries")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "REFUSED:INSA_WITNESS_INCOMPLETE".to_string())?;

    // Axiom gate: the load-bearing sorry-freedom proof, now family-aware
    // (L196). See [`axiom_family`] for the adjudication and residual risk.
    let family = axiom_family(&axioms)?;
    // Head-equality gate: a roundtrip replayed against a diverged head
    // proves nothing about the anchor — typed refusal, never evidence.
    if sync_head != anchor_head {
        return Err("REFUSED:INSA_SYNC_DIVERGED".to_string());
    }
    if theorem.is_empty() || source_sha256.is_empty() {
        return Err("REFUSED:INSA_WITNESS_INCOMPLETE".to_string());
    }
    for head in [&sync_head, &source_sha256] {
        if head.len() != 64 || !head.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("REFUSED:INSA_WITNESS_INCOMPLETE".to_string());
        }
    }

    Ok(InsaWitnessEvidence {
        input: EvidenceInput {
            metric: "insa.sync.verified".to_string(),
            value: MetricValue::Number(verified_entries as f64),
            subject: format!("insa@{}:{}#{}", source_sha256, theorem, family.tag()),
            observed_at: observed_at.to_string(),
            epistemic_class: EvidenceEpistemicClass::Replayed,
        },
    })
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const OBSERVED_AT: &str = "2026-10-13T00:00:00Z";

    const SHA1: &str = "a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8";
    const SHA2: &str = "b1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8";

    fn valid_line() -> String {
        format!(
            r#"{{"schema":"insa-witness/v1","theorem":"insa.sync.roundtrip","axioms":["propext","Classical.choice","Quot.sound"],"source_sha256":"{SHA1}","sync_head":"{SHA2}","verified_entries":42,"anchor_head":"{SHA2}"}}"#
        )
    }

    #[test]
    fn valid_witness_admits_as_replayed() {
        let evidence =
            evidence_input_from_insa_witness(&valid_line(), OBSERVED_AT).expect("valid witness lifts");
        let input = evidence.input;
        assert_eq!(input.metric, "insa.sync.verified");
        assert_eq!(input.value, MetricValue::Number(42.0));
        assert_eq!(input.epistemic_class, EvidenceEpistemicClass::Replayed);
        assert_eq!(input.subject, format!("insa@{SHA1}:insa.sync.roundtrip#lean"));
        assert_eq!(input.observed_at, OBSERVED_AT);
    }

    #[test]
    fn rust_marker_family_admits_as_replayed() {
        // L182's real finding: the HONEST Rust-compiled witness must admit.
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"rust-compiled:no-lean-kernel\"]",
        );
        let evidence =
            evidence_input_from_insa_witness(&line, OBSERVED_AT).expect("marker family lifts");
        assert_eq!(evidence.input.epistemic_class, EvidenceEpistemicClass::Replayed);
        assert_eq!(evidence.input.subject, format!("insa@{SHA1}:insa.sync.roundtrip#rust"));
        assert_eq!(evidence.input.metric, "insa.sync.verified");
    }

    #[test]
    fn mixed_marker_and_lean_axioms_refuse() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"rust-compiled:no-lean-kernel\",\"propext\"]",
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn empty_axiom_set_refuses() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[]",
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn fabricated_lean_names_on_rust_producer_refuse_only_outside_the_trio() {
        // Fabrication residual (documented in axiom_family): the adapter
        // cannot distinguish a real Lean producer from a Rust producer lying
        // with trio names — the trio path admits on producer trust. The gate
        // still refuses every name OUTSIDE the trio, so the lie must be a
        // perfect fabrication to pass. This test pins the refusal boundary
        // that DOES hold mechanically.
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\",\"sorryAx\"]",
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn refused_witness_is_typed_refusal_never_evidence() {
        // L182's tamper-refusal witness: `refused: true` must never lift to
        // Replayed evidence with verified_entries: 0. Note this line ALSO
        // diverges (sync_head all-zeros) — the refusal gate fires first.
        let line = valid_line()
            .replace(
                "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
                "\"axioms\":[\"rust-compiled:no-lean-kernel\"]",
            )
            .replace(
                &format!("\"sync_head\":\"{SHA2}\""),
                "\"sync_head\":\"0000000000000000000000000000000000000000000000000000000000000000\"",
            )
            .replace("\"verified_entries\":42", "\"verified_entries\":0")
            .replace(
                &format!("\"anchor_head\":\"{SHA2}\""),
                &format!("\"anchor_head\":\"{SHA2}\",\"refused\":true"),
            );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_WITNESS_IS_REFUSAL".to_string())
        );
    }

    #[test]
    fn refused_flag_false_or_absent_does_not_block() {
        let line = valid_line().replace(
            &format!("\"anchor_head\":\"{SHA2}\""),
            &format!("\"anchor_head\":\"{SHA2}\",\"refused\":false"),
        );
        assert!(evidence_input_from_insa_witness(&line, OBSERVED_AT).is_ok());
    }

    #[test]
    fn head_divergence_is_refused() {
        let line = valid_line().replace(
            &format!("\"anchor_head\":\"{SHA2}\""),
            &format!("\"anchor_head\":\"{SHA1}\""),
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_SYNC_DIVERGED".to_string())
        );
    }

    #[test]
    fn sorry_ax_is_refused_never_admitted() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\",\"sorryAx\"]",
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn nonstandard_axiom_is_refused() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"propext\",\"myCustomAxiom\"]",
        );
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn unknown_schema_is_refused() {
        let line = valid_line().replace("insa-witness/v1", "insa-witness/v2");
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_SCHEMA_UNKNOWN".to_string())
        );
    }

    #[test]
    fn missing_schema_is_refused() {
        let line = valid_line().replace("\"schema\":\"insa-witness/v1\",", "");
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_SCHEMA_UNKNOWN".to_string())
        );
    }

    #[test]
    fn malformed_json_is_refused() {
        assert_eq!(
            evidence_input_from_insa_witness("{not json", OBSERVED_AT),
            Err("REFUSED:INSA_WITNESS_MALFORMED_JSON".to_string())
        );
    }

    #[test]
    fn missing_verified_entries_is_refused() {
        let line = valid_line().replace("\"verified_entries\":42,", "");
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_WITNESS_INCOMPLETE".to_string())
        );
    }

    #[test]
    fn short_head_is_refused() {
        let line = valid_line().replace(SHA2, "deadbeef");
        // Corrupts both heads identically — still equal, so the refusal must
        // come from the hex/length gate, not divergence.
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_WITNESS_INCOMPLETE".to_string())
        );
    }

    #[test]
    fn empty_theorem_is_refused() {
        let line = valid_line().replace("insa.sync.roundtrip", "");
        assert_eq!(
            evidence_input_from_insa_witness(&line, OBSERVED_AT),
            Err("REFUSED:INSA_WITNESS_INCOMPLETE".to_string())
        );
    }

    #[test]
    fn verified_entries_value_rides_the_metric() {
        for n in [0u64, 1, 7, 4096] {
            let line = valid_line().replace("\"verified_entries\":42", &format!("\"verified_entries\":{n}"));
            let evidence = evidence_input_from_insa_witness(&line, OBSERVED_AT).expect("lifts");
            assert_eq!(evidence.input.value, MetricValue::Number(n as f64));
        }
    }
}
