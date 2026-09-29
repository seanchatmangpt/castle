//! Payments handlers: all logic for the `payments` noun. Money is handled only as
//! canonical integer minor-unit strings; nothing here parses amounts numerically.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use castle::payments::{
    admit_payment, execute_payment, reconcile, recover_journal, AdmissionContext, ClaimStore, Currency,
    DurableNonceFence, ExecutionContext, FileJournalLedger, LedgerPort, Money, PaymentEffect, PaymentStanding,
    ReconcileResolution, SpendPolicy,
};
use castle::sa2a_security::epoch::SecurityEpochs;
use castle::sa2a_security::{ActuationCertificate, KeyRecord, KeyRegistry, PreparedEffect};
use castle::v26_8_18::{decode_seed_hex, Ed25519RuntimeSigner, NativeBlake3};
use clap_noun_verb::NounVerbError;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

const REFERENCE_LEDGER_NOT_CONFIRMED: &str = "REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED";
const LEDGER_NOT_FOUND: &str = "REFUSED:PAYMENT_LEDGER_NOT_FOUND";
const CLAIM_NOT_FOUND: &str = "REFUSED:PAYMENT_CLAIM_NOT_FOUND";
const LEDGER_UNAVAILABLE: &str = "REFUSED:PAYMENT_LEDGER_UNAVAILABLE";

fn exec_err(message: impl Into<String>) -> NounVerbError {
    NounVerbError::ExecutionError { message: message.into() }
}

fn read_json<T: DeserializeOwned>(path: &str, what: &str) -> Result<T> {
    let bytes = std::fs::read(path).map_err(|e| exec_err(format!("REFUSED:PAYMENT_INPUT_UNREADABLE:{what}:{e}")))?;
    serde_json::from_slice(&bytes).map_err(|e| exec_err(format!("REFUSED:PAYMENT_INPUT_INVALID:{what}:{e}")))
}

fn parse_currency(code: &str) -> Result<Currency> {
    serde_json::from_value(Value::String(code.to_string())).map_err(|_| exec_err("REFUSED:PAYMENT_CURRENCY_UNSUPPORTED"))
}

fn require_reference_ledger(confirmed: bool) -> Result<()> {
    if confirmed {
        Ok(())
    } else {
        Err(exec_err(REFERENCE_LEDGER_NOT_CONFIRMED))
    }
}

/// Open an already-initialised reference ledger. Never creates one: an empty
/// ledger fabricated here would let `reconcile` "prove absence" falsely.
fn open_existing_ledger(ledger_dir: &str) -> Result<FileJournalLedger> {
    if !Path::new(ledger_dir).join("opening.json").is_file() {
        return Err(exec_err(LEDGER_NOT_FOUND));
    }
    FileJournalLedger::open(ledger_dir, &[]).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))
}

fn claims_dir(state_dir: &str) -> PathBuf {
    Path::new(state_dir).join("claims")
}

pub fn prepare_handler(
    principal: String,
    payer: String,
    payee: String,
    amount_minor: String,
    currency: String,
    obligation_id: String,
    purpose: String,
    reverses: Option<String>,
) -> Result<Value> {
    let currency = parse_currency(&currency)?;
    let prepared = PaymentEffect::prepare(
        &principal,
        &payer,
        &payee,
        &amount_minor,
        currency,
        &obligation_id,
        &purpose,
        reverses.as_deref(),
    )
    .map_err(exec_err)?;
    // Strict parse validates shape and amount; the result is discarded (inert).
    let effect = PaymentEffect::from_prepared(prepared.clone()).map_err(exec_err)?;
    Ok(json!({
        "effect": serde_json::to_value(&prepared).map_err(|e| exec_err(e.to_string()))?,
        "effect_digest": effect.digest(),
        "grants_authority": false,
    }))
}

#[derive(Debug, Deserialize)]
struct OpeningRow {
    account: String,
    currency: Currency,
    amount_minor: u64,
}

pub struct ExecuteArgs {
    pub effect_path: String,
    pub certificate_path: String,
    pub registry_path: String,
    pub policy_path: String,
    pub state_dir: String,
    pub ledger_dir: String,
    pub opening_path: String,
    pub reference_ledger: bool,
    pub audience: String,
    pub policy_epoch: i64,
    pub revocation_epoch: i64,
    pub generation: i64,
    pub now_ms: i64,
    pub receipt_key_id: String,
    pub receipt_seed_hex: String,
    pub allowed_authority: String,
}

fn to_u64(value: i64, what: &str) -> Result<u64> {
    u64::try_from(value).map_err(|_| exec_err(format!("REFUSED:PAYMENT_INPUT_INVALID:{what}_NEGATIVE")))
}

pub fn execute_handler(a: ExecuteArgs) -> Result<Value> {
    require_reference_ledger(a.reference_ledger)?;
    let prepared: PreparedEffect = read_json(&a.effect_path, "effect")?;
    let certificate: ActuationCertificate = read_json(&a.certificate_path, "certificate")?;
    let records: Vec<KeyRecord> = read_json(&a.registry_path, "registry")?;
    let policy: SpendPolicy = read_json(&a.policy_path, "policy")?;
    let opening_rows: Vec<OpeningRow> = read_json(&a.opening_path, "opening")?;
    let opening: Vec<(&str, Currency, u64)> =
        opening_rows.iter().map(|r| (r.account.as_str(), r.currency, r.amount_minor)).collect();

    let registry = KeyRegistry::from_records(records);
    let epochs = SecurityEpochs {
        policy: to_u64(a.policy_epoch, "policy_epoch")?,
        revocation: to_u64(a.revocation_epoch, "revocation_epoch")?,
        generation: to_u64(a.generation, "generation")?,
    };
    let now_ms = to_u64(a.now_ms, "now_ms")?;

    let state = Path::new(&a.state_dir);
    let claims = ClaimStore::open(claims_dir(&a.state_dir)).map_err(exec_err)?;
    let nonces = DurableNonceFence::open(state.join("nonces")).map_err(exec_err)?;
    let ledger = FileJournalLedger::open(&a.ledger_dir, &opening).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))?;

    let seed = decode_seed_hex(&a.receipt_seed_hex).map_err(exec_err)?;
    let signer = Ed25519RuntimeSigner::from_seed(a.receipt_key_id, seed).map_err(exec_err)?;
    let verifier = signer.verifier();
    let blake3 = NativeBlake3;

    let admission = admit_payment(
        prepared,
        &certificate,
        &AdmissionContext {
            registry: &registry,
            epochs,
            audience: &a.audience,
            now_ms,
            policy: &policy,
            claims: &claims,
            nonces: &nonces,
        },
    )
    .map_err(exec_err)?;

    let ctx = ExecutionContext {
        blake3: &blake3,
        signer: &signer,
        verifier: &verifier,
        allowed_authorities: BTreeSet::from([a.allowed_authority]),
        journal_root: state.join("brce"),
        now_epoch_ms: a.now_ms,
        ledger: &ledger,
        claims: &claims,
        policy: &policy,
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| exec_err(format!("BLOCKED:PAYMENT_RUNTIME_UNAVAILABLE:{e}")))?;
    let exec = runtime.block_on(execute_payment(admission, &ctx)).map_err(exec_err)?;

    let standing = match exec.standing {
        PaymentStanding::Settled => "Settled",
        PaymentStanding::Refused => "Refused",
        PaymentStanding::UnknownOutcome => "UnknownOutcome",
    };
    if exec.standing != PaymentStanding::Settled {
        return Err(exec_err(format!("{} standing={standing} effect_digest={}", exec.detail, exec.effect_digest)));
    }
    Ok(json!({
        "standing": standing,
        "effect_digest": exec.effect_digest,
        "construct_digest": exec.construct_digest,
        "ocel_receipt_digest": exec.ocel_receipt_digest,
        "brce_prepare_receipt_digests": exec.brce_prepare_receipt_digests,
        "brce_outcome_receipt_digests": exec.brce_outcome_receipt_digests,
        "ledger_entry": exec.ledger_entry,
        "detail": exec.detail,
    }))
}

pub fn reconcile_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> {
    require_reference_ledger(reference_ledger)?;
    let ledger = open_existing_ledger(&ledger_dir)?;
    let claims = ClaimStore::open(claims_dir(&state_dir)).map_err(exec_err)?;
    let resolution = reconcile(&effect_digest, &claims, &ledger).map_err(exec_err)?;
    Ok(match resolution {
        ReconcileResolution::Settled(entry) => json!({ "effect_digest": effect_digest, "resolution": "settled", "ledger_entry": entry }),
        ReconcileResolution::ProvenAbsent => json!({ "effect_digest": effect_digest, "resolution": "proven_absent" }),
        ReconcileResolution::StillUnknown => json!({ "effect_digest": effect_digest, "resolution": "still_unknown" }),
    })
}

pub fn recover_handler(state_dir: String) -> Result<Value> {
    let recovery = recover_journal(&Path::new(&state_dir).join("brce")).map_err(exec_err)?;
    Ok(json!({
        "complete": recovery.complete,
        "complete_count": recovery.complete.len(),
        "prepared_without_outcome": recovery.prepared_without_outcome,
        "prepared_without_outcome_count": recovery.prepared_without_outcome.len(),
    }))
}

/// Board answer from persisted claim and ledger entry only; no model is consulted.
pub fn explain_handler(effect_digest: String, state_dir: String, ledger_dir: String, reference_ledger: bool) -> Result<Value> {
    require_reference_ledger(reference_ledger)?;
    let ledger = open_existing_ledger(&ledger_dir)?;
    let claims = ClaimStore::open(claims_dir(&state_dir)).map_err(exec_err)?;
    let claim = claims.get(&effect_digest).map_err(exec_err)?.ok_or_else(|| exec_err(CLAIM_NOT_FOUND))?;
    let entry = ledger.lookup(&effect_digest).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))?;
    let amount_decimal = Money { minor: claim.amount_minor, currency: claim.currency }.to_decimal_string();
    Ok(json!({
        "effect_digest": claim.effect_digest,
        "principal": claim.principal,
        "payer": claim.payer,
        "payee": claim.payee,
        "amount_minor": claim.amount_minor.to_string(),
        "amount_decimal": amount_decimal,
        "currency": claim.currency.code(),
        "obligation_id": claim.obligation_id,
        "purpose": claim.purpose,
        "authority_audience": claim.audience,
        "verified_custodian_ids": claim.verified_custodian_ids,
        "construct_digest": claim.construct_digest,
        "ledger_seq": entry.as_ref().map(|e| e.seq),
        "claim_state": serde_json::to_value(claim.state).map_err(|e| exec_err(e.to_string()))?,
        "reverses": claim.reverses,
        "detail": claim.detail,
    }))
}
