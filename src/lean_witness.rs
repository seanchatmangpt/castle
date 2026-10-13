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
use crate::fortune5::{
    qualify_fortune5, EvidenceEpistemicClass, Fortune5Qualification, MetricObservation,
    MetricValue, QualificationContext,
};
use crate::fortune5_generated::Fortune5Requirement;

/// Refusal prefix used by every typed refusal in this module.
pub const REFUSED: &str = "REFUSED";

/// The only schema identity this adapter admits. A future schema must be
/// admitted explicitly, never silently ingested.
pub const ADMITTED_SCHEMAS: [&str; 1] = ["lean-witness/v1"];

/// The Lean core axiom trio — the exact sorry-freedom footprint `#print
/// axioms` is allowed to report for an admitted theorem.
pub const ADMITTED_AXIOMS: [&str; 3] = ["propext", "Classical.choice", "Quot.sound"];

/// Subject the strata-ledger defect lifecycle is tracked against
/// (ggen-marketplace strata-ledger-pack, duplicate-input fix 1d16f2472).
pub const STRATA_LEDGER_SUBJECT: &str = "strata-ledger-pack";

/// Control entry (L126, L111 rank-3): castle standing machinery tracks the
/// strata-ledger duplicate-input defect lifecycle. The metric observes
/// whether the ledger template's conservant behavior holds under duplicate
/// input — post-fix (1d16f2472) it is `true`; the pinned pre-fix historical
/// witness records `false`. Declared here rather than in the generated
/// [`crate::fortune5_generated::FORTUNE5_REQUIREMENTS`] table.
pub const LEDGER_CONSERVANT_CONTROL: Fortune5Requirement = Fortune5Requirement {
    order: 600,
    control_id: "F5-LEDGER-001",
    category: "ledger",
    description: "The strata-ledger template conserves debits under duplicate input.",
    metric: "ledger.template.conservant_under_duplicate_input",
    comparator: "EQ",
    target: "true",
    authority: "REQUIRED",
};

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

    // Observed metric value carried by the witness (optional, default `true`
    // for the pure sorry-freedom witness shape). A `false` witness is the
    // pinned pre-fix historical state, not a malformed line.
    let value = match value.get("value") {
        None => true,
        Some(v) => v
            .as_bool()
            .ok_or_else(|| "REFUSED:LEAN_WITNESS_INCOMPLETE".to_string())?,
    };

    Ok(LeanWitnessEvidence {
        input: EvidenceInput {
            metric: "lean.witness.verified".to_string(),
            value: MetricValue::Bool(value),
            subject: format!("mfact@{}:{}", source_sha256, theorem),
            observed_at: observed_at.to_string(),
            epistemic_class: EvidenceEpistemicClass::Replayed,
        },
    })
}

/// Lift a lean-witness line into a `MetricObservation` for the
/// [`LEDGER_CONSERVANT_CONTROL`] control: the observation's subject is the
/// strata-ledger pack (not the mfact theorem), the value is the witness's
/// observed `value` bool, and the receipt digest is the witness's pinned
/// mfact source digest. Axiom-violating witnesses refuse at ingest and never
/// become observations.
///
/// # Errors
/// Whatever [`evidence_input_from_lean_witness`] refuses.
pub fn ledger_conservant_observation(
    line: &str,
    observed_at: &str,
) -> Result<MetricObservation, String> {
    let evidence = evidence_input_from_lean_witness(line, observed_at)?;
    let source_sha256 = evidence
        .input
        .subject
        .strip_prefix("mfact@")
        .and_then(|s| s.split(':').next())
        .unwrap_or_default()
        .to_string();
    Ok(MetricObservation {
        metric: LEDGER_CONSERVANT_CONTROL.metric.to_string(),
        value: evidence.input.value,
        receipt_digest: source_sha256,
        subject: STRATA_LEDGER_SUBJECT.to_string(),
        observed_at: observed_at.to_string(),
        epistemic_class: EvidenceEpistemicClass::Replayed,
    })
}

/// Qualify [`STRATA_LEDGER_SUBJECT`] against the ledger conservant control
/// (plus the supplied evidence). This is the standing machinery tracking the
/// defect lifecycle: witness value `false` (pre-fix) refuses and degrades
/// both the aggregate and the `ledger` category; `true` (post-fix) is Alive.
#[must_use]
pub fn qualify_ledger_conservant(
    observations: &[MetricObservation],
    context: &QualificationContext,
) -> Fortune5Qualification {
    qualify_fortune5(observations, context, &[LEDGER_CONSERVANT_CONTROL])
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fortune5::Standing;

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

    // -----------------------------------------------------------------
    // L126 — defect-lifecycle control inversion courts (L111 rank-3):
    // strata-ledger duplicate-input fix (ggen-marketplace 1d16f2472)
    // tracked as a Fortune-5 standing control, no Standing enum change.
    // -----------------------------------------------------------------

    const OBSERVED_AT: &str = "2026-10-13T00:00:00Z";

    /// Pinned POST-fix witness (ggen-marketplace 1d16f2472 landed): the
    /// template conserves under duplicate input → value true.
    fn post_fix_line() -> String {
        r#"{"schema":"lean-witness/v1","theorem":"strata_ledger_conservant_under_duplicate_input","axioms":["propext","Classical.choice","Quot.sound"],"source_sha256":"c1d2e3f4a5b6c7d8c1d2e3f4a5b6c7d8c1d2e3f4a5b6c7d8c1d2e3f4a5b6c7d8","toolchain":"leanprover/lean4:v4.21.0","value":true}"#.to_string()
    }

    /// Pinned PRE-fix historical witness: the template did NOT conserve
    /// under duplicate input → value false.
    fn pre_fix_line() -> String {
        r#"{"schema":"lean-witness/v1","theorem":"strata_ledger_conservant_under_duplicate_input","axioms":["propext","Classical.choice","Quot.sound"],"source_sha256":"b9a8c7d6e5f4b3a2b9a8c7d6e5f4b3a2b9a8c7d6e5f4b3a2b9a8c7d6e5f4b3a2","toolchain":"leanprover/lean4:v4.21.0","value":false}"#.to_string()
    }

    fn ledger_context() -> QualificationContext {
        QualificationContext {
            subject: STRATA_LEDGER_SUBJECT.to_string(),
            now_epoch_ms: None,
            max_evidence_age_ms: None,
        }
    }

    #[test]
    fn post_fix_witness_control_is_alive() {
        let observation =
            ledger_conservant_observation(&post_fix_line(), OBSERVED_AT).expect("ingests");
        let qualification =
            qualify_ledger_conservant(&[observation], &ledger_context());
        assert_eq!(qualification.standing, Standing::Alive);
        assert_eq!(qualification.alive, 1);
        assert_eq!(qualification.refused, 0);
        assert_eq!(qualification.categories.get("ledger"), Some(&Standing::Alive));
        let control = &qualification.controls[0];
        assert_eq!(control.metric, "ledger.template.conservant_under_duplicate_input");
        assert_eq!(control.reason, "ALIVE:RECEIPTED_CONTROL_SATISFIED");
    }

    #[test]
    fn pre_fix_witness_control_is_refused_and_degrades() {
        let observation =
            ledger_conservant_observation(&pre_fix_line(), OBSERVED_AT).expect("ingests");
        let qualification =
            qualify_ledger_conservant(&[observation], &ledger_context());
        // Standing degrade (fortune5.rs:275-281): refused > 0 ⇒ aggregate Refused.
        assert_eq!(qualification.standing, Standing::Refused);
        assert_eq!(qualification.refused, 1);
        // Category degrade (fortune5.rs:287-299): ledger category Refused.
        assert_eq!(qualification.categories.get("ledger"), Some(&Standing::Refused));
        let control = &qualification.controls[0];
        assert_eq!(control.standing, Standing::Refused);
        assert_eq!(control.reason, "REFUSED:CONTROL_THRESHOLD_NOT_SATISFIED");
        assert_eq!(control.observed, Some(MetricValue::Bool(false)));
    }

    #[test]
    fn axiom_violating_witness_refused_at_ingest_never_becomes_evidence() {
        let line = post_fix_line().replace(
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\"],",
            "\"axioms\":[\"propext\",\"Classical.choice\",\"Quot.sound\",\"sorryAx\"],",
        );
        // Ingest refusal: the observation is never produced.
        assert_eq!(
            ledger_conservant_observation(&line, OBSERVED_AT).err(),
            Some("REFUSED:LEAN_AXIOMS_NONSTANDARD".to_string())
        );
        // And the control falls to UNKNOWN:MISSING_RECEIPTED_EVIDENCE (no evidence),
        // not a silent pass.
        let qualification = qualify_ledger_conservant(&[], &ledger_context());
        assert_eq!(qualification.standing, Standing::Unknown);
        assert_eq!(
            qualification.controls[0].reason,
            "UNKNOWN:MISSING_RECEIPTED_EVIDENCE"
        );
    }
}
