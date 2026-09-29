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

/// Admit (screened when sanctions/counterparties given, journaled when --journal-dir given), then
/// place a hold and submit to the SimRail. Refuses without --reference-ledger and --reference-rail.
#[verb("rail-submit", "payments")]
fn payments_rail_submit(
    effect_path: String,
    certificate_path: String,
    registry_path: String,
    policy_path: String,
    state_dir: String,
    ledger_dir: String,
    opening_path: String,
    reference_ledger: bool,
    reference_rail: bool,
    rail_dir: String,
    rail_mode: String,
    audience: String,
    policy_epoch: i64,
    revocation_epoch: i64,
    generation: i64,
    now_ms: i64,
    receipt_key_id: String,
    receipt_seed_hex: String,
    allowed_authority: String,
    debtor_agent_bic: String,
    creditor_agent_bic: String,
    created_at_iso: String,
    sanctions_path: Option<String>,
    counterparties_path: Option<String>,
    journal_dir: Option<String>,
    debtor_name: Option<String>,
    creditor_name: Option<String>,
    effect_expires_at_ms: Option<String>,
) -> Result<serde_json::Value> {
    super::payments_handlers::rail_submit_handler(super::payments_handlers::RailSubmitArgs {
        base: super::payments_handlers::ExecuteArgs {
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
        },
        reference_rail,
        rail_dir,
        rail_mode,
        sanctions_path,
        counterparties_path,
        journal_dir,
        debtor_agent_bic,
        creditor_agent_bic,
        created_at_iso,
        debtor_name,
        creditor_name,
        effect_expires_at_ms,
    })
}

/// Poll the rail once and apply finality: prints Pending, Applied, StillUnknown or ProvenAbsent.
#[verb("finalize", "payments")]
fn payments_finalize(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    rail_mode: Option<String>,
    reference_ledger: bool,
    reference_rail: bool,
) -> Result<serde_json::Value> {
    super::payments_handlers::finalize_handler(effect_digest, state_dir, ledger_dir, rail_dir, rail_mode, reference_ledger, reference_rail)
}

/// Re-derive the stateless admission decision from the persisted admission journal.
#[verb("replay", "payments")]
fn payments_replay(journal_dir: String, effect_digest: String) -> Result<serde_json::Value> {
    super::payments_handlers::replay_handler(journal_dir, effect_digest)
}

/// Seal and print the event receipt built from persisted claim, ledger, rail and journal state.
#[verb("receipt", "payments")]
fn payments_receipt(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    journal_dir: String,
    reference_ledger: bool,
    reference_rail: bool,
) -> Result<serde_json::Value> {
    super::payments_handlers::receipt_handler(effect_digest, state_dir, ledger_dir, rail_dir, journal_dir, reference_ledger, reference_rail)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PAYMENTS_NOUN: fn() = register_payments_noun;

fn register_payments_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "payments",
        "SA2A payments: prepare, execute, rail-submit, finalize, reconcile, recover, explain, replay, receipt.",
    );
}
