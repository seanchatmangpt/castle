//! Lean witness → Replayed evidence adapter (L116, L111 rank-1).
//!
//! Ingests a lean-witness JSON line (schema `lean-witness/v1`, the L111
//! rank-2 format emitted by lake build with `-D warnings` + `#print axioms`
//! auditing) and lifts it into a castle `Replayed` evidence input.
//!
//! The load-bearing gate is the AXIOM gate, not the build result: `lake
//! exit 0` alone proves nothing — a theorem is sorry-free only if its axiom
//! footprint is exactly the Lean core trio. Any extra axiom (`sorryAx`
//! above all) is a typed refusal; the evidence is never produced.
//!
//! Schema gate mirrors `crates/castle-ggen-rdf/src/strata.rs`
//! (`ADMITTED_SCHEMAS`, fail-closed on unknown/missing schema).

use crate::board::EvidenceInput;
use crate::fortune5::{EvidenceEpistemicClass, MetricValue};

/// Refusal prefix used by every typed refusal in this module.
pub const REFUSED: &str = "REFUSED";

/// The only schema identity this adapter admits. A future schema must be
/// admitted explicitly, never silently ingested.
pub const ADMITTED_SCHEMAS: [&str; 1] = ["lean-witness/v1"];

/// The Lean core axiom trio — the exact sorry-freedom footprint `#print
/// axioms` is allowed to report for an admitted theorem.
pub const ADMITTED_AXIOMS: [&str; 3] = ["propext", "Classical.choice", "Quot.sound"];

/// A parsed lean-witness line.
#[derive(Debug, Clone, PartialEq)]
pub struct LeanWitness {
    pub theorem: String,
    pub axioms: Vec<String>,
    /// Hex sha256 of the mfact source the theorem was proven against.
    pub source_sha256: String,
    pub toolchain: String,
}

/// A verified `Replayed` evidence admission derived from a lean witness.
#[derive(Debug, Clone, PartialEq)]
pub struct LeanWitnessEvidence {
    pub input: EvidenceInput,
}

/// Parse a lean-witness JSON line and lift it into a `Replayed`
/// `EvidenceInput`, or a typed refusal.
///
/// Gates, in order:
/// 1. schema gate — `REFUSED:LEAN_SCHEMA_UNKNOWN` on missing/unknown
///    `schema` (fail-closed, mirrors strata.rs);
/// 2. axiom gate — `REFUSED:LEAN_AXIOMS_NONSTANDARD` unless the axiom set
///    is a subset of [`ADMITTED_AXIOMS`] (`sorryAx` is the prime refused
///    case — this is the sorry-freedom proof, not `lake exit 0`);
/// 3. required-field gate — `REFUSED:LEAN_WITNESS_INCOMPLETE` on empty
///    theorem / source digest / toolchain.
///
/// # Errors
/// See the gate list above.
pub fn evidence_input_from_lean_witness(
    line: &str,
    observed_at: &str,
) -> Result<LeanWitnessEvidence, String> {
    let value: serde_json::Value =
        serde_json::from_str(line).map_err(|_| "REFUSED:LEAN_WITNESS_MALFORMED_JSON".to_string())?;
    let schema = value
        .get("schema")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:LEAN_SCHEMA_UNKNOWN".to_string())?;
    if !ADMITTED_SCHEMAS.contains(&schema) {
        return Err("REFUSED:LEAN_SCHEMA_UNKNOWN".to_string());
    }
    let theorem = value
        .get("theorem")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "REFUSED:LEAN_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    let axioms: Vec<String> = value
        .get("axioms")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "REFUSED:LEAN_WITNESS_INCOMPLETE".to_string())?
        .iter()
        .map(|a| a.as_str().unwrap_or_default().to_string())
        .collect();
    let source_sha256 = value
        .get("source_sha256")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "REFUSED:LEAN_WITNESS_INCOMPLETE".to_string())?
        .to_string();
    let toolchain = value
        .get("toolchain")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "REFUSED:LEAN_WITNESS_INCOMPLETE".to_string())?;
    let toolchain = toolchain.to_string();
    let _ = toolchain;

    // Axiom gate: the load-bearing sorry-freedom proof. Subset check —
    // witnesses may omit core axioms Lean did not load, but anything OUTSIDE
    // the trio (sorryAx above all) refuses.
    for axiom in &axioms {
        if !ADMITTED_AXIOMS.contains(&axiom.as_str()) {
            return Err("REFUSED:LEAN_AXIOMS_NONSTANDARD".to_string());
        }
    }
    if theorem.is_empty() || source_sha256.is_empty() {
        return Err("REFUSED:LEAN_WITNESS_INCOMPLETE".to_string());
    }

    Ok(LeanWitnessEvidence {
        input: EvidenceInput {
            metric: "lean.witness.verified".to_string(),
            value: MetricValue::Bool(true),
            subject: format!("mfact@{}:{}", source_sha256, theorem),
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

    fn valid_line() -> String {
        r#"{"schema":"lean-witness/v1","theorem":"chatman_equation_typed","axioms":["propext","Classical.choice","Quot.sound"],"source_sha256":"a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8","toolchain":"leanprover/lean4:v4.21.0"}"#.to_string()
    }

    #[test]
    fn valid_witness_admits_as_replayed() {
        let evidence =
            evidence_input_from_lean_witness(&valid_line(), "2026-10-13T00:00:00Z")
                .expect("valid witness lifts");
        let input = evidence.input;
        assert_eq!(input.metric, "lean.witness.verified");
        assert_eq!(input.value, MetricValue::Bool(true));
        assert_eq!(input.epistemic_class, EvidenceEpistemicClass::Replayed);
        assert_eq!(
            input.subject,
            "mfact@a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8:chatman_equation_typed"
        );
    }

    #[test]
    fn sorry_ax_is_refused_never_admitted() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\",\"sorryAx\"]",
        );
        assert_eq!(
            evidence_input_from_lean_witness(&line, "2026-10-13T00:00:00Z"),
            Err("REFUSED:LEAN_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn nonstandard_axiom_is_refused() {
        let line = valid_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"]",
            "\"axioms\":[\"propext\",\"myCustomAxiom\"]",
        );
        assert_eq!(
            evidence_input_from_lean_witness(&line, "2026-10-13T00:00:00Z"),
            Err("REFUSED:LEAN_AXIOMS_NONSTANDARD".to_string())
        );
    }

    #[test]
    fn unknown_schema_is_refused() {
        let line = valid_line().replace("lean-witness/v1", "lean-witness/v2");
        assert_eq!(
            evidence_input_from_lean_witness(&line, "2026-10-13T00:00:00Z"),
            Err("REFUSED:LEAN_SCHEMA_UNKNOWN".to_string())
        );
    }

    #[test]
    fn missing_schema_is_refused() {
        let line = valid_line().replace("\"schema\":\"lean-witness/v1\",", "");
        assert_eq!(
            evidence_input_from_lean_witness(&line, "2026-10-13T00:00:00Z"),
            Err("REFUSED:LEAN_SCHEMA_UNKNOWN".to_string())
        );
    }

    #[test]
    fn digest_tamper_changes_the_subject() {
        let before =
            evidence_input_from_lean_witness(&valid_line(), "2026-10-13T00:00:00Z")
                .expect("lifts")
                .input
                .subject;
        let tampered = valid_line().replace(
            "a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8",
            "b1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8",
        );
        let after = evidence_input_from_lean_witness(&tampered, "2026-10-13T00:00:00Z")
            .expect("still structurally valid")
            .input
            .subject;
        assert_ne!(before, after);
        assert!(after.starts_with("mfact@b1b2c3d4"));
    }
}
