//! The single EVENT RECEIPT (court step 15): one digest-sealed record binding
//! an economic event's admission, CONSTRUCT, BRCE, rail, finality, ledger and
//! compliance evidence. `explain` answers the audit questions from the
//! receipt alone; fields the receipt does not carry are null with a reason.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::sa2a_security::encoding::sha256_tagged;

use super::ledger::LedgerEntry;
use super::refusal::PayResult;

pub const EVENT_RECEIPT_INCOMPLETE: &str = "REFUSED:EVENT_RECEIPT_INCOMPLETE";
pub const EVENT_RECEIPT_TAMPERED: &str = "REFUSED:EVENT_RECEIPT_TAMPERED";

const RECEIPT_DOMAIN: &[u8] = b"CASTLE-EVENT-RECEIPT-V1";
const LEDGER_DOMAIN: &[u8] = b"CASTLE-LEDGER-ENTRY-V1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventInputs {
    pub effect_id: String,
    pub obligation_id: String,
    pub admission_decision_digest: String,
    pub pee_effect_id: Option<String>,
    pub construct_digest: Option<String>,
    pub brce_prepare_digests: Vec<String>,
    pub brce_outcome_digests: Vec<String>,
    pub rail_correlation_id: Option<String>,
    pub rail_payload_digest: Option<String>,
    pub finality_evidence_digest: Option<String>,
    pub ledger_entry_digest: Option<String>,
    pub claim_state: String,
    pub compliance_bundle_digest: Option<String>,
    pub counterparty_evidence_digest: Option<String>,
    pub settled_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventReceipt {
    pub inputs: EventInputs,
    pub receipt_digest: String,
}

fn digest_of(inputs: &EventInputs) -> PayResult<String> {
    let bytes = serde_json_canonicalizer::to_vec(inputs).map_err(|_| EVENT_RECEIPT_INCOMPLETE.to_string())?;
    Ok(sha256_tagged(RECEIPT_DOMAIN, &bytes))
}

fn complete(i: &EventInputs) -> bool {
    !i.effect_id.is_empty() && !i.obligation_id.is_empty() && !i.admission_decision_digest.is_empty()
}

pub fn seal_event_receipt(inputs: &EventInputs) -> PayResult<EventReceipt> {
    if !complete(inputs) {
        return Err(EVENT_RECEIPT_INCOMPLETE.to_string());
    }
    Ok(EventReceipt { inputs: inputs.clone(), receipt_digest: digest_of(inputs)? })
}

pub fn verify_event_receipt(receipt: &EventReceipt) -> PayResult<()> {
    match digest_of(&receipt.inputs) {
        Ok(d) if d == receipt.receipt_digest && complete(&receipt.inputs) => Ok(()),
        _ => Err(EVENT_RECEIPT_TAMPERED.to_string()),
    }
}

#[must_use]
pub fn ledger_entry_digest(entry: &LedgerEntry) -> String {
    let bytes = serde_json_canonicalizer::to_vec(entry).unwrap_or_default();
    sha256_tagged(LEDGER_DOMAIN, &bytes)
}

fn field(value: Option<Value>, reason: &str) -> Value {
    match value {
        Some(v) => v,
        None => json!({ "value": null, "reason": reason }),
    }
}

/// Answer the audit questions from the receipt alone. Never invents: anything
/// the receipt does not carry is `{value: null, reason}`.
#[must_use]
pub fn explain(receipt: &EventReceipt) -> Value {
    let i = &receipt.inputs;
    json!({
        "receipt_digest": receipt.receipt_digest,
        "whose_money": field(None, "receipt carries only digests; payer/principal live in the ledger entry and claim keyed by effect_id"),
        "to_whom": field(None, "receipt carries only digests; payee lives in the ledger entry and claim keyed by effect_id"),
        "why": { "obligation_id": i.obligation_id, "effect_id": i.effect_id },
        "under_whose_authority": { "admission_decision_digest": i.admission_decision_digest, "construct_digest": i.construct_digest },
        "through_which_rail": field(
            i.rail_correlation_id.as_ref().map(|c| json!({ "correlation_id": c, "payload_digest": i.rail_payload_digest })),
            "no rail correlation id: internal-ledger or pre-rail event",
        ),
        "when_final": field(
            i.settled_at_ms.map(|t| json!({ "settled_at_ms": t, "finality_evidence_digest": i.finality_evidence_digest })),
            "no settled_at_ms: finality not observed",
        ),
        "effect": field(
            i.pee_effect_id.as_ref().map(|id| json!({ "pee_effect_id": id })),
            "no pee_effect_id: event was not bound to a sealed PreparedEconomicEffect",
        ),
        "what_happened": { "claim_state": i.claim_state, "ledger_entry_digest": i.ledger_entry_digest },
    })
}
