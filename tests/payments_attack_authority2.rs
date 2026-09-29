//! Authority skeptic lane 2: attacks the rail/finality/PEE/screening/replay surface.
//! Tests assert the SECURE behavior. Plain tests are controls that hold today;
//! `#[ignore = "FINDING: ..."]` tests fail today and demonstrate a real defect
//! (run with `--ignored`). Real collaborators only; the hostile rail is a hand-written
//! real `RailActuator` impl, not a mock.
mod common;
use common::payments::*;

use castle::payments::admission::{admit_payment_screened, Screening};
use castle::payments::compliance::{ComplianceControl, SanctionsList};
use castle::payments::counterparty::CounterpartyRegistry;
use castle::payments::execute_rail::*;
use castle::payments::rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus};
use castle::payments::replay::{replay_admission, AdmissionJournal, AdmissionRecord, ReplayVerdict};
use castle::payments::settlement::{apply_finality, FinalityEvidence, FinalityKind};
use castle::payments::*;
use castle::sa2a_security::{CertificateSignature, KeyRecord, KeyRegistry, KeyState, SignatureAlgorithm};
use ed25519_dalek::{Signer, SigningKey};

fn bindings(cp: &str) -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: "law:v1".into(),
        counterparty_evidence_digest: cp.into(),
        funding_source_id: "fund:treasury".into(),
        resource_reservation_id: "res:1".into(),
        rail_profile_id: "sim-rail".into(),
        message_profile_version: "pain.001.001.09".into(),
        parent_receipt: "receipt:parent".into(),
        created_at_ms: 0,
        expires_at_ms: 100_000,
    }
}

fn params(cp: &str) -> RailExecutionParams {
    RailExecutionParams {
        bindings: bindings(cp),
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

async fn submit(fx: &Fixture, rail: &dyn RailActuator, amount: &str, obl: &str, nonce: &str) -> RailSubmission {
    let a = fx.admit(fx.effect(amount, obl), nonce).expect("admits");
    submit_via_rail(a, &params("cp:v1"), &fx.exec_ctx(), rail).await.expect("submit_via_rail")
}

fn state(fx: &Fixture, d: &str) -> ClaimState {
    fx.claims.get(d).unwrap().unwrap().state
}

fn forged(d: &str, kind: FinalityKind) -> FinalityEvidence {
    FinalityEvidence {
        effect_digest: d.to_string(),
        correlation_id: "E2E-attacker-chosen".to_string(),
        evidence_digest: "sha256:attacker-invented".to_string(),
        kind,
        reason: "FORGED".to_string(),
    }
}

/// A real, hand-written rail that forgets everything it was told (replica lag / amnesia /
/// hostile). Submits and status reports are forwarded, except status reports `Unknown`.
struct AmnesiacRail<'a>(&'a SimRail);
impl RailActuator for AmnesiacRail<'_> {
    fn submit(&self, i: &RailInstruction) -> Result<RailAck, RailError> {
        self.0.submit(i)
    }
    fn status(&self, _c: &str) -> Result<RailStatus, RailError> {
        Ok(RailStatus::Unknown)
    }
}

// ---------------------------------------------------------------- controls (hold today)

#[tokio::test]
async fn control_submit_only_holds_and_writes_no_ledger_entry() {
    let fx = Fixture::new("a2-ctl-hold");
    let rail = sim(&fx, "a", SimMode::SettleAfterPolls(1000));
    let s = submit(&fx, &rail, "1000", "inv-1", "n-1").await;
    assert_eq!(s.standing, RailStandingAfterSubmit::Submitted);
    assert!(fx.ledger.entries().unwrap().is_empty());
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
}

#[tokio::test]
async fn control_foreign_claim_store_places_no_hold_and_calls_no_rail() {
    let fx = Fixture::new("a2-ctl-foreign");
    let rail = sim(&fx, "a", SimMode::Honest);
    let a = fx.admit(fx.effect("1000", "inv-f"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    let other = ClaimStore::open(fx.dir.join("other-claims")).unwrap();
    let mut ctx = fx.exec_ctx();
    ctx.claims = &other;
    let r = submit_via_rail(a, &params("cp:v1"), &ctx, &rail).await;
    assert!(r.is_err());
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
    assert_eq!(rail.submissions_seen(&RailInstruction::correlation_id_for(&d)), 0);
}

#[tokio::test]
async fn control_stale_claim_cannot_submit() {
    let fx = Fixture::new("a2-ctl-stale");
    let rail = sim(&fx, "a", SimMode::Honest);
    let a = fx.admit(fx.effect("1000", "inv-s"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    fx.claims.transition(&d, &[ClaimState::Reserved], ClaimState::Refused, None, "released").unwrap();
    assert!(submit_via_rail(a, &params("cp:v1"), &fx.exec_ctx(), &rail).await.is_err());
    assert!(fx.ledger.hold_of(&d).unwrap().is_none());
}

#[test]
fn control_finality_on_unsubmitted_claim_is_refused() {
    let fx = Fixture::new("a2-ctl-unsub");
    let a = fx.admit(fx.effect("1000", "inv-u"), "n-1").unwrap();
    let d = a.effect().digest().to_string();
    let r = apply_finality(&fx.claims, &fx.ledger, &forged(&d, FinalityKind::Final));
    assert!(r.is_err(), "Reserved claim must not settle: {r:?}");
    assert!(fx.ledger.entries().unwrap().is_empty());
}

#[test]
fn control_unscreened_admit_refused_when_policy_requires_screening() {
    let mut fx = Fixture::new("a2-ctl-scr");
    fx.policy.require_screening = true;
    let e = fx.effect("1000", "inv-x");
    let err = fx.admit(e, "n-1").unwrap_err();
    assert_eq!(err, "REFUSED:PAYMENT_SCREENING_REQUIRED");
}

#[test]
fn control_replay_returns_a_verdict_never_an_admission_and_touches_no_state() {
    let fx = Fixture::new("a2-ctl-replay");
    let j = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let e = fx.effect("1000", "inv-r");
    let cert = fx.cert(&e, "n-1", &["mac", "phone"]);
    let a = castle::payments::replay::admit_payment_journaled(e, &cert, &fx.admission_ctx(), &j).unwrap();
    let d = a.effect().digest().to_string();
    let before = fx.claims.get(&d).unwrap().unwrap();
    let v = replay_admission(&j, &d).unwrap();
    assert!(matches!(v, ReplayVerdict::Reproduced { .. }));
    assert_eq!(fx.claims.get(&d).unwrap().unwrap(), before);
    assert!(fx.ledger.entries().unwrap().is_empty());
}

// ---------------------------------------------------------------- FINDINGS

/// FINDING-A1: `apply_finality` is `pub` and `FinalityEvidence` is a plain struct with pub
/// fields, so any caller holding (claims, ledger) can settle a payment the rail never
/// settled. Nothing binds evidence to the rail (no unforgeable RailReport token) and the
/// `correlation_id` is not even compared to the claim's.
#[tokio::test]
#[ignore = "FINDING: apply_finality accepts attacker-authored FinalityEvidence; ledger settles with no rail settlement"]
async fn forged_final_evidence_cannot_settle_a_payment_the_rail_never_settled() {
    let fx = Fixture::new("a2-forge-final");
    let rail = sim(&fx, "a", SimMode::SettleAfterPolls(1000));
    let s = submit(&fx, &rail, "1000", "inv-1", "n-1").await;
    let _ = apply_finality(&fx.claims, &fx.ledger, &forged(&s.effect_id, FinalityKind::Final));
    assert!(fx.ledger.lookup(&s.effect_id).unwrap().is_none(), "ledger settled from forged evidence");
    assert_ne!(state(&fx, &s.effect_id), ClaimState::Final);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
}

/// FINDING-A2: forged REJECTED evidence releases the hold and refuses the claim while the
/// rail goes on to settle: funds are freed in the books but paid on the rail.
#[tokio::test]
#[ignore = "FINDING: forged Rejected evidence releases the hold of a payment the rail accepted"]
async fn forged_rejection_cannot_release_a_hold_on_an_accepted_payment() {
    let fx = Fixture::new("a2-forge-reject");
    let rail = sim(&fx, "a", SimMode::SettleAfterPolls(1000));
    let s = submit(&fx, &rail, "1000", "inv-1", "n-1").await;
    let _ = apply_finality(&fx.claims, &fx.ledger, &forged(&s.effect_id, FinalityKind::Rejected));
    assert!(fx.ledger.hold_of(&s.effect_id).unwrap().is_some(), "hold released by forged rejection");
    assert_eq!(state(&fx, &s.effect_id), ClaimState::Submitted);
}

/// FINDING-A3: forged RETURNED evidence after genuine settlement posts an inverse entry,
/// clawing the funds back to the payer in the books with no return on the rail.
#[tokio::test]
#[ignore = "FINDING: forged Returned evidence reverses a genuinely settled payment"]
async fn forged_return_cannot_reverse_a_settled_payment() {
    let fx = Fixture::new("a2-forge-return");
    let rail = sim(&fx, "a", SimMode::Honest);
    let s = submit(&fx, &rail, "1000", "inv-1", "n-1").await;
    for _ in 0..6 {
        if finalize_via_rail(&s.effect_id, &fx.claims, &fx.ledger, &rail).unwrap() != FinalizeResult::Pending {
            break;
        }
    }
    assert_eq!(state(&fx, &s.effect_id), ClaimState::Final);
    let payee_before = fx.ledger.balance(PAYEE, Currency::USD).unwrap();
    let _ = apply_finality(&fx.claims, &fx.ledger, &forged(&s.effect_id, FinalityKind::Returned));
    assert!(fx.ledger.returns().unwrap().is_empty(), "inverse entry posted from forged evidence");
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), payee_before);
}

/// FINDING-A4: `RailStatus::Unknown` from a reachable rail is treated as proof of absence
/// (`ProvenAbsent`): hold released, claim Refused. A lagging/amnesiac/hostile rail that
/// actually accepted the payment turns UnknownOutcome into a released hold, and the later
/// genuine settlement is then unbookable (EVIDENCE_CONFLICT): rail paid, books say refused.
#[tokio::test]
#[ignore = "FINDING: a rail reporting Unknown is trusted as proof the payment never happened"]
async fn amnesiac_rail_unknown_cannot_release_an_unknown_outcome_hold() {
    let fx = Fixture::new("a2-amnesia");
    let real = sim(&fx, "a", SimMode::DropAckAfterAccept);
    let a = fx.admit(fx.effect("1000", "inv-1"), "n-1").unwrap();
    let s = submit_via_rail(a, &params("cp:v1"), &fx.exec_ctx(), &real).await.unwrap();
    assert_eq!(s.standing, RailStandingAfterSubmit::UnknownOutcome);
    let liar = AmnesiacRail(&real);
    let _ = finalize_via_rail(&s.effect_id, &fx.claims, &fx.ledger, &liar);
    assert_eq!(state(&fx, &s.effect_id), ClaimState::UnknownOutcome, "Unknown status released the claim");
    assert!(fx.ledger.hold_of(&s.effect_id).unwrap().is_some(), "hold released on unverified absence");
}

/// FINDING-A5: `PreparedEconomicEffect::verify_identity` is self-referential and
/// `reseal_identity` is public: a PEE with a substituted beneficiary, resealed, verifies.
/// There is no `verify_against(&PaymentAdmission)`, and `RailSubmission.pee` is never
/// compared with the instruction actually sent.
#[test]
#[ignore = "FINDING: PEE with substituted beneficiary passes verify_identity after reseal"]
fn pee_with_substituted_beneficiary_fails_verification() {
    let fx = Fixture::new("a2-pee");
    let a = fx.admit(fx.effect("1000", "inv-1"), "n-1").unwrap();
    let mut pee = PreparedEconomicEffect::seal(&a, &bindings("cp:v1")).unwrap();
    assert_ne!(pee.beneficiary_account, "acct:attacker");
    pee.beneficiary_account = "acct:attacker".to_string();
    pee.reseal_identity().unwrap();
    assert!(pee.verify_identity().is_err(), "substituted beneficiary verified as authentic");
}

/// FINDING-A6: `EffectBindings.counterparty_evidence_digest` / `law_state_digest` are free
/// strings; `submit_via_rail` never checks them against the `ScreeningEvidence` that
/// `admit_payment_screened` produced, so a screened payment's PEE can carry an arbitrary
/// (or unrelated) screening binding.
#[tokio::test]
#[ignore = "FINDING: PEE counterparty binding is not verified against the screening evidence"]
async fn pee_counterparty_binding_must_match_screening_evidence() {
    let mut fx = Fixture::new("a2-bind");
    fx.policy.require_screening = true;
    let e = fx.effect("1000", "inv-1");
    let cert = fx.cert(&e, "n-1", &["mac", "phone"]);
    let reg = CounterpartyRegistry::from_json(
        serde_json::json!({ "parties": [
            { "party_id": "p-t", "legal_name": "Treasury Ltd", "accounts": [PAYER] },
            { "party_id": "p-s", "legal_name": "Supplier GmbH", "accounts": [PAYEE] } ] })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    let list = SanctionsList::from_json(br#"{"list_id":"L","version":"1","entries":["acct:nobody"]}"#).unwrap();
    let controls: Vec<&dyn ComplianceControl> = vec![&list];
    let (a, ev) = admit_payment_screened(
        e,
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
    )
    .unwrap();
    assert_ne!(ev.counterparty_evidence_digest, "cp:unrelated-garbage");
    let rail = sim(&fx, "a", SimMode::Honest);
    let r = submit_via_rail(a, &params("cp:unrelated-garbage"), &fx.exec_ctx(), &rail).await;
    assert!(r.is_err(), "submitted with a counterparty binding unrelated to the screening evidence");
}

/// FINDING-A7: the replay record is unauthenticated and carries its own key-registry
/// snapshot. An attacker who can write a file into the journal dir (or a rogue journal)
/// forges a record signed by attacker-owned custodian keys and `replay_admission` returns
/// `Reproduced`: a lawful-looking decision digest for an effect no trusted registry ever
/// authorized. Replay does not admit, but its verdict is not evidence of authority.
#[test]
#[ignore = "FINDING: replay Reproduced verdict is derived from an attacker-supplied registry snapshot"]
fn replay_of_a_forged_record_signed_by_rogue_keys_is_not_reproduced() {
    let fx = Fixture::new("a2-replay-forge");
    let root = fx.dir.join("journal");
    let j = AdmissionJournal::open(&root).unwrap();
    let effect = fx.effect("1000", "inv-forged");
    let digest = effect.digest().unwrap();

    let rogue: Vec<(String, SigningKey)> =
        ["r1", "r2"].iter().enumerate().map(|(i, n)| ((*n).to_string(), SigningKey::from_bytes(&[(i as u8) + 77; 32]))).collect();
    let registry = KeyRegistry::from_records(rogue.iter().map(|(n, k)| KeyRecord {
        key_id: format!("rk-{n}"),
        custodian_id: format!("device:rogue-{n}"),
        algorithm: SignatureAlgorithm::Ed25519,
        public_key: k.verifying_key().to_bytes().to_vec(),
        state: KeyState::Active,
        not_before_ms: 0,
        expires_at_ms: 10_000_000,
        revocation_epoch: 0,
    }));
    let mut cert = fx.cert(&effect, "rogue-nonce", &[]);
    let msg = cert.signing_message().unwrap();
    for (n, k) in &rogue {
        cert.signatures.push(CertificateSignature {
            key_id: format!("rk-{n}"),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: k.sign(&msg).to_bytes().to_vec(),
        });
    }
    let rec = AdmissionRecord {
        prepared: effect,
        certificate: cert,
        registry: registry.records(),
        policy_epoch: 7,
        revocation_epoch: 0,
        generation: 1,
        audience: AUDIENCE.to_string(),
        now_ms: NOW_MS,
        policy: fx.policy.clone(),
    };
    let path = root.join(format!("{}.admission.json", digest.trim_start_matches("sha256:")));
    std::fs::write(&path, serde_json::to_vec(&rec).unwrap()).unwrap();

    let v = replay_admission(&j, &digest);
    assert!(!matches!(v, Ok(ReplayVerdict::Reproduced { .. })), "forged record reproduced: {v:?}");
}
