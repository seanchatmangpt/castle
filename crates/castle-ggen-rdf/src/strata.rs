//! Strata receipt ingest: parse a ggen `ggen-receipt/v2` sync receipt, verify
//! its praxis chain hash honestly (recompute via the ggen-engine-exposed
//! `ReceiptRecord::recompute_chain_hash` — the exact emission-side formula,
//! which folds the v2 epoch payload in), verify structural linkage for
//! multi-record logs, and emit one OCEL event per sync record.
//!
//! # Typed refusals
//!
//! - `REFUSED:STRATA_SCHEMA_UNKNOWN` — missing/unknown `schema` field, or a
//!   record shape that does not deserialize as a `SyncReceipt`.
//! - `REFUSED:STRATA_CHAIN_BREAK` — chain-hash recompute mismatch, broken
//!   structural linkage, or a head-pin mismatch against an expected head.

use std::collections::BTreeMap;
use std::path::Path;

use ggen_engine::sync::SyncReceipt;
use serde_json::{json, Value};

/// Schema identities this ingest admits. Anything else is
/// `REFUSED:STRATA_SCHEMA_UNKNOWN` (fail-closed: a future schema must be
/// admitted explicitly, never silently ingested).
pub const ADMITTED_SCHEMAS: [&str; 1] = ["ggen-receipt/v2"];

/// Genesis prev-chain value ggen uses for the first record of a chain.
pub const GENESIS_PREV: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// OCEL event shape mirrored from `castle::castle::OcelEvent`
/// (`/Users/sac/castle/src/castle.rs`).
///
/// Mirrored rather than depended-upon: `castle-ggen-rdf` does not depend on
/// the `castle` crate, and adding that edge would pull the entire castle
/// runtime into an RDF-ingest crate. Field-for-field identical (`id`, `kind`,
/// `time`, `attributes`, `object_ids`).
#[derive(Debug, Clone)]
pub struct StrataOcelEvent {
    pub id: String,
    pub kind: String,
    pub time: String,
    pub attributes: BTreeMap<String, Value>,
    pub object_ids: Vec<String>,
}

/// One ingest run's result: the projected OCEL events plus the verified head
/// (chain hash of the last record) and schema identity.
#[derive(Debug, Clone)]
pub struct StrataIngest {
    pub events: Vec<StrataOcelEvent>,
    /// Chain hash (hex) of the last verified record — the ingest head.
    pub head: String,
    /// Schema identity of the ingested receipt(s).
    pub schema: String,
}

/// Ingest one receipt file (single `SyncReceipt` JSON document).
///
/// `expected_head`: optional pin — if present, the record's chain hash must
/// equal it, else `REFUSED:STRATA_CHAIN_BREAK`.
pub fn ingest_receipt_file(
    path: &Path,
    expected_head: Option<&str>,
) -> Result<StrataIngest, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("REFUSED:STRATA_READ:{path:?}:{e}"))?;
    ingest_receipt_bytes(&bytes, expected_head)
}

/// Ingest raw `SyncReceipt` JSON bytes (single document).
pub fn ingest_receipt_bytes(
    bytes: &[u8],
    expected_head: Option<&str>,
) -> Result<StrataIngest, String> {
    let doc: Value = serde_json::from_slice(bytes)
        .map_err(|e| format!("REFUSED:STRATA_PARSE:{e}"))?;
    ingest_receipt_doc(&doc, expected_head)
}

/// Ingest a parsed receipt document (single record).
pub fn ingest_receipt_doc(
    doc: &Value,
    expected_head: Option<&str>,
) -> Result<StrataIngest, String> {
    check_schema(doc)?;
    if doc.get("record").is_none() {
        return Err("REFUSED:STRATA_SCHEMA_UNKNOWN:missing `record` object".to_string());
    }
    let receipt = parse_sync_receipt(doc, 0)?;
    verify_record(&receipt)?;
    let head = receipt.record.chain_hash_hex.clone();
    if let Some(pin) = expected_head {
        if head != pin {
            return Err(format!(
                "REFUSED:STRATA_CHAIN_BREAK:head pin mismatch: pinned {pin}, ingest head {head}"
            ));
        }
    }
    let event = record_to_event(&receipt, 0);
    Ok(StrataIngest {
        events: vec![event],
        head,
        schema: receipt.record.schema.clone(),
    })
}

/// Ingest an append-only `receipt-log.jsonl` corpus (one JSON line per sync).
pub fn ingest_receipt_log(
    bytes: &[u8],
    expected_head: Option<&str>,
) -> Result<StrataIngest, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("REFUSED:STRATA_PARSE:{e}"))?;
    let mut events = Vec::new();
    let mut prev: Option<String> = None;
    let mut head = String::new();
    let mut schema: Option<String> = None;
    for (idx, line) in text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
    {
        let doc: Value = serde_json::from_str(line)
            .map_err(|e| format!("REFUSED:STRATA_PARSE:line {idx}: {e}"))?;
        check_schema(&doc)?;
        let receipt = parse_sync_receipt(&doc, idx)?;
        verify_record(&receipt)?;
        // Structural linkage: record 0 chains from genesis; record i>0 chains
        // from record i-1's stored chain hash.
        let expected_prev = prev.clone().unwrap_or_else(|| GENESIS_PREV.to_string());
        if receipt.record.prev_chain_hash_hex != expected_prev {
            return Err(format!(
                "REFUSED:STRATA_CHAIN_BREAK:line {idx}: prev {} does not link to {}",
                receipt.record.prev_chain_hash_hex, expected_prev
            ));
        }
        prev = Some(receipt.record.chain_hash_hex.clone());
        head = receipt.record.chain_hash_hex.clone();
        schema.get_or_insert_with(|| receipt.record.schema.clone());
        events.push(record_to_event(&receipt, idx));
    }
    if events.is_empty() {
        return Err("REFUSED:STRATA_PARSE:empty receipt log".to_string());
    }
    if let Some(pin) = expected_head {
        if head != pin {
            return Err(format!(
                "REFUSED:STRATA_CHAIN_BREAK:head pin mismatch: pinned {pin}, ingest head {head}"
            ));
        }
    }
    Ok(StrataIngest {
        events,
        head,
        schema: schema.unwrap_or_default(),
    })
}

/// Deserialize a receipt document into the pinned engine's [`SyncReceipt`].
///
/// The pinned ggen-engine (`rev 162e466`) parses `payload` with
/// `deny_unknown_fields`, while newer emissions (ggen@26.10.11) add
/// `payload.build_provenance`. That extension is NOT chain-bound — the chain
/// hash binds `record.payload_hash_hex` and the record's own fields, never
/// the payload extension's bytes — so stripping it here is
/// verification-neutral. Any future payload extension beyond
/// `build_provenance` still fails closed (`REFUSED:STRATA_SCHEMA_UNKNOWN`).
fn parse_sync_receipt(doc: &Value, idx: usize) -> Result<SyncReceipt, String> {
    let mut stripped = doc.clone();
    if let Some(payload) = stripped.get_mut("payload").and_then(Value::as_object_mut) {
        payload.remove("build_provenance");
    }
    serde_json::from_value(stripped)
        .map_err(|e| format!("REFUSED:STRATA_SCHEMA_UNKNOWN:line {idx}: record shape: {e}"))
}

/// Fail-closed schema gate.
fn check_schema(doc: &Value) -> Result<(), String> {
    let name = doc
        .get("record")
        .and_then(|r| r.get("schema"))
        .and_then(Value::as_str)
        .ok_or_else(|| "REFUSED:STRATA_SCHEMA_UNKNOWN:missing schema field".to_string())?;
    if ADMITTED_SCHEMAS.contains(&name) {
        Ok(())
    } else {
        Err(format!("REFUSED:STRATA_SCHEMA_UNKNOWN:{name}"))
    }
}

/// Honest chain-hash verification: recompute via the ggen-engine-exposed
/// emission-side formula (`ReceiptRecord::recompute_chain_hash`, folding in
/// the v2 epoch payload) and compare against the stored chain hash.
fn verify_record(receipt: &SyncReceipt) -> Result<(), String> {
    let recomputed = receipt
        .record
        .recompute_chain_hash()
        .map_err(|e| format!("REFUSED:STRATA_CHAIN_BREAK:recompute failed: {e}"))?;
    if hex32(&recomputed) != receipt.record.chain_hash_hex {
        return Err(format!(
            "REFUSED:STRATA_CHAIN_BREAK:stored chain hash {} does not match recompute {}",
            receipt.record.chain_hash_hex,
            hex32(&recomputed)
        ));
    }
    Ok(())
}

/// Project one verified record to an OCEL event (`kind: "ggen.sync"`).
fn record_to_event(receipt: &SyncReceipt, idx: usize) -> StrataOcelEvent {
    let mut attributes = BTreeMap::new();
    attributes.insert(
        "payload_hash".to_string(),
        json!(receipt.record.payload_hash_hex),
    );
    attributes.insert(
        "andon".to_string(),
        json!(format!("{:?}", receipt.record.andon)),
    );
    attributes.insert(
        "standing_ceiling".to_string(),
        json!(
            receipt
                .record
                .v2
                .as_ref()
                .map(|v2| format!("{:?}", v2.standing_ceiling))
                .unwrap_or_else(|| "unknown".to_string())
        ),
    );
    attributes.insert(
        "chain_hash".to_string(),
        json!(receipt.record.chain_hash_hex),
    );
    attributes.insert("ts_ns".to_string(), json!(receipt.record.ts_ns.to_string()));
    // object_ids = [pack]; the record's own `object_ids` (law:...) ride the
    // attributes-free pack identity here — one event per sync record, object
    // of interest is the pack.
    let object_ids = vec![pack_object_id(&receipt.record.object_ids)];
    StrataOcelEvent {
        id: format!("event:strata:{}:{}", idx, receipt.record.chain_hash_hex),
        kind: "ggen.sync".to_string(),
        time: receipt.record.ts_ns.to_string(),
        attributes,
        object_ids,
    }
}

/// Derive the pack object id. The pinned fixture's record carries
/// `object_ids: ["law:<payload_hash prefix>"]`; the pack identity is the
/// payload scope, so `pack:law:<prefix>` — stable, derived, never invented.
fn pack_object_id(record_object_ids: &[String]) -> String {
    let first = record_object_ids
        .first()
        .map(String::as_str)
        .unwrap_or("law:unknown");
    format!("pack:{first}")
}

/// Lowercase hex of a 32-byte hash.
fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{b:02x}"));
    }
    s
}

/// CLI entry: `strata-ingest --receipt-file <path> [--expect-head <hex>]`.
/// Returns a JSON envelope for the bin's existing typed-refusal surface.
pub fn cli_strata_ingest(args: &[String]) -> Result<Value, String> {
    let index = args
        .iter()
        .position(|a| a == "--receipt-file")
        .ok_or_else(|| "REFUSED:MISSING_ARGUMENT:--receipt-file".to_string())?;
    let path = args
        .get(index + 1)
        .map(String::as_str)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| "REFUSED:MISSING_ARGUMENT_VALUE:--receipt-file".to_string())?;
    let expected_head = args
        .iter()
        .position(|a| a == "--expect-head")
        .and_then(|i| args.get(i + 1).map(String::as_str));
    let ingest = ingest_receipt_file(Path::new(path), expected_head)?;
    Ok(json!({
        "valid": true,
        "head": ingest.head,
        "schema": ingest.schema,
        "event_count": ingest.events.len(),
        "events": ingest.events.iter().map(|e| json!({
            "id": e.id,
            "kind": e.kind,
            "time": e.time,
            "attributes": e.attributes,
            "object_ids": e.object_ids,
        })).collect::<Vec<_>>()
    }))
}
