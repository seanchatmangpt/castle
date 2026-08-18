use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{InterfaceOrigin, IntentMode, RELEASE_KIND, RELEASE_VERSION};

fn digest_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_vec(value)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|e| format!("REFUSED:SERIALIZATION_FAILED:{e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudProvider {
    Aws,
    Azure,
    Gcp,
    PrivateCloud,
    Edge,
}

impl CloudProvider {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::Azure => "azure",
            Self::Gcp => "gcp",
            Self::PrivateCloud => "private-cloud",
            Self::Edge => "edge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseStanding {
    Unknown,
    PartialAlive,
    Alive,
    Blocked,
    BuildBroken,
    Unsupported,
    Refused,
}

impl ReleaseStanding {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::PartialAlive => "PARTIAL_ALIVE",
            Self::Alive => "ALIVE",
            Self::Blocked => "BLOCKED",
            Self::BuildBroken => "BUILD_BROKEN",
            Self::Unsupported => "UNSUPPORTED",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConstitution {
    pub constitution_id: String,
    pub ontology_version: String,
    pub invariant_set_digest: String,
    pub trust_root_ids: BTreeSet<String>,
    pub provider_semantics: BTreeMap<String, String>,
    pub issued_at_epoch_ms: i64,
    pub expires_at_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterBinding {
    pub adapter_id: String,
    pub kind: String,
    pub provider_semantics_key: String,
    pub provider_semantics_version: String,
    pub allowed_transition_ids: BTreeSet<String>,
    pub workload_identity: String,
    #[serde(default)]
    pub ambient_credentials: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CastleCellManifest {
    pub cell_id: String,
    pub region: String,
    pub provider: CloudProvider,
    pub authority_domain: String,
    pub residency: String,
    pub subject_prefixes: Vec<String>,
    pub local_receipt_store: String,
    pub local_ocel_store: String,
    pub max_parallel_do: u32,
    pub adapters: Vec<AdapterBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSurface {
    pub surface: InterfaceOrigin,
    pub endpoint: String,
    pub modes: BTreeSet<IntentMode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentManifest {
    pub kind: String,
    pub release: String,
    pub constitution: GlobalConstitution,
    pub cells: Vec<CastleCellManifest>,
    pub protocol_surfaces: Vec<ProtocolSurface>,
    pub required_providers: BTreeSet<CloudProvider>,
    pub required_adapter_kinds: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentQualification {
    pub standing: ReleaseStanding,
    pub manifest_digest: String,
    pub findings: Vec<String>,
    pub cells: usize,
    pub providers: BTreeSet<CloudProvider>,
    pub adapter_kinds: BTreeSet<String>,
}

impl DeploymentQualification {
    #[must_use]
    pub fn alive(&self) -> bool {
        self.standing == ReleaseStanding::Alive
    }
}

/// Deterministically qualify the global topology. The gate refuses ambient
/// credentials, semantic drift, missing region-local evidence durability,
/// duplicated authority cells, and incomplete provider/protocol coverage.
#[must_use]
pub fn qualify_deployment(manifest: &DeploymentManifest, now_epoch_ms: i64) -> DeploymentQualification {
    let manifest_digest = digest_serializable(manifest).unwrap_or_else(|_| "0".repeat(64));
    let mut findings = Vec::new();

    if manifest.kind != RELEASE_KIND {
        findings.push("REFUSED:INVALID_RELEASE_KIND".to_string());
    }
    if manifest.release != RELEASE_VERSION {
        findings.push("REFUSED:RELEASE_VERSION_MISMATCH".to_string());
    }
    if manifest.constitution.constitution_id.is_empty() || manifest.constitution.ontology_version.is_empty() {
        findings.push("REFUSED:INCOMPLETE_CONSTITUTION".to_string());
    }
    if manifest.constitution.invariant_set_digest.len() != 64
        || !manifest.constitution.invariant_set_digest.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        findings.push("REFUSED:INVALID_INVARIANT_SET_DIGEST".to_string());
    }
    if manifest.constitution.trust_root_ids.is_empty() {
        findings.push("REFUSED:NO_TRUST_ROOTS".to_string());
    }
    if now_epoch_ms < manifest.constitution.issued_at_epoch_ms {
        findings.push("REFUSED:CONSTITUTION_FROM_FUTURE".to_string());
    }
    if now_epoch_ms > manifest.constitution.expires_at_epoch_ms {
        findings.push("REFUSED:CONSTITUTION_EXPIRED".to_string());
    }

    let mut cell_ids = BTreeSet::new();
    let mut providers = BTreeSet::new();
    let mut adapter_kinds = BTreeSet::new();
    if manifest.cells.is_empty() {
        findings.push("REFUSED:NO_CASTLE_CELLS".to_string());
    }

    for cell in &manifest.cells {
        if !cell_ids.insert(cell.cell_id.clone()) {
            findings.push(format!("REFUSED:DUPLICATE_CELL:{}", cell.cell_id));
        }
        providers.insert(cell.provider);
        if cell.region.is_empty() || cell.authority_domain.is_empty() || cell.residency.is_empty() || cell.subject_prefixes.is_empty() {
            findings.push(format!("REFUSED:INCOMPLETE_CELL:{}", cell.cell_id));
        }
        if cell.local_receipt_store.is_empty() {
            findings.push(format!("REFUSED:NO_LOCAL_RECEIPT_DURABILITY:{}", cell.cell_id));
        }
        if cell.local_ocel_store.is_empty() {
            findings.push(format!("REFUSED:NO_LOCAL_OCEL_DURABILITY:{}", cell.cell_id));
        }
        if cell.max_parallel_do == 0 {
            findings.push(format!("REFUSED:ZERO_DO_CAPACITY:{}", cell.cell_id));
        }
        if cell.adapters.is_empty() {
            findings.push(format!("REFUSED:NO_ADAPTERS:{}", cell.cell_id));
        }

        let mut adapter_ids = BTreeSet::new();
        for adapter in &cell.adapters {
            if !adapter_ids.insert(adapter.adapter_id.clone()) {
                findings.push(format!("REFUSED:DUPLICATE_ADAPTER:{}:{}", cell.cell_id, adapter.adapter_id));
            }
            adapter_kinds.insert(adapter.kind.clone());
            if adapter.ambient_credentials {
                findings.push(format!("REFUSED:AMBIENT_CREDENTIALS:{}:{}", cell.cell_id, adapter.adapter_id));
            }
            if adapter.workload_identity.is_empty() {
                findings.push(format!("REFUSED:MISSING_WORKLOAD_IDENTITY:{}:{}", cell.cell_id, adapter.adapter_id));
            }
            if adapter.allowed_transition_ids.is_empty() {
                findings.push(format!("REFUSED:UNBOUNDED_ADAPTER:{}:{}", cell.cell_id, adapter.adapter_id));
            }
            match manifest.constitution.provider_semantics.get(&adapter.provider_semantics_key) {
                Some(version) if version == &adapter.provider_semantics_version => {}
                Some(_) => findings.push(format!("REFUSED:PROVIDER_SEMANTICS_DRIFT:{}:{}", cell.cell_id, adapter.adapter_id)),
                None => findings.push(format!("REFUSED:UNKNOWN_PROVIDER_SEMANTICS:{}:{}", cell.cell_id, adapter.adapter_id)),
            }
        }
    }

    for provider in &manifest.required_providers {
        if !providers.contains(provider) {
            findings.push(format!("REFUSED:MISSING_PROVIDER:{}", provider.as_str()));
        }
    }
    for required in &manifest.required_adapter_kinds {
        if !adapter_kinds.contains(required) {
            findings.push(format!("REFUSED:MISSING_ADAPTER_KIND:{required}"));
        }
    }

    for required in [InterfaceOrigin::Cli, InterfaceOrigin::Api, InterfaceOrigin::Mcp, InterfaceOrigin::A2a] {
        match manifest.protocol_surfaces.iter().find(|surface| surface.surface == required) {
            None => findings.push(format!("REFUSED:MISSING_PROTOCOL_SURFACE:{}", required.as_str())),
            Some(surface) if !surface.modes.contains(&IntentMode::Select) || !surface.modes.contains(&IntentMode::Construct) => {
                findings.push(format!("REFUSED:INCOMPLETE_PROTOCOL_SURFACE:{}", required.as_str()));
            }
            Some(_) => {}
        }
    }

    DeploymentQualification {
        standing: if findings.is_empty() { ReleaseStanding::Alive } else { ReleaseStanding::Refused },
        manifest_digest,
        findings,
        cells: manifest.cells.len(),
        providers,
        adapter_kinds,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellStandingRow {
    pub cell_id: String,
    pub observation: ReleaseStanding,
    pub construct: ReleaseStanding,
    pub do_standing: ReleaseStanding,
    pub replay: ReleaseStanding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalStanding {
    pub standing: ReleaseStanding,
    pub cells: Vec<CellStandingRow>,
}

#[must_use]
pub fn aggregate_global_standing(mut cells: Vec<CellStandingRow>) -> GlobalStanding {
    cells.sort_by(|a, b| a.cell_id.cmp(&b.cell_id));
    let standing = if cells.is_empty() {
        ReleaseStanding::Unknown
    } else if cells.iter().all(|row| {
        row.observation == ReleaseStanding::Alive
            && row.construct == ReleaseStanding::Alive
            && row.do_standing == ReleaseStanding::Alive
            && row.replay == ReleaseStanding::Alive
    }) {
        ReleaseStanding::Alive
    } else if cells.iter().any(|row| {
        row.observation == ReleaseStanding::Blocked
            || row.construct == ReleaseStanding::Blocked
            || row.do_standing == ReleaseStanding::Blocked
            || row.replay == ReleaseStanding::Blocked
    }) {
        ReleaseStanding::Blocked
    } else {
        ReleaseStanding::PartialAlive
    };
    GlobalStanding { standing, cells }
}
