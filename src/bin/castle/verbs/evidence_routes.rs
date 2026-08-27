//! Read-only evidence noun/verb routes.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("verify", "evidence")]
fn evidence_verify(evidence_path: String) -> Result<serde_json::Value> {
    super::evidence_handlers::evidence_verify_handler(evidence_path)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_EVIDENCE_NOUN: fn() = register_evidence_noun;

fn register_evidence_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "evidence",
        "Verify already committed durable CASTLE evidence without actuation.",
    );
}
