use std::collections::{BTreeMap, BTreeSet};

const GOAL: &str = include_str!("../docs/sjira/v26.9.28/goal.ttl");
const REPOS: &str = include_str!("../docs/sjira/v26.9.28/repos.ttl");
const STOP: &str = include_str!("../docs/sjira/v26.9.28/courts/stop.rq");
const OWNERSHIP: &str = include_str!("../docs/sjira/v26.9.28/courts/ownership.rq");
const RUNTIME_CROWN: &str = include_str!("../docs/sjira/v26.9.28/courts/runtime_crown.rq");
const EXCLUSIONS: &str = include_str!("../docs/sjira/v26.9.28/courts/exclusions.rq");

fn strip_triple_quoted(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    let mut inside = false;

    while let Some(index) = rest.find("\"\"\"") {
        if !inside {
            out.push_str(&rest[..index]);
        }
        rest = &rest[index + 3..];
        inside = !inside;
    }

    assert!(!inside, "unclosed Turtle triple-quoted literal");
    out.push_str(rest);
    out
}

fn quoted_value(block: &str, predicate: &str) -> Option<String> {
    let line = block
        .lines()
        .find(|line| line.trim_start().starts_with(predicate))?;
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

#[test]
fn goal_graph_is_a_goal_graph_not_a_hand_written_backlog() {
    let graph = strip_triple_quoted(GOAL);

    assert!(
        !graph.contains("a sj:WorkOrder"),
        "goal.ttl must never author WorkOrder instances"
    );
    assert!(
        !graph.contains("a sj:Receipt"),
        "goal.ttl must never author Receipt instances"
    );
    assert!(
        !graph.contains(r#"sj:standing "ALIVE""#),
        "standing is observed by courts, never declared by goal.ttl"
    );

    assert_eq!(
        graph.matches(r#"sj:authorityCeiling "CONSTRUCT""#).count(),
        16,
        "root + successor + fourteen gates must all stop at CONSTRUCT"
    );
    assert!(
        !graph.contains(r#"sj:authorityCeiling "DO""#),
        "sJira may not mint consequential DO authority"
    );
}

#[test]
fn crown_has_exactly_fourteen_required_gates() {
    let graph = strip_triple_quoted(GOAL);

    for gate in 0..14 {
        let id = format!(r#"dcterms:identifier "CASTLE-28-{gate}""#);
        assert!(graph.contains(&id), "missing crown gate CASTLE-28-{gate}");
    }

    assert_eq!(
        graph.matches(r#"dcterms:identifier "CASTLE-28-"#).count(),
        14,
        "unexpected crown-gate cardinality"
    );
}

#[test]
fn crown_preserves_runtime_and_consequence_separation() {
    let graph = strip_triple_quoted(GOAL);

    assert!(graph.contains("Runtime crown: XaaS owns runtime existence and composition"));
    assert!(graph.contains(
        "Product crown: CASTLE owns board constitution and consequence admissibility"
    ));
    assert!(graph.contains("One protected BRCE consequence boundary"));
    assert!(graph.contains("Board SELECT never implies DO"));
}

#[test]
fn repository_registry_is_exact_subject_and_duplicate_free() {
    let mut repositories = BTreeSet::new();
    let mut count = 0usize;

    for block in REPOS.split("\n\n") {
        if !block.contains("a c:RepositorySubject") {
            continue;
        }
        count += 1;
        let repository = quoted_value(block, "sj:repository ")
            .expect("every RepositorySubject must declare sj:repository");
        let base_sha =
            quoted_value(block, "sj:baseSha ").expect("every RepositorySubject must pin sj:baseSha");

        assert_eq!(base_sha.len(), 40, "{repository} must pin an exact git SHA");
        assert!(
            base_sha.chars().all(|ch| ch.is_ascii_hexdigit()),
            "{repository} baseSha must be hexadecimal"
        );
        assert!(
            repositories.insert(repository.clone()),
            "duplicate repository subject: {repository}"
        );
    }

    assert_eq!(count, 45, "repository conservation set changed");
    assert_eq!(repositories.len(), 45);
}

#[test]
fn retained_irreducible_capabilities_have_exactly_one_owner() {
    let mut owners = BTreeMap::<String, String>::new();
    let mut retained = 0usize;

    for block in REPOS.split("\n\n") {
        if !block.contains("a c:RepositorySubject") {
            continue;
        }
        if !block.contains(r#"c:disposition "KEEP" ;"#) {
            continue;
        }

        retained += 1;
        let repository = quoted_value(block, "sj:repository ").unwrap();
        let capability = quoted_value(block, "c:ownsCapability ")
            .unwrap_or_else(|| panic!("{repository} is KEEP but owns no irreducible capability"));

        assert!(
            owners.insert(capability.clone(), repository.clone()).is_none(),
            "duplicate irreducible owner for {capability}"
        );
    }

    assert_eq!(retained, 22);
    assert_eq!(owners.len(), 22);
    assert_eq!(
        owners.get("RUNTIME_EXISTENCE").map(String::as_str),
        Some("seanchatmangpt/xaas")
    );
    assert_eq!(
        owners.get("CONSEQUENTIAL_ADMISSIBILITY").map(String::as_str),
        Some("seanchatmangpt/castle")
    );
    assert_eq!(
        owners.get("SEMANTIC_LAW_DERIVATION").map(String::as_str),
        Some("seanchatmangpt/graphlaw")
    );
    assert_eq!(
        owners.get("CRYPTOGRAPHIC_STANDING").map(String::as_str),
        Some("seanchatmangpt/affidavit")
    );
}

#[test]
fn supporting_candidates_are_non_sovereign_until_dispositioned() {
    let mut supporting = 0usize;

    for block in REPOS.split("\n\n") {
        if !block.contains(r#"c:disposition "SUPPORTING_CANDIDATE""#) {
            continue;
        }
        supporting += 1;
        assert!(
            !block.contains("c:ownsCapability"),
            "supporting candidate may not claim irreducible ownership before disposition"
        );
    }

    assert_eq!(supporting, 7);
}

#[test]
fn live_crown_subjects_are_pinned_to_observed_heads() {
    assert!(REPOS.contains(
        "sj:repository \"seanchatmangpt/castle\" ;\n    sj:baseSha \"d854333ead2b5f47cc38bbf5680dd7db13fbf009\""
    ));
    assert!(REPOS.contains(
        "sj:repository \"seanchatmangpt/xaas\" ;\n    sj:baseSha \"f8ea07db9c588633ea6a2453dfd133977aa7a0d3\""
    ));
}

#[test]
fn stop_court_is_receipt_driven_and_anti_vacuous() {
    for gate in 0..14 {
        let id = format!(r#""CASTLE-28-{gate}""#);
        assert!(STOP.contains(&id), "stop court omits {id}");
    }
    assert!(STOP.contains("sj:receipt ?receipt"));
    assert!(STOP.contains(r#"sj:standing "ALIVE""#));
    assert!(STOP.contains("FILTER NOT EXISTS"));
    assert!(STOP.contains("?order a sj:WorkOrder"));
}

#[test]
fn supporting_courts_encode_the_architectural_laws() {
    assert!(OWNERSHIP.contains("HAVING(COUNT(?repo) != 1)"));
    assert!(RUNTIME_CROWN.contains(r#""RUNTIME_EXISTENCE""#));
    assert!(RUNTIME_CROWN.contains(r#""seanchatmangpt/xaas""#));
    assert!(RUNTIME_CROWN.contains(r#""CONSEQUENTIAL_ADMISSIBILITY""#));
    assert!(RUNTIME_CROWN.contains(r#""seanchatmangpt/castle""#));
    assert!(EXCLUSIONS.contains("?subject a sj:WorkOrder"));
    assert!(EXCLUSIONS.contains("?subject a sj:Receipt"));
}
