//! Reference-stack chain: the one lawful ordering of the ecosystem owners.
//!
//! GraphLaw semantic delta -> GraphLaw-native Knowledge Hook projection -> powerless SA2A
//! intent -> Affidavit standing -> Reactor realization -> CASTLE BRCE handoff ->
//! independent postcondition -> receipt/OCEL reference -> Beam4PM feedback.
//!
//! This module owns *ordering and non-ownership only*. It never mints a receipt, never
//! actuates, never evaluates semantics, and never replays: each of those stays with the
//! owner named in [`OWNERS`]. The chain ends at authority ceiling `CONSTRUCT`; the
//! consequential step is a handoff to `admit_construct_for_do`, not a `DO`.

use crate::blake3::blake3_hex_utf8;

/// Maximum authority this chain may carry.
pub const AUTHORITY_CEILING: &str = "CONSTRUCT";

/// Irreducible capability -> sole owner. A projection may not claim any of these.
pub const OWNERS: &[(&str, &str)] = &[
    ("SEMANTIC_AUTHORITY", "seanchatmangpt/graphlaw"),
    ("KNOWLEDGE_HOOK_TRIGGER", "seanchatmangpt/graphlaw"),
    ("STANDING", "seanchatmangpt/affidavit"),
    ("RUNTIME_EXISTENCE", "seanchatmangpt/xaas"),
    ("CONSEQUENTIAL_AUTHORITY", "seanchatmangpt/castle"),
    ("RECEIPT_TRUTH", "seanchatmangpt/castle"),
    ("REPLAY_SEMANTICS", "seanchatmangpt/castle"),
    ("PROCESS_FEEDBACK", "seanchatmangpt/beam4pm"),
];

/// Owner of an irreducible capability, if it is one.
#[must_use]
pub fn owner_of(capability: &str) -> Option<&'static str> {
    OWNERS.iter().find(|(c, _)| *c == capability).map(|(_, o)| *o)
}

/// Falsifier gate: a projection claiming an owned capability is refused.
///
/// # Errors
/// `REFUSED:PROJECTION_OWNS_<CAPABILITY>` when `projection` is not the capability's owner.
pub fn refuse_ownership_claim(projection: &str, capability: &str) -> Result<(), String> {
    match owner_of(capability) {
        Some(owner) if owner != projection => {
            Err(format!("REFUSED:PROJECTION_OWNS_{capability}"))
        }
        _ => Ok(()),
    }
}

fn digest(parts: &[&str]) -> String {
    blake3_hex_utf8(&parts.join("\u{1f}"))
}

/// A semantic change observed by GraphLaw (input; not evaluated here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDelta {
    pub graph_before: String,
    pub graph_after: String,
    pub predicate: String,
}

/// GraphLaw-native Knowledge Hook projection. Fires only from a delta; carries no authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeHookProjection {
    pub hook_id: String,
    pub delta_digest: String,
}

/// SA2A intent: powerless by construction (no actuation field exists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sa2aIntent {
    pub intent_id: String,
    pub hook_digest: String,
}

/// Affidavit standing verdict over an intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffidavitStanding {
    pub intent_digest: String,
    pub standing: bool,
}

/// Reactor realization plan. Realizes a *request*; cannot itself act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Realization {
    pub realizer_id: String,
    pub standing_digest: String,
    pub consequential_paths: Vec<String>,
}

/// The single handoff into CASTLE BRCE. Ceiling is fixed at `CONSTRUCT`; actuation is `NONE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrceHandoff {
    pub realization_digest: String,
    pub path: String,
}

impl BrceHandoff {
    #[must_use]
    pub const fn authority_ceiling(&self) -> &'static str {
        AUTHORITY_CEILING
    }
    #[must_use]
    pub const fn actuation(&self) -> &'static str {
        "NONE"
    }
}

/// Independent postcondition observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Postcondition {
    pub observer_id: String,
    pub handoff_digest: String,
    pub holds: bool,
}

/// Reference to a CASTLE-minted receipt/OCEL log. This module never mints one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptRef {
    pub receipt_id: String,
    pub minted_by: String,
    pub postcondition_digest: String,
}

/// Beam4PM feedback derived from the receipt reference; grants nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessFeedback {
    pub receipt_id: String,
    pub conforms: bool,
}

impl SemanticDelta {
    #[must_use]
    pub fn digest(&self) -> String {
        digest(&[&self.graph_before, &self.graph_after, &self.predicate])
    }
}

/// # Errors
/// `REFUSED:NO_SEMANTIC_DELTA` when before/after graphs are identical.
pub fn project_knowledge_hook(delta: &SemanticDelta, hook_id: &str) -> Result<KnowledgeHookProjection, String> {
    if delta.graph_before == delta.graph_after {
        return Err("REFUSED:NO_SEMANTIC_DELTA".to_string());
    }
    Ok(KnowledgeHookProjection { hook_id: hook_id.to_string(), delta_digest: delta.digest() })
}

#[must_use]
pub fn derive_intent(hook: &KnowledgeHookProjection, intent_id: &str) -> Sa2aIntent {
    Sa2aIntent { intent_id: intent_id.to_string(), hook_digest: digest(&[&hook.hook_id, &hook.delta_digest]) }
}

#[must_use]
pub fn record_standing(intent: &Sa2aIntent, standing: bool) -> AffidavitStanding {
    AffidavitStanding { intent_digest: digest(&[&intent.intent_id, &intent.hook_digest]), standing }
}

/// # Errors
/// `REFUSED:NO_STANDING` without standing; `REFUSED:NOT_EXACTLY_ONE_CONSEQUENTIAL_PATH`
/// unless exactly one path is declared.
pub fn realize(standing: &AffidavitStanding, realizer_id: &str, consequential_paths: &[&str]) -> Result<Realization, String> {
    if !standing.standing {
        return Err("REFUSED:NO_STANDING".to_string());
    }
    if consequential_paths.len() != 1 {
        return Err("REFUSED:NOT_EXACTLY_ONE_CONSEQUENTIAL_PATH".to_string());
    }
    Ok(Realization {
        realizer_id: realizer_id.to_string(),
        standing_digest: digest(&[&standing.intent_digest, if standing.standing { "1" } else { "0" }]),
        consequential_paths: consequential_paths.iter().map(ToString::to_string).collect(),
    })
}

/// # Errors
/// `REFUSED:PATH_NOT_BRCE` unless the sole path terminates in CASTLE BRCE.
pub fn hand_off_to_brce(realization: &Realization) -> Result<BrceHandoff, String> {
    let path = &realization.consequential_paths[0];
    if !path.ends_with("castle::brce") {
        return Err("REFUSED:PATH_NOT_BRCE".to_string());
    }
    Ok(BrceHandoff {
        realization_digest: digest(&[&realization.realizer_id, &realization.standing_digest, path]),
        path: path.clone(),
    })
}

/// # Errors
/// `REFUSED:POSTCONDITION_NOT_INDEPENDENT` when the observer is the realizer.
pub fn observe_postcondition(handoff: &BrceHandoff, realization: &Realization, observer_id: &str, holds: bool) -> Result<Postcondition, String> {
    if observer_id == realization.realizer_id {
        return Err("REFUSED:POSTCONDITION_NOT_INDEPENDENT".to_string());
    }
    Ok(Postcondition {
        observer_id: observer_id.to_string(),
        handoff_digest: digest(&[&handoff.realization_digest, &handoff.path]),
        holds,
    })
}

/// # Errors
/// `REFUSED:RECEIPT_NOT_CASTLE_MINTED` unless minted by CASTLE; `REFUSED:POSTCONDITION_FAILED`
/// when the postcondition does not hold.
pub fn reference_receipt(post: &Postcondition, receipt_id: &str, minted_by: &str) -> Result<ReceiptRef, String> {
    if owner_of("RECEIPT_TRUTH") != Some(minted_by) {
        return Err("REFUSED:RECEIPT_NOT_CASTLE_MINTED".to_string());
    }
    if !post.holds {
        return Err("REFUSED:POSTCONDITION_FAILED".to_string());
    }
    Ok(ReceiptRef {
        receipt_id: receipt_id.to_string(),
        minted_by: minted_by.to_string(),
        postcondition_digest: digest(&[&post.observer_id, &post.handoff_digest]),
    })
}

#[must_use]
pub fn feed_back(receipt: &ReceiptRef, conforms: bool) -> ProcessFeedback {
    ProcessFeedback { receipt_id: receipt.receipt_id.clone(), conforms }
}

/// Synthetic FIBO payment-authorization reference case (no real financial data).
pub mod fibo {
    use super::{SemanticDelta};

    pub const CASE_ID: &str = "FIBO-SYNTHETIC-PAYMENT-AUTHORIZATION";
    pub const HOOK_PREDICATE: &str = "fibo-fnd:PaymentObligation/ExceedsAuthorizedLimit";
    pub const BRCE_PATH: &str = "reactor::payment_release->castle::brce";

    #[must_use]
    pub fn delta() -> SemanticDelta {
        SemanticDelta {
            graph_before: "synthetic:ledger@t0 limit=10000 pending=9000".to_string(),
            graph_after: "synthetic:ledger@t1 limit=10000 pending=12000".to_string(),
            predicate: HOOK_PREDICATE.to_string(),
        }
    }
}
