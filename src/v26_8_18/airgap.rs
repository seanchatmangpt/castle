use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{GlobalConstitution, ReleaseStanding, RELEASE_VERSION};

fn digest_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|e| format!("REFUSED:SERIALIZATION_FAILED:{e}"))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirgapBundle {
    pub kind: String,
    pub release: String,
    pub bundle_id: String,
    pub constitution_id: String,
    pub ontology_version: String,
    pub provider_semantics: BTreeMap<String, String>,
    pub invariant_set_digest: String,
    pub o_star_snapshot: Value,
    pub construct_graph: Value,
    pub prohibited_goals: Value,
    pub network_dependencies: Vec<String>,
    pub secret_dependencies: Vec<String>,
    pub bundle_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirgapResult {
    pub bundle_id: String,
    pub input_bundle_digest: String,
    pub result_digest: String,
    pub network_used: bool,
    pub secret_material_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirgapAdmission {
    pub standing: ReleaseStanding,
    pub reason: String,
}

pub fn manufacture_airgap_bundle(
    bundle_id: String,
    constitution: &GlobalConstitution,
    o_star_snapshot: Value,
    construct_graph: Value,
    prohibited_goals: Value,
) -> Result<AirgapBundle, String> {
    let mut bundle = AirgapBundle {
        kind: "CASTLE_AIRGAP_CONSTRUCT_V1".to_string(),
        release: RELEASE_VERSION.to_string(),
        bundle_id,
        constitution_id: constitution.constitution_id.clone(),
        ontology_version: constitution.ontology_version.clone(),
        provider_semantics: constitution.provider_semantics.clone(),
        invariant_set_digest: constitution.invariant_set_digest.clone(),
        o_star_snapshot,
        construct_graph,
        prohibited_goals,
        network_dependencies: Vec::new(),
        secret_dependencies: Vec::new(),
        bundle_digest: String::new(),
    };
    if bundle.bundle_id.is_empty() {
        return Err("REFUSED:EMPTY_AIRGAP_BUNDLE_ID".to_string());
    }
    bundle.bundle_digest = digest_serializable(&bundle)?;
    Ok(bundle)
}

#[must_use]
pub fn admit_airgap_result(bundle: &AirgapBundle, result: &AirgapResult) -> AirgapAdmission {
    let expected = {
        let mut copy = bundle.clone();
        copy.bundle_digest.clear();
        digest_serializable(&copy).unwrap_or_else(|_| "0".repeat(64))
    };
    let reason = if bundle.release != RELEASE_VERSION || bundle.kind != "CASTLE_AIRGAP_CONSTRUCT_V1" {
        "REFUSED:INVALID_AIRGAP_BUNDLE"
    } else if !bundle.network_dependencies.is_empty() || !bundle.secret_dependencies.is_empty() {
        "REFUSED:AIRGAP_BUNDLE_HAS_EXTERNAL_DEPENDENCIES"
    } else if expected != bundle.bundle_digest {
        "REFUSED:AIRGAP_BUNDLE_DIGEST_MISMATCH"
    } else if result.bundle_id != bundle.bundle_id || result.input_bundle_digest != bundle.bundle_digest {
        "REFUSED:AIRGAP_RESULT_INPUT_MISMATCH"
    } else if result.network_used {
        "REFUSED:AIRGAP_NETWORK_USED"
    } else if result.secret_material_used {
        "REFUSED:AIRGAP_SECRET_MATERIAL_USED"
    } else if result.result_digest.len() != 64
        || !result.result_digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        "REFUSED:INVALID_AIRGAP_RESULT_DIGEST"
    } else {
        "ALIVE:AIRGAP_RESULT_ADMITTED"
    };
    AirgapAdmission {
        standing: if reason.starts_with("ALIVE:") { ReleaseStanding::Alive } else { ReleaseStanding::Refused },
        reason: reason.to_string(),
    }
}
