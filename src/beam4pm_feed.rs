//! Live-feed adapter: castle `OcelLog` -> beam4pm OCEL wire envelope.
//!
//! Wire contract (beam4pm `lib/beam4pm_ocel_ingest.ex`):
//! `{"events": [{event_id, event_type, event_time, attributes, relationships: [{qualifier, object_id}]}]}`.
//! Each castle `OcelEvent.object_ids` entry maps to one relationship whose
//! qualifier is the object's `kind`, resolved from `OcelLog.objects`.

use crate::castle::{OcelEvent, OcelLog};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Render a full `OcelLog` as the beam4pm batch envelope.
pub fn beam4pm_batch(log: &OcelLog) -> Value {
    let kinds: BTreeMap<&str, &str> =
        log.objects.iter().map(|o| (o.id.as_str(), o.kind.as_str())).collect();

    json!({
        "events": log.events.iter().map(|e| beam4pm_event(e, &kinds)).collect::<Vec<_>>(),
    })
}

/// Render a single event; `object_kinds` maps object id -> kind.
pub fn beam4pm_event(event: &OcelEvent, object_kinds: &BTreeMap<&str, &str>) -> Value {
    json!({
        "event_id": event.id,
        "event_type": event.kind,
        "event_time": event.time,
        "attributes": event.attributes,
        "relationships": event.object_ids.iter().map(|oid| {
            let qualifier = object_kinds
                .get(oid.as_str())
                .map(|k| k.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            json!({"qualifier": qualifier, "object_id": oid})
        }).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::castle::{EpistemicClass, OcelObject, Receipt, ReceiptedOcelLog};

    #[test]
    fn renders_receipted_log_into_beam4pm_envelope() {
        let receipt = Receipt {
            algorithm: "BLAKE3-256",
            artifact_digest: "a".repeat(64),
            receipt_digest: "b".repeat(64),
            epistemic_class: EpistemicClass::Observed,
            subject: "system:fixture".to_string(),
            parent_digests: vec!["c".repeat(64)],
            origin_key_id: "key-1".to_string(),
            origin_signature: "sig".to_string(),
        };
        let log = OcelLog {
            version: "2.0",
            objects: vec![
                OcelObject { id: "sys:1".to_string(), kind: "System".to_string() },
                OcelObject { id: "docker:web:abc".to_string(), kind: "Container".to_string() },
            ],
            events: vec![OcelEvent {
                id: "event:1:observe".to_string(),
                kind: "observe".to_string(),
                time: "1728800000000".to_string(),
                attributes: BTreeMap::from([
                    ("epistemic_class".to_string(), json!("OBSERVED")),
                    ("construct_digest".to_string(), json!("d".repeat(64))),
                ]),
                object_ids: vec!["docker:web:abc".to_string(), "sys:1".to_string()],
            }],
        };
        let receipted = ReceiptedOcelLog { log, construct_digest: "d".repeat(64), receipt };

        let batch = beam4pm_batch(&receipted.log);
        let events = batch.get("events").and_then(Value::as_array).expect("events array");
        assert_eq!(events.len(), 1);

        let event = &events[0];
        for key in ["event_id", "event_type", "event_time", "attributes", "relationships"] {
            assert!(event.get(key).is_some(), "missing {key}");
        }
        assert_eq!(event["event_id"], "event:1:observe");
        assert_eq!(event["event_type"], "observe");
        assert_eq!(event["attributes"]["epistemic_class"], "OBSERVED");

        let rels = event["relationships"].as_array().expect("relationships array");
        assert!(!rels.is_empty());
        for rel in rels {
            assert!(rel["qualifier"].is_string(), "qualifier must be a string");
            assert!(rel["object_id"].is_string(), "object_id must be a string");
        }
        let by_id: BTreeMap<&str, &str> = rels
            .iter()
            .map(|r| (r["object_id"].as_str().unwrap(), r["qualifier"].as_str().unwrap()))
            .collect();
        assert_eq!(by_id.get("sys:1"), Some(&"System"));
        assert_eq!(by_id.get("docker:web:abc"), Some(&"Container"));
    }
}
