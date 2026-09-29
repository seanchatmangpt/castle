//! Board-level strategic command for CASTLE.
//!
//! This module ports the reusable semantics from the Berthier campaign compiler
//! (chatman-ecosystem PR #285) into CASTLE's sellable board surface. It is
//! deliberately non-actuating: strategy may OBSERVE, SIMULATE, SELECT, DECOMPOSE,
//! ROUTE, REPLAN, VERIFY, or CONSTRUCT, but it may never DO. A board-selected
//! campaign leaves this module as an inert mandate packet whose only successor is
//! the existing CASTLE BRCE boundary.
//!
//! The strategic operator vocabulary is the compact machine form of the earlier
//! 33 Strategies of War / Napoleon-Berthier / Fuller / DfCM work. Those sources
//! are prior-art/provenance, not authority. Runtime authority still comes only
//! from CASTLE's admitted consequence path.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

pub const STRATEGIC_AUTHORITY_CEILING: &str = "CONSTRUCT";
pub const STRATEGIC_SUCCESSOR_BOUNDARY: &str = "BRCE";
pub const STRATEGIC_ACTUATION: &str = "NONE";

fn canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err("REFUSED:NON_FINITE_CANONICAL_VALUE".to_string());
                }
            }
            Ok(n.to_string())
        }
        Value::String(s) => serde_json::to_string(s)
            .map_err(|e| format!("REFUSED:CANONICAL_JSON:{e}")),
        Value::Array(items) => {
            let parts: Result<Vec<String>, String> = items.iter().map(canonical_json).collect();
            Ok(format!("[{}]", parts?.join(",")))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                let encoded_key = serde_json::to_string(key)
                    .map_err(|e| format!("REFUSED:CANONICAL_JSON:{e}"))?;
                parts.push(format!("{encoded_key}:{}", canonical_json(&map[key])?));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
    }
}

fn canonical_digest(value: &Value) -> Result<String, String> {
    Ok(blake3::hash(canonical_json(value)?.as_bytes())
        .to_hex()
        .to_string())
}

fn lowercase_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoardAvatar {
    Audit,
    Risk,
    CapitalAllocation,
    Resilience,
    Governance,
    Safety,
    CompetitiveStrategy,
    LeadIndependent,
}

impl BoardAvatar {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Audit => "audit",
            Self::Risk => "risk",
            Self::CapitalAllocation => "capital-allocation",
            Self::Resilience => "resilience",
            Self::Governance => "governance",
            Self::Safety => "safety",
            Self::CompetitiveStrategy => "competitive-strategy",
            Self::LeadIndependent => "lead-independent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardLens {
    pub avatar: BoardAvatar,
    pub question: &'static str,
    pub governed_surface: &'static str,
}

pub const BOARD_LENSES: [BoardLens; 8] = [
    BoardLens {
        avatar: BoardAvatar::Audit,
        question: "Can we prove the corporation did what the board authorized?",
        governed_surface: "evidence-and-replay",
    },
    BoardLens {
        avatar: BoardAvatar::Risk,
        question: "What can still cause material harm when upstream assumptions fail?",
        governed_surface: "prohibited-outcomes-and-residual-reachability",
    },
    BoardLens {
        avatar: BoardAvatar::CapitalAllocation,
        question: "Where does the next unit of capital preserve or create the most strategic option value?",
        governed_surface: "capital-bounds-and-optionality",
    },
    BoardLens {
        avatar: BoardAvatar::Resilience,
        question: "Can the mission continue lawfully after a route, provider, or plan fails?",
        governed_surface: "lawful-recovery-and-replanning",
    },
    BoardLens {
        avatar: BoardAvatar::Governance,
        question: "Did strategic intent mutate while it traveled through the corporation?",
        governed_surface: "mandate-lineage-and-delegated-authority",
    },
    BoardLens {
        avatar: BoardAvatar::Safety,
        question: "How much can the company explore before an irreversible consequence occurs?",
        governed_surface: "reversibility-and-commitment-boundaries",
    },
    BoardLens {
        avatar: BoardAvatar::CompetitiveStrategy,
        question: "Are we merely choosing moves, or changing the game so better options become reachable?",
        governed_surface: "maneuver-and-state-transformation",
    },
    BoardLens {
        avatar: BoardAvatar::LeadIndependent,
        question: "Which decisions remain constitutionally the board's and may not be silently delegated?",
        governed_surface: "nondelegable-corporate-sovereignty",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrategicAxis {
    Position,
    Topology,
    OptionSpace,
    Belief,
    Tempo,
    Objective,
    Authority,
    Reversibility,
}

impl StrategicAxis {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Position => "P",
            Self::Topology => "Gamma",
            Self::OptionSpace => "Omega",
            Self::Belief => "B",
            Self::Tempo => "tau",
            Self::Objective => "J",
            Self::Authority => "A",
            Self::Reversibility => "R",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyOperator {
    pub id: &'static str,
    pub provenance: &'static str,
    pub reads: &'static [StrategicAxis],
    pub writes: &'static [StrategicAxis],
}

pub const STRATEGY_OPERATORS: &[StrategyOperator] = &[
    StrategyOperator {
        id: "refuse-last-war",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Belief, StrategicAxis::Objective],
        writes: &[StrategicAxis::Belief, StrategicAxis::Objective],
    },
    StrategyOperator {
        id: "concentrate",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Position, StrategicAxis::OptionSpace, StrategicAxis::Tempo],
        writes: &[StrategicAxis::Position, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "segment",
        provenance: "Napoleon-Berthier-derived",
        reads: &[StrategicAxis::Topology, StrategicAxis::Authority],
        writes: &[StrategicAxis::Topology, StrategicAxis::Authority],
    },
    StrategyOperator {
        id: "change-terrain",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Topology, StrategicAxis::OptionSpace],
        writes: &[StrategicAxis::Topology, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "indirect-approach",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Position, StrategicAxis::Topology],
        writes: &[StrategicAxis::Position, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "tempo",
        provenance: "Napoleon-Berthier-derived",
        reads: &[StrategicAxis::Tempo, StrategicAxis::OptionSpace],
        writes: &[StrategicAxis::Tempo, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "preserve-options",
        provenance: "DfCM",
        reads: &[StrategicAxis::Reversibility, StrategicAxis::OptionSpace],
        writes: &[StrategicAxis::Reversibility, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "coalition",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Topology, StrategicAxis::OptionSpace],
        writes: &[StrategicAxis::Topology, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "withdraw",
        provenance: "33SW-derived",
        reads: &[StrategicAxis::Reversibility, StrategicAxis::Objective],
        writes: &[StrategicAxis::Position, StrategicAxis::OptionSpace],
    },
    StrategyOperator {
        id: "closure",
        provenance: "Fuller-Chatman",
        reads: &[StrategicAxis::Objective, StrategicAxis::Topology, StrategicAxis::OptionSpace],
        writes: &[StrategicAxis::Objective, StrategicAxis::Topology, StrategicAxis::OptionSpace],
    },
];

fn strategy_operator_exists(id: &str) -> bool {
    STRATEGY_OPERATORS.iter().any(|operator| operator.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionInput {
    pub subject: String,
    pub mandate_id: String,
    pub objectives: Vec<String>,
    pub prohibited_outcomes: Vec<String>,
    pub invariants: Vec<String>,
    pub delegated_authority: Vec<String>,
    pub nondelegable_decisions: Vec<String>,
    pub capital_at_risk_limit: u64,
    pub escalation_conditions: Vec<String>,
    pub withdrawal_conditions: Vec<String>,
    pub evidence_requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardConstitution {
    pub subject: String,
    pub mandate_id: String,
    pub objectives: Vec<String>,
    pub prohibited_outcomes: Vec<String>,
    pub invariants: Vec<String>,
    pub delegated_authority: Vec<String>,
    pub nondelegable_decisions: Vec<String>,
    pub capital_at_risk_limit: u64,
    pub escalation_conditions: Vec<String>,
    pub withdrawal_conditions: Vec<String>,
    pub evidence_requirements: Vec<String>,
    pub authority_ceiling: &'static str,
    pub constitution_digest: String,
}

impl BoardConstitution {
    fn core_json(&self) -> Value {
        json!({
            "subject": self.subject,
            "mandate_id": self.mandate_id,
            "objectives": self.objectives,
            "prohibited_outcomes": self.prohibited_outcomes,
            "invariants": self.invariants,
            "delegated_authority": self.delegated_authority,
            "nondelegable_decisions": self.nondelegable_decisions,
            "capital_at_risk_limit": self.capital_at_risk_limit,
            "escalation_conditions": self.escalation_conditions,
            "withdrawal_conditions": self.withdrawal_conditions,
            "evidence_requirements": self.evidence_requirements,
            "authority_ceiling": self.authority_ceiling,
        })
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut value = self.core_json();
        if let Value::Object(map) = &mut value {
            map.insert(
                "constitution_digest".to_string(),
                Value::String(self.constitution_digest.clone()),
            );
        }
        value
    }
}

pub fn compile_board_constitution(input: ConstitutionInput) -> Result<BoardConstitution, String> {
    let subject = input.subject.trim().to_string();
    let mandate_id = input.mandate_id.trim().to_string();
    let objectives = sorted_unique(input.objectives);
    let prohibited_outcomes = sorted_unique(input.prohibited_outcomes);
    let mut invariants = sorted_unique(input.invariants);
    let delegated_authority = sorted_unique(input.delegated_authority);
    let nondelegable_decisions = sorted_unique(input.nondelegable_decisions);
    let escalation_conditions = sorted_unique(input.escalation_conditions);
    let withdrawal_conditions = sorted_unique(input.withdrawal_conditions);
    let evidence_requirements = sorted_unique(input.evidence_requirements);

    if subject.is_empty() || mandate_id.is_empty() {
        return Err("REFUSED:UNBOUNDED_BOARD_MANDATE".to_string());
    }
    if objectives.is_empty()
        || prohibited_outcomes.is_empty()
        || nondelegable_decisions.is_empty()
        || evidence_requirements.is_empty()
        || input.capital_at_risk_limit == 0
    {
        return Err("REFUSED:UNBOUNDED_BOARD_MANDATE".to_string());
    }

    invariants.extend([
        "SELECT != CONSTRUCT != DO".to_string(),
        "strategy compilation has no actuation authority".to_string(),
        "board selection cannot manufacture DO authority".to_string(),
    ]);
    invariants = sorted_unique(invariants);

    let mut constitution = BoardConstitution {
        subject,
        mandate_id,
        objectives,
        prohibited_outcomes,
        invariants,
        delegated_authority,
        nondelegable_decisions,
        capital_at_risk_limit: input.capital_at_risk_limit,
        escalation_conditions,
        withdrawal_conditions,
        evidence_requirements,
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING,
        constitution_digest: String::new(),
    };
    constitution.constitution_digest = canonical_digest(&constitution.core_json())?;
    Ok(constitution)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyDoctrine {
    pub subject: String,
    pub constitution_digest: String,
    pub premise_digests: BTreeMap<String, String>,
    pub invariants: Vec<String>,
    pub capability_ids: Vec<String>,
    pub provenance: Vec<String>,
    pub authority_ceiling: &'static str,
    pub doctrine_digest: String,
}

impl StrategyDoctrine {
    fn core_json(&self) -> Value {
        json!({
            "subject": self.subject,
            "constitution_digest": self.constitution_digest,
            "premise_digests": self.premise_digests,
            "invariants": self.invariants,
            "capability_ids": self.capability_ids,
            "provenance": self.provenance,
            "authority_ceiling": self.authority_ceiling,
        })
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut value = self.core_json();
        if let Value::Object(map) = &mut value {
            map.insert(
                "doctrine_digest".to_string(),
                Value::String(self.doctrine_digest.clone()),
            );
        }
        value
    }
}

pub fn compile_strategy_doctrine(
    constitution: &BoardConstitution,
    premise_digests: BTreeMap<String, String>,
    capability_ids: Vec<String>,
    provenance: Vec<String>,
) -> Result<StrategyDoctrine, String> {
    if premise_digests.is_empty() {
        return Err("REFUSED:STRATEGY_UNBOUNDED".to_string());
    }
    for (premise, digest) in &premise_digests {
        if premise.trim().is_empty() || !lowercase_hex_64(digest) {
            return Err("REFUSED:INVALID_STRATEGIC_PREMISE".to_string());
        }
    }
    let capability_ids = sorted_unique(capability_ids);
    let provenance = sorted_unique(provenance);
    if provenance.is_empty() {
        return Err("REFUSED:MISSING_STRATEGIC_PROVENANCE".to_string());
    }

    let mut doctrine = StrategyDoctrine {
        subject: constitution.subject.clone(),
        constitution_digest: constitution.constitution_digest.clone(),
        premise_digests,
        invariants: constitution.invariants.clone(),
        capability_ids,
        provenance,
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING,
        doctrine_digest: String::new(),
    };
    doctrine.doctrine_digest = canonical_digest(&doctrine.core_json())?;
    Ok(doctrine)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyPartition {
    pub strategy_id: String,
    pub strategy: String,
    pub doctrine_digest: String,
    pub local_premise_digests: BTreeMap<String, String>,
    pub local_constraints: Vec<String>,
    pub operator_ids: Vec<String>,
    pub partition_digest: String,
}

impl StrategyPartition {
    fn core_json(&self) -> Value {
        json!({
            "strategy_id": self.strategy_id,
            "strategy": self.strategy,
            "doctrine_digest": self.doctrine_digest,
            "local_premise_digests": self.local_premise_digests,
            "local_constraints": self.local_constraints,
            "operator_ids": self.operator_ids,
        })
    }
}

pub fn partition_strategy(
    doctrine: &StrategyDoctrine,
    strategy_id: impl Into<String>,
    strategy: impl Into<String>,
    local_premise_digests: BTreeMap<String, String>,
    local_constraints: Vec<String>,
    operator_ids: Vec<String>,
) -> Result<StrategyPartition, String> {
    let strategy_id = strategy_id.into().trim().to_string();
    let strategy = strategy.into().trim().to_string();
    let local_constraints = sorted_unique(local_constraints);
    let operator_ids = sorted_unique(operator_ids);

    if strategy_id.is_empty()
        || strategy.is_empty()
        || local_premise_digests.is_empty()
        || local_constraints.is_empty()
        || operator_ids.is_empty()
    {
        return Err("REFUSED:STRATEGY_UNBOUNDED".to_string());
    }
    if local_premise_digests
        .iter()
        .any(|(key, digest)| key.trim().is_empty() || !lowercase_hex_64(digest))
    {
        return Err("REFUSED:INVALID_STRATEGIC_PREMISE".to_string());
    }
    if operator_ids.iter().any(|id| !strategy_operator_exists(id)) {
        return Err("REFUSED:UNKNOWN_STRATEGY_OPERATOR".to_string());
    }

    let mut partition = StrategyPartition {
        strategy_id,
        strategy,
        doctrine_digest: doctrine.doctrine_digest.clone(),
        local_premise_digests,
        local_constraints,
        operator_ids,
        partition_digest: String::new(),
    };
    partition.partition_digest = canonical_digest(&partition.core_json())?;
    Ok(partition)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignCandidate {
    pub candidate_id: String,
    pub subject: String,
    pub strategy_id: String,
    pub constitution_digest: String,
    pub doctrine_digest: String,
    pub partition_digest: String,
    pub preserved_invariants: Vec<String>,
    pub capabilities: Vec<String>,
    pub actions: Vec<String>,
    pub assumptions: Vec<String>,
    pub falsifier: String,
    pub objectives: BTreeMap<String, i64>,
    pub expected_outcomes: Vec<String>,
    pub capital_committed: u64,
    pub reversible_capital: u64,
    pub authority_ceiling: String,
    pub observed_partition_digests: Vec<String>,
}

impl CampaignCandidate {
    fn core_json(&self) -> Value {
        json!({
            "candidate_id": self.candidate_id,
            "subject": self.subject,
            "strategy_id": self.strategy_id,
            "constitution_digest": self.constitution_digest,
            "doctrine_digest": self.doctrine_digest,
            "partition_digest": self.partition_digest,
            "preserved_invariants": self.preserved_invariants,
            "capabilities": self.capabilities,
            "actions": self.actions,
            "assumptions": self.assumptions,
            "falsifier": self.falsifier,
            "objectives": self.objectives,
            "expected_outcomes": self.expected_outcomes,
            "capital_committed": self.capital_committed,
            "reversible_capital": self.reversible_capital,
            "authority_ceiling": self.authority_ceiling,
            "observed_partition_digests": self.observed_partition_digests,
        })
    }

    pub fn candidate_digest(&self) -> Result<String, String> {
        canonical_digest(&self.core_json())
    }
}

pub fn construct_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate_id: impl Into<String>,
    assumptions: Vec<String>,
    falsifier: impl Into<String>,
    objectives: BTreeMap<String, i64>,
    expected_outcomes: Vec<String>,
    capital_committed: u64,
    reversible_capital: u64,
) -> Result<CampaignCandidate, String> {
    let candidate_id = candidate_id.into().trim().to_string();
    let falsifier = falsifier.into().trim().to_string();
    if candidate_id.is_empty() || falsifier.is_empty() || objectives.is_empty() {
        return Err("REFUSED:STRATEGY_UNBOUNDED".to_string());
    }
    Ok(CampaignCandidate {
        candidate_id,
        subject: constitution.subject.clone(),
        strategy_id: partition.strategy_id.clone(),
        constitution_digest: constitution.constitution_digest.clone(),
        doctrine_digest: doctrine.doctrine_digest.clone(),
        partition_digest: partition.partition_digest.clone(),
        preserved_invariants: doctrine.invariants.clone(),
        capabilities: doctrine.capability_ids.clone(),
        actions: vec![
            "OBSERVE".to_string(),
            "SIMULATE".to_string(),
            "COUNTERPLAN".to_string(),
            "SELECT".to_string(),
            "DECOMPOSE".to_string(),
            "ROUTE".to_string(),
            "REPLAN".to_string(),
            "VERIFY".to_string(),
            "CONSTRUCT".to_string(),
        ],
        assumptions: sorted_unique(assumptions),
        falsifier,
        objectives,
        expected_outcomes: sorted_unique(expected_outcomes),
        capital_committed,
        reversible_capital,
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING.to_string(),
        observed_partition_digests: vec![partition.partition_digest.clone()],
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategicStanding {
    Alive,
    Refused,
}

impl StrategicStanding {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alive => "ALIVE",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignVerdict {
    pub standing: StrategicStanding,
    pub candidate_id: String,
    pub candidate_digest: String,
    pub refusals: Vec<String>,
}

pub fn judge_campaign_candidate(
    constitution: &BoardConstitution,
    doctrine: &StrategyDoctrine,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    current_local_premises: &BTreeMap<String, String>,
) -> CampaignVerdict {
    let mut refusals = Vec::new();

    if candidate.subject != constitution.subject
        || candidate.constitution_digest != constitution.constitution_digest
    {
        refusals.push("REFUSED:MANDATE_DRIFT".to_string());
    }
    if doctrine.constitution_digest != constitution.constitution_digest
        || candidate.doctrine_digest != doctrine.doctrine_digest
        || partition.doctrine_digest != doctrine.doctrine_digest
    {
        refusals.push("REFUSED:DOCTRINE_DRIFT".to_string());
    }
    if candidate.strategy_id != partition.strategy_id
        || candidate.partition_digest != partition.partition_digest
    {
        refusals.push("REFUSED:PARTITION_DRIFT".to_string());
    }

    let candidate_invariants: BTreeSet<&str> = candidate
        .preserved_invariants
        .iter()
        .map(String::as_str)
        .collect();
    if doctrine
        .invariants
        .iter()
        .any(|required| !candidate_invariants.contains(required.as_str()))
    {
        refusals.push("REFUSED:CONSTRAINT_WEAKENING".to_string());
    }

    if candidate.observed_partition_digests.len() != 1
        || candidate.observed_partition_digests[0] != partition.partition_digest
    {
        refusals.push("REFUSED:CROSS_PARTITION_CONTAMINATION".to_string());
    }

    const ALLOWED_ACTIONS: &[&str] = &[
        "OBSERVE",
        "SIMULATE",
        "COUNTERPLAN",
        "SELECT",
        "DECOMPOSE",
        "ROUTE",
        "REPLAN",
        "VERIFY",
        "CONSTRUCT",
    ];
    if candidate.actions.is_empty()
        || candidate
            .actions
            .iter()
            .any(|action| !ALLOWED_ACTIONS.contains(&action.as_str()))
    {
        refusals.push("REFUSED:STRATEGY_UNBOUNDED".to_string());
    }

    if candidate.authority_ceiling != STRATEGIC_AUTHORITY_CEILING {
        refusals.push("REFUSED:AUTHORITY_INCREASE".to_string());
    }
    if candidate.falsifier.trim().is_empty() {
        refusals.push("REFUSED:MISSING_FALSIFIER".to_string());
    }
    if candidate.capital_committed > constitution.capital_at_risk_limit {
        refusals.push("REFUSED:CAPITAL_BOUND_EXCEEDED".to_string());
    }
    if candidate.reversible_capital > candidate.capital_committed {
        refusals.push("REFUSED:INVALID_REVERSIBILITY_BOUND".to_string());
    }

    if candidate.expected_outcomes.iter().any(|outcome| {
        constitution
            .prohibited_outcomes
            .iter()
            .any(|prohibited| prohibited == outcome)
    }) {
        refusals.push("REFUSED:PROHIBITED_OUTCOME".to_string());
    }

    for (premise, compiled_digest) in &partition.local_premise_digests {
        if current_local_premises.get(premise) != Some(compiled_digest) {
            refusals.push(format!("REFUSED:STALE_PROJECTION:{premise}"));
        }
    }

    let candidate_digest = match candidate.candidate_digest() {
        Ok(digest) => digest,
        Err(reason) => {
            refusals.push(reason);
            String::new()
        }
    };

    refusals.sort();
    refusals.dedup();
    CampaignVerdict {
        standing: if refusals.is_empty() {
            StrategicStanding::Alive
        } else {
            StrategicStanding::Refused
        },
        candidate_id: candidate.candidate_id.clone(),
        candidate_digest,
        refusals,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSelectionRequest {
    pub candidate_id: String,
    pub selection_authority_digest: String,
    pub selected_by: String,
    pub selected_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicMandatePacket {
    pub subject: String,
    pub mandate_id: String,
    pub constitution_digest: String,
    pub candidate_id: String,
    pub candidate_digest: String,
    pub selection_authority_digest: String,
    pub selected_by: String,
    pub selected_at: String,
    pub evidence_requirements: Vec<String>,
    pub authority_ceiling: &'static str,
    pub actuation: &'static str,
    pub successor_boundary: &'static str,
    pub packet_digest: String,
}

impl StrategicMandatePacket {
    fn core_json(&self) -> Value {
        json!({
            "subject": self.subject,
            "mandate_id": self.mandate_id,
            "constitution_digest": self.constitution_digest,
            "candidate_id": self.candidate_id,
            "candidate_digest": self.candidate_digest,
            "selection_authority_digest": self.selection_authority_digest,
            "selected_by": self.selected_by,
            "selected_at": self.selected_at,
            "evidence_requirements": self.evidence_requirements,
            "authority_ceiling": self.authority_ceiling,
            "actuation": self.actuation,
            "successor_boundary": self.successor_boundary,
        })
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut value = self.core_json();
        if let Value::Object(map) = &mut value {
            map.insert(
                "packet_digest".to_string(),
                Value::String(self.packet_digest.clone()),
            );
        }
        value
    }
}

pub fn admit_board_selection(
    constitution: &BoardConstitution,
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    request: BoardSelectionRequest,
) -> Result<StrategicMandatePacket, String> {
    if !lowercase_hex_64(&request.selection_authority_digest) {
        return Err("REFUSED:INVALID_BOARD_SELECTION_AUTHORITY".to_string());
    }
    if request.selected_by.trim().is_empty() || request.selected_at.trim().is_empty() {
        return Err("REFUSED:INCOMPLETE_BOARD_SELECTION".to_string());
    }
    let matching_candidates: Vec<&CampaignCandidate> = candidates
        .iter()
        .filter(|candidate| candidate.candidate_id == request.candidate_id)
        .collect();
    if matching_candidates.is_empty() {
        return Err("REFUSED:UNKNOWN_CAMPAIGN_CANDIDATE".to_string());
    }
    if matching_candidates.len() != 1 {
        return Err("REFUSED:AMBIGUOUS_CAMPAIGN_CANDIDATE".to_string());
    }
    let candidate = matching_candidates[0];

    let matching_verdicts: Vec<&CampaignVerdict> = verdicts
        .iter()
        .filter(|verdict| verdict.candidate_id == request.candidate_id)
        .collect();
    if matching_verdicts.is_empty() {
        return Err("REFUSED:MISSING_CAMPAIGN_VERDICT".to_string());
    }
    if matching_verdicts.len() != 1 {
        return Err("REFUSED:AMBIGUOUS_CAMPAIGN_VERDICT".to_string());
    }
    let verdict = matching_verdicts[0];

    if verdict.standing != StrategicStanding::Alive {
        return Err("REFUSED:BOARD_SELECTED_REFUSED_CAMPAIGN".to_string());
    }
    if candidate.subject != constitution.subject
        || candidate.constitution_digest != constitution.constitution_digest
    {
        return Err("REFUSED:MANDATE_DRIFT".to_string());
    }
    let candidate_digest = candidate.candidate_digest()?;
    if verdict.candidate_digest != candidate_digest {
        return Err("REFUSED:STALE_CAMPAIGN_VERDICT".to_string());
    }

    let mut packet = StrategicMandatePacket {
        subject: constitution.subject.clone(),
        mandate_id: constitution.mandate_id.clone(),
        constitution_digest: constitution.constitution_digest.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_digest,
        selection_authority_digest: request.selection_authority_digest,
        selected_by: request.selected_by.trim().to_string(),
        selected_at: request.selected_at.trim().to_string(),
        evidence_requirements: constitution.evidence_requirements.clone(),
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING,
        actuation: STRATEGIC_ACTUATION,
        successor_boundary: STRATEGIC_SUCCESSOR_BOUNDARY,
        packet_digest: String::new(),
    };
    packet.packet_digest = canonical_digest(&packet.core_json())?;
    Ok(packet)
}

/// Bind an exact board-selected strategic mandate into CASTLE's existing
/// inert CONSTRUCT request. The mandate does not gain DO authority here; its
/// digest becomes part of the config graph that the normal CASTLE construct
/// receipt already hashes and signs.
///
/// The selection-authority digest is a reference to independently admitted
/// board authority evidence. This layer binds that identity but does not
/// implement key custody or signature verification; those remain external
/// evidence/authority concerns and the existing BRCE gate still decides DO.
pub fn bind_strategic_mandate_construct_request(
    mut request: crate::castle::ConstructRequest,
    mandate: &StrategicMandatePacket,
) -> Result<crate::castle::ConstructRequest, String> {
    if mandate.subject != request.subject {
        return Err("REFUSED:STRATEGIC_MANDATE_SUBJECT_MISMATCH".to_string());
    }
    if mandate.authority_ceiling != STRATEGIC_AUTHORITY_CEILING
        || mandate.actuation != STRATEGIC_ACTUATION
        || mandate.successor_boundary != STRATEGIC_SUCCESSOR_BOUNDARY
    {
        return Err("REFUSED:STRATEGIC_MANDATE_AUTHORITY_DRIFT".to_string());
    }
    if !lowercase_hex_64(&mandate.constitution_digest)
        || !lowercase_hex_64(&mandate.candidate_digest)
        || !lowercase_hex_64(&mandate.selection_authority_digest)
        || !lowercase_hex_64(&mandate.packet_digest)
    {
        return Err("REFUSED:INVALID_STRATEGIC_MANDATE_DIGEST".to_string());
    }
    if canonical_digest(&mandate.core_json())? != mandate.packet_digest {
        return Err("REFUSED:STRATEGIC_MANDATE_CONTENT_MISMATCH".to_string());
    }

    let Value::Object(mut config) = request.config_graph else {
        return Err("REFUSED:CONFIG_GRAPH_NOT_OBJECT".to_string());
    };
    if config.contains_key("_castle_board_mandate") {
        return Err("REFUSED:STRATEGIC_MANDATE_ALREADY_BOUND".to_string());
    }

    config.insert(
        "_castle_board_mandate".to_string(),
        json!({
            "mandate_id": mandate.mandate_id,
            "constitution_digest": mandate.constitution_digest,
            "candidate_id": mandate.candidate_id,
            "candidate_digest": mandate.candidate_digest,
            "selection_authority_digest": mandate.selection_authority_digest,
            "selected_by": mandate.selected_by,
            "selected_at": mandate.selected_at,
            "packet_digest": mandate.packet_digest,
            "authority_ceiling": mandate.authority_ceiling,
            "actuation": mandate.actuation,
            "successor_boundary": mandate.successor_boundary,
        }),
    );
    request.config_graph = Value::Object(config);
    Ok(request)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecompileScope {
    None,
    Campaign(Vec<String>),
    Strategic,
}

#[must_use]
pub fn determine_recompile_scope(
    doctrine: &StrategyDoctrine,
    partitions: &[StrategyPartition],
    current_global_premises: &BTreeMap<String, String>,
    current_local_premises: &BTreeMap<String, String>,
) -> RecompileScope {
    if &doctrine.premise_digests != current_global_premises {
        return RecompileScope::Strategic;
    }

    let mut stale = Vec::new();
    for partition in partitions {
        if partition
            .local_premise_digests
            .iter()
            .any(|(key, digest)| current_local_premises.get(key) != Some(digest))
        {
            stale.push(partition.strategy_id.clone());
        }
    }
    stale.sort();
    stale.dedup();
    if stale.is_empty() {
        RecompileScope::None
    } else {
        RecompileScope::Campaign(stale)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplanLevel {
    PolicyBranch,
    SuffixReuse,
    FollowBiasedTailRepair,
    BoundedFullReplan,
    HierarchyRecompile,
    StrategicRecompile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DivergenceEvidence {
    pub policy_branch_available: bool,
    pub valid_suffix_available: bool,
    pub tail_repair_available: bool,
    pub hierarchy_premise_broken: bool,
    pub strategic_premise_broken: bool,
}

#[must_use]
pub const fn route_replan(evidence: DivergenceEvidence) -> ReplanLevel {
    if evidence.strategic_premise_broken {
        ReplanLevel::StrategicRecompile
    } else if evidence.hierarchy_premise_broken {
        ReplanLevel::HierarchyRecompile
    } else if evidence.policy_branch_available {
        ReplanLevel::PolicyBranch
    } else if evidence.valid_suffix_available {
        ReplanLevel::SuffixReuse
    } else if evidence.tail_repair_available {
        ReplanLevel::FollowBiasedTailRepair
    } else {
        ReplanLevel::BoundedFullReplan
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardReceiptInput {
    pub falsified_premises: Vec<String>,
    pub material_exceptions: Vec<String>,
    pub options_remaining: u32,
    pub prohibited_outcome_witnesses: Vec<String>,
    pub authority_expansions: Vec<String>,
    pub next_board_decision: Option<String>,
    pub evidence_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardStrategicReceipt {
    pub standing: StrategicStanding,
    pub mandate_id: String,
    pub constitution_digest: String,
    pub campaign_id: String,
    pub campaign_digest: String,
    pub falsified_premises: Vec<String>,
    pub material_exceptions: Vec<String>,
    pub options_remaining: u32,
    pub prohibited_outcome_witnesses: Vec<String>,
    pub authority_expansions: Vec<String>,
    pub next_board_decision: Option<String>,
    pub evidence_digest: String,
    pub authority_ceiling: &'static str,
    pub actuation: &'static str,
    pub receipt_digest: String,
}

impl BoardStrategicReceipt {
    fn core_json(&self) -> Value {
        json!({
            "standing": self.standing.as_str(),
            "mandate_id": self.mandate_id,
            "constitution_digest": self.constitution_digest,
            "campaign_id": self.campaign_id,
            "campaign_digest": self.campaign_digest,
            "falsified_premises": self.falsified_premises,
            "material_exceptions": self.material_exceptions,
            "options_remaining": self.options_remaining,
            "prohibited_outcome_witnesses": self.prohibited_outcome_witnesses,
            "authority_expansions": self.authority_expansions,
            "next_board_decision": self.next_board_decision,
            "evidence_digest": self.evidence_digest,
            "authority_ceiling": self.authority_ceiling,
            "actuation": self.actuation,
        })
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut value = self.core_json();
        if let Value::Object(map) = &mut value {
            map.insert(
                "receipt_digest".to_string(),
                Value::String(self.receipt_digest.clone()),
            );
        }
        value
    }
}

pub fn compile_board_strategic_receipt(
    constitution: &BoardConstitution,
    candidate: &CampaignCandidate,
    mandate: &StrategicMandatePacket,
    input: BoardReceiptInput,
) -> Result<BoardStrategicReceipt, String> {
    if !lowercase_hex_64(&input.evidence_digest) {
        return Err("REFUSED:INVALID_BOARD_EVIDENCE_DIGEST".to_string());
    }
    if mandate.candidate_id != candidate.candidate_id
        || mandate.constitution_digest != constitution.constitution_digest
        || mandate.candidate_digest != candidate.candidate_digest()?
    {
        return Err("REFUSED:BOARD_RECEIPT_SUBJECT_DRIFT".to_string());
    }
    let prohibited_outcome_witnesses = sorted_unique(input.prohibited_outcome_witnesses);
    let authority_expansions = sorted_unique(input.authority_expansions);
    let standing = if prohibited_outcome_witnesses.is_empty() && authority_expansions.is_empty() {
        StrategicStanding::Alive
    } else {
        StrategicStanding::Refused
    };

    let mut receipt = BoardStrategicReceipt {
        standing,
        mandate_id: constitution.mandate_id.clone(),
        constitution_digest: constitution.constitution_digest.clone(),
        campaign_id: candidate.candidate_id.clone(),
        campaign_digest: candidate.candidate_digest()?,
        falsified_premises: sorted_unique(input.falsified_premises),
        material_exceptions: sorted_unique(input.material_exceptions),
        options_remaining: input.options_remaining,
        prohibited_outcome_witnesses,
        authority_expansions,
        next_board_decision: input
            .next_board_decision
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        evidence_digest: input.evidence_digest,
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING,
        actuation: STRATEGIC_ACTUATION,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = canonical_digest(&receipt.core_json())?;
    Ok(receipt)
}
