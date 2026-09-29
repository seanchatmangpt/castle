//! Observed-finality settlement: the ledger observes settlement, it does not
//! fabricate it. A ledger entry becomes `settled` only through a sealed
//! observation minted here from rail-finality evidence, never from a
//! successful outbound call.

use super::claim_store::{ClaimState, ClaimStore};
use super::ledger::{LedgerEntry, LedgerError, LedgerPort};
use super::refusal::{self, refuse, PayResult};

pub const EVIDENCE_CONFLICT: &str = "REFUSED:PAYMENT_EVIDENCE_CONFLICT";
pub const PAYMENT_NOT_HELD: &str = "REFUSED:PAYMENT_NOT_HELD";
pub const PAYMENT_NOT_SUBMITTED: &str = "REFUSED:PAYMENT_NOT_SUBMITTED";
pub const PAYMENT_NOT_FINAL: &str = "REFUSED:PAYMENT_NOT_FINAL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalityKind {
    Final,
    Rejected,
    Returned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityEvidence {
    pub effect_digest: String,
    pub correlation_id: String,
    pub evidence_digest: String,
    pub kind: FinalityKind,
    pub reason: String,
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
/// is refused with `EVIDENCE_CONFLICT` and changes no state.
pub fn apply_finality(claims: &ClaimStore, ledger: &dyn LedgerPort, ev: &FinalityEvidence) -> PayResult<FinalityOutcome> {
    if ev.evidence_digest.is_empty() || ev.effect_digest.is_empty() {
        return refuse(refusal::PAYLOAD_INVALID);
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
                claims.transition(
                    digest,
                    &[ClaimState::Submitted, ClaimState::UnknownOutcome],
                    ClaimState::Final,
                    None,
                    &format!("final:{}", ev.evidence_digest),
                )?;
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
                claims.transition(
                    digest,
                    &[ClaimState::Submitted, ClaimState::UnknownOutcome],
                    ClaimState::Refused,
                    None,
                    &format!("rejected:{}:{}", ev.evidence_digest, ev.reason),
                )?;
                Ok(FinalityOutcome::Released)
            }
            _ => refuse(PAYMENT_NOT_SUBMITTED),
        },
        FinalityKind::Returned => match claim.state {
            ClaimState::Final | ClaimState::Returned => {
                let entry = ledger.settle_return(&ReturnObservation::new(ev)).map_err(ledger_err)?;
                if claim.state == ClaimState::Final {
                    claims.transition(
                        digest,
                        &[ClaimState::Final],
                        ClaimState::Returned,
                        None,
                        &format!("returned:{}:{}", ev.evidence_digest, ev.reason),
                    )?;
                }
                Ok(FinalityOutcome::Returned(entry))
            }
            ClaimState::Submitted | ClaimState::UnknownOutcome | ClaimState::Refused => refuse(EVIDENCE_CONFLICT),
            _ => refuse(PAYMENT_NOT_FINAL),
        },
    }
}
