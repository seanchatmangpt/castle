//! L139 E2E: the real mfact producer output (/Users/sac/mfact/witnesses.jsonl,
//! emitted by scripts/witness_receipt.py via `lake env lean` + `#print axioms`)
//! ingests through the castle lean_witness adapter — all 5 Petri witnesses
//! admit as `Replayed`; a tampered digest diverges the subject.

use castle::lean_witness::evidence_input_from_lean_witness;
use castle::fortune5::EvidenceEpistemicClass;

const WITNESSES: &str = "/Users/sac/mfact/witnesses.jsonl";

#[test]
fn l139_all_five_produced_witnesses_admit_as_replayed() {
    let data = std::fs::read_to_string(WITNESSES)
        .expect("witnesses.jsonl exists (producer receipt)");
    let lines: Vec<&str> = data.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 5, "5 Petri witnesses");

    let mut theorems = Vec::new();
    for line in &lines {
        let evidence = evidence_input_from_lean_witness(line, "2026-10-13T00:00:00Z")
            .unwrap_or_else(|e| panic!("witness refused: {e}: {line}"));
        let input = evidence.input;
        assert_eq!(input.metric, "lean.witness.verified");
        assert_eq!(input.value, castle::fortune5::MetricValue::Bool(true));
        assert_eq!(
            input.epistemic_class,
            EvidenceEpistemicClass::Replayed,
            "admits as Replayed"
        );
        assert!(input.subject.starts_with("mfact@"));
        let theorem = line
            .split("\"theorem\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .expect("theorem field")
            .to_string();
        assert!(
            input.subject.ends_with(&format!(":{theorem}")),
            "subject theorem binding"
        );
        theorems.push(theorem);
    }
    for expected in [
        "ProcInt.PN64.pn64_sequence_conserves",
        "ProcInt.head_unique",
        "ProcInt.StrataNet.mempool_capacity",
        "ProcInt.StrataStore.recover_after_crash",
        "ProcInt.StrataLedger.strata_ledger_total_value_conserved",
    ] {
        assert!(theorems.contains(&expected.to_string()), "missing {expected}");
    }
}

#[test]
fn l139_tampered_digest_diverges_the_subject() {
    let data = std::fs::read_to_string(WITNESSES).expect("witnesses.jsonl");
    let line = data.lines().next().expect("at least one line");
    let before = evidence_input_from_lean_witness(line, "2026-10-13T00:00:00Z")
        .expect("admits")
        .input
        .subject;
    // Flip one source_sha256 byte (first hex char).
    let tampered = flip_first_hex_byte(line);
    assert_ne!(tampered, line, "tamper actually changes the line");
    let after = evidence_input_from_lean_witness(&tampered, "2026-10-13T00:00:00Z")
        .expect("structurally valid after tamper");
    assert_ne!(before, after.input.subject, "subject diverges");
}

/// Flip the first hex character of source_sha256 in a witness line.
fn flip_first_hex_byte(line: &str) -> String {
    let start = line.find("source_sha256\":\"").expect("field present")
        + "source_sha256\":\"".len();
    let mut chars: Vec<char> = line.chars().collect();
    let c = chars[start];
    let flipped = if c == '0' { '1' } else { '0' };
    chars[start] = flipped;
    chars.into_iter().collect()
}
