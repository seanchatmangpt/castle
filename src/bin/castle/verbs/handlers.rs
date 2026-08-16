//! Hand-written handler seam behind the generated-style `#[verb]` wrappers in
//! `routes.rs`. All business logic lives here and calls straight into the
//! `castle` library crate — the routes layer is never allowed to touch
//! `castle::*` directly.

use std::fs;

use castle::fortune5::{
    admit_replay, minimum_impact_coverage, qualify_fortune5_default, AdversarialImpactClass,
    EvidenceEpistemicClass, MetricObservation, MetricValue, QualificationContext, ReplayManifest,
    ReplaySubject,
};
use clap_noun_verb::NounVerbError;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError { message: message.into() }
}

fn read_json_file(path: &str) -> Result<Value> {
    let contents = fs::read_to_string(path).map_err(|e| exec_err(format!("failed to read {path}: {e}")))?;
    serde_json::from_str(&contents).map_err(|e| exec_err(format!("invalid JSON in {path}: {e}")))
}

fn metric_value_from_json(value: &Value) -> Result<MetricValue> {
    match value {
        Value::Bool(b) => Ok(MetricValue::Bool(*b)),
        Value::Number(n) => n.as_f64().map(MetricValue::Number).ok_or_else(|| exec_err("non-finite metric value")),
        Value::String(s) => Ok(MetricValue::Str(s.clone())),
        _ => Err(exec_err("metric value must be a bool, number, or string")),
    }
}

fn epistemic_class_from_json(value: &str) -> Result<EvidenceEpistemicClass> {
    match value {
        "OBSERVED" => Ok(EvidenceEpistemicClass::Observed),
        "REPLAYED" => Ok(EvidenceEpistemicClass::Replayed),
        "INFERRED" => Ok(EvidenceEpistemicClass::Inferred),
        other => Err(exec_err(format!("unknown epistemic_class: {other}"))),
    }
}

fn observation_from_json(row: &Value) -> Result<MetricObservation> {
    let get_str = |key: &str| -> Result<String> {
        row.get(key).and_then(Value::as_str).map(str::to_string).ok_or_else(|| exec_err(format!("evidence row missing string field '{key}'")))
    };
    Ok(MetricObservation {
        metric: get_str("metric")?,
        value: metric_value_from_json(row.get("value").ok_or_else(|| exec_err("evidence row missing 'value'"))?)?,
        receipt_digest: get_str("receipt_digest")?,
        subject: get_str("subject")?,
        observed_at: get_str("observed_at")?,
        epistemic_class: epistemic_class_from_json(&get_str("epistemic_class")?)?,
    })
}

/// `fortune5 requirements` — list the 40 generated readiness controls.
pub fn fortune5_requirements_handler() -> Result<Value> {
    let controls: Vec<Value> = castle::fortune5_generated::FORTUNE5_REQUIREMENTS
        .iter()
        .map(|r| {
            json!({
                "order": r.order,
                "controlId": r.control_id,
                "category": r.category,
                "description": r.description,
                "metric": r.metric,
                "comparator": r.comparator,
                "target": r.target,
                "authority": r.authority,
            })
        })
        .collect();
    Ok(json!({ "count": controls.len(), "requirements": controls }))
}

/// `fortune5 qualify <subject> <evidence_path>` — evaluate Fortune-5 readiness
/// from a receipted evidence JSON file: `[{metric, value, receipt_digest,
/// subject, observed_at, epistemic_class}, ...]`.
pub fn fortune5_qualify_handler(
    subject: String,
    evidence_path: String,
    now_epoch_ms: Option<i64>,
    max_evidence_age_ms: Option<i64>,
) -> Result<Value> {
    let raw = read_json_file(&evidence_path)?;
    let rows = raw.as_array().ok_or_else(|| exec_err("evidence file must contain a JSON array"))?;
    let observations: Vec<MetricObservation> = rows.iter().map(observation_from_json).collect::<Result<_>>()?;

    let context = QualificationContext { subject, now_epoch_ms, max_evidence_age_ms };
    let qualification = qualify_fortune5_default(&observations, &context);

    let controls: Vec<Value> = qualification
        .controls
        .iter()
        .map(|c| {
            json!({
                "controlId": c.control_id,
                "category": c.category,
                "metric": c.metric,
                "standing": c.standing.as_str(),
                "expected": c.expected,
                "reason": c.reason,
            })
        })
        .collect();

    Ok(json!({
        "standing": qualification.standing.as_str(),
        "subject": qualification.subject,
        "alive": qualification.alive,
        "refused": qualification.refused,
        "unknown": qualification.unknown,
        "controls": controls,
    }))
}

/// `replay admit ...` — check whether a replay class is admitted against the
/// current subject's structural signature, ontology, provider semantics, and
/// invariant set.
#[allow(clippy::too_many_arguments)]
pub fn replay_admit_handler(
    replay_class_id: String,
    structural_signature: String,
    ontology_version: String,
    provider_semantics_version: String,
    invariant_set_digest: String,
    process_digest: String,
    invariants_hold: bool,
) -> Result<Value> {
    let manifest = ReplayManifest {
        replay_class_id,
        structural_signature: structural_signature.clone(),
        ontology_version: ontology_version.clone(),
        provider_semantics_version: provider_semantics_version.clone(),
        invariant_set_digest: invariant_set_digest.clone(),
        process_digest,
    };
    let subject = ReplaySubject { structural_signature, ontology_version, provider_semantics_version, invariant_set_digest, invariants_hold };
    let admission = admit_replay(&manifest, &subject);
    Ok(json!({
        "standing": admission.standing.as_str(),
        "replayClassId": admission.replay_class_id,
        "reasons": admission.reasons,
    }))
}

/// `impact coverage <classes_path> [--target-coverage-bps N]` — select the
/// smallest deterministic prefix of highest-impact classes reaching the
/// requested Pareto coverage target from a JSON file `[{key, impact}, ...]`.
pub fn impact_coverage_handler(classes_path: String, target_coverage_bps: Option<i64>) -> Result<Value> {
    let raw = read_json_file(&classes_path)?;
    let rows = raw.as_array().ok_or_else(|| exec_err("classes file must contain a JSON array"))?;
    let classes: Vec<AdversarialImpactClass> = rows
        .iter()
        .map(|row| {
            let key = row.get("key").and_then(Value::as_str).ok_or_else(|| exec_err("class row missing string field 'key'"))?.to_string();
            let impact = row.get("impact").and_then(Value::as_f64).ok_or_else(|| exec_err("class row missing numeric field 'impact'"))?;
            Ok(AdversarialImpactClass { key, impact })
        })
        .collect::<Result<_>>()?;

    let selection = minimum_impact_coverage(&classes, target_coverage_bps.unwrap_or(8000)).map_err(exec_err)?;
    Ok(json!({
        "selected": selection.selected.iter().map(|c| json!({"key": c.key, "impact": c.impact})).collect::<Vec<_>>(),
        "coverageBps": selection.coverage_bps,
        "totalImpact": selection.total_impact,
        "selectedImpact": selection.selected_impact,
    }))
}

/// `inventory components` — list the marketplace-generated architecture
/// component inventory.
pub fn inventory_components_handler() -> Result<Value> {
    let components: Vec<Value> = castle::generated_components()
        .map(|c| json!({ "order": c.order, "identifier": c.identifier, "slug": c.slug, "role": c.role, "authority": c.authority }))
        .collect();
    Ok(json!({ "count": components.len(), "components": components }))
}

/// `inventory goals` — list the marketplace-generated default prohibited
/// adversarial goals.
pub fn inventory_goals_handler() -> Result<Value> {
    let goals: Vec<Value> = castle::default_adversarial_goals()
        .iter()
        .map(|g| json!({ "id": g.id, "predicate": g.predicate, "consequence": g.consequence }))
        .collect();
    Ok(json!({ "count": goals.len(), "goals": goals }))
}
