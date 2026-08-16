//! Subprocess-based integration tests for the `castle replay`, `castle
//! impact`, and `castle inventory` CLI surfaces — completes the coverage
//! `tests/cli_fortune5.rs` started for the `fortune5` noun. Chicago-style:
//! real compiled binary, real subprocess, real JSON fixture files.

use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn run(args: &[&str]) -> (bool, serde_json::Value, String) {
    let output = Command::new(castle_bin()).args(args).output().expect("failed to spawn castle binary");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    let parsed = if stdout.trim().is_empty() { serde_json::Value::Null } else { serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}\nstderr: {stderr}")) };
    (output.status.success(), parsed, stderr)
}

/// `castle replay admit` with a fully self-consistent manifest/subject (the
/// CLI passes the same values for both sides) admits ALIVE.
#[test]
fn replay_admit_with_self_consistent_signature_is_alive() {
    let digest64 = "1".repeat(64);
    let invariant_digest = "2".repeat(64);
    let process_digest = "3".repeat(64);
    let (ok, json, stderr) = run(&[
        "replay",
        "admit",
        "--replay-class-id",
        "class:authority-loss:v1",
        "--structural-signature",
        &digest64,
        "--ontology-version",
        "castle-ontology:26.8.15",
        "--provider-semantics-version",
        "aws-control-plane:2026-08-15",
        "--invariant-set-digest",
        &invariant_digest,
        "--process-digest",
        &process_digest,
        "--invariants-hold",
        "--format",
        "json",
    ]);
    assert!(ok, "castle replay admit exited non-zero: {stderr}");
    assert_eq!(json["standing"].as_str(), Some("ALIVE"));
    assert_eq!(json["replayClassId"].as_str(), Some("class:authority-loss:v1"));
}

/// Omitting `--invariants-hold` (a boolean flag) means the CLI-side subject
/// evaluates as `invariants_hold: false`, so admission is REFUSED.
#[test]
fn replay_admit_without_invariants_hold_flag_is_refused() {
    let digest64 = "4".repeat(64);
    let invariant_digest = "5".repeat(64);
    let process_digest = "6".repeat(64);
    let (ok, json, stderr) = run(&[
        "replay",
        "admit",
        "--replay-class-id",
        "class:integrity-loss:v1",
        "--structural-signature",
        &digest64,
        "--ontology-version",
        "castle-ontology:26.8.15",
        "--provider-semantics-version",
        "aws-control-plane:2026-08-15",
        "--invariant-set-digest",
        &invariant_digest,
        "--process-digest",
        &process_digest,
        "--format",
        "json",
    ]);
    assert!(ok, "castle replay admit exited non-zero: {stderr}");
    assert_eq!(json["standing"].as_str(), Some("REFUSED"));
    let reasons = json["reasons"].as_array().expect("reasons must be an array");
    assert!(reasons.iter().any(|r| r.as_str() == Some("REFUSED:INVARIANTS_NOT_SATISFIED")), "expected INVARIANTS_NOT_SATISFIED, got {reasons:?}");
}

/// `castle impact coverage` against a known evidence fixture (authority 60,
/// confidentiality 20, integrity 10, availability 10) selects the smallest
/// prefix reaching 8000bps: authority + confidentiality.
#[test]
fn impact_coverage_selects_the_minimal_pareto_prefix() {
    let (ok, json, stderr) = run(&["impact", "coverage", "--classes-path", "tests/fixtures/impact_classes.json", "--target-coverage-bps", "8000", "--format", "json"]);
    assert!(ok, "castle impact coverage exited non-zero: {stderr}");
    assert_eq!(json["coverageBps"].as_i64(), Some(8000));
    assert_eq!(json["totalImpact"].as_f64(), Some(100.0));
    assert_eq!(json["selectedImpact"].as_f64(), Some(80.0));
    let selected = json["selected"].as_array().expect("selected must be an array");
    let keys: Vec<&str> = selected.iter().map(|c| c["key"].as_str().unwrap()).collect();
    assert_eq!(keys, vec!["authority", "confidentiality"]);
}

/// `castle inventory components` lists all 10 marketplace-generated
/// architecture components.
#[test]
fn inventory_components_lists_all_ten_components() {
    let (ok, json, stderr) = run(&["inventory", "components", "--format", "json"]);
    assert!(ok, "castle inventory components exited non-zero: {stderr}");
    assert_eq!(json["count"].as_u64(), Some(10));
    let components = json["components"].as_array().expect("components must be an array");
    assert_eq!(components.len(), 10);
    assert!(components.iter().any(|c| c["identifier"].as_str() == Some("DfCMGoalInversion")));
}

/// `castle inventory goals` lists all 5 marketplace-generated default
/// prohibited adversarial goals.
#[test]
fn inventory_goals_lists_all_five_default_goals() {
    let (ok, json, stderr) = run(&["inventory", "goals", "--format", "json"]);
    assert!(ok, "castle inventory goals exited non-zero: {stderr}");
    assert_eq!(json["count"].as_u64(), Some(5));
    let goals = json["goals"].as_array().expect("goals must be an array");
    assert_eq!(goals.len(), 5);
    assert_eq!(goals[0]["id"].as_str(), Some("unauthorized-authority"));
}
