use std::collections::BTreeSet;

use castle::v26_9_28::*;

fn digest(ch: char) -> String {
    std::iter::repeat(ch).take(64).collect()
}

#[test]
fn manifest_binds_exact_review_subjects() {
    let manifest = ecosystem_manifest().unwrap();
    assert_eq!(manifest.release_epoch, "v26.9.28");
    assert_eq!(manifest.castle_base_sha, "5059c45a8ef007cd1e095213c729c8bfe6db9e79");
    assert_eq!(manifest.marketplace_pack.r#ref, "v26.9.29");
    assert_eq!(manifest.marketplace_pack.commit_sha, "637b561cc6384fc9ac0e4282d048cc7624256258");
    assert!(qualify_v26_9_28_upgrade(&manifest).standing.is_alive());
}

#[test]
fn open_prs_are_reviewed_but_not_admitted_as_source_subjects() {
    let manifest = ecosystem_manifest().unwrap();
    let admitted_prs: BTreeSet<(&str, u64)> = manifest
        .subjects
        .iter()
        .filter_map(|s| s.pr.map(|pr| (s.repo.as_str(), pr)))
        .collect();
    for excluded in &manifest.excluded_open_prs {
        assert!(!admitted_prs.contains(&(excluded.repo.as_str(), excluded.pr)));
    }
}

#[test]
fn external_witness_requires_exact_sha_bounds_and_no_do_authority() {
    let manifest = ecosystem_manifest().unwrap();
    let source = manifest.subjects.iter().find(|s| s.id == "graphlaw-v26.9.28").unwrap();
    let mut witness = ExternalWitness {
        source_id: source.id.clone(),
        source_sha: source.sha.clone(),
        subject: "subject:payments".to_string(),
        kind: WitnessKind::Semantic,
        input_digest: digest('a'),
        output_digest: digest('b'),
        direct_do_authority: false,
        limits: WitnessLimits { max_steps: 10_000, max_bytes: 1_048_576, deadline_ms: 30_000 },
    };
    assert!(admit_external_witness(&manifest, &witness).is_alive());

    witness.source_sha = "0".repeat(40);
    assert_eq!(
        admit_external_witness(&manifest, &witness),
        EvidenceStanding::Refused("REFUSED:SOURCE_SUBJECT_DRIFT".to_string())
    );
    witness.source_sha = source.sha.clone();
    witness.direct_do_authority = true;
    assert_eq!(
        admit_external_witness(&manifest, &witness),
        EvidenceStanding::Refused("REFUSED:EXTERNAL_DO_AUTHORITY".to_string())
    );
}

#[test]
fn unbounded_external_compute_is_refused() {
    let manifest = ecosystem_manifest().unwrap();
    let source = manifest.subjects.iter().find(|s| s.id == "dspy-wasm-pr8").unwrap();
    let witness = ExternalWitness {
        source_id: source.id.clone(),
        source_sha: source.sha.clone(),
        subject: "subject:model-advice".to_string(),
        kind: WitnessKind::ModelCompute,
        input_digest: digest('a'),
        output_digest: digest('b'),
        direct_do_authority: false,
        limits: WitnessLimits {
            max_steps: MAX_EXTERNAL_WITNESS_STEPS + 1,
            max_bytes: 1024,
            deadline_ms: 1000,
        },
    };
    assert_eq!(
        admit_external_witness(&manifest, &witness),
        EvidenceStanding::Refused("REFUSED:UNBOUNDED_EXTERNAL_WITNESS".to_string())
    );
}

#[test]
fn portability_requires_same_result_on_distinct_runtime_families() {
    let witnesses = vec![
        PortableRuntimeWitness {
            engine_family: "cranelift".to_string(),
            module_digest: digest('a'),
            input_digest: digest('b'),
            output_digest: digest('c'),
        },
        PortableRuntimeWitness {
            engine_family: "v8".to_string(),
            module_digest: digest('a'),
            input_digest: digest('b'),
            output_digest: digest('c'),
        },
    ];
    assert!(qualify_portable_runtime(&witnesses).is_alive());

    let mut divergent = witnesses;
    divergent[1].output_digest = digest('d');
    assert_eq!(
        qualify_portable_runtime(&divergent),
        EvidenceStanding::Refused("REFUSED:RUNTIME_WITNESS_DIVERGENCE".to_string())
    );
}

#[test]
fn plan_cannot_grade_itself() {
    let good = IndependentPlanCheck {
        plan_digest: digest('a'),
        valid_under_independent_dynamics: true,
        claimed_cost: 18,
        independently_verified_cost: 18,
    };
    assert!(admit_independent_plan(&good).is_alive());

    let bad = IndependentPlanCheck {
        independently_verified_cost: 19,
        ..good
    };
    assert_eq!(
        admit_independent_plan(&bad),
        EvidenceStanding::Refused("REFUSED:PLAN_COST_MISMATCH".to_string())
    );
}

#[test]
fn fond_policy_requires_differential_agreement() {
    let good = FondDifferentialCheck {
        policy_digest: digest('a'),
        model_digest: digest('b'),
        agreed: true,
        counterexample: None,
    };
    assert!(admit_fond_differential(&good).is_alive());

    let bad = FondDifferentialCheck {
        agreed: false,
        counterexample: Some("state:s3/action:retry".to_string()),
        ..good
    };
    assert_eq!(
        admit_fond_differential(&bad),
        EvidenceStanding::Refused("REFUSED:FOND_TLA_DIFFERENTIAL_MISMATCH".to_string())
    );
}

#[test]
fn recovery_excludes_failed_edge_without_stopping_graph() {
    let providers = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
    let failed = BTreeSet::from(["p1".to_string(), "p2".to_string()]);
    let routed = route_edge_local_recovery("subject:exact", &providers, &failed).unwrap();
    assert_eq!(routed.subject, "subject:exact");
    assert_eq!(routed.selected_provider, "p3");
    assert_eq!(routed.excluded_failed_providers, vec!["p1", "p2"]);

    let exhausted = BTreeSet::from(["p1".to_string(), "p2".to_string(), "p3".to_string()]);
    assert_eq!(
        route_edge_local_recovery("subject:exact", &providers, &exhausted),
        Err("BLOCKED:NO_LAWFUL_RECOVERY_EDGE".to_string())
    );
}
