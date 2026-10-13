//! Live-feed adapter: xaas OCEL ndjson egress -> castle `OcelLog`.
//!
//! Wire contract (xaas `Xaas.Telemetry.OcelNdjson` aggregate form; the
//! same `:xaas_ocel_prefixed` dialect beam4pm's
//! `lib/beam4pm_ocel_session_facts.ex` reads): each non-blank ndjson line
//! is a complete OCEL 2.0 JSON document carrying ALL FOUR top-level keys
//! as arrays -- `ocel:objectTypes`, `ocel:eventTypes`, `ocel:events`,
//! `ocel:objects`. Events carry plain spec keys
//! `{id, type, time, attributes: {..}, relationships?: [{objectId,
//! qualifier}]}`; objects carry `{id, type, attributes: {..},
//! relationships?: [...]}`.
//!
//! Court law mirrors the upstream authority
//! (xaas `Xaas.Ultracode.Ocel.Validator`), applied per line then over
//! the aggregate: closed vocabulary at every structural level; unique
//! non-empty string ids; types declared; ISO8601 event times at zero UTC
//! offset; relationships exactly `{objectId, qualifier}` resolving to a
//! declared object. Fail-closed, typed refusals, never silent.
//!
//! The pre-reshape legacy flat-event shape (`ocel:eid` + `ocel:activity`)
//! is named exactly in its refusal -- it is not this dialect.

use crate::castle::{OcelEvent, OcelLog, OcelObject};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// One typed refusal: exact ndjson path + structural reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub path: String,
    pub reason: String,
}

/// Fail-closed refusal: one or more typed violations; assembly is all-or-nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XaasFeedRefused {
    pub violations: Vec<Violation>,
}

impl std::fmt::Display for XaasFeedRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in &self.violations {
            writeln!(f, "{}: {}", v.path, v.reason)?;
        }
        Ok(())
    }
}

impl std::error::Error for XaasFeedRefused {}

/// Top-level keys every ndjson line (an OCEL 2.0 document) must carry as arrays.
const LINE_KEYS: [&str; 4] = [
    "ocel:objectTypes",
    "ocel:eventTypes",
    "ocel:events",
    "ocel:objects",
];

/// Closed vocabulary for events (validator event keys).
const EVENT_KEYS: [&str; 5] = ["id", "type", "time", "attributes", "relationships"];
/// Closed vocabulary for objects (validator object keys -- no "time").
const OBJECT_KEYS: [&str; 4] = ["id", "type", "attributes", "relationships"];

/// An assembled, per-line-valid document awaiting merge.
struct LineDoc {
    events: Vec<OcelEvent>,
    objects: Vec<OcelObject>,
}

/// Assemble xaas OCEL ndjson lines into one castle `OcelLog`.
///
/// All-or-nothing: any malformed line refuses the whole input with typed
/// violations naming exact line paths (`line <n>`, 1-based, blanks never
/// counted). Events concatenate in file order; objects deduplicate by id
/// (first occurrence wins); declared types union across lines, and every
/// type actually used is added as a declaration -- assembly never
/// manufactures a used-but-undeclared type.
///
/// # Errors
///
/// Refuses (`XaasFeedRefused`, all-or-nothing) with typed violations naming
/// exact line paths when any line is malformed JSON, not an object, the
/// legacy flat-event shape, missing top-level keys, a malformed type
/// declaration, an undeclared type, a duplicate event id, a non-UTC or
/// unparseable event time, non-object attributes, or a dangling/non-conforming
/// relationship.
pub fn parse_xaas_ndjson(lines: &[&str]) -> Result<OcelLog, XaasFeedRefused> {
    let mut events: Vec<OcelEvent> = Vec::new();
    let mut objects: Vec<OcelObject> = Vec::new();
    let mut object_ids: BTreeSet<String> = BTreeSet::new();
    let mut event_ids: BTreeSet<String> = BTreeSet::new();
    let mut violations: Vec<Violation> = Vec::new();

    for (index, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let path = format!("line {}", index + 1);
        match ingest_line(line, &path, &object_ids, &event_ids) {
            Ok(doc) => merge_doc(doc, &mut events, &mut objects, &mut object_ids, &mut event_ids),
            Err(mut v) => violations.append(&mut v),
        }
    }

    if !violations.is_empty() {
        return Err(XaasFeedRefused { violations });
    }

    Ok(OcelLog {
        version: "2.0",
        objects,
        events,
    })
}

/// Parse file contents (split on newlines) into an `OcelLog`.
///
/// # Errors
///
/// Same refusal law as [`parse_xaas_ndjson`].
pub fn parse_xaas_ndjson_file_contents(contents: &str) -> Result<OcelLog, XaasFeedRefused> {
    let lines: Vec<&str> = contents.split('\n').collect();
    parse_xaas_ndjson(&lines)
}

fn ingest_line(
    line: &str,
    path: &str,
    prior_object_ids: &BTreeSet<String>,
    prior_event_ids: &BTreeSet<String>,
) -> Result<LineDoc, Vec<Violation>> {
    let doc: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Err(vec![Violation {
                path: path.to_string(),
                reason: format!("malformed JSON: {e}"),
            }]);
        }
    };

    let Some(obj) = doc.as_object() else {
        return Err(vec![Violation {
            path: path.to_string(),
            reason: "not an OCEL 2.0 document: top level must be a JSON object".to_string(),
        }]);
    };

    // Permanent tripwire: the pre-reshape legacy flat event shape, named
    // exactly, BEFORE the generic missing-key law.
    if obj.contains_key("ocel:eid") && obj.contains_key("ocel:activity") {
        return Err(vec![Violation {
            path: path.to_string(),
            reason: "not an OCEL 2.0 document: legacy flat event shape (ocel:eid/ocel:activity) \
                     emitted before the OCEL v2 reshape -- regenerate this log"
                .to_string(),
        }]);
    }

    // All four top-level keys, each an array.
    let mut violations: Vec<Violation> = Vec::new();
    for key in LINE_KEYS {
        if !obj.get(key).is_some_and(Value::is_array) {
            violations.push(Violation {
                path: format!("{path}.{key}"),
                reason: format!(
                    "must be an array, got: {}",
                    obj.get(key).map_or_else(|| "absent".to_string(), describe)
                ),
            });
        }
    }
    if !violations.is_empty() {
        return Err(violations);
    }

    // Declaration element law: non-empty string, or object with exactly one
    // key "name" whose value is a non-empty string.
    for key in ["ocel:eventTypes", "ocel:objectTypes"] {
        let decls = obj[key].as_array().expect("checked array above");
        for (i, decl) in decls.iter().enumerate() {
            if !well_formed_declaration(decl) {
                violations.push(Violation {
                    path: format!("{path}.{key}[{i}]"),
                    reason: format!(
                        "malformed type declaration {decl}: must be a non-empty string \
                         or an object with exactly one key \"name\" whose value is a non-empty string"
                    ),
                });
            }
        }
    }
    if !violations.is_empty() {
        return Err(violations);
    }

    let event_type_decls = declared_names(obj.get("ocel:eventTypes").expect("checked"));
    let object_type_decls = declared_names(obj.get("ocel:objectTypes").expect("checked"));

    // Relationship resolution: objects of this line (processed first) plus
    // every object assembled from prior lines. Objects are deduplicated
    // first-wins upstream (every emitter line re-states class-level
    // objects), so they carry no uniqueness refusal here; events do.
    let mut line_object_ids: BTreeSet<String> = prior_object_ids.clone();
    let mut line_objects: Vec<OcelObject> = Vec::new();
    let mut line_events: Vec<OcelEvent> = Vec::new();
    let mut line_event_ids: BTreeSet<String> = prior_event_ids.clone();

    for (i, item) in obj["ocel:objects"].as_array().expect("checked").iter().enumerate() {
        let base = format!("{path}.ocel:objects[{i}]");
        let mut no_dup_registry: BTreeSet<String> = BTreeSet::new();
        match check_entity(item, &base, false, &object_type_decls, &line_object_ids, &mut no_dup_registry) {
            Ok(OcelEntity::Object(object)) => {
                line_object_ids.insert(object.id.clone());
                line_objects.push(object);
            }
            Ok(OcelEntity::Event(_)) => unreachable!("asked for an object"),
            Err(mut v) => violations.append(&mut v),
        }
    }

    for (i, item) in obj["ocel:events"].as_array().expect("checked").iter().enumerate() {
        let base = format!("{path}.ocel:events[{i}]");
        match check_entity(item, &base, true, &event_type_decls, &line_object_ids, &mut line_event_ids) {
            Ok(OcelEntity::Event(event)) => line_events.push(event),
            Ok(OcelEntity::Object(_)) => unreachable!("asked for an event"),
            Err(mut v) => violations.append(&mut v),
        }
    }

    if !violations.is_empty() {
        return Err(violations);
    }

    Ok(LineDoc { events: line_events, objects: line_objects })
}

fn merge_doc(
    doc: LineDoc,
    events: &mut Vec<OcelEvent>,
    objects: &mut Vec<OcelObject>,
    object_ids: &mut BTreeSet<String>,
    event_ids: &mut BTreeSet<String>,
) {
    for object in doc.objects {
        if object_ids.insert(object.id.clone()) {
            objects.push(object);
        }
    }
    for event in doc.events {
        event_ids.insert(event.id.clone());
        events.push(event);
    }
}

/// Object/event law: closed vocabulary, required keys, id, declared type,
/// ISO8601-zero-UTC event time, attributes object, resolving relationships.
/// Returns the mapped castle entity on success.
fn check_entity(
    item: &Value,
    base: &str,
    is_event: bool,
    type_decls: &BTreeSet<String>,
    object_ids: &BTreeSet<String>,
    seen_ids: &mut BTreeSet<String>,
) -> Result<OcelEntity, Vec<Violation>> {
    let Some(entity) = item.as_object() else {
        return Err(vec![Violation {
            path: base.to_string(),
            reason: format!("must be a JSON object, got: {item}"),
        }]);
    };

    let allowed: &[&str] = if is_event { &EVENT_KEYS } else { &OBJECT_KEYS };
    let mut violations: Vec<Violation> = Vec::new();

    for key in entity.keys() {
        if !allowed.contains(&key.as_str()) {
            violations.push(Violation {
                path: format!("{base}.{key}"),
                reason: format!("unexpected key '{key}' (closed vocabulary)"),
            });
        }
    }
    for key in ["id", "type", "attributes"] {
        if !entity.contains_key(key) {
            violations.push(Violation {
                path: format!("{base}.{key}"),
                reason: "missing required key".to_string(),
            });
        }
    }
    if is_event && !entity.contains_key("time") {
        violations.push(Violation {
            path: format!("{base}.time"),
            reason: "missing required key".to_string(),
        });
    }
    if !violations.is_empty() {
        return Err(violations);
    }

    // id law: unique, non-empty string (per collection).
    let id = match entity["id"].as_str() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return Err(vec![Violation {
                path: format!("{base}.id"),
                reason: "must be a non-empty string".to_string(),
            }]);
        }
    };
    if !seen_ids.insert(id.clone()) {
        return Err(vec![Violation {
            path: format!("{base}.id"),
            reason: format!("duplicate id '{id}'"),
        }]);
    }

    // type law: declared in the matching declaration key.
    let kind = match entity["type"].as_str() {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => {
            return Err(vec![Violation {
                path: format!("{base}.type"),
                reason: "must be a non-empty string".to_string(),
            }]);
        }
    };
    if !type_decls.contains(&kind) {
        violations.push(Violation {
            path: format!("{base}.type"),
            reason: format!("'{kind}' not declared in the matching type-declaration key"),
        });
    }

    // time law (events only; an object carrying "time" hits closed vocabulary).
    let time = if is_event {
        match entity["time"].as_str() {
            Some(t) => match check_utc_time(t) {
                Ok(()) => t.to_string(),
                Err(reason) => {
                    violations.push(Violation {
                        path: format!("{base}.time"),
                        reason,
                    });
                    String::new()
                }
            },
            None => {
                violations.push(Violation {
                    path: format!("{base}.time"),
                    reason: "must be an ISO8601 string with zero UTC offset".to_string(),
                });
                String::new()
            }
        }
    } else {
        String::new()
    };

    // attributes law: a JSON object (string-keyed by JSON construction).
    if !entity["attributes"].is_object() {
        violations.push(Violation {
            path: format!("{base}.attributes"),
            reason: format!(
                "must be an object, got: {}",
                describe(&entity["attributes"])
            ),
        });
    }

    // relationships law: optional array of exactly {objectId, qualifier}
    // resolving to a declared object id.
    let mut object_refs: Vec<String> = Vec::new();
    if let Some(rels) = entity.get("relationships") {
        match rels.as_array() {
            Some(arr) => {
                for (i, rel) in arr.iter().enumerate() {
                    if let Err(reason) = check_relationship(rel, object_ids) {
                        violations.push(Violation {
                            path: format!("{base}.relationships[{i}]"),
                            reason,
                        });
                    }
                }
                for rel in arr.iter().filter_map(|r| r.get("objectId")).filter_map(|v| v.as_str()) {
                    object_refs.push(rel.to_string());
                }
            }
            None => violations.push(Violation {
                path: format!("{base}.relationships"),
                reason: format!("must be an array, got: {rels}"),
            }),
        }
    }

    if !violations.is_empty() {
        return Err(violations);
    }

    let attributes: BTreeMap<String, Value> = entity["attributes"]
        .as_object()
        .expect("checked")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok(if is_event {
        OcelEntity::Event(OcelEvent {
            id,
            kind,
            time,
            attributes,
            object_ids: object_refs,
        })
    } else {
        OcelEntity::Object(OcelObject { id, kind })
    })
}

enum OcelEntity {
    Event(OcelEvent),
    Object(OcelObject),
}

fn check_relationship(rel: &Value, object_ids: &BTreeSet<String>) -> Result<(), String> {
    let obj = match rel.as_object() {
        Some(o) => o,
        None => return Err(format!("must be an object, got: {rel}")),
    };
    let extra: Vec<&String> = obj
        .keys()
        .filter(|k| *k != "objectId" && *k != "qualifier")
        .collect();
    if !extra.is_empty() {
        return Err(format!(
            "unexpected key '{}' (closed vocabulary: exactly {{objectId, qualifier}})",
            extra[0]
        ));
    }
    let object_id = obj.get("objectId").and_then(Value::as_str);
    let qualifier = obj.get("qualifier").and_then(Value::as_str);
    match (object_id, qualifier) {
        (Some(oid), Some(q)) => {
            if oid.is_empty() {
                return Err("'objectId' must be a non-empty string".to_string());
            }
            if !object_ids.contains(oid) {
                return Err(format!(
                    "objectId '{oid}' does not resolve to any object in ocel:objects"
                ));
            }
            let _ = q;
            Ok(())
        }
        _ => Err("must be exactly {objectId, qualifier}, both strings".to_string()),
    }
}

/// ISO8601 with zero UTC offset: `YYYY-MM-DDTHH:MM:SS(.fff...)?Z`
/// (also accepts the `+00:00`/`-00:00` zero-offset forms, matching
/// `DateTime.from_iso8601/1` acceptance of zero offsets).
fn check_utc_time(t: &str) -> Result<(), String> {
    let bytes = t.as_bytes();
    let fail = || format!("'{t}' is not a parseable ISO8601 time with zero UTC offset");

    if bytes.len() < 20 {
        return Err(fail());
    }
    let digits = |slice: &[u8]| -> bool { slice.iter().all(u8::is_ascii_digit) };
    if !digits(&bytes[0..4])
        || bytes[4] != b'-'
        || !digits(&bytes[5..7])
        || bytes[7] != b'-'
        || !digits(&bytes[8..10])
    {
        return Err(fail());
    }
    let sep = bytes[10];
    // `DateTime.from_iso8601/1` requires the "T" delimiter.
    if sep != b'T' {
        return Err(fail());
    }
    if !digits(&bytes[11..13])
        || bytes[13] != b':'
        || !digits(&bytes[14..16])
        || bytes[16] != b':'
        || !digits(&bytes[17..19])
    {
        return Err(fail());
    }
    let month: u32 = t[5..7].parse().map_err(|_| fail())?;
    let day: u32 = t[8..10].parse().map_err(|_| fail())?;
    let hour: u32 = t[11..13].parse().map_err(|_| fail())?;
    let minute: u32 = t[14..16].parse().map_err(|_| fail())?;
    let second: u32 = t[17..19].parse().map_err(|_| fail())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(fail());
    }
    if hour > 23 || minute > 59 || second > 60 {
        return Err(fail());
    }

    let mut rest = &t[19..];
    if let Some(frac) = rest.strip_prefix('.') {
        match frac.find(|c: char| !c.is_ascii_digit()) {
            Some(end) if end > 0 => rest = &frac[end..],
            Some(_) | None => return Err(fail()),
        }
    }
    matches!(rest, "Z" | "+00:00" | "-00:00" | "+0000" | "-0000")
        .then_some(())
        .ok_or_else(fail)
}

fn describe(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{s}\""),
        Value::Array(_) => "an array".to_string(),
        Value::Object(_) => "an object".to_string(),
    }
}

fn well_formed_declaration(v: &Value) -> bool {
    match v {
        Value::String(s) => !s.is_empty(),
        Value::Object(o) => {
            o.len() == 1
                && o.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|n| !n.is_empty())
        }
        _ => false,
    }
}

fn declared_names(v: &Value) -> BTreeSet<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|d| match d {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(_) => d.get("name").and_then(Value::as_str).map(String::from),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}


#[cfg(test)]
mod tests {
    use super::*;

    // -- fixture ----------------------------------------------------------
    // Provenance: no on-disk /Users/sac/xaas *.jsonocel matches this
    // dialect (the shipped .jsonocel files are pretty-printed single
    // documents in the zcode plain-key dialect, not per-line
    // `ocel:`-prefixed ndjson). Shape authority is the real emitter/court
    // surface -- xaas `Xaas.Telemetry.OcelNdjson` +
    // `Xaas.Ultracode.Ocel.Validator` law and its test corpus
    // (test/xaas/telemetry/ocel_ndjson_test.exs `conforming_line/2`) --
    // with concrete values drawn from the real run log
    // /Users/sac/xaas/receipts/ocel/v2/swarm/runtime/
    // 2026-09-28-xaas-execution-federation-r13.jsonocel (run_id, repo,
    // base/tree/commit SHAs).
    //
    // Line 1: three events bound to one runtime_factory object.
    // Line 2: the class-level object re-stated (dedup keeps first) plus a
    // new event on line 2 only.
    const LINE_1: &str = concat!(
        r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#,
        r#""ocel:eventTypes":[{"name":"bind"},{"name":"construct"},{"name":"durable"},{"name":"frontier_advance"}],"#,
        r#""ocel:events":["#,
        r#"{"id":"bind:r13","type":"bind","time":"2026-09-28T13:27:37Z","attributes":{"repo":"seanchatmangpt/xaas","base":"37f39e7ce9310563a50523bc788cf7de5c7d095c","family":"research-execution-federation"},"relationships":[{"objectId":"run:r13","qualifier":"runtime_factory"}]},"#,
        r#"{"id":"construct:r13","type":"construct","time":"2026-09-28T13:27:38Z","attributes":{"qualifying_files":56,"tree":"81deaf8df5177cda282c190d14fe898837220eeb"},"relationships":[{"objectId":"run:r13","qualifier":"runtime_factory"}]},"#,
        r#"{"id":"durable:r13","type":"durable","time":"2026-09-28T13:27:39Z","attributes":{"commit":"9018394579b549f9471e29e23ae1860974e1cc3e","branch":"runtime/v26.9.28-execution-federation-r13"},"relationships":[{"objectId":"run:r13","qualifier":"runtime_factory"}]}"#,
        r#"],"#,
        r#""ocel:objects":[{"id":"run:r13","type":"runtime_factory","attributes":{"run_id":"2026-09-28-xaas-execution-federation-r13"},"relationships":[]}"#,
        "]}"
    );

    const LINE_2: &str = concat!(
        r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#,
        r#""ocel:eventTypes":[{"name":"bind"},{"name":"construct"},{"name":"durable"},{"name":"frontier_advance"}],"#,
        r#""ocel:events":["#,
        r#"{"id":"frontier:r13","type":"frontier_advance","time":"2026-09-28T13:27:40Z","attributes":{"frontier":"w-wave"},"relationships":[{"objectId":"run:r13","qualifier":"runtime_factory"}]}"#,
        r#"],"#,
        r#""ocel:objects":[{"id":"run:r13","type":"runtime_factory","attributes":{"run_id":"2026-09-28-xaas-execution-federation-r13"},"relationships":[]}"#,
        "]}"
    );

    fn fixture_lines() -> Vec<&'static str> {
        vec![LINE_1, LINE_2]
    }

    #[test]
    fn ingests_real_shaped_ndjson_into_castle_ocel_log() {
        let log = parse_xaas_ndjson(&fixture_lines()).expect("fixture must ingest");

        assert_eq!(log.version, "2.0");
        // Object dedup: first occurrence wins, exactly one run object.
        assert_eq!(log.objects.len(), 1);
        assert_eq!(log.objects[0].id, "run:r13");
        assert_eq!(log.objects[0].kind, "runtime_factory");

        // Events concatenate in file order across lines.
        assert_eq!(log.events.len(), 4);
        assert_eq!(
            log.events.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            vec!["bind:r13", "construct:r13", "durable:r13", "frontier:r13"]
        );

        let bind = &log.events[0];
        assert_eq!(bind.kind, "bind");
        assert_eq!(bind.time, "2026-09-28T13:27:37Z");
        assert_eq!(bind.attributes["repo"], "seanchatmangpt/xaas");
        assert_eq!(bind.object_ids, vec!["run:r13".to_string()]);

        let construct = &log.events[1];
        assert_eq!(construct.kind, "construct");
        assert_eq!(construct.attributes["qualifying_files"], serde_json::json!(56));
        assert_eq!(construct.attributes["tree"], "81deaf8df5177cda282c190d14fe898837220eeb");
    }

    #[test]
    fn blank_lines_are_skipped_and_line_numbers_still_count_content() {
        let lines = vec!["", LINE_1, "   ", LINE_2];
        let log = parse_xaas_ndjson(&lines).expect("blanks must not refuse");
        assert_eq!(log.events.len(), 4);

        // A malformed line at physical index 3 (line number 4).
        let lines = vec![LINE_1, "", "{not json", LINE_2];
        let err = parse_xaas_ndjson(&lines).expect_err("malformed JSON must refuse");
        assert_eq!(err.violations.len(), 1);
        assert_eq!(err.violations[0].path, "line 3");
        assert!(err.violations[0].reason.starts_with("malformed JSON:"));
    }

    #[test]
    fn legacy_flat_event_shape_is_refused_and_named() {
        let legacy = r#"{"ocel:eid":"e1","ocel:activity":"poke","ocel:vmap":{},"ocel:events":[],"ocel:objects":[],"ocel:objectTypes":[],"ocel:eventTypes":[]}"#;
        let lines = vec![LINE_1, legacy];
        let err = parse_xaas_ndjson(&lines).expect_err("legacy shape must refuse");
        assert_eq!(err.violations.len(), 1);
        assert_eq!(err.violations[0].path, "line 2");
        assert!(
            err.violations[0]
                .reason
                .contains("legacy flat event shape (ocel:eid/ocel:activity)"),
            "refusal must name the legacy shape exactly, got: {}",
            err.violations[0].reason
        );
    }

    #[test]
    fn missing_top_level_keys_refuse_with_exact_paths() {
        let half = r#"{"ocel:events":[],"ocel:objects":[]}"#;
        let err = parse_xaas_ndjson(&[half]).expect_err("missing keys must refuse");
        let mut paths: Vec<&str> = err.violations.iter().map(|v| v.path.as_str()).collect();
        paths.sort_unstable();
        assert_eq!(
            paths,
            vec![
                "line 1.ocel:eventTypes",
                "line 1.ocel:objectTypes",
            ]
        );
    }

    #[test]
    fn undeclared_type_and_dangling_relationship_refuse_with_exact_paths() {
        let bad = concat!(
            r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#,
            r#""ocel:eventTypes":[{"name":"bind"}],"#,
            r#""ocel:events":[{"id":"e1","type":"construct","time":"2026-09-28T13:27:37Z","attributes":{},"relationships":[{"objectId":"run:r13","qualifier":"runtime_factory"}]}],"#,
            r#""ocel:objects":[{"id":"run:r13","type":"runtime_factory","attributes":{}}]}"#
        );
        let err = parse_xaas_ndjson(&[bad]).expect_err("undeclared type must refuse");
        assert!(
            err.violations
                .iter()
                .any(|v| v.path == "line 1.ocel:events[0].type"
                    && v.reason.contains("'construct' not declared")),
            "got: {err}"
        );

        let dangling = concat!(
            r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#,
            r#""ocel:eventTypes":[{"name":"bind"}],"#,
            r#""ocel:events":[{"id":"e1","type":"bind","time":"2026-09-28T13:27:37Z","attributes":{},"relationships":[{"objectId":"nope","qualifier":"runtime_factory"}]}],"#,
            r#""ocel:objects":[{"id":"run:r13","type":"runtime_covering","attributes":{}}]}"#
        );
        // Note: "runtime_covering" is itself undeclared -> also refused; the
        // dangling-relationship violation must be present too.
        let err = parse_xaas_ndjson(&[dangling]).expect_err("dangling relationship must refuse");
        assert!(
            err.violations
                .iter()
                .any(|v| v.reason.contains("objectId 'nope' does not resolve")),
            "got: {err}"
        );
    }

    #[test]
    fn non_utc_and_malformed_times_refuse() {
        let mk = |time: &str| {
            let mut s = String::from(r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#);
            s.push_str(r#""ocel:eventTypes":[{"name":"bind"}],"#);
            s.push_str(r#""ocel:events":[{"id":"e1","type":"bind","time":""#);
            s.push_str(time);
            s.push_str(r#"","attributes":{}}],"#);
            s.push_str(r#""ocel:objects":[{"id":"run:r13","type":"runtime_factory","attributes":{}}]}"#);
            s
        };
        for t in ["2026-09-28T13:27:37+02:00", "2026-09-28 13:27:37Z", "not-a-time", "2026-13-01T00:00:00Z"] {
            let err = parse_xaas_ndjson(&[mk(t).as_str()]).expect_err("must refuse");
            assert!(
                err.violations
                    .iter()
                    .any(|v| v.path.ends_with(".time")),
                "time={t} got: {err}"
            );
        }
        // Zero-offset alternates are accepted.
        assert!(parse_xaas_ndjson(&[mk("2026-09-28T13:27:37.123456Z").as_str()]).is_ok());
        assert!(parse_xaas_ndjson(&[mk("2026-09-28T13:27:37+00:00").as_str()]).is_ok());
    }

    #[test]
    fn duplicate_event_ids_across_lines_are_refused() {
        // Duplicate the whole first line: same event ids again.
        let lines = vec![LINE_1, LINE_1];
        let err = parse_xaas_ndjson(&lines).expect_err("duplicate ids must refuse");
        assert!(
            err.violations
                .iter()
                .any(|v| v.path == "line 2.ocel:events[0].id" && v.reason.contains("duplicate id 'bind:r13'")),
            "got: {err}"
        );
    }

    #[test]
    fn object_carrying_time_key_hits_closed_vocabulary() {
        let bad = concat!(
            r#"{"ocel:objectTypes":[{"name":"runtime_factory"}],"#,
            r#""ocel:eventTypes":[{"name":"bind"}],"#,
            r#""ocel:events":[],"#,
            r#""ocel:objects":[{"id":"run:r13","type":"runtime_factory","time":"2026-09-28T13:27:37Z","attributes":{}}]}"#
        );
        let err = parse_xaas_ndjson(&[bad]).expect_err("extra key must refuse");
        assert!(
            err.violations
                .iter()
                .any(|v| v.path == "line 1.ocel:objects[0].time"
                    && v.reason.contains("closed vocabulary")),
            "got: {err}"
        );
    }

    // -- sabotage twin ----------------------------------------------------
    // One flipped field must be refused: the court detects a mutated row,
    // never silently ingests it. Mirror of tests/mfact_witnesses_fixture.rs.

    #[test]
    fn sabotaged_row_is_refused_not_silently_ingested() {
        let sabotaged = LINE_1.replace(
            r#""id":"durable:r13","type":"durable""#,
            r#""id":"durable:r13","type":"undurable""#,
        );
        assert_ne!(sabotaged, LINE_1, "sabotage must mutate the row");
        let err = parse_xaas_ndjson(&[sabotaged.as_str()]).expect_err("sabotaged row must refuse");
        assert!(
            err.violations
                .iter()
                .any(|v| v.path == "line 1.ocel:events[2].type"
                    && v.reason.contains("'undurable' not declared")),
            "got: {err}"
        );
    }

    // -- fixture self-integrity --------------------------------------------

    #[test]
    fn fixture_lines_are_valid_json_documents() {
        for line in fixture_lines() {
            let v: Value = serde_json::from_str(line).expect("fixture line must be valid JSON");
            for key in LINE_KEYS {
                assert!(v.get(key).is_some_and(Value::is_array), "missing {key}");
            }
        }
    }

    #[test]
    fn refusal_is_display_and_error() {
        let err = parse_xaas_ndjson(&["{not json"]).expect_err("must refuse");
        let text = err.to_string();
        assert!(text.contains("line 1: malformed JSON:"), "got: {text}");
        fn assert_error<T: std::error::Error>(_: &T) {}
        assert_error(&err);
    }
}
