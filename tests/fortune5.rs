//! Port of `test/fortune5.test.ts`.

use castle::fortune5::*;
use castle::fortune5_generated::FORTUNE5_REQUIREMENTS;

const SUBJECT: &str = "castle:enterprise:test";
const NOW: i64 = 1_786_824_000_000; // 2026-08-15T20:00:00.000Z

fn target_value(target: &str) -> MetricValue {
    if target == "true" {
        return MetricValue::Bool(true);
    }
    if target == "false" {
        return MetricValue::Bool(false);
    }
    if let Ok(n) = target.parse::<f64>() {
        return MetricValue::Number(n);
    }
    MetricValue::Str(target.to_string())
}

fn perfect_evidence() -> Vec<MetricObservation> {
    FORTUNE5_REQUIREMENTS
        .iter()
        .enumerate()
        .map(|(index, requirement)| MetricObservation {
            metric: requirement.metric.to_string(),
            value: target_value(requirement.target),
            receipt_digest: format!("{:x}", index % 16).repeat(64),
            subject: SUBJECT.to_string(),
            observed_at: "2026-08-15T19:59:00.000Z".to_string(),
            epistemic_class: EvidenceEpistemicClass::Observed,
        })
        .collect()
}

#[test]
fn marketplace_projection_exposes_the_complete_fortune5_readiness_profile() {
    assert_eq!(FORTUNE5_REQUIREMENTS.len(), 40);
    let control_ids: std::collections::HashSet<_> = FORTUNE5_REQUIREMENTS.iter().map(|r| r.control_id).collect();
    assert_eq!(control_ids.len(), 40);
    let metrics: std::collections::HashSet<_> = FORTUNE5_REQUIREMENTS.iter().map(|r| r.metric).collect();
    assert_eq!(metrics.len(), 40);
    for category in [
        "authority",
        "isolation",
        "supply-chain",
        "replay",
        "observability",
        "resilience",
        "availability",
        "slo",
        "data",
        "policy",
        "scale",
        "determinism",
        "evidence",
        "refusal",
        "change",
        "air-gap",
        "adversarial-coverage",
    ] {
        assert!(FORTUNE5_REQUIREMENTS.iter().any(|r| r.category == category), "missing {category}");
    }
}

#[test]
fn missing_evidence_is_unknown_rather_than_vacuously_passing() {
    let result = qualify_fortune5(&[], &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: Some(NOW), max_evidence_age_ms: None }, FORTUNE5_REQUIREMENTS);
    assert_eq!(result.standing, Standing::Unknown);
    assert_eq!(result.alive, 0);
    assert_eq!(result.refused, 0);
    assert_eq!(result.unknown, 40);
    assert!(result.controls.iter().all(|c| c.reason == "UNKNOWN:MISSING_RECEIPTED_EVIDENCE"));
}

#[test]
fn complete_receipted_boundary_evidence_qualifies_alive() {
    let result = qualify_fortune5(
        &perfect_evidence(),
        &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: Some(NOW), max_evidence_age_ms: Some(5 * 60 * 1000) },
        FORTUNE5_REQUIREMENTS,
    );
    assert_eq!(result.standing, Standing::Alive);
    assert_eq!(result.alive, 40);
    assert_eq!(result.refused, 0);
    assert_eq!(result.unknown, 0);
    assert!(result.categories.values().all(|s| *s == Standing::Alive));
}

#[test]
fn a_failed_hard_control_refuses_the_aggregate_profile() {
    let mut evidence = perfect_evidence();
    let index = evidence.iter().position(|o| o.metric == "zero_unreceipted_actuations").unwrap();
    evidence[index].value = MetricValue::Number(1.0);
    let result = qualify_fortune5(&evidence, &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: Some(NOW), max_evidence_age_ms: None }, FORTUNE5_REQUIREMENTS);
    assert_eq!(result.standing, Standing::Refused);
    let control = result.controls.iter().find(|c| c.control_id == "F5-AUTH-001").unwrap();
    assert_eq!(control.standing, Standing::Refused);
    assert_eq!(control.reason, "REFUSED:CONTROL_THRESHOLD_NOT_SATISFIED");
}

#[test]
fn unreceipted_stale_cross_subject_and_ambiguous_evidence_is_refused() {
    let base = perfect_evidence();
    let metric = "deny_by_default";
    let index = base.iter().position(|o| o.metric == metric).unwrap();

    let mut bad_digest = base.clone();
    bad_digest[index].receipt_digest = "not-a-digest".to_string();
    assert_eq!(qualify_fortune5(&bad_digest, &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: None, max_evidence_age_ms: None }, FORTUNE5_REQUIREMENTS).standing, Standing::Refused);

    let mut stale = base.clone();
    stale[index].observed_at = "2026-08-15T18:00:00.000Z".to_string();
    assert_eq!(
        qualify_fortune5(&stale, &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: Some(NOW), max_evidence_age_ms: Some(60_000) }, FORTUNE5_REQUIREMENTS).standing,
        Standing::Refused
    );

    let mut wrong_subject = base.clone();
    wrong_subject[index].subject = "castle:other".to_string();
    assert_eq!(qualify_fortune5(&wrong_subject, &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: None, max_evidence_age_ms: None }, FORTUNE5_REQUIREMENTS).standing, Standing::Refused);

    let mut ambiguous = base.clone();
    ambiguous.push(base[index].clone());
    let result = qualify_fortune5(&ambiguous, &QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: None, max_evidence_age_ms: None }, FORTUNE5_REQUIREMENTS);
    assert_eq!(result.standing, Standing::Refused);
    assert_eq!(result.controls.iter().find(|c| c.metric == metric).unwrap().reason, "REFUSED:AMBIGUOUS_EVIDENCE");
}

#[test]
fn replay_standing_requires_structural_ontology_provider_and_invariant_compatibility() {
    let manifest = ReplayManifest {
        replay_class_id: "class:authority-loss:v1".to_string(),
        structural_signature: "1".repeat(64),
        ontology_version: "castle-ontology:26.8.15+f5.1".to_string(),
        provider_semantics_version: "aws-control-plane:2026-08-15".to_string(),
        invariant_set_digest: "2".repeat(64),
        process_digest: "3".repeat(64),
    };
    let subject = ReplaySubject {
        structural_signature: manifest.structural_signature.clone(),
        ontology_version: manifest.ontology_version.clone(),
        provider_semantics_version: manifest.provider_semantics_version.clone(),
        invariant_set_digest: manifest.invariant_set_digest.clone(),
        invariants_hold: true,
    };
    assert_eq!(admit_replay(&manifest, &subject).standing, Standing::Alive);

    let provider_drift = admit_replay(&manifest, &ReplaySubject { provider_semantics_version: "aws-control-plane:2026-08-16".to_string(), ..subject.clone() });
    assert_eq!(provider_drift.standing, Standing::Refused);
    assert!(provider_drift.reasons.contains(&"REFUSED:PROVIDER_SEMANTICS_VERSION_MISMATCH".to_string()));

    let invariant_failure = admit_replay(&manifest, &ReplaySubject { invariants_hold: false, ..subject.clone() });
    assert_eq!(invariant_failure.standing, Standing::Refused);
    assert!(invariant_failure.reasons.contains(&"REFUSED:INVARIANTS_NOT_SATISFIED".to_string()));
}

#[test]
fn pareto_coverage_is_measured_from_consequence_impact_rather_than_assumed() {
    let result = minimum_impact_coverage(
        &[
            AdversarialImpactClass { key: "authority".to_string(), impact: 60.0 },
            AdversarialImpactClass { key: "confidentiality".to_string(), impact: 20.0 },
            AdversarialImpactClass { key: "integrity".to_string(), impact: 10.0 },
            AdversarialImpactClass { key: "availability".to_string(), impact: 10.0 },
        ],
        8000,
    )
    .unwrap();
    assert_eq!(result.selected.iter().map(|c| c.key.clone()).collect::<Vec<_>>(), vec!["authority".to_string(), "confidentiality".to_string()]);
    assert_eq!(result.coverage_bps, 8000);
    assert_eq!(result.selected_impact, 80.0);
    assert_eq!(result.total_impact, 100.0);
}

#[test]
fn invalid_impact_evidence_and_impossible_coverage_targets_are_typed_refusals() {
    assert_eq!(minimum_impact_coverage(&[AdversarialImpactClass { key: "x".to_string(), impact: -1.0 }], 8000).unwrap_err(), "REFUSED:INVALID_IMPACT");
    assert_eq!(minimum_impact_coverage(&[AdversarialImpactClass { key: "x".to_string(), impact: 1.0 }], 0).unwrap_err(), "REFUSED:INVALID_COVERAGE_TARGET");
    assert_eq!(minimum_impact_coverage(&[AdversarialImpactClass { key: "x".to_string(), impact: 0.0 }], 8000).unwrap_err(), "REFUSED:NO_POSITIVE_IMPACT_EVIDENCE");
}

#[test]
fn extreme_finite_impact_values_sort_deterministically_without_panicking() {
    // Regression test for the former `b.impact.partial_cmp(&a.impact).unwrap()` sort
    // comparator in `minimum_impact_coverage`. That `unwrap()` only avoided a panic
    // because of the `is_finite` guard nine lines above it in the same function — a
    // real but decoupled precondition that a future refactor could split from the sort.
    // `f64::total_cmp` removes the dependency on that guard entirely: it is a total
    // order over every f64 bit pattern (including NaN and signed zero), so the
    // comparator itself can never return "unorderable" regardless of what reaches it.
    // This exercises adjacent/extreme finite values (signed zero, f64::MAX, subnormals)
    // that are the sharpest edge partial_cmp and total_cmp can disagree on, and confirms
    // the call completes with an ordered, deterministic result instead of panicking.
    let classes = vec![
        AdversarialImpactClass { key: "max".to_string(), impact: f64::MAX },
        AdversarialImpactClass { key: "neg_zero".to_string(), impact: -0.0 },
        AdversarialImpactClass { key: "pos_zero".to_string(), impact: 0.0 },
        AdversarialImpactClass { key: "min_positive".to_string(), impact: f64::MIN_POSITIVE },
    ];

    let selection = minimum_impact_coverage(&classes, 10000).expect("all-finite, non-negative impact values are accepted");
    assert_eq!(selection.selected.first().unwrap().key, "max");
}
