use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ReleaseStanding;

fn digest_ok(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChaosScenario {
    RegionLoss,
    WanPartition,
    StalePolicy,
    RevokedAuthority,
    ClockSkew,
    ProviderThrottle,
    PartialActuation,
    ReceiptStoreLoss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChaosEvidence {
    pub scenario: ChaosScenario,
    pub exercised: bool,
    pub failed_closed: bool,
    pub receipt_digest: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChaosQualification {
    pub standing: ReleaseStanding,
    pub reasons: Vec<String>,
    pub scenarios: BTreeMap<ChaosScenario, ReleaseStanding>,
}

#[must_use]
pub fn required_chaos_scenarios() -> Vec<ChaosScenario> {
    vec![
        ChaosScenario::RegionLoss,
        ChaosScenario::WanPartition,
        ChaosScenario::StalePolicy,
        ChaosScenario::RevokedAuthority,
        ChaosScenario::ClockSkew,
        ChaosScenario::ProviderThrottle,
        ChaosScenario::PartialActuation,
        ChaosScenario::ReceiptStoreLoss,
    ]
}

/// Qualification is evidence-driven: missing scenario evidence is UNKNOWN,
/// invalid or fail-open evidence is REFUSED, and only all exercised +
/// fail-closed + receipted scenarios produce ALIVE.
#[must_use]
pub fn qualify_chaos(evidence: &[ChaosEvidence]) -> ChaosQualification {
    let by_scenario: BTreeMap<ChaosScenario, &ChaosEvidence> = evidence.iter().map(|row| (row.scenario, row)).collect();
    let mut reasons = Vec::new();
    let mut scenarios = BTreeMap::new();

    for scenario in required_chaos_scenarios() {
        let standing = match by_scenario.get(&scenario) {
            None => {
                reasons.push(format!("UNKNOWN:MISSING_CHAOS_EVIDENCE:{scenario:?}"));
                ReleaseStanding::Unknown
            }
            Some(row) if !row.exercised => {
                reasons.push(format!("UNKNOWN:CHAOS_NOT_EXERCISED:{scenario:?}"));
                ReleaseStanding::Unknown
            }
            Some(row) if !digest_ok(&row.receipt_digest) => {
                reasons.push(format!("REFUSED:INVALID_CHAOS_RECEIPT:{scenario:?}"));
                ReleaseStanding::Refused
            }
            Some(row) if !row.failed_closed => {
                reasons.push(format!("REFUSED:CHAOS_FAILED_OPEN:{scenario:?}"));
                ReleaseStanding::Refused
            }
            Some(_) => ReleaseStanding::Alive,
        };
        scenarios.insert(scenario, standing);
    }

    let standing = if scenarios.values().any(|standing| *standing == ReleaseStanding::Refused) {
        ReleaseStanding::Refused
    } else if scenarios.values().any(|standing| *standing == ReleaseStanding::Unknown) {
        ReleaseStanding::Unknown
    } else {
        ReleaseStanding::Alive
    };
    ChaosQualification { standing, reasons, scenarios }
}
