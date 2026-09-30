use castle::reference_stack::{
    bind_verification_receipt, canonical_owners, project_fibo_payment, KnowledgeHookCandidate,
    ECOSYSTEM_REFERENCE_HEAD, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT,
    GRAPHLAW_REFERENCE_HEAD, INHERITED_COURTS,
};
use castle::sa2a_security::VerificationReceipt;
use serde_json::json;

fn candidate() -> KnowledgeHookCandidate {
    KnowledgeHookCandidate {
        version: 1,
        exact_subject: json!("system:fibo:payment:001"),
        source_digest: format!("sha256:{}", "a".repeat(64)),
        replay_identity: "replay:fibo:payment:001".to_string(),
        authority: "NONE".to_string(),
        capability: FIBO_PAYMENT_CAPABILITY.to_string(),
        principal: "principal:synthetic-treasury".to_string(),
        payload: json!({"amount":"1250.00","currency":"USD","beneficiary":"counterparty:42"}),
    }
}

#[test]
fn fibo_delta_reuses_existing_prepared_effect_identity() {
    let projection = project_fibo_payment(candidate()).unwrap();
    assert_eq!(projection.hook_contract, GRAPHLAW_HOOK_CONTRACT);
    assert_eq!(projection.prepared_effect.capability, FIBO_PAYMENT_CAPABILITY);
    assert_eq!(projection.effect_digest, projection.prepared_effect.digest().unwrap());
}

#[test]
fn hook_authority_smuggling_is_refused_before_castle() {
    let mut input = candidate();
    input.authority = "DO".to_string();
    assert_eq!(
        project_fibo_payment(input).unwrap_err(),
        "REFUSED:KNOWLEDGE_HOOK_AUTHORITY"
    );
}

#[test]
fn independent_verification_receipt_binds_existing_effect() {
    let projection = project_fibo_payment(candidate()).unwrap();
    let receipt = VerificationReceipt {
        effect_digest: projection.effect_digest.clone(),
        principal: projection.prepared_effect.principal.clone(),
        policy_epoch: 7,
        revocation_epoch: 3,
        generation: 11,
        audience: "castle:brce".to_string(),
        verified_key_ids: vec!["key:authority-a".to_string(), "key:authority-b".to_string()],
        verified_custodian_ids: vec!["custodian:a".to_string(), "custodian:b".to_string()],
    };
    let evidence = bind_verification_receipt(&projection, &receipt).unwrap();
    assert_eq!(evidence.effect_digest, projection.effect_digest);
    assert_eq!(evidence.signer_count, 2);
}

#[test]
fn compounding_stack_has_one_owner_per_meaning() {
    let owners = canonical_owners();
    assert_eq!(owners.iter().filter(|o| o.layer == "semantic-law").count(), 1);
    assert_eq!(owners.iter().filter(|o| o.layer == "consequence-thin-waist").count(), 1);
    assert_eq!(owners.iter().filter(|o| o.layer == "product-constitution").count(), 1);
    assert!(INHERITED_COURTS.len() >= 8);
    assert_eq!(GRAPHLAW_REFERENCE_HEAD, "48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc");
    assert_eq!(ECOSYSTEM_REFERENCE_HEAD, "bebffdeb2ea4dd6eace31d69298bfb51a073b265");
}
