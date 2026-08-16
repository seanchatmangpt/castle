//! Subprocess-based integration tests for the `castle fortune5` CLI surface.
//!
//! Chicago-style: invokes the real, compiled `castle` binary (located via
//! Cargo's standard `CARGO_BIN_EXE_<name>` mechanism — no `assert_cmd`
//! dev-dependency needed) as a real child process, reads real stdout/stderr,
//! and asserts on real parsed JSON / exit codes. No mocking.

use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

/// `castle fortune5 requirements` prints valid JSON listing all 40
/// Fortune-5 readiness controls.
#[test]
fn fortune5_requirements_prints_all_forty_controls_as_json() {
    let output = Command::new(castle_bin())
        .args(["fortune5", "requirements", "--format", "json"])
        .output()
        .expect("failed to spawn castle binary");

    assert!(output.status.success(), "castle fortune5 requirements exited non-zero: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));

    assert_eq!(parsed["count"].as_u64(), Some(40));
    let requirements = parsed["requirements"].as_array().expect("requirements field must be a JSON array");
    assert_eq!(requirements.len(), 40);

    // Spot-check the shape of the first control against the known ordering.
    let first = &requirements[0];
    assert_eq!(first["controlId"].as_str(), Some("F5-AUTH-001"));
    assert!(first["metric"].is_string());
    assert!(first["comparator"].is_string());
    assert!(first["target"].is_string());
    assert!(first["authority"].is_string());
}

/// `castle fortune5 qualify` against the real evidence fixture (all 40
/// controls satisfied) exits 0 and reports ALIVE standing with alive: 40.
#[test]
fn fortune5_qualify_with_alive_fixture_reports_alive_standing() {
    let output = Command::new(castle_bin())
        .args([
            "fortune5",
            "qualify",
            "--subject",
            "castle:cli-fixture",
            "--evidence-path",
            "tests/fixtures/fortune5_evidence_alive.json",
            "--now-epoch-ms",
            "1786824000000",
            "--max-evidence-age-ms",
            "300000",
            "--format",
            "json",
        ])
        .output()
        .expect("failed to spawn castle binary");

    assert!(output.status.success(), "castle fortune5 qualify exited non-zero: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout).expect("stdout was not valid UTF-8");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stdout was not valid JSON ({e}): {stdout}"));

    assert_eq!(parsed["standing"].as_str(), Some("ALIVE"));
    assert_eq!(parsed["alive"].as_u64(), Some(40));
    assert_eq!(parsed["refused"].as_u64(), Some(0));
    assert_eq!(parsed["unknown"].as_u64(), Some(0));
    assert_eq!(parsed["subject"].as_str(), Some("castle:cli-fixture"));

    let controls = parsed["controls"].as_array().expect("controls field must be a JSON array");
    assert_eq!(controls.len(), 40);
    assert!(controls.iter().all(|c| c["standing"].as_str() == Some("ALIVE")), "every control must report ALIVE standing");
}

/// `castle fortune5 qualify` against a nonexistent evidence file exits
/// non-zero with a clean, typed error message on stderr — no Rust panic or
/// backtrace text.
#[test]
fn fortune5_qualify_with_missing_evidence_file_fails_cleanly() {
    let output = Command::new(castle_bin())
        .args([
            "fortune5",
            "qualify",
            "--subject",
            "castle:cli-fixture",
            "--evidence-path",
            "/nonexistent/fortune5_evidence.json",
        ])
        .output()
        .expect("failed to spawn castle binary");

    assert!(!output.status.success(), "expected a nonzero exit code for a missing evidence file");

    let stderr = String::from_utf8(output.stderr).expect("stderr was not valid UTF-8");
    assert!(stderr.contains("failed to read"), "stderr should contain the typed read-failure message, got: {stderr}");
    assert!(stderr.contains("/nonexistent/fortune5_evidence.json"), "stderr should name the missing path, got: {stderr}");

    let lowered = stderr.to_lowercase();
    assert!(!lowered.contains("panicked"), "stderr must not contain a Rust panic, got: {stderr}");
    assert!(!lowered.contains("backtrace"), "stderr must not contain backtrace text, got: {stderr}");
    assert!(!lowered.contains("runtimeerror"), "stderr must not contain a runtime error dump, got: {stderr}");
}
