//! Chicago courts for the `evidence negation-ingest` / `evidence
//! negation-status` CLI verbs: real subprocess invocations against the real
//! durable ledger file. Sabotage twins assert corrupted input is caught with
//! typed refusals, never a silent pass.

use std::fs;
use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

struct Scratch(String);

impl Scratch {
    fn new(tag: &str) -> Self {
        Self(format!(
            "{}/castle-negation-{}-{}.json",
            std::env::temp_dir().to_string_lossy(),
            tag,
            std::process::id()
        ))
    }

    fn path(&self) -> &str {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn run(args: &[&str]) -> (bool, serde_json::Value, String) {
    let output = Command::new(castle_bin()).args(args).output().expect("spawn castle");
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let parsed = if stdout.trim().is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_str(&stdout)
            .unwrap_or_else(|error| panic!("stdout not JSON ({error}): {stdout}\nstderr:{stderr}"))
    };
    (output.status.success(), parsed, stderr)
}

const SORRY_AX_LINE: &str = r#"{"schema":"lean-witness/v1","theorem":"chatman_equation_typed","axioms":["propext","sorryAx"],"source_sha256":"a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8a1b2c3d4e5f6a7b8","toolchain":"leanprover/lean4:v4.21.0"}"#;

#[test]
fn negation_ingests_durable_and_queries_back_through_the_real_cli() {
    let ledger = Scratch::new("happy");
    let path = ledger.path();

    let (ok, ingested, err) = run(&[
        "evidence",
        "negation-ingest",
        "--ledger-path",
        path,
        "--subject",
        "mfact@aaa:theorem_x",
        "--predicate",
        "lean.witness.verified",
        "--searched-by",
        "castle-lane",
    ]);
    assert!(ok, "ingest failed: {err}");
    assert_eq!(ingested["admitted"], true);
    assert_eq!(ingested["status"]["type"], "absent");
    assert_eq!(ingested["status"]["standing"], "UNKNOWN");
    // RFC 3339 now by default (flag omitted).
    let searched_at = ingested["searched_at"].as_str().expect("searched_at");
    assert_eq!(searched_at.len(), 20);
    assert!(searched_at.ends_with('Z'));

    // Durable: a second process sees the record.
    let (ok, status, err) = run(&[
        "evidence",
        "negation-status",
        "--ledger-path",
        path,
        "--subject",
        "mfact@aaa:theorem_x",
        "--predicate",
        "lean.witness.verified",
    ]);
    assert!(ok, "status failed: {err}");
    assert_eq!(status["status"]["type"], "absent");
    assert_eq!(status["status"]["standing"], "UNKNOWN");

    // Full-ledger summary verb.
    let (ok, summary, err) = run(&["evidence", "negation-status", "--ledger-path", path]);
    assert!(ok, "summary failed: {err}");
    assert_eq!(summary["record_count"], 1);
}

#[test]
fn incomplete_args_are_typed_refused_not_silent() {
    let ledger = Scratch::new("incomplete");

    // Neither explicit record nor witness-refusal pair.
    let (ok, _, err) = run(&[
        "evidence",
        "negation-ingest",
        "--ledger-path",
        ledger.path(),
        "--searched-by",
        "castle-lane",
    ]);
    assert!(!ok, "incomplete ingest unexpectedly admitted");
    assert!(err.contains("REFUSED:NEGATION_INCOMPLETE"), "stderr: {err}");
    // No ledger file fabricated by a refused ingest of an empty record set.
    assert!(
        !std::path::Path::new(ledger.path()).exists()
            || fs::read_to_string(ledger.path())
                .map(|bytes| bytes.contains("\"records\":[]"))
                .unwrap_or(false),
        "refused ingest must not admit a record"
    );

    // Half a key is also refused on the status verb.
    let (ok, _, err) = run(&[
        "evidence",
        "negation-status",
        "--ledger-path",
        ledger.path(),
        "--subject",
        "mfact@aaa:theorem_x",
    ]);
    assert!(!ok, "half-key status unexpectedly answered");
    assert!(err.contains("REFUSED:NEGATION_INCOMPLETE"), "stderr: {err}");
}

#[test]
fn witness_refusal_lands_in_the_ledger_as_negation() {
    let ledger = Scratch::new("witness-refusal");
    let path = ledger.path();

    let (ok, ingested, err) = run(&[
        "evidence",
        "negation-ingest",
        "--ledger-path",
        path,
        "--searched-by",
        "castle-lane",
        "--witness-json",
        SORRY_AX_LINE,
        "--refusal-reason",
        "REFUSED:LEAN_AXIOMS_NONSTANDARD",
    ]);
    assert!(ok, "witness-refusal ingest failed: {err}");
    assert_eq!(ingested["subject"], "chatman_equation_typed");
    assert_eq!(ingested["predicate"], "REFUSED:LEAN_AXIOMS_NONSTANDARD");
    assert_eq!(ingested["status"]["type"], "absent");

    let (ok, status, _) = run(&[
        "evidence",
        "negation-status",
        "--ledger-path",
        path,
        "--subject",
        "chatman_equation_typed",
        "--predicate",
        "REFUSED:LEAN_AXIOMS_NONSTANDARD",
    ]);
    assert!(ok);
    assert_eq!(status["status"]["type"], "absent");
}

#[test]
fn non_refusal_reason_is_typed_refused() {
    let ledger = Scratch::new("not-a-refusal");
    let (ok, _, err) = run(&[
        "evidence",
        "negation-ingest",
        "--ledger-path",
        ledger.path(),
        "--searched-by",
        "castle-lane",
        "--witness-json",
        SORRY_AX_LINE,
        "--refusal-reason",
        "ALIVE:LOOKED_EVERYWHERE",
    ]);
    assert!(!ok, "a non-REFUSED reason unexpectedly admitted");
    assert!(err.contains("REFUSED:NOT_A_TYPED_REFUSAL"), "stderr: {err}");
}

// Sabotage twin: corrupt the durable ledger bytes behind the verb's back; the
// verb must refuse with the typed malformed-ledger identity, never silently
// rebuild an empty ledger (which would let absence-of-record read as never-looked).
#[test]
fn sabotage_tampered_ledger_file_is_typed_refused() {
    let ledger = Scratch::new("sabotage");
    fs::write(ledger.path(), "{\"records\":[{\"record\":{").unwrap();

    let (ok, _, err) = run(&[
        "evidence",
        "negation-status",
        "--ledger-path",
        ledger.path(),
    ]);
    assert!(!ok, "tampered ledger unexpectedly loaded");
    assert!(
        err.contains("REFUSED:NEGATION_LEDGER_MALFORMED"),
        "stderr: {err}"
    );

    let (ok, _, err) = run(&[
        "evidence",
        "negation-ingest",
        "--ledger-path",
        ledger.path(),
        "--subject",
        "s",
        "--predicate",
        "p",
        "--searched-by",
        "lane",
    ]);
    assert!(!ok, "tampered ledger unexpectedly accepted an ingest");
    assert!(err.contains("REFUSED:NEGATION_LEDGER_MALFORMED"), "stderr: {err}");
}

// Sabotage twin 2: a missing ledger is UNKNOWN-shaped, never an error and
// never a fabricated ALIVE.
#[test]
fn missing_ledger_reports_never_looked() {
    let ledger = Scratch::new("missing");
    let (ok, status, err) = run(&[
        "evidence",
        "negation-status",
        "--ledger-path",
        ledger.path(),
        "--subject",
        "mfact@aaa:theorem_x",
        "--predicate",
        "lean.witness.verified",
    ]);
    assert!(ok, "missing ledger must not error: {err}");
    assert_eq!(status["status"]["type"], "never_looked");
}
