//! Qualification courts, end to end, real collaborators only (real files, crypto,
//! durable BRCE, durable hostile SimRail, FileJournalLedger, AdmissionJournal).
//!
//! COURT A  hostile rail: Obligation -> Intent -> Admission -> PreparedEffect -> DurableClaim
//!          -> DO -> UNKNOWN -> Reconciliation -> Settlement -> Ledger -> Receipt -> Replay,
//!          asserting persisted state at every arrow and that no second effect is ever made.
//! COURT B  one-bit consequence change: approved beneficiary A, candidate beneficiary B.
//! COURT C  restart durability: every store dropped and re-opened between UNKNOWN and
//!          reconciliation still converges to exactly one settlement.
//!
//! `#[ignore = "FINDING: ..."]` tests fail when run with `--ignored`; they are real defects.
mod common;
use common::payments::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use castle::payments::event_receipt::{
    explain, ledger_entry_digest, seal_event_receipt, verify_event_receipt, EventInputs, EventReceipt,
};
use castle::payments::execute_rail::{finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit};
use castle::payments::iso20022::pain001_customer_credit_transfer;
use castle::payments::obligation::{derive_obligation_id, prepare_for_invoice};
use castle::payments::rail::{RailAck, RailActuator, RailInstruction};
use castle::payments::reconcile::recover_journal;
use castle::payments::replay::{admit_payment_journaled, replay_admission, AdmissionJournal, ReplayVerdict};
use castle::payments::settlement::{apply_finality, FinalityEvidence, FinalityKind};
use castle::payments::*;
use serde_json::Value;

static B3: RealBlake3 = RealBlake3;

// ---------------------------------------------------------------- helpers

fn bindings() -> EffectBindings {
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
        expires_at_ms: 100_000,
    }
}

fn params() -> RailExecutionParams {
    RailExecutionParams {
        bindings: bindings(),
        created_at_iso: "2026-09-29T12:34:56Z".into(),
        debtor_name: "Treasury Ltd".into(),
        creditor_name: "Supplier 9821 GmbH".into(),
        debtor_agent_bic: "DEUTDEFF".into(),
        creditor_agent_bic: "BOFAUS3N".into(),
        rail_profile: "pain.001.001.09".into(),
    }
}

fn hex(digest: &str) -> &str {
    digest.trim_start_matches("sha256:")
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

/// Raw persisted claim (read from the file, not through any store handle).
fn raw_claim(fx: &Fixture, digest: &str) -> Value {
    read_json(&fx.dir.join("claims").join(format!("{}.claim.json", hex(digest))))
}

fn raw_entry_files(fx: &Fixture) -> Vec<PathBuf> {
    files_with_suffix(&fx.dir.join("ledger"), ".entry.json")
}

fn raw_hold_exists(fx: &Fixture, digest: &str) -> bool {
    fx.dir.join("ledger").join(format!("{}.hold.json", hex(digest))).exists()
}

fn raw_release_marker_exists(fx: &Fixture, digest: &str) -> bool {
    fx.dir.join("ledger").join(format!("{}.hold.released.json", hex(digest))).exists()
}

/// The rail's own persisted record for a correlation id.
fn raw_rail_record(fx: &Fixture, correlation: &str) -> Option<Value> {
    files_with_suffix(&fx.dir.join("rail").join("records"), ".json")
        .iter()
        .map(|p| read_json(p))
        .find(|v| v["correlation_id"] == correlation)
}

fn raw_balance_untouched(fx: &Fixture) {
    let opening = read_json(&fx.dir.join("ledger").join("opening.json"));
    assert!(opening.is_array());
    assert!(raw_entry_files(fx).is_empty());
}

/// Every durable store, freshly opened from disk. Dropping a `World` and calling
/// `World::open` again is a process restart of the whole payment stack.
struct World {
    claims: ClaimStore,
    ledger: FileJournalLedger,
    nonces: DurableNonceFence,
    journal: AdmissionJournal,
    rail: SimRail,
}

impl World {
    fn open(fx: &Fixture, mode: SimMode) -> Self {
        Self::open_with(fx, mode, 1_000_000)
    }

    fn open_with(fx: &Fixture, mode: SimMode, treasury: u64) -> Self {
        Self {
            claims: ClaimStore::open(fx.dir.join("claims")).unwrap(),
            ledger: FileJournalLedger::open(fx.dir.join("ledger"), &[(PAYER, Currency::USD, treasury), (PAYEE, Currency::USD, 0)]).unwrap(),
            nonces: DurableNonceFence::open(fx.dir.join("nonces")).unwrap(),
            journal: AdmissionJournal::open(fx.dir.join("journal")).unwrap(),
            rail: SimRail::open(&fx.dir.join("rail"), mode).unwrap(),
        }
    }

    fn exec_ctx<'a>(&'a self, fx: &'a Fixture) -> ExecutionContext<'a> {
        ExecutionContext {
            blake3: &B3,
            signer: &fx.signer,
            verifier: &fx.verifier,
            allowed_authorities: BTreeSet::from([AUDIENCE.to_string()]),
            journal_root: fx.dir.join("brce"),
            now_epoch_ms: 1_000,
            ledger: &self.ledger,
            claims: &self.claims,
            policy: &fx.policy,
        }
    }

    fn admission_ctx<'a>(&'a self, fx: &'a Fixture) -> AdmissionContext<'a> {
        AdmissionContext {
            registry: &fx.registry,
            epochs: fx.epochs,
            audience: AUDIENCE,
            now_ms: NOW_MS,
            policy: &fx.policy,
            claims: &self.claims,
            nonces: &self.nonces,
        }
    }

    fn admit(&self, fx: &Fixture, effect: castle::sa2a_security::PreparedEffect, nonce: &str) -> Result<PaymentAdmission, String> {
        let cert = fx.cert(&effect, nonce, &["mac", "phone"]);
        admit_payment_journaled(effect, &cert, &self.admission_ctx(fx), &self.journal)
    }

    fn finalize(&self, d: &str) -> FinalizeResult {
        finalize_via_rail(d, &self.claims, &self.ledger, &self.rail).expect("finalize")
    }

    /// Poll until a terminal (non-Pending) result.
    fn settle(&self, d: &str) -> FinalizeResult {
        for _ in 0..16 {
            let r = self.finalize(d);
            if r != FinalizeResult::Pending {
                return r;
            }
        }
        panic!("rail never left Pending");
    }
}

fn decision_of(w: &World, d: &str) -> String {
    match replay_admission(&w.journal, d).expect("replay record") {
        ReplayVerdict::Reproduced { decision_digest, .. } => decision_digest,
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------- COURT A

#[tokio::test]
async fn court_a_hostile_rail_obligation_to_replay_with_exactly_one_effect() {
    let fx = Fixture::new("court-a");
    let w = World::open(&fx, SimMode::DropAckAfterAccept);

    // 1. OBLIGATION + 2. INTENT: an inert candidate. The obligation id is bounded to Max35Text so it
    //    survives the pain.001 EndToEndId projection (see the derived-obligation finding below).
    let obligation = "INV-2026-0042".to_string();
    let prepared = fx.effect("470000", &obligation);
    let intent = PaymentEffect::from_prepared(prepared.clone()).unwrap();
    assert_eq!(intent.obligation_id(), obligation);
    let digest = intent.digest().to_string();
    let correlation = RailInstruction::correlation_id_for(&digest);
    assert!(files_with_suffix(&fx.dir.join("claims"), ".claim.json").is_empty(), "intent wrote no claim");
    assert!(files_with_suffix(&fx.dir.join("nonces"), ".nonce").is_empty(), "intent burned no nonce");
    raw_balance_untouched(&fx);
    assert!(raw_rail_record(&fx, &correlation).is_none(), "rail has never heard of the intent");

    // 3. ADMISSION: SA2A cert + policy + nonce + budget, journaled for replay.
    let cert = fx.cert(&prepared, "n-1", &["mac", "phone"]);
    let admission = admit_payment_journaled(prepared, &cert, &w.admission_ctx(&fx), &w.journal).expect("admits");
    assert_eq!(admission.effect().digest(), digest);
    assert_eq!(files_with_suffix(&fx.dir.join("nonces"), ".nonce").len(), 1, "nonce durably burned");
    let journal_file = fx.dir.join("journal").join(format!("{}.admission.json", hex(&digest)));
    assert!(journal_file.exists(), "admission snapshot persisted");
    let decision = decision_of(&w, &digest);

    // 4. PREPARED EFFECT (PEE): sealed, self-identifying, deterministic.
    let pee_pre = PreparedEconomicEffect::seal(&admission, &bindings()).unwrap();
    pee_pre.verify_identity().unwrap();
    assert_eq!(pee_pre.obligation_id, obligation);
    assert_eq!(pee_pre.amount_minor, "470000");
    assert_eq!(pee_pre.beneficiary_account, PAYEE);
    std::fs::create_dir_all(fx.dir.join("pee")).unwrap();
    std::fs::write(fx.dir.join("pee").join(format!("{}.pee.json", hex(&digest))), serde_json::to_vec(&pee_pre).unwrap()).unwrap();

    // 5. DURABLE CLAIM: Reserved on disk BEFORE any DO.
    let c = raw_claim(&fx, &digest);
    assert_eq!(c["state"], "reserved");
    assert_eq!(c["effect_digest"], digest.as_str());
    assert_eq!(c["obligation_id"], obligation.as_str());
    assert_eq!((c["payer"].as_str(), c["payee"].as_str(), c["amount_minor"].as_u64()), (Some(PAYER), Some(PAYEE), Some(470_000)));
    raw_balance_untouched(&fx);
    assert!(!raw_hold_exists(&fx, &digest), "no hold before DO");
    assert!(raw_rail_record(&fx, &correlation).is_none());

    // 6. DO through durable BRCE. The rail ACCEPTS then destroys the response.
    let s = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await.expect("submit_via_rail");
    assert_eq!(s.effect_id, digest);
    assert_eq!(s.correlation_id, correlation);

    // 7. UNKNOWN: persisted, hold retained, books untouched, rail holds exactly one accepted instruction.
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s.detail);
    assert_eq!(s.pee, pee_pre, "DO ran under the PEE that was sealed pre-DO");
    let c = raw_claim(&fx, &digest);
    assert_eq!(c["state"], "unknown_outcome");
    assert_eq!(c["construct_digest"], s.construct_digest.clone().unwrap().as_str());
    assert!(raw_hold_exists(&fx, &digest), "hold retained across UNKNOWN");
    assert!(!raw_release_marker_exists(&fx, &digest));
    raw_balance_untouched(&fx);
    let rec = raw_rail_record(&fx, &correlation).expect("rail persisted the accept");
    assert_eq!(rec["submissions"], 1);
    assert_eq!(rec["settlement_count"], 0);
    assert_eq!(rec["payload_digest"], s.payload_digest.as_str());
    let prepares = files_with_suffix(&fx.dir.join("brce").join("rail"), ".prepare.json");
    assert_eq!(prepares.len(), 2, "BRCE PREPARE durable for hold and rail_submit");
    let recovery = recover_journal(&fx.dir.join("brce").join("rail")).unwrap();
    assert_eq!(recovery.complete.len() + recovery.prepared_without_outcome.len(), 2);

    // The system cannot manufacture a second effect out of the UNKNOWN.
    for nonce in ["n-2", "n-3"] {
        let retry = fx.effect("470000", "INV-2026-0042");
        assert_eq!(w.admit(&fx, retry, nonce).unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");
    }
    assert_eq!(raw_rail_record(&fx, &correlation).unwrap()["submissions"], 1);
    assert_eq!(raw_claim(&fx, &digest)["state"], "unknown_outcome");

    // 8. RECONCILIATION by the immutable correlation id against the authoritative rail.
    assert_eq!(correlation, RailInstruction::correlation_id_for(&digest));
    assert_eq!(w.finalize(&digest), FinalizeResult::Pending, "rail Accepted observed: UNKNOWN resolves to Submitted");
    assert_eq!(raw_claim(&fx, &digest)["state"], "submitted");
    raw_balance_untouched(&fx);
    assert!(raw_hold_exists(&fx, &digest));

    let FinalizeResult::Applied(FinalityOutcome::Settled(entry)) = w.settle(&digest) else {
        panic!("rail must resolve to observed finality");
    };

    // 9. SETTLEMENT: claim Final, bound to the evidence digest.
    let c = raw_claim(&fx, &digest);
    assert_eq!(c["state"], "final");
    let evidence = c["detail"].as_str().unwrap().strip_prefix("final:").expect("final:<evidence>").to_string();
    assert!(evidence.starts_with("sha256:"));

    // 10. LEDGER: exactly one entry, hold consumed, books moved once.
    let files = raw_entry_files(&fx);
    assert_eq!(files.len(), 1);
    let persisted: LedgerEntry = serde_json::from_value(read_json(&files[0])).unwrap();
    assert_eq!(persisted, entry);
    assert_eq!(persisted.effect_digest, digest);
    assert_eq!((persisted.debit_account.as_str(), persisted.credit_account.as_str(), persisted.amount_minor), (PAYER, PAYEE, 470_000));
    assert!(!raw_hold_exists(&fx, &digest), "the one hold was consumed by settlement");
    assert!(!raw_release_marker_exists(&fx, &digest), "and not released");
    assert!(w.ledger.hold_of(&digest).unwrap().is_none());
    assert_eq!(w.ledger.balance(PAYER, Currency::USD).unwrap(), 530_000);
    assert_eq!(w.ledger.balance(PAYEE, Currency::USD).unwrap(), 470_000);
    assert!(w.ledger.conserves(Currency::USD).unwrap());

    // 11. RECEIPT: sealed over persisted facts, verifies, tamper-evident.
    let inputs = EventInputs {
        effect_id: digest.clone(),
        obligation_id: obligation.clone(),
        admission_decision_digest: decision,
        pee_effect_id: Some(s.pee.effect_id.clone()),
        construct_digest: s.construct_digest.clone(),
        brce_prepare_digests: s.brce_prepare_receipt_digests.clone(),
        brce_outcome_digests: s.brce_outcome_receipt_digests.clone(),
        rail_correlation_id: Some(correlation.clone()),
        rail_payload_digest: Some(s.payload_digest.clone()),
        finality_evidence_digest: Some(evidence.clone()),
        ledger_entry_digest: Some(ledger_entry_digest(&persisted)),
        claim_state: "Final".into(),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: Some(5_000),
    };
    let receipt = seal_event_receipt(&inputs).unwrap();
    verify_event_receipt(&receipt).unwrap();
    std::fs::create_dir_all(fx.dir.join("receipts")).unwrap();
    let rpath = fx.dir.join("receipts").join(format!("{}.receipt.json", hex(&digest)));
    std::fs::write(&rpath, serde_json::to_vec(&receipt).unwrap()).unwrap();
    let reloaded: EventReceipt = serde_json::from_slice(&std::fs::read(&rpath).unwrap()).unwrap();
    verify_event_receipt(&reloaded).unwrap();
    let mut forged = reloaded.clone();
    forged.inputs.settled_at_ms = Some(1);
    assert!(verify_event_receipt(&forged).is_err());
    let e = explain(&reloaded);
    assert_eq!(e["why"]["obligation_id"], obligation.as_str());
    assert_eq!(e["through_which_rail"]["correlation_id"], correlation.as_str());

    // 12. REPLAY: reproduces the admission decision and writes nothing anywhere.
    let before = snapshot(&fx.dir);
    match replay_admission(&w.journal, &digest).unwrap() {
        ReplayVerdict::Reproduced { effect_digest, decision_digest } => {
            assert_eq!(effect_digest, digest);
            assert_eq!(decision_digest, inputs.admission_decision_digest);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(snapshot(&fx.dir), before);

    // NEVER a second effect.
    for _ in 0..3 {
        assert_eq!(w.finalize(&digest), FinalizeResult::Applied(FinalityOutcome::AlreadyFinal));
    }
    assert_eq!(w.rail.settlement_count(&correlation), 1);
    assert_eq!(w.rail.submissions_seen(&correlation), 1);
    assert_eq!(w.ledger.entries().unwrap().len(), 1);
    assert_eq!(raw_entry_files(&fx).len(), 1);
    assert_eq!(files_with_suffix(&fx.dir.join("claims"), ".claim.json").len(), 1);
    assert_eq!(w.rail.accepted_correlations(), vec![correlation.clone()]);
    assert_eq!(raw_rail_record(&fx, &correlation).unwrap()["settlement_count"], 1);
}

// ---------------------------------------------------------------- COURT B

/// Approved: beneficiary A / 10_000_000 USD. Candidate: beneficiary B, same amount.
#[tokio::test]
async fn court_b_one_bit_consequence_change_is_refused_on_both_paths() {
    let mut fx = Fixture::with("court-b", 20_000_000, 10_000_000, 20_000_000);
    let beneficiary_b = "acct:supplier-9822"; // one character away from A = acct:supplier-9821
    assert_eq!(beneficiary_b.len(), PAYEE.len());
    fx.policy.principals.get_mut(PRINCIPAL).unwrap().allowed_payees.insert(beneficiary_b.into());
    let w = World::open_with(&fx, SimMode::SettleAfterPolls(1), 20_000_000);

    let approved = fx.effect("10000000", "inv-b");
    let candidate = PaymentEffect::prepare(PRINCIPAL, PAYER, beneficiary_b, "10000000", Currency::USD, "inv-b", "invoice-payment", None).unwrap();
    assert_ne!(approved.digest().unwrap(), candidate.digest().unwrap());
    let cert = fx.cert(&approved, "n-1", &["mac", "phone"]);

    // PATH 1: cert-digest. Policy would allow B; only the digest binding refuses it.
    assert_eq!(
        admit_payment_journaled(candidate.clone(), &cert, &w.admission_ctx(&fx), &w.journal).unwrap_err(),
        "REFUSED:EffectDigestMismatch"
    );
    assert!(files_with_suffix(&fx.dir.join("claims"), ".claim.json").is_empty(), "no claim for the candidate");
    assert!(files_with_suffix(&fx.dir.join("journal"), ".admission.json").is_empty(), "no admission snapshot for the candidate");
    assert!(files_with_suffix(&fx.dir.join("nonces"), ".nonce").is_empty(), "refusal happened before the nonce burned");
    raw_balance_untouched(&fx);

    // Control: the approved effect admits with the very same certificate.
    let admission = admit_payment_journaled(approved.clone(), &cert, &w.admission_ctx(&fx), &w.journal).expect("approved admits");
    let a_digest = admission.effect().digest().to_string();
    assert_eq!(admission.effect().payee(), PAYEE);

    // PATH 2: PEE. The sealed effect for A cannot be edited into B, and B is a different identity.
    let pee_a = PreparedEconomicEffect::seal(&admission, &bindings()).unwrap();
    pee_a.verify_identity().unwrap();
    let mut pee_b = pee_a.clone();
    pee_b.beneficiary_account = beneficiary_b.into();
    assert_eq!(pee_b.verify_identity().unwrap_err(), "REFUSED:EFFECT_IDENTITY_MISMATCH");
    let mut only_id_fixed = pee_b.clone();
    only_id_fixed.effect_id = pee_a.effect_id.clone();
    assert_eq!(only_id_fixed.verify_identity().unwrap_err(), "REFUSED:EFFECT_IDENTITY_MISMATCH", "keeping A's id does not launder B");
    pee_b.reseal_identity().unwrap();
    assert_ne!(pee_b.effect_id, pee_a.effect_id);
    assert_ne!(pee_b.idempotency_key, pee_a.idempotency_key);
    assert_eq!(pee_b.amount_minor, pee_a.amount_minor, "amount identical: beneficiary alone changed the identity");

    // B has no claim, so nothing can finalize or settle it at the books.
    let b_digest = candidate.digest().unwrap();
    assert_eq!(
        finalize_via_rail(&b_digest, &w.claims, &w.ledger, &w.rail).unwrap_err(),
        "REFUSED:PAYMENT_NOT_SUBMITTED"
    );
    let fake = FinalityEvidence {
        effect_digest: b_digest.clone(),
        correlation_id: RailInstruction::correlation_id_for(&b_digest),
        evidence_digest: "sha256:whatever".into(),
        kind: FinalityKind::Final,
        reason: "SETTLED".into(),
    };
    assert_eq!(apply_finality(&w.claims, &w.ledger, &fake).unwrap_err(), "REFUSED:PAYMENT_NOT_SUBMITTED");
    raw_balance_untouched(&fx);

    // The approved effect goes through the rail; B's instruction under A's correlation is rejected.
    let s = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted, "{}", s.detail);
    let to_b = RailInstruction {
        effect_id: a_digest.clone(),
        correlation_id: s.correlation_id.clone(),
        message_profile: "pain.001.001.09".into(),
        payload: "<b/>".into(),
        amount_minor: 10_000_000,
        currency: Currency::USD,
        payer: PAYER.into(),
        payee: beneficiary_b.into(),
    };
    match w.rail.submit(&to_b).unwrap() {
        RailAck::Rejected { reason_code, .. } => assert_eq!(reason_code, "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD"),
        RailAck::Accepted { .. } => panic!("rail accepted beneficiary B under A's correlation"),
    }
    let FinalizeResult::Applied(FinalityOutcome::Settled(entry)) = w.settle(&a_digest) else { panic!("A settles") };
    assert_eq!((entry.credit_account.as_str(), entry.amount_minor), (PAYEE, 10_000_000));
    assert_eq!(w.ledger.balance(beneficiary_b, Currency::USD).unwrap(), 0, "B never received a cent");
    assert_eq!(w.ledger.entries().unwrap().len(), 1);
    assert_eq!(w.rail.settlement_count(&s.correlation_id), 1);
    assert!(w.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(files_with_suffix(&fx.dir.join("claims"), ".claim.json").len(), 1);
}

// ---------------------------------------------------------------- COURT C

/// Drop and re-open every store between UNKNOWN and reconciliation, and again between
/// every reconciliation step: exactly one settlement, one submission, one entry.
#[tokio::test]
async fn court_c_restart_durability_converges_to_exactly_one_settlement() {
    let fx = Fixture::new("court-c");
    let mode = SimMode::DropAckAfterAccept;

    let w = World::open(&fx, mode);
    let eff = fx.effect("470000", "INV-C-1");
    let admission = w.admit(&fx, eff, "n-1").expect("admits");
    let d = admission.effect().digest().to_string();
    let correlation = RailInstruction::correlation_id_for(&d);
    let decision = decision_of(&w, &d);
    let s = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome, "{}", s.detail);
    let obligation = "INV-C-1".to_string();
    drop(w); // ---- RESTART 1: between UNKNOWN and reconciliation ----

    let w = World::open(&fx, mode);
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::UnknownOutcome, "UNKNOWN survived the restart");
    assert_eq!(w.claims.get(&d).unwrap().unwrap().obligation_id, obligation);
    assert!(w.ledger.hold_of(&d).unwrap().is_some(), "hold survived the restart");
    assert!(w.ledger.entries().unwrap().is_empty());
    assert_eq!(w.rail.submissions_seen(&correlation), 1, "rail state survived");
    // The nonce fence survived: the spent certificate nonce cannot authorize anything else.
    let other = fx.effect("1000", "inv-other");
    let cert_reused = fx.cert(&other, "n-1", &["mac", "phone"]);
    assert_eq!(admit_payment(other, &cert_reused, &w.admission_ctx(&fx)).unwrap_err(), "REFUSED:NonceReplay");
    // The claim survived: the same effect cannot be re-admitted.
    let retry = fx.effect("470000", "INV-C-1");
    assert_eq!(w.admit(&fx, retry, "n-2").unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");
    // The admission journal survived: replay still reproduces the same decision.
    assert_eq!(decision_of(&w, &d), decision);
    assert_eq!(w.finalize(&d), FinalizeResult::Pending);
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::Submitted);
    drop(w); // ---- RESTART 2: mid reconciliation ----

    let w = World::open(&fx, mode);
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::Submitted);
    assert!(w.ledger.entries().unwrap().is_empty(), "still no settlement without observed finality");
    let FinalizeResult::Applied(FinalityOutcome::Settled(entry)) = w.settle(&d) else { panic!("settles") };
    assert_eq!(entry.amount_minor, 470_000);
    drop(w); // ---- RESTART 3: after settlement ----

    let w = World::open(&fx, mode);
    for _ in 0..3 {
        assert_eq!(w.finalize(&d), FinalizeResult::Applied(FinalityOutcome::AlreadyFinal));
    }
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::Final);
    assert_eq!(w.ledger.entries().unwrap().len(), 1);
    assert_eq!(w.ledger.entries().unwrap()[0], entry);
    assert!(w.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(w.ledger.balance(PAYER, Currency::USD).unwrap(), 530_000);
    assert_eq!(w.ledger.balance(PAYEE, Currency::USD).unwrap(), 470_000);
    assert!(w.ledger.conserves(Currency::USD).unwrap());
    assert_eq!(w.rail.settlement_count(&correlation), 1);
    assert_eq!(w.rail.submissions_seen(&correlation), 1);
    assert_eq!(w.rail.accepted_correlations(), vec![correlation.clone()]);
    assert!(matches!(replay_admission(&w.journal, &d).unwrap(), ReplayVerdict::Reproduced { .. }));
    assert_eq!(files_with_suffix(&fx.dir.join("claims"), ".claim.json").len(), 1);
    assert_eq!(raw_entry_files(&fx).len(), 1);
}

/// Restart while the rail is DOWN: nothing is invented, and recovery completes when it returns.
#[tokio::test]
async fn court_c_restart_with_rail_down_then_recovered_settles_once() {
    let fx = Fixture::new("court-c-down");
    let w = World::open(&fx, SimMode::Down);
    let eff = fx.effect("300000", "INV-C-2");
    let admission = w.admit(&fx, eff, "n-1").unwrap();
    let d = admission.effect().digest().to_string();
    let correlation = RailInstruction::correlation_id_for(&d);
    let s = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome);
    drop(w);

    let w = World::open(&fx, SimMode::Down);
    assert_eq!(w.finalize(&d), FinalizeResult::StillUnknown);
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::UnknownOutcome);
    assert!(w.ledger.entries().unwrap().is_empty());
    drop(w);

    // Rail recovers: it never saw the effect, so the UNKNOWN resolves to proven-absent and the hold frees.
    let w = World::open(&fx, SimMode::Honest);
    assert_eq!(w.finalize(&d), FinalizeResult::ProvenAbsent);
    assert_eq!(w.claims.get(&d).unwrap().unwrap().state, ClaimState::Refused);
    assert!(w.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(w.ledger.available(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert!(w.ledger.entries().unwrap().is_empty());
    assert_eq!(w.rail.submissions_seen(&correlation), 0);
    drop(w);

    // A fresh admission after a restart proceeds normally and settles exactly once.
    let w = World::open(&fx, SimMode::SettleAfterPolls(1));
    let eff = fx.effect("300000", "INV-C-2");
    let admission = w.admit(&fx, eff, "n-2").expect("re-admission after proven absence");
    assert_eq!(admission.effect().digest(), d, "same effect identity");
    let s2 = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await.unwrap();
    assert_eq!(s2.standing, RailStandingAfterSubmit::Submitted, "{}", s2.detail);
    let FinalizeResult::Applied(FinalityOutcome::Settled(_)) = w.settle(&d) else { panic!("settles") };
    assert_eq!(w.ledger.entries().unwrap().len(), 1);
    assert_eq!(w.rail.settlement_count(&correlation), 1);
    assert_eq!(w.rail.submissions_seen(&correlation), 1);
}

/// FINDING (crash window): a process killed after the rail accepted but before the claim left
/// `Reserved` leaves state {claim Reserved, hold live, rail Accepted}. The documented resolver
/// `finalize_via_rail` cannot recover it: Final evidence on a Reserved claim is refused, so the
/// rail can settle while the books never do.
#[tokio::test]
#[ignore = "FINDING: crash after rail accept but before the Reserved->Submitted transition is unrecoverable via finalize_via_rail (Reserved is not in-flight for apply_finality; src/payments/settlement.rs apply_finality `inflight`, src/payments/execute_rail.rs finalize_via_rail). Patch: in finalize_via_rail, when claim.state == Reserved and the rail reports anything but Unknown, first transition Reserved->UnknownOutcome (detail crash-recovered); when the rail reports Unknown for a Reserved claim also route through UnknownOutcome so ProvenAbsent releases the hold."]
async fn court_c_finding_reserved_claim_after_rail_accept_is_unrecoverable() {
    let fx = Fixture::new("court-c-crash");
    let rail_root = fx.dir.join("rail");
    let rail = SimRail::open(&rail_root, SimMode::SettleAfterPolls(1)).unwrap();
    let eff = fx.effect("470000", "inv-crash");
    let adm = fx.admit(eff, "n-1").unwrap();
    let d = adm.effect().digest().to_string();
    let p = params();
    let xml = pain001_customer_credit_transfer(&adm, &p.created_at_iso, &p.debtor_name, &p.creditor_name, &p.debtor_agent_bic, &p.creditor_agent_bic).unwrap();
    let correlation = RailInstruction::correlation_id_for(&d);
    // Exactly the durable state a kill -9 leaves between `rail.submit` returning and the claim transition:
    fx.ledger.hold(&adm, &fx.token_for(&adm)).unwrap();
    let ack = rail
        .submit(&RailInstruction {
            effect_id: d.clone(),
            correlation_id: correlation.clone(),
            message_profile: p.rail_profile.clone(),
            payload: xml,
            amount_minor: 470_000,
            currency: Currency::USD,
            payer: PAYER.into(),
            payee: PAYEE.into(),
        })
        .unwrap();
    assert!(matches!(ack, RailAck::Accepted { .. }));
    assert_eq!(fx.claims.get(&d).unwrap().unwrap().state, ClaimState::Reserved);
    drop(adm);

    // Restart and recover with the documented resolver.
    let claims = ClaimStore::open(fx.dir.join("claims")).unwrap();
    let rail = SimRail::open(&rail_root, SimMode::SettleAfterPolls(1)).unwrap();
    let mut settled = false;
    for _ in 0..6 {
        if let Ok(FinalizeResult::Applied(FinalityOutcome::Settled(_))) = finalize_via_rail(&d, &claims, &fx.ledger, &rail) {
            settled = true;
            break;
        }
    }
    assert_eq!(rail.settlement_count(&correlation), 1, "the rail settled the payment");
    assert!(settled, "books must converge to the rail: exactly one ledger entry for a rail settlement");
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
}

/// FINDING: an obligation id produced by the kernel's own `prepare_for_invoice` cannot be sent
/// through the rail: `obl:sha256:<64 hex>` exceeds ISO 20022 Max35Text for `EndToEndId`, so
/// `submit_via_rail` refuses PROJECTION_FIELD_INVALID and the payment is dead-on-arrival.
#[tokio::test]
#[ignore = "FINDING: derived obligation ids (obl:sha256:<64hex>, ~75 chars) exceed pain.001 EndToEndId Max35Text; submit_via_rail refuses REFUSED:PROJECTION_FIELD_INVALID for every prepare_for_invoice effect (src/payments/iso20022.rs check_max(e.obligation_id(), 35); src/payments/obligation.rs derive_obligation_id). Patch: project EndToEndId as a bounded surrogate (e.g. first 35 chars of a digest of the obligation id) and carry the full obligation id in a Ustrd marker, or shorten derive_obligation_id to <=35 chars."]
async fn court_finding_derived_obligation_ids_cannot_traverse_the_rail() {
    let fx = Fixture::new("court-derived");
    let w = World::open(&fx, SimMode::SettleAfterPolls(1));
    let eff = prepare_for_invoice(PRINCIPAL, PAYER, PAYEE, "470000", Currency::USD, "invoice-payment", "INV-2026-0042", None).unwrap();
    assert!(derive_obligation_id(PAYER, PAYEE, "invoice-payment", "INV-2026-0042").starts_with("obl:sha256:"));
    let admission = w.admit(&fx, eff, "n-1").expect("admits");
    let r = submit_via_rail(admission, &params(), &w.exec_ctx(&fx), &w.rail).await;
    assert!(r.is_ok(), "kernel-derived obligation must be rail-projectable: {:?}", r.err());
}
