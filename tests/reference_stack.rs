use std::collections::BTreeSet;

use castle::castle::*;
use castle::reference_stack::*;
use ed25519_dalek::{Signer as _, SigningKey, Verifier as _, VerifyingKey};
use serde_json::json;

struct RealBlake3;
impl Blake3Provider for RealBlake3 {
    fn digest_utf8(&self, input: &str) -> String {
        blake3::hash(input.as_bytes()).to_hex().to_string()
    }
}

fn hex_decode(h: &str) -> Vec<u8> {
    (0..h.len()).step_by(2).map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap()).collect()
}

struct Signer(SigningKey);
impl ReceiptSigner for Signer {
    fn key_id(&self) -> &str {
        "construct-root"
    }
    fn sign_digest(&self, d: &str) -> String {
        self.0.sign(&hex_decode(d)).to_bytes().iter().map(|b| format!("{b:02x}")).collect()
    }
}

struct Verifier(VerifyingKey);
impl ReceiptVerifier for Verifier {
    fn verify_digest(&self, key_id: &str, d: &str, sig: &str) -> bool {
        let Ok(bytes): Result<[u8; 64], _> = hex_decode(sig).try_into() else { return false };
        key_id == "construct-root" && self.0.verify(&hex_decode(d), &ed25519_dalek::Signature::from_bytes(&bytes)).is_ok()
    }
}

struct Gym;
#[async_trait::async_trait]
impl GymActAdapter for Gym {
    async fn execute(&self, a: &PowlActivity, _s: &WorldState, _p: &ActuationPermit) -> GymActResult {
        GymActResult {
            transition_id: a.transition_id.clone(),
            status: GymActStatus::Observed,
            objects: vec![OcelObject { id: format!("object:{}", a.transition_id), kind: "SyntheticPayment".to_string() }],
            attributes: Default::default(),
        }
    }
}

fn chain() -> (Realization, BrceHandoff) {
    let hook = project_knowledge_hook(&fibo::delta(), "hook-1").unwrap();
    let standing = record_standing(&derive_intent(&hook, "intent-1"), true);
    let real = realize(&standing, "reactor", &[fibo::BRCE_PATH]).unwrap();
    let handoff = hand_off_to_brce(&real).unwrap();
    assert_eq!((handoff.authority_ceiling(), handoff.actuation()), ("CONSTRUCT", "NONE"));
    (real, handoff)
}

fn process_and_envelope() -> (PowlProcess, TestEnvelope) {
    let rule = TransitionRule { id: "release-payment".into(), preconditions: vec![], effects: vec!["goal:payment-released".into()], cost: None, planner_hint: None };
    let goal = AdversarialGoal { id: "g".into(), predicate: "goal:payment-released".into(), consequence: 1 };
    let vuln = derive_vulnerabilities(&goal, &[rule.clone()], 2).remove(0);
    let process = compile_witness_to_powl("fibo-payment", &vuln, &[rule]);
    let envelope = TestEnvelope { system_id: "system:fibo-synthetic".into(), allowed_transition_ids: BTreeSet::from(["release-payment".to_string()]), max_steps: 1, expires_at_epoch_ms: 10_000 };
    (process, envelope)
}

/// Full chain over the REAL castle CONSTRUCT -> admission -> GymAct -> OCEL receipt path.
#[tokio::test]
async fn fibo_case_runs_through_real_castle_brce_without_new_authority() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let (signer, verifier, b3) = (Signer(key.clone()), Verifier(key.verifying_key()), RealBlake3);
    let (real, handoff) = chain();
    let (process, envelope) = process_and_envelope();
    let cap = manufacture_construct_capability(
        construct_request(&handoff, &real, "defensive-test", json!({"zeroUnreceiptedActuation": true}), json!({"version": "castle-pack-v1"}), process.clone(), envelope.clone()),
        &b3,
        &signer,
    )
    .unwrap();
    let policy = ConstructTrustPolicy { trusted_origin_key_ids: BTreeSet::from(["construct-root".to_string()]), allowed_authorities: BTreeSet::from(["defensive-test".to_string()]) };
    let admission = admit_construct_for_do(&cap, &process, &envelope, &b3, &verifier, &policy, || 1).unwrap();
    let state = WorldState { system_id: envelope.system_id.clone(), facts: BTreeSet::new() };
    let log = execute_powl_with_gym_act(&process, &state, &envelope, &Gym, DoAuthorizationContext { admission: &admission, blake3: &b3, receipt_signer: &signer, now: Box::new(|| 5) }).await.unwrap();

    let post = observe_postcondition(&handoff, &real, "independent-observer", true).unwrap();
    let receipt = reference_receipt(&post, &handoff, &real, &admission, &log, &b3).unwrap();
    assert_eq!(receipt.receipt_digest, log.receipt.receipt_digest);
    assert!(feed_back(&receipt, true).conforms);

    // A different chain's provenance is refused against this admission.
    let hook2 = project_knowledge_hook(&fibo::delta(), "hook-OTHER").unwrap();
    let st2 = record_standing(&derive_intent(&hook2, "intent-1"), true);
    let real2 = realize(&st2, "reactor", &[fibo::BRCE_PATH]).unwrap();
    let handoff2 = hand_off_to_brce(&real2).unwrap();
    assert_eq!(reference_receipt(&post, &handoff2, &real2, &admission, &log, &b3).unwrap_err(), "REFUSED:PROVENANCE_NOT_BOUND");
    // A failed postcondition never yields a receipt reference.
    let bad = observe_postcondition(&handoff, &real, "independent-observer", false).unwrap();
    assert_eq!(reference_receipt(&bad, &handoff, &real, &admission, &log, &b3).unwrap_err(), "REFUSED:POSTCONDITION_FAILED");
}

#[test]
fn no_delta_no_hook() {
    let mut d = fibo::delta();
    d.graph_after = d.graph_before.clone();
    assert_eq!(project_knowledge_hook(&d, "h").unwrap_err(), "REFUSED:NO_SEMANTIC_DELTA");
}

#[test]
fn no_standing_no_realization() {
    let hook = project_knowledge_hook(&fibo::delta(), "h").unwrap();
    let st = record_standing(&derive_intent(&hook, "i"), false);
    assert_eq!(realize(&st, "r", &[fibo::BRCE_PATH]).unwrap_err(), "REFUSED:NO_STANDING");
}

#[test]
fn exactly_one_consequential_path_terminating_in_brce() {
    let hook = project_knowledge_hook(&fibo::delta(), "h").unwrap();
    let st = record_standing(&derive_intent(&hook, "i"), true);
    assert!(realize(&st, "r", &[]).is_err());
    assert!(realize(&st, "r", &[fibo::BRCE_PATH, "reactor::other->castle::brce"]).is_err());
    let r = realize(&st, "r", &["reactor::direct_write"]).unwrap();
    assert_eq!(hand_off_to_brce(&r).unwrap_err(), "REFUSED:PATH_NOT_BRCE");
}

#[test]
fn postcondition_must_be_independent() {
    let hook = project_knowledge_hook(&fibo::delta(), "h").unwrap();
    let st = record_standing(&derive_intent(&hook, "i"), true);
    let r = realize(&st, "reactor", &[fibo::BRCE_PATH]).unwrap();
    let h = hand_off_to_brce(&r).unwrap();
    assert_eq!(observe_postcondition(&h, &r, "reactor", true).unwrap_err(), "REFUSED:POSTCONDITION_NOT_INDEPENDENT");
}

#[test]
fn falsifier_projection_cannot_own_semantic_consequential_receipt_or_replay_authority() {
    for cap in ["SEMANTIC_AUTHORITY", "CONSEQUENTIAL_AUTHORITY", "RECEIPT_TRUTH", "REPLAY_SEMANTICS", "STANDING"] {
        for projection in ["seanchatmangpt/ash_r2rml", "seanchatmangpt/xaas", "seanchatmangpt/beam4pm"] {
            let owner = owner_of(cap).unwrap();
            let r = refuse_ownership_claim(projection, cap);
            if projection == owner { assert!(r.is_ok()) } else { assert_eq!(r.unwrap_err(), format!("REFUSED:PROJECTION_OWNS_{cap}")) }
        }
    }
    assert!(refuse_ownership_claim("seanchatmangpt/castle", "CONSEQUENTIAL_AUTHORITY").is_ok());
}

#[test]
fn every_owned_capability_has_exactly_one_owner() {
    let mut seen = std::collections::BTreeSet::new();
    for (c, _) in OWNERS { assert!(seen.insert(*c), "duplicate owner for {c}"); }
}
