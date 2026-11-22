//! `deletion` noun/verb routes. Thin wrappers only; logic lives in `deletion_handlers`.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Dump statutory clocks and citations (read-only, no I/O).
#[verb("info", "deletion")]
fn deletion_info() -> Result<serde_json::Value> {
    super::deletion_handlers::deletion_info_handler()
}

/// Open a sealed deletion case from an intake JSON path.
#[verb("open", "deletion")]
fn deletion_open(intake_path: String) -> Result<serde_json::Value> {
    super::deletion_handlers::deletion_open_handler(intake_path)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_DELETION_NOUN: fn() = register_deletion_noun;

fn register_deletion_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "deletion",
        "Track first-party 45-day CPRA/SB 923 data-deletion cases and receipts.",
    );
}
