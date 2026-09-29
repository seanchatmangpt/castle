const GRAPH: &str = include_str!("../docs/sjira/v26.9.28/capability-intake.ttl");

#[test]
fn projected_product_capabilities_terminate_at_castle_without_runtime_takeover() {
    assert!(GRAPH.contains("50fdfa20c84205a80c6eb94e916cffbedc4b816e"));
    assert!(GRAPH.contains("CONSEQUENTIAL_ADMISSIBILITY"));
    assert!(GRAPH.contains("runtimeCore \"seanchatmangpt/xaas\""));
    assert!(GRAPH.contains("projectionStanding \"CANDIDATE\""));
    assert!(GRAPH.contains("authorityCeiling \"CONSTRUCT\""));
    assert!(!GRAPH.contains("authorityCeiling \"DO\""));
    assert_eq!(GRAPH.matches("a eco:ProjectedCapability").count(), 5);
}

#[test]
fn all_product_donors_are_exact_and_non_sovereign() {
    let donors = [
        ("seanchatmangpt/ash_surface", "c066a55f452f23b348df375c00aae7811f6f72be"),
        ("seanchatmangpt/zoela", "cae4a862744380f6a78b962e3f345ede8bd860dd"),
        ("seanchatmangpt/cargo-cicd", "59214d6a1293d794b7a27095276047cdff73a9ec"),
        ("seanchatmangpt/chatman-ecosystem", "92cb17cda899a8d85abdaa04db10a3d2334e116d"),
        ("seanchatmangpt/chatman-nano-stack", "de7eb619310a522e44e9d6cf20e9460806b314ea"),
    ];

    for (repo, sha) in donors {
        assert!(GRAPH.contains(repo), "missing donor {repo}");
        assert!(GRAPH.contains(sha), "missing exact donor subject {repo}@{sha}");
    }

    assert!(!GRAPH.contains("RUNTIME_CORE"));
    assert!(!GRAPH.contains("eco:ownerCapability \"RUNTIME_EXISTENCE\""));
}
