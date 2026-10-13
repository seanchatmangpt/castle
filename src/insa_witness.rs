//! insa sync witness → Replayed evidence adapter (L32 rank-1, L179).
//!
//! Ingests an insa-witness JSON line (schema `insa-witness/v1`, the L32
//! rank-1 format emitted by insa `cargo test` sync-roundtrip runs) and lifts
//! it into a castle `Replayed` evidence input.
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
/// 2. axiom gate — `REFUSED:INSA_AXIOMS_NONSTANDARD` unless the axiom set
///    is a subset of [`ADMITTED_AXIOMS`] (`sorryAx` is the prime refused
///    case — this is the sorry-freedom proof);
/// 3. head-equality gate — `REFUSED:INSA_SYNC_DIVERGED` unless
///    `sync_head == anchor_head` (a roundtrip replayed against a diverged
///    head proves nothing about the anchor);
/// 4. required-field gate — `REFUSED:INSA_WITNESS_INCOMPLETE` on empty
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

    // Axiom gate: the load-bearing sorry-freedom proof. Subset check —
    // witnesses may omit core axioms Lean did not load, but anything OUTSIDE
    // the trio (sorryAx above all) refuses.
    for axiom in &axioms {
        if !ADMITTED_AXIOMS.contains(&axiom.as_str()) {
            return Err("REFUSED:INSA_AXIOMS_NONSTANDARD".to_string());
        }
    }
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
            subject: format!("insa@{}:{}", source_sha256, theorem),
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
        assert_eq!(input.subject, format!("insa@{SHA1}:insa.sync.roundtrip"));
        assert_eq!(input.observed_at, OBSERVED_AT);
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
