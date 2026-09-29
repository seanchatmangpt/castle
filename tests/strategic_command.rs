use std::collections::{BTreeMap, BTreeSet};

use castle::strategic_command::{
    admit_board_selection, compile_board_constitution, compile_board_strategic_receipt,
    compile_strategy_doctrine, construct_campaign_candidate, determine_recompile_scope,
    judge_campaign_candidate, judge_counterstrategy, partition_strategy, qualify_campaign_portfolio,
    route_replan, assess_board_reentry, assess_counterstrategies, build_strategic_board_package,
    build_strategic_twin_snapshot, diff_strategic_twins, verify_strategic_board_package_offline,
    BoardAvatar, BoardReceiptInput, BoardSelectionRequest, CampaignCandidate, CampaignPortfolioPolicy,
    CampaignVerdict, ConstitutionInput, CounterstrategyScenario, DivergenceEvidence, RecompileScope,
    ReplanLevel, StrategicStanding, StrategyDoctrine, StrategyPartition, BOARD_LENSES,
    STRATEGIC_ACTUATION, STRATEGIC_AUTHORITY_CEILING, STRATEGIC_SUCCESSOR_BOUNDARY,
    STRATEGY_OPERATORS,
};

fn digest(ch: char) -> String {
    ch.to_string().repeat(64)
}

fn constitution() -> castle::strategic_command::BoardConstitution {
    compile_board_constitution(ConstitutionInput {
        subject: "enterprise:fortune5:alpha".to_string(),
        mandate_id: "mandate:board:2026-q4".to_string(),
        objectives: vec![
            "enter-market-x".to_string(),
            "preserve-strategic-optionality".to_string(),
        ],
        prohibited_outcomes: vec![
            "unauthorized-capital-commitment".to_string(),
            "unapproved-liability-class-z".to_string(),
        ],
        invariants: vec![
            "zero-unreceipted-actuation".to_string(),
            "exact-subject-authority".to_string(),
        ],
        delegated_authority: vec![
            "construct-campaigns".to_string(),
            "simulate-counterstrategy".to_string(),
            "bounded-replan".to_string(),
        ],
        nondelegable_decisions: vec![
            "select-material-campaign".to_string(),
            "expand-board-capital-bound".to_string(),
            "override-prohibited-outcome".to_string(),
        ],
        capital_at_risk_limit: 4_000_000_000,
        escalation_conditions: vec![
            "capital-bound-would-be-exceeded".to_string(),
            "strategic-premise-broken".to_string(),
        ],
        withdrawal_conditions: vec![
            "market-thesis-falsified".to_string(),
            "liability-class-z-becomes-required".to_string(),
        ],
        evidence_requirements: vec![
            "exact-subject-receipt".to_string(),
            "independent-counterstrategy-court".to_string(),
            "replayable-campaign-lineage".to_string(),
        ],
    })
    .expect("bounded board constitution")
}

fn doctrine() -> StrategyDoctrine {
    let constitution = constitution();
    compile_strategy_doctrine(
        &constitution,
        BTreeMap::from([
            ("market:structure".to_string(), digest('1')),
            ("board:risk-appetite".to_string(), digest('2')),
            ("enterprise:capabilities".to_string(), digest('3')),
        ]),
        vec![
            "HDDL".to_string(),
            "FOND".to_string(),
            "POWL".to_string(),
            "OCEL2".to_string(),
        ],
        vec![
            "33SW".to_string(),
            "Napoleon-Berthier".to_string(),
            "DfCM".to_string(),
            "Fuller".to_string(),
        ],
    )
    .expect("bounded strategy doctrine")
}

fn partitions(doctrine: &StrategyDoctrine) -> Vec<StrategyPartition> {
    vec![
        partition_strategy(
            doctrine,
            "build",
            "build a concentrated internal capability",
            BTreeMap::from([("strategy:build:cost".to_string(), digest('a'))]),
            vec![
                "capital-at-risk-within-board-limit".to_string(),
                "preserve-exit-option".to_string(),
            ],
            vec!["concentrate".to_string(), "preserve-options".to_string()],
        )
        .expect("build partition"),
        partition_strategy(
            doctrine,
            "partner",
            "compose a coalition and avoid the strongest incumbent position",
            BTreeMap::from([("strategy:partner:counterparty".to_string(), digest('b'))]),
            vec![
                "partner-authority-does-not-expand-board-mandate".to_string(),
                "substitution-remains-available".to_string(),
            ],
            vec!["coalition".to_string(), "indirect-approach".to_string()],
        )
        .expect("partner partition"),
        partition_strategy(
            doctrine,
            "transform",
            "change the terrain so the old competition becomes less relevant",
            BTreeMap::from([("strategy:transform:ontology".to_string(), digest('c'))]),
            vec![
                "old-system-remains-reversible-until-closure".to_string(),
                "closure-has-explicit-falsifier".to_string(),
            ],
            vec!["change-terrain".to_string(), "closure".to_string()],
        )
        .expect("transform partition"),
    ]
}

fn candidate(
    constitution: &castle::strategic_command::BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    id: &str,
    falsifier: &str,
    capital: u64,
) -> CampaignCandidate {
    construct_campaign_candidate(
        constitution,
        doctrine,
        partition,
        id,
        vec![
            "competitor-responds-rationally".to_string(),
            "current-market-evidence-remains-admitted".to_string(),
        ],
        falsifier,
        BTreeMap::from([
            ("option-value-bps".to_string(), 8000),
            ("capital-efficiency-bps".to_string(), 7000),
        ]),
        vec!["market-position-improved".to_string()],
        capital,
        capital / 2,
    )
    .expect("bounded campaign candidate")
}

fn current_local(partitions: &[StrategyPartition]) -> BTreeMap<String, String> {
    partitions
        .iter()
        .flat_map(|partition| {
            partition
                .local_premise_digests
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
        })
        .collect()
}

#[test]
fn board_surface_is_exactly_eight_non_c_suite_lenses() {
    assert_eq!(BOARD_LENSES.len(), 8);
    let avatars: Vec<_> = BOARD_LENSES.iter().map(|lens| lens.avatar).collect();
    assert_eq!(
        avatars,
        vec![
            BoardAvatar::Audit,
            BoardAvatar::Risk,
            BoardAvatar::CapitalAllocation,
            BoardAvatar::Resilience,
            BoardAvatar::Governance,
            BoardAvatar::Safety,
            BoardAvatar::CompetitiveStrategy,
            BoardAvatar::LeadIndependent,
        ]
    );
    for lens in BOARD_LENSES {
        assert!(!lens.question.trim().is_empty());
        assert!(!lens.governed_surface.trim().is_empty());
    }
}

#[test]
fn strategic_operator_catalog_preserves_the_prior_art_lineage() {
    let by_id: BTreeMap<_, _> = STRATEGY_OPERATORS
        .iter()
        .map(|operator| (operator.id, operator.provenance))
        .collect();

    assert_eq!(by_id.get("concentrate"), Some(&"33SW-derived"));
    assert_eq!(by_id.get("segment"), Some(&"Napoleon-Berthier-derived"));
    assert_eq!(by_id.get("preserve-options"), Some(&"DfCM"));
    assert_eq!(by_id.get("closure"), Some(&"Fuller-Chatman"));
    assert!(by_id.contains_key("change-terrain"));
    assert!(by_id.contains_key("withdraw"));
}

#[test]
fn constitution_and_doctrine_are_non_actuating_by_construction() {
    let constitution = constitution();
    let doctrine = doctrine();

    assert_eq!(constitution.authority_ceiling, STRATEGIC_AUTHORITY_CEILING);
    assert_eq!(doctrine.authority_ceiling, STRATEGIC_AUTHORITY_CEILING);
    assert!(constitution
        .invariants
        .contains(&"SELECT != CONSTRUCT != DO".to_string()));
    assert!(constitution
        .invariants
        .contains(&"board selection cannot manufacture DO authority".to_string()));
    assert_eq!(constitution.constitution_digest.len(), 64);
    assert_eq!(doctrine.doctrine_digest.len(), 64);
}

#[test]
fn campaign_court_admits_three_isolated_bounded_strategies() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);

    for (index, partition) in partitions.iter().enumerate() {
        let candidate = candidate(
            &constitution,
            &doctrine,
            partition,
            &format!("campaign:{index}"),
            "counterexample-invalidates-this-campaign",
            1_000_000_000,
        );
        let verdict =
            judge_campaign_candidate(&constitution, &doctrine, partition, &candidate, &current);
        assert_eq!(verdict.standing, StrategicStanding::Alive);
        assert!(verdict.refusals.is_empty());
        assert_eq!(
            candidate.observed_partition_digests,
            vec![partition.partition_digest.clone()]
        );
    }
}

#[test]
fn campaign_court_refuses_constraint_weakening_cross_partition_do_and_authority_increase() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let base = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:build",
        "counterexample:build",
        1_000_000_000,
    );

    let mut weakened = base.clone();
    weakened.preserved_invariants.pop();
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &weakened, &current);
    assert!(verdict
        .refusals
        .contains(&"REFUSED:CONSTRAINT_WEAKENING".to_string()));

    let mut contaminated = base.clone();
    contaminated
        .observed_partition_digests
        .push(partitions[1].partition_digest.clone());
    let verdict = judge_campaign_candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        &contaminated,
        &current,
    );
    assert!(verdict
        .refusals
        .contains(&"REFUSED:CROSS_PARTITION_CONTAMINATION".to_string()));

    let mut do_smuggling = base.clone();
    do_smuggling.actions.push("DO".to_string());
    let verdict = judge_campaign_candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        &do_smuggling,
        &current,
    );
    assert!(verdict
        .refusals
        .contains(&"REFUSED:STRATEGY_UNBOUNDED".to_string()));

    let mut escalation = base.clone();
    escalation.authority_ceiling = "DO".to_string();
    let verdict = judge_campaign_candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        &escalation,
        &current,
    );
    assert!(verdict
        .refusals
        .contains(&"REFUSED:AUTHORITY_INCREASE".to_string()));
}

#[test]
fn campaign_court_refuses_capital_escape_and_prohibited_outcomes() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let mut candidate = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:build",
        "counterexample:build",
        constitution.capital_at_risk_limit + 1,
    );
    candidate
        .expected_outcomes
        .push("unauthorized-capital-commitment".to_string());

    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &candidate, &current);
    assert_eq!(verdict.standing, StrategicStanding::Refused);
    assert!(verdict
        .refusals
        .contains(&"REFUSED:CAPITAL_BOUND_EXCEEDED".to_string()));
    assert!(verdict
        .refusals
        .contains(&"REFUSED:PROHIBITED_OUTCOME".to_string()));
}

#[test]
fn local_premise_drift_invalidates_only_its_strategy_but_global_drift_recompiles_strategy() {
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let globals = doctrine.premise_digests.clone();
    let mut locals = current_local(&partitions);

    assert_eq!(
        determine_recompile_scope(&doctrine, &partitions, &globals, &locals),
        RecompileScope::None
    );

    locals.insert("strategy:partner:counterparty".to_string(), digest('d'));
    assert_eq!(
        determine_recompile_scope(&doctrine, &partitions, &globals, &locals),
        RecompileScope::Campaign(vec!["partner".to_string()])
    );

    let mut changed_globals = globals.clone();
    changed_globals.insert("market:structure".to_string(), digest('e'));
    assert_eq!(
        determine_recompile_scope(&doctrine, &partitions, &changed_globals, &locals),
        RecompileScope::Strategic
    );
}

#[test]
fn board_selection_is_a_separate_act_and_emits_only_an_inert_brce_bound_packet() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let candidates: Vec<_> = partitions
        .iter()
        .enumerate()
        .map(|(i, partition)| {
            candidate(
                &constitution,
                &doctrine,
                partition,
                &format!("campaign:{i}"),
                "counterexample",
                1_000_000_000,
            )
        })
        .collect();
    let verdicts: Vec<CampaignVerdict> = candidates
        .iter()
        .zip(partitions.iter())
        .map(|(candidate, partition)| {
            judge_campaign_candidate(
                &constitution,
                &doctrine,
                partition,
                candidate,
                &current,
            )
        })
        .collect();

    let packet = admit_board_selection(
        &constitution,
        &candidates,
        &verdicts,
        BoardSelectionRequest {
            candidate_id: "campaign:1".to_string(),
            selection_authority_digest: digest('f'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:00:00-07:00".to_string(),
        },
    )
    .expect("board may select an ALIVE campaign");

    assert_eq!(packet.actuation, STRATEGIC_ACTUATION);
    assert_eq!(packet.authority_ceiling, STRATEGIC_AUTHORITY_CEILING);
    assert_eq!(packet.successor_boundary, STRATEGIC_SUCCESSOR_BOUNDARY);
    assert_eq!(packet.candidate_id, "campaign:1");
    assert_eq!(packet.packet_digest.len(), 64);
}

#[test]
fn board_cannot_select_a_refused_campaign_even_if_it_prefers_it() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let mut refused = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:refused",
        "counterexample",
        1_000_000_000,
    );
    refused.actions.push("DO".to_string());
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &refused, &current);
    assert_eq!(verdict.standing, StrategicStanding::Refused);

    let err = admit_board_selection(
        &constitution,
        &[refused],
        &[verdict],
        BoardSelectionRequest {
            candidate_id: "campaign:refused".to_string(),
            selection_authority_digest: digest('9'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:01:00-07:00".to_string(),
        },
    )
    .expect_err("board preference does not rescue a refused campaign");

    assert_eq!(err, "REFUSED:BOARD_SELECTED_REFUSED_CAMPAIGN");
}

#[test]
fn board_selection_refuses_a_stale_verdict_after_same_id_candidate_mutation() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let original = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:stable-id",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &original, &current);
    assert_eq!(verdict.standing, StrategicStanding::Alive);

    let mut mutated = original;
    mutated.objectives.insert("option-value-bps".to_string(), 9999);

    let err = admit_board_selection(
        &constitution,
        &[mutated],
        &[verdict],
        BoardSelectionRequest {
            candidate_id: "campaign:stable-id".to_string(),
            selection_authority_digest: digest('5'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:03:00-07:00".to_string(),
        },
    )
    .expect_err("same ID must not let a changed candidate reuse earlier standing");

    assert_eq!(err, "REFUSED:STALE_CAMPAIGN_VERDICT");
}

#[test]
fn board_selection_refuses_ambiguous_candidate_identity() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let candidate_a = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:duplicate",
        "counterexample:a",
        1_000_000_000,
    );
    let candidate_b = candidate(
        &constitution,
        &doctrine,
        &partitions[1],
        "campaign:duplicate",
        "counterexample:b",
        1_000_000_000,
    );
    let verdict = judge_campaign_candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        &candidate_a,
        &current,
    );

    let err = admit_board_selection(
        &constitution,
        &[candidate_a, candidate_b],
        &[verdict],
        BoardSelectionRequest {
            candidate_id: "campaign:duplicate".to_string(),
            selection_authority_digest: digest('4'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:04:00-07:00".to_string(),
        },
    )
    .expect_err("candidate identity must be unique");

    assert_eq!(err, "REFUSED:AMBIGUOUS_CAMPAIGN_CANDIDATE");
}

#[test]
fn strategic_mandate_is_bound_into_existing_construct_identity_without_gaining_do() {
    use castle::castle::{ConstructRequest, PowlProcess, TestEnvelope};
    use castle::strategic_command::bind_strategic_mandate_construct_request;
    use serde_json::json;

    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:bind",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('3'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:05:00-07:00".to_string(),
        },
    )
    .expect("board mandate");

    let request = ConstructRequest {
        subject: constitution.subject.clone(),
        authority: "bounded-test".to_string(),
        o_star: json!({"subject": constitution.subject}),
        config_graph: json!({"zeroUnreceiptedActuation": true}),
        ontology: json!({"version": "castle-board-v26.9.28"}),
        process: PowlProcess {
            id: "powl:board".to_string(),
            goal_id: "goal:board".to_string(),
            activities: vec![],
        },
        envelope: TestEnvelope {
            system_id: constitution.subject.clone(),
            allowed_transition_ids: BTreeSet::new(),
            max_steps: 0,
            expires_at_epoch_ms: 100,
        },
    };

    let bound = bind_strategic_mandate_construct_request(request, &mandate)
        .expect("inert mandate binds into config receipt identity");
    let marker = &bound.config_graph["_castle_board_mandate"];
    assert_eq!(marker["packet_digest"], mandate.packet_digest);
    assert_eq!(marker["candidate_digest"], mandate.candidate_digest);
    assert_eq!(marker["authority_ceiling"], STRATEGIC_AUTHORITY_CEILING);
    assert_eq!(marker["actuation"], STRATEGIC_ACTUATION);
    assert_eq!(marker["successor_boundary"], STRATEGIC_SUCCESSOR_BOUNDARY);
}

#[test]
fn strategic_mandate_binding_refuses_subject_content_or_authority_drift() {
    use castle::castle::{ConstructRequest, PowlProcess, TestEnvelope};
    use castle::strategic_command::bind_strategic_mandate_construct_request;
    use serde_json::json;

    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:bind-refusal",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('2'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:06:00-07:00".to_string(),
        },
    )
    .expect("board mandate");

    let request = |subject: &str| ConstructRequest {
        subject: subject.to_string(),
        authority: "bounded-test".to_string(),
        o_star: json!({}),
        config_graph: json!({}),
        ontology: json!({}),
        process: PowlProcess {
            id: "p".to_string(),
            goal_id: "g".to_string(),
            activities: vec![],
        },
        envelope: TestEnvelope {
            system_id: subject.to_string(),
            allowed_transition_ids: BTreeSet::new(),
            max_steps: 0,
            expires_at_epoch_ms: 100,
        },
    };

    assert_eq!(
        bind_strategic_mandate_construct_request(request("enterprise:other"), &mandate)
            .expect_err("wrong subject must refuse"),
        "REFUSED:STRATEGIC_MANDATE_SUBJECT_MISMATCH"
    );

    let mut tampered = mandate.clone();
    tampered.candidate_id = "campaign:tampered".to_string();
    assert_eq!(
        bind_strategic_mandate_construct_request(request(&constitution.subject), &tampered)
            .expect_err("changed content must invalidate packet digest"),
        "REFUSED:STRATEGIC_MANDATE_CONTENT_MISMATCH"
    );

    let mut escalated = mandate;
    escalated.authority_ceiling = "DO";
    assert_eq!(
        bind_strategic_mandate_construct_request(request(&constitution.subject), &escalated)
            .expect_err("strategy cannot gain DO at binding"),
        "REFUSED:STRATEGIC_MANDATE_AUTHORITY_DRIFT"
    );
}

#[test]
fn replanning_escalates_only_as_far_as_observed_divergence_requires() {
    assert_eq!(
        route_replan(DivergenceEvidence {
            policy_branch_available: true,
            valid_suffix_available: true,
            tail_repair_available: true,
            hierarchy_premise_broken: false,
            strategic_premise_broken: false,
        }),
        ReplanLevel::PolicyBranch
    );
    assert_eq!(
        route_replan(DivergenceEvidence {
            policy_branch_available: false,
            valid_suffix_available: true,
            tail_repair_available: true,
            hierarchy_premise_broken: false,
            strategic_premise_broken: false,
        }),
        ReplanLevel::SuffixReuse
    );
    assert_eq!(
        route_replan(DivergenceEvidence {
            policy_branch_available: false,
            valid_suffix_available: false,
            tail_repair_available: true,
            hierarchy_premise_broken: false,
            strategic_premise_broken: false,
        }),
        ReplanLevel::FollowBiasedTailRepair
    );
    assert_eq!(
        route_replan(DivergenceEvidence {
            policy_branch_available: false,
            valid_suffix_available: false,
            tail_repair_available: false,
            hierarchy_premise_broken: true,
            strategic_premise_broken: false,
        }),
        ReplanLevel::HierarchyRecompile
    );
    assert_eq!(
        route_replan(DivergenceEvidence {
            policy_branch_available: true,
            valid_suffix_available: true,
            tail_repair_available: true,
            hierarchy_premise_broken: true,
            strategic_premise_broken: true,
        }),
        ReplanLevel::StrategicRecompile
    );
}

#[test]
fn board_receipt_surfaces_residual_consequence_and_authority_witnesses() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[2],
        "campaign:transform",
        "counterexample:transformation-does-not-change-option-space",
        2_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[2], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('8'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T21:02:00-07:00".to_string(),
        },
    )
    .expect("selected campaign mandate");

    let alive = compile_board_strategic_receipt(
        &constitution,
        &campaign,
        &mandate,
        BoardReceiptInput {
            falsified_premises: vec![],
            material_exceptions: vec![],
            options_remaining: 4,
            prohibited_outcome_witnesses: vec![],
            authority_expansions: vec![],
            next_board_decision: Some("only if trigger-c occurs".to_string()),
            evidence_digest: digest('7'),
        },
    )
    .expect("board receipt");
    assert_eq!(alive.standing, StrategicStanding::Alive);
    assert_eq!(alive.receipt_digest.len(), 64);

    let refused = compile_board_strategic_receipt(
        &constitution,
        &campaign,
        &mandate,
        BoardReceiptInput {
            falsified_premises: vec!["market-thesis".to_string()],
            material_exceptions: vec!["exception:1".to_string()],
            options_remaining: 2,
            prohibited_outcome_witnesses: vec![
                "path:unauthorized-capital-commitment".to_string(),
            ],
            authority_expansions: vec!["authority:unapproved-expansion".to_string()],
            next_board_decision: Some("revise-or-withdraw".to_string()),
            evidence_digest: digest('6'),
        },
    )
    .expect("receipt is emitted even when standing is refused");
    assert_eq!(refused.standing, StrategicStanding::Refused);
    assert_eq!(
        refused.prohibited_outcome_witnesses,
        vec!["path:unauthorized-capital-commitment".to_string()]
    );
    assert_eq!(
        refused.authority_expansions,
        vec!["authority:unapproved-expansion".to_string()]
    );
}


#[test]
fn counterstrategy_court_falsifies_campaigns_without_gaining_authority() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:counterstrategy",
        "competitor-response-breaks-build-thesis",
        1_000_000_000,
    );

    let benign = CounterstrategyScenario {
        scenario_id: "response:benign".to_string(),
        local_premise_mutations: BTreeMap::new(),
        falsifier_triggered: false,
        prohibited_outcomes_reached: vec![],
        authority_expansion_attempts: vec![],
        additional_capital_required: 100_000_000,
        remaining_options: 3,
    };
    let benign_verdict =
        judge_counterstrategy(&constitution, &partitions[0], &campaign, &benign).unwrap();
    assert_eq!(benign_verdict.standing, StrategicStanding::Alive);
    assert_eq!(benign_verdict.verdict_digest.len(), 64);

    let hostile = CounterstrategyScenario {
        scenario_id: "response:hostile".to_string(),
        local_premise_mutations: BTreeMap::from([(
            "strategy:build:cost".to_string(),
            digest('d'),
        )]),
        falsifier_triggered: true,
        prohibited_outcomes_reached: vec!["unauthorized-capital-commitment".to_string()],
        authority_expansion_attempts: vec!["grant-do-to-strategy-compiler".to_string()],
        additional_capital_required: 4_000_000_000,
        remaining_options: 0,
    };
    let hostile_verdict =
        judge_counterstrategy(&constitution, &partitions[0], &campaign, &hostile).unwrap();
    assert_eq!(hostile_verdict.standing, StrategicStanding::Refused);
    assert!(hostile_verdict
        .refusals
        .contains(&"REFUSED:CAMPAIGN_FALSIFIER_TRIGGERED".to_string()));
    assert!(hostile_verdict
        .refusals
        .iter()
        .any(|reason| reason.starts_with("REFUSED:COUNTERSTRATEGY_FALSIFIED_PREMISE:")));
    assert!(hostile_verdict
        .refusals
        .contains(&"REFUSED:COUNTERSTRATEGY_REACHES_PROHIBITED_OUTCOME".to_string()));
    assert!(hostile_verdict
        .refusals
        .contains(&"REFUSED:COUNTERSTRATEGY_REQUIRES_AUTHORITY_EXPANSION".to_string()));
    assert!(hostile_verdict
        .refusals
        .contains(&"REFUSED:COUNTERSTRATEGY_EXCEEDS_CAPITAL_BOUND".to_string()));
    assert!(hostile_verdict
        .refusals
        .contains(&"REFUSED:COUNTERSTRATEGY_EXHAUSTS_OPTION_SPACE".to_string()));

    let assessment =
        assess_counterstrategies(&constitution, &partitions[0], &campaign, &[benign, hostile])
            .unwrap();
    assert_eq!(assessment.standing, StrategicStanding::Refused);
    assert_eq!(assessment.verdicts.len(), 2);
    assert_eq!(assessment.assessment_digest.len(), 64);
}

#[test]
fn portfolio_analysis_enforces_capital_concentration_and_reversibility_without_selecting_a_winner() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);

    let campaigns = vec![
        candidate(
            &constitution,
            &doctrine,
            &partitions[0],
            "campaign:portfolio-build",
            "counterexample:build",
            1_000_000_000,
        ),
        candidate(
            &constitution,
            &doctrine,
            &partitions[1],
            "campaign:portfolio-partner",
            "counterexample:partner",
            1_000_000_000,
        ),
    ];
    let verdicts: Vec<_> = campaigns
        .iter()
        .zip(partitions.iter())
        .map(|(campaign, partition)| {
            judge_campaign_candidate(&constitution, &doctrine, partition, campaign, &current)
        })
        .collect();

    let analysis = qualify_campaign_portfolio(
        &campaigns,
        &verdicts,
        &CampaignPortfolioPolicy {
            aggregate_capital_at_risk_limit: 3_000_000_000,
            max_single_campaign_concentration_bps: 6_000,
            min_reversible_capital_bps: 4_000,
        },
    )
    .unwrap();

    assert_eq!(analysis.standing, StrategicStanding::Alive);
    assert_eq!(analysis.aggregate_capital_committed, 2_000_000_000);
    assert_eq!(analysis.aggregate_reversible_capital, 1_000_000_000);
    assert_eq!(analysis.reversible_capital_bps, 5_000);
    assert_eq!(analysis.max_single_campaign_concentration_bps, 5_000);
    assert_eq!(
        analysis.campaign_ids,
        vec![
            "campaign:portfolio-build".to_string(),
            "campaign:portfolio-partner".to_string()
        ]
    );
    assert_eq!(analysis.analysis_digest.len(), 64);

    let refused = qualify_campaign_portfolio(
        &campaigns,
        &verdicts,
        &CampaignPortfolioPolicy {
            aggregate_capital_at_risk_limit: 1_500_000_000,
            max_single_campaign_concentration_bps: 4_000,
            min_reversible_capital_bps: 6_000,
        },
    )
    .unwrap();
    assert_eq!(refused.standing, StrategicStanding::Refused);
    assert!(refused
        .reasons
        .contains(&"REFUSED:PORTFOLIO_CAPITAL_BOUND_EXCEEDED".to_string()));
    assert!(refused
        .reasons
        .contains(&"REFUSED:PORTFOLIO_CONCENTRATION_BOUND_EXCEEDED".to_string()));
    assert!(refused
        .reasons
        .contains(&"REFUSED:PORTFOLIO_REVERSIBILITY_BELOW_BOUND".to_string()));
}

#[test]
fn materiality_and_strategic_falsification_force_explicit_board_reentry() {
    use castle::board::{MaterialityDimension, MaterialityEvent, MaterialityPolicy};

    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:materiality",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('a'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T22:00:00-07:00".to_string(),
        },
    )
    .unwrap();
    let receipt = compile_board_strategic_receipt(
        &constitution,
        &campaign,
        &mandate,
        BoardReceiptInput {
            falsified_premises: vec!["market-thesis".to_string()],
            material_exceptions: vec![],
            options_remaining: 2,
            prohibited_outcome_witnesses: vec![],
            authority_expansions: vec![],
            next_board_decision: Some("revise-campaign".to_string()),
            evidence_digest: digest('b'),
        },
    )
    .unwrap();

    let event = MaterialityEvent {
        id: "event:material".to_string(),
        subject: constitution.subject.clone(),
        occurred_at_epoch_ms: 1_000,
        impact_bps: BTreeMap::from([(MaterialityDimension::Financial, 750)]),
    };
    let policy = MaterialityPolicy {
        policy_digest: digest('c'),
        authority_digest: digest('d'),
        per_dimension_threshold_bps: BTreeMap::from([(MaterialityDimension::Financial, 500)]),
        aggregate_threshold_bps: 9_000,
        escalation_within_ms: 3_600_000,
    };

    let reentry = assess_board_reentry(&constitution, &receipt, &event, &policy).unwrap();
    assert!(reentry.required);
    assert!(reentry
        .reasons
        .contains(&"MATERIALITY_THRESHOLD_CROSSED".to_string()));
    assert!(reentry
        .reasons
        .contains(&"STRATEGIC_PREMISE_FALSIFIED".to_string()));
    assert_eq!(reentry.escalate_by_epoch_ms, Some(3_601_000));
}

#[test]
fn strategic_twin_collapses_runtime_state_to_a_board_delta() {
    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:twin",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('e'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T22:01:00-07:00".to_string(),
        },
    )
    .unwrap();
    let receipt = compile_board_strategic_receipt(
        &constitution,
        &campaign,
        &mandate,
        BoardReceiptInput {
            falsified_premises: vec![],
            material_exceptions: vec![],
            options_remaining: 4,
            prohibited_outcome_witnesses: vec![],
            authority_expansions: vec![],
            next_board_decision: None,
            evidence_digest: digest('f'),
        },
    )
    .unwrap();
    let portfolio = qualify_campaign_portfolio(
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        &CampaignPortfolioPolicy {
            aggregate_capital_at_risk_limit: 2_000_000_000,
            max_single_campaign_concentration_bps: 10_000,
            min_reversible_capital_bps: 5_000,
        },
    )
    .unwrap();
    let scenario = CounterstrategyScenario {
        scenario_id: "response:stable".to_string(),
        local_premise_mutations: BTreeMap::new(),
        falsifier_triggered: false,
        prohibited_outcomes_reached: vec![],
        authority_expansion_attempts: vec![],
        additional_capital_required: 0,
        remaining_options: 4,
    };
    let counterstrategy = assess_counterstrategies(
        &constitution,
        &partitions[0],
        &campaign,
        &[scenario],
    )
    .unwrap();

    let previous = build_strategic_twin_snapshot(
        &constitution,
        &mandate,
        &receipt,
        &portfolio,
        &counterstrategy,
    )
    .unwrap();
    assert_eq!(previous.standing, StrategicStanding::Alive);

    let mut current_snapshot = previous.clone();
    current_snapshot.options_remaining = 2;
    current_snapshot.standing = StrategicStanding::Refused;
    current_snapshot
        .falsified_premises
        .push("market-thesis".to_string());
    current_snapshot
        .failed_counterstrategy_scenarios
        .push("response:hostile".to_string());
    current_snapshot.snapshot_digest = digest('1');

    let delta = diff_strategic_twins(&previous, &current_snapshot).unwrap();
    assert!(delta.requires_board_attention);
    assert!(delta
        .changed_dimensions
        .contains(&"options-remaining".to_string()));
    assert!(delta
        .changed_dimensions
        .contains(&"falsified-premises".to_string()));
    assert!(delta
        .changed_dimensions
        .contains(&"counterstrategy".to_string()));
    assert!(delta.changed_dimensions.contains(&"standing".to_string()));
}

#[test]
fn strategic_board_package_is_self_contained_and_offline_verifiable() {
    use castle::board::{
        BoardPackage, MaterialityDimension, MaterialityEvent, MaterialityPolicy,
    };
    use castle::fortune5::Standing;

    let constitution = constitution();
    let doctrine = doctrine();
    let partitions = partitions(&doctrine);
    let current = current_local(&partitions);
    let campaign = candidate(
        &constitution,
        &doctrine,
        &partitions[0],
        "campaign:package",
        "counterexample",
        1_000_000_000,
    );
    let verdict =
        judge_campaign_candidate(&constitution, &doctrine, &partitions[0], &campaign, &current);
    let mandate = admit_board_selection(
        &constitution,
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        BoardSelectionRequest {
            candidate_id: campaign.candidate_id.clone(),
            selection_authority_digest: digest('2'),
            selected_by: "board:independent-directors".to_string(),
            selected_at: "2026-09-28T22:02:00-07:00".to_string(),
        },
    )
    .unwrap();
    let receipt = compile_board_strategic_receipt(
        &constitution,
        &campaign,
        &mandate,
        BoardReceiptInput {
            falsified_premises: vec![],
            material_exceptions: vec![],
            options_remaining: 4,
            prohibited_outcome_witnesses: vec![],
            authority_expansions: vec![],
            next_board_decision: None,
            evidence_digest: digest('3'),
        },
    )
    .unwrap();
    let portfolio = qualify_campaign_portfolio(
        std::slice::from_ref(&campaign),
        std::slice::from_ref(&verdict),
        &CampaignPortfolioPolicy {
            aggregate_capital_at_risk_limit: 2_000_000_000,
            max_single_campaign_concentration_bps: 10_000,
            min_reversible_capital_bps: 5_000,
        },
    )
    .unwrap();
    let scenario = CounterstrategyScenario {
        scenario_id: "response:stable".to_string(),
        local_premise_mutations: BTreeMap::new(),
        falsifier_triggered: false,
        prohibited_outcomes_reached: vec![],
        authority_expansion_attempts: vec![],
        additional_capital_required: 0,
        remaining_options: 4,
    };
    let counterstrategy = assess_counterstrategies(
        &constitution,
        &partitions[0],
        &campaign,
        &[scenario],
    )
    .unwrap();
    let twin = build_strategic_twin_snapshot(
        &constitution,
        &mandate,
        &receipt,
        &portfolio,
        &counterstrategy,
    )
    .unwrap();

    let event = MaterialityEvent {
        id: "event:nonmaterial".to_string(),
        subject: constitution.subject.clone(),
        occurred_at_epoch_ms: 2_000,
        impact_bps: BTreeMap::from([(MaterialityDimension::Financial, 100)]),
    };
    let policy = MaterialityPolicy {
        policy_digest: digest('4'),
        authority_digest: digest('5'),
        per_dimension_threshold_bps: BTreeMap::from([(MaterialityDimension::Financial, 500)]),
        aggregate_threshold_bps: 9_000,
        escalation_within_ms: 3_600_000,
    };
    let reentry = assess_board_reentry(&constitution, &receipt, &event, &policy).unwrap();
    assert!(!reentry.required);

    let base = BoardPackage {
        profile: "CASTLE_FORTUNE5_BOARD_V1",
        enterprise_subject: constitution.subject.clone(),
        castle_subject: "castle:self".to_string(),
        generated_at: "2026-09-28T22:02:00-07:00".to_string(),
        enterprise_standing: Standing::Alive,
        castle_standing: Standing::Alive,
        material_refused_subjects: 0,
        risk_appetite_breaches: 0,
        control_count: 40,
        evidence_digest: digest('6'),
    };

    let package = build_strategic_board_package(
        &base,
        &constitution,
        &mandate,
        &receipt,
        &portfolio,
        &counterstrategy,
        &twin,
        &reentry,
        "2026-09-28T22:03:00-07:00",
    )
    .unwrap();

    assert_eq!(package.profile, "CASTLE_STRATEGIC_BOARD_V1");
    assert_eq!(package.authority_ceiling, STRATEGIC_AUTHORITY_CEILING);
    assert_eq!(package.actuation, STRATEGIC_ACTUATION);
    assert_eq!(package.package_digest.len(), 64);
    let verification = verify_strategic_board_package_offline(&package);
    assert_eq!(verification.standing, StrategicStanding::Alive);
    assert!(verification.reasons.is_empty());

    let mut tampered = package;
    tampered.board_reentry_required = true;
    let verification = verify_strategic_board_package_offline(&tampered);
    assert_eq!(verification.standing, StrategicStanding::Refused);
    assert!(verification
        .reasons
        .contains(&"REFUSED:STRATEGIC_BOARD_PACKAGE_CONTENT_MISMATCH".to_string()));
}
