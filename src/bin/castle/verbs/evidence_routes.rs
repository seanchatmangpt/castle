//! Read-only evidence noun/verb routes.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("verify", "evidence")]
fn evidence_verify(evidence_path: String) -> Result<serde_json::Value> {
    super::evidence_handlers::evidence_verify_handler(evidence_path)
}

/// Admit one negative finding ("looked for evidence, it was absent") into the
/// durable negation ledger. Either `--subject`+`--predicate` or a typed
/// witness refusal (`--witness-json`+`--refusal-reason`).
#[verb("negation-ingest", "evidence")]
fn evidence_negation_ingest(
    ledger_path: String,
    subject: Option<String>,
    predicate: Option<String>,
    searched_by: String,
    searched_at: Option<String>,
    refusal_reason: Option<String>,
    witness_json: Option<String>,
) -> Result<serde_json::Value> {
    super::evidence_handlers::negative_ingest_handler(
        ledger_path,
        subject,
        predicate,
        searched_by,
        searched_at,
        refusal_reason,
        witness_json,
    )
}

/// Query the negation ledger: latest status for `--subject`+`--predicate`, or
/// the full stored record summary when neither is given.
#[verb("negation-status", "evidence")]
fn evidence_negation_status(
    ledger_path: String,
    subject: Option<String>,
    predicate: Option<String>,
) -> Result<serde_json::Value> {
    super::evidence_handlers::negative_status_handler(ledger_path, subject, predicate)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_EVIDENCE_NOUN: fn() = register_evidence_noun;

fn register_evidence_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "evidence",
        "Verify committed CASTLE evidence without actuation; ingest and query negative-evidence (negation) records.",
    );
}
