//! Port of `fortune5.ts`: Fortune-5 readiness qualification, replay admission,
//! and Pareto impact-coverage selection.

use std::collections::{BTreeMap, HashMap};

use crate::fortune5_generated::{Fortune5Requirement, FORTUNE5_REQUIREMENTS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Standing {
    Alive,
    Refused,
    Unknown,
}

impl Standing {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Standing::Alive => "ALIVE",
            Standing::Refused => "REFUSED",
            Standing::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Number(f64),
    Bool(bool),
    Str(String),
}

fn digest_re_ok(digest: &str) -> bool {
    digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn parse_target(target: &str) -> MetricValue {
    if target == "true" {
        return MetricValue::Bool(true);
    }
    if target == "false" {
        return MetricValue::Bool(false);
    }
    if let Ok(n) = target.parse::<f64>() {
        // mirror the TS regex's integer/decimal-only acceptance (reject e.g. leading zeros)
        let looks_numeric = target.chars().next().map(|c| c == '-' || c.is_ascii_digit()).unwrap_or(false);
        if looks_numeric {
            return MetricValue::Number(n);
        }
    }
    MetricValue::Str(target.to_string())
}

fn compare_value(comparator: &str, observed: &MetricValue, target: &str) -> bool {
    let expected = parse_target(target);
    match comparator {
        "EQ" => observed == &expected,
        "GTE" => match (observed, &expected) {
            (MetricValue::Number(o), MetricValue::Number(e)) => o.is_finite() && o >= e,
            _ => false,
        },
        "LTE" => match (observed, &expected) {
            (MetricValue::Number(o), MetricValue::Number(e)) => o.is_finite() && o <= e,
            _ => false,
        },
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct MetricObservation {
    pub metric: String,
    pub value: MetricValue,
    pub receipt_digest: String,
    pub subject: String,
    pub observed_at: String,
    pub epistemic_class: EvidenceEpistemicClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceEpistemicClass {
    Observed,
    Replayed,
    Inferred,
}

#[derive(Debug, Clone)]
pub struct QualificationContext {
    pub subject: String,
    pub now_epoch_ms: Option<i64>,
    pub max_evidence_age_ms: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ControlEvaluation {
    pub control_id: String,
    pub category: String,
    pub metric: String,
    pub standing: Standing,
    pub observed: Option<MetricValue>,
    pub expected: String,
    pub receipt_digest: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct Fortune5Qualification {
    pub standing: Standing,
    pub subject: String,
    pub profile: &'static str,
    pub controls: Vec<ControlEvaluation>,
    pub alive: usize,
    pub refused: usize,
    pub unknown: usize,
    pub categories: BTreeMap<String, Standing>,
}

/// Parse an RFC 3339 / ISO-8601 timestamp into epoch milliseconds. Returns `None` on failure,
/// mirroring `Number.isFinite(Date.parse(...))` in the TS source.
fn parse_epoch_ms(iso: &str) -> Option<i64> {
    // Minimal, dependency-free RFC3339 parser sufficient for the "YYYY-MM-DDTHH:MM:SS.sssZ" shape
    // used throughout this codebase and its tests.
    let bytes = iso.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[7] != b'-' || (bytes[10] != b'T' && bytes[10] != b' ') {
        return None;
    }
    let year: i64 = iso.get(0..4)?.parse().ok()?;
    let month: i64 = iso.get(5..7)?.parse().ok()?;
    let day: i64 = iso.get(8..10)?.parse().ok()?;
    let hour: i64 = iso.get(11..13)?.parse().ok()?;
    let minute: i64 = iso.get(14..16)?.parse().ok()?;
    let second: i64 = iso.get(17..19)?.parse().ok()?;
    let mut millis: i64 = 0;
    let rest = iso.get(19..)?;
    let rest = rest.trim_end_matches('Z');
    if let Some(frac) = rest.strip_prefix('.') {
        let frac3: String = frac.chars().chain(std::iter::repeat('0')).take(3).collect();
        millis = frac3.parse().ok()?;
    }
    if !(1..=9999).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Days since epoch via a proleptic Gregorian calendar calculation.
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    let epoch_jdn = 2440588; // JDN for 1970-01-01
    let days = jdn - epoch_jdn;
    let epoch_seconds = days * 86_400 + hour * 3600 + minute * 60 + second;
    Some(epoch_seconds * 1000 + millis)
}

fn evidence_problem(observation: &MetricObservation, context: &QualificationContext) -> Option<String> {
    if observation.subject != context.subject {
        return Some("REFUSED:EVIDENCE_SUBJECT_MISMATCH".to_string());
    }
    if !digest_re_ok(&observation.receipt_digest) {
        return Some("REFUSED:INVALID_RECEIPT_DIGEST".to_string());
    }
    let observed_at = parse_epoch_ms(&observation.observed_at)?;
    if let Some(max_age) = context.max_evidence_age_ms {
        let now = context.now_epoch_ms.unwrap_or(observed_at);
        if max_age < 0 {
            return Some("REFUSED:INVALID_EVIDENCE_AGE_POLICY".to_string());
        }
        if observed_at > now {
            return Some("REFUSED:EVIDENCE_FROM_FUTURE".to_string());
        }
        if now - observed_at > max_age {
            return Some("REFUSED:STALE_EVIDENCE".to_string());
        }
    }
    None
}

pub fn qualify_fortune5(
    observations: &[MetricObservation],
    context: &QualificationContext,
    requirements: &[Fortune5Requirement],
) -> Fortune5Qualification {
    let _span = tracing::info_span!(
        "qualify_fortune5",
        subject = %context.subject,
        observation_count = observations.len(),
        standing = tracing::field::Empty,
    )
    .entered();
    let mut by_metric: HashMap<&str, Vec<&MetricObservation>> = HashMap::new();
    for observation in observations {
        by_metric.entry(observation.metric.as_str()).or_default().push(observation);
    }

    let mut ordered: Vec<&Fortune5Requirement> = requirements.iter().collect();
    ordered.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.control_id.cmp(b.control_id)));

    let controls: Vec<ControlEvaluation> = ordered
        .into_iter()
        .map(|requirement| {
            let evidence = by_metric.get(requirement.metric).cloned().unwrap_or_default();
            let expected = format!("{} {}", requirement.comparator, requirement.target);
            if evidence.is_empty() {
                return ControlEvaluation {
                    control_id: requirement.control_id.to_string(),
                    category: requirement.category.to_string(),
                    metric: requirement.metric.to_string(),
                    standing: Standing::Unknown,
                    observed: None,
                    expected,
                    receipt_digest: None,
                    reason: "UNKNOWN:MISSING_RECEIPTED_EVIDENCE".to_string(),
                };
            }
            if evidence.len() != 1 {
                return ControlEvaluation {
                    control_id: requirement.control_id.to_string(),
                    category: requirement.category.to_string(),
                    metric: requirement.metric.to_string(),
                    standing: Standing::Refused,
                    observed: None,
                    expected,
                    receipt_digest: None,
                    reason: "REFUSED:AMBIGUOUS_EVIDENCE".to_string(),
                };
            }

            let observation = evidence[0];
            if let Some(problem) = evidence_problem(observation, context) {
                return ControlEvaluation {
                    control_id: requirement.control_id.to_string(),
                    category: requirement.category.to_string(),
                    metric: requirement.metric.to_string(),
                    standing: Standing::Refused,
                    observed: Some(observation.value.clone()),
                    expected,
                    receipt_digest: Some(observation.receipt_digest.clone()),
                    reason: problem,
                };
            }

            if !matches!(requirement.comparator, "EQ" | "GTE" | "LTE") {
                return ControlEvaluation {
                    control_id: requirement.control_id.to_string(),
                    category: requirement.category.to_string(),
                    metric: requirement.metric.to_string(),
                    standing: Standing::Refused,
                    observed: Some(observation.value.clone()),
                    expected,
                    receipt_digest: Some(observation.receipt_digest.clone()),
                    reason: "REFUSED:UNSUPPORTED_COMPARATOR".to_string(),
                };
            }

            let passes = compare_value(requirement.comparator, &observation.value, requirement.target);
            ControlEvaluation {
                control_id: requirement.control_id.to_string(),
                category: requirement.category.to_string(),
                metric: requirement.metric.to_string(),
                standing: if passes { Standing::Alive } else { Standing::Refused },
                observed: Some(observation.value.clone()),
                expected,
                receipt_digest: Some(observation.receipt_digest.clone()),
                reason: if passes {
                    "ALIVE:RECEIPTED_CONTROL_SATISFIED".to_string()
                } else {
                    "REFUSED:CONTROL_THRESHOLD_NOT_SATISFIED".to_string()
                },
            }
        })
        .collect();

    let alive = controls.iter().filter(|c| c.standing == Standing::Alive).count();
    let refused = controls.iter().filter(|c| c.standing == Standing::Refused).count();
    let unknown = controls.iter().filter(|c| c.standing == Standing::Unknown).count();
    let standing = if refused > 0 {
        Standing::Refused
    } else if unknown > 0 {
        Standing::Unknown
    } else {
        Standing::Alive
    };

    let mut category_members: BTreeMap<String, Vec<Standing>> = BTreeMap::new();
    for control in &controls {
        category_members.entry(control.category.clone()).or_default().push(control.standing);
    }
    let categories: BTreeMap<String, Standing> = category_members
        .into_iter()
        .map(|(category, members)| {
            let s = if members.contains(&Standing::Refused) {
                Standing::Refused
            } else if members.contains(&Standing::Unknown) {
                Standing::Unknown
            } else {
                Standing::Alive
            };
            (category, s)
        })
        .collect();

    _span.record("standing", tracing::field::debug(standing));
    Fortune5Qualification {
        standing,
        subject: context.subject.clone(),
        profile: "CASTLE_FORTUNE5_READINESS_V1",
        controls,
        alive,
        refused,
        unknown,
        categories,
    }
}

#[must_use]
pub fn qualify_fortune5_default(observations: &[MetricObservation], context: &QualificationContext) -> Fortune5Qualification {
    qualify_fortune5(observations, context, FORTUNE5_REQUIREMENTS)
}

#[derive(Debug, Clone)]
pub struct ReplayManifest {
    pub replay_class_id: String,
    pub structural_signature: String,
    pub ontology_version: String,
    pub provider_semantics_version: String,
    pub invariant_set_digest: String,
    pub process_digest: String,
}

#[derive(Debug, Clone)]
pub struct ReplaySubject {
    pub structural_signature: String,
    pub ontology_version: String,
    pub provider_semantics_version: String,
    pub invariant_set_digest: String,
    pub invariants_hold: bool,
}

#[derive(Debug, Clone)]
pub struct ReplayAdmission {
    pub standing: Standing,
    pub replay_class_id: String,
    pub reasons: Vec<String>,
}

/// Replay is reuse of a proved, parameterized state transformer, not literal API-call playback.
/// A class loses standing when its structural signature, ontology, provider semantics, or invariant
/// set no longer matches the current admitted subject.
#[must_use]
pub fn admit_replay(manifest: &ReplayManifest, subject: &ReplaySubject) -> ReplayAdmission {
    let mut reasons: Vec<String> = Vec::new();
    if !digest_re_ok(&manifest.structural_signature) || !digest_re_ok(&subject.structural_signature) {
        reasons.push("REFUSED:INVALID_STRUCTURAL_SIGNATURE".to_string());
    } else if manifest.structural_signature != subject.structural_signature {
        reasons.push("REFUSED:STRUCTURAL_SIGNATURE_MISMATCH".to_string());
    }
    if manifest.ontology_version.is_empty() || manifest.ontology_version != subject.ontology_version {
        reasons.push("REFUSED:ONTOLOGY_VERSION_MISMATCH".to_string());
    }
    if manifest.provider_semantics_version.is_empty() || manifest.provider_semantics_version != subject.provider_semantics_version {
        reasons.push("REFUSED:PROVIDER_SEMANTICS_VERSION_MISMATCH".to_string());
    }
    if !digest_re_ok(&manifest.invariant_set_digest) || !digest_re_ok(&subject.invariant_set_digest) {
        reasons.push("REFUSED:INVALID_INVARIANT_SET_DIGEST".to_string());
    } else if manifest.invariant_set_digest != subject.invariant_set_digest {
        reasons.push("REFUSED:INVARIANT_SET_MISMATCH".to_string());
    }
    if !digest_re_ok(&manifest.process_digest) {
        reasons.push("REFUSED:INVALID_PROCESS_DIGEST".to_string());
    }
    if !subject.invariants_hold {
        reasons.push("REFUSED:INVARIANTS_NOT_SATISFIED".to_string());
    }

    let standing = if reasons.is_empty() { Standing::Alive } else { Standing::Refused };
    if reasons.is_empty() {
        reasons.push("ALIVE:REPLAY_CLASS_ADMITTED".to_string());
    }
    ReplayAdmission {
        standing,
        replay_class_id: manifest.replay_class_id.clone(),
        reasons,
    }
}

#[derive(Debug, Clone)]
pub struct AdversarialImpactClass {
    pub key: String,
    pub impact: f64,
}

#[derive(Debug, Clone)]
pub struct ImpactCoverageSelection {
    pub selected: Vec<AdversarialImpactClass>,
    pub coverage_bps: i64,
    pub total_impact: f64,
    pub selected_impact: f64,
}

/// Deterministically select the smallest prefix of highest-impact classes that reaches a requested
/// consequence-coverage target. The Pareto skew is measured from evidence rather than assumed.
pub fn minimum_impact_coverage(classes: &[AdversarialImpactClass], target_coverage_bps: i64) -> Result<ImpactCoverageSelection, String> {
    if !(1..=10000).contains(&target_coverage_bps) {
        return Err("REFUSED:INVALID_COVERAGE_TARGET".to_string());
    }
    if classes.iter().any(|item| !item.impact.is_finite() || item.impact < 0.0) {
        return Err("REFUSED:INVALID_IMPACT".to_string());
    }
    let total_impact: f64 = classes.iter().map(|c| c.impact).sum();
    if total_impact <= 0.0 {
        return Err("REFUSED:NO_POSITIVE_IMPACT_EVIDENCE".to_string());
    }

    let mut ordered: Vec<&AdversarialImpactClass> = classes.iter().collect();
    ordered.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap().then_with(|| a.key.cmp(&b.key)));

    let mut selected: Vec<AdversarialImpactClass> = Vec::new();
    let mut selected_impact = 0.0f64;
    for item in ordered {
        selected.push(item.clone());
        selected_impact += item.impact;
        let coverage_bps = ((selected_impact * 10000.0) / total_impact).floor() as i64;
        if coverage_bps >= target_coverage_bps {
            break;
        }
    }

    let coverage_bps = ((selected_impact * 10000.0) / total_impact).floor() as i64;
    Ok(ImpactCoverageSelection {
        selected,
        coverage_bps,
        total_impact,
        selected_impact,
    })
}
