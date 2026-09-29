//! Subprocess tests for the v26.9.29 `castle payments` rail verbs
//! (`rail-submit`, `finalize`, `replay`, `receipt`).
//!
//! Chicago-style: the real compiled binary, real Ed25519 custodian signatures, real fsynced
//! claim/nonce/ledger/BRCE/journal state and a real durable `SimRail` in temp dirs. The rail is
//! a test rail; nothing here contacts a bank.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use castle::payments::rail::RailInstruction;
use castle::payments::{ClaimState, ClaimStore, FileJournalLedger};
use castle::sa2a_security::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyState, PreparedEffect, SignatureAlgorithm,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

const AUDIENCE: &str = "actuator:payments";
const PRINCIPAL: &str = "principal:procurement-agent";
const PAYER: &str = "acct:treasury";
const PAYEE: &str = "acct:supplier-9821";
const RECEIPT_SEED_HEX: &str = "6363636363636363636363636363636363636363636363636363636363636363";
const NOW_MS: i64 = 1_000;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn fixture(name: &str) -> String {
    format!("{}/fixtures/payments/cli/{name}", env!("CARGO_MANIFEST_DIR"))
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("castle-cli-rail-{}-{n}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(castle_bin()).args(args).arg("--format").arg("json").output().expect("spawn castle")
}

fn combined(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn json_of(o: &Output) -> Value {
    assert!(o.status.success(), "non-zero exit: {}", combined(o));
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("stdout not JSON ({e}): {}", combined(o)))
}

/// Real custodians: three Ed25519 keys, threshold 2.
fn custodians() -> Vec<(&'static str, SigningKey)> {
    ["mac", "phone", "hsm"].iter().enumerate().map(|(i, n)| (*n, SigningKey::from_bytes(&[(i as u8) + 11; 32]))).collect()
}

fn write_registry(path: &Path) {
    let records: Vec<KeyRecord> = custodians()
        .iter()
        .map(|(n, k)| KeyRecord {
            key_id: format!("key-{n}"),
            custodian_id: format!("device:{n}"),
            algorithm: SignatureAlgorithm::Ed25519,
            public_key: k.verifying_key().to_bytes().to_vec(),
            state: KeyState::Active,
            not_before_ms: 0,
            expires_at_ms: 10_000_000,
            revocation_epoch: 0,
        })
        .collect();
    std::fs::write(path, serde_json::to_vec(&records).unwrap()).unwrap();
}

fn write_cert(path: &Path, effect: &PreparedEffect, nonce: &str) {
    let mut cert = ActuationCertificate {
        version: 1,
        effect_digest: effect.digest().unwrap(),
        principal: effect.principal.clone(),
        policy_epoch: 7,
        revocation_epoch: 0,
        generation: 1,
        nonce: nonce.to_string(),
        not_before_ms: 0,
        expires_at_ms: 100_000,
        audience: AUDIENCE.to_string(),
        threshold: 2,
        signatures: vec![],
    };
    let msg = cert.signing_message().unwrap();
    for (name, key) in custodians().iter().filter(|(n, _)| *n == "mac" || *n == "phone") {
        cert.signatures.push(CertificateSignature {
            key_id: format!("key-{name}"),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: key.sign(&msg).to_bytes().to_vec(),
        });
    }
    std::fs::write(path, serde_json::to_vec(&cert).unwrap()).unwrap();
}


struct World {
    dir: PathBuf,
}

/// Options for one `rail-submit` invocation.
struct Submit<'a> {
    cert_file: &'a str,
    mode: &'a str,
    reference_ledger: bool,
    reference_rail: bool,
    journal: bool,
    sanctions: Option<String>,
    counterparties: Option<String>,
}

impl<'a> Submit<'a> {
    fn new(cert_file: &'a str, mode: &'a str) -> Self {
        Self { cert_file, mode, reference_ledger: true, reference_rail: true, journal: true, sanctions: None, counterparties: None }
    }
}

impl World {
    fn new(tag: &str) -> Self {
        let dir = unique_dir(tag);
        write_registry(&dir.join("registry.json"));
        Self { dir }
    }
    fn p(&self, name: &str) -> String {
        self.dir.join(name).to_string_lossy().into_owned()
    }
    fn prepare(&self, amount: &str, obligation: &str) -> (PreparedEffect, String) {
        let out = json_of(&run(&[
            "payments", "prepare", "--principal", PRINCIPAL, "--payer", PAYER, "--payee", PAYEE, "--amount-minor", amount,
            "--currency", "USD", "--obligation-id", obligation, "--purpose", "invoice-payment",
        ]));
        let effect: PreparedEffect = serde_json::from_value(out["effect"].clone()).unwrap();
        let digest = out["effect_digest"].as_str().unwrap().to_string();
        std::fs::write(self.dir.join("effect.json"), serde_json::to_vec(&effect).unwrap()).unwrap();
        (effect, digest)
    }
    fn rail_submit(&self, s: &Submit<'_>) -> Output {
        let mut args: Vec<String> = vec![
            "payments".into(), "rail-submit".into(),
            "--effect-path".into(), self.p("effect.json"),
            "--certificate-path".into(), self.p(s.cert_file),
            "--registry-path".into(), self.p("registry.json"),
            "--policy-path".into(), fixture("policy.json"),
            "--state-dir".into(), self.p("state"),
            "--ledger-dir".into(), self.p("ledger"),
            "--opening-path".into(), fixture("opening.json"),
            "--rail-dir".into(), self.p("rail"),
            "--rail-mode".into(), s.mode.into(),
            "--audience".into(), AUDIENCE.into(),
            "--policy-epoch".into(), "7".into(),
            "--revocation-epoch".into(), "0".into(),
            "--generation".into(), "1".into(),
            "--now-ms".into(), NOW_MS.to_string(),
            "--receipt-key-id".into(), "receipt-key".into(),
            "--receipt-seed-hex".into(), RECEIPT_SEED_HEX.into(),
            "--allowed-authority".into(), AUDIENCE.into(),
            "--debtor-agent-bic".into(), "DEUTDEFF".into(),
            "--creditor-agent-bic".into(), "BOFAUS3N".into(),
            "--created-at-iso".into(), "2026-09-29T12:34:56Z".into(),
        ];
        if s.reference_ledger {
            args.push("--reference-ledger".into());
        }
        if s.reference_rail {
            args.push("--reference-rail".into());
        }
        if s.journal {
            args.extend(["--journal-dir".into(), self.p("journal")]);
        }
        if let Some(p) = &s.sanctions {
            args.extend(["--sanctions-path".into(), p.clone()]);
        }
        if let Some(p) = &s.counterparties {
            args.extend(["--counterparties-path".into(), p.clone()]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&refs)
    }
    fn finalize(&self, digest: &str, mode: Option<&str>) -> Output {
        let mut args = vec![
            "payments", "finalize", "--effect-digest", digest, "--state-dir", &self.p("state"), "--ledger-dir",
            &self.p("ledger"), "--rail-dir", &self.p("rail"), "--reference-ledger", "--reference-rail",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        if let Some(m) = mode {
            args.extend(["--rail-mode".to_string(), m.to_string()]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&refs)
    }
    fn receipt(&self, digest: &str) -> Output {
        run(&[
            "payments", "receipt", "--effect-digest", digest, "--state-dir", &self.p("state"), "--ledger-dir", &self.p("ledger"),
            "--rail-dir", &self.p("rail"), "--journal-dir", &self.p("journal"), "--reference-ledger", "--reference-rail",
        ])
    }
    fn replay(&self, digest: &str) -> Output {
        run(&["payments", "replay", "--journal-dir", &self.p("journal"), "--effect-digest", digest])
    }
    fn claim_state(&self, digest: &str) -> ClaimState {
        ClaimStore::open(Path::new(&self.p("state")).join("claims")).unwrap().get(digest).unwrap().unwrap().state
    }
    fn ledger_entries(&self) -> usize {
        FileJournalLedger::open(self.p("ledger"), &[]).unwrap().entries().unwrap().len()
    }
    /// Run `finalize` until the claim is `Final`; returns the sequence of `result` labels.
    fn finalize_to_final(&self, digest: &str) -> Vec<String> {
        let mut labels = Vec::new();
        for _ in 0..8 {
            let out = json_of(&self.finalize(digest, None));
            labels.push(out["result"].as_str().unwrap().to_string());
            if self.claim_state(digest) == ClaimState::Final {
                return labels;
            }
        }
        panic!("claim never reached Final: {labels:?}");
    }
}

fn prepared_world(tag: &str, amount: &str, obligation: &str) -> (World, String) {
    let w = World::new(tag);
    let (effect, digest) = w.prepare(amount, obligation);
    write_cert(&w.dir.join("cert.json"), &effect, "nonce-1");
    (w, digest)
}

#[test]
fn honest_flow_submit_finalize_receipt_replay_all_exit_zero() {
    let (w, digest) = prepared_world("honest", "400000", "inv-rail-1");
    let submitted = json_of(&w.rail_submit(&Submit::new("cert.json", "settle-after-polls:2")));
    assert_eq!(submitted["standing"], "Submitted");
    assert_eq!(submitted["effect_digest"], digest.as_str());
    assert_eq!(submitted["journaled"], true);
    assert_eq!(w.claim_state(&digest), ClaimState::Submitted);
    // Hold placed, nothing in the books before observed finality.
    assert_eq!(w.ledger_entries(), 0);

    let labels = w.finalize_to_final(&digest);
    assert_eq!(labels.last().map(String::as_str), Some("applied"), "{labels:?}");
    assert!(labels[..labels.len() - 1].iter().all(|l| l == "pending"), "{labels:?}");
    assert_eq!(w.ledger_entries(), 1);

    let out = json_of(&w.receipt(&digest));
    assert_eq!(out["verified"], true);
    let inputs = &out["receipt"]["inputs"];
    let correlation = RailInstruction::correlation_id_for(&digest);
    assert_eq!(inputs["effect_id"], digest.as_str());
    assert_eq!(inputs["obligation_id"], "inv-rail-1");
    assert_eq!(inputs["rail_correlation_id"], correlation.as_str());
    assert_eq!(inputs["claim_state"], "final");
    assert_eq!(inputs["pee_effect_id"], submitted["pee_effect_id"]);
    assert_eq!(inputs["rail_payload_digest"], submitted["rail_payload_digest"]);
    assert_eq!(inputs["construct_digest"], submitted["construct_digest"]);
    assert!(inputs["finality_evidence_digest"].as_str().is_some_and(|d| d.starts_with("sha256:")));
    assert!(inputs["ledger_entry_digest"].as_str().is_some_and(|d| d.starts_with("sha256:")));
    assert!(inputs["settled_at_ms"].as_u64().is_some());
    assert_eq!(inputs["brce_prepare_digests"].as_array().unwrap().len(), 2);
    assert_eq!(inputs["brce_outcome_digests"].as_array().unwrap().len(), 2);
    // explain() names rail correlation and finality from the receipt alone.
    let ex = &out["explain"];
    assert_eq!(ex["through_which_rail"]["correlation_id"], correlation.as_str());
    assert!(ex["when_final"]["finality_evidence_digest"].is_string());
    assert_eq!(ex["what_happened"]["claim_state"], "final");

    let replay = json_of(&w.replay(&digest));
    assert_eq!(replay["verdict"], "Reproduced");
    assert_eq!(replay["effect_digest"], digest.as_str());
    // The receipt's admission digest is the replay decision digest.
    assert_eq!(inputs["admission_decision_digest"], replay["decision_digest"]);
}

#[test]
fn drop_ack_is_unknown_nonzero_then_finalize_applies_exactly_one_entry() {
    let (w, digest) = prepared_world("dropack", "400000", "inv-rail-2");
    let out = w.rail_submit(&Submit::new("cert.json", "drop-ack"));
    assert!(!out.status.success(), "UnknownOutcome must exit non-zero: {}", combined(&out));
    let text = combined(&out);
    assert!(text.contains("UnknownOutcome"), "{text}");
    assert!(text.contains(&digest), "{text}");
    assert_eq!(w.claim_state(&digest), ClaimState::UnknownOutcome);
    assert_eq!(w.ledger_entries(), 0);

    // Blind re-submission of the same effect is refused; state is unchanged.
    let (effect, _) = w.prepare("400000", "inv-rail-2");
    write_cert(&w.dir.join("cert2.json"), &effect, "nonce-2");
    let again = w.rail_submit(&Submit::new("cert2.json", "drop-ack"));
    assert!(!again.status.success());
    assert!(combined(&again).contains("REFUSED:PAYMENT_OUTCOME_UNKNOWN"), "{}", combined(&again));
    assert_eq!(w.claim_state(&digest), ClaimState::UnknownOutcome);

    let labels = w.finalize_to_final(&digest);
    assert_eq!(labels.last().map(String::as_str), Some("applied"), "{labels:?}");
    assert_eq!(w.ledger_entries(), 1, "exactly one ledger entry");
    // Finalize again is idempotent: still exactly one entry.
    let _ = w.finalize(&digest, None);
    assert_eq!(w.ledger_entries(), 1);
}

#[test]
fn missing_reference_rail_refuses_before_any_state_is_written() {
    let (w, _digest) = prepared_world("noref", "400000", "inv-rail-3");
    let mut s = Submit::new("cert.json", "honest");
    s.reference_rail = false;
    let out = w.rail_submit(&s);
    assert!(!out.status.success(), "{}", combined(&out));
    assert!(combined(&out).contains("REFUSED:REFERENCE_RAIL_NOT_CONFIRMED"), "{}", combined(&out));
    assert!(!w.dir.join("state").exists(), "no claim/nonce state may be written");
    assert!(!w.dir.join("rail").exists(), "no rail may be fabricated");
    assert!(!w.dir.join("ledger").exists());
}

#[test]
fn missing_reference_ledger_and_bad_mode_refuse() {
    let (w, _digest) = prepared_world("noledger", "400000", "inv-rail-4");
    let mut s = Submit::new("cert.json", "honest");
    s.reference_ledger = false;
    let out = w.rail_submit(&s);
    assert!(!out.status.success());
    assert!(combined(&out).contains("REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED"), "{}", combined(&out));

    let bad = w.rail_submit(&Submit::new("cert.json", "teleport"));
    assert!(!bad.status.success());
    assert!(combined(&bad).contains("REFUSED:PAYMENT_RAIL_MODE_INVALID"), "{}", combined(&bad));
    assert!(!w.dir.join("state").exists());
}

#[test]
fn replay_is_reproduced_and_missing_record_refuses() {
    let (w, digest) = prepared_world("replay", "400000", "inv-rail-5");
    json_of(&w.rail_submit(&Submit::new("cert.json", "honest")));
    let replay = json_of(&w.replay(&digest));
    assert_eq!(replay["verdict"], "Reproduced");
    let unknown = w.replay("sha256:0000000000000000000000000000000000000000000000000000000000000000");
    assert!(!unknown.status.success());
    assert!(combined(&unknown).contains("REFUSED:REPLAY_RECORD_MISSING"), "{}", combined(&unknown));
}

#[test]
fn finalize_on_down_rail_is_still_unknown_and_changes_nothing() {
    let (w, digest) = prepared_world("down", "400000", "inv-rail-6");
    json_of(&w.rail_submit(&Submit::new("cert.json", "settle-after-polls:1")));
    let out = json_of(&w.finalize(&digest, Some("down")));
    assert_eq!(out["result"], "still_unknown");
    assert_eq!(w.claim_state(&digest), ClaimState::Submitted);
    assert_eq!(w.ledger_entries(), 0);
    // Rail recovers: same effect finalizes.
    let labels = {
        let mut l = vec![json_of(&w.finalize(&digest, Some("settle-after-polls:1")))["result"].as_str().unwrap().to_string()];
        while w.claim_state(&digest) != ClaimState::Final && l.len() < 6 {
            l.push(json_of(&w.finalize(&digest, None))["result"].as_str().unwrap().to_string());
        }
        l
    };
    assert_eq!(labels.last().map(String::as_str), Some("applied"), "{labels:?}");
    assert_eq!(w.ledger_entries(), 1);
}

#[test]
fn receipt_and_finalize_refuse_without_reference_rail_and_without_rail() {
    let (w, digest) = prepared_world("receiptref", "400000", "inv-rail-7");
    json_of(&w.rail_submit(&Submit::new("cert.json", "honest")));
    let no_ref = run(&[
        "payments", "receipt", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"),
        "--rail-dir", &w.p("rail"), "--journal-dir", &w.p("journal"), "--reference-ledger",
    ]);
    assert!(!no_ref.status.success());
    assert!(combined(&no_ref).contains("REFUSED:REFERENCE_RAIL_NOT_CONFIRMED"), "{}", combined(&no_ref));
    let no_rail = run(&[
        "payments", "finalize", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"),
        "--rail-dir", &w.p("absent-rail"), "--reference-ledger", "--reference-rail",
    ]);
    assert!(!no_rail.status.success());
    assert!(combined(&no_rail).contains("REFUSED:PAYMENT_RAIL_NOT_FOUND"), "{}", combined(&no_rail));
    assert!(!w.dir.join("absent-rail").exists(), "finalize must not fabricate a rail");
}

#[test]
fn rail_reject_flow_exits_nonzero_and_leaves_no_ledger_entry() {
    let (w, digest) = prepared_world("reject", "400000", "inv-rail-8");
    json_of(&w.rail_submit(&Submit::new("cert.json", "reject-after-polls:1")));
    let mut last = None;
    for _ in 0..6 {
        let out = w.finalize(&digest, None);
        if !out.status.success() {
            last = Some(out);
            break;
        }
    }
    let out = last.expect("a rail rejection must surface as a non-zero exit");
    assert!(combined(&out).contains("refused"), "{}", combined(&out));
    assert_eq!(w.claim_state(&digest), ClaimState::Refused);
    assert_eq!(w.ledger_entries(), 0);
}

#[test]
fn screened_admission_refuses_sanctions_hit_and_admits_clear_list() {
    let (w, digest) = prepared_world("screen", "400000", "inv-rail-9");
    let mut hit = Submit::new("cert.json", "honest");
    hit.journal = false;
    hit.sanctions = Some(fixture("sanctions-hit.json"));
    hit.counterparties = Some(fixture("counterparties.json"));
    let out = w.rail_submit(&hit);
    assert!(!out.status.success());
    assert!(combined(&out).contains("REFUSED:PAYMENT_SANCTIONS_HIT"), "{}", combined(&out));
    // Refused before the nonce was burned or budget reserved: the same certificate still admits.
    let mut clear = Submit::new("cert.json", "honest");
    clear.journal = false;
    clear.sanctions = Some(fixture("sanctions-clear.json"));
    clear.counterparties = Some(fixture("counterparties.json"));
    let ok = json_of(&w.rail_submit(&clear));
    assert_eq!(ok["standing"], "Submitted");
    assert_eq!(ok["screened"], true);
    assert_eq!(w.claim_state(&digest), ClaimState::Submitted);

    // Half a screening input, or screening plus journal, is refused typed.
    let mut half = Submit::new("cert.json", "honest");
    half.sanctions = Some(fixture("sanctions-clear.json"));
    half.journal = false;
    let out = w.rail_submit(&half);
    assert!(combined(&out).contains("REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE"), "{}", combined(&out));
    let mut both = Submit::new("cert.json", "honest");
    both.sanctions = Some(fixture("sanctions-clear.json"));
    both.counterparties = Some(fixture("counterparties.json"));
    let out = w.rail_submit(&both);
    assert!(combined(&out).contains("REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED"), "{}", combined(&out));
}

#[test]
fn claim_states_written_by_the_rail_flow_conform_to_claim_schema_enum() {
    let (w, digest) = prepared_world("schema", "400000", "inv-rail-10");
    json_of(&w.rail_submit(&Submit::new("cert.json", "settle-after-polls:1")));
    let schema: Value = serde_json::from_slice(
        &std::fs::read(format!("{}/schemas/payments/claim-v1.schema.json", env!("CARGO_MANIFEST_DIR"))).unwrap(),
    )
    .unwrap();
    let allowed = schema["properties"]["state"]["enum"].as_array().unwrap().clone();
    let claim_file = std::fs::read_dir(w.dir.join("state").join("claims"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .expect("claim file");
    let claim: Value = serde_json::from_slice(&std::fs::read(claim_file).unwrap()).unwrap();
    assert_eq!(claim["state"], "submitted");
    assert!(allowed.contains(&claim["state"]), "submitted missing from claim-v1 enum");
    w.finalize_to_final(&digest);
    let claim_file = std::fs::read_dir(w.dir.join("state").join("claims"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .expect("claim file");
    let claim: Value = serde_json::from_slice(&std::fs::read(claim_file).unwrap()).unwrap();
    assert!(allowed.contains(&claim["state"]), "{} missing from claim-v1 enum", claim["state"]);
    assert_eq!(claim["state"], "final");
}

/// `payments reconcile` must not prove absence from the ledger while a rail hold is live: the
/// ledger has no entry until finality, so only `finalize` (against the rail) may resolve it.
/// The library `reconcile()` (src/payments/reconcile.rs) still lacks this guard; the CLI has it.
#[test]
fn reconcile_must_not_prove_absence_while_rail_hold_is_live() {
    let (w, digest) = prepared_world("reconcile-hold", "400000", "inv-rail-11");
    let out = w.rail_submit(&Submit::new("cert.json", "drop-ack"));
    assert!(!out.status.success());
    let rec = run(&[
        "payments", "reconcile", "--effect-digest", &digest, "--state-dir", &w.p("state"), "--ledger-dir", &w.p("ledger"),
        "--reference-ledger",
    ]);
    assert!(!rec.status.success(), "reconcile must refuse while a rail hold is live: {}", combined(&rec));
    assert!(combined(&rec).contains("REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE"), "{}", combined(&rec));
    assert_eq!(w.claim_state(&digest), ClaimState::UnknownOutcome, "claim must stay UnknownOutcome");
}
