//! `castle` CLI — binary entry point.
//!
//! Follows the ggen-marketplace `clap-noun-verb` convention (see
//! `~/ggen-marketplace/packs/clap-noun-verb-crate-pack` and
//! `~/ggen/examples/clap-noun-verb-cli`): a thin `main.rs` that hands off to
//! `clap_noun_verb::run()`, which auto-discovers every `#[verb]` command
//! registered by `verbs::routes` via `linkme::distributed_slice`. No
//! explicit command wiring needed here.

use std::process::ExitCode;

mod verbs;

/// Machine-readable failure for callers that shell out (XaaS effectors): the error is also
/// printed as JSON on stdout. Exit 2 = typed refusal/block (`REFUSED:`/`BLOCKED:` prefix, or an
/// already-structured refused/blocked JSON embedded in an ExecutionError string),
/// 3 = non-settled standing (`standing=` in the detail), 1 = anything else. stderr keeps the
/// legacy `ERROR: ...` line.
fn failure_json(error: &str) -> (serde_json::Value, u8) {
    let detail = error.trim();
    // Promote a handler-emitted refused/blocked JSON (e.g. publish anonymize court) out of the
    // clap-noun-verb ExecutionError wrapper into a bare envelope + exit 2 (F-ANON-2 contract).
    if let Some(start) = detail.find('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&detail[start..]) {
            let class = v.get("class").and_then(|c| c.as_str()).unwrap_or("");
            let ok_false = v.get("ok") == Some(&serde_json::Value::Bool(false));
            if ok_false && (class == "refused" || class == "blocked") {
                return (v, 2);
            }
        }
    }
    let marker = ["REFUSED:", "BLOCKED:"].iter().find_map(|p| detail.find(p).map(|i| (i, *p)));
    if let Some((i, prefix)) = marker {
        let rest = &detail[i..];
        let code = rest.split_whitespace().next().unwrap_or(rest);
        let class = prefix.trim_end_matches(':').to_lowercase();
        return (serde_json::json!({"ok": false, "class": class, "code": code, "detail": detail}), 2);
    }
    if let Some(i) = detail.find("standing=") {
        let standing = detail[i + "standing=".len()..].split_whitespace().next().unwrap_or("");
        return (serde_json::json!({"ok": false, "class": "standing", "standing": standing, "detail": detail}), 3);
    }
    (serde_json::json!({"ok": false, "class": "error", "detail": detail}), 1)
}

fn main() -> ExitCode {
    match clap_noun_verb::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let text = error.to_string();
            eprintln!("ERROR: {text}");
            let (json, code) = failure_json(&text);
            println!("{json}");
            ExitCode::from(code)
        }
    }
}
