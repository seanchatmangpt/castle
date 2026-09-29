//! Admission-hardening courts: quorum floor, PEE binding, replay authenticity, explain, ISO ids.
//! Real collaborators only (real Ed25519 custodians, real stores, real journal files).

mod common;
use common::payments::*;

use castle::payments::admission::{admit_payment_screened, Screening, PAYMENT_SCREENING_REQUIRED};
use castle::payments::compliance::{ComplianceControl, SanctionsList};
use castle::payments::counterparty::CounterpartyRegistry;
use castle::payments::event_receipt::{explain, seal_event_receipt, EventInputs};
use castle::payments::iso20022::*;
use castle::payments::obligation::prepare_for_invoice;
use castle::payments::pee::{PEE_BINDING_INCOMPLETE, PEE_IDENTITY_MISMATCH};
use castle::payments::policy::INSUFFICIENT_QUORUM;
use castle::payments::replay::{
    admit_payment_journaled, admit_payment_screened_journaled, replay_admission, replay_admission_anchored,
    REPLAY_REGISTRY_UNTRUSTED,
};
use castle::payments::*;
use castle::sa2a_security::epoch::SecurityEpochs;
use castle::sa2a_security::{
    ActuationCertificate, CertificateSignature, KeyRecord, KeyRegistry, KeyState, PreparedEffect, SignatureAlgorithm,
};
use ed25519_dalek::{Signer, SigningKey};

const TS: &str = "2026-09-29T12:34:56Z";

// ------------------------------------------------------------ 1. quorum floor

/// Certificate signed by the single custodian "mac" that declares `threshold` itself.
fn self_declared(fx: &Fixture, eff: &PreparedEffect, nonce: &str, threshold: u16) -> ActuationCertificate {
    let mut cert = fx.cert(eff, nonce, &[]);
    cert.threshold = threshold;
    let msg = cert.signing_message().unwrap();
    let mac = SigningKey::from_bytes(&[11; 32]);
    cert.signatures.push(CertificateSignature {
        key_id: "key-mac".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        signature: mac.sign(&msg).to_bytes().to_vec(),
    });
    cert
}

#[test]
fn single_custodian_threshold_one_is_refused_before_nonce_or_budget() {
    let fx = Fixture::new("hard-q1");
    let eff = fx.effect("1000", "inv-a");
    let digest = eff.digest().unwrap();
    let cert = self_declared(&fx, &eff, "n-1", 1);
    assert_eq!(admit_payment(eff.clone(), &cert, &fx.admission_ctx()).unwrap_err(), INSUFFICIENT_QUORUM);
    assert!(fx.claims.get(&digest).unwrap().is_none(), "no budget reserved");
    // Nonce n-1 was not burned: a genuine 2-of-3 certificate under the same nonce admits.
    let good = fx.cert(&eff, "n-1", &["mac", "phone"]);
    assert!(admit_payment(eff, &good, &fx.admission_ctx()).is_ok());
}

#[test]
fn threshold_two_admits_and_explicit_min_quorum_one_allows_single_custodian() {
    let mut fx = Fixture::new("hard-q2");
    let eff = fx.effect("1000", "inv-a");
    assert!(fx.admit(eff, "n-1").is_ok());

    fx.policy.principals.get_mut(PRINCIPAL).unwrap().min_quorum = 1;
    let eff2 = fx.effect("1000", "inv-b");
    let cert = self_declared(&fx, &eff2, "n-2", 1);
    assert!(admit_payment(eff2, &cert, &fx.admission_ctx()).is_ok(), "explicit min_quorum=1 allows it");
}

#[test]
fn min_quorum_defaults_to_two_including_from_json() {
    assert_eq!(PrincipalPolicy::default().min_quorum, 2);
    let p: PrincipalPolicy = serde_json::from_str(
        r#"{"allowed_payers":[],"allowed_payees":[],"per_effect_cap":{},"epoch_cap":{}}"#,
    )
    .unwrap();
    assert_eq!(p.min_quorum, 2, "absent field is default-deny");
    let p1: PrincipalPolicy = serde_json::from_str(
        r#"{"allowed_payers":[],"allowed_payees":[],"per_effect_cap":{},"epoch_cap":{},"min_quorum":1}"#,
    )
    .unwrap();
    assert_eq!(p1.min_quorum, 1);
}

#[test]
fn replay_also_enforces_the_quorum_floor() {
    let mut fx = Fixture::new("hard-q-replay");
    fx.policy.principals.get_mut(PRINCIPAL).unwrap().min_quorum = 1;
    let journal = AdmissionJournal::open(fx.dir.join("j")).unwrap();
    let eff = fx.effect("1000", "inv-a");
    let cert = self_declared(&fx, &eff, "n-1", 1);
    let a = admit_payment_journaled(eff, &cert, &fx.admission_ctx(), &journal).unwrap();
    let d = a.effect().digest().to_string();
    assert!(matches!(replay_admission(&journal, &d).unwrap(), ReplayVerdict::Reproduced { .. }));
    // Tighten the persisted policy: the same record now diverges on the floor.
    let mut rec = journal.load(&d).unwrap();
    rec.policy.principals.get_mut(PRINCIPAL).unwrap().min_quorum = 2;
    let path = fx.dir.join("j").join(format!("{}.admission.json", d.trim_start_matches("sha256:")));
    std::fs::write(path, serde_json::to_vec(&rec).unwrap()).unwrap();
    match replay_admission(&journal, &d).unwrap() {
        ReplayVerdict::Diverged { reason } => assert_eq!(reason, INSUFFICIENT_QUORUM),
        v => panic!("expected divergence, got {v:?}"),
    }
}

// ------------------------------------------------------------ 2. PEE binding

fn bindings(law: &str, cp: &str) -> EffectBindings {
    EffectBindings {
        policy_profile_id: "policy:v1".into(),
        law_state_digest: law.into(),
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

fn registry() -> CounterpartyRegistry {
    let parties = serde_json::json!({ "parties": [
        { "party_id": "p-treasury", "legal_name": "Treasury Ltd", "accounts": [PAYER] },
        { "party_id": "p-supplier", "legal_name": "Supplier GmbH", "accounts": [PAYEE] } ] });
    CounterpartyRegistry::from_json(parties.to_string().as_bytes()).unwrap()
}

fn clean_list() -> SanctionsList {
    let j = serde_json::json!({ "list_id": "OFAC-REF", "version": "1", "entries": ["acct:nobody"] });
    SanctionsList::from_json(j.to_string().as_bytes()).unwrap()
}

#[test]
fn resealed_pee_with_substituted_beneficiary_is_refused_against_its_admission() {
    let fx = Fixture::new("hard-pee");
    let adm = fx.admit(fx.effect("100", "inv-1"), "n1").unwrap();
    let honest = PreparedEconomicEffect::seal(&adm, &bindings("sha256:law", "sha256:cp")).unwrap();
    assert!(honest.verify_against(&adm).is_ok());

    let mut forged = honest.clone();
    forged.beneficiary_account = "acct:attacker".into();
    forged.reseal_identity().unwrap();
    assert!(forged.verify_identity().is_ok(), "self-consistent: identity alone cannot catch it");
    assert_eq!(forged.verify_against(&adm).unwrap_err(), PEE_IDENTITY_MISMATCH);

    let mutations: Vec<(&str, Box<dyn Fn(&mut PreparedEconomicEffect)>)> = vec![
        ("payer", Box::new(|p| p.payer_account = "acct:x".into())),
        ("amount", Box::new(|p| p.amount_minor = "101".into())),
        ("currency", Box::new(|p| p.currency = Currency::JPY)),
        ("purpose", Box::new(|p| p.purpose = "other".into())),
        ("obligation", Box::new(|p| p.obligation_id = "inv-2".into())),
        ("principal", Box::new(|p| p.principal_id = "principal:x".into())),
        ("nonce", Box::new(|p| p.nonce = "n-other".into())),
        ("authority", Box::new(|p| p.authority_digest = "sha256:x".into())),
    ];
    for (name, m) in mutations {
        let mut p = honest.clone();
        m(&mut p);
        p.reseal_identity().unwrap();
        assert_eq!(p.verify_against(&adm).unwrap_err(), PEE_IDENTITY_MISMATCH, "{name}");
    }
    // A PEE sealed from a different admission does not verify against this one.
    let other = fx.admit(fx.effect("100", "inv-9"), "n9").unwrap();
    assert_eq!(honest.verify_against(&other).unwrap_err(), PEE_IDENTITY_MISMATCH);
}

#[test]
fn screened_admission_pins_seal_bindings_to_the_screening_evidence() {
    let fx = Fixture::new("hard-pee-scr");
    let reg = registry();
    let list = clean_list();
    let controls: Vec<&dyn ComplianceControl> = vec![&list];
    let (adm, ev) = admit_payment_screened(
        fx.effect("100", "inv-1"),
        &fx.cert(&fx.effect("100", "inv-1"), "n1", &["mac", "phone"]),
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
    )
    .unwrap();
    let sc = adm.screening().expect("screening recorded");
    assert_eq!(sc, &ev);

    assert_eq!(
        PreparedEconomicEffect::seal(&adm, &bindings("sha256:law", &ev.counterparty_evidence_digest)).unwrap_err(),
        PEE_BINDING_INCOMPLETE,
        "law_state_digest must equal compliance_bundle_digest"
    );
    assert_eq!(
        PreparedEconomicEffect::seal(&adm, &bindings(&ev.compliance_bundle_digest, "sha256:cp")).unwrap_err(),
        PEE_BINDING_INCOMPLETE,
        "counterparty digest must equal the screening evidence"
    );
    let pee =
        PreparedEconomicEffect::seal(&adm, &bindings(&ev.compliance_bundle_digest, &ev.counterparty_evidence_digest)).unwrap();
    assert!(pee.verify_against(&adm).is_ok());
    // Resealing with substituted screening digests is caught by verify_against.
    let mut forged = pee.clone();
    forged.counterparty_evidence_digest = "sha256:other".into();
    forged.reseal_identity().unwrap();
    assert_eq!(forged.verify_against(&adm).unwrap_err(), PEE_IDENTITY_MISMATCH);
}

#[test]
fn require_screening_policy_admits_only_with_screening_evidence() {
    let mut fx = Fixture::new("hard-req-scr");
    fx.policy.require_screening = true;
    let eff = fx.effect("100", "inv-1");
    let cert = fx.cert(&eff, "n1", &["mac", "phone"]);
    assert_eq!(admit_payment(eff.clone(), &cert, &fx.admission_ctx()).unwrap_err(), PAYMENT_SCREENING_REQUIRED);
    let reg = registry();
    let list = clean_list();
    let controls: Vec<&dyn ComplianceControl> = vec![&list];
    let (adm, _) =
        admit_payment_screened(eff, &cert, &fx.admission_ctx(), &Screening { controls: &controls, counterparties: &reg })
            .unwrap();
    assert!(adm.screening().is_some(), "every admission under require_screening carries evidence");

    let mut fx2 = Fixture::new("hard-unscr");
    let a = fx2.admit(fx2.effect("100", "inv-1"), "n1").unwrap();
    assert!(a.screening().is_none(), "unscreened admission carries none");
    fx2.policy.require_screening = false;
}

// ------------------------------------------------------------ 3. replay authenticity

fn rogue_world(tag: &str) -> (KeyRegistry, SigningKey, SpendPolicy, ClaimStore, DurableNonceFence) {
    let dir = unique_dir(tag);
    let key = SigningKey::from_bytes(&[77; 32]);
    let registry = KeyRegistry::from_records(vec![KeyRecord {
        key_id: "key-rogue".into(),
        custodian_id: "device:rogue".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        public_key: key.verifying_key().to_bytes().to_vec(),
        state: KeyState::Active,
        not_before_ms: 0,
        expires_at_ms: 10_000_000,
        revocation_epoch: 0,
    }]);
    let mut policy = SpendPolicy::default();
    policy.principals.insert(
        PRINCIPAL.to_string(),
        PrincipalPolicy {
            allowed_payers: [PAYER.to_string()].into(),
            allowed_payees: [PAYEE.to_string()].into(),
            per_effect_cap: [(Currency::USD, 500_000)].into(),
            epoch_cap: [(Currency::USD, 2_000_000)].into(),
            min_quorum: 1,
        },
    );
    (registry, key, policy, ClaimStore::open(dir.join("claims")).unwrap(), DurableNonceFence::open(dir.join("nonces")).unwrap())
}

#[test]
fn rogue_signed_record_reproduces_unanchored_but_diverges_when_anchored() {
    let fx = Fixture::new("hard-replay");
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();

    // The rogue admits in ITS OWN world, journaling into the victim's journal dir.
    let (rreg, rkey, rpolicy, rclaims, rnonces) = rogue_world("rogue");
    let eff = fx.effect("1000", "inv-rogue");
    let mut cert = fx.cert(&eff, "rn-1", &[]);
    cert.threshold = 1;
    let msg = cert.signing_message().unwrap();
    cert.signatures.push(CertificateSignature {
        key_id: "key-rogue".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        signature: rkey.sign(&msg).to_bytes().to_vec(),
    });
    let ctx = AdmissionContext {
        registry: &rreg,
        epochs: SecurityEpochs { policy: 7, revocation: 0, generation: 1 },
        audience: AUDIENCE,
        now_ms: NOW_MS,
        policy: &rpolicy,
        claims: &rclaims,
        nonces: &rnonces,
    };
    let a = admit_payment_journaled(eff, &cert, &ctx, &journal).expect("rogue world admits its own record");
    let d = a.effect().digest().to_string();

    assert!(
        matches!(replay_admission(&journal, &d).unwrap(), ReplayVerdict::Reproduced { .. }),
        "unanchored replay proves only internal consistency"
    );
    match replay_admission_anchored(&journal, &d, &fx.registry).unwrap() {
        ReplayVerdict::Diverged { reason } => assert_eq!(reason, REPLAY_REGISTRY_UNTRUSTED),
        v => panic!("anchored replay must diverge, got {v:?}"),
    }
}

#[test]
fn genuine_record_reproduces_when_anchored_and_screened_journaled_records() {
    let mut fx = Fixture::new("hard-replay-ok");
    fx.policy.require_screening = true;
    let journal = AdmissionJournal::open(fx.dir.join("journal")).unwrap();
    let reg = registry();
    let list = clean_list();
    let controls: Vec<&dyn ComplianceControl> = vec![&list];
    let eff = fx.effect("1000", "inv-a");
    let cert = fx.cert(&eff, "n-1", &["mac", "phone"]);
    let (a, ev) = admit_payment_screened_journaled(
        eff,
        &cert,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
        &journal,
    )
    .unwrap();
    assert!(a.screening().is_some() && !ev.compliance_bundle_digest.is_empty());
    let d = a.effect().digest();
    assert!(matches!(replay_admission_anchored(&journal, d, &fx.registry).unwrap(), ReplayVerdict::Reproduced { .. }));

    // A refused screened admission writes no record.
    let bad = SanctionsList::from_json(
        serde_json::json!({ "list_id": "L", "version": "1", "entries": [PAYEE] }).to_string().as_bytes(),
    )
    .unwrap();
    let controls: Vec<&dyn ComplianceControl> = vec![&bad];
    let eff2 = fx.effect("1000", "inv-b");
    let d2 = eff2.digest().unwrap();
    let cert2 = fx.cert(&eff2, "n-2", &["mac", "phone"]);
    assert!(admit_payment_screened_journaled(
        eff2,
        &cert2,
        &fx.admission_ctx(),
        &Screening { controls: &controls, counterparties: &reg },
        &journal
    )
    .is_err());
    assert_eq!(journal.load(&d2).unwrap_err(), "REFUSED:REPLAY_RECORD_MISSING");
}

// ------------------------------------------------------------ 4. explain

fn inputs(pee: Option<&str>) -> EventInputs {
    EventInputs {
        effect_id: "sha256:e".into(),
        obligation_id: "inv-1".into(),
        admission_decision_digest: "sha256:d".into(),
        pee_effect_id: pee.map(str::to_string),
        construct_digest: None,
        brce_prepare_digests: vec![],
        brce_outcome_digests: vec![],
        rail_correlation_id: None,
        rail_payload_digest: None,
        finality_evidence_digest: None,
        ledger_entry_digest: None,
        claim_state: "Reserved".into(),
        compliance_bundle_digest: None,
        counterparty_evidence_digest: None,
        settled_at_ms: None,
    }
}

#[test]
fn explain_exposes_pee_effect_id_under_effect_and_null_with_reason_when_absent() {
    let with = explain(&seal_event_receipt(&inputs(Some("sha256:pee-1"))).unwrap());
    assert_eq!(with["effect"]["pee_effect_id"], "sha256:pee-1");
    let without = explain(&seal_event_receipt(&inputs(None)).unwrap());
    assert!(without["effect"]["value"].is_null());
    assert!(without["effect"]["reason"].as_str().is_some_and(|r| !r.is_empty()));
}

// ------------------------------------------------------------ 5. ISO EndToEndId

fn between<'a>(xml: &'a str, open: &str, close: &str) -> &'a str {
    let s = xml.find(open).unwrap() + open.len();
    &xml[s..s + xml[s..].find(close).unwrap()]
}

fn derived_admission(fx: &Fixture) -> PaymentAdmission {
    let eff =
        prepare_for_invoice(PRINCIPAL, PAYER, PAYEE, "470000", Currency::USD, "invoice-payment", "INV-2026-0042", None)
            .unwrap();
    fx.admit(eff, "n-d").unwrap()
}

#[test]
fn derived_long_obligation_id_projects_bounded_end_to_end_id_and_full_marker() {
    let fx = Fixture::new("hard-iso");
    let a = derived_admission(&fx);
    let full = a.effect().obligation_id().to_string();
    assert!(full.chars().count() > 35);
    let pain = pain001_customer_credit_transfer(&a, TS, "Acme", "Supplier", "CASTUS33", "SUPPGB2LXXX").unwrap();
    let e2e = between(&pain, "<EndToEndId>", "</EndToEndId>");
    assert!(e2e.starts_with("OBL-") && e2e.chars().count() <= 35, "{e2e}");
    assert_eq!(project_obligation_id_from_pain001(&pain).as_deref(), Some(full.as_str()));
    assert_eq!(project_effect_digest_from_pain001(&pain).as_deref(), Some(a.effect().digest()));
    // Deterministic.
    let again = pain001_customer_credit_transfer(&a, TS, "Acme", "Supplier", "CASTUS33", "SUPPGB2LXXX").unwrap();
    assert_eq!(pain, again);
    let pacs = pacs008_fi_credit_transfer(&a, TS, "CASTUS33", "SUPPGB2LXXX", "CASTUS33", "SUPPGB2LXXX").unwrap();
    assert!(between(&pacs, "<EndToEndId>", "</EndToEndId>").chars().count() <= 35);
    assert!(pacs.contains(&format!("castle:obligation-id={full}")));
}

#[test]
fn short_ids_are_unchanged_and_unfittable_ids_refuse_typed() {
    let fx = Fixture::new("hard-iso-short");
    let a = fx.admit(fx.effect("100", "inv-1"), "n1").unwrap();
    let pain = pain001_customer_credit_transfer(&a, TS, "Acme", "Supplier", "CASTUS33", "SUPPGB2LXXX").unwrap();
    assert_eq!(between(&pain, "<EndToEndId>", "</EndToEndId>"), "inv-1");
    assert!(!pain.contains("castle:obligation-id="));
    assert_eq!(project_obligation_id_from_pain001(&pain).as_deref(), Some("inv-1"));
    assert_eq!(project_obligation_id_from_pain001("<Document/>"), None);

    let huge = "o".repeat(130);
    let a2 = fx.admit(fx.effect("100", &huge), "n2").unwrap();
    assert_eq!(
        pain001_customer_credit_transfer(&a2, TS, "Acme", "Supplier", "CASTUS33", "SUPPGB2LXXX").unwrap_err(),
        PROJECTION_FIELD_INVALID
    );
}
