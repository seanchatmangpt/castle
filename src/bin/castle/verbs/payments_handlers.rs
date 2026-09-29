//! Payments handlers: all logic for the `payments` noun. Money is handled only as
//! canonical integer minor-unit strings; nothing here parses amounts numerically.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use castle::payments::admission::{admit_payment_screened, Screening, ScreeningEvidence};
use castle::payments::compliance::{ComplianceControl, SanctionsList};
use castle::payments::counterparty::CounterpartyRegistry;
use castle::payments::dirlock::{publish_new, publish_replace};
use castle::payments::event_receipt::{explain, seal_event_receipt, verify_event_receipt, EventInputs};
use castle::payments::execute_rail::{finalize_via_rail, submit_via_rail, FinalizeResult, RailExecutionParams, RailStandingAfterSubmit};
use castle::payments::rail::{RailInstruction, RailStatus};
use castle::payments::replay::{admit_payment_journaled, replay_admission, AdmissionJournal, ReplayVerdict};
use castle::payments::settlement::FinalityOutcome;
use castle::payments::{
    admit_payment, execute_payment, reconcile, recover_journal, AdmissionContext, ClaimState, ClaimStore, Currency,
    DurableNonceFence, EffectBindings, ExecutionContext, FileJournalLedger, LedgerPort, Money, PaymentEffect,
    PaymentStanding, ReconcileResolution, SimMode, SimRail, SpendPolicy,
};
use castle::sa2a_security::encoding::sha256_tagged;
use castle::sa2a_security::epoch::SecurityEpochs;
use castle::sa2a_security::{ActuationCertificate, KeyRecord, KeyRegistry, PreparedEffect};
use castle::v26_8_18::{decode_seed_hex, Ed25519RuntimeSigner, NativeBlake3};
use clap_noun_verb::NounVerbError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

type Result<T> = std::result::Result<T, NounVerbError>;

const REFERENCE_LEDGER_NOT_CONFIRMED: &str = "REFUSED:REFERENCE_LEDGER_NOT_CONFIRMED";
const LEDGER_NOT_FOUND: &str = "REFUSED:PAYMENT_LEDGER_NOT_FOUND";
const CLAIM_NOT_FOUND: &str = "REFUSED:PAYMENT_CLAIM_NOT_FOUND";
const RECONCILE_RAIL_HOLD_LIVE: &str = "REFUSED:PAYMENT_RECONCILE_RAIL_HOLD_LIVE";
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
    // A live rail hold means the authoritative record is the rail, not this ledger: the
    // ledger legitimately has no entry until finality, so "absent" here would be false.
    if ledger.hold_of(&effect_digest).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))?.is_some() {
        return Err(exec_err(RECONCILE_RAIL_HOLD_LIVE));
    }
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

// ---------------------------------------------------------------------------
// v26.9.29: rail submit, finalize, replay, receipt
// ---------------------------------------------------------------------------

const REFERENCE_RAIL_NOT_CONFIRMED: &str = "REFUSED:REFERENCE_RAIL_NOT_CONFIRMED";
const RAIL_NOT_FOUND: &str = "REFUSED:PAYMENT_RAIL_NOT_FOUND";
const RAIL_MODE_INVALID: &str = "REFUSED:PAYMENT_RAIL_MODE_INVALID";
const SCREENING_INPUTS_INCOMPLETE: &str = "REFUSED:PAYMENT_SCREENING_INPUTS_INCOMPLETE";
const JOURNAL_WITH_SCREENING_UNSUPPORTED: &str = "REFUSED:PAYMENT_JOURNAL_WITH_SCREENING_UNSUPPORTED";
const RAIL_HISTORY_MISSING_SETTLEMENT: &str = "REFUSED:RECEIPT_RAIL_HISTORY_MISSING_SETTLEMENT";
const FINALITY_DOMAIN: &[u8] = b"CASTLE-CLI-FINALITY-OBSERVATION-V1";
const DEFAULT_EFFECT_TTL_MS: u64 = 3_600_000;

fn require_reference_rail(confirmed: bool) -> Result<()> {
    if confirmed {
        Ok(())
    } else {
        Err(exec_err(REFERENCE_RAIL_NOT_CONFIRMED))
    }
}

fn parse_polls(text: &str) -> Result<u32> {
    text.parse::<u32>().map_err(|_| exec_err(format!("{RAIL_MODE_INVALID}:{text}")))
}

/// `honest|drop-ack|settle-after-polls:N|reject-after-polls:N|return-after-settle|down|duplicate-reports`
fn parse_sim_mode(text: &str) -> Result<SimMode> {
    match text {
        "honest" => Ok(SimMode::Honest),
        "drop-ack" => Ok(SimMode::DropAckAfterAccept),
        "return-after-settle" => Ok(SimMode::ReturnAfterSettle),
        "down" => Ok(SimMode::Down),
        "duplicate-reports" => Ok(SimMode::DuplicateReports),
        other => match other.split_once(':') {
            Some(("settle-after-polls", n)) => Ok(SimMode::SettleAfterPolls(parse_polls(n)?)),
            Some(("reject-after-polls", n)) => Ok(SimMode::RejectAfterPolls(parse_polls(n)?)),
            _ => Err(exec_err(format!("{RAIL_MODE_INVALID}:{other}"))),
        },
    }
}

/// Open the simulated rail. `create` is true only for `rail-submit`; every other verb
/// refuses a rail root that was never initialised instead of fabricating an empty rail.
fn open_sim_rail(rail_dir: &str, mode: Option<SimMode>, create: bool) -> Result<SimRail> {
    let mode_path = Path::new(rail_dir).join("mode.json");
    if !create && !mode_path.is_file() {
        return Err(exec_err(RAIL_NOT_FOUND));
    }
    let mode = match mode {
        Some(m) => m,
        None => {
            let bytes = std::fs::read(&mode_path).map_err(|_| exec_err(RAIL_NOT_FOUND))?;
            serde_json::from_slice(&bytes).map_err(|_| exec_err(RAIL_NOT_FOUND))?
        }
    };
    SimRail::open(Path::new(rail_dir), mode).map_err(exec_err)
}

fn hex_of(effect_digest: &str) -> String {
    effect_digest.trim_start_matches("sha256:").to_string()
}

fn cli_rail_dir(state_dir: &str) -> PathBuf {
    Path::new(state_dir).join("cli-rail")
}

fn submission_path(state_dir: &str, effect_digest: &str) -> PathBuf {
    cli_rail_dir(state_dir).join(format!("{}.submission.json", hex_of(effect_digest)))
}

fn finality_path(state_dir: &str, effect_digest: &str) -> PathBuf {
    cli_rail_dir(state_dir).join(format!("{}.finality.json", hex_of(effect_digest)))
}

/// What `rail-submit` observed, persisted so `receipt` can be built from disk alone.
#[derive(Debug, Serialize, Deserialize)]
struct SubmissionRecord {
    effect_id: String,
    pee_effect_id: String,
    correlation_id: String,
    payload_digest: String,
    standing: String,
    construct_digest: Option<String>,
    brce_prepare_receipt_digests: Vec<String>,
    brce_outcome_receipt_digests: Vec<String>,
    compliance_bundle_digest: Option<String>,
    counterparty_evidence_digest: Option<String>,
    detail: String,
}

/// The first time `finalize` saw the rail report settlement.
#[derive(Debug, Serialize, Deserialize)]
struct FinalityRecord {
    effect_id: String,
    evidence_digest: String,
    observed_at_ms: u64,
}

fn persist_new(state_dir: &str, path: &Path, value: &impl Serialize) -> Result<()> {
    let dir = cli_rail_dir(state_dir);
    std::fs::create_dir_all(&dir).map_err(|e| exec_err(format!("BLOCKED:PAYMENT_CLI_STATE_UNWRITABLE:{e}")))?;
    let bytes = serde_json::to_vec(value).map_err(|e| exec_err(e.to_string()))?;
    publish_new(&dir, path, &bytes).map_err(exec_err)?;
    Ok(())
}

fn persist_replace(state_dir: &str, path: &Path, value: &impl Serialize) -> Result<()> {
    let dir = cli_rail_dir(state_dir);
    std::fs::create_dir_all(&dir).map_err(|e| exec_err(format!("BLOCKED:PAYMENT_CLI_STATE_UNWRITABLE:{e}")))?;
    let bytes = serde_json::to_vec(value).map_err(|e| exec_err(e.to_string()))?;
    publish_replace(&dir, path, &bytes).map_err(exec_err)
}

fn read_optional<T: DeserializeOwned>(path: &Path, what: &str) -> Result<Option<T>> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|e| exec_err(format!("REFUSED:PAYMENT_INPUT_INVALID:{what}:{e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(exec_err(format!("REFUSED:PAYMENT_INPUT_UNREADABLE:{what}:{e}"))),
    }
}

fn claim_state_name(state: ClaimState) -> Result<String> {
    match serde_json::to_value(state).map_err(|e| exec_err(e.to_string()))? {
        Value::String(s) => Ok(s),
        other => Ok(other.to_string()),
    }
}

pub struct RailSubmitArgs {
    pub base: ExecuteArgs,
    pub reference_rail: bool,
    pub rail_dir: String,
    pub rail_mode: String,
    pub sanctions_path: Option<String>,
    pub counterparties_path: Option<String>,
    pub journal_dir: Option<String>,
    pub debtor_agent_bic: String,
    pub creditor_agent_bic: String,
    pub created_at_iso: String,
    pub debtor_name: Option<String>,
    pub creditor_name: Option<String>,
    pub effect_expires_at_ms: Option<String>,
}

pub fn rail_submit_handler(a: RailSubmitArgs) -> Result<Value> {
    let b = &a.base;
    // Both confirmations precede any read or write: SimRail is a test rail, not a bank.
    require_reference_ledger(b.reference_ledger)?;
    require_reference_rail(a.reference_rail)?;
    let mode = parse_sim_mode(&a.rail_mode)?;
    let screening_files = match (&a.sanctions_path, &a.counterparties_path) {
        (Some(s), Some(c)) => Some((s.clone(), c.clone())),
        (None, None) => None,
        _ => return Err(exec_err(SCREENING_INPUTS_INCOMPLETE)),
    };
    if screening_files.is_some() && a.journal_dir.is_some() {
        return Err(exec_err(JOURNAL_WITH_SCREENING_UNSUPPORTED));
    }

    let prepared: PreparedEffect = read_json(&b.effect_path, "effect")?;
    let certificate: ActuationCertificate = read_json(&b.certificate_path, "certificate")?;
    let records: Vec<KeyRecord> = read_json(&b.registry_path, "registry")?;
    let policy: SpendPolicy = read_json(&b.policy_path, "policy")?;
    let opening_rows: Vec<OpeningRow> = read_json(&b.opening_path, "opening")?;
    let opening: Vec<(&str, Currency, u64)> =
        opening_rows.iter().map(|r| (r.account.as_str(), r.currency, r.amount_minor)).collect();
    let sanctions = match &screening_files {
        Some((s, _)) => Some(
            SanctionsList::from_json(&std::fs::read(s).map_err(|e| exec_err(format!("REFUSED:PAYMENT_INPUT_UNREADABLE:sanctions:{e}")))?)
                .map_err(exec_err)?,
        ),
        None => None,
    };
    let counterparties = match &screening_files {
        Some((_, c)) => Some(
            CounterpartyRegistry::from_json(&std::fs::read(c).map_err(|e| exec_err(format!("REFUSED:PAYMENT_INPUT_UNREADABLE:counterparties:{e}")))?)
                .map_err(exec_err)?,
        ),
        None => None,
    };

    let registry = KeyRegistry::from_records(records);
    let epochs = SecurityEpochs {
        policy: to_u64(b.policy_epoch, "policy_epoch")?,
        revocation: to_u64(b.revocation_epoch, "revocation_epoch")?,
        generation: to_u64(b.generation, "generation")?,
    };
    let now_ms = to_u64(b.now_ms, "now_ms")?;
    let expires_at_ms = match &a.effect_expires_at_ms {
        Some(t) => t.parse::<u64>().map_err(|_| exec_err("REFUSED:PAYMENT_INPUT_INVALID:effect_expires_at_ms"))?,
        None => now_ms.saturating_add(DEFAULT_EFFECT_TTL_MS),
    };

    let state = Path::new(&b.state_dir);
    let claims = ClaimStore::open(claims_dir(&b.state_dir)).map_err(exec_err)?;
    let nonces = DurableNonceFence::open(state.join("nonces")).map_err(exec_err)?;
    let ledger = FileJournalLedger::open(&b.ledger_dir, &opening).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))?;
    let rail = open_sim_rail(&a.rail_dir, Some(mode), true)?;

    let seed = decode_seed_hex(&b.receipt_seed_hex).map_err(exec_err)?;
    let signer = Ed25519RuntimeSigner::from_seed(b.receipt_key_id.clone(), seed).map_err(exec_err)?;
    let verifier = signer.verifier();
    let blake3 = NativeBlake3;

    let admission_ctx = AdmissionContext {
        registry: &registry,
        epochs,
        audience: &b.audience,
        now_ms,
        policy: &policy,
        claims: &claims,
        nonces: &nonces,
    };
    let (admission, screening_evidence): (_, Option<ScreeningEvidence>) = match (&sanctions, &counterparties) {
        (Some(list), Some(cps)) => {
            let controls: [&dyn ComplianceControl; 1] = [list];
            let (adm, ev) = admit_payment_screened(
                prepared,
                &certificate,
                &admission_ctx,
                &Screening { controls: &controls, counterparties: cps },
            )
            .map_err(exec_err)?;
            (adm, Some(ev))
        }
        _ => match &a.journal_dir {
            Some(dir) => {
                let journal = AdmissionJournal::open(dir).map_err(exec_err)?;
                (admit_payment_journaled(prepared, &certificate, &admission_ctx, &journal).map_err(exec_err)?, None)
            }
            None => (admit_payment(prepared, &certificate, &admission_ctx).map_err(exec_err)?, None),
        },
    };

    let payer = admission.effect().payer().to_string();
    let digest = admission.effect().digest().to_string();
    let params = RailExecutionParams {
        bindings: EffectBindings {
            policy_profile_id: "policy:cli".into(),
            law_state_digest: screening_evidence
                .as_ref()
                .map_or_else(|| "law:unscreened".to_string(), |e| e.compliance_bundle_digest.clone()),
            counterparty_evidence_digest: screening_evidence
                .as_ref()
                .map_or_else(|| "cp:unscreened".to_string(), |e| e.counterparty_evidence_digest.clone()),
            funding_source_id: format!("fund:{payer}"),
            resource_reservation_id: format!("res:{}", hex_of(&digest)),
            rail_profile_id: "sim-rail".into(),
            message_profile_version: "pain.001.001.09".into(),
            parent_receipt: "receipt:none".into(),
            created_at_ms: now_ms,
            expires_at_ms,
        },
        created_at_iso: a.created_at_iso.clone(),
        debtor_name: a.debtor_name.clone().unwrap_or_else(|| admission.effect().payer().to_string()),
        creditor_name: a.creditor_name.clone().unwrap_or_else(|| admission.effect().payee().to_string()),
        debtor_agent_bic: a.debtor_agent_bic.clone(),
        creditor_agent_bic: a.creditor_agent_bic.clone(),
        rail_profile: "pain.001.001.09".into(),
    };

    let ctx = ExecutionContext {
        blake3: &blake3,
        signer: &signer,
        verifier: &verifier,
        allowed_authorities: BTreeSet::from([b.allowed_authority.clone()]),
        journal_root: state.join("brce"),
        now_epoch_ms: b.now_ms,
        ledger: &ledger,
        claims: &claims,
        policy: &policy,
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(|e| exec_err(format!("BLOCKED:PAYMENT_RUNTIME_UNAVAILABLE:{e}")))?;
    let submission = runtime.block_on(submit_via_rail(admission, &params, &ctx, &rail)).map_err(exec_err)?;

    let standing = match submission.standing {
        RailStandingAfterSubmit::Submitted => "Submitted",
        RailStandingAfterSubmit::Refused => "Refused",
        RailStandingAfterSubmit::UnknownOutcome => "UnknownOutcome",
    };
    let record = SubmissionRecord {
        effect_id: submission.effect_id.clone(),
        pee_effect_id: submission.pee.effect_id.clone(),
        correlation_id: submission.correlation_id.clone(),
        payload_digest: submission.payload_digest.clone(),
        standing: standing.to_string(),
        construct_digest: submission.construct_digest.clone(),
        brce_prepare_receipt_digests: submission.brce_prepare_receipt_digests.clone(),
        brce_outcome_receipt_digests: submission.brce_outcome_receipt_digests.clone(),
        compliance_bundle_digest: screening_evidence.as_ref().map(|e| e.compliance_bundle_digest.clone()),
        counterparty_evidence_digest: screening_evidence.as_ref().map(|e| e.counterparty_evidence_digest.clone()),
        detail: submission.detail.clone(),
    };
    persist_replace(&b.state_dir, &submission_path(&b.state_dir, &submission.effect_id), &record)?;

    if submission.standing != RailStandingAfterSubmit::Submitted {
        return Err(exec_err(format!("{} standing={standing} effect_digest={}", submission.detail, submission.effect_id)));
    }
    Ok(json!({
        "standing": standing,
        "effect_digest": submission.effect_id,
        "pee_effect_id": submission.pee.effect_id,
        "rail_correlation_id": submission.correlation_id,
        "rail_payload_digest": submission.payload_digest,
        "construct_digest": submission.construct_digest,
        "brce_prepare_receipt_digests": submission.brce_prepare_receipt_digests,
        "brce_outcome_receipt_digests": submission.brce_outcome_receipt_digests,
        "screened": screening_evidence.is_some(),
        "journaled": a.journal_dir.is_some(),
        "rail": "SimRail (test rail, not a bank)",
        "detail": submission.detail,
    }))
}

fn observed_settlement(rail: &SimRail, correlation: &str) -> Option<RailStatus> {
    rail.status_history(correlation).into_iter().find(|s| matches!(s, RailStatus::Settled { .. }))
}

fn now_epoch_ms() -> Result<u64> {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| exec_err(format!("BLOCKED:PAYMENT_CLOCK_UNAVAILABLE:{e}")))?;
    u64::try_from(d.as_millis()).map_err(|_| exec_err("BLOCKED:PAYMENT_CLOCK_UNAVAILABLE"))
}

pub fn finalize_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    rail_mode: Option<String>,
    reference_ledger: bool,
    reference_rail: bool,
) -> Result<Value> {
    require_reference_ledger(reference_ledger)?;
    require_reference_rail(reference_rail)?;
    let mode = rail_mode.as_deref().map(parse_sim_mode).transpose()?;
    let ledger = open_existing_ledger(&ledger_dir)?;
    let claims = ClaimStore::open(claims_dir(&state_dir)).map_err(exec_err)?;
    let rail = open_sim_rail(&rail_dir, mode, false)?;

    let result = finalize_via_rail(&effect_digest, &claims, &ledger, &rail).map_err(exec_err)?;
    let claim = claims.get(&effect_digest).map_err(exec_err)?.ok_or_else(|| exec_err(CLAIM_NOT_FOUND))?;
    let claim_state = claim_state_name(claim.state)?;

    // Persist the first observation of rail settlement (observation time, not authority).
    if matches!(claim.state, ClaimState::Final | ClaimState::Returned) {
        let correlation = RailInstruction::correlation_id_for(&effect_digest);
        if let Some(status) = observed_settlement(&rail, &correlation) {
            let body = serde_json::to_vec(&status).map_err(|e| exec_err(e.to_string()))?;
            let evidence = sha256_tagged(FINALITY_DOMAIN, &[correlation.as_bytes(), b"|", &body].concat());
            persist_new(
                &state_dir,
                &finality_path(&state_dir, &effect_digest),
                &FinalityRecord { effect_id: effect_digest.clone(), evidence_digest: evidence, observed_at_ms: now_epoch_ms()? },
            )?;
        }
    }

    let (label, outcome) = match result {
        FinalizeResult::Pending => ("pending", Value::Null),
        FinalizeResult::StillUnknown => ("still_unknown", Value::Null),
        FinalizeResult::ProvenAbsent => ("proven_absent", Value::Null),
        FinalizeResult::Applied { outcome: o, .. } => (
            "applied",
            match o {
                FinalityOutcome::Settled(e) => json!({ "outcome": "settled", "ledger_entry": e }),
                FinalityOutcome::AlreadyFinal => json!({ "outcome": "already_final" }),
                FinalityOutcome::Released => json!({ "outcome": "released" }),
                FinalityOutcome::Returned(e) => json!({ "outcome": "returned", "ledger_entry": e }),
            },
        ),
    };
    let body = json!({ "effect_digest": effect_digest, "result": label, "applied": outcome, "claim_state": claim_state });
    if matches!(claim.state, ClaimState::Refused | ClaimState::UnknownOutcome) {
        return Err(exec_err(format!("{claim_state} finalize_result={label} {body}")));
    }
    Ok(body)
}

pub fn replay_handler(journal_dir: String, effect_digest: String) -> Result<Value> {
    if !Path::new(&journal_dir).is_dir() {
        return Err(exec_err(castle::payments::replay::REPLAY_RECORD_MISSING));
    }
    let journal = AdmissionJournal::open(&journal_dir).map_err(exec_err)?;
    match replay_admission(&journal, &effect_digest).map_err(exec_err)? {
        ReplayVerdict::Reproduced { effect_digest, decision_digest } => Ok(json!({
            "verdict": "Reproduced",
            "effect_digest": effect_digest,
            "decision_digest": decision_digest,
            "scope": "stateless checks only; nonce, budget and claim uniqueness are excluded",
        })),
        ReplayVerdict::Diverged { reason } => Err(exec_err(format!("{reason} verdict=Diverged effect_digest={effect_digest}"))),
    }
}

pub fn receipt_handler(
    effect_digest: String,
    state_dir: String,
    ledger_dir: String,
    rail_dir: String,
    journal_dir: String,
    reference_ledger: bool,
    reference_rail: bool,
) -> Result<Value> {
    require_reference_ledger(reference_ledger)?;
    require_reference_rail(reference_rail)?;
    let ledger = open_existing_ledger(&ledger_dir)?;
    let claims = ClaimStore::open(claims_dir(&state_dir)).map_err(exec_err)?;
    let rail = open_sim_rail(&rail_dir, None, false)?;
    let claim = claims.get(&effect_digest).map_err(exec_err)?.ok_or_else(|| exec_err(CLAIM_NOT_FOUND))?;
    let entry = ledger.lookup(&effect_digest).map_err(|e| exec_err(format!("{LEDGER_UNAVAILABLE}:{e:?}")))?;

    if !Path::new(&journal_dir).is_dir() {
        return Err(exec_err(castle::payments::replay::REPLAY_RECORD_MISSING));
    }
    let journal = AdmissionJournal::open(&journal_dir).map_err(exec_err)?;
    let decision_digest = match replay_admission(&journal, &effect_digest).map_err(exec_err)? {
        ReplayVerdict::Reproduced { decision_digest, .. } => decision_digest,
        ReplayVerdict::Diverged { reason } => return Err(exec_err(format!("{reason} verdict=Diverged"))),
    };

    let submission: Option<SubmissionRecord> = read_optional(&submission_path(&state_dir, &effect_digest), "submission")?;
    let finality: Option<FinalityRecord> = read_optional(&finality_path(&state_dir, &effect_digest), "finality")?;
    if let Some(s) = &submission {
        let expected = RailInstruction::correlation_id_for(&effect_digest);
        if s.correlation_id != expected || s.effect_id != effect_digest {
            return Err(exec_err("REFUSED:RECEIPT_SUBMISSION_RECORD_MISMATCH"));
        }
        // The books may show settlement only if the rail itself reported it.
        if matches!(claim.state, ClaimState::Final | ClaimState::Returned) && observed_settlement(&rail, &expected).is_none() {
            return Err(exec_err(RAIL_HISTORY_MISSING_SETTLEMENT));
        }
    }

    let inputs = EventInputs {
        effect_id: claim.effect_digest.clone(),
        obligation_id: claim.obligation_id.clone(),
        admission_decision_digest: decision_digest,
        pee_effect_id: submission.as_ref().map(|s| s.pee_effect_id.clone()),
        construct_digest: claim.construct_digest.clone(),
        brce_prepare_digests: submission.as_ref().map(|s| s.brce_prepare_receipt_digests.clone()).unwrap_or_default(),
        brce_outcome_digests: submission.as_ref().map(|s| s.brce_outcome_receipt_digests.clone()).unwrap_or_default(),
        rail_correlation_id: submission.as_ref().map(|s| s.correlation_id.clone()),
        rail_payload_digest: submission.as_ref().map(|s| s.payload_digest.clone()),
        finality_evidence_digest: finality.as_ref().map(|f| f.evidence_digest.clone()),
        ledger_entry_digest: entry.as_ref().map(castle::payments::event_receipt::ledger_entry_digest),
        claim_state: claim_state_name(claim.state)?,
        compliance_bundle_digest: submission.as_ref().and_then(|s| s.compliance_bundle_digest.clone()),
        counterparty_evidence_digest: submission.as_ref().and_then(|s| s.counterparty_evidence_digest.clone()),
        settled_at_ms: finality.as_ref().map(|f| f.observed_at_ms),
    };
    let receipt = seal_event_receipt(&inputs).map_err(exec_err)?;
    verify_event_receipt(&receipt).map_err(exec_err)?;
    Ok(json!({
        "receipt": serde_json::to_value(&receipt).map_err(|e| exec_err(e.to_string()))?,
        "explain": explain(&receipt),
        "verified": true,
    }))
}
