//! Cross-language receipt: gymact's real emitted ndjson byte-for-byte through
//! castle's actual parser (`castle::xaas_feed::parse_xaas_ndjson`), closing the
//! gap where gymact's suite validated only an in-repo mirror of castle's law.
//!
//! Fixture provenance (real bytes, not hand-written):
//! - Producer: `/Users/sac/gymact` working tree at HEAD `b0e77081006b4820f35dbd644ede5323b4d5a03e`
//!   (2026-10-13 09:47:08 -0700, "test(pins+evidence): autofde capability pins
//!   vendored + refusal->negation producer (L-wave-F-gym)"). The converter
//!   `src/gymact/castle_feed.py` was an in-flight untracked lane file at capture
//!   time (same bytes as landed).
//! - Captured command: `cd /Users/sac/gymact && .venv/bin/python -c "..."` — a
//!   real GymAct episode on `MemoryProvider` (materialize, act x2, teardown,
//!   4 receipts), first line of `receipts_to_castle_ndjson(receipts)` written to
//!   `/tmp/gymact_castle_line.json`.
//! - Fixture sha256 (file bytes): 2e97ce9dcd3bf6edab8c0a27eb5f83b04220779206774271787e929fd3ff3f4b

use castle::xaas_feed::parse_xaas_ndjson;

/// The real emitted line, byte-for-byte from gymact's converter (see module
/// provenance above). One event ("materialize"), 3 objects
/// (episode, environment, capability).
const GYMACT_LINE: &str = r#"{"ocel:eventTypes":[{"name":"materialize"}],"ocel:events":[{"attributes":{"authority":"none","provenance":"observation_only","standing":"ALIVE","subject_ref":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3"},"id":"28c1f3755bff492cba8978cf3eaab333","relationships":[{"objectId":"7b890e27e8194060b3682ab2b387b4f8","qualifier":"episode"},{"objectId":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3","qualifier":"environment"},{"objectId":"urn:gymact:operation:materialize","qualifier":"capability"}],"time":"2026-10-13T17:03:34.825941+00:00","type":"materialize"}],"ocel:objectTypes":[{"name":"capability"},{"name":"environment"},{"name":"episode"}],"ocel:objects":[{"attributes":{},"id":"7b890e27e8194060b3682ab2b387b4f8","type":"episode"},{"attributes":{},"id":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3","type":"environment"},{"attributes":{},"id":"urn:gymact:operation:materialize","type":"capability"}]}"#;

/// Sabotage twin: flip one byte of the first relationship's objectId so it
/// dangles (no matching object) — must produce a typed refusal, not a silent
/// admit.
const GYMACT_LINE_SABOTAGED: &str = r#"{"ocel:eventTypes":[{"name":"materialize"}],"ocel:events":[{"attributes":{"authority":"none","provenance":"observation_only","standing":"ALIVE","subject_ref":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3"},"id":"28c1f3755bff492cba8978cf3eaab333","relationships":[{"objectId":"Xb890e27e8194060b3682ab2b387b4f8","qualifier":"episode"},{"objectId":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3","qualifier":"environment"},{"objectId":"urn:gymact:operation:materialize","qualifier":"capability"}],"time":"2026-10-13T17:03:34.825941+00:00","type":"materialize"}],"ocel:objectTypes":[{"name":"capability"},{"name":"environment"},{"name":"episode"}],"ocel:objects":[{"attributes":{},"id":"7b890e27e8194060b3682ab2b387b4f8","type":"episode"},{"attributes":{},"id":"urn:gymact:memory:environment:3e97b1fd3e174eceaaeb076421207df3","type":"environment"},{"attributes":{},"id":"urn:gymact:operation:materialize","type":"capability"}]}"#;

#[test]
fn real_gymact_line_admits_through_castle_parser() {
    let log = parse_xaas_ndjson(&[GYMACT_LINE]).expect("real gymact line must admit");
    assert_eq!(log.version, "2.0");
    assert_eq!(log.events.len(), 1, "one event expected");
    assert_eq!(log.objects.len(), 3, "episode+environment+capability expected");

    let event = &log.events[0];
    assert_eq!(event.id, "28c1f3755bff492cba8978cf3eaab333");
    assert_eq!(event.kind, "materialize");
    assert_eq!(event.time, "2026-10-13T17:03:34.825941+00:00");
    assert_eq!(event.object_ids.len(), 3);
    assert_eq!(
        event.attributes.get("provenance").and_then(|v| v.as_str()),
        Some("observation_only")
    );

    let mut kinds: Vec<&str> = log.objects.iter().map(|o| o.kind.as_str()).collect();
    kinds.sort_unstable();
    assert_eq!(kinds, ["capability", "environment", "episode"]);
}

#[test]
fn sabotaged_line_refuses_with_typed_violation() {
    // Differ by exactly one byte in the first relationship objectId.
    let a = GYMACT_LINE.as_bytes();
    let b = GYMACT_LINE_SABOTAGED.as_bytes();
    assert_eq!(a.len(), b.len(), "sabotage twin must stay byte-aligned");
    assert_eq!(a.iter().zip(b).filter(|(x, y)| x != y).count(), 1);

    let err = parse_xaas_ndjson(&[GYMACT_LINE_SABOTAGED])
        .expect_err("dangling relationship must refuse");
    assert!(!err.violations.is_empty());
    assert!(
        err.violations.iter().any(|v| v.path.starts_with("line 1") && v.reason.contains("does not resolve")),
        "expected a typed dangling-relationship violation, got: {err}"
    );
}

#[test]
fn gymact_checkout_drift_meta_test() {
    // BLOCKED:-skip when the producer checkout is absent; when the venv is
    // present, re-run the capture and require fresh bytes to still admit
    // (a converter/parser drift tripwire).
    let venv = std::path::Path::new("/Users/sac/gymact/.venv/bin/python");
    if !venv.exists() {
        eprintln!("BLOCKED:-skip gymact checkout or venv absent");
        return;
    }
    let script = "\
import asyncio, json
from gymact import GymAct
from gymact.providers import MemoryProvider
from gymact.models import ActuationIntent, MaterializationIntent
from gymact.castle_feed import receipts_to_castle_ndjson

async def main():
    runtime = GymAct()
    runtime.register_provider(MemoryProvider())
    receipts = []
    result = await runtime.materialize(MaterializationIntent(provider='memory', config={'initial': {'k': 1}}, idempotency_key='castle-feed-fixture'))
    receipts.append(result.receipt)
    episode_id = result.episode.episode_id
    for _ in range(2):
        result = await runtime.act(ActuationIntent(episode_id=episode_id, capability='urn:gymact:memory:capability:increment'))
        receipts.append(result.receipt)
    receipts.append(await runtime.teardown(episode_id))
    print(receipts_to_castle_ndjson(receipts).splitlines()[0])

asyncio.run(main())
";
    let out = std::process::Command::new(venv)
        .current_dir("/Users/sac/gymact")
        .args(["-c", script])
        .output()
        .expect("spawn gymact venv python");
    assert!(out.status.success(), "capture command failed: {out:?}");
    let fresh = String::from_utf8(out.stdout).expect("utf-8 ndjson");
    let fresh = fresh.trim_end_matches('\n');
    // Fresh bytes differ (fresh UUIDs/timestamps) so byte equality is not the
    // law; the law is: fresh real producer output still admits, same shape.
    let log = parse_xaas_ndjson(&[fresh]).expect("fresh gymact output must admit");
    assert_eq!(log.events.len(), 1);
    assert_eq!(log.objects.len(), 3);
    assert_eq!(log.events[0].kind, "materialize");
}
