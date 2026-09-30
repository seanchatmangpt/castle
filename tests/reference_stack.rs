use castle::reference_stack::*;

fn happy() -> Result<ProcessFeedback, String> {
    let hook = project_knowledge_hook(&fibo::delta(), "hook-1")?;
    let intent = derive_intent(&hook, "intent-1");
    let standing = record_standing(&intent, true);
    let real = realize(&standing, "reactor", &[fibo::BRCE_PATH])?;
    let handoff = hand_off_to_brce(&real)?;
    assert_eq!(handoff.authority_ceiling(), "CONSTRUCT");
    assert_eq!(handoff.actuation(), "NONE");
    let post = observe_postcondition(&handoff, &real, "independent-observer", true)?;
    let receipt = reference_receipt(&post, "rcpt-1", "seanchatmangpt/castle")?;
    Ok(feed_back(&receipt, true))
}

#[test]
fn fibo_case_runs_through_current_owners_without_new_authority() {
    assert!(happy().unwrap().conforms);
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
fn postcondition_must_be_independent_and_receipt_castle_minted() {
    let hook = project_knowledge_hook(&fibo::delta(), "h").unwrap();
    let st = record_standing(&derive_intent(&hook, "i"), true);
    let r = realize(&st, "reactor", &[fibo::BRCE_PATH]).unwrap();
    let h = hand_off_to_brce(&r).unwrap();
    assert_eq!(observe_postcondition(&h, &r, "reactor", true).unwrap_err(), "REFUSED:POSTCONDITION_NOT_INDEPENDENT");
    let p = observe_postcondition(&h, &r, "obs", true).unwrap();
    assert_eq!(reference_receipt(&p, "x", "seanchatmangpt/beam4pm").unwrap_err(), "REFUSED:RECEIPT_NOT_CASTLE_MINTED");
    let bad = observe_postcondition(&h, &r, "obs", false).unwrap();
    assert_eq!(reference_receipt(&bad, "x", "seanchatmangpt/castle").unwrap_err(), "REFUSED:POSTCONDITION_FAILED");
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
