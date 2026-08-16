//! `castle` CLI routes — thin `#[verb]` wrappers with no business logic of
//! their own; every route delegates to `super::handlers::*`. Pattern aligned
//! with `~/ggen/examples/clap-noun-verb-cli/src/clap_noun_verb_routes.rs`.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// List the 40 generated Fortune-5 readiness controls.
#[verb("requirements", "fortune5")]
fn fortune5_requirements() -> Result<serde_json::Value> {
    super::handlers::fortune5_requirements_handler()
}

/// Evaluate Fortune-5 readiness for a subject from a receipted evidence JSON file.
#[verb("qualify", "fortune5")]
fn fortune5_qualify(
    subject: String,
    evidence_path: String,
    now_epoch_ms: Option<i64>,
    max_evidence_age_ms: Option<i64>,
) -> Result<serde_json::Value> {
    super::handlers::fortune5_qualify_handler(subject, evidence_path, now_epoch_ms, max_evidence_age_ms)
}

/// Check whether a replay class is admitted against a subject's current signature.
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

/// Select the minimal Pareto-coverage prefix of adversarial impact classes from a JSON file.
#[verb("coverage", "impact")]
fn impact_coverage(classes_path: String, target_coverage_bps: Option<i64>) -> Result<serde_json::Value> {
    super::handlers::impact_coverage_handler(classes_path, target_coverage_bps)
}

/// List the marketplace-generated architecture component inventory.
#[verb("components", "inventory")]
fn inventory_components() -> Result<serde_json::Value> {
    super::handlers::inventory_components_handler()
}

/// List the marketplace-generated default prohibited adversarial goals.
#[verb("goals", "inventory")]
fn inventory_goals() -> Result<serde_json::Value> {
    super::handlers::inventory_goals_handler()
}

// ---------------------------------------------------------------------
// Noun `--help` descriptions. `__NOUN_REGISTRY` entries all run before
// `__VERB_REGISTRY` entries at first access, and `CommandRegistry::
// register_noun` is first-writer-wins, so this registration always wins
// over the `#[verb]` macro's own runtime doc-comment scrape attempt (which
// only resolves when the process's cwd happens to be the source checkout).
// ---------------------------------------------------------------------

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
