//! Falsifier court F1..F10: each test asserts the NEGATION of a falsifier on real behavior.
//! Real BLAKE3/Ed25519/fsynced stores; no mocks. The only double is a hand-written real
//! `LedgerPort` that genuinely fails after posting (F4/F7).
mod common;
use common::payments::*;

use std::collections::BTreeSet;
use std::path::Path;

use castle::payments::*;
use castle::sa2a_security::{CertificateSignature, SignatureAlgorithm};
use ed25519_dalek::{Signer, SigningKey};

async fn settle(fx: &Fixture, amount: &str, obligation: &str, nonce: &str) -> PaymentExecution {
    let a = fx.admit(fx.effect(amount, obligation), nonce).expect("admits");
    execute_payment(a, &fx.exec_ctx()).await.expect("executes")
}

/// Real ledger whose `post` durably writes the entry and then reports Unavailable.
struct PostThenFail<'a>(&'a FileJournalLedger);
impl LedgerPort for PostThenFail<'_> {
    fn balance(&self, account: &str, currency: Currency) -> Result<u64, LedgerError> {
        self.0.balance(account, currency)
    }
    fn lookup(&self, d: &str) -> Result<Option<LedgerEntry>, LedgerError> {
        self.0.lookup(d)
    }
    fn post(&self, a: &PaymentAdmission, c: &str) -> Result<LedgerEntry, LedgerError> {
        self.0.post(a, c)?;
        Err(LedgerError::Unavailable("connection reset after commit".into()))
    }
}

fn exec_ctx_with<'a>(fx: &'a Fixture, ledger: &'a dyn LedgerPort) -> ExecutionContext<'a> {
    ExecutionContext { ledger, ..fx.exec_ctx() }
}

fn assert_no_money_moved(fx: &Fixture) {
    assert!(fx.ledger.entries().unwrap().is_empty(), "ledger must be empty");
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000);
    assert_eq!(fx.ledger.balance(PAYEE, Currency::USD).unwrap(), 0);
}

// ---------------------------------------------------------------- F1
#[test]
fn f1_agent_credential_is_not_payment_authority() {
    let fx = Fixture::new("f1");
    let effect = fx.effect("1000", "inv-f1");

    // Unsigned certificate.
    let unsigned = fx.cert(&effect, "n-unsigned", &[]);
    assert!(admit_payment(effect.clone(), &unsigned, &fx.admission_ctx()).is_err());

    // Signed by a key that is NOT in the registry.
    let mut rogue = fx.cert(&effect, "n-rogue", &[]);
    let msg = rogue.signing_message().unwrap();
    let rogue_key = SigningKey::from_bytes(&[200; 32]);
    for id in ["key-rogue-a", "key-rogue-b"] {
        rogue.signatures.push(CertificateSignature {
            key_id: id.into(),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: rogue_key.sign(&msg).to_bytes().to_vec(),
        });
    }
    assert!(admit_payment(effect.clone(), &rogue, &fx.admission_ctx()).is_err());

    // Registered key ids, but signatures made by a different private key.
    let mut forged = fx.cert(&effect, "n-forged", &[]);
    let msg = forged.signing_message().unwrap();
    for id in ["key-mac", "key-phone"] {
        forged.signatures.push(CertificateSignature {
            key_id: id.into(),
            algorithm: SignatureAlgorithm::Ed25519,
            signature: rogue_key.sign(&msg).to_bytes().to_vec(),
        });
    }
    assert!(admit_payment(effect.clone(), &forged, &fx.admission_ctx()).is_err());

    // A single valid custodian is below the 2-of-3 threshold.
    let one = fx.cert(&effect, "n-one", &["mac"]);
    assert!(admit_payment(effect, &one, &fx.admission_ctx()).is_err());

    assert_no_money_moved(&fx);
    assert!(fx.claims.list().unwrap().is_empty(), "no claim may exist for refused admissions");

    // Source scan: a `PaymentAdmission { .. }` struct literal exists only in admission.rs.
    // (A compile_fail doctest would be the type-level proof; this is the scan-level witness.)
    let mut offenders = Vec::new();
    scan(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(), &mut offenders);
    assert!(offenders.is_empty(), "PaymentAdmission literal outside admission.rs: {offenders:?}");
}

fn scan(dir: &Path, offenders: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            scan(&p, offenders);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            let text = std::fs::read_to_string(&p).unwrap();
            let in_admission = p.ends_with("payments/admission.rs");
            let hits = text
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    l.contains("PaymentAdmission {")
                        && !t.starts_with("//")
                        && !t.starts_with("pub struct PaymentAdmission")
                        && !t.starts_with("impl PaymentAdmission")
                })
                .count();
            if hits > 0 && !in_admission {
                offenders.push(p.display().to_string());
            }
        }
    }
}

// ---------------------------------------------------------------- F2
#[test]
fn f2_policy_violation_is_never_a_constructable_payment() {
    let fx = Fixture::new("f2-cap");
    // per-effect cap is 500_000
    let err = fx.admit(fx.effect("500001", "inv-over"), "n-1").unwrap_err();
    assert_eq!(err, "REFUSED:PAYMENT_AMOUNT_EXCEEDS_CAP");
    assert!(fx.claims.list().unwrap().is_empty());
    assert_no_money_moved(&fx);

    // Epoch cap: 1000 total; a 2000 effect (under per-effect cap) is refused at reservation.
    let fx = Fixture::with("f2-epoch", 1_000_000, 500_000, 1_000);
    let err = fx.admit(fx.effect("2000", "inv-epoch"), "n-1").unwrap_err();
    assert_eq!(err, "REFUSED:PAYMENT_BUDGET_EXCEEDED");
    assert!(fx.claims.list().unwrap().is_empty());
    assert_no_money_moved(&fx);

    // Payee outside the allow-list.
    let bad = PaymentEffect::prepare(PRINCIPAL, PAYER, "acct:stranger", "100", Currency::USD, "inv-x", "p", None).unwrap();
    assert_eq!(fx.admit(bad, "n-2").unwrap_err(), "REFUSED:PAYMENT_ACCOUNT_NOT_ALLOWED");
    assert!(fx.claims.list().unwrap().is_empty());
}

// ---------------------------------------------------------------- F3
#[test]
fn f3_changed_effect_cannot_reuse_prior_authorization() {
    let fx = Fixture::new("f3");
    let signed_for = fx.effect("1000", "inv-f3");
    let cert = fx.cert(&signed_for, "n-1", &["mac", "phone"]);
    let changed = fx.effect("1001", "inv-f3");
    let err = admit_payment(changed, &cert, &fx.admission_ctx()).unwrap_err();
    assert!(err.starts_with("REFUSED:") || err.starts_with("BLOCKED:"), "{err}");
    assert!(fx.claims.list().unwrap().is_empty());
    assert_no_money_moved(&fx);

    // Same for a swapped payee.
    let swapped = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYER, "1000", Currency::USD, "inv-f3", "invoice-payment", None).unwrap();
    assert!(admit_payment(swapped, &cert, &fx.admission_ctx()).is_err());
    assert_no_money_moved(&fx);
}

// ---------------------------------------------------------------- F4
#[tokio::test]
async fn f4_unknown_outcome_never_permits_blind_retry() {
    let fx = Fixture::new("f4");
    let failing = PostThenFail(&fx.ledger);
    let effect = fx.effect("25000", "inv-f4");
    let digest = effect.digest().unwrap();
    let a = fx.admit(effect.clone(), "n-1").unwrap();
    let exec = execute_payment(a, &exec_ctx_with(&fx, &failing)).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::UnknownOutcome, "{}", exec.detail);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::UnknownOutcome);
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);

    // Fresh, valid certificate with a new nonce for the same effect: refused.
    let retry = fx.admit(effect, "n-2");
    assert_eq!(retry.unwrap_err(), "REFUSED:PAYMENT_OUTCOME_UNKNOWN");
    assert_eq!(fx.ledger.entries().unwrap().len(), 1, "blind retry must not add an entry");
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 975_000);
    assert_eq!(fx.claims.get(&digest).unwrap().unwrap().state, ClaimState::UnknownOutcome);
}

// ---------------------------------------------------------------- F5
#[tokio::test]
async fn f5_one_settlement_per_obligation_and_per_effect_digest() {
    let fx = Fixture::new("f5");
    // Two DIFFERENT effects (different purpose) for the SAME obligation and payee.
    let e1 = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "1000", Currency::USD, "inv-dup", "invoice-payment", None).unwrap();
    let e2 = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "1000", Currency::USD, "inv-dup", "invoice-payment-again", None).unwrap();
    assert_ne!(e1.digest().unwrap(), e2.digest().unwrap());

    let x1 = execute_payment(fx.admit(e1.clone(), "n-1").unwrap(), &fx.exec_ctx()).await.unwrap();
    assert_eq!(x1.standing, PaymentStanding::Settled);
    // One settlement per (principal, obligation_id): a distinct digest for the same obligation is refused.
    assert_eq!(fx.admit(e2, "n-2").unwrap_err(), "REFUSED:PAYMENT_OBLIGATION_ALREADY_CLAIMED");

    // Same effect digest can never settle twice.
    assert_eq!(fx.admit(e1, "n-3").unwrap_err(), "REFUSED:PAYMENT_ALREADY_SETTLED");
    assert_eq!(fx.ledger.entries().unwrap().len(), 1);
}

#[tokio::test]
async fn f5_second_live_claim_for_same_obligation_is_refused() {
    let fx = Fixture::new("f5-obl");
    let e1 = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "1000", Currency::USD, "inv-dup", "invoice-payment", None).unwrap();
    let e2 = PaymentEffect::prepare(PRINCIPAL, PAYER, PAYEE, "1000", Currency::USD, "inv-dup", "invoice-payment-again", None).unwrap();
    let x1 = execute_payment(fx.admit(e1, "n-1").unwrap(), &fx.exec_ctx()).await.unwrap();
    assert_eq!(x1.standing, PaymentStanding::Settled);
    assert!(fx.admit(e2, "n-2").is_err(), "second live claim for the same obligation must be refused");
}

// ---------------------------------------------------------------- F6
fn read_journal(root: &Path) -> (Vec<castle::v26_8_18::DurableBrcePrepareRecord>, Vec<castle::v26_8_18::DurableBrceOutcomeRecord>) {
    let (mut p, mut o) = (Vec::new(), Vec::new());
    for e in std::fs::read_dir(root).unwrap() {
        let path = e.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(&path).unwrap();
        if name.ends_with(".prepare.json") {
            p.push(serde_json::from_slice(&bytes).unwrap());
        } else if name.ends_with(".outcome.json") {
            o.push(serde_json::from_slice(&bytes).unwrap());
        }
    }
    (p, o)
}

#[tokio::test]
async fn f6_receipts_reconstruct_the_consequence() {
    let fx = Fixture::new("f6");
    let exec = settle(&fx, "70000", "inv-f6", "n-1").await;
    assert_eq!(exec.standing, PaymentStanding::Settled, "{}", exec.detail);
    let construct = exec.construct_digest.clone().expect("construct digest");
    assert!(exec.ocel_receipt_digest.as_deref().is_some_and(|d| !d.is_empty()));
    assert_eq!(exec.brce_prepare_receipt_digests.len(), 2);
    assert_eq!(exec.brce_outcome_receipt_digests.len(), 2);

    let claim = fx.claims.get(&exec.effect_digest).unwrap().unwrap();
    assert_eq!(claim.construct_digest.as_deref(), Some(construct.as_str()));
    let entry = fx.ledger.lookup(&exec.effect_digest).unwrap().expect("ledger entry");
    assert_eq!(entry.construct_digest, construct);
    assert_eq!(exec.ledger_entry.as_ref().unwrap(), &entry);

    // Durable BRCE journal on disk agrees with the execution's digests and the construct.
    let (prepares, outcomes) = read_journal(&fx.dir.join("brce"));
    let on_disk_p: BTreeSet<_> = prepares.iter().map(|r| r.prepare_receipt.receipt_digest.clone()).collect();
    let on_disk_o: BTreeSet<_> = outcomes.iter().map(|r| r.outcome_receipt.receipt_digest.clone()).collect();
    assert_eq!(on_disk_p, exec.brce_prepare_receipt_digests.iter().cloned().collect());
    assert_eq!(on_disk_o, exec.brce_outcome_receipt_digests.iter().cloned().collect());
    assert!(prepares.iter().all(|r| r.construct_digest == construct));
    assert!(outcomes.iter().all(|o| on_disk_p.contains(&o.prepare_receipt_digest)));
}

// ---------------------------------------------------------------- F7
#[tokio::test]
async fn f7_rail_failure_keeps_economic_identity() {
    let fx = Fixture::new("f7");
    let failing = PostThenFail(&fx.ledger);
    let effect = fx.effect("31000", "inv-f7");
    let digest = effect.digest().unwrap();
    let exec = execute_payment(fx.admit(effect, "n-1").unwrap(), &exec_ctx_with(&fx, &failing)).await.unwrap();
    assert_eq!(exec.standing, PaymentStanding::UnknownOutcome);

    let claim = fx.claims.get(&digest).unwrap().expect("claim survives rail failure");
    assert_eq!(claim.state, ClaimState::UnknownOutcome);
    assert_eq!(claim.effect_digest, digest);
    assert_eq!(claim.amount_minor, 31_000);
    assert_eq!(claim.payee, PAYEE);
    assert_eq!(claim.payer, PAYER);
    assert_eq!(claim.currency, Currency::USD);
    assert_eq!(claim.principal, PRINCIPAL);
    // The claim still counts against budget (not silently released).
    assert!(fx.claims.list().unwrap().iter().any(|c| c.effect_digest == digest && c.state != ClaimState::Refused));
}

// ---------------------------------------------------------------- F8
#[test]
fn f8_experience_module_exists() {
    assert!(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/payments/experience.rs").is_file());
}

// ---------------------------------------------------------------- F9
#[tokio::test]
async fn f9_reported_value_reconciles_to_ledger_evidence() {
    let fx = Fixture::new("f9");
    for (i, amt) in ["1000", "2500", "40000"].iter().enumerate() {
        let x = settle(&fx, amt, &format!("inv-f9-{i}"), &format!("n-{i}")).await;
        assert_eq!(x.standing, PaymentStanding::Settled);
    }
    let claimed: u64 = fx
        .claims
        .list()
        .unwrap()
        .iter()
        .filter(|c| c.state == ClaimState::Executed && c.currency == Currency::USD)
        .map(|c| c.amount_minor)
        .sum();
    let debited: u64 = fx
        .ledger
        .entries()
        .unwrap()
        .iter()
        .filter(|e| e.currency == Currency::USD && e.debit_account == PAYER)
        .map(|e| e.amount_minor)
        .sum();
    assert_eq!(claimed, 43_500);
    assert_eq!(claimed, debited);
    assert_eq!(fx.ledger.balance(PAYER, Currency::USD).unwrap(), 1_000_000 - debited);
    assert!(fx.ledger.conserves(Currency::USD).unwrap());
}

// ---------------------------------------------------------------- F10
/// Explanation built ONLY from persisted state (claim file, ledger entry file, BRCE journal
/// files) plus the PaymentExecution value; returns the fields that are missing.
fn explain(fx: &Fixture, exec: &PaymentExecution) -> (serde_json::Value, Vec<&'static str>) {
    let claim = fx.claims.get(&exec.effect_digest).unwrap().unwrap();
    let entry = fx.ledger.lookup(&exec.effect_digest).unwrap().unwrap();
    let claim_json = serde_json::to_value(&claim).unwrap();
    let mut persisted = claim_json.to_string();
    persisted.push_str(&serde_json::to_string(&entry).unwrap());
    for e in std::fs::read_dir(fx.dir.join("brce")).unwrap() {
        persisted.push_str(&std::fs::read_to_string(e.unwrap().path()).unwrap());
    }
    let expl = serde_json::json!({
        "principal": claim.principal,
        "payer": entry.debit_account,
        "payee": entry.credit_account,
        "amount_minor": entry.amount_minor,
        "currency": entry.currency.code(),
        "construct_digest": exec.construct_digest,
        "ocel_receipt_digest": exec.ocel_receipt_digest,
    });
    let mut missing = Vec::new();
    for (name, needle) in [
        ("obligation_id", "inv-f10"),
        ("purpose", "invoice-payment"),
        ("authority_audience", AUDIENCE),
        ("verified_custodian_ids", "device:mac"),
    ] {
        if !persisted.contains(needle) {
            missing.push(name);
        }
    }
    (expl, missing)
}

#[tokio::test]
async fn f10_core_explanation_fields_derive_from_persisted_state() {
    let fx = Fixture::new("f10");
    let exec = settle(&fx, "12345", "inv-f10", "n-1").await;
    assert_eq!(exec.standing, PaymentStanding::Settled);
    let (e, _missing) = explain(&fx, &exec);
    assert_eq!(e["principal"], PRINCIPAL);
    assert_eq!(e["payer"], PAYER);
    assert_eq!(e["payee"], PAYEE);
    assert_eq!(e["amount_minor"], 12_345);
    assert_eq!(e["currency"], "USD");
    assert!(e["construct_digest"].is_string());
    assert!(e["ocel_receipt_digest"].is_string());
}

/// FINDING: obligation_id, purpose, authority audience and verified custodian ids are NOT
/// persisted anywhere a board can read after the fact (Claim, LedgerEntry, BRCE journal).
#[tokio::test]

async fn f10_board_can_determine_why_money_moved_from_persisted_state_alone() {
    let fx = Fixture::new("f10-full");
    let exec = settle(&fx, "12345", "inv-f10", "n-1").await;
    let (_e, missing) = explain(&fx, &exec);
    assert!(missing.is_empty(), "not derivable from persisted state: {missing:?}");
}
