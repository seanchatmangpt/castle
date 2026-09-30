use castle::reference_stack::{project_fibo_payment, KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY};
use serde_json::json;

fn candidate() -> KnowledgeHookCandidate {
    KnowledgeHookCandidate {
        version: 1,
        exact_subject: json!({"payment_id":"001","account":"treasury"}),
        source_digest: format!("sha256:{}", "b".repeat(64)),
        replay_identity: "replay:fibo:001".to_string(),
        authority: "NONE".to_string(),
        capability: FIBO_PAYMENT_CAPABILITY.to_string(),
        principal: "principal:treasury".to_string(),
        payload: json!({"amount":"10.00","currency":"USD"}),
    }
}

#[test]
fn replay_reconstructs_same_candidate_identity_without_second_consequence() {
    let first = project_fibo_payment(candidate()).unwrap();
    let replay = project_fibo_payment(candidate()).unwrap();
    assert_eq!(first.effect_digest, replay.effect_digest);
    assert_eq!(first.replay_identity, replay.replay_identity);
    assert_eq!(first.prepared_effect, replay.prepared_effect);
}
