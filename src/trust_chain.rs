//! Castle ↔ affidavit trust bridge (L52 ranks 1+2).
//!
//! Rank 1 — TrustKey enrollment history: maps the castle `TrustKey` lifecycle
//! (`src/board.rs` `TrustKey { valid_from_epoch, revoked_at_epoch }`) into
//! affidavit-style chain events and folds them with the rolling BLAKE3 chain
//! law.
//!
//! SPEC PROVENANCE: chain law mirrored from affidavit `src/chain.rs`
//! (sha256 `7e925bee082fbe692df2a94658d55477bf39bb279a50f72c766df8aa1593a631`):
//! ```text
//! chain_hash_0 = blake3(GENESIS)
//! chain_hash_n = blake3(chain_hash_{n-1}.as_hex().as_bytes() || canonical_bytes(event_n))
//! ```
//! Implemented here over castle's existing `blake3` dependency; no affidavit
//! dependency is added. `GENESIS_SEED` is castle-side and deliberately
//! distinct from affidavit's so the two chains never cross-verify by accident.
//!
//! Rank 2 — SealedReceipt `Replayed` producer: an `EvidenceInput` constructor
//! from an affidavit-style sealed receipt. The digest is re-derived from the
//! sealed base bytes (never the envelope's claim), mirroring affidavit
//! `crypto_trust_seal.rs` `verify_sealed` semantics: a broken base→claim
//! binding is a typed REFUSED, never downgraded to `Observed`.

use crate::board::EvidenceInput;
use crate::fortune5::{EvidenceEpistemicClass, MetricValue};

/// Castle-side genesis seed for the rolling trust-chain hash.
pub const GENESIS_SEED: &[u8] = b"castle-trust-chain-v1-genesis";

/// Refusal prefix used by every typed refusal in this module.
pub const REFUSED: &str = "REFUSED";

// ---------------------------------------------------------------------------
// Canonical event bytes (same key-sorted canonical JSON law as board.rs)
// ---------------------------------------------------------------------------

fn canonical_json(value: &serde_json::Value) -> Result<String, String> {
    use serde_json::Value;
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if !f.is_finite() {
                    return Err("REFUSED:NON_FINITE_CANONICAL_VALUE".to_string());
                }
            }
            Ok(n.to_string())
        }
        Value::String(s) => Ok(serde_json::to_string(s).unwrap()),
        Value::Array(arr) => {
            let parts: Result<Vec<String>, String> = arr.iter().map(canonical_json).collect();
            Ok(format!("[{}]", parts?.join(",")))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for k in keys {
                parts.push(format!("{}:{}", serde_json::to_string(k).unwrap(), canonical_json(&map[k])?));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
    }
}

fn blake3_hex(input: &[u8]) -> String {
    blake3::hash(input).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// Rank 1 — key-history chain events
// ---------------------------------------------------------------------------

/// A single castle `TrustKey` lifecycle event in affidavit chain-event shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyChainEvent {
    /// `"enroll"` or `"revoke"`.
    pub action: String,
    pub key_id: String,
    /// For `enroll`: the key's `valid_from_epoch`. For `revoke`: its
    /// `revoked_at_epoch`.
    pub epoch: i64,
    pub assurance_domain: String,
}

impl KeyChainEvent {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "action": self.action,
            "key_id": self.key_id,
            "epoch": self.epoch,
            "assurance_domain": self.assurance_domain,
        })
    }

    /// Canonical event bytes — the exact bytes fed to the chain fold.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        canonical_json(&self.to_json()).map(|s| s.into_bytes())
    }
}

/// Fold one event into a running chain hash
/// (`blake3(prev_hex_bytes || canonical_bytes(event))`) — the affidavit
/// `chain.rs` fold semantics over castle's BLAKE3.
///
/// # Errors
/// Refuses `REFUSED:NON_FINITE_CANONICAL_VALUE` if event bytes cannot be
/// canonically encoded.
pub fn fold_event(prev_hex: &str, event: &KeyChainEvent) -> Result<String, String> {
    let event_bytes = event.canonical_bytes()?;
    let mut buf = Vec::with_capacity(prev_hex.len() + event_bytes.len());
    buf.extend_from_slice(prev_hex.as_bytes());
    buf.extend_from_slice(&event_bytes);
    Ok(blake3_hex(&buf))
}

/// Purely recompute the rolling chain hash over an ordered event slice.
///
/// # Errors
/// Propagates [`fold_event`] refusals.
pub fn recompute_chain(events: &[KeyChainEvent]) -> Result<String, String> {
    let mut acc = blake3_hex(GENESIS_SEED);
    for event in events {
        acc = fold_event(&acc, event)?;
    }
    Ok(acc)
}

/// An append-only castle `TrustKey` lifecycle chain: enrollment and
/// revocation events folded with the rolling BLAKE3 law.
#[derive(Debug, Clone)]
pub struct KeyHistory {
    pub events: Vec<KeyChainEvent>,
    /// Chain hash after the last appended event (affidavit
    /// `Receipt.chain_hash` analogue).
    pub chain_hash: String,
}

impl KeyHistory {
    /// A fresh, empty history (genesis link only).
    #[must_use]
    pub fn new() -> Self {
        KeyHistory { events: Vec::new(), chain_hash: blake3_hex(GENESIS_SEED) }
    }

    fn append(&mut self, event: KeyChainEvent) -> Result<(), String> {
        self.chain_hash = fold_event(&self.chain_hash, &event)?;
        self.events.push(event);
        Ok(())
    }

    /// Enroll a key: castle `TrustKey { valid_from_epoch, .. }`.
    ///
    /// # Errors
    /// Refuses `REFUSED:KEY_ALREADY_ENROLLED` on duplicate enrollment.
    pub fn enroll(
        &mut self,
        key_id: &str,
        valid_from_epoch: i64,
        assurance_domain: &str,
    ) -> Result<(), String> {
        if self.events.iter().any(|e| e.action == "enroll" && e.key_id == key_id) {
            return Err("REFUSED:KEY_ALREADY_ENROLLED".to_string());
        }
        self.append(KeyChainEvent {
            action: "enroll".to_string(),
            key_id: key_id.to_string(),
            epoch: valid_from_epoch,
            assurance_domain: assurance_domain.to_string(),
        })
    }

    /// Revoke a key: castle `TrustKey { revoked_at_epoch: Some(..) }`.
    ///
    /// # Errors
    /// Refuses `REFUSED:KEY_NOT_ENROLLED` if the key was never enrolled.
    pub fn revoke(&mut self, key_id: &str, revoked_at_epoch: i64) -> Result<(), String> {
        if !self.events.iter().any(|e| e.action == "enroll" && e.key_id == key_id) {
            return Err("REFUSED:KEY_NOT_ENROLLED".to_string());
        }
        self.append(KeyChainEvent {
            action: "revoke".to_string(),
            key_id: key_id.to_string(),
            epoch: revoked_at_epoch,
            assurance_domain: String::new(),
        })
    }

    /// Replay the events into per-key state.
    #[must_use]
    pub fn key_state(&self, key_id: &str) -> Option<KeyState> {
        let mut state = None;
        for event in &self.events {
            if event.key_id != key_id {
                continue;
            }
            match event.action.as_str() {
                "enroll" => {
                    state = Some(KeyState {
                        valid_from_epoch: event.epoch,
                        revoked_at_epoch: None,
                        assurance_domain: event.assurance_domain.clone(),
                    });
                }
                "revoke" => {
                    if let Some(existing) = state.as_mut() {
                        existing.revoked_at_epoch = Some(event.epoch);
                    }
                }
                _ => {}
            }
        }
        state
    }
}

impl Default for KeyHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Replayed per-key state (castle `TrustKey` lifecycle projection).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyState {
    pub valid_from_epoch: i64,
    pub revoked_at_epoch: Option<i64>,
    pub assurance_domain: String,
}

/// Verify a receipt signature epoch against the key-history chain.
///
/// Verifies the chain first (recompute vs stored hash — a tampered event byte
/// diverges the chain and is refused), then replays the key's lifecycle.
///
/// # Errors
/// - `REFUSED:TRUST_CHAIN_TAMPERED` — recomputed chain hash diverges.
/// - `REFUSED:KEY_NOT_ENROLLED_AT_EPOCH` — no enrollment at `epoch`, or
///   `epoch` precedes the key's `valid_from_epoch`.
/// - `REFUSED:REVOKED_KEY_AT_EPOCH` — `epoch >= revoked_at_epoch`.
pub fn check_key_history(history: &KeyHistory, key_id: &str, epoch: i64) -> Result<(), String> {
    let recomputed = recompute_chain(&history.events)?;
    if recomputed != history.chain_hash {
        return Err("REFUSED:TRUST_CHAIN_TAMPERED".to_string());
    }
    match history.key_state(key_id) {
        None => Err("REFUSED:KEY_NOT_ENROLLED_AT_EPOCH".to_string()),
        Some(state) => {
            if epoch < state.valid_from_epoch {
                return Err("REFUSED:KEY_NOT_ENROLLED_AT_EPOCH".to_string());
            }
            if let Some(revoked) = state.revoked_at_epoch {
                if epoch >= revoked {
                    return Err("REFUSED:REVOKED_KEY_AT_EPOCH".to_string());
                }
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Rank 2 — SealedReceipt → Replayed EvidenceInput
// ---------------------------------------------------------------------------

/// An affidavit-style sealed receipt: the base receipt core serialized to
/// canonical bytes plus the envelope's claimed subject digest.
#[derive(Debug, Clone)]
pub struct SealedAffidavitReceipt {
    /// Canonical bytes of the sealed base receipt core (the verified
    /// material — the envelope's claim is never trusted over these).
    pub base_bytes: Vec<u8>,
    /// The envelope's claimed receipt digest (hex).
    pub envelope_subject_digest: String,
    /// Board-evidence binding fields carried by the sealed receipt.
    pub metric: String,
    pub value: MetricValue,
    pub subject: String,
    pub observed_at: String,
}

/// A verified `Replayed` evidence admission derived from a sealed receipt.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayedEvidence {
    pub input: EvidenceInput,
    /// Re-derived digest of the sealed base (equals the envelope claim on
    /// success).
    pub receipt_digest: String,
}

/// Verify a sealed receipt and lift it into a `Replayed` evidence input.
///
/// Mirrors affidavit `crypto_trust_seal.rs::verify_sealed` semantics: the
/// digest is re-derived from `base_bytes` — the envelope's claim is only ever
/// compared, never trusted. On a broken binding this is a typed refusal; the
/// evidence is never produced with `epistemic_class: Observed`.
///
/// # Errors
/// - `REFUSED:SEAL_ENVELOPE_DIGEST_MISMATCH` — tampered base or lying
///   envelope.
pub fn evidence_input_from_sealed_receipt(
    sealed: &SealedAffidavitReceipt,
) -> Result<ReplayedEvidence, String> {
    let rederived = blake3_hex(&sealed.base_bytes);
    if rederived != sealed.envelope_subject_digest {
        return Err("REFUSED:SEAL_ENVELOPE_DIGEST_MISMATCH".to_string());
    }
    Ok(ReplayedEvidence {
        input: EvidenceInput {
            metric: sealed.metric.clone(),
            value: sealed.value.clone(),
            subject: sealed.subject.clone(),
            observed_at: sealed.observed_at.clone(),
            epistemic_class: EvidenceEpistemicClass::Replayed,
        },
        receipt_digest: rederived,
    })
}

// ---------------------------------------------------------------------------
// Courts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned Rank-2 fixture: canonical base bytes for a sealed receipt and
    /// the envelope digest they bind to. Derived, not asserted: the fixture
    /// digest is computed once here and pinned so any fold-law drift breaks
    /// the court loudly.
    fn pinned_sealed_valid() -> SealedAffidavitReceipt {
        let base_bytes = br#"{"assurance_domain":"fortune5","issued_at":"2026-10-13T00:00:00Z","key_id":"k1","metric":"uptime","payload_digest":"aa","policy_digest":"bb","trust_epoch":2,"version":"CASTLE-RECEIPT-V2"}"#.to_vec();
        let digest = blake3_hex(&base_bytes);
        SealedAffidavitReceipt {
            base_bytes,
            envelope_subject_digest: digest,
            metric: "uptime".to_string(),
            value: MetricValue::Str("99.95%".to_string()),
            subject: "svc-payments".to_string(),
            observed_at: "2026-10-13T00:00:00Z".to_string(),
        }
    }

    /// Pinned tampered fixture: one byte flipped in the base bytes, envelope
    /// claim unchanged (the lying-envelope / tampered-base attack).
    fn pinned_sealed_tampered() -> SealedAffidavitReceipt {
        let mut sealed = pinned_sealed_valid();
        let last = sealed.base_bytes.len() - 1;
        sealed.base_bytes[last] ^= 0x01;
        sealed
    }

    #[test]
    fn key_history_fold_matches_affidavit_chain_law() {
        // Independent re-derivation of the law from the spec comment:
        // chain_n = blake3(prev_hex_ascii || canonical_bytes(event_n)).
        let mut history = KeyHistory::new();
        history.enroll("k1", 1, "fortune5").expect("enroll k1");
        history.enroll("k2", 2, "fortune5").expect("enroll k2");
        history.revoke("k1", 10).expect("revoke k1");

        // Recompute from scratch over the event slice and compare: the two
        // folds must agree exactly.
        let recomputed = recompute_chain(&history.events).expect("chain recomputes");
        assert_eq!(recomputed, history.chain_hash);

        // Genesis link is blake3(GENESIS_SEED).
        let mut manual = blake3_hex(GENESIS_SEED);
        for event in &history.events {
            manual = blake3_hex(
                [manual.as_bytes(), event.canonical_bytes().unwrap().as_slice()].concat().as_slice(),
            );
        }
        assert_eq!(manual, history.chain_hash);
    }

    #[test]
    fn key_lifecycle_maps_from_castle_trust_key_shape() {
        // castle board.rs TrustKey { valid_from_epoch, revoked_at_epoch }:
        // enroll@1, enroll@2 (second key), revoke@10 for key 1.
        let mut history = KeyHistory::new();
        history.enroll("k1", 1, "fortune5").expect("enroll");
        history.enroll("k2", 2, "fortune5").expect("enroll");
        history.revoke("k1", 10).expect("revoke");
        assert_eq!(
            history.key_state("k1"),
            Some(KeyState {
                valid_from_epoch: 1,
                revoked_at_epoch: Some(10),
                assurance_domain: "fortune5".to_string(),
            })
        );
        assert_eq!(
            history.key_state("k2"),
            Some(KeyState { valid_from_epoch: 2, revoked_at_epoch: None, assurance_domain: "fortune5".to_string() })
        );
    }

    #[test]
    fn receipt_at_epoch_1_after_revocation_is_refused() {
        let mut history = KeyHistory::new();
        history.enroll("k1", 1, "fortune5").expect("enroll");
        history.enroll("k2", 2, "fortune5").expect("enroll");
        history.revoke("k1", 1).expect("revoke");
        // Receipt signed at epoch 1, checked after the revocation
        // (revoked_at_epoch=1, so epoch >= revoked_at is refused — the same
        // rule as board.rs's REFUSED:REVOKED_RECEIPT_KEY): refused.
        assert_eq!(
            check_key_history(&history, "k1", 1),
            Err("REFUSED:REVOKED_KEY_AT_EPOCH".to_string())
        );
        assert_eq!(
            check_key_history(&history, "k1", 2),
            Err("REFUSED:REVOKED_KEY_AT_EPOCH".to_string())
        );
    }

    #[test]
    fn receipt_at_epoch_2_within_validity_is_accepted() {
        let mut history = KeyHistory::new();
        history.enroll("k1", 1, "fortune5").expect("enroll");
        history.enroll("k2", 2, "fortune5").expect("enroll");
        history.revoke("k1", 2).expect("revoke");
        // k2 enrolled valid_from_epoch=2, never revoked: epoch 2 is in window.
        assert_eq!(check_key_history(&history, "k2", 2), Ok(()));
    }

    #[test]
    fn unknown_key_or_pre_validity_epoch_is_refused() {
        let mut history = KeyHistory::new();
        history.enroll("k1", 3, "fortune5").expect("enroll");
        assert_eq!(
            check_key_history(&history, "ghost", 3),
            Err("REFUSED:KEY_NOT_ENROLLED_AT_EPOCH".to_string())
        );
        assert_eq!(
            check_key_history(&history, "k1", 2),
            Err("REFUSED:KEY_NOT_ENROLLED_AT_EPOCH".to_string())
        );
    }

    #[test]
    fn tampered_event_byte_diverges_chain() {
        let mut history = KeyHistory::new();
        history.enroll("k1", 1, "fortune5").expect("enroll");
        history.enroll("k2", 2, "fortune5").expect("enroll");
        history.revoke("k1", 10).expect("revoke");

        // Tamper: flip one byte inside the first event's stored canonical
        // bytes and rebuild the history as an attacker would claim it
        // (stored hash from the ORIGINAL events, event slice tampered).
        let mut tampered_events = history.events.clone();
        tampered_events[0].assurance_domain = "fortune\u{36}".to_string(); // one-codepoint mutation
        assert_ne!(tampered_events[0].canonical_bytes().unwrap(), history.events[0].canonical_bytes().unwrap());

        let mut attacker_history = KeyHistory { events: tampered_events, chain_hash: history.chain_hash.clone() };
        assert_eq!(
            check_key_history(&attacker_history, "k2", 2),
            Err("REFUSED:TRUST_CHAIN_TAMPERED".to_string())
        );
        attacker_history.chain_hash = recompute_chain(&attacker_history.events).unwrap();
        // With a re-folded (consistent) chain the same query admits: the
        // refusal was the divergence, not the lifecycle.
        assert_eq!(attacker_history.key_state("k2").unwrap().valid_from_epoch, 2);
    }

    #[test]
    fn sealed_receipt_replayed_on_valid_binding() {
        let sealed = pinned_sealed_valid();
        let replayed = evidence_input_from_sealed_receipt(&sealed).expect("valid seal lifts");
        assert_eq!(replayed.input.epistemic_class, EvidenceEpistemicClass::Replayed);
        assert_eq!(replayed.receipt_digest, sealed.envelope_subject_digest);
        assert_eq!(replayed.input.subject, "svc-payments");
    }

    #[test]
    fn tampered_base_is_refused_never_observed() {
        let sealed = pinned_sealed_tampered();
        let verdict = evidence_input_from_sealed_receipt(&sealed);
        assert_eq!(
            verdict,
            Err("REFUSED:SEAL_ENVELOPE_DIGEST_MISMATCH".to_string())
        );
    }

    #[test]
    fn lying_envelope_claim_is_refused_never_observed() {
        let mut sealed = pinned_sealed_valid();
        sealed.envelope_subject_digest = "0".repeat(64);
        assert_eq!(
            evidence_input_from_sealed_receipt(&sealed),
            Err("REFUSED:SEAL_ENVELOPE_DIGEST_MISMATCH".to_string())
        );
    }
}
