//! `payments` noun/verb routes. Thin wrappers only; logic lives in `payments_handlers`.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Build the portable PreparedEffect and print it with its digest. Grants nothing.
#[verb("prepare", "payments")]
fn payments_prepare(
    principal: String,
    payer: String,
    payee: String,
    amount_minor: String,
    currency: String,
    obligation_id: String,
    purpose: String,
    reverses: Option<String>,
) -> Result<serde_json::Value> {
    super::payments_handlers::prepare_handler(principal, payer, payee, amount_minor, currency, obligation_id, purpose, reverses)
}

/// Admit then execute a payment through the durable BRCE DO path.
#[verb("execute", "payments")]
fn payments_execute(
    effect_path: String,
    certificate_path: String,
    registry_path: String,
    policy_path: String,
    state_dir: String,
    ledger_dir: String,
    opening_path: String,
    reference_ledger: bool,
    audience: String,
    policy_epoch: i64,
    revocation_epoch: i64,
    generation: i64,
    now_ms: i64,
    receipt_key_id: String,
    receipt_seed_hex: String,
    allowed_authority: String,
) -> Result<serde_json::Value> {
    super::payments_handlers::execute_handler(super::payments_handlers::ExecuteArgs {
        effect_path,
        certificate_path,
        registry_path,
        policy_path,
        state_dir,
        ledger_dir,
        opening_path,
        reference_ledger,
        audience,
        policy_epoch,
        revocation_epoch,
        generation,
        now_ms,
        receipt_key_id,
        receipt_seed_hex,
        allowed_authority,
    })
}

/// Resolve an UnknownOutcome claim strictly from the ledger's own record.
#[verb("reconcile", "payments")]
fn payments_reconcile(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<serde_json::Value> {
    super::payments_handlers::reconcile_handler(effect_digest, state_dir, ledger_dir, reference_ledger)
}

/// Report BRCE journal prepare/outcome pairing after a crash.
#[verb("recover", "payments")]
fn payments_recover(state_dir: String) -> Result<serde_json::Value> {
    super::payments_handlers::recover_handler(state_dir)
}

/// Board answer for one payment, built from persisted claim and ledger state only.
#[verb("explain", "payments")]
fn payments_explain(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<serde_json::Value> {
    super::payments_handlers::explain_handler(effect_digest, state_dir, ledger_dir, reference_ledger)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PAYMENTS_NOUN: fn() = register_payments_noun;

fn register_payments_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "payments",
        "SA2A payments: prepare, admit and execute, reconcile, recover, explain.",
    );
}
