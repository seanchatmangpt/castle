//! A hand-written REAL `RailActuator` (not a mock): replays a fixed script of statuses,
//! repeating the last one forever. Trust boundary is honest: it is the operator-configured
//! rail collaborator; tests drive finality only through it.
#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::Mutex;

use castle::payments::{RailAck, RailActuator, RailError, RailInstruction, RailStatus};

pub struct ScriptedRail {
    script: Mutex<VecDeque<RailStatus>>,
    last: Mutex<RailStatus>,
    authoritative: bool,
}

impl ScriptedRail {
    pub fn new(script: Vec<RailStatus>) -> Self {
        Self { script: Mutex::new(script.into()), last: Mutex::new(RailStatus::Unknown), authoritative: false }
    }
    pub fn authoritative(mut self) -> Self {
        self.authoritative = true;
        self
    }
    pub fn push(&self, s: RailStatus) {
        self.script.lock().unwrap().push_back(s);
    }
}

pub fn settled() -> RailStatus {
    RailStatus::Settled { final_ref: "FINAL-SCRIPTED".into() }
}
pub fn rejected() -> RailStatus {
    RailStatus::Rejected { reason: "SCRIPTED_REJECT".into() }
}
pub fn returned() -> RailStatus {
    RailStatus::Returned { reason: "SCRIPTED_RETURN".into(), return_ref: "RET-SCRIPTED".into() }
}

impl RailActuator for ScriptedRail {
    fn submit(&self, i: &RailInstruction) -> Result<RailAck, RailError> {
        Ok(RailAck::Accepted { correlation_id: i.correlation_id.clone() })
    }
    fn status(&self, _correlation_id: &str) -> Result<RailStatus, RailError> {
        let next = self.script.lock().unwrap().pop_front();
        let mut last = self.last.lock().unwrap();
        if let Some(n) = next {
            *last = n;
        }
        Ok(last.clone())
    }
    fn absence_is_authoritative(&self) -> bool {
        self.authoritative
    }
}
