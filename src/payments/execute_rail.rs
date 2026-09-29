//! Rail-mediated execution. `submit_via_rail` places a ledger HOLD and hands an ISO 20022
//! pain.001 instruction to a `RailActuator`, both inside the exclusive DO path
//! (`castle::execute_powl_with_gym_act` under durable BRCE). It NEVER writes a ledger
//! entry: value moves in the books only in `finalize_via_rail`, from observed rail
//! finality (`settlement::apply_finality`). An ambiguous rail outcome is `UnknownOutcome`,
//! the hold is retained, and the effect is never resubmitted blindly.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::json;

use crate::castle::{
    execute_powl_with_gym_act, ActuationPermit, DoAuthorizationContext, GymActAdapter, GymActResult, GymActStatus,
    OcelObject, PowlActivity, PowlProcess, WorldState,
};
use crate::sa2a_security::encoding::sha256_tagged;
use crate::v26_8_18::BrceGymActAdapter;

use super::admission::PaymentAdmission;
use super::claim_store::{ClaimState, ClaimStore};
use super::execute::{build_construct_with, subject_of, ExecutionContext};
use super::iso20022::pain001_customer_credit_transfer;
use super::ledger::{ActuationToken, LedgerError, LedgerPort};
use super::pee::{EffectBindings, PreparedEconomicEffect};
use super::rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus};
use super::refusal::{self, refuse, PayResult};
use super::settlement::{
    apply_finality, apply_rail_report, observe_rail, status_evidence, FinalityEvidence, FinalityKind, FinalityOutcome,
    PAYMENT_NOT_SUBMITTED,
};

pub const T_HOLD: &str = "payments.hold";
pub const T_RAIL_SUBMIT: &str = "payments.rail_submit";

pub const RAIL_NEVER_SAW_EFFECT: &str = "RAIL_NEVER_SAW_EFFECT";
pub const RAIL_NEVER_SUBMITTED: &str = "RAIL_NEVER_SUBMITTED";
pub const RAIL_HOLD_UNAVAILABLE: &str = "REFUSED:PAYMENT_RAIL_HOLD_UNAVAILABLE";
pub const RAIL_TIME_INVALID: &str = "REFUSED:PAYMENT_RAIL_TIME_INVALID";

pub const NOT_ABANDONABLE: &str = "REFUSED:PAYMENT_NOT_ABANDONABLE";
const LOCAL_EVIDENCE_DOMAIN: &[u8] = b"CASTLE-RAIL-LOCAL-EVIDENCE-V1";

#[derive(Debug, Clone)]
pub struct RailExecutionParams {
    pub bindings: EffectBindings,
    pub created_at_iso: String,
    pub debtor_name: String,
    pub creditor_name: String,
    pub debtor_agent_bic: String,
    pub creditor_agent_bic: String,
    pub rail_profile: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailStandingAfterSubmit {
    /// Rail acknowledged; funds held; finality not yet observed.
    Submitted,
    /// Definitely not sent (or definitely rejected); hold released or never placed.
    Refused,
    /// Ambiguous; hold retained; only `finalize_via_rail` may resolve it.
    UnknownOutcome,
}

#[derive(Debug, Clone)]
pub struct RailSubmission {
    /// Claim key: the payment effect digest.
    pub effect_id: String,
    pub correlation_id: String,
    pub standing: RailStandingAfterSubmit,
    pub construct_digest: Option<String>,
    pub brce_prepare_receipt_digests: Vec<String>,
    pub brce_outcome_receipt_digests: Vec<String>,
    pub pee: PreparedEconomicEffect,
    pub payload_digest: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    PermitRefused,
    Held,
    HoldInsufficient,
    HoldUnavailable,
    RailCalled,
    RailAccepted,
    RailRejected(String),
    RailIndeterminate(String),
}

struct RailGymActAdapter<'a> {
    ledger: &'a dyn LedgerPort,
    rail: &'a dyn RailActuator,
    admission: &'a PaymentAdmission,
    subject: String,
    token: ActuationToken,
    instruction: RailInstruction,
    steps: Mutex<Vec<Step>>,
}

impl RailGymActAdapter<'_> {
    fn record(&self, s: Step) {
        if let Ok(mut v) = self.steps.lock() {
            v.push(s);
        }
    }

    fn steps(&self) -> Vec<Step> {
        self.steps.lock().map(|v| v.clone()).unwrap_or_default()
    }

    fn refused(&self, activity: &PowlActivity, code: &str, definite: bool) -> GymActResult {
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Refused,
            objects: Vec::new(),
            attributes: BTreeMap::from([("reason".to_string(), json!(code)), ("definite".to_string(), json!(definite))]),
        }
    }

    fn observed(&self, activity: &PowlActivity, extra: BTreeMap<String, serde_json::Value>) -> GymActResult {
        let e = self.admission.effect();
        let mut attributes = BTreeMap::from([
            ("effect_digest".to_string(), json!(e.digest())),
            ("correlation_id".to_string(), json!(self.instruction.correlation_id)),
            ("payload_digest".to_string(), json!(self.instruction.payload_digest())),
            ("amount_minor".to_string(), json!(e.money().minor.to_string())),
            ("currency".to_string(), json!(e.money().currency.code())),
        ]);
        attributes.extend(extra);
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Observed,
            objects: vec![
                OcelObject { id: format!("payment:{}", e.digest()), kind: "Payment".to_string() },
                OcelObject { id: format!("account:{}", e.payer()), kind: "Account".to_string() },
                OcelObject { id: format!("account:{}", e.payee()), kind: "Account".to_string() },
            ],
            attributes,
        }
    }
}

#[async_trait]
impl GymActAdapter for RailGymActAdapter<'_> {
    async fn execute(&self, activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        if permit.transition_id != activity.transition_id
            || permit.construct_digest != self.token.construct_digest()
            || permit.subject != self.subject
            || state.system_id != self.subject
        {
            self.record(Step::PermitRefused);
            return self.refused(activity, refusal::CAPABILITY_NOT_ADMITTED, true);
        }
        match activity.transition_id.as_str() {
            T_HOLD => match self.ledger.hold(self.admission, &self.token) {
                Ok(h) => {
                    self.record(Step::Held);
                    self.observed(activity, BTreeMap::from([("hold_amount_minor".to_string(), json!(h.amount_minor.to_string()))]))
                }
                Err(LedgerError::InsufficientFunds) => {
                    self.record(Step::HoldInsufficient);
                    self.refused(activity, refusal::INSUFFICIENT_FUNDS, true)
                }
                Err(LedgerError::Unavailable(_)) => {
                    self.record(Step::HoldUnavailable);
                    self.refused(activity, RAIL_HOLD_UNAVAILABLE, false)
                }
            },
            T_RAIL_SUBMIT => {
                if !self.steps().contains(&Step::Held) {
                    self.record(Step::PermitRefused);
                    return self.refused(activity, refusal::CAPABILITY_NOT_ADMITTED, true);
                }
                self.record(Step::RailCalled);
                match self.rail.submit(&self.instruction) {
                    Ok(RailAck::Accepted { correlation_id }) if correlation_id == self.instruction.correlation_id => {
                        self.record(Step::RailAccepted);
                        self.observed(activity, BTreeMap::from([("rail_ack".to_string(), json!("ACCEPTED"))]))
                    }
                    Ok(RailAck::Accepted { .. }) => {
                        // Ack for a different correlation is not an ack for this effect.
                        self.record(Step::RailIndeterminate("ACK_CORRELATION_MISMATCH".to_string()));
                        self.refused(activity, refusal::OUTCOME_UNKNOWN, false)
                    }
                    Ok(RailAck::Rejected { reason_code, .. }) => {
                        self.record(Step::RailRejected(reason_code.clone()));
                        self.refused(activity, "REFUSED:PAYMENT_RAIL_REJECTED", true)
                    }
                    Err(e) => {
                        let d = match e {
                            RailError::Timeout => "RAIL_TIMEOUT".to_string(),
                            RailError::Unavailable(u) => format!("RAIL_UNAVAILABLE:{u}"),
                        };
                        self.record(Step::RailIndeterminate(d));
                        self.refused(activity, refusal::OUTCOME_UNKNOWN, false)
                    }
                }
            }
            _ => {
                self.record(Step::PermitRefused);
                self.refused(activity, refusal::CAPABILITY_NOT_ADMITTED, true)
            }
        }
    }
}

fn rail_process(admission: &PaymentAdmission) -> PowlProcess {
    PowlProcess {
        id: subject_of(admission),
        goal_id: "payments.rail_submit".to_string(),
        activities: vec![
            PowlActivity { id: "a1".to_string(), transition_id: T_HOLD.to_string(), predecessors: vec![] },
            PowlActivity { id: "a2".to_string(), transition_id: T_RAIL_SUBMIT.to_string(), predecessors: vec!["a1".to_string()] },
        ],
    }
}

fn local_evidence(tag: &str, correlation: &str, detail: &str) -> String {
    sha256_tagged(LOCAL_EVIDENCE_DOMAIN, format!("{tag}|{correlation}|{detail}").as_bytes())
}

/// Definite non-submission after a claim is `Reserved`: release any hold via a sealed
/// rejection (through `Submitted`), else refuse the claim directly. Returns the standing
/// that matches the claim's actual resulting state.
fn release_unsubmitted(
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    digest: &str,
    correlation: &str,
    detail: &str,
) -> PayResult<RailStandingAfterSubmit> {
    match ledger.hold_of(digest) {
        Ok(None) => {
            claims.transition(digest, &[ClaimState::Reserved], ClaimState::Refused, None, detail)?;
            Ok(RailStandingAfterSubmit::Refused)
        }
        Ok(Some(_)) => {
            claims.transition(digest, &[ClaimState::Reserved], ClaimState::Submitted, None, "held-not-submitted")?;
            apply_finality(
                claims,
                ledger,
                &FinalityEvidence::new(
                    digest,
                    correlation,
                    local_evidence("NEVER_SUBMITTED", correlation, detail),
                    FinalityKind::Rejected,
                    RAIL_NEVER_SUBMITTED,
                ),
            )?;
            Ok(RailStandingAfterSubmit::Refused)
        }
        Err(_) => {
            claims.transition(digest, &[ClaimState::Reserved], ClaimState::UnknownOutcome, None, refusal::LEDGER_UNAVAILABLE)?;
            Ok(RailStandingAfterSubmit::UnknownOutcome)
        }
    }
}

/// CONSTRUCT -> admit -> durable-BRCE DO (`hold`, then `rail_submit`). Never writes a
/// ledger entry.
pub async fn submit_via_rail(
    admission: PaymentAdmission,
    params: &RailExecutionParams,
    ctx: &ExecutionContext<'_>,
    rail: &dyn RailActuator,
) -> PayResult<RailSubmission> {
    let digest = admission.effect().digest().to_string();
    let correlation = RailInstruction::correlation_id_for(&digest);
    let subject = subject_of(&admission);

    // Single-use + freshness, identical to `execute_payment`.
    match ctx.claims.get(&digest)? {
        Some(c) if c.state == ClaimState::Reserved && c.generation == admission.generation() => {}
        _ => return refuse(refusal::CLAIM_NOT_RESERVED),
    }

    // (a) PEE: seal, identity, freshness. Failure refuses before DO; no hold is placed.
    let pee_checked = PreparedEconomicEffect::seal(&admission, &params.bindings).and_then(|pee| {
        pee.verify_against(&admission)?;
        pee.verify_identity()?;
        let now = u64::try_from(ctx.now_epoch_ms).map_err(|_| RAIL_TIME_INVALID.to_string())?;
        pee.check_fresh(now)?;
        Ok(pee)
    });
    // (b) pain.001 payload.
    let built = pee_checked.and_then(|pee| {
        let xml = pain001_customer_credit_transfer(
            &admission,
            &params.created_at_iso,
            &params.debtor_name,
            &params.creditor_name,
            &params.debtor_agent_bic,
            &params.creditor_agent_bic,
        )?;
        Ok((pee, xml))
    });
    let (pee, xml) = match built {
        Ok(v) => v,
        Err(detail) => {
            ctx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Refused, None, &detail)?;
            // Re-seal is impossible here; surface the refusal as an error carrying the code.
            return Err(detail);
        }
    };
    let e = admission.effect();
    let instruction = RailInstruction {
        effect_id: digest.clone(),
        correlation_id: correlation.clone(),
        message_profile: params.rail_profile.clone(),
        payload: xml,
        amount_minor: e.money().minor,
        currency: e.money().currency,
        payer: e.payer().to_string(),
        payee: e.payee().to_string(),
    };
    let payload_digest = instruction.payload_digest();

    // (d) CONSTRUCT + admit.
    let process = rail_process(&admission);
    let allowed = BTreeSet::from([T_HOLD.to_string(), T_RAIL_SUBMIT.to_string()]);
    let (construct_admission, process, envelope) = match build_construct_with(&admission, ctx, process, allowed) {
        Ok(v) => v,
        Err(detail) => {
            ctx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Refused, None, &detail)?;
            return Err(detail);
        }
    };
    let construct_digest = construct_admission.construct_digest.clone();
    let token = ActuationToken::from_construct(&construct_admission);
    let now = ctx.now_epoch_ms;

    let inner = RailGymActAdapter {
        ledger: ctx.ledger,
        rail,
        admission: &admission,
        subject: subject.clone(),
        token,
        instruction,
        steps: Mutex::new(Vec::new()),
    };
    let brce = BrceGymActAdapter::new_durable(&inner, ctx.blake3, ctx.signer, ctx.journal_root.join("rail"));
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
    let prepare: Vec<String> = journal.iter().map(|r| r.prepare_receipt.receipt_digest.clone()).collect();
    let outcome: Vec<String> =
        journal.iter().filter_map(|r| r.outcome_receipt.as_ref().map(|x| x.receipt_digest.clone())).collect();
    let steps = inner.steps();
    let result_note = result.err().unwrap_or_default();

    // (e) outcome mapping from what the adapter actually observed.
    let held = steps.contains(&Step::Held);
    let (standing, detail) = if steps.contains(&Step::RailAccepted) {
        ctx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Submitted, Some(&construct_digest), "SUBMITTED")?;
        (RailStandingAfterSubmit::Submitted, "SUBMITTED".to_string())
    } else if let Some(reason) = steps.iter().find_map(|s| if let Step::RailRejected(r) = s { Some(r.clone()) } else { None }) {
        ctx.claims.transition(&digest, &[ClaimState::Reserved], ClaimState::Submitted, Some(&construct_digest), "rail-rejected")?;
        apply_finality(
            ctx.claims,
            ctx.ledger,
            &FinalityEvidence::new(&digest, &correlation, local_evidence("REJECT_ACK", &correlation, &reason), FinalityKind::Rejected, &reason),
        )?;
        (RailStandingAfterSubmit::Refused, format!("REFUSED:PAYMENT_RAIL_REJECTED:{reason}"))
    } else if steps.iter().any(|s| matches!(s, Step::RailIndeterminate(_) | Step::RailCalled)) {
        // The rail may have accepted. Hold retained; no resubmission.
        ctx.claims.transition(
            &digest,
            &[ClaimState::Reserved],
            ClaimState::UnknownOutcome,
            Some(&construct_digest),
            refusal::OUTCOME_UNKNOWN,
        )?;
        (RailStandingAfterSubmit::UnknownOutcome, refusal::OUTCOME_UNKNOWN.to_string())
    } else if held {
        // Hold placed but the rail was provably never called.
        let st = release_unsubmitted(ctx.claims, ctx.ledger, &digest, &correlation, &result_note)?;
        (st, format!("REFUSED:PAYMENT_NOT_SUBMITTED:{result_note}"))
    } else if steps.contains(&Step::HoldInsufficient) {
        ctx.claims.transition(
            &digest,
            &[ClaimState::Reserved],
            ClaimState::Refused,
            Some(&construct_digest),
            refusal::INSUFFICIENT_FUNDS,
        )?;
        (RailStandingAfterSubmit::Refused, refusal::INSUFFICIENT_FUNDS.to_string())
    } else {
        // HoldUnavailable / permit refusal / BRCE refusal before the adapter ran.
        let d = if result_note.is_empty() { RAIL_HOLD_UNAVAILABLE.to_string() } else { result_note.clone() };
        let st = release_unsubmitted(ctx.claims, ctx.ledger, &digest, &correlation, &d)?;
        (st, d)
    };

    Ok(RailSubmission {
        effect_id: digest,
        correlation_id: correlation,
        standing,
        construct_digest: Some(construct_digest),
        brce_prepare_receipt_digests: prepare,
        brce_outcome_receipt_digests: outcome,
        pee,
        payload_digest,
        detail,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizeResult {
    /// Rail has accepted but not reached finality.
    Pending,
    /// Finality applied (or already applied; idempotent), with the evidence digest used.
    Applied { outcome: FinalityOutcome, evidence_digest: String },
    /// Rail unreachable or uninformative; state unchanged.
    StillUnknown,
    /// Authoritative rail proved it never saw the effect; hold released, claim refused.
    ProvenAbsent,
}

/// Poll the rail and apply sealed finality. Safe to call repeatedly and against
/// duplicate/reordered reports: settlement is idempotent and refuses conflicts.
/// `RailStatus::Unknown` is proof of absence only if `rail.absence_is_authoritative()`.
pub fn finalize_via_rail(
    effect_digest: &str,
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
) -> PayResult<FinalizeResult> {
    let Some(mut claim) = claims.get(effect_digest)? else {
        return refuse(PAYMENT_NOT_SUBMITTED);
    };
    let correlation = RailInstruction::correlation_id_for(effect_digest);
    let Ok(report) = observe_rail(rail, effect_digest) else {
        return Ok(FinalizeResult::StillUnknown);
    };
    // A crash after the rail accepted but before Reserved->Submitted: the rail has seen it.
    if claim.state == ClaimState::Reserved && !matches!(report.status(), RailStatus::Unknown) {
        if let Err(e) = claims.transition(effect_digest, &[ClaimState::Reserved], ClaimState::Submitted, None, "rail-observed") {
            // A racing finalizer may have moved it already.
            match claims.get(effect_digest)? {
                Some(c) if c.state != ClaimState::Reserved => claim = c,
                _ => return Err(e),
            }
        } else {
            claim.state = ClaimState::Submitted;
        }
    }
    match report.status() {
        RailStatus::Settled { .. } | RailStatus::Rejected { .. } | RailStatus::Returned { .. } => {
            let outcome = apply_rail_report(claims, ledger, &report)?;
            Ok(FinalizeResult::Applied { outcome, evidence_digest: report.evidence_digest().to_string() })
        }
        RailStatus::Accepted => {
            if claim.state == ClaimState::UnknownOutcome {
                claims.transition(
                    effect_digest,
                    &[ClaimState::UnknownOutcome],
                    ClaimState::Submitted,
                    None,
                    "rail-accepted-observed",
                )?;
            }
            Ok(FinalizeResult::Pending)
        }
        RailStatus::Unknown => {
            if claim.state == ClaimState::UnknownOutcome && rail.absence_is_authoritative() {
                let d = status_evidence("NEVER_SAW", report.status())?;
                apply_finality(
                    claims,
                    ledger,
                    &FinalityEvidence::new(effect_digest, &correlation, d, FinalityKind::Rejected, RAIL_NEVER_SAW_EFFECT),
                )?;
                Ok(FinalizeResult::ProvenAbsent)
            } else {
                Ok(FinalizeResult::StillUnknown)
            }
        }
    }
}

/// Recovery for an orphan `Reserved` claim (crash between `ledger.hold` and the rail
/// submit; the admission is consumed by value so the effect cannot be resumed). If the
/// rail has seen the effect this delegates to `finalize_via_rail`. If an
/// authoritative-absence rail reports Unknown, the hold is released and the claim refused
/// (`ProvenAbsent`); a non-authoritative rail yields `StillUnknown`.
///
/// Remaining gap: there is no automatic timer or submit lease. The caller must ensure no
/// `submit_via_rail` for this effect is still in flight before invoking this.
pub fn abandon_unsubmitted(
    claims: &ClaimStore,
    ledger: &dyn LedgerPort,
    rail: &dyn RailActuator,
    effect_digest: &str,
) -> PayResult<FinalizeResult> {
    let Some(claim) = claims.get(effect_digest)? else {
        return refuse(PAYMENT_NOT_SUBMITTED);
    };
    if claim.state != ClaimState::Reserved {
        return refuse(NOT_ABANDONABLE);
    }
    let Ok(report) = observe_rail(rail, effect_digest) else {
        return Ok(FinalizeResult::StillUnknown);
    };
    if !matches!(report.status(), RailStatus::Unknown) {
        return finalize_via_rail(effect_digest, claims, ledger, rail);
    }
    if !rail.absence_is_authoritative() {
        return Ok(FinalizeResult::StillUnknown);
    }
    let correlation = RailInstruction::correlation_id_for(effect_digest);
    match release_unsubmitted(claims, ledger, effect_digest, &correlation, RAIL_NEVER_SUBMITTED)? {
        RailStandingAfterSubmit::Refused => Ok(FinalizeResult::ProvenAbsent),
        _ => Ok(FinalizeResult::StillUnknown),
    }
}
