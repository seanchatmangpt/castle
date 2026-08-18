//! `castle` CLI routes — thin `#[verb]` wrappers with no business logic of
//! their own; every route delegates to `super::handlers::*`. Pattern aligned
//! with `~/ggen/examples/clap-noun-verb-cli/src/clap_noun_verb_routes.rs`.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("requirements", "fortune5")]
fn fortune5_requirements() -> Result<serde_json::Value> {
    super::handlers::fortune5_requirements_handler()
}

#[verb("qualify", "fortune5")]
fn fortune5_qualify(
    subject: String,
    evidence_path: String,
    now_epoch_ms: Option<i64>,
    max_evidence_age_ms: Option<i64>,
) -> Result<serde_json::Value> {
    super::handlers::fortune5_qualify_handler(subject, evidence_path, now_epoch_ms, max_evidence_age_ms)
}

#[verb("admit", "replay")]
fn replay_admit(
    replay_class_id: String,
    structural_signature: String,
    ontology_version: String,
    provider_semantics_version: String,
    invariant_set_digest: String,
    process_digest: String,
    invariants_hold: bool,
) -> Result<serde_json::Value> {
    super::handlers::replay_admit_handler(
        replay_class_id,
        structural_signature,
        ontology_version,
        provider_semantics_version,
        invariant_set_digest,
        process_digest,
        invariants_hold,
    )
}

#[verb("coverage", "impact")]
fn impact_coverage(classes_path: String, target_coverage_bps: Option<i64>) -> Result<serde_json::Value> {
    super::handlers::impact_coverage_handler(classes_path, target_coverage_bps)
}

#[verb("components", "inventory")]
fn inventory_components() -> Result<serde_json::Value> {
    super::handlers::inventory_components_handler()
}

#[verb("goals", "inventory")]
fn inventory_goals() -> Result<serde_json::Value> {
    super::handlers::inventory_goals_handler()
}

/// Qualify a v26.8.18 Fortune-5 global deployment manifest.
#[verb("qualify", "deployment")]
fn deployment_qualify(manifest_path: String, now_epoch_ms: i64) -> Result<serde_json::Value> {
    super::handlers::deployment_qualify_handler(manifest_path, now_epoch_ms)
}

/// List the official v26.8.18 provider adapter families and authority models.
#[verb("adapters", "deployment")]
fn deployment_adapters() -> Result<serde_json::Value> {
    super::handlers::deployment_adapters_handler()
}

/// Return the v26.8.18 release identity and constitutional invariants.
#[verb("info", "release")]
fn release_info() -> Result<serde_json::Value> {
    super::handlers::release_info_handler()
}

/// Return the MCP tool contract. DO remains receipt-bound.
#[verb("mcp", "protocol")]
fn protocol_mcp() -> Result<serde_json::Value> {
    super::handlers::protocol_mcp_handler()
}

/// Return the A2A agent card. Default authority remains CONSTRUCT_ONLY.
#[verb("a2a", "protocol")]
fn protocol_a2a() -> Result<serde_json::Value> {
    super::handlers::protocol_a2a_handler()
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_FORTUNE5_NOUN: fn() = register_fortune5_noun;
fn register_fortune5_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("fortune5", "Query and evaluate the Fortune-5 readiness profile.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_REPLAY_NOUN: fn() = register_replay_noun;
fn register_replay_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("replay", "Admit or refuse replay classes against a subject's current signature.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_IMPACT_NOUN: fn() = register_impact_noun;
fn register_impact_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("impact", "Select the minimal Pareto-coverage prefix of adversarial impact classes.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_INVENTORY_NOUN: fn() = register_inventory_noun;
fn register_inventory_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("inventory", "Query the marketplace-generated component and goal inventory.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_DEPLOYMENT_NOUN: fn() = register_deployment_noun;
fn register_deployment_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("deployment", "Qualify the cellular Fortune-5 global deployment and inspect provider adapters.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_RELEASE_NOUN: fn() = register_release_noun;
fn register_release_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("release", "Inspect CASTLE release identity and constitutional invariants.");
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PROTOCOL_NOUN: fn() = register_protocol_noun;
fn register_protocol_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("protocol", "Inspect MCP and A2A semantic surfaces without granting ambient DO authority.");
}
