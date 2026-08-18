//! `castle` CLI routes — thin `#[verb]` wrappers with no business logic of
//! their own; every route delegates to `super::handlers::*`.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("requirements", "fortune5")]
fn fortune5_requirements() -> Result<serde_json::Value> { super::handlers::fortune5_requirements_handler() }

#[verb("qualify", "fortune5")]
fn fortune5_qualify(subject: String, evidence_path: String, now_epoch_ms: Option<i64>, max_evidence_age_ms: Option<i64>) -> Result<serde_json::Value> {
    super::handlers::fortune5_qualify_handler(subject, evidence_path, now_epoch_ms, max_evidence_age_ms)
}

#[verb("admit", "replay")]
fn replay_admit(replay_class_id: String, structural_signature: String, ontology_version: String, provider_semantics_version: String, invariant_set_digest: String, process_digest: String, invariants_hold: bool) -> Result<serde_json::Value> {
    super::handlers::replay_admit_handler(replay_class_id, structural_signature, ontology_version, provider_semantics_version, invariant_set_digest, process_digest, invariants_hold)
}

#[verb("coverage", "impact")]
fn impact_coverage(classes_path: String, target_coverage_bps: Option<i64>) -> Result<serde_json::Value> { super::handlers::impact_coverage_handler(classes_path, target_coverage_bps) }

#[verb("components", "inventory")]
fn inventory_components() -> Result<serde_json::Value> { super::handlers::inventory_components_handler() }

#[verb("goals", "inventory")]
fn inventory_goals() -> Result<serde_json::Value> { super::handlers::inventory_goals_handler() }

#[verb("qualify", "deployment")]
fn deployment_qualify(manifest_path: String, now_epoch_ms: i64) -> Result<serde_json::Value> { super::handlers::deployment_qualify_handler(manifest_path, now_epoch_ms) }

#[verb("adapters", "deployment")]
fn deployment_adapters() -> Result<serde_json::Value> { super::handlers::deployment_adapters_handler() }

#[verb("info", "release")]
fn release_info() -> Result<serde_json::Value> { super::handlers::release_info_handler() }

#[verb("mcp", "protocol")]
fn protocol_mcp() -> Result<serde_json::Value> { super::handlers::protocol_mcp_handler() }

#[verb("a2a", "protocol")]
fn protocol_a2a() -> Result<serde_json::Value> { super::handlers::protocol_a2a_handler() }

/// Manufacture an inert, deterministic CONSTRUCT checkpoint from a runtime request.
#[verb("manufacture", "construct")]
fn construct_manufacture(request_path: String, signing_key_path: String, key_id: String) -> Result<serde_json::Value> {
    super::handlers::construct_manufacture_handler(request_path, signing_key_path, key_id)
}

/// Recompute, admit, and execute the exact previously manufactured CONSTRUCT.
#[verb("execute", "do")]
fn do_execute(request_path: String, signing_key_path: String, key_id: String, expected_construct_digest: String, now_epoch_ms: i64) -> Result<serde_json::Value> {
    super::handlers::do_execute_handler(request_path, signing_key_path, key_id, expected_construct_digest, now_epoch_ms)
}

#[verb("capabilities", "crypto")]
fn crypto_capabilities() -> Result<serde_json::Value> { super::handlers::crypto_capabilities_handler() }

#[verb("qualify", "chaos")]
fn chaos_qualify(evidence_path: String) -> Result<serde_json::Value> { super::handlers::chaos_qualify_handler(evidence_path) }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_FORTUNE5_NOUN: fn() = register_fortune5_noun;
fn register_fortune5_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("fortune5", "Query and evaluate the Fortune-5 readiness profile."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_REPLAY_NOUN: fn() = register_replay_noun;
fn register_replay_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("replay", "Admit or refuse replay classes against a subject's current signature."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_IMPACT_NOUN: fn() = register_impact_noun;
fn register_impact_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("impact", "Select the minimal Pareto-coverage prefix of adversarial impact classes."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_INVENTORY_NOUN: fn() = register_inventory_noun;
fn register_inventory_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("inventory", "Query the marketplace-generated component and goal inventory."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_DEPLOYMENT_NOUN: fn() = register_deployment_noun;
fn register_deployment_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("deployment", "Qualify the cellular Fortune-5 global deployment and inspect provider adapters."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_RELEASE_NOUN: fn() = register_release_noun;
fn register_release_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("release", "Inspect CASTLE release identity and constitutional invariants."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PROTOCOL_NOUN: fn() = register_protocol_noun;
fn register_protocol_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("protocol", "Inspect MCP and A2A semantic surfaces without granting ambient DO authority."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_CONSTRUCT_NOUN: fn() = register_construct_noun;
fn register_construct_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("construct", "Manufacture inert receipt-bound CONSTRUCT checkpoints."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_DO_NOUN: fn() = register_do_noun;
fn register_do_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("do", "Execute only a recomputed and admitted CONSTRUCT through BRCE."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_CRYPTO_NOUN: fn() = register_crypto_noun;
fn register_crypto_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("crypto", "Inspect cryptographic identity and signature-suite standing."); }

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_CHAOS_NOUN: fn() = register_chaos_noun;
fn register_chaos_noun() { ::clap_noun_verb::cli::registry::CommandRegistry::register_noun("chaos", "Qualify receipted failure-domain evidence."); }
