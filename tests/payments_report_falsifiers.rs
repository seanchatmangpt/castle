//! Operator-report falsifiers rf1..rf10. Each test asserts the NEGATION of a
//! report failure claim against real behavior: real files, real crypto, real
//! BRCE, durable SimRail, FileJournalLedger. No mocks.
//!
//! Tests marked `#[ignore = "FINDING: ..."]` are courts that FAIL on the
//! current code when run with `--ignored`; they document real defects.
mod common;
use common::payments::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use castle::payments::event_receipt::{
    explain, ledger_entry_digest, seal_event_receipt, verify_event_receipt, EventInputs,
};
use castle::payments::execute_rail::{finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit, RailSubmission};
use castle::payments::experience::requires_intelligence;
use castle::payments::iso20022::{pain001_customer_credit_transfer, project_effect_digest_from_pain001, ISO20022_PROFILE};
use castle::payments::rail::{RailAck, RailActuator, RailInstruction};
use castle::payments::replay::{admit_payment_journaled, replay_admission, AdmissionJournal, ReplayVerdict};
use castle::payments::settlement::{apply_finality, FinalityEvidence, FinalityKind};
use castle::payments::*;
use castle::sa2a_security::{CertificateSignature, PreparedEffect, SignatureAlgorithm};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::Value;

// ---------------------------------------------------------------- helpers

fn bindings(expires_at_ms: u64) -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: "law:v1".into(),
        counterparty_evidence_digest: "cp:v1".into(),
        funding_source_id: "fund:treasury".into(),
        resource_reservation_id: "res:1".into(),
        rail_profile_id: "sim-rail".into(),
        message_profile_version: "pain.001.001.09".into(),
        parent_receipt: "receipt:parent".into(),
        created_at_ms: 0,
        expires_at_ms,
    }
}

fn params() -> RailExecutionParams {
    RailExecutionParams {
        bindings: bindings(100_000),
        created_at_iso: "2026-09-29T12:34:56Z".into(),
        debtor_name: "Treasury Ltd".into(),
        creditor_name: "Supplier 9821 GmbH".into(),
        debtor_agent_bic: "DEUTDEFF".into(),
        creditor_agent_bic: "BOFAUS3N".into(),
        rail_profile: "pain.001.001.09".into(),
    }
}

fn sim(fx: &Fixture, tag: &str, mode: SimMode) -> SimRail {
    SimRail::open(&fx.dir.join(format!("rail-{tag}")), mode).expect("sim rail")
}

fn hex(digest: &str) -> &str {
    digest.trim_start_matches("sha256:")
}

fn claim_state(fx: &Fixture, digest: &str) -> ClaimState {
    fx.claims.get(digest).unwrap().unwrap().state
}

/// Journaled admission + rail submit. Returns the submission and the replayed decision digest.
async fn submit_journaled(
    fx: &Fixture,
    journal: &AdmissionJournal,
    rail: &SimRail,
    amount: &str,
    obligation: &str,
    nonce: &str,
) -> (RailSubmission, String) {
    let eff = fx.effect(amount, obligation);
    let cert = fx.cert(&eff, nonce, &["mac", "phone"]);
    let adm = admit_payment_journaled(eff, &cert, &fx.admission_ctx(), journal).expect("admits");
    let digest = adm.effect().digest().to_string();
    let decision = match replay_admission(journal, &digest).expect("replay") {
        ReplayVerdict::Reproduced { decision_digest, .. } => decision_digest,
        other => panic!("{other:?}"),
    };
    let s = submit_via_rail(adm, &params(), &fx.exec_ctx(), rail).await.expect("submit_via_rail");
    (s, decision)
}

async fn submit(fx: &Fixture, rail: &SimRail, amount: &str, obligation: &str, nonce: &str) -> RailSubmission {
    let a = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    submit_via_rail(a, &params(), &fx.exec_ctx(), rail).await.expect("submit_via_rail")
}

fn finalize(fx: &Fixture, rail: &SimRail, digest: &str) -> FinalizeResult {
    finalize_via_rail(digest, &fx.claims, &fx.ledger, rail).expect("finalize")
}

fn files_with_suffix(dir: &Path, suffix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(suffix) && !n.starts_with('.')))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn read_json(p: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
}

/// Every file under `dir`, keyed by relative path (byte-exact snapshot).
fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else if let Ok(b) = std::fs::read(&p) {
                out.insert(p.strip_prefix(base).unwrap().display().to_string(), b);
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// Number of persisted ledger entry files (raw directory, not the ledger API).
fn ledger_entry_files(fx: &Fixture) -> usize {
    files_with_suffix(&fx.dir.join("ledger"), ".entry.json").len()
}

/// The ledger shows no settlement and no balance movement (books untouched).
fn assert_books_untouched(fx: &Fixture, ctx: &str) {
    assert!(fx.ledger.entries().unwrap().is_empty(), "{ctx}: ledger entries");
    assert_eq!(ledger_entry_files(fx), 0, "{ctx}: persisted entry files");
    assert!(fx.ledger.returns().unwrap().is_empty(), "{ctx}: returns");
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000, "{ctx}: payer balance");
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0, "{ctx}: payee balance");
    assert!(fx.ledger.conserves(Currency::USD).unwrap(), "{ctx}: conservation");
}

fn between<'a>(s: &'a str, open: &str, close: &str) -> &'a str {
    let a = s.find(open).unwrap_or_else(|| panic!("missing {open}")) + open.len();
    let b = s[a..].find(close).unwrap_or_else(|| panic!("missing {close}")) + a;
    &s[a..b]
}

// ---------------------------------------------------------------- rf1

/// rf1: Authenticated(x) does NOT imply AuthorizedPayment(x).
#[test]
fn rf1_authenticated_does_not_imply_authorized_payment() {
    let fx = Fixture::new("rf1");
    let approved = fx.effect("1000", "inv-a");
    let other = fx.effect("1001", "inv-a");
    let cert_for_approved = fx.cert(&approved, "n-1", &["mac", "phone"]);

    // (a) valid registry signatures over the WRONG effect.
    assert_eq!(
        admit_payment(other.clone(), &cert_for_approved, &fx.admission_ctx()).unwrap_err(),
        "REFUSED:EffectDigestMismatch"
    );

    // (b) same valid signatures, digest field rewritten to the other effect: signatures no longer verify.
    let mut rewritten = cert_for_approved.clone();
    rewritten.effect_digest = other.digest().unwrap();
    let err = admit_payment(other.clone(), &rewritten, &fx.admission_ctx()).unwrap_err();
    assert!(err.starts_with("REFUSED:"), "{err}");
    assert_ne!(err, "", "must refuse");

    // (c) valid cert, insufficient quorum (threshold 2, one signer).
    let one = fx.cert(&approved, "n-2", &["mac"]);
    assert_eq!(admit_payment(approved.clone(), &one, &fx.admission_ctx()).unwrap_err(), "REFUSED:InsufficientQuorum");

    // (d) the same custodian twice does not make a quorum.
    let twice = fx.cert(&approved, "n-3", &["mac", "mac"]);
    assert!(admit_payment(approved.clone(), &twice, &fx.admission_ctx()).is_err());

    // Nothing was admitted, reserved, or moved by any refusal.
    assert!(fx.claims.list().unwrap().is_empty());
    assert_books_untouched(&fx, "rf1");

    // Control: the approved effect with a full quorum IS admitted (the refusals were about x, not the fixture).
    let ok = fx.cert(&approved, "n-4", &["mac", "phone"]);
    assert!(admit_payment(approved, &ok, &fx.admission_ctx()).is_ok());
}

/// rf1 (finding): the verifier takes the quorum threshold from the certificate itself.
/// One custodian can therefore sign a certificate that declares `threshold = 1`
/// and obtain a PaymentAdmission. No policy floor exists.
#[test]
#[ignore = "FINDING: CertificateVerifier honors certificate.threshold with no policy minimum; a single custodian self-declares threshold=1 and is admitted (src/sa2a_security/verifier.rs:60-63). Patch: add min_threshold to CertificateVerifier/AdmissionContext (source it from PrincipalPolicy) and refuse InsufficientQuorum when certificate.threshold < min_threshold."]
fn rf1_finding_single_custodian_self_declared_threshold_is_admitted() {
    let fx = Fixture::new("rf1-threshold");
    let eff = fx.effect("1000", "inv-a");
    let mut cert = fx.cert(&eff, "n-1", &[]);
    cert.threshold = 1;
    let msg = cert.signing_message().unwrap();
    let mac = SigningKey::from_bytes(&[11; 32]); // fixture derivation for custodian "mac"
    cert.signatures.push(CertificateSignature {
        key_id: "key-mac".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        signature: mac.sign(&msg).to_bytes().to_vec(),
    });
    let r = admit_payment(eff, &cert, &fx.admission_ctx());
    assert!(r.is_err(), "a 1-of-3 self-declared quorum must not authorize a payment");
}

// ---------------------------------------------------------------- rf2

/// rf2: a changed effect cannot reuse the authorization of the approved one.
#[test]
fn rf2_changed_effect_cannot_reuse_authorization() {
    let mut fx = Fixture::new("rf2");
    // Policy would allow the alternate beneficiary: ONLY the digest binding may stop it.
    fx.policy.principals.get_mut(PRINCIPAL).unwrap().allowed_payees.insert("acct:other-beneficiary".into());

    let approved = fx.effect("10000", "inv-1");
    let cert = fx.cert(&approved, "n-1", &["mac", "phone"]);

    let mk = |payee: &str, amount: &str, obligation: &str, purpose: &str| -> PreparedEffect {
        PaymentEffect::prepare(PRINCIPAL, PAYER, payee, amount, Currency::USD, obligation, purpose, None).unwrap()
    };
    let variants: Vec<(&str, PreparedEffect)> = vec![
        ("beneficiary", mk("acct:other-beneficiary", "10000", "inv-1", "invoice-payment")),
        ("amount+1", mk(PAYEE, "10001", "inv-1", "invoice-payment")),
        ("obligation", mk(PAYEE, "10000", "inv-2", "invoice-payment")),
        ("purpose", mk(PAYEE, "10000", "inv-1", "other-purpose")),
    ];
    for (name, v) in variants {
        assert_eq!(
            admit_payment(v, &cert, &fx.admission_ctx()).unwrap_err(),
            "REFUSED:EffectDigestMismatch",
            "{name}"
        );
    }
    assert!(fx.claims.list().unwrap().is_empty());
    assert_books_untouched(&fx, "rf2 admission");

    // The refusals burned neither the certificate nonce nor anything else: the approved effect still admits.
    let admission = admit_payment(approved, &cert, &fx.admission_ctx()).expect("approved effect admits");

    // PEE path: tamper with the sealed effect; identity refuses; a deliberate change is a DIFFERENT effect.
    let pee = PreparedEconomicEffect::seal(&admission, &bindings(100_000)).unwrap();
    pee.verify_identity().unwrap();
    let mut b = pee.clone();
    b.beneficiary_account = "acct:other-beneficiary".into();
    assert_eq!(b.verify_identity().unwrap_err(), "REFUSED:EFFECT_IDENTITY_MISMATCH");
    let mut c = pee.clone();
    c.amount_minor = "10001".into();
    assert_eq!(c.verify_identity().unwrap_err(), "REFUSED:EFFECT_IDENTITY_MISMATCH");
    let mut d = pee.clone();
    d.effect_id = "sha256:forged".into();
    assert_eq!(d.verify_identity().unwrap_err(), "REFUSED:EFFECT_IDENTITY_MISMATCH");
    b.reseal_identity().unwrap();
    b.verify_identity().unwrap();
    assert_ne!(b.effect_id, pee.effect_id, "changed beneficiary is a different effect identity");
    assert_ne!(b.idempotency_key, pee.idempotency_key);

    // Nothing reached the ledger; the only persisted claim is the approved one, still Reserved.
    assert_books_untouched(&fx, "rf2 pee");
    assert_eq!(fx.claims.list().unwrap().len(), 1);
    assert_eq!(fx.claims.list().unwrap()[0].payee, PAYEE);
    assert_eq!(fx.claims.list().unwrap()[0].state, ClaimState::Reserved);
    let rail = sim(&fx, "rf2", SimMode::Honest);
    assert!(rail.accepted_correlations().is_empty());
}

// ---------------------------------------------------------------- rf3

/// rf3: Timeout does NOT imply Retry.
#[tokio::test]
async fn rf3_timeout_is_never_a_retry() {
    // Rail accepted, then destroyed the response.
    let fx = Fixture::new("rf3");
    let rail = sim(&fx, "drop", SimMode::DropAckAfterAccept);
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s.detail);
    assert_eq!(claim_state(&fx, &s.effect_id), ClaimState::UnknownOutcome);

    for (i, nonce) in ["n-2", "n-3", "n-4"].iter().enumerate() {
        let again = fx.admit(fx.effect("400000", "inv-1"), nonce);
        assert_eq!(again.unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN", "attempt {i}");
    }
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);
    assert_eq!(claim_state(&fx, &s.effect_id), ClaimState::UnknownOutcome);
    assert!(fx.ledger.hold_of(&s.effect_id).unwrap().is_some(), "hold retained across the unknown");
    assert!(fx.ledger.entries().unwrap().is_empty());

    // Rail unreachable at submit time: also UNKNOWN, also never retried.
    let fx2 = Fixture::new("rf3-down");
    let down = sim(&fx2, "down", SimMode::Down);
    let s2 = submit(&fx2, &down, "400000", "inv-1", "n-1").await;
    assert_eq!(s2.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s2.detail);
    let again = fx2.admit(fx2.effect("400000", "inv-1"), "n-2");
    assert_eq!(again.unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");
    assert_eq!(down.submissions_seen(&s2.correlation_id), 0);
    assert!(down.accepted_correlations().is_empty());
}

// ---------------------------------------------------------------- rf4

fn find_json(dir: &Path, suffix: &str, pred: impl Fn(&Value) -> bool) -> Value {
    let hits: Vec<Value> = files_with_suffix(dir, suffix).iter().map(|p| read_json(p)).filter(|v| pred(v)).collect();
    assert_eq!(hits.len(), 1, "{} in {}: expected exactly one match, got {}", suffix, dir.display(), hits.len());
    hits.into_iter().next().unwrap()
}

struct Joined {
    claim: Value,
    entry: Value,
    pee: Value,
    receipt: Value,
    rail: Value,
}

/// Rebuild the whole join from persisted files only (no store API, no in-memory handles).
fn join_persisted(root: &Path, rail_root: &Path, obligation_id: Option<&str>, effect_id: Option<&str>) -> Joined {
    let claim = find_json(&root.join("claims"), ".claim.json", |v| match (obligation_id, effect_id) {
        (Some(o), _) => v["obligation_id"] == o,
        (None, Some(e)) => v["effect_digest"] == e,
        _ => false,
    });
    let effect = claim["effect_digest"].as_str().unwrap().to_string();
    let obligation = claim["obligation_id"].as_str().unwrap().to_string();
    let entry: Value = read_json(&root.join("ledger").join(format!("{}.entry.json", hex(&effect))));
    let receipt = find_json(&root.join("receipts"), ".receipt.json", |v| {
        v["inputs"]["effect_id"] == effect.as_str() && v["inputs"]["obligation_id"] == obligation.as_str()
    });
    let pee = find_json(&root.join("pee"), ".pee.json", |v| v["effect_id"] == receipt["inputs"]["pee_effect_id"]);
    let corr = RailInstruction::correlation_id_for(&effect);
    let rail = find_json(&rail_root.join("records"), ".json", |v| v["correlation_id"] == corr.as_str());
    Joined { claim, entry, pee, receipt, rail }
}

/// rf4: ObligationID <-> EffectID is a durable join across claim, ledger, PEE, receipt, rail.
#[tokio::test]
async fn rf4_obligation_effect_join_is_durable_and_total() {
    let fx = Fixture::new("rf4");
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let rail = sim(&fx, "rf4", SimMode::SettleAfterPolls(1));
    let (s, decision) = submit_journaled(&fx, &journal, &rail, "470000", "invoice-9821", "n-1").await;
    let d = s.effect_id.clone();
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);

    let mut entry = None;
    for _ in 0..8 {
        match finalize(&fx, &rail, &d) {
            FinalizeResult::Pending => continue,
            FinalizeResult::Applied(FinalityOutcome::Settled(e)) => {
                entry = Some(e);
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    let entry = entry.expect("settled");

    // Persist the artifacts the kernel hands back but does not itself store (PEE, event receipt).
    let claim = fx.claims.get(&d).unwrap().unwrap();
    let evidence = claim.detail.strip_prefix("final:").expect("final detail carries evidence digest").to_string();
    let inputs = EventInputs {
        effect_id: d.clone(),
        obligation_id: claim.obligation_id.clone(),
        admission_decision_digest: decision,
        pee_effect_id: Some(s.pee.effect_id.clone()),
        construct_digest: s.construct_digest.clone(),
        brce_prepare_digests: s.brce_prepare_receipt_digests.clone(),
        brce_outcome_digests: s.brce_outcome_receipt_digests.clone(),
        rail_correlation_id: Some(s.correlation_id.clone()),
        rail_payload_digest: Some(s.payload_digest.clone()),
        finality_evidence_digest: Some(evidence),
        ledger_entry_digest: Some(ledger_entry_digest(&entry)),
        claim_state: format!("{:?}", claim.state),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: Some(5_000),
    };
    let receipt = seal_event_receipt(&inputs).unwrap();
    verify_event_receipt(&receipt).unwrap();
    std::fs::create_dir_all(fx.dir.join("pee")).unwrap();
    std::fs::create_dir_all(fx.dir.join("receipts")).unwrap();
    std::fs::write(fx.dir.join("pee").join(format!("{}.pee.json", hex(&d))), serde_json::to_vec(&s.pee).unwrap()).unwrap();
    std::fs::write(fx.dir.join("receipts").join(format!("{}.receipt.json", hex(&d))), serde_json::to_vec(&receipt).unwrap()).unwrap();

    // Join twice, from each key, reading persisted files only.
    let rail_root = fx.dir.join("rail-rf4");
    let by_obl = join_persisted(&fx.dir, &rail_root, Some("invoice-9821"), None);
    let by_eff = join_persisted(&fx.dir, &rail_root, None, Some(&d));
    assert_eq!(by_obl.claim, by_eff.claim);
    assert_eq!(by_obl.entry, by_eff.entry);
    assert_eq!(by_obl.pee, by_eff.pee);
    assert_eq!(by_obl.receipt, by_eff.receipt);
    assert_eq!(by_obl.rail, by_eff.rail);

    let j = by_obl;
    assert_eq!(j.claim["state"], "final");
    assert_eq!(j.claim["obligation_id"], "invoice-9821");
    assert_eq!(j.claim["effect_digest"], d.as_str());
    assert_eq!(j.entry["effect_digest"], d.as_str());
    assert_eq!(j.entry["amount_minor"], 470_000);
    assert_eq!(j.entry["debit_account"], PAYER);
    assert_eq!(j.entry["credit_account"], PAYEE);
    assert_eq!(j.pee["obligation_id"], "invoice-9821");
    assert_eq!(j.pee["beneficiary_account"], PAYEE);
    assert_eq!(j.pee["amount_minor"], "470000");
    assert_eq!(j.receipt["inputs"]["obligation_id"], "invoice-9821");
    assert_eq!(j.receipt["inputs"]["pee_effect_id"], j.pee["effect_id"]);
    assert_eq!(j.receipt["inputs"]["rail_correlation_id"], j.rail["correlation_id"]);
    assert_eq!(j.rail["correlation_id"], RailInstruction::correlation_id_for(&d).as_str());
    assert_eq!(j.rail["payload_digest"], j.receipt["inputs"]["rail_payload_digest"]);
    assert_eq!(j.rail["amount_minor"], 470_000);
    assert_eq!(j.rail["payee"], PAYEE);
    // The receipt's ledger digest is the digest of the persisted entry.
    let persisted_entry: LedgerEntry = serde_json::from_value(j.entry.clone()).unwrap();
    assert_eq!(j.receipt["inputs"]["ledger_entry_digest"], ledger_entry_digest(&persisted_entry).as_str());
    // The persisted receipt verifies after a JSON round trip.
    let reloaded: castle::payments::event_receipt::EventReceipt = serde_json::from_value(j.receipt.clone()).unwrap();
    verify_event_receipt(&reloaded).unwrap();
    // The persisted admission record is keyed by the same effect id.
    assert!(matches!(replay_admission(&journal, &d).unwrap(), ReplayVerdict::Reproduced { .. }));
}

// ---------------------------------------------------------------- rf5

fn poll_with_checks(fx: &Fixture, rail: &SimRail, d: &str, ctx: &str) -> FinalizeResult {
    for i in 0..16 {
        let r = finalize(fx, rail, d);
        match r {
            FinalizeResult::Applied(_) | FinalizeResult::ProvenAbsent => return r,
            FinalizeResult::Pending | FinalizeResult::StillUnknown => assert_books_untouched(fx, &format!("{ctx} poll {i}")),
        }
    }
    panic!("{ctx}: never terminal");
}

/// rf5: the ledger is never settled without observed finality, at every observable step.
#[tokio::test]
async fn rf5_ledger_never_settled_before_observed_finality() {
    // Honest hold path: rail keeps saying Accepted for three polls.
    let fx = Fixture::new("rf5");
    assert_books_untouched(&fx, "before admission");
    let eff = fx.effect("400000", "inv-1");
    let cert = fx.cert(&eff, "n-1", &["mac", "phone"]);
    let adm = admit_payment(eff, &cert, &fx.admission_ctx()).unwrap();
    let d = adm.effect().digest().to_string();
    assert_books_untouched(&fx, "after admission");
    // Fabricated finality against a merely Reserved claim moves nothing.
    let early = FinalityEvidence {
        effect_digest: d.clone(),
        correlation_id: RailInstruction::correlation_id_for(&d),
        evidence_digest: "sha256:early".into(),
        kind: FinalityKind::Final,
        reason: "SETTLED".into(),
    };
    assert!(apply_finality(&fx.claims, &fx.ledger, &early).is_err());
    assert_books_untouched(&fx, "early finality against Reserved");

    let rail = sim(&fx, "honest", SimMode::SettleAfterPolls(3));
    let s = submit_via_rail(adm, &params(), &fx.exec_ctx(), &rail).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);
    assert_books_untouched(&fx, "after submit (rail Accepted)");
    assert_eq!(fx.ledger.available(PAYER, Currency::USD).unwrap(), 600_000, "funds are held, not moved");
    match poll_with_checks(&fx, &rail, &d, "honest") {
        FinalizeResult::Applied(FinalityOutcome::Settled(e)) => assert_eq!(e.amount_minor, 400_000),
        other => panic!("{other:?}"),
    }
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);

    // Dropped ack: rail says Accepted while the claim is UnknownOutcome.
    let fx2 = Fixture::new("rf5-drop");
    let rail2 = sim(&fx2, "drop", SimMode::DropAckAfterAccept);
    let s2 = submit(&fx2, &rail2, "400000", "inv-1", "n-1").await;
    assert_eq!(s2.standing, RailStandingAfterSubmit::UnknownOutcome);
    assert_books_untouched(&fx2, "after dropped ack");
    assert!(matches!(poll_with_checks(&fx2, &rail2, &s2.effect_id, "drop"), FinalizeResult::Applied(FinalityOutcome::Settled(_))));

    // Rail down: uninformative, books untouched.
    let fx3 = Fixture::new("rf5-down");
    let rail3 = sim(&fx3, "down", SimMode::Down);
    let s3 = submit(&fx3, &rail3, "400000", "inv-1", "n-1").await;
    for i in 0..3 {
        assert_eq!(finalize(&fx3, &rail3, &s3.effect_id), FinalizeResult::StillUnknown);
        assert_books_untouched(&fx3, &format!("down poll {i}"));
    }

    // Rejection: books never touched, even after the terminal report.
    let fx4 = Fixture::new("rf5-reject");
    let rail4 = sim(&fx4, "reject", SimMode::RejectAfterPolls(2));
    let s4 = submit(&fx4, &rail4, "400000", "inv-1", "n-1").await;
    assert!(matches!(poll_with_checks(&fx4, &rail4, &s4.effect_id, "reject"), FinalizeResult::Applied(FinalityOutcome::Released)));
    assert_books_untouched(&fx4, "after rejection");
}

/// rf5 (finding): `apply_finality` is public and its evidence is a plain struct, so a caller
/// can settle the ledger with fabricated "Final" evidence while the rail still says Accepted.
#[tokio::test]
#[ignore = "FINDING: apply_finality trusts caller-supplied FinalityEvidence (all-pub struct, digest unchecked): fabricated Final evidence settles the ledger while the rail reports Accepted (src/payments/settlement.rs apply_finality, FinalityEvidence). Patch: make FinalityEvidence fields private with a pub(crate) constructor from a RailStatus fetched by finalize_via_rail (or a signed rail attestation), and make apply_finality pub(crate); also check ev.correlation_id == correlation_id_for(effect_digest)."]
async fn rf5_finding_fabricated_finality_evidence_settles_ledger() {
    let fx = Fixture::new("rf5-forged");
    let rail = sim(&fx, "accepted", SimMode::SettleAfterPolls(1000));
    let s = submit(&fx, &rail, "400000", "inv-1", "n-1").await;
    assert_eq!(finalize(&fx, &rail, &s.effect_id), FinalizeResult::Pending, "rail says Accepted, not settled");
    let forged = FinalityEvidence {
        effect_digest: s.effect_id.clone(),
        correlation_id: s.correlation_id.clone(),
        evidence_digest: "sha256:made-up-by-caller".into(),
        kind: FinalityKind::Final,
        reason: "SETTLED".into(),
    };
    let r = apply_finality(&fx.claims, &fx.ledger, &forged);
    assert!(r.is_err(), "fabricated finality must not settle: {r:?}");
    assert_books_untouched(&fx, "after forged finality");
}

// ---------------------------------------------------------------- rf6

/// rf6: replay is never an external DO.
#[tokio::test]
async fn rf6_replay_never_actuates() {
    let fx = Fixture::new("rf6");
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let rail = sim(&fx, "rf6", SimMode::SettleAfterPolls(1));
    let (s, _) = submit_journaled(&fx, &journal, &rail, "250000", "inv-1", "n-1").await;
    let d = s.effect_id.clone();
    let mut settled = false;
    for _ in 0..8 {
        if let FinalizeResult::Applied(FinalityOutcome::Settled(_)) = finalize(&fx, &rail, &d) {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);

    // Admission replay: whole directory tree byte-identical before and after.
    let before = snapshot(&fx.dir);
    for _ in 0..3 {
        assert!(matches!(replay_admission(&journal, &d).unwrap(), ReplayVerdict::Reproduced { .. }));
    }
    assert_eq!(snapshot(&fx.dir), before, "replay_admission wrote nothing anywhere");
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);

    // Re-applying the same finality evidence (directly) is a no-op on claims, ledger and BRCE journal.
    let claim = fx.claims.get(&d).unwrap().unwrap();
    let ev = FinalityEvidence {
        effect_digest: d.clone(),
        correlation_id: s.correlation_id.clone(),
        evidence_digest: claim.detail.strip_prefix("final:").unwrap().to_string(),
        kind: FinalityKind::Final,
        reason: "SETTLED".into(),
    };
    let scoped = |fx: &Fixture| (snapshot(&fx.dir.join("claims")), snapshot(&fx.dir.join("ledger")), snapshot(&fx.dir.join("brce")));
    let frozen = scoped(&fx);
    for _ in 0..3 {
        assert_eq!(apply_finality(&fx.claims, &fx.ledger, &ev).unwrap(), FinalityOutcome::AlreadyFinal);
    }
    assert_eq!(scoped(&fx), frozen);

    // Re-running the finalizer against the rail: still no second submission, no second entry.
    for _ in 0..3 {
        assert_eq!(finalize(&fx, &rail, &d), FinalizeResult::Applied(FinalityOutcome::AlreadyFinal));
    }
    assert_eq!(scoped(&fx), frozen);
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
}

// ---------------------------------------------------------------- rf7

/// rf7: the rail projection cannot carry more authority than the admission.
#[tokio::test]
async fn rf7_rail_adapter_cannot_increase_authority() {
    let fx = Fixture::new("rf7");
    let rail = sim(&fx, "rf7", SimMode::SettleAfterPolls(1));
    let p = params();
    let eff = fx.effect("400000", "inv-1");
    let cert = fx.cert(&eff, "n-1", &["mac", "phone"]);
    let adm = admit_payment(eff, &cert, &fx.admission_ctx()).unwrap();
    let digest = adm.effect().digest().to_string();
    let xml = pain001_customer_credit_transfer(
        &adm,
        &p.created_at_iso,
        &p.debtor_name,
        &p.creditor_name,
        &p.debtor_agent_bic,
        &p.creditor_agent_bic,
    )
    .unwrap();

    // Parse the projection and compare every consequential field to the admission.
    let after = xml.split("<InstdAmt Ccy=\"").nth(1).expect("InstdAmt");
    assert_eq!(after.split('"').next().unwrap(), "USD");
    assert_eq!(after.split('>').nth(1).unwrap().split('<').next().unwrap(), "4000.00");
    assert_eq!(adm.effect().money().minor, 400_000);
    assert_eq!(between(&xml, "<CdtrAcct><Id><Othr><Id>", "</Id>"), PAYEE);
    assert_eq!(between(&xml, "<DbtrAcct><Id><Othr><Id>", "</Id>"), PAYER);
    assert_eq!(between(&xml, "<EndToEndId>", "</EndToEndId>"), "inv-1");
    assert_eq!(between(&xml, "<CtrlSum>", "</CtrlSum>"), "4000.00");
    assert_eq!(project_effect_digest_from_pain001(&xml).as_deref(), Some(digest.as_str()));

    let s = submit_via_rail(adm, &p, &fx.exec_ctx(), &rail).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);
    let honest = RailInstruction {
        effect_id: digest.clone(),
        correlation_id: s.correlation_id.clone(),
        message_profile: p.rail_profile.clone(),
        payload: xml.clone(),
        amount_minor: 400_000,
        currency: Currency::USD,
        payer: PAYER.into(),
        payee: PAYEE.into(),
    };
    assert_eq!(honest.payload_digest(), s.payload_digest, "the rail received exactly the projection of the admission");

    // Any instruction that differs, fed to the rail under the same correlation, is rejected.
    let mut more = honest.clone();
    more.amount_minor = 400_001;
    let mut elsewhere = honest.clone();
    elsewhere.payee = "acct:attacker".into();
    let mut other_payload = honest.clone();
    other_payload.payload = xml.replace("4000.00", "9999.00");
    for (name, i) in [("amount", more), ("payee", elsewhere), ("payload", other_payload)] {
        match rail.submit(&i).expect("rail answers") {
            RailAck::Rejected { reason_code, .. } => assert_eq!(reason_code, "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD", "{name}"),
            RailAck::Accepted { .. } => panic!("{name}: rail accepted a different instruction under the same correlation"),
        }
    }

    // The single settlement is the admitted amount, to the admitted payee.
    let mut entry = None;
    for _ in 0..8 {
        if let FinalizeResult::Applied(FinalityOutcome::Settled(e)) = finalize(&fx, &rail, &digest) {
            entry = Some(e);
            break;
        }
    }
    let e = entry.expect("settled");
    assert_eq!((e.amount_minor, e.credit_account.as_str(), e.currency), (400_000, PAYEE, Currency::USD));
    assert_eq!(rail.settlement_count(&s.correlation_id), 1);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 400_000);
    assert_eq!(fx.ledger.balance("acct:attacker", Currency::USD).unwrap(), 0);
}

// ---------------------------------------------------------------- rf8

fn text_files_under(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            text_files_under(&p, out);
        } else if let Ok(t) = std::fs::read_to_string(&p) {
            out.push((p, t));
        }
    }
}

/// rf8: ISO conformance does not rest on the historical example-ontology.
#[test]
fn rf8_iso_conformance_is_not_the_example_ontology() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    text_files_under(&root.join("src/payments"), &mut files);
    text_files_under(&root.join("fixtures/payments"), &mut files);
    assert!(files.len() > 20, "scanned only {} files", files.len());
    let offenders: Vec<String> = files
        .iter()
        .filter(|(_, t)| t.to_lowercase().contains("example.org"))
        .map(|(p, _)| p.display().to_string())
        .collect();
    assert!(offenders.is_empty(), "example.org found in {offenders:?}");

    let profile: Value = serde_json::from_str(ISO20022_PROFILE).unwrap();
    let msgs = profile["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 2);
    let mut seen = Vec::new();
    for m in msgs {
        let ns = m["xsd_target_namespace"].as_str().unwrap();
        let id = m["id"].as_str().unwrap();
        assert_eq!(ns, format!("urn:iso:std:iso:20022:tech:xsd:{id}"), "official ISO 20022 targetNamespace shape");
        seen.push(ns.to_string());
    }
    assert!(seen.contains(&"urn:iso:std:iso:20022:tech:xsd:pain.001.001.09".to_string()));
    assert!(seen.contains(&"urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08".to_string()));
    assert_eq!(profile["status"], "CONFORMANCE_CHECKED_AGAINST_XSD");

    // The emitted document declares exactly the profile's namespace.
    let fx = Fixture::new("rf8");
    let adm = fx.admit(fx.effect("1000", "inv-1"), "n-1").unwrap();
    let xml = pain001_customer_credit_transfer(&adm, "2026-09-29T12:34:56Z", "A Ltd", "B GmbH", "DEUTDEFF", "BOFAUS3N").unwrap();
    assert!(xml.contains(&format!("<Document xmlns=\"{}\">", seen[0])));
    assert!(!xml.to_lowercase().contains("example"));
    // Committed ISO fixtures carry the official namespaces too.
    for f in ["pain001_usd.xml", "pain001_jpy.xml", "pain001_kwd.xml"] {
        let t = std::fs::read_to_string(root.join("fixtures/payments/iso20022").join(f)).unwrap();
        assert!(t.contains("urn:iso:std:iso:20022:tech:xsd:pain.001.001.09"), "{f}");
    }
}

// ---------------------------------------------------------------- rf9

async fn settled_rail_payment(fx: &Fixture, journal: &AdmissionJournal, rail: &SimRail) -> (RailSubmission, LedgerEntry, castle::payments::event_receipt::EventReceipt) {
    let (s, decision) = submit_journaled(fx, journal, rail, "470000", "invoice-9821", "n-1").await;
    let mut entry = None;
    for _ in 0..8 {
        if let FinalizeResult::Applied(FinalityOutcome::Settled(e)) = finalize(fx, rail, &s.effect_id) {
            entry = Some(e);
            break;
        }
    }
    let entry = entry.expect("settled");
    let claim = fx.claims.get(&s.effect_id).unwrap().unwrap();
    let inputs = EventInputs {
        effect_id: s.effect_id.clone(),
        obligation_id: claim.obligation_id.clone(),
        admission_decision_digest: decision,
        pee_effect_id: Some(s.pee.effect_id.clone()),
        construct_digest: s.construct_digest.clone(),
        brce_prepare_digests: s.brce_prepare_receipt_digests.clone(),
        brce_outcome_digests: s.brce_outcome_receipt_digests.clone(),
        rail_correlation_id: Some(s.correlation_id.clone()),
        rail_payload_digest: Some(s.payload_digest.clone()),
        finality_evidence_digest: claim.detail.strip_prefix("final:").map(str::to_string),
        ledger_entry_digest: Some(ledger_entry_digest(&entry)),
        claim_state: format!("{:?}", claim.state),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: Some(5_000),
    };
    let receipt = seal_event_receipt(&inputs).unwrap();
    (s, entry, receipt)
}

fn present(v: &Value) -> bool {
    !v.is_null() && v != &Value::String(String::new())
}

/// rf9: the receipt DOES reconstruct subject, authority, effect, rail, observation, finality.
#[tokio::test]
async fn rf9_receipt_reconstructs_the_six_questions() {
    let fx = Fixture::new("rf9");
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let rail = sim(&fx, "rf9", SimMode::SettleAfterPolls(1));
    let (s, entry, receipt) = settled_rail_payment(&fx, &journal, &rail).await;
    verify_event_receipt(&receipt).unwrap();
    let e = explain(&receipt);

    // subject: which obligation/effect
    assert_eq!(e["why"]["obligation_id"], "invoice-9821");
    assert_eq!(e["why"]["effect_id"], s.effect_id.as_str());
    // authority: admission decision + CONSTRUCT
    assert!(present(&e["under_whose_authority"]["admission_decision_digest"]));
    assert_eq!(e["under_whose_authority"]["construct_digest"], s.construct_digest.clone().unwrap().as_str());
    // effect: what happened to the books
    assert_eq!(e["what_happened"]["ledger_entry_digest"], ledger_entry_digest(&entry).as_str());
    assert_eq!(e["what_happened"]["claim_state"], "Final");
    // rail
    assert_eq!(e["through_which_rail"]["correlation_id"], s.correlation_id.as_str());
    assert_eq!(e["through_which_rail"]["payload_digest"], s.payload_digest.as_str());
    // observation: the finality evidence digest
    assert!(present(&e["when_final"]["finality_evidence_digest"]), "{e}");
    assert!(e["when_final"]["finality_evidence_digest"].as_str().unwrap().starts_with("sha256:"));
    // finality: the time
    assert_eq!(e["when_final"]["settled_at_ms"], 5_000);

    // whose money / to whom: null WITH reason, and the named source resolves them by effect_id.
    for k in ["whose_money", "to_whom"] {
        assert!(e[k]["value"].is_null(), "{k}");
        assert!(e[k]["reason"].as_str().is_some_and(|r| r.contains("effect_id")), "{k}");
    }
    let claim = fx.claims.get(&s.effect_id).unwrap().unwrap();
    assert_eq!((claim.payer.as_str(), claim.payee.as_str()), (PAYER, PAYEE));

    // A payment without a rail leg: rail + finality are null with a reason.
    let fx2 = Fixture::new("rf9-internal");
    let j2 = AdmissionJournal::open(fx2.dir.join("journal")).unwrap();
    let eff = fx2.effect("1000", "inv-int");
    let cert = fx2.cert(&eff, "n-1", &["mac", "phone"]);
    let adm = admit_payment_journaled(eff, &cert, &fx2.admission_ctx(), &j2).unwrap();
    let digest = adm.effect().digest().to_string();
    let decision = match replay_admission(&j2, &digest).unwrap() {
        ReplayVerdict::Reproduced { decision_digest, .. } => decision_digest,
        o => panic!("{o:?}"),
    };
    let exec = execute_payment(adm, &fx2.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled);
    let entry2 = exec.ledger_entry.clone().unwrap();
    let r2 = seal_event_receipt(&EventInputs {
        effect_id: digest,
        obligation_id: "inv-int".into(),
        admission_decision_digest: decision,
        pee_effect_id: None,
        construct_digest: exec.construct_digest.clone(),
        brce_prepare_digests: exec.brce_prepare_receipt_digests.clone(),
        brce_outcome_digests: exec.brce_outcome_receipt_digests.clone(),
        rail_correlation_id: None,
        rail_payload_digest: None,
        finality_evidence_digest: None,
        ledger_entry_digest: Some(ledger_entry_digest(&entry2)),
        claim_state: "Executed".into(),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: None,
    })
    .unwrap();
    let e2 = explain(&r2);
    for k in ["through_which_rail", "when_final"] {
        assert!(e2[k]["value"].is_null(), "{k}");
        assert!(!e2[k]["reason"].as_str().unwrap_or("").is_empty(), "{k} null must carry a reason");
    }
    assert!(present(&e2["under_whose_authority"]["admission_decision_digest"]));
    assert!(present(&e2["what_happened"]["ledger_entry_digest"]));
}

/// rf9 (finding): the receipt binds the PEE identity, but `explain` cannot answer
/// "which sealed economic effect was this?" because it omits `pee_effect_id`.
#[tokio::test]
#[ignore = "FINDING: explain() drops EventInputs.pee_effect_id (the sealed PreparedEconomicEffect identity), so the effect binding is not reconstructable from explain() alone (src/payments/event_receipt.rs explain). Patch: add \"effect\": {\"pee_effect_id\": i.pee_effect_id} (null-with-reason when absent) to the explain() JSON."]
async fn rf9_finding_explain_omits_the_sealed_effect_identity() {
    let fx = Fixture::new("rf9-pee");
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let rail = sim(&fx, "rf9pee", SimMode::SettleAfterPolls(1));
    let (s, _entry, receipt) = settled_rail_payment(&fx, &journal, &rail).await;
    let text = explain(&receipt).to_string();
    assert!(text.contains(&s.pee.effect_id), "explain() must expose pee_effect_id");
}

// ---------------------------------------------------------------- rf10

/// rf10: a KNOWN payment class never needs intelligence, including on the full rail path.
#[tokio::test]
async fn rf10_known_class_completes_full_rail_path_without_intelligence() {
    let fx = Fixture::new("rf10");
    let store = ExperienceStore::open(fx.dir.join("experience")).unwrap();
    let eff_of = |amount: &str, obl: &str| PaymentEffect::from_prepared(fx.effect(amount, obl)).unwrap();

    // First occurrence: UNKNOWN. Settled through the internal-ledger DO path, which is what teaches.
    let first = eff_of("1000", "inv-a");
    let k1 = store.classify(&first).unwrap();
    assert_eq!(k1, Knowledge::Unknown);
    assert!(requires_intelligence(&k1));
    let adm = fx.admit(fx.effect("1000", "inv-a"), "n-1").unwrap();
    let exec = execute_payment(adm, &fx.exec_ctx()).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::Settled, "{}", exec.detail);
    store.record_settled(&first, &exec).unwrap();

    // Second occurrence of the settled class: KNOWN.
    let second = eff_of("800", "inv-b");
    let k2 = store.classify(&second).unwrap();
    assert!(matches!(k2, Knowledge::Known(_)));
    assert!(!requires_intelligence(&k2));
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);

    // Full rail path for the KNOWN class: admit -> PEE -> hold -> submit -> poll -> observed finality.
    let rail = sim(&fx, "rf10", SimMode::SettleAfterPolls(1));
    let s = submit(&fx, &rail, "800", "inv-b", "n-2").await;
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);
    assert!(s.pee.verify_identity().is_ok());
    let mut entry = None;
    for _ in 0..8 {
        if let FinalizeResult::Applied(FinalityOutcome::Settled(e)) = finalize(&fx, &rail, &s.effect_id) {
            entry = Some(e);
            break;
        }
    }
    let entry = entry.expect("rail path settles with no intelligence step");
    assert_eq!(entry.amount_minor, 800);
    assert_eq!(claim_state(&fx, &s.effect_id), ClaimState::Final);
    assert_eq!(fx.ledger.entries().unwrap().len(), 2);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(rail.submissions_seen(&s.correlation_id), 1);

    // Classification is unchanged by the rail run and still needs no intelligence.
    assert!(!requires_intelligence(&store.classify(&eff_of("799", "inv-c")).unwrap()));
    // A larger amount than ever settled in the class is UNKNOWN again (KNOWN is scoped, not blanket).
    assert!(requires_intelligence(&store.classify(&eff_of("1001", "inv-d")).unwrap()));

    // The rail/admission/settlement path contains no model call sites.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/payments");
    for f in ["experience.rs", "execute_rail.rs", "admission.rs", "settlement.rs", "rail_sim.rs", "pee.rs"] {
        let t = std::fs::read_to_string(root.join(f)).unwrap().to_lowercase();
        for banned in ["anthropic", "openai", "llm(", "model(", "claude"] {
            assert!(!t.contains(banned), "{f} contains {banned}");
        }
    }
}
