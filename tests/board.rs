//! Coverage for `src/board.rs` — the independent `ReceiptV2` (BLAKE3 + real
//! Ed25519) chain used for Fortune-5 evidence admission and board-package
//! qualification. `board.ts` (the TypeScript predecessor) shipped with zero
//! tests; this file closes that gap. Chicago-style throughout: a real
//! `ed25519-dalek` keypair and the real `blake3` crate, state-based
//! assertions on the actual returned structs — no mocking.

use std::collections::{BTreeMap, HashMap};

use castle::board::*;
use castle::fortune5::{EvidenceEpistemicClass, MetricValue, QualificationContext, Standing};
use castle::fortune5_generated::FORTUNE5_REQUIREMENTS;
use ed25519_dalek::SigningKey;

const SUBJECT: &str = "castle:board-test";

fn digest(seed: &str) -> String {
    blake3::hash(seed.as_bytes()).to_hex().to_string()
}

fn trust_store(key_id: &str, verifying_key: ed25519_dalek::VerifyingKey, assurance_domain: &str) -> TrustStore {
    let mut keys = HashMap::new();
    keys.insert(
        key_id.to_string(),
        TrustKey { key_id: key_id.to_string(), verifying_key, valid_from_epoch: 0, revoked_at_epoch: None, assurance_domain: assurance_domain.to_string() },
    );
    TrustStore { current_epoch: 5, keys }
}

fn issue(metric: &str, value: MetricValue, signing_key: &SigningKey, key_id: &str, assurance_domain: &str, trust_epoch: i64) -> EvidenceBundle {
    let input = EvidenceInput { metric: metric.to_string(), value, subject: SUBJECT.to_string(), observed_at: "2026-08-15T19:59:00.000Z".to_string(), epistemic_class: EvidenceEpistemicClass::Observed };
    let context = ReceiptIssueContext {
        policy_digest: digest("policy"),
        authority_digest: digest("authority"),
        parent_digests: vec![],
        key_id: key_id.to_string(),
        trust_epoch,
        assurance_domain: assurance_domain.to_string(),
    };
    issue_evidence_receipt(&input, &context, signing_key).expect("real Ed25519 receipt issuance succeeds")
}

#[test]
fn issued_receipt_verifies_alive_against_its_real_trust_store() {
    let signing_key = SigningKey::from_bytes(&[3u8; 32]);
    let verifying_key = signing_key.verifying_key();
    let trust = trust_store("board-root", verifying_key, "castle-assurance");
    let bundle = issue("deny_by_default", MetricValue::Bool(true), &signing_key, "board-root", "castle-assurance", 5);

    let verification = verify_receipt_dag(&bundle.receipt, &trust, &HashMap::new());
    assert_eq!(verification.standing, Standing::Alive);
    assert_eq!(verification.reasons, vec!["ALIVE:RECEIPT_DAG_VERIFIED".to_string()]);
    assert_eq!(verification.verified_digests, vec![bundle.receipt.receipt_digest.clone()]);
}

#[test]
fn evidence_from_an_untrusted_key_is_refused_not_silently_accepted() {
    let issuing_key = SigningKey::from_bytes(&[9u8; 32]);
    let bundle = issue("deny_by_default", MetricValue::Bool(true), &issuing_key, "board-root", "castle-assurance", 5);

    // Trust store knows a DIFFERENT key under the same key_id (real key-mismatch scenario,
    // not a fabricated signature) — the verifying key here never signed this receipt.
    let other_key = SigningKey::from_bytes(&[10u8; 32]).verifying_key();
    let wrong_trust = trust_store("board-root", other_key, "castle-assurance");

    let verification = verify_receipt_dag(&bundle.receipt, &wrong_trust, &HashMap::new());
    assert_eq!(verification.standing, Standing::Refused);
    assert!(verification.reasons.contains(&"REFUSED:INVALID_RECEIPT_SIGNATURE".to_string()));
}

#[test]
fn admit_evidence_round_trips_a_real_issued_receipt_to_alive() {
    let signing_key = SigningKey::from_bytes(&[4u8; 32]);
    let trust = trust_store("board-root", signing_key.verifying_key(), "castle-assurance");
    let bundle = issue("sbom_coverage_bps", MetricValue::Number(10000.0), &signing_key, "board-root", "castle-assurance", 5);

    let admission = admit_evidence(&bundle, &trust, &HashMap::new());
    assert_eq!(admission.standing, Standing::Alive);
    assert!(admission.observation.is_some());
    assert_eq!(admission.observation.unwrap().metric, "sbom_coverage_bps");
}

#[test]
fn admit_evidence_refuses_when_the_bundles_receipt_digest_reference_is_tampered() {
    let signing_key = SigningKey::from_bytes(&[5u8; 32]);
    let trust = trust_store("board-root", signing_key.verifying_key(), "castle-assurance");
    let mut bundle = issue("sbom_coverage_bps", MetricValue::Number(10000.0), &signing_key, "board-root", "castle-assurance", 5);
    bundle.observation.receipt_digest = digest("a-different-artifact-entirely");

    let admission = admit_evidence(&bundle, &trust, &HashMap::new());
    assert_eq!(admission.standing, Standing::Refused);
    assert!(admission.reasons.contains(&"REFUSED:EVIDENCE_RECEIPT_REFERENCE_MISMATCH".to_string()));
    assert!(admission.observation.is_none());
}

#[test]
fn qualify_verified_fortune5_qualifies_alive_only_from_admitted_receipted_evidence() {
    let signing_key = SigningKey::from_bytes(&[6u8; 32]);
    let trust = trust_store("board-root", signing_key.verifying_key(), "castle-assurance");

    let requirement = FORTUNE5_REQUIREMENTS.iter().find(|r| r.metric == "deny_by_default").unwrap();
    let bundle = issue(requirement.metric, MetricValue::Bool(true), &signing_key, "board-root", "castle-assurance", 5);

    let context = QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: None, max_evidence_age_ms: None };
    let result = qualify_verified_fortune5(&[bundle], &context, &trust, &HashMap::new(), std::slice::from_ref(requirement));

    assert_eq!(result.standing, Standing::Alive);
    assert_eq!(result.qualification.alive, 1);
    assert!(result.evidence_refusals.is_empty());
}

#[test]
fn qualify_fortune5_board_requires_genuinely_independent_assurance_domains() {
    let signing_key = SigningKey::from_bytes(&[8u8; 32]);
    let trust = trust_store("board-root", signing_key.verifying_key(), "castle-assurance");
    let bundle = issue("deny_by_default", MetricValue::Bool(true), &signing_key, "board-root", "castle-assurance", 5);
    let context = QualificationContext { subject: SUBJECT.to_string(), now_epoch_ms: None, max_evidence_age_ms: None };

    // Same assurance domain on both sides -> not independent -> REFUSED, even though the
    // underlying evidence itself verifies fine.
    let admission = qualify_fortune5_board(BoardAdmissionInput {
        enterprise_context: context.clone(),
        enterprise_evidence: vec![bundle.clone()],
        castle_context: context.clone(),
        castle_evidence: vec![bundle],
        trust: &trust,
        receipt_store: HashMap::new(),
        castle_assurance_domain: "castle-assurance".to_string(),
        independent_assurance_domain: "castle-assurance".to_string(),
    });
    assert_eq!(admission.standing, Standing::Refused);
    assert!(admission.reasons.contains(&"REFUSED:ASSURANCE_NOT_INDEPENDENT".to_string()));
}

#[test]
fn failure_semantics_fail_closed_never_actuates_without_castle() {
    let decision = admit_failure_semantics(&FailureSemanticsInput { mode: FailureMode::FailClosed, castle_available: false, local_capability_verified: false, receipt_channel_available: false });
    assert_eq!(decision.standing, Standing::Alive);
    assert!(!decision.may_actuate);
    assert_eq!(decision.reason, "ALIVE:FAIL_CLOSED");
}

#[test]
fn failure_semantics_refuses_a_degraded_do_with_no_receipt_channel() {
    let decision = admit_failure_semantics(&FailureSemanticsInput { mode: FailureMode::SafeDegrade, castle_available: false, local_capability_verified: false, receipt_channel_available: false });
    assert_eq!(decision.standing, Standing::Refused);
    assert!(!decision.may_actuate);
    assert_eq!(decision.reason, "REFUSED:UNRECEIPTABLE_DEGRADED_DO");
}

#[test]
fn materiality_assessment_triggers_and_schedules_escalation_when_a_dimension_crosses_threshold() {
    let policy = MaterialityPolicy {
        policy_digest: digest("materiality-policy"),
        authority_digest: digest("materiality-authority"),
        per_dimension_threshold_bps: BTreeMap::from([(MaterialityDimension::Financial, 500)]),
        aggregate_threshold_bps: 9000,
        escalation_within_ms: 3_600_000,
    };
    let event = MaterialityEvent { id: "evt:1".to_string(), subject: SUBJECT.to_string(), occurred_at_epoch_ms: 1_000, impact_bps: BTreeMap::from([(MaterialityDimension::Financial, 750)]) };

    let assessment = assess_materiality(&event, &policy).expect("valid materiality inputs");
    assert!(assessment.material);
    assert_eq!(assessment.triggering_dimensions, vec![MaterialityDimension::Financial]);
    assert_eq!(assessment.escalate_by_epoch_ms, Some(1_000 + 3_600_000));
}

#[test]
fn icfr_classification_flags_a_key_financial_process_in_scope() {
    let classification = classify_icfr_subject(&IcfrSubject { subject: SUBJECT.to_string(), processes: vec!["payroll".to_string()], material_accounts: vec![], affects_financial_reporting: false });
    assert!(classification.in_scope);
    assert_eq!(classification.reasons, vec!["key-financial-process".to_string()]);
}

#[test]
fn icfr_classification_leaves_an_unrelated_process_out_of_scope() {
    let classification = classify_icfr_subject(&IcfrSubject { subject: SUBJECT.to_string(), processes: vec!["marketing-analytics".to_string()], material_accounts: vec![], affects_financial_reporting: false });
    assert!(!classification.in_scope);
    assert!(classification.reasons.is_empty());
}

#[test]
fn segregation_of_duty_detects_a_real_incompatible_role_pair() {
    let assignments = vec![
        RoleAssignment { principal: "alice".to_string(), role: "payments-initiate".to_string() },
        RoleAssignment { principal: "alice".to_string(), role: "payments-approve".to_string() },
        RoleAssignment { principal: "bob".to_string(), role: "payments-initiate".to_string() },
    ];
    let incompatible = vec![("payments-initiate".to_string(), "payments-approve".to_string())];

    let violations = detect_segregation_of_duty_violations(&assignments, &incompatible);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].principal, "alice");
    assert_eq!(violations[0].roles, ("payments-initiate".to_string(), "payments-approve".to_string()));
}

#[test]
fn board_requirements_exposes_the_full_generated_forty_control_profile() {
    assert_eq!(board_requirements().len(), 40);
}
