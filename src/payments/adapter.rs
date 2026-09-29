use std::collections::BTreeMap;
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::castle::{
    ActuationPermit, GymActAdapter, GymActResult, GymActStatus, OcelObject, PowlActivity, WorldState,
};

use super::admission::PaymentAdmission;
use super::ledger::{LedgerEntry, LedgerError, LedgerPort};
use super::refusal;

pub const T_RESERVE: &str = "payments.reserve";
pub const T_POST: &str = "payments.post";

/// What the adapter observed at each step. `definite` refusals prove nothing
/// was posted; indefinite ones leave the outcome UNKNOWN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Reserved,
    Posted(LedgerEntry),
    Refused { code: &'static str, definite: bool },
}

/// The only implementor that can move value. Constructed inside
/// `execute_payment`; not reachable without a `PaymentAdmission`.
pub struct PaymentGymActAdapter<'a> {
    ledger: &'a dyn LedgerPort,
    admission: &'a PaymentAdmission,
    subject: String,
    outcomes: Mutex<Vec<StepOutcome>>,
}

impl<'a> PaymentGymActAdapter<'a> {
    pub(crate) fn new(ledger: &'a dyn LedgerPort, admission: &'a PaymentAdmission, subject: String) -> Self {
        Self { ledger, admission, subject, outcomes: Mutex::new(Vec::new()) }
    }

    #[must_use]
    pub fn outcomes(&self) -> Vec<StepOutcome> {
        self.outcomes.lock().map(|o| o.clone()).unwrap_or_default()
    }

    fn record(&self, outcome: StepOutcome) {
        if let Ok(mut o) = self.outcomes.lock() {
            o.push(outcome);
        }
    }

    fn refused(&self, activity: &PowlActivity, code: &'static str, definite: bool) -> GymActResult {
        self.record(StepOutcome::Refused { code, definite });
        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Refused,
            objects: Vec::new(),
            attributes: BTreeMap::from([
                ("reason".to_string(), json!(code)),
                ("definite".to_string(), json!(definite)),
            ]),
        }
    }

    fn observed(&self, activity: &PowlActivity, extra: BTreeMap<String, Value>) -> GymActResult {
        let e = self.admission.effect();
        let mut attributes = BTreeMap::from([
            ("effect_digest".to_string(), json!(e.digest())),
            ("payer".to_string(), json!(e.payer())),
            ("payee".to_string(), json!(e.payee())),
            ("amount_minor".to_string(), json!(e.money().minor.to_string())),
            ("currency".to_string(), json!(e.money().currency.code())),
            ("obligation_id".to_string(), json!(e.obligation_id())),
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
impl GymActAdapter for PaymentGymActAdapter<'_> {
    async fn execute(&self, activity: &PowlActivity, state: &WorldState, permit: &ActuationPermit) -> GymActResult {
        if permit.transition_id != activity.transition_id
            || permit.subject != self.subject
            || state.system_id != self.subject
        {
            return self.refused(activity, refusal::CAPABILITY_NOT_ADMITTED, true);
        }
        let e = self.admission.effect();
        match activity.transition_id.as_str() {
            T_RESERVE => match self.ledger.lookup(e.digest()) {
                Ok(Some(_)) => self.refused(activity, refusal::ALREADY_SETTLED, false),
                Err(_) => self.refused(activity, refusal::LEDGER_UNAVAILABLE, true),
                Ok(None) => match self.ledger.balance(e.payer(), e.money().currency) {
                    Ok(b) if b >= e.money().minor => {
                        self.record(StepOutcome::Reserved);
                        self.observed(activity, BTreeMap::new())
                    }
                    Ok(_) => self.refused(activity, refusal::INSUFFICIENT_FUNDS, true),
                    Err(_) => self.refused(activity, refusal::LEDGER_UNAVAILABLE, true),
                },
            },
            T_POST => match self.ledger.post(self.admission, &permit.construct_digest) {
                Ok(entry) => {
                    let seq = entry.seq;
                    self.record(StepOutcome::Posted(entry));
                    self.observed(activity, BTreeMap::from([("ledger_seq".to_string(), json!(seq))]))
                }
                Err(LedgerError::InsufficientFunds) => self.refused(activity, refusal::INSUFFICIENT_FUNDS, true),
                Err(LedgerError::Unavailable(_)) => self.refused(activity, refusal::LEDGER_UNAVAILABLE, false),
            },
            _ => self.refused(activity, refusal::CAPABILITY_NOT_ADMITTED, true),
        }
    }
}
