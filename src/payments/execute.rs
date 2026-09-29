use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::json;

use crate::castle::{
    admit_construct_for_do, ConstructAdmission, execute_powl_with_gym_act, manufacture_construct_capability, Blake3Provider,
    ConstructRequest, ConstructTrustPolicy, DoAuthorizationContext, PowlActivity, PowlProcess, ReceiptSigner,
    ReceiptVerifier, TestEnvelope, WorldState,
};
use crate::v26_8_18::{BrceGymActAdapter, ReleaseStanding};

use super::adapter::{PaymentGymActAdapter, StepOutcome, T_POST, T_RESERVE};
use super::admission::PaymentAdmission;
use super::claim_store::{ClaimState, ClaimStore};
use super::ledger::{ActuationToken, LedgerEntry, LedgerPort};
use super::policy::SpendPolicy;
use super::refusal::{self, PayResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentStanding {
    /// Settlement observed, BRCE PREPARE+OUTCOME durable, claim `Executed`.
    Settled,
    /// Definitely not posted; budget released.
    Refused,
    /// Ambiguous. Blind retry is forbidden; run `reconcile`.
    UnknownOutcome,
}

#[derive(Debug, Clone)]
pub struct PaymentExecution {
    pub standing: PaymentStanding,
    pub effect_digest: String,
    pub construct_digest: Option<String>,
    pub ocel_receipt_digest: Option<String>,
    pub brce_prepare_receipt_digests: Vec<String>,
    pub brce_outcome_receipt_digests: Vec<String>,
    pub ledger_entry: Option<LedgerEntry>,
    pub detail: String,
}

pub struct ExecutionContext<'a> {
    pub blake3: &'a dyn Blake3Provider,
    pub signer: &'a dyn ReceiptSigner,
    pub verifier: &'a dyn ReceiptVerifier,
    pub allowed_authorities: BTreeSet<String>,
    pub journal_root: PathBuf,
    pub now_epoch_ms: i64,
    pub ledger: &'a dyn LedgerPort,
    pub claims: &'a ClaimStore,
    pub policy: &'a SpendPolicy,
}

fn subject_of(admission: &PaymentAdmission) -> String {
    format!("payment:{}", admission.effect().digest())
}

fn process_of(admission: &PaymentAdmission) -> PowlProcess {
    PowlProcess {
        id: subject_of(admission),
        goal_id: "payments.settle".to_string(),
        activities: vec![
            PowlActivity { id: "a1".to_string(), transition_id: T_RESERVE.to_string(), predecessors: vec![] },
            PowlActivity { id: "a2".to_string(), transition_id: T_POST.to_string(), predecessors: vec!["a1".to_string()] },
        ],
    }
}

fn envelope_of(admission: &PaymentAdmission, now_epoch_ms: i64) -> TestEnvelope {
    TestEnvelope {
        system_id: subject_of(admission),
        allowed_transition_ids: BTreeSet::from([T_RESERVE.to_string(), T_POST.to_string()]),
        max_steps: 4,
        expires_at_epoch_ms: now_epoch_ms + 60_000,
    }
}

/// CONSTRUCT → admit → durable-BRCE DO → observe → claim settlement.
/// The sole actuation path is `castle::execute_powl_with_gym_act`.
pub async fn execute_payment(admission: PaymentAdmission, ctx: &ExecutionContext<'_>) -> PayResult<PaymentExecution> {
    let digest = admission.effect().digest().to_string();
    let subject = subject_of(&admission);

    let refuse_before_do = |detail: String| -> PayResult<PaymentExecution> {
        ctx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Refused, None, &detail)?;
        Ok(PaymentExecution {
            standing: PaymentStanding::Refused,
            effect_digest: digest.clone(),
            construct_digest: None,
            ocel_receipt_digest: None,
            brce_prepare_receipt_digests: vec![],
            brce_outcome_receipt_digests: vec![],
            ledger_entry: None,
            detail,
        })
    };

    // Single-use + freshness: DO may only start while this exact admission still
    // owns a Reserved claim in the caller's store (no stale/foreign admissions).
    match ctx.claims.get(&digest)? {
        Some(c) if c.state == ClaimState::Reserved && c.generation == admission.generation() => {}
        _ => return Err(refusal::CLAIM_NOT_RESERVED.to_string()),
    }

    let (construct_admission, process, envelope) = match build_construct(&admission, ctx) {
        Ok(v) => v,
        Err(e) => return refuse_before_do(e),
    };
    let now = ctx.now_epoch_ms;
    let construct_digest = construct_admission.construct_digest.clone();
    let token = ActuationToken::from_construct(&construct_admission);

    let inner = PaymentGymActAdapter::new(ctx.ledger, &admission, subject.clone(), token);
    let brce = BrceGymActAdapter::new_durable(&inner, ctx.blake3, ctx.signer, ctx.journal_root.clone());
    let state = WorldState { system_id: subject, facts: BTreeSet::new() };
    let result = execute_powl_with_gym_act(
        &process,
        &state,
        &envelope,
        &brce,
        DoAuthorizationContext {
            admission: &construct_admission,
            blake3: ctx.blake3,
            receipt_signer: ctx.signer,
            now: Box::new(move || now),
        },
    )
    .await;
    let journal = brce.journal();
    let outcomes = inner.outcomes();
    let posted = outcomes.iter().find_map(|o| if let StepOutcome::Posted(e) = o { Some(e.clone()) } else { None });
    let any_indefinite = outcomes.iter().any(|o| matches!(o, StepOutcome::Refused { definite: false, .. }));
    let prepare: Vec<String> = journal.iter().map(|r| r.prepare_receipt.receipt_digest.clone()).collect();
    let outcome: Vec<String> = journal
        .iter()
        .filter_map(|r| r.outcome_receipt.as_ref().map(|x| x.receipt_digest.clone()))
        .collect();
    let journal_complete = journal.len() == 2
        && journal.iter().all(|r| r.standing == ReleaseStanding::Alive && r.outcome_receipt.is_some());

    let (standing, detail, ocel) = match (result, &posted) {
        (Ok(log), Some(_)) if journal_complete => (PaymentStanding::Settled, "SETTLED".to_string(), Some(log.receipt.receipt_digest)),
        (Ok(_), _) => (PaymentStanding::UnknownOutcome, refusal::OUTCOME_UNKNOWN.to_string(), None),
        (Err(e), Some(_)) => (PaymentStanding::UnknownOutcome, format!("{}:{e}", refusal::OUTCOME_UNKNOWN), None),
        (Err(e), None) if any_indefinite => (PaymentStanding::UnknownOutcome, format!("{}:{e}", refusal::OUTCOME_UNKNOWN), None),
        (Err(e), None) => (PaymentStanding::Refused, e, None),
    };

    let to = match standing {
        PaymentStanding::Settled => ClaimState::Executed,
        PaymentStanding::Refused => ClaimState::Refused,
        PaymentStanding::UnknownOutcome => ClaimState::UnknownOutcome,
    };
    ctx.claims.transition(&digest, &[ClaimState::Reserved], to, Some(&construct_digest), &detail)?;

    Ok(PaymentExecution {
        standing,
        effect_digest: digest,
        construct_digest: Some(construct_digest),
        ocel_receipt_digest: ocel,
        brce_prepare_receipt_digests: prepare,
        brce_outcome_receipt_digests: outcome,
        ledger_entry: posted,
        detail,
    })
}

/// CONSTRUCT step alone: manufacture the receipted capability and admit it. Inert
/// (`CONSTRUCT != DO`): it only yields the sealed `ConstructAdmission` plus the
/// exact process/envelope that admission binds.
pub fn build_construct(
    admission: &PaymentAdmission,
    ctx: &ExecutionContext<'_>,
) -> PayResult<(ConstructAdmission, PowlProcess, TestEnvelope)> {
    let digest = admission.effect().digest().to_string();
    let subject = subject_of(admission);
    let process = process_of(admission);
    let envelope = envelope_of(admission, ctx.now_epoch_ms);
    let authority = admission.verification().audience.clone();
    let request = ConstructRequest {
        subject: subject.clone(),
        authority: authority.clone(),
        o_star: json!({
            "kind": "CASTLE_PAYMENT_O_STAR_V1",
            "effect_digest": digest,
            "principal": admission.effect().principal(),
            "verified_key_ids": admission.verification().verified_key_ids,
            "verified_custodian_ids": admission.verification().verified_custodian_ids,
            "policy_epoch": admission.verification().policy_epoch,
            "revocation_epoch": admission.verification().revocation_epoch,
            "generation": admission.generation(),
        }),
        config_graph: json!({ "spend_policy": ctx.policy.to_json() }),
        ontology: json!({ "kind": "CASTLE_PAYMENT_ONTOLOGY_V1", "capability": super::PAYMENT_CAPABILITY }),
        process: process.clone(),
        envelope: envelope.clone(),
    };
    let capability = match manufacture_construct_capability(request, ctx.blake3, ctx.signer) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    let policy = ConstructTrustPolicy {
        trusted_origin_key_ids: BTreeSet::from([ctx.signer.key_id().to_string()]),
        allowed_authorities: ctx.allowed_authorities.clone(),
    };
    let now = ctx.now_epoch_ms;
    let construct_admission =
        match admit_construct_for_do(&capability, &process, &envelope, ctx.blake3, ctx.verifier, &policy, || now) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };
    Ok((construct_admission, process, envelope))
}
