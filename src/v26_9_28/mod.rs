//! CASTLE v26.9.28 ecosystem interoperability layer.
//!
//! This module consumes exact external evidence without granting any external
//! component CASTLE actuation authority. It is intentionally upstream of the
//! existing opaque ConstructAdmission -> BRCE -> DO path.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const ECOSYSTEM_EPOCH: &str = "v26.9.28";
pub const MAX_EXTERNAL_WITNESS_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_EXTERNAL_WITNESS_STEPS: u64 = 1_000_000;
pub const MAX_EXTERNAL_WITNESS_DEADLINE_MS: u64 = 3_600_000;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MarketplacePack {
    pub repo: String,
    pub r#ref: String,
    pub commit_sha: String,
    pub path: String,
    pub ontology_sha: String,
    pub security_universe_sha: String,
    pub security_tools_sha: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SourceSubject {
    pub id: String,
    pub repo: String,
    pub source_kind: String,
    pub reference: String,
    pub sha: String,
    pub pr: Option<u64>,
    pub role: String,
    pub authority_ceiling: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExcludedPr {
    pub repo: String,
    pub pr: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReviewWindow {
    pub timezone: String,
    pub local_start: String,
    pub task_cutoff: String,
    pub utc_start: String,
    pub utc_cutoff: String,
    pub pr_count: u64,
    pub merged_pr_count: u64,
    pub open_pr_count: u64,
    pub closed_unmerged_pr_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EcosystemManifest {
    pub kind: String,
    pub release_epoch: String,
    pub reviewed_local_date: String,
    pub timezone: String,
    pub castle_base_sha: String,
    pub review_window: ReviewWindow,
    pub marketplace_pack: MarketplacePack,
    pub subjects: Vec<SourceSubject>,
    pub excluded_open_prs: Vec<ExcludedPr>,
}

pub fn ecosystem_manifest() -> Result<EcosystemManifest, String> {
    serde_json::from_str(include_str!("../../configs/ecosystem-v26.9.28.json"))
        .map_err(|e| format!("REFUSED:INVALID_ECOSYSTEM_MANIFEST:{e}"))
}

fn lowercase_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WitnessKind {
    Semantic,
    Receipt,
    Process,
    Planner,
    Recovery,
    Federation,
    ModelCompute,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WitnessLimits {
    pub max_steps: u64,
    pub max_bytes: u64,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExternalWitness {
    pub source_id: String,
    pub source_sha: String,
    pub subject: String,
    pub kind: WitnessKind,
    pub input_digest: String,
    pub output_digest: String,
    pub direct_do_authority: bool,
    pub limits: WitnessLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceStanding {
    Alive,
    Refused(String),
}

impl EvidenceStanding {
    #[must_use]
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive)
    }
}

pub fn admit_external_witness(
    manifest: &EcosystemManifest,
    witness: &ExternalWitness,
) -> EvidenceStanding {
    let Some(source) = manifest.subjects.iter().find(|s| s.id == witness.source_id) else {
        return EvidenceStanding::Refused("REFUSED:UNKNOWN_SOURCE_SUBJECT".to_string());
    };

    if source.sha != witness.source_sha || !lowercase_hex(&witness.source_sha, 40) {
        return EvidenceStanding::Refused("REFUSED:SOURCE_SUBJECT_DRIFT".to_string());
    }
    if witness.subject.trim().is_empty() {
        return EvidenceStanding::Refused("REFUSED:EMPTY_EXACT_SUBJECT".to_string());
    }
    if !lowercase_hex(&witness.input_digest, 64) || !lowercase_hex(&witness.output_digest, 64) {
        return EvidenceStanding::Refused("REFUSED:INVALID_WITNESS_DIGEST".to_string());
    }
    if witness.direct_do_authority {
        return EvidenceStanding::Refused("REFUSED:EXTERNAL_DO_AUTHORITY".to_string());
    }
    if witness.limits.max_steps == 0
        || witness.limits.max_steps > MAX_EXTERNAL_WITNESS_STEPS
        || witness.limits.max_bytes == 0
        || witness.limits.max_bytes > MAX_EXTERNAL_WITNESS_BYTES
        || witness.limits.deadline_ms == 0
        || witness.limits.deadline_ms > MAX_EXTERNAL_WITNESS_DEADLINE_MS
    {
        return EvidenceStanding::Refused("REFUSED:UNBOUNDED_EXTERNAL_WITNESS".to_string());
    }
    if source.authority_ceiling == "DO" || source.authority_ceiling == "CASTLE_DO" {
        return EvidenceStanding::Refused("REFUSED:SOURCE_AUTHORITY_CEILING".to_string());
    }

    EvidenceStanding::Alive
}


/// Bind admitted Sep-28 ecosystem evidence into a normal CASTLE ConstructRequest.
///
/// This does not manufacture a capability. It only enriches the inert config
/// graph so the existing `manufacture_construct_capability` path will include
/// the exact review manifest and witness set in its normal config receipt.
pub fn bind_v26_9_28_construct_request(
    mut request: crate::castle::ConstructRequest,
    manifest: &EcosystemManifest,
    witnesses: &[ExternalWitness],
) -> Result<crate::castle::ConstructRequest, String> {
    let qualification = qualify_v26_9_28_upgrade(manifest);
    if let EvidenceStanding::Refused(reason) = qualification.standing {
        return Err(reason);
    }

    for witness in witnesses {
        if let EvidenceStanding::Refused(reason) = admit_external_witness(manifest, witness) {
            return Err(reason);
        }
        if witness.subject != request.subject {
            return Err("REFUSED:EXTERNAL_WITNESS_SUBJECT_MISMATCH".to_string());
        }
    }

    let Value::Object(mut config) = request.config_graph else {
        return Err("REFUSED:CONFIG_GRAPH_NOT_OBJECT".to_string());
    };

    let manifest_value = serde_json::to_value(manifest)
        .map_err(|e| format!("REFUSED:MANIFEST_SERIALIZATION:{e}"))?;
    let manifest_digest = blake3::hash(crate::castle::canonical_json(&manifest_value).as_bytes())
        .to_hex()
        .to_string();

    let mut ordered = witnesses.to_vec();
    ordered.sort_by(|a, b| {
        (
            a.source_id.as_str(),
            a.subject.as_str(),
            a.kind,
            a.input_digest.as_str(),
            a.output_digest.as_str(),
        )
            .cmp(&(
                b.source_id.as_str(),
                b.subject.as_str(),
                b.kind,
                b.input_digest.as_str(),
                b.output_digest.as_str(),
            ))
    });
    let witness_value = serde_json::to_value(&ordered)
        .map_err(|e| format!("REFUSED:WITNESS_SERIALIZATION:{e}"))?;
    let witness_digest = blake3::hash(crate::castle::canonical_json(&witness_value).as_bytes())
        .to_hex()
        .to_string();

    config.insert(
        "_castle_v26_9_28".to_string(),
        json!({
            "epoch": ECOSYSTEM_EPOCH,
            "manifest_digest": manifest_digest,
            "marketplace_commit": manifest.marketplace_pack.commit_sha,
            "witness_digest": witness_digest,
            "witness_count": ordered.len(),
            "sources": ordered.iter().map(|w| json!({
                "id": w.source_id,
                "sha": w.source_sha,
                "subject": w.subject,
                "kind": w.kind,
                "input_digest": w.input_digest,
                "output_digest": w.output_digest,
            })).collect::<Vec<_>>(),
        }),
    );
    request.config_graph = Value::Object(config);
    Ok(request)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortableRuntimeWitness {
    pub engine_family: String,
    pub module_digest: String,
    pub input_digest: String,
    pub output_digest: String,
}

pub fn qualify_portable_runtime(witnesses: &[PortableRuntimeWitness]) -> EvidenceStanding {
    if witnesses.len() < 2 {
        return EvidenceStanding::Refused("REFUSED:INSUFFICIENT_RUNTIME_DIVERSITY".to_string());
    }
    let first = &witnesses[0];
    if !lowercase_hex(&first.module_digest, 64)
        || !lowercase_hex(&first.input_digest, 64)
        || !lowercase_hex(&first.output_digest, 64)
    {
        return EvidenceStanding::Refused("REFUSED:INVALID_PORTABILITY_DIGEST".to_string());
    }

    let mut engines = BTreeSet::new();
    for witness in witnesses {
        if witness.module_digest != first.module_digest
            || witness.input_digest != first.input_digest
            || witness.output_digest != first.output_digest
        {
            return EvidenceStanding::Refused("REFUSED:RUNTIME_WITNESS_DIVERGENCE".to_string());
        }
        if witness.engine_family.trim().is_empty() {
            return EvidenceStanding::Refused("REFUSED:EMPTY_RUNTIME_ENGINE".to_string());
        }
        engines.insert(witness.engine_family.as_str());
    }

    if engines.len() < 2 {
        return EvidenceStanding::Refused("REFUSED:INSUFFICIENT_RUNTIME_DIVERSITY".to_string());
    }
    EvidenceStanding::Alive
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependentPlanCheck {
    pub plan_digest: String,
    pub valid_under_independent_dynamics: bool,
    pub claimed_cost: i64,
    pub independently_verified_cost: i64,
}

pub fn admit_independent_plan(check: &IndependentPlanCheck) -> EvidenceStanding {
    if !lowercase_hex(&check.plan_digest, 64) {
        return EvidenceStanding::Refused("REFUSED:INVALID_PLAN_DIGEST".to_string());
    }
    if !check.valid_under_independent_dynamics {
        return EvidenceStanding::Refused("REFUSED:PLAN_INVALID_UNDER_INDEPENDENT_DYNAMICS".to_string());
    }
    if check.claimed_cost != check.independently_verified_cost {
        return EvidenceStanding::Refused("REFUSED:PLAN_COST_MISMATCH".to_string());
    }
    EvidenceStanding::Alive
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FondDifferentialCheck {
    pub policy_digest: String,
    pub model_digest: String,
    pub agreed: bool,
    pub counterexample: Option<String>,
}

pub fn admit_fond_differential(check: &FondDifferentialCheck) -> EvidenceStanding {
    if !lowercase_hex(&check.policy_digest, 64) || !lowercase_hex(&check.model_digest, 64) {
        return EvidenceStanding::Refused("REFUSED:INVALID_DIFFERENTIAL_DIGEST".to_string());
    }
    if !check.agreed || check.counterexample.is_some() {
        return EvidenceStanding::Refused("REFUSED:FOND_TLA_DIFFERENTIAL_MISMATCH".to_string());
    }
    EvidenceStanding::Alive
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryDecision {
    pub subject: String,
    pub selected_provider: String,
    pub excluded_failed_providers: Vec<String>,
}

pub fn route_edge_local_recovery(
    subject: &str,
    ordered_providers: &[String],
    failed_providers: &BTreeSet<String>,
) -> Result<RecoveryDecision, String> {
    if subject.trim().is_empty() {
        return Err("REFUSED:EMPTY_EXACT_SUBJECT".to_string());
    }
    let selected = ordered_providers
        .iter()
        .find(|provider| !failed_providers.contains(provider.as_str()))
        .cloned()
        .ok_or_else(|| "BLOCKED:NO_LAWFUL_RECOVERY_EDGE".to_string())?;

    Ok(RecoveryDecision {
        subject: subject.to_string(),
        selected_provider: selected,
        excluded_failed_providers: failed_providers.iter().cloned().collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeQualification {
    pub standing: EvidenceStanding,
    pub missing_subjects: Vec<String>,
}

pub fn qualify_v26_9_28_upgrade(manifest: &EcosystemManifest) -> UpgradeQualification {
    const REQUIRED: &[&str] = &[
        "ggen-marketplace-v26.9.29",
        "graphlaw-v26.9.28",
        "ggen-v26.9.28",
        "ash-r2rml-v26.9.28",
        "ash-r2rml-pr48",
        "affidavit-pr89",
        "wasm4pm-pr646",
        "dspy-wasm-pr8",
        "autofde-lab-pr207",
        "ash-pplan-pr15",
        "ash-a2a-pr60",
        "gymact-pr154",
        "beam4pm-pr104",
        "ferroplan-pr49",
        "xaas-pr98",
        "xaas-pr95",
    ];
    let present: BTreeSet<&str> = manifest.subjects.iter().map(|s| s.id.as_str()).collect();
    let missing: Vec<String> = REQUIRED
        .iter()
        .filter(|id| !present.contains(*id))
        .map(|id| (*id).to_string())
        .collect();

    let reviewed_total = manifest.review_window.merged_pr_count
        + manifest.review_window.open_pr_count
        + manifest.review_window.closed_unmerged_pr_count;
    let standing = if manifest.release_epoch != ECOSYSTEM_EPOCH {
        EvidenceStanding::Refused("REFUSED:ECOSYSTEM_EPOCH_DRIFT".to_string())
    } else if manifest.review_window.timezone != "America/Los_Angeles"
        || manifest.review_window.pr_count != 66
        || reviewed_total != manifest.review_window.pr_count
    {
        EvidenceStanding::Refused("REFUSED:REVIEW_WINDOW_DRIFT".to_string())
    } else if !lowercase_hex(&manifest.castle_base_sha, 40) {
        EvidenceStanding::Refused("REFUSED:INVALID_CASTLE_BASE".to_string())
    } else if !lowercase_hex(&manifest.marketplace_pack.commit_sha, 40) {
        EvidenceStanding::Refused("REFUSED:INVALID_MARKETPLACE_SUBJECT".to_string())
    } else if missing.is_empty() {
        EvidenceStanding::Alive
    } else {
        EvidenceStanding::Refused("REFUSED:MISSING_REQUIRED_ECOSYSTEM_SUBJECT".to_string())
    };

    UpgradeQualification {
        standing,
        missing_subjects: missing,
    }
}
