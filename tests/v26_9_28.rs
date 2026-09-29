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
    assert_eq!(manifest.review_window.pr_count, 66);
    assert_eq!(manifest.review_window.merged_pr_count, 57);
    assert_eq!(manifest.review_window.open_pr_count, 8);
    assert_eq!(manifest.review_window.closed_unmerged_pr_count, 1);
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
    assert_eq!(routed.excluded_failed_providers, vec!["p1".to_string(), "p2".to_string()]);

    let exhausted = BTreeSet::from(["p1".to_string(), "p2".to_string(), "p3".to_string()]);
    assert_eq!(
        route_edge_local_recovery("subject:exact", &providers, &exhausted),
        Err("BLOCKED:NO_LAWFUL_RECOVERY_EDGE".to_string())
    );
}


#[test]
fn ecosystem_evidence_is_bound_into_construct_config_identity() {
    use castle::castle::{ConstructRequest, PowlProcess, TestEnvelope};
    use serde_json::json;

    let manifest = ecosystem_manifest().unwrap();
    let source = manifest.subjects.iter().find(|s| s.id == "graphlaw-v26.9.28").unwrap();
    let request = ConstructRequest {
        subject: "subject:payments".to_string(),
        authority: "bounded-test".to_string(),
        o_star: json!({"subject": "subject:payments"}),
        config_graph: json!({"zeroUnreceiptedActuation": true}),
        ontology: json!({"version": "castle-pack-v26.9.29"}),
        process: PowlProcess {
            id: "powl:test".to_string(),
            goal_id: "goal:test".to_string(),
            activities: vec![],
        },
        envelope: TestEnvelope {
            system_id: "subject:payments".to_string(),
            allowed_transition_ids: BTreeSet::new(),
            max_steps: 0,
            expires_at_epoch_ms: 100,
        },
    };
    let witness = ExternalWitness {
        source_id: source.id.clone(),
        source_sha: source.sha.clone(),
        subject: "subject:payments".to_string(),
        kind: WitnessKind::Semantic,
        input_digest: digest('a'),
        output_digest: digest('b'),
        direct_do_authority: false,
        limits: WitnessLimits { max_steps: 1, max_bytes: 1024, deadline_ms: 1000 },
    };

    let bound = bind_v26_9_28_construct_request(request, &manifest, &[witness.clone()]).unwrap();
    let marker = bound.config_graph.get("_castle_v26_9_28").unwrap();
    assert_eq!(marker["epoch"], "v26.9.28");
    assert_eq!(marker["witness_count"], 1);
    assert_eq!(marker["sources"][0]["sha"], source.sha);

    let mut changed = witness;
    changed.output_digest = digest('c');
    let request2 = ConstructRequest {
        subject: bound.subject.clone(),
        authority: bound.authority.clone(),
        o_star: bound.o_star.clone(),
        config_graph: json!({"zeroUnreceiptedActuation": true}),
        ontology: bound.ontology.clone(),
        process: bound.process.clone(),
        envelope: bound.envelope.clone(),
    };
    let rebound = bind_v26_9_28_construct_request(request2, &manifest, &[changed]).unwrap();
    assert_ne!(
        bound.config_graph["_castle_v26_9_28"]["witness_digest"],
        rebound.config_graph["_castle_v26_9_28"]["witness_digest"]
    );
}

#[test]
fn ecosystem_witness_cannot_cross_construct_subject_boundary() {
    use castle::castle::{ConstructRequest, PowlProcess, TestEnvelope};
    use serde_json::json;

    let manifest = ecosystem_manifest().unwrap();
    let source = manifest.subjects.iter().find(|s| s.id == "affidavit-pr89").unwrap();
    let request = ConstructRequest {
        subject: "subject:a".to_string(),
        authority: "bounded-test".to_string(),
        o_star: json!({}),
        config_graph: json!({}),
        ontology: json!({}),
        process: PowlProcess { id: "p".to_string(), goal_id: "g".to_string(), activities: vec![] },
        envelope: TestEnvelope {
            system_id: "subject:a".to_string(),
            allowed_transition_ids: BTreeSet::new(),
            max_steps: 0,
            expires_at_epoch_ms: 100,
        },
    };
    let witness = ExternalWitness {
        source_id: source.id.clone(),
        source_sha: source.sha.clone(),
        subject: "subject:b".to_string(),
        kind: WitnessKind::Receipt,
        input_digest: digest('a'),
        output_digest: digest('b'),
        direct_do_authority: false,
        limits: WitnessLimits { max_steps: 1, max_bytes: 1024, deadline_ms: 1000 },
    };
    assert_eq!(
        bind_v26_9_28_construct_request(request, &manifest, &[witness]).unwrap_err(),
        "REFUSED:EXTERNAL_WITNESS_SUBJECT_MISMATCH"
    );
}


#[test]
fn sa2a_portable_envelope_is_consumed_without_rederiving_policy() {
    use serde_json::json;

    let mut envelope = PortableReplanEnvelope {
        schema: SA2A_REPLAN_SCHEMA_ID.to_string(),
        contract_digest: SA2A_REPLAN_CONTRACT_DIGEST.to_string(),
        exact_subject: json!("subject:payments"),
        receipt_id: "receipt:42".to_string(),
        consequence: "failed".to_string(),
        decision: PortableReplanDecision {
            kind: "replan".to_string(),
            reason: "recoverable_failure".to_string(),
            authority: "none".to_string(),
        },
        provider: Some("p1".to_string()),
        projection_digest: Some(digest('a')),
        source_replay_key: Some("replay:42".to_string()),
    };
    assert!(admit_sa2a_replan_envelope("subject:payments", &envelope).is_alive());

    // CASTLE validates the published envelope but does not recreate SA2A's
    // consequence->decision algebra. Terminal/unknown examples are accepted
    // when the upstream contract and no-authority fence are intact.
    envelope.consequence = "reconciled".to_string();
    envelope.decision.kind = "stop".to_string();
    envelope.decision.reason = "terminal_reconciled".to_string();
    assert!(admit_sa2a_replan_envelope("subject:payments", &envelope).is_alive());

    envelope.consequence = "unknown_outcome".to_string();
    envelope.decision.kind = "replan".to_string();
    envelope.decision.reason = "reconcile_unknown".to_string();
    assert!(admit_sa2a_replan_envelope("subject:payments", &envelope).is_alive());
}

#[test]
fn sa2a_portable_envelope_refuses_subject_or_authority_drift() {
    use serde_json::json;

    let mut envelope = PortableReplanEnvelope {
        schema: SA2A_REPLAN_SCHEMA_ID.to_string(),
        contract_digest: SA2A_REPLAN_CONTRACT_DIGEST.to_string(),
        exact_subject: json!("subject:a"),
        receipt_id: "receipt:1".to_string(),
        consequence: "refused".to_string(),
        decision: PortableReplanDecision {
            kind: "stop".to_string(),
            reason: "typed_refusal".to_string(),
            authority: "none".to_string(),
        },
        provider: None,
        projection_digest: None,
        source_replay_key: None,
    };

    assert_eq!(
        admit_sa2a_replan_envelope("subject:b", &envelope),
        EvidenceStanding::Refused("REFUSED:SA2A_EXACT_SUBJECT_MISMATCH".to_string())
    );

    envelope.exact_subject = json!("subject:a");
    envelope.decision.authority = "do".to_string();
    assert_eq!(
        admit_sa2a_replan_envelope("subject:a", &envelope),
        EvidenceStanding::Refused("REFUSED:SA2A_AUTHORITY_ESCALATION".to_string())
    );
}


#[test]
fn source_witness_kind_is_typed() {
    let manifest = ecosystem_manifest().unwrap();
    let source = manifest.subjects.iter().find(|s| s.id == "graphlaw-v26.9.28").unwrap();
    let witness = ExternalWitness {
        source_id: source.id.clone(),
        source_sha: source.sha.clone(),
        subject: "subject:payments".to_string(),
        kind: WitnessKind::ModelCompute,
        input_digest: digest('a'),
        output_digest: digest('b'),
        direct_do_authority: false,
        limits: WitnessLimits { max_steps: 1, max_bytes: 1024, deadline_ms: 1000 },
    };
    assert_eq!(
        admit_external_witness(&manifest, &witness),
        EvidenceStanding::Refused("REFUSED:SOURCE_WITNESS_KIND_MISMATCH".to_string())
    );
}
