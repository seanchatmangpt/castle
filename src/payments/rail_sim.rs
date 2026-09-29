//! Independent, durable, hostile rail simulator. State lives under a root dir
//! (fsynced atomic publish) so it survives a "process restart" (re-`open`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::dirlock::{publish_new, publish_replace, DirLock};
use super::rail::{RailAck, RailActuator, RailError, RailInstruction, RailStatus};
use crate::sa2a_security::encoding::sha256_tagged;

pub const SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD: &str = "DUPLICATE_CORRELATION_DIFFERENT_PAYLOAD";
pub const SIM_REJECT_AFTER_DELAY: &str = "REJECTED_BY_RAIL_AFTER_DELAY";
pub const SIM_RETURN_REASON: &str = "RETURNED_BY_BENEFICIARY_BANK";
const SIM_DOMAIN: &[u8] = b"CASTLE-SIMRAIL-V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimMode {
    /// First poll Accepted, second poll Settled.
    Honest,
    /// Accept is persisted, caller gets Timeout. Settles on the second poll.
    DropAckAfterAccept,
    /// Accepted for n polls, then Rejected.
    RejectAfterPolls(u32),
    /// Accepted for n polls, then Settled.
    SettleAfterPolls(u32),
    /// Accepted, Settled, then Returned on later polls.
    ReturnAfterSettle,
    Down,
    /// Accepted then Settled; the terminal report is repeated identically forever.
    DuplicateReports,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    correlation_id: String,
    payload_digest: String,
    amount_minor: u64,
    payee: String,
    submissions: usize,
    polls: u32,
    settlement_count: usize,
    last: Option<RailStatus>,
    history: Vec<RailStatus>,
}

#[derive(Debug)]
pub struct SimRail {
    root: PathBuf,
}

fn io<E: std::fmt::Display>(e: E) -> String {
    format!("REFUSED:SIM_RAIL_IO:{e}")
}

fn unavailable(e: String) -> RailError {
    RailError::Unavailable(e)
}

fn key(correlation: &str) -> String {
    sha256_tagged(SIM_DOMAIN, correlation.as_bytes()).trim_start_matches("sha256:").to_string()
}

impl SimRail {
    pub fn open(root: &Path, mode: SimMode) -> Result<Self, String> {
        std::fs::create_dir_all(root.join("records")).map_err(io)?;
        let s = Self { root: root.to_path_buf() };
        s.set_mode(mode)?;
        Ok(s)
    }

    pub fn set_mode(&self, mode: SimMode) -> Result<(), String> {
        let _l = DirLock::acquire(&self.root)?;
        let bytes = serde_json::to_vec(&mode).map_err(io)?;
        publish_replace(&self.root, &self.root.join("mode.json"), &bytes)
    }

    fn mode(&self) -> Result<SimMode, String> {
        let b = std::fs::read(self.root.join("mode.json")).map_err(io)?;
        serde_json::from_slice(&b).map_err(io)
    }

    fn rec_path(&self, correlation: &str) -> PathBuf {
        self.root.join("records").join(format!("{}.json", key(correlation)))
    }

    fn load(&self, correlation: &str) -> Result<Option<Record>, String> {
        match std::fs::read(self.rec_path(correlation)) {
            Ok(b) => serde_json::from_slice(&b).map(Some).map_err(io),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io(e)),
        }
    }

    fn store(&self, r: &Record) -> Result<(), String> {
        let bytes = serde_json::to_vec(r).map_err(io)?;
        let dir = self.root.join("records");
        publish_replace(&dir, &self.rec_path(&r.correlation_id), &bytes)
    }

    fn create(&self, r: &Record) -> Result<bool, String> {
        let bytes = serde_json::to_vec(r).map_err(io)?;
        let dir = self.root.join("records");
        publish_new(&dir, &self.rec_path(&r.correlation_id), &bytes)
    }

    fn all(&self) -> Result<Vec<Record>, String> {
        let mut out = Vec::new();
        for e in std::fs::read_dir(self.root.join("records")).map_err(io)? {
            let p = e.map_err(io)?.path();
            if p.extension().and_then(|x| x.to_str()) == Some("json") {
                let b = std::fs::read(&p).map_err(io)?;
                out.push(serde_json::from_slice::<Record>(&b).map_err(io)?);
            }
        }
        out.sort_by(|a, b| a.correlation_id.cmp(&b.correlation_id));
        Ok(out)
    }

    /// Number of settlements ever created for the correlation (must never exceed 1).
    #[must_use]
    pub fn settlement_count(&self, correlation: &str) -> usize {
        self.load(correlation).ok().flatten().map_or(0, |r| r.settlement_count)
    }

    /// Number of `submit` calls that reached the rail for the correlation.
    #[must_use]
    pub fn submissions_seen(&self, correlation: &str) -> usize {
        self.load(correlation).ok().flatten().map_or(0, |r| r.submissions)
    }

    #[must_use]
    pub fn accepted_correlations(&self) -> Vec<String> {
        self.all().map(|v| v.into_iter().map(|r| r.correlation_id).collect()).unwrap_or_default()
    }

    /// Every distinct report status() has emitted, in order (consecutive repeats collapsed).
    #[must_use]
    pub fn status_history(&self, correlation: &str) -> Vec<RailStatus> {
        self.load(correlation).ok().flatten().map(|r| r.history).unwrap_or_default()
    }

    fn compute(mode: SimMode, polls: u32) -> RailStatus {
        let settled = || RailStatus::Settled { final_ref: String::new() };
        let rejected = || RailStatus::Rejected { reason: SIM_REJECT_AFTER_DELAY.to_string() };
        match mode {
            SimMode::Honest | SimMode::DropAckAfterAccept | SimMode::DuplicateReports => {
                let at = if mode == SimMode::Honest || mode == SimMode::DuplicateReports { 2 } else { 2 };
                if polls >= at { settled() } else { RailStatus::Accepted }
            }
            SimMode::RejectAfterPolls(n) => if polls > n { rejected() } else { RailStatus::Accepted },
            SimMode::SettleAfterPolls(n) => if polls > n { settled() } else { RailStatus::Accepted },
            SimMode::ReturnAfterSettle => match polls {
                0 | 1 => RailStatus::Accepted,
                2 => settled(),
                _ => RailStatus::Returned { reason: SIM_RETURN_REASON.to_string(), return_ref: String::new() },
            },
            SimMode::Down => RailStatus::Unknown,
        }
    }
}

impl RailActuator for SimRail {
    fn submit(&self, i: &RailInstruction) -> Result<RailAck, RailError> {
        let mode = self.mode().map_err(unavailable)?;
        if mode == SimMode::Down {
            return Err(RailError::Unavailable("SIM_RAIL_DOWN".into()));
        }
        let _l = DirLock::acquire(&self.root).map_err(unavailable)?;
        let digest = i.payload_digest();
        let ack = match self.load(&i.correlation_id).map_err(unavailable)? {
            Some(mut r) => {
                r.submissions += 1;
                let same = r.payload_digest == digest && r.amount_minor == i.amount_minor && r.payee == i.payee;
                self.store(&r).map_err(unavailable)?;
                if same {
                    RailAck::Accepted { correlation_id: i.correlation_id.clone() }
                } else {
                    RailAck::Rejected {
                        correlation_id: i.correlation_id.clone(),
                        reason_code: SIM_REJECT_DUPLICATE_DIFFERENT_PAYLOAD.to_string(),
                    }
                }
            }
            None => {
                let r = Record {
                    correlation_id: i.correlation_id.clone(),
                    payload_digest: digest,
                    amount_minor: i.amount_minor,
                    payee: i.payee.clone(),
                    submissions: 1,
                    polls: 0,
                    settlement_count: 0,
                    last: None,
                    history: Vec::new(),
                };
                self.create(&r).map_err(unavailable)?;
                RailAck::Accepted { correlation_id: i.correlation_id.clone() }
            }
        };
        if mode == SimMode::DropAckAfterAccept && matches!(ack, RailAck::Accepted { .. }) {
            return Err(RailError::Timeout);
        }
        Ok(ack)
    }

    fn status(&self, correlation_id: &str) -> Result<RailStatus, RailError> {
        let mode = self.mode().map_err(unavailable)?;
        if mode == SimMode::Down {
            return Err(RailError::Unavailable("SIM_RAIL_DOWN".into()));
        }
        let _l = DirLock::acquire(&self.root).map_err(unavailable)?;
        let Some(mut r) = self.load(correlation_id).map_err(unavailable)? else {
            return Ok(RailStatus::Unknown);
        };
        r.polls = r.polls.saturating_add(1);
        let k = key(correlation_id);
        let mut st = Self::compute(mode, r.polls);
        match (&r.last, &st) {
            // A rejection is final; a settlement never regresses to Accepted on a mode change.
            (Some(l @ RailStatus::Rejected { .. }), _) => st = l.clone(),
            (Some(l @ (RailStatus::Settled { .. } | RailStatus::Returned { .. })), RailStatus::Accepted) => {
                st = l.clone();
            }
            _ => {}
        }
        match &mut st {
            RailStatus::Settled { final_ref } if final_ref.is_empty() => {
                *final_ref = match &r.last {
                    Some(RailStatus::Settled { final_ref: f }) => f.clone(),
                    _ => format!("FINAL-{}", &k[..16]),
                };
            }
            RailStatus::Returned { return_ref, .. } if return_ref.is_empty() => {
                *return_ref = format!("RETURN-{}", &k[..16]);
            }
            _ => {}
        }
        if matches!(st, RailStatus::Settled { .. }) && r.settlement_count == 0 {
            r.settlement_count = 1;
        }
        if r.history.last() != Some(&st) {
            r.history.push(st.clone());
        }
        r.last = Some(st.clone());
        self.store(&r).map_err(unavailable)?;
        Ok(st)
    }
}
