//! Board operating loop layered on top of strategic_command.
//!
//! The compiler/authority core remains in strategic_command. This module closes
//! the board loop with competitive counterstrategy courts, bounded campaign
//! portfolio analysis, materiality-triggered board re-entry, a strategic digital
//! twin delta, and an offline-verifiable board package. None of these surfaces
//! can manufacture DO authority or select a material campaign.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::strategic_command::{
    BoardConstitution, BoardStrategicReceipt, CampaignCandidate, CampaignVerdict,
    StrategicMandatePacket, StrategicStanding, StrategyPartition, STRATEGIC_ACTUATION,
    STRATEGIC_AUTHORITY_CEILING,
};

fn digest_value(value: &Value) -> String {
    blake3::hash(crate::castle::canonical_json(value).as_bytes())
        .to_hex()
        .to_string()
}

fn lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn mandate_core_json(mandate: &StrategicMandatePacket) -> Value {
    json!({
        "subject": mandate.subject,
        "mandate_id": mandate.mandate_id,
        "constitution_digest": mandate.constitution_digest,
        "candidate_id": mandate.candidate_id,
        "candidate_digest": mandate.candidate_digest,
        "selection_authority_digest": mandate.selection_authority_digest,
        "selected_by": mandate.selected_by,
        "selected_at": mandate.selected_at,
        "evidence_requirements": mandate.evidence_requirements,
        "authority_ceiling": mandate.authority_ceiling,
        "actuation": mandate.actuation,
        "successor_boundary": mandate.successor_boundary,
    })
}

fn receipt_core_json(receipt: &BoardStrategicReceipt) -> Value {
    json!({
        "standing": receipt.standing.as_str(),
        "mandate_id": receipt.mandate_id,
        "constitution_digest": receipt.constitution_digest,
        "campaign_id": receipt.campaign_id,
        "campaign_digest": receipt.campaign_digest,
        "falsified_premises": receipt.falsified_premises,
        "material_exceptions": receipt.material_exceptions,
        "options_remaining": receipt.options_remaining,
        "prohibited_outcome_witnesses": receipt.prohibited_outcome_witnesses,
        "authority_expansions": receipt.authority_expansions,
        "next_board_decision": receipt.next_board_decision,
        "evidence_digest": receipt.evidence_digest,
        "authority_ceiling": receipt.authority_ceiling,
        "actuation": receipt.actuation,
    })
}

fn verify_mandate(mandate: &StrategicMandatePacket) -> Result<(), String> {
    if !lower_hex_64(&mandate.packet_digest)
        || !lower_hex_64(&mandate.constitution_digest)
        || !lower_hex_64(&mandate.candidate_digest)
        || !lower_hex_64(&mandate.selection_authority_digest)
    {
        return Err("REFUSED:INVALID_STRATEGIC_MANDATE_DIGEST".to_string());
    }
    if mandate.authority_ceiling != STRATEGIC_AUTHORITY_CEILING
        || mandate.actuation != STRATEGIC_ACTUATION
    {
        return Err("REFUSED:STRATEGIC_MANDATE_AUTHORITY_DRIFT".to_string());
    }
    if digest_value(&mandate_core_json(mandate)) != mandate.packet_digest {
        return Err("REFUSED:STRATEGIC_MANDATE_CONTENT_MISMATCH".to_string());
    }
    Ok(())
}

fn verify_receipt(receipt: &BoardStrategicReceipt) -> Result<(), String> {
    if !lower_hex_64(&receipt.receipt_digest)
        || !lower_hex_64(&receipt.constitution_digest)
        || !lower_hex_64(&receipt.campaign_digest)
        || !lower_hex_64(&receipt.evidence_digest)
    {
        return Err("REFUSED:INVALID_STRATEGIC_RECEIPT_DIGEST".to_string());
    }
    if receipt.authority_ceiling != STRATEGIC_AUTHORITY_CEILING
        || receipt.actuation != STRATEGIC_ACTUATION
    {
        return Err("REFUSED:STRATEGIC_RECEIPT_AUTHORITY_DRIFT".to_string());
    }
    if digest_value(&receipt_core_json(receipt)) != receipt.receipt_digest {
        return Err("REFUSED:STRATEGIC_RECEIPT_CONTENT_MISMATCH".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterstrategyScenario {
    pub scenario_id: String,
    pub local_premise_mutations: BTreeMap<String, String>,
    pub falsifier_triggered: bool,
    pub prohibited_outcomes_reached: Vec<String>,
    pub authority_expansion_attempts: Vec<String>,
    pub additional_capital_required: u64,
    pub remaining_options: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterstrategyVerdict {
    pub scenario_id: String,
    pub standing: StrategicStanding,
    pub refusals: Vec<String>,
    pub verdict_digest: String,
}

fn counterstrategy_verdict_core(verdict: &CounterstrategyVerdict) -> Value {
    json!({
        "scenario_id": verdict.scenario_id,
        "standing": verdict.standing.as_str(),
        "refusals": verdict.refusals,
    })
}

pub fn judge_counterstrategy(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenario: &CounterstrategyScenario,
) -> Result<CounterstrategyVerdict, String> {
    let scenario_id = scenario.scenario_id.trim();
    if scenario_id.is_empty() {
        return Err("REFUSED:EMPTY_COUNTERSTRATEGY_SCENARIO".to_string());
    }
    if candidate.subject != constitution.subject
        || candidate.constitution_digest != constitution.constitution_digest
    {
        return Err("REFUSED:COUNTERSTRATEGY_SUBJECT_DRIFT".to_string());
    }
    if candidate.partition_digest != partition.partition_digest
        || candidate.strategy_id != partition.strategy_id
    {
        return Err("REFUSED:COUNTERSTRATEGY_PARTITION_DRIFT".to_string());
    }
    for (premise, digest) in &scenario.local_premise_mutations {
        if premise.trim().is_empty() || !lower_hex_64(digest) {
            return Err("REFUSED:INVALID_COUNTERSTRATEGY_PREMISE".to_string());
        }
    }

    let mut refusals = Vec::new();
    if scenario.falsifier_triggered {
        refusals.push("REFUSED:CAMPAIGN_FALSIFIER_TRIGGERED".to_string());
    }
    for (premise, compiled) in &partition.local_premise_digests {
        if scenario
            .local_premise_mutations
            .get(premise)
            .is_some_and(|mutated| mutated != compiled)
        {
            refusals.push(format!(
                "REFUSED:COUNTERSTRATEGY_FALSIFIED_PREMISE:{premise}"
            ));
        }
    }
    if scenario.prohibited_outcomes_reached.iter().any(|outcome| {
        constitution
            .prohibited_outcomes
            .iter()
            .any(|prohibited| prohibited == outcome)
    }) {
        refusals.push("REFUSED:COUNTERSTRATEGY_REACHES_PROHIBITED_OUTCOME".to_string());
    }
    if !scenario.authority_expansion_attempts.is_empty() {
        refusals.push("REFUSED:COUNTERSTRATEGY_REQUIRES_AUTHORITY_EXPANSION".to_string());
    }
    let projected_capital = candidate
        .capital_committed
        .checked_add(scenario.additional_capital_required)
        .ok_or_else(|| "REFUSED:COUNTERSTRATEGY_CAPITAL_OVERFLOW".to_string())?;
    if projected_capital > constitution.capital_at_risk_limit {
        refusals.push("REFUSED:COUNTERSTRATEGY_EXCEEDS_CAPITAL_BOUND".to_string());
    }
    if scenario.remaining_options == 0 {
        refusals.push("REFUSED:COUNTERSTRATEGY_EXHAUSTS_OPTION_SPACE".to_string());
    }

    refusals.sort();
    refusals.dedup();
    let mut verdict = CounterstrategyVerdict {
        scenario_id: scenario_id.to_string(),
        standing: if refusals.is_empty() {
            StrategicStanding::Alive
        } else {
            StrategicStanding::Refused
        },
        refusals,
        verdict_digest: String::new(),
    };
    verdict.verdict_digest = digest_value(&counterstrategy_verdict_core(&verdict));
    Ok(verdict)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterstrategyAssessment {
    pub campaign_id: String,
    pub campaign_digest: String,
    pub standing: StrategicStanding,
    pub verdicts: Vec<CounterstrategyVerdict>,
    pub assessment_digest: String,
}

fn counterstrategy_assessment_core(assessment: &CounterstrategyAssessment) -> Value {
    json!({
        "campaign_id": assessment.campaign_id,
        "campaign_digest": assessment.campaign_digest,
        "standing": assessment.standing.as_str(),
        "verdicts": assessment.verdicts.iter().map(|verdict| json!({
            "scenario_id": verdict.scenario_id,
            "standing": verdict.standing.as_str(),
            "refusals": verdict.refusals,
            "verdict_digest": verdict.verdict_digest,
        })).collect::<Vec<_>>(),
    })
}

pub fn assess_counterstrategies(
    constitution: &BoardConstitution,
    partition: &StrategyPartition,
    candidate: &CampaignCandidate,
    scenarios: &[CounterstrategyScenario],
) -> Result<CounterstrategyAssessment, String> {
    if scenarios.is_empty() {
        return Err("REFUSED:MISSING_COUNTERSTRATEGY_SCENARIOS".to_string());
    }
    let mut ids = BTreeSet::new();
    let mut verdicts = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        if !ids.insert(scenario.scenario_id.trim().to_string()) {
            return Err("REFUSED:AMBIGUOUS_COUNTERSTRATEGY_SCENARIO".to_string());
        }
        verdicts.push(judge_counterstrategy(
            constitution,
            partition,
            candidate,
            scenario,
        )?);
    }
    verdicts.sort_by(|left, right| left.scenario_id.cmp(&right.scenario_id));
    let standing = if verdicts
        .iter()
        .all(|verdict| verdict.standing == StrategicStanding::Alive)
    {
        StrategicStanding::Alive
    } else {
        StrategicStanding::Refused
    };
    let mut assessment = CounterstrategyAssessment {
        campaign_id: candidate.candidate_id.clone(),
        campaign_digest: candidate.candidate_digest()?,
        standing,
        verdicts,
        assessment_digest: String::new(),
    };
    assessment.assessment_digest = digest_value(&counterstrategy_assessment_core(&assessment));
    Ok(assessment)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignPortfolioPolicy {
    pub aggregate_capital_at_risk_limit: u64,
    pub max_single_campaign_concentration_bps: u64,
    pub min_reversible_capital_bps: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignPortfolioAnalysis {
    pub standing: StrategicStanding,
    pub subject: String,
    pub constitution_digest: String,
    pub campaign_ids: Vec<String>,
    pub aggregate_capital_committed: u64,
    pub aggregate_reversible_capital: u64,
    pub reversible_capital_bps: u64,
    pub max_single_campaign_concentration_bps: u64,
    pub reasons: Vec<String>,
    pub analysis_digest: String,
}

fn portfolio_core(analysis: &CampaignPortfolioAnalysis) -> Value {
    json!({
        "standing": analysis.standing.as_str(),
        "subject": analysis.subject,
        "constitution_digest": analysis.constitution_digest,
        "campaign_ids": analysis.campaign_ids,
        "aggregate_capital_committed": analysis.aggregate_capital_committed,
        "aggregate_reversible_capital": analysis.aggregate_reversible_capital,
        "reversible_capital_bps": analysis.reversible_capital_bps,
        "max_single_campaign_concentration_bps": analysis.max_single_campaign_concentration_bps,
        "reasons": analysis.reasons,
    })
}

pub fn qualify_campaign_portfolio(
    candidates: &[CampaignCandidate],
    verdicts: &[CampaignVerdict],
    policy: &CampaignPortfolioPolicy,
) -> Result<CampaignPortfolioAnalysis, String> {
    let Some(first) = candidates.first() else {
        return Err("REFUSED:EMPTY_CAMPAIGN_PORTFOLIO".to_string());
    };
    if policy.aggregate_capital_at_risk_limit == 0
        || policy.max_single_campaign_concentration_bps > 10_000
        || policy.min_reversible_capital_bps > 10_000
    {
        return Err("REFUSED:INVALID_CAMPAIGN_PORTFOLIO_POLICY".to_string());
    }

    let subject = first.subject.clone();
    let constitution_digest = first.constitution_digest.clone();
    let mut ids = BTreeSet::new();
    let mut reasons = Vec::new();
    let mut aggregate_capital = 0u64;
    let mut aggregate_reversible = 0u64;
    let mut max_single = 0u64;

    for candidate in candidates {
        if candidate.subject != subject || candidate.constitution_digest != constitution_digest {
            return Err("REFUSED:PORTFOLIO_SUBJECT_DRIFT".to_string());
        }
        if !ids.insert(candidate.candidate_id.clone()) {
            return Err("REFUSED:AMBIGUOUS_CAMPAIGN_CANDIDATE".to_string());
        }
        let matching: Vec<&CampaignVerdict> = verdicts
            .iter()
            .filter(|verdict| verdict.candidate_id == candidate.candidate_id)
            .collect();
        if matching.len() != 1 {
            return Err(if matching.is_empty() {
                "REFUSED:MISSING_CAMPAIGN_VERDICT".to_string()
            } else {
                "REFUSED:AMBIGUOUS_CAMPAIGN_VERDICT".to_string()
            });
        }
        let verdict = matching[0];
        let candidate_digest = candidate.candidate_digest()?;
        if verdict.candidate_digest != candidate_digest {
            return Err("REFUSED:STALE_CAMPAIGN_VERDICT".to_string());
        }
        if verdict.standing != StrategicStanding::Alive {
            reasons.push(format!(
                "REFUSED:PORTFOLIO_CONTAINS_REFUSED_CAMPAIGN:{}",
                candidate.candidate_id
            ));
        }

        aggregate_capital = aggregate_capital
            .checked_add(candidate.capital_committed)
            .ok_or_else(|| "REFUSED:PORTFOLIO_CAPITAL_OVERFLOW".to_string())?;
        aggregate_reversible = aggregate_reversible
            .checked_add(candidate.reversible_capital)
            .ok_or_else(|| "REFUSED:PORTFOLIO_CAPITAL_OVERFLOW".to_string())?;
        max_single = max_single.max(candidate.capital_committed);
    }

    if aggregate_capital > policy.aggregate_capital_at_risk_limit {
        reasons.push("REFUSED:PORTFOLIO_CAPITAL_BOUND_EXCEEDED".to_string());
    }
    let reversible_bps = if aggregate_capital == 0 {
        10_000
    } else {
        ((u128::from(aggregate_reversible) * 10_000) / u128::from(aggregate_capital)) as u64
    };
    let concentration_bps = if aggregate_capital == 0 {
        0
    } else {
        ((u128::from(max_single) * 10_000) / u128::from(aggregate_capital)) as u64
    };
    if reversible_bps < policy.min_reversible_capital_bps {
        reasons.push("REFUSED:PORTFOLIO_REVERSIBILITY_BELOW_BOUND".to_string());
    }
    if concentration_bps > policy.max_single_campaign_concentration_bps {
        reasons.push("REFUSED:PORTFOLIO_CONCENTRATION_BOUND_EXCEEDED".to_string());
    }

    reasons.sort();
    reasons.dedup();
    let mut analysis = CampaignPortfolioAnalysis {
        standing: if reasons.is_empty() {
            StrategicStanding::Alive
        } else {
            StrategicStanding::Refused
        },
        subject,
        constitution_digest,
        campaign_ids: ids.into_iter().collect(),
        aggregate_capital_committed: aggregate_capital,
        aggregate_reversible_capital: aggregate_reversible,
        reversible_capital_bps: reversible_bps,
        max_single_campaign_concentration_bps: concentration_bps,
        reasons,
        analysis_digest: String::new(),
    };
    analysis.analysis_digest = digest_value(&portfolio_core(&analysis));
    Ok(analysis)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardReentryDecision {
    pub required: bool,
    pub reasons: Vec<String>,
    pub escalate_by_epoch_ms: Option<i64>,
    pub materiality_score_bps: i64,
    pub triggering_dimensions: Vec<crate::board::MaterialityDimension>,
}

pub fn assess_board_reentry(
    constitution: &BoardConstitution,
    strategic_receipt: &BoardStrategicReceipt,
    event: &crate::board::MaterialityEvent,
    policy: &crate::board::MaterialityPolicy,
) -> Result<BoardReentryDecision, String> {
    if event.subject != constitution.subject
        || strategic_receipt.constitution_digest != constitution.constitution_digest
        || strategic_receipt.mandate_id != constitution.mandate_id
    {
        return Err("REFUSED:BOARD_REENTRY_SUBJECT_DRIFT".to_string());
    }
    verify_receipt(strategic_receipt)?;

    let materiality = crate::board::assess_materiality(event, policy)?;
    let mut reasons = Vec::new();
    if materiality.material {
        reasons.push("MATERIALITY_THRESHOLD_CROSSED".to_string());
    }
    if strategic_receipt.standing == StrategicStanding::Refused {
        reasons.push("STRATEGIC_RECEIPT_REFUSED".to_string());
    }
    if !strategic_receipt.falsified_premises.is_empty() {
        reasons.push("STRATEGIC_PREMISE_FALSIFIED".to_string());
    }
    if !strategic_receipt.material_exceptions.is_empty() {
        reasons.push("MATERIAL_EXCEPTION_RECORDED".to_string());
    }
    if strategic_receipt.options_remaining == 0 {
        reasons.push("OPTION_SPACE_EXHAUSTED".to_string());
    }
    reasons.sort();
    reasons.dedup();

    let required = !reasons.is_empty();
    Ok(BoardReentryDecision {
        required,
        reasons,
        escalate_by_epoch_ms: if required {
            materiality
                .escalate_by_epoch_ms
                .or(Some(event.occurred_at_epoch_ms))
        } else {
            None
        },
        materiality_score_bps: materiality.score_bps,
        triggering_dimensions: materiality.triggering_dimensions,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicTwinSnapshot {
    pub subject: String,
    pub mandate_id: String,
    pub campaign_id: String,
    pub standing: StrategicStanding,
    pub aggregate_capital_committed: u64,
    pub aggregate_reversible_capital: u64,
    pub options_remaining: u32,
    pub falsified_premises: Vec<String>,
    pub failed_counterstrategy_scenarios: Vec<String>,
    pub prohibited_outcome_witnesses: Vec<String>,
    pub authority_expansions: Vec<String>,
    pub snapshot_digest: String,
}

fn twin_core(twin: &StrategicTwinSnapshot) -> Value {
    json!({
        "subject": twin.subject,
        "mandate_id": twin.mandate_id,
        "campaign_id": twin.campaign_id,
        "standing": twin.standing.as_str(),
        "aggregate_capital_committed": twin.aggregate_capital_committed,
        "aggregate_reversible_capital": twin.aggregate_reversible_capital,
        "options_remaining": twin.options_remaining,
        "falsified_premises": twin.falsified_premises,
        "failed_counterstrategy_scenarios": twin.failed_counterstrategy_scenarios,
        "prohibited_outcome_witnesses": twin.prohibited_outcome_witnesses,
        "authority_expansions": twin.authority_expansions,
    })
}

pub fn build_strategic_twin_snapshot(
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
) -> Result<StrategicTwinSnapshot, String> {
    verify_mandate(mandate)?;
    verify_receipt(receipt)?;
    if digest_value(&portfolio_core(portfolio)) != portfolio.analysis_digest
        || digest_value(&counterstrategy_assessment_core(counterstrategy))
            != counterstrategy.assessment_digest
    {
        return Err("REFUSED:STRATEGIC_TWIN_INPUT_CONTENT_MISMATCH".to_string());
    }
    if mandate.subject != constitution.subject
        || mandate.constitution_digest != constitution.constitution_digest
        || receipt.constitution_digest != constitution.constitution_digest
        || receipt.mandate_id != constitution.mandate_id
        || receipt.campaign_id != mandate.candidate_id
        || portfolio.subject != constitution.subject
        || portfolio.constitution_digest != constitution.constitution_digest
        || counterstrategy.campaign_id != mandate.candidate_id
        || counterstrategy.campaign_digest != mandate.candidate_digest
    {
        return Err("REFUSED:STRATEGIC_TWIN_SUBJECT_DRIFT".to_string());
    }

    let failed_counterstrategy_scenarios = counterstrategy
        .verdicts
        .iter()
        .filter(|verdict| verdict.standing == StrategicStanding::Refused)
        .map(|verdict| verdict.scenario_id.clone())
        .collect::<Vec<_>>();
    let standing = if receipt.standing == StrategicStanding::Alive
        && portfolio.standing == StrategicStanding::Alive
        && counterstrategy.standing == StrategicStanding::Alive
    {
        StrategicStanding::Alive
    } else {
        StrategicStanding::Refused
    };

    let mut twin = StrategicTwinSnapshot {
        subject: constitution.subject.clone(),
        mandate_id: constitution.mandate_id.clone(),
        campaign_id: mandate.candidate_id.clone(),
        standing,
        aggregate_capital_committed: portfolio.aggregate_capital_committed,
        aggregate_reversible_capital: portfolio.aggregate_reversible_capital,
        options_remaining: receipt.options_remaining,
        falsified_premises: receipt.falsified_premises.clone(),
        failed_counterstrategy_scenarios,
        prohibited_outcome_witnesses: receipt.prohibited_outcome_witnesses.clone(),
        authority_expansions: receipt.authority_expansions.clone(),
        snapshot_digest: String::new(),
    };
    twin.snapshot_digest = digest_value(&twin_core(&twin));
    Ok(twin)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicBoardDelta {
    pub subject: String,
    pub mandate_id: String,
    pub changed_dimensions: Vec<String>,
    pub requires_board_attention: bool,
    pub previous_snapshot_digest: String,
    pub current_snapshot_digest: String,
}

pub fn diff_strategic_twins(
    previous: &StrategicTwinSnapshot,
    current: &StrategicTwinSnapshot,
) -> Result<StrategicBoardDelta, String> {
    if previous.subject != current.subject || previous.mandate_id != current.mandate_id {
        return Err("REFUSED:STRATEGIC_TWIN_LINEAGE_DRIFT".to_string());
    }
    if !lower_hex_64(&previous.snapshot_digest) || !lower_hex_64(&current.snapshot_digest) {
        return Err("REFUSED:INVALID_STRATEGIC_TWIN_DIGEST".to_string());
    }

    let mut changes = Vec::new();
    if previous.campaign_id != current.campaign_id {
        changes.push("campaign".to_string());
    }
    if previous.standing != current.standing {
        changes.push("standing".to_string());
    }
    if previous.aggregate_capital_committed != current.aggregate_capital_committed {
        changes.push("capital-committed".to_string());
    }
    if previous.aggregate_reversible_capital != current.aggregate_reversible_capital {
        changes.push("capital-reversible".to_string());
    }
    if previous.options_remaining != current.options_remaining {
        changes.push("options-remaining".to_string());
    }
    if previous.falsified_premises != current.falsified_premises {
        changes.push("falsified-premises".to_string());
    }
    if previous.failed_counterstrategy_scenarios != current.failed_counterstrategy_scenarios {
        changes.push("counterstrategy".to_string());
    }
    if previous.prohibited_outcome_witnesses != current.prohibited_outcome_witnesses {
        changes.push("prohibited-outcomes".to_string());
    }
    if previous.authority_expansions != current.authority_expansions {
        changes.push("authority".to_string());
    }
    changes.sort();
    changes.dedup();

    let requires_board_attention = current.standing == StrategicStanding::Refused
        || current.options_remaining < previous.options_remaining
        || current.aggregate_capital_committed > previous.aggregate_capital_committed
        || current.falsified_premises.len() > previous.falsified_premises.len()
        || current.failed_counterstrategy_scenarios.len()
            > previous.failed_counterstrategy_scenarios.len()
        || current.prohibited_outcome_witnesses.len()
            > previous.prohibited_outcome_witnesses.len()
        || current.authority_expansions.len() > previous.authority_expansions.len();

    Ok(StrategicBoardDelta {
        subject: current.subject.clone(),
        mandate_id: current.mandate_id.clone(),
        changed_dimensions: changes,
        requires_board_attention,
        previous_snapshot_digest: previous.snapshot_digest.clone(),
        current_snapshot_digest: current.snapshot_digest.clone(),
    })
}

fn fortune5_board_package_digest(package: &crate::board::BoardPackage) -> String {
    digest_value(&json!({
        "profile": package.profile,
        "enterprise_subject": package.enterprise_subject,
        "castle_subject": package.castle_subject,
        "generated_at": package.generated_at,
        "enterprise_standing": package.enterprise_standing.as_str(),
        "castle_standing": package.castle_standing.as_str(),
        "material_refused_subjects": package.material_refused_subjects,
        "risk_appetite_breaches": package.risk_appetite_breaches,
        "control_count": package.control_count,
        "evidence_digest": package.evidence_digest,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategicBoardPackage {
    pub profile: &'static str,
    pub subject: String,
    pub generated_at: String,
    pub fortune5_board_package_digest: String,
    pub constitution_digest: String,
    pub mandate_packet_digest: String,
    pub strategic_receipt_digest: String,
    pub portfolio_analysis_digest: String,
    pub counterstrategy_assessment_digest: String,
    pub twin_snapshot_digest: String,
    pub strategic_standing: StrategicStanding,
    pub board_reentry_required: bool,
    pub authority_ceiling: &'static str,
    pub actuation: &'static str,
    pub package_digest: String,
}

fn strategic_package_core(package: &StrategicBoardPackage) -> Value {
    json!({
        "profile": package.profile,
        "subject": package.subject,
        "generated_at": package.generated_at,
        "fortune5_board_package_digest": package.fortune5_board_package_digest,
        "constitution_digest": package.constitution_digest,
        "mandate_packet_digest": package.mandate_packet_digest,
        "strategic_receipt_digest": package.strategic_receipt_digest,
        "portfolio_analysis_digest": package.portfolio_analysis_digest,
        "counterstrategy_assessment_digest": package.counterstrategy_assessment_digest,
        "twin_snapshot_digest": package.twin_snapshot_digest,
        "strategic_standing": package.strategic_standing.as_str(),
        "board_reentry_required": package.board_reentry_required,
        "authority_ceiling": package.authority_ceiling,
        "actuation": package.actuation,
    })
}

pub fn build_strategic_board_package(
    base: &crate::board::BoardPackage,
    constitution: &BoardConstitution,
    mandate: &StrategicMandatePacket,
    receipt: &BoardStrategicReceipt,
    portfolio: &CampaignPortfolioAnalysis,
    counterstrategy: &CounterstrategyAssessment,
    twin: &StrategicTwinSnapshot,
    reentry: &BoardReentryDecision,
    generated_at: &str,
) -> Result<StrategicBoardPackage, String> {
    if generated_at.trim().is_empty() {
        return Err("REFUSED:EMPTY_BOARD_PACKAGE_TIME".to_string());
    }
    verify_mandate(mandate)?;
    verify_receipt(receipt)?;
    if base.enterprise_standing != crate::fortune5::Standing::Alive
        || base.castle_standing != crate::fortune5::Standing::Alive
        || !lower_hex_64(&base.evidence_digest)
    {
        return Err("REFUSED:STRATEGIC_BOARD_PACKAGE_BASE_NOT_ALIVE".to_string());
    }
    if digest_value(&portfolio_core(portfolio)) != portfolio.analysis_digest
        || digest_value(&counterstrategy_assessment_core(counterstrategy))
            != counterstrategy.assessment_digest
        || digest_value(&twin_core(twin)) != twin.snapshot_digest
    {
        return Err("REFUSED:STRATEGIC_BOARD_PACKAGE_INPUT_CONTENT_MISMATCH".to_string());
    }
    if base.enterprise_subject != constitution.subject
        || mandate.subject != constitution.subject
        || mandate.constitution_digest != constitution.constitution_digest
        || receipt.constitution_digest != constitution.constitution_digest
        || receipt.campaign_id != mandate.candidate_id
        || portfolio.subject != constitution.subject
        || portfolio.constitution_digest != constitution.constitution_digest
        || twin.subject != constitution.subject
        || twin.mandate_id != constitution.mandate_id
    {
        return Err("REFUSED:STRATEGIC_BOARD_PACKAGE_SUBJECT_DRIFT".to_string());
    }
    if twin.standing == StrategicStanding::Refused && !reentry.required {
        return Err("REFUSED:REFUSED_STRATEGY_WITHOUT_BOARD_REENTRY".to_string());
    }

    let mut package = StrategicBoardPackage {
        profile: "CASTLE_STRATEGIC_BOARD_V1",
        subject: constitution.subject.clone(),
        generated_at: generated_at.trim().to_string(),
        fortune5_board_package_digest: fortune5_board_package_digest(base),
        constitution_digest: constitution.constitution_digest.clone(),
        mandate_packet_digest: mandate.packet_digest.clone(),
        strategic_receipt_digest: receipt.receipt_digest.clone(),
        portfolio_analysis_digest: portfolio.analysis_digest.clone(),
        counterstrategy_assessment_digest: counterstrategy.assessment_digest.clone(),
        twin_snapshot_digest: twin.snapshot_digest.clone(),
        strategic_standing: twin.standing,
        board_reentry_required: reentry.required,
        authority_ceiling: STRATEGIC_AUTHORITY_CEILING,
        actuation: STRATEGIC_ACTUATION,
        package_digest: String::new(),
    };
    package.package_digest = digest_value(&strategic_package_core(&package));
    Ok(package)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineBoardPackageVerification {
    pub standing: StrategicStanding,
    pub reasons: Vec<String>,
}

pub fn verify_strategic_board_package_offline(
    package: &StrategicBoardPackage,
) -> OfflineBoardPackageVerification {
    let mut reasons = Vec::new();
    if package.profile != "CASTLE_STRATEGIC_BOARD_V1" {
        reasons.push("REFUSED:UNSUPPORTED_STRATEGIC_BOARD_PACKAGE".to_string());
    }
    if package.authority_ceiling != STRATEGIC_AUTHORITY_CEILING
        || package.actuation != STRATEGIC_ACTUATION
    {
        reasons.push("REFUSED:STRATEGIC_BOARD_PACKAGE_AUTHORITY_DRIFT".to_string());
    }
    if package.strategic_standing == StrategicStanding::Refused
        && !package.board_reentry_required
    {
        reasons.push("REFUSED:REFUSED_STRATEGY_WITHOUT_BOARD_REENTRY".to_string());
    }
    for digest in [
        &package.fortune5_board_package_digest,
        &package.constitution_digest,
        &package.mandate_packet_digest,
        &package.strategic_receipt_digest,
        &package.portfolio_analysis_digest,
        &package.counterstrategy_assessment_digest,
        &package.twin_snapshot_digest,
        &package.package_digest,
    ] {
        if !lower_hex_64(digest) {
            reasons.push("REFUSED:INVALID_STRATEGIC_BOARD_PACKAGE_DIGEST".to_string());
            break;
        }
    }
    if digest_value(&strategic_package_core(package)) != package.package_digest {
        reasons.push("REFUSED:STRATEGIC_BOARD_PACKAGE_CONTENT_MISMATCH".to_string());
    }
    reasons.sort();
    reasons.dedup();
    OfflineBoardPackageVerification {
        standing: if reasons.is_empty() {
            StrategicStanding::Alive
        } else {
            StrategicStanding::Refused
        },
        reasons,
    }
}
