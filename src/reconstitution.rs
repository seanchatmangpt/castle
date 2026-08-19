//! Fail-closed intake of the ggen-legacy EMPIRE/OSTAR reconstitution contract.
//!
//! The ggen-legacy observation begins at `NO_AUTHORITY`; CASTLE refuses that
//! artifact. A separately authored, SHA-256-bound bounded contract may become an
//! opaque `EmpireReconstitutionAdmission`, but that admission is still inert:
//! it can only be projected into the O* input of CASTLE's signed CONSTRUCT rail.
//! It grants no `DO` authority.

use std::collections::BTreeSet;
use std::fmt::{self, Write as _};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const ADMISSION_SCHEMA: &str = "ggen.legacy.authority-vacuum.admission.v1";
const RECEIPT_SCHEMA: &str = "ggen.legacy.authority-vacuum.receipt.v1";
const STUDY_ID: &str = "OSTAR-EMPIRE-001";
const REQUIRED_CAPABILITIES: [&str; 6] = [
    "ontostar-admission-manufacture",
    "ostar-cli-load",
    "ostar-codemanufactory-manufacture",
    "ostar-dteam-process-intelligence",
    "ostar-ggen-projection-contract",
    "ostar-governance-pipeline",
];
const OBSERVABLE_SURFACES: [&str; 10] = [
    "diagnostics",
    "event_order",
    "exit_code",
    "filesystem_delta",
    "generated_bytes",
    "receipt_fields",
    "recovery_result",
    "side_effects",
    "stderr",
    "stdout",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstitutionRefusal {
    pub code: &'static str,
    pub detail: String,
}

impl ReconstitutionRefusal {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ReconstitutionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "REFUSED:{}:{}", self.code, self.detail)
    }
}

impl std::error::Error for ReconstitutionRefusal {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum FinalDisposition {
    Preserved,
    Subsumed,
    Replaced,
    Archived,
    Refused,
}

impl FinalDisposition {
    fn parse(value: &str) -> Result<Self, ReconstitutionRefusal> {
        match value {
            "PRESERVED" => Ok(Self::Preserved),
            "SUBSUMED" => Ok(Self::Subsumed),
            "REPLACED" => Ok(Self::Replaced),
            "ARCHIVED" => Ok(Self::Archived),
            "REFUSED" => Ok(Self::Refused),
            _ => Err(ReconstitutionRefusal::new(
                "ADMISSION_DISPOSITION_UNKNOWN",
                format!("unrecognized final disposition {value:?}"),
            )),
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preserved => "PRESERVED",
            Self::Subsumed => "SUBSUMED",
            Self::Replaced => "REPLACED",
            Self::Archived => "ARCHIVED",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstitutedCapability {
    id: String,
    disposition: FinalDisposition,
    evidence_ids: Vec<String>,
    observable_surfaces: Vec<String>,
}

impl ReconstitutedCapability {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn disposition(&self) -> FinalDisposition {
        self.disposition
    }

    #[must_use]
    pub fn evidence_ids(&self) -> &[String] {
        &self.evidence_ids
    }

    #[must_use]
    pub fn observable_surfaces(&self) -> &[String] {
        &self.observable_surfaces
    }
}

mod sealed {
    #[derive(Debug, Clone, Copy)]
    pub struct AdmissionBrand;
}

/// Opaque proof that a bounded EMPIRE/OSTAR contract passed the CASTLE intake
/// fence. This proves eligibility for CONSTRUCT input only, never actuation.
#[derive(Debug, Clone)]
pub struct EmpireReconstitutionAdmission {
    study_id: String,
    admission_digest: String,
    observation_receipt_digest: String,
    authority_id: String,
    authority_digest: String,
    capabilities: Vec<ReconstitutedCapability>,
    _brand: sealed::AdmissionBrand,
}

impl EmpireReconstitutionAdmission {
    #[must_use]
    pub fn study_id(&self) -> &str {
        &self.study_id
    }

    #[must_use]
    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }

    #[must_use]
    pub fn observation_receipt_digest(&self) -> &str {
        &self.observation_receipt_digest
    }

    #[must_use]
    pub fn authority_id(&self) -> &str {
        &self.authority_id
    }

    #[must_use]
    pub fn capabilities(&self) -> &[ReconstitutedCapability] {
        &self.capabilities
    }

    /// The reconstitution gate never grants `DO`, even after successful intake.
    #[must_use]
    pub const fn may_actuate(&self) -> bool {
        false
    }

    /// Project the admitted bounded contract into an inert O* value. CASTLE's
    /// ordinary signed CONSTRUCT and DO admission gates remain downstream.
    #[must_use]
    pub fn to_o_star_value(&self) -> Value {
        let capabilities = self
            .capabilities
            .iter()
            .map(|capability| {
                json!({
                    "id": capability.id,
                    "disposition": capability.disposition.as_str(),
                    "evidence_ids": capability.evidence_ids,
                    "observable_surfaces": capability.observable_surfaces,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "schema": "castle.empire.reconstitution.o-star.v1",
            "study_id": self.study_id,
            "admission_digest": self.admission_digest,
            "observation_receipt_digest": self.observation_receipt_digest,
            "authority": {
                "id": self.authority_id,
                "digest": self.authority_digest,
            },
            "claim_ceiling": "SCHEMA_VALIDATED",
            "capabilities": capabilities,
            "actuation_authority": false,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdmissionEnvelope {
    core: AdmissionCore,
    receipt: AdmissionReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdmissionCore {
    schema: String,
    study_id: String,
    authority_state: String,
    claim_ceiling: String,
    semantic_scope: SemanticScope,
    authority: ExplicitAuthority,
    observation_receipt_digest: String,
    capabilities: Vec<CapabilityRecord>,
    disposition_coverage: Vec<String>,
    standing: String,
    actuation_authority: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SemanticScope {
    mode: String,
    universal_equivalence_claimed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rice_boundary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExplicitAuthority {
    id: String,
    digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityRecord {
    id: String,
    disposition: String,
    evidence_ids: Vec<String>,
    observable_surfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdmissionReceipt {
    schema: String,
    algorithm: String,
    artifact_digest: String,
    epistemic_class: String,
    authority: bool,
    parent_digests: Vec<String>,
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(string) => serde_json::to_string(string).expect("JSON string serialization"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            let fields = keys
                .into_iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("JSON object key serialization"),
                        canonical_json(&map[key])
                    )
                })
                .collect::<Vec<_>>();
            format!("{{{}}}", fields.join(","))
        }
    }
}

fn sha256_canonical(value: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(value).as_bytes());
    hasher.update(b"\n");
    let bytes = hasher.finalize();
    let mut output = String::with_capacity(64);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unique_nonempty(values: &[String]) -> bool {
    !values.is_empty()
        && values.iter().all(|value| !value.trim().is_empty())
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn refuse<T>(code: &'static str, detail: impl Into<String>) -> Result<T, ReconstitutionRefusal> {
    Err(ReconstitutionRefusal::new(code, detail))
}

fn verify_core_boundary(core: &AdmissionCore) -> Result<(), ReconstitutionRefusal> {
    if core.schema != ADMISSION_SCHEMA || core.study_id != STUDY_ID {
        return refuse(
            "ADMISSION_SCHEMA_INVALID",
            "expected the exact EMPIRE/OSTAR admission subject",
        );
    }
    if core.authority_state != "ADMITTED_CANDIDATE"
        || core.claim_ceiling != "SCHEMA_VALIDATED"
        || core.standing != "PARTIAL_ALIVE"
        || core.actuation_authority
    {
        return refuse(
            "RECONSTITUTION_NOT_CONSTRUCT_ONLY",
            "admission must be a non-actuating PARTIAL_ALIVE candidate",
        );
    }
    if core.semantic_scope.mode != "bounded-observable-surfaces"
        || core.semantic_scope.universal_equivalence_claimed
    {
        return refuse(
            "RICE_SCOPE_UNBOUNDED",
            "only explicitly bounded observable surfaces are admissible",
        );
    }
    if core.authority.id.trim().is_empty() || !is_lower_hex_64(&core.authority.digest) {
        return refuse(
            "ADMISSION_AUTHORITY_INVALID",
            "authority requires a non-empty id and lowercase SHA-256 digest",
        );
    }
    if !is_lower_hex_64(&core.observation_receipt_digest) {
        return refuse(
            "OBSERVATION_RECEIPT_INVALID",
            "observation receipt digest must be 64 lowercase hex characters",
        );
    }
    Ok(())
}

fn verify_receipt(
    core_value: &Value,
    core: &AdmissionCore,
    receipt: &AdmissionReceipt,
) -> Result<(), ReconstitutionRefusal> {
    if receipt.schema != RECEIPT_SCHEMA
        || receipt.algorithm != "SHA-256"
        || receipt.epistemic_class != "CONSTRUCTED"
        || receipt.authority
        || !is_lower_hex_64(&receipt.artifact_digest)
    {
        return refuse(
            "ADMISSION_RECEIPT_INVALID",
            "receipt kind, algorithm, epistemic class, authority, or digest is invalid",
        );
    }
    if receipt.artifact_digest != sha256_canonical(core_value) {
        return refuse(
            "ADMISSION_RECEIPT_INVALID",
            "canonical core SHA-256 does not match the receipt",
        );
    }
    let expected_parents = BTreeSet::from([
        core.authority.digest.clone(),
        core.observation_receipt_digest.clone(),
    ]);
    let observed_parents = receipt
        .parent_digests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if receipt.parent_digests.len() != 2 || observed_parents != expected_parents {
        return refuse(
            "ADMISSION_PARENT_CHAIN_INVALID",
            "receipt must bind exactly the authority and observation digests",
        );
    }
    Ok(())
}

fn close_capabilities(
    core: &AdmissionCore,
) -> Result<Vec<ReconstitutedCapability>, ReconstitutionRefusal> {
    let expected_ids = REQUIRED_CAPABILITIES.into_iter().collect::<BTreeSet<_>>();
    let observed_ids = core
        .capabilities
        .iter()
        .map(|capability| capability.id.as_str())
        .collect::<BTreeSet<_>>();
    if core.capabilities.len() != expected_ids.len() || observed_ids != expected_ids {
        return refuse(
            "ADMISSION_CAPABILITY_CLOSURE_INCOMPLETE",
            "every observed EMPIRE/OSTAR candidate requires exactly one disposition",
        );
    }

    let allowed_surfaces = OBSERVABLE_SURFACES.into_iter().collect::<BTreeSet<_>>();
    let mut capabilities = Vec::with_capacity(core.capabilities.len());
    let mut dispositions = BTreeSet::new();
    for capability in &core.capabilities {
        let disposition = FinalDisposition::parse(&capability.disposition)?;
        if !unique_nonempty(&capability.evidence_ids) {
            return refuse(
                "ADMISSION_EVIDENCE_INVALID",
                format!("capability {} lacks unique evidence ids", capability.id),
            );
        }
        let surfaces = capability
            .observable_surfaces
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if !unique_nonempty(&capability.observable_surfaces)
            || !surfaces.is_subset(&allowed_surfaces)
        {
            return refuse(
                "ADMISSION_SCOPE_INVALID",
                format!(
                    "capability {} has invalid observable surfaces",
                    capability.id
                ),
            );
        }
        dispositions.insert(disposition);
        capabilities.push(ReconstitutedCapability {
            id: capability.id.clone(),
            disposition,
            evidence_ids: capability.evidence_ids.clone(),
            observable_surfaces: capability.observable_surfaces.clone(),
        });
    }
    verify_disposition_coverage(core, &dispositions)?;
    capabilities.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(capabilities)
}

fn verify_disposition_coverage(
    core: &AdmissionCore,
    dispositions: &BTreeSet<FinalDisposition>,
) -> Result<(), ReconstitutionRefusal> {
    if !dispositions.contains(&FinalDisposition::Refused) {
        return refuse(
            "SCOPING_FAILURE_NO_REFUSAL",
            "the case study must be re-scoped when every candidate is admitted",
        );
    }
    let required = BTreeSet::from([
        FinalDisposition::Preserved,
        FinalDisposition::Subsumed,
        FinalDisposition::Replaced,
        FinalDisposition::Archived,
        FinalDisposition::Refused,
    ]);
    let declared = core
        .disposition_coverage
        .iter()
        .map(|value| FinalDisposition::parse(value))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if dispositions != &required
        || declared != required
        || core.disposition_coverage.len() != required.len()
    {
        return refuse(
            "ADMISSION_DISPOSITION_COVERAGE_INCOMPLETE",
            "PRESERVED, SUBSUMED, REPLACED, ARCHIVED, and REFUSED must all be exercised",
        );
    }
    Ok(())
}

/// Admit a ggen-legacy reconstitution contract as inert EMPIRE CONSTRUCT input.
///
/// A `NO_AUTHORITY` observation is deliberately the wrong schema and is refused.
/// The accepted envelope must close all six observed candidates, exercise all
/// five final dispositions (including a real refusal), bind the explicit authority
/// and observation receipt as parents, and reproduce ggen-legacy's canonical
/// SHA-256 receipt exactly.
///
/// # Errors
///
/// Returns a named `ReconstitutionRefusal` when parsing, subject identity,
/// Rice scope, receipt binding, capability closure, evidence, or disposition
/// coverage fails.
pub fn admit_empire_reconstitution_for_construct(
    document: &str,
) -> Result<EmpireReconstitutionAdmission, ReconstitutionRefusal> {
    let raw: Value = serde_json::from_str(document)
        .map_err(|error| ReconstitutionRefusal::new("ADMISSION_JSON_INVALID", error.to_string()))?;
    let envelope: AdmissionEnvelope = serde_json::from_value(raw.clone()).map_err(|error| {
        ReconstitutionRefusal::new("ADMISSION_SCHEMA_INVALID", error.to_string())
    })?;
    let core_value = raw
        .get("core")
        .ok_or_else(|| ReconstitutionRefusal::new("ADMISSION_SCHEMA_INVALID", "missing core"))?;
    let core = &envelope.core;
    let receipt = &envelope.receipt;
    verify_core_boundary(core)?;
    verify_receipt(core_value, core, receipt)?;
    let capabilities = close_capabilities(core)?;
    Ok(EmpireReconstitutionAdmission {
        study_id: core.study_id.clone(),
        admission_digest: receipt.artifact_digest.clone(),
        observation_receipt_digest: core.observation_receipt_digest.clone(),
        authority_id: core.authority.id.clone(),
        authority_digest: core.authority.digest.clone(),
        capabilities,
        _brand: sealed::AdmissionBrand,
    })
}
