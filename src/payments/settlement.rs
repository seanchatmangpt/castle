//! Observed-finality settlement: the ledger observes settlement, it does not
//! fabricate it. A ledger entry becomes `settled` only through a sealed
//! observation minted here from rail-finality evidence, never from a
//! successful outbound call.
//!
//! Trust boundary (honest): a `RailActuator` is an operator-configured, TRUSTED
//! collaborator; whatever its `status` returns is treated as the rail's word. What
//! this module guarantees is that no caller can *fabricate* finality: the only way to
//! obtain a `RailReport` is `observe_rail`, which actually calls `RailActuator::status`
//! for the effect's own correlation id, and `FinalityEvidence` has private fields with a
//! crate-private constructor, so code outside this crate cannot author evidence.
//! The only public settler is `apply_rail_report`.

use super::claim_store::{ClaimState, ClaimStore};
use super::ledger::{LedgerEntry, LedgerError, LedgerPort};
use super::rail::{RailActuator, RailError, RailInstruction, RailStatus};
use super::refusal::{self, refuse, PayResult};
use crate::sa2a_security::encoding::sha256_tagged;

pub const EVIDENCE_CONFLICT: &str = "REFUSED:PAYMENT_EVIDENCE_CONFLICT";
pub const PAYMENT_NOT_HELD: &str = "REFUSED:PAYMENT_NOT_HELD";
pub const PAYMENT_NOT_SUBMITTED: &str = "REFUSED:PAYMENT_NOT_SUBMITTED";
pub const PAYMENT_NOT_FINAL: &str = "REFUSED:PAYMENT_NOT_FINAL";
pub const REPORT_NOT_TERMINAL: &str = "REFUSED:PAYMENT_REPORT_NOT_TERMINAL";
pub const REPORT_CORRELATION_MISMATCH: &str = "REFUSED:PAYMENT_REPORT_CORRELATION_MISMATCH";

const STATUS_EVIDENCE_DOMAIN: &[u8] = b"CASTLE-RAIL-STATUS-EVIDENCE-V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalityKind {
    Final,
    Rejected,
    Returned,
}

/// Finality evidence. Fields are private and the constructor is crate-private: callers
/// outside the crate cannot author evidence, only obtain it through `apply_rail_report`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityEvidence {
    effect_digest: String,
    correlation_id: String,
    evidence_digest: String,
    kind: FinalityKind,
    reason: String,
}

impl FinalityEvidence {
    pub(crate) fn new(effect_digest: &str, correlation_id: &str, evidence_digest: String, kind: FinalityKind, reason: &str) -> Self {
        Self {
            effect_digest: effect_digest.to_string(),
            correlation_id: correlation_id.to_string(),
            evidence_digest,
            kind,
            reason: reason.to_string(),
        }
    }
    #[must_use]
    pub fn effect_digest(&self) -> &str {
        &self.effect_digest
    }
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }
    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
    #[must_use]
    pub fn kind(&self) -> FinalityKind {
        self.kind
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

pub(crate) fn status_evidence(tag: &str, status: &RailStatus) -> PayResult<String> {
    let bytes = serde_json::to_vec(status).map_err(|_| refusal::PAYLOAD_INVALID.to_string())?;
    let mut body = tag.as_bytes().to_vec();
    body.push(b'|');
    body.extend_from_slice(&bytes);
    Ok(sha256_tagged(STATUS_EVIDENCE_DOMAIN, &body))
}

/// A status report actually obtained from a `RailActuator` for one effect. Sealed: no
/// public constructor; only `observe_rail` mints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailReport {
    effect_digest: String,
    correlation_id: String,
    status: RailStatus,
    evidence_digest: String,
    _seal: (),
}

impl RailReport {
    #[must_use]
    pub fn effect_digest(&self) -> &str {
        &self.effect_digest
    }
    #[must_use]
    pub fn correlation_id(&self) -> &str {
        &self.correlation_id
    }
    #[must_use]
    pub fn status(&self) -> &RailStatus {
        &self.status
    }
    /// Digest of the observed status (the evidence digest recorded on the claim).
    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
}

/// Poll the rail for the effect's own correlation id and seal the answer.
pub fn observe_rail(rail: &dyn RailActuator, effect_digest: &str) -> Result<RailReport, RailError> {
    let correlation_id = RailInstruction::correlation_id_for(effect_digest);
    let status = rail.status(&correlation_id)?;
    let tag = match &status {
        RailStatus::Unknown => "UNKNOWN",
        RailStatus::Accepted => "ACCEPTED",
        RailStatus::Settled { .. } => "SETTLED",
        RailStatus::Rejected { .. } => "REJECTED",
        RailStatus::Returned { .. } => "RETURNED",
    };
    let evidence_digest = status_evidence(tag, &status).map_err(RailError::Unavailable)?;
    Ok(RailReport { effect_digest: effect_digest.to_string(), correlation_id, status, evidence_digest, _seal: () })
}

/// The only public settler: convert a sealed rail report into ledger/claim transitions.
/// Refuses non-terminal reports, reports whose correlation is not the claim's own, and
/// reports for unknown claims.
pub fn apply_rail_report(claims: &ClaimStore, ledger: &dyn LedgerPort, report: &RailReport) -> PayResult<FinalityOutcome> {
    let Some(claim) = claims.get(&report.effect_digest)? else {
        return refuse(PAYMENT_NOT_SUBMITTED);
    };
    if report.correlation_id != RailInstruction::correlation_id_for(&claim.effect_digest) {
        return refuse(REPORT_CORRELATION_MISMATCH);
    }
    let d = &claim.effect_digest;
    let c = &report.correlation_id;
    match &report.status {
        RailStatus::Settled { .. } => apply_finality(
            claims,
            ledger,
            &FinalityEvidence::new(d, c, report.evidence_digest.clone(), FinalityKind::Final, "SETTLED"),
        ),
        RailStatus::Rejected { reason } => apply_finality(
            claims,
            ledger,
            &FinalityEvidence::new(d, c, report.evidence_digest.clone(), FinalityKind::Rejected, reason),
        ),
        RailStatus::Returned { reason, .. } => {
            // A return is itself proof of prior settlement: settle first if not yet seen.
            if matches!(claim.state, ClaimState::Submitted | ClaimState::UnknownOutcome) {
                let implied = status_evidence("SETTLED_IMPLIED_BY_RETURN", &report.status)?;
                apply_finality(
                    claims,
                    ledger,
                    &FinalityEvidence::new(d, c, implied, FinalityKind::Final, "SETTLED_IMPLIED_BY_RETURN"),
                )?;
            }
            apply_finality(
                claims,
                ledger,
                &FinalityEvidence::new(d, c, report.evidence_digest.clone(), FinalityKind::Returned, reason),
            )
        }
        RailStatus::Accepted | RailStatus::Unknown => refuse(REPORT_NOT_TERMINAL),
    }
}

macro_rules! sealed_observation {
    ($(#[$m:meta])* $name:ident { $($extra:ident),* }) => {
        $(#[$m])*
        #[derive(Debug)]
        pub struct $name {
            effect_digest: String,
            correlation_id: String,
            evidence_digest: String,
            $($extra: String,)*
            _seal: (),
        }
        impl $name {
            #[must_use]
            pub fn effect_digest(&self) -> &str { &self.effect_digest }
            #[must_use]
            pub fn correlation_id(&self) -> &str { &self.correlation_id }
            #[must_use]
            pub fn evidence_digest(&self) -> &str { &self.evidence_digest }
        }
    };
}

sealed_observation!(
    /// Observed rail finality for an effect; only `apply_finality` can mint one.
    SettlementObservation { payee }
);
sealed_observation!(
    /// Observed rail rejection (definite absence); only `apply_finality` can mint one.
    RejectionObservation {}
);
sealed_observation!(
    /// Observed return of settled funds; only `apply_finality` can mint one.
    ReturnObservation {}
);

impl SettlementObservation {
    #[must_use]
    pub fn payee(&self) -> &str {
        &self.payee
    }
    fn new(ev: &FinalityEvidence, payee: &str) -> Self {
        Self {
            effect_digest: ev.effect_digest.clone(),
            correlation_id: ev.correlation_id.clone(),
            evidence_digest: ev.evidence_digest.clone(),
            payee: payee.to_string(),
            _seal: (),
        }
    }
}
impl RejectionObservation {
    fn new(ev: &FinalityEvidence) -> Self {
        Self {
            effect_digest: ev.effect_digest.clone(),
            correlation_id: ev.correlation_id.clone(),
            evidence_digest: ev.evidence_digest.clone(),
            _seal: (),
        }
    }
}
impl ReturnObservation {
    fn new(ev: &FinalityEvidence) -> Self {
        Self {
            effect_digest: ev.effect_digest.clone(),
            correlation_id: ev.correlation_id.clone(),
            evidence_digest: ev.evidence_digest.clone(),
            _seal: (),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalityOutcome {
    Settled(LedgerEntry),
    AlreadyFinal,
    Released,
    Returned(LedgerEntry),
}

fn ledger_err(e: LedgerError) -> String {
    match e {
        LedgerError::InsufficientFunds => refusal::INSUFFICIENT_FUNDS.to_string(),
        LedgerError::Unavailable(d) => format!("{}:{d}", refusal::LEDGER_UNAVAILABLE),
    }
}

/// Apply one piece of rail-finality evidence. Conflicting or reordered evidence
/// is refused with `EVIDENCE_CONFLICT` and changes no state. `FinalityEvidence` cannot be
/// constructed outside this crate, so external callers cannot reach this with forged
/// evidence; in-crate callers are `apply_rail_report` and the local never-submitted paths.
/// (Kept `pub` only because `payments/mod.rs` re-exports it; coordinator should demote it
/// to `pub(crate)` and drop the re-export.)
pub fn apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) -> PayResult<FinalityOutcome> {
    if ev.evidence_digest.is_empty() || ev.effect_digest.is_empty() {
        return refuse(refusal::PAYLOAD_INVALID);
    }
    if ev.correlation_id != RailInstruction::correlation_id_for(&ev.effect_digest) {
        return refuse(REPORT_CORRELATION_MISMATCH);
    }
    let Some(claim) = claims.get(&ev.effect_digest)? else {
        return refuse(PAYMENT_NOT_SUBMITTED);
    };
    let digest = claim.effect_digest.as_str();
    let inflight = matches!(claim.state, ClaimState::Submitted | ClaimState::UnknownOutcome);
    match ev.kind {
        FinalityKind::Final => match claim.state {
            ClaimState::Final => Ok(FinalityOutcome::AlreadyFinal),
            ClaimState::Refused | ClaimState::Returned => refuse(EVIDENCE_CONFLICT),
            _ if inflight => {
                // A prior attempt may have settled the ledger before the claim moved.
                if ledger.hold_of(digest).map_err(ledger_err)?.is_none() && ledger.lookup(digest).map_err(ledger_err)?.is_none() {
                    return refuse(PAYMENT_NOT_HELD);
                }
                let entry = ledger.settle_hold(&SettlementObservation::new(ev, &claim.payee)).map_err(ledger_err)?;
                if let Err(e) = claims.transition(
                    digest,
                    &[ClaimState::Submitted, ClaimState::UnknownOutcome],
                    ClaimState::Final,
                    None,
                    &format!("final:{}", ev.evidence_digest),
                ) {
                    // A racing finalizer may already have moved the claim to the target.
                    return match claims.get(digest)? {
                        Some(c) if matches!(c.state, ClaimState::Final | ClaimState::Returned) => Ok(FinalityOutcome::AlreadyFinal),
                        _ => Err(e),
                    };
                }
                Ok(FinalityOutcome::Settled(entry))
            }
            _ => refuse(PAYMENT_NOT_SUBMITTED),
        },
        FinalityKind::Rejected => match claim.state {
            ClaimState::Final | ClaimState::Returned => refuse(EVIDENCE_CONFLICT),
            ClaimState::Refused => Ok(FinalityOutcome::Released),
            _ if inflight => {
                if ledger.lookup(digest).map_err(ledger_err)?.is_some() {
                    return refuse(EVIDENCE_CONFLICT); // funds already settled
                }
                if ledger.hold_of(digest).map_err(ledger_err)?.is_some() {
                    ledger.release_hold(&RejectionObservation::new(ev)).map_err(ledger_err)?;
                }
                if let Err(e) = claims.transition(
                    digest,
                    &[ClaimState::Submitted, ClaimState::UnknownOutcome],
                    ClaimState::Refused,
                    None,
                    &format!("rejected:{}:{}", ev.evidence_digest, ev.reason),
                ) {
                    return match claims.get(digest)? {
                        Some(c) if c.state == ClaimState::Refused => Ok(FinalityOutcome::Released),
                        _ => Err(e),
                    };
                }
                Ok(FinalityOutcome::Released)
            }
            _ => refuse(PAYMENT_NOT_SUBMITTED),
        },
        FinalityKind::Returned => match claim.state {
            ClaimState::Final | ClaimState::Returned => {
                let entry = ledger.settle_return(&ReturnObservation::new(ev)).map_err(ledger_err)?;
                if claim.state == ClaimState::Final {
                    if let Err(e) = claims.transition(
                        digest,
                        &[ClaimState::Final],
                        ClaimState::Returned,
                        None,
                        &format!("returned:{}:{}", ev.evidence_digest, ev.reason),
                    ) {
                        match claims.get(digest)? {
                            Some(c) if c.state == ClaimState::Returned => {}
                            _ => return Err(e),
                        }
                    }
                }
                Ok(FinalityOutcome::Returned(entry))
            }
            ClaimState::Submitted | ClaimState::UnknownOutcome | ClaimState::Refused => refuse(EVIDENCE_CONFLICT),
            _ => refuse(PAYMENT_NOT_FINAL),
        },
    }
}
