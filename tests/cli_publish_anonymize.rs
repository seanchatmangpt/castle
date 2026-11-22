//! Automated F-ANON-1..4 CLI witnesses for `castle publish --public-anonymize`.
//!
//! Chicago-style: real `CARGO_BIN_EXE_castle` subprocess, real
//! `brands/fourth-watch.brand.toml`, real exit codes / JSON envelopes.
//! Complements `tests/publish_redaction_court.rs` (lib RedactionCourt) and
//! `tests/cli_publish_brand.rs` (explicit-`--brand` court). No mocks, no sleeps.
//!
//! Wire law (mid-flight court): refuse-not-strip — clean subject admits with an
//! ALIVE RedactionCourt receipt; private residue under the flag exits 2.
//!
//! Spec: `docs/sjira/v26.11.20/falsifier-checklist.md` Invariant 5.

use std::path::{Path, PathBuf};
use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn brand_toml() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("brands/fourth-watch.brand.toml")
}

fn run(args: &[&str]) -> (i32, Vec<u8>, String, String) {
    let output = Command::new(castle_bin())
        .args(args)
        .output()
        .expect("spawn castle");
    let stdout_bytes = output.stdout.clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let code = output.status.code().unwrap_or(-1);
    (code, stdout_bytes, stdout, stderr)
}

fn parse_json_envelope(stdout: &str, stderr: &str) -> serde_json::Value {
    let start = stdout.find('{').unwrap_or(0);
    serde_json::from_str(&stdout[start..]).unwrap_or_else(|e| {
        panic!("stdout not JSON ({e}): {stdout}\nstderr:{stderr}")
    })
}

fn machine_hostname() -> String {
    let output = Command::new("hostname")
        .output()
        .expect("spawn hostname");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn assert_public_artifact_clean(artifact: &str) {
    // F-ANON-3 checklist: no home-path *shapes*, emails, or hostname.
    // Match `/Users/<name>` / `/home/<name>` so court meta like
    // "stripped /Users/ or /home/ path(s)" does not false-fail the envelope
    // while still catching real residual paths.
    assert!(
        !contains_home_path_shape(artifact),
        "public artifact must not leak home-path shapes: {artifact}"
    );
    let email_re = regex_lite_email(artifact);
    assert!(
        email_re.is_none(),
        "public artifact must not leak email {:?}: {artifact}",
        email_re
    );
    let host = machine_hostname();
    if !host.is_empty() && host != "localhost" {
        assert!(
            !artifact.contains(&host),
            "public artifact must not leak hostname {host}: {artifact}"
        );
    }
}

fn contains_home_path_shape(text: &str) -> bool {
    for prefix in ["/Users/", "/home/"] {
        let mut search = text;
        while let Some(idx) = search.find(prefix) {
            let after = &search[idx + prefix.len()..];
            let name_len = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(*c, '.' | '_' | '-'))
                .count();
            if name_len > 0 {
                return true;
            }
            search = &search[idx + prefix.len()..];
        }
    }
    false
}

/// Tiny ASCII email finder — no new crate; mirrors anonymize's conservative scanner.
fn regex_lite_email(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        if !chars[i].is_ascii_alphanumeric() {
            continue;
        }
        let mut j = i;
        while j < chars.len()
            && (chars[j].is_ascii_alphanumeric() || matches!(chars[j], '.' | '_' | '%' | '+' | '-'))
        {
            j += 1;
        }
        if j >= chars.len() || chars[j] != '@' {
            continue;
        }
        let mut k = j + 1;
        if k >= chars.len() || !chars[k].is_ascii_alphanumeric() {
            continue;
        }
        while k < chars.len()
            && (chars[k].is_ascii_alphanumeric() || matches!(chars[k], '.' | '-'))
        {
            k += 1;
        }
        let domain: String = chars[j + 1..k].iter().collect();
        if domain.contains('.') && domain.split('.').all(|p| !p.is_empty()) {
            return Some(chars[i..k].iter().collect());
        }
    }
    None
}

/// F-ANON-1 — `--public-anonymize` exists on the publish surface.
#[test]
fn f_anon_1_public_anonymize_flag_on_publish_help() {
    let (code_noun, _, noun_help, noun_err) = run(&["publish", "--help"]);
    assert_eq!(code_noun, 0, "publish --help must succeed: {noun_err}");
    let noun = format!("{noun_help}{noun_err}");
    assert!(
        noun.contains("--public-anonymize"),
        "publish --help must name --public-anonymize: {noun}"
    );

    let (code_post, _, post_help, post_err) = run(&["publish", "post", "--help"]);
    assert_eq!(code_post, 0, "publish post --help must succeed: {post_err}");
    let post = format!("{post_help}{post_err}");
    assert!(
        post.contains("--public-anonymize"),
        "publish post --help must list --public-anonymize: {post}"
    );
    assert!(
        post.contains("--brand"),
        "publish post --help must list --brand: {post}"
    );
}

/// F-ANON-2 — unflagged public post fails closed (exit 2 + bare refused envelope).
#[test]
fn f_anon_2_unflagged_post_refuses_no_public_anonymize() {
    let (code, _, stdout, stderr) = run(&["publish", "post", "--subject", "t"]);
    assert_eq!(
        code, 2,
        "unflagged public post must exit 2: stdout={stdout} stderr={stderr}"
    );
    let json = parse_json_envelope(&stdout, &stderr);
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert_eq!(json["code"], "REFUSED_NO_PUBLIC_ANONYMIZE");
    assert_eq!(json["broken_term"], "REFUSED_NO_AUTHORITY");
    // Bare envelope on stdout (main::failure_json promotion) — not nested under error wrappers.
    assert!(
        stdout.trim().starts_with('{'),
        "refused envelope must be bare JSON on stdout: {stdout}"
    );
}

/// F-ANON-3 — flagged run with real `brands/fourth-watch.brand.toml` admits clean.
///
/// Wire law is refuse-not-strip: clean subject exits 0 with ALIVE court receipt;
/// private residue in subject fails closed (see sibling test below).
#[test]
fn f_anon_3_flagged_post_with_fourth_watch_brand_is_public_clean() {
    let brand = brand_toml();
    assert!(
        brand.is_file(),
        "real brand twin must exist at {}",
        brand.display()
    );
    let brand_s = brand.to_string_lossy();
    let (code, raw, stdout, stderr) = run(&[
        "publish",
        "post",
        "--subject",
        "t",
        "--public-anonymize",
        "--brand",
        brand_s.as_ref(),
    ]);
    assert_eq!(
        code, 0,
        "flagged + real brand must exit 0: stdout={stdout} stderr={stderr}"
    );
    let json = parse_json_envelope(&stdout, &stderr);
    assert_eq!(json["ok"], true, "sealed artifact must admit: {json}");
    assert_eq!(json["kind"], "linkedin-post");
    assert_eq!(json["brand_id"], "fourth-watch");
    assert_eq!(json["redaction_court"]["public_anonymize"], true);
    assert_eq!(json["redaction_court"]["standing"], "ALIVE");
    assert_eq!(json["redaction_court"]["code"], "ADMITTED");
    assert_eq!(json["redaction_court"]["collaborator"], "RedactionCourt");

    let body = json["body"].as_str().unwrap_or("");
    let full = json["full_text"].as_str().unwrap_or("");
    for surface in [body, full] {
        let lower = surface.to_ascii_lowercase();
        assert!(
            !lower.contains("https://")
                && !lower.contains("http://")
                && !lower.contains("www."),
            "post body must stay link-marker free (Featured URL is first_comment only): {surface}"
        );
        assert!(
            surface.contains("cos.fnox.io") || surface.contains("Featured"),
            "body must keep Featured CTA honesty in prose (host-of-record, no scheme): {surface}"
        );
    }
    let first = json["first_comment"].as_str().unwrap_or("");
    assert!(
        first.contains("https://cos.fnox.io/drop/lease-public"),
        "Featured absolute URL must ship in first_comment via cta_link: {first}"
    );

    let artifact = String::from_utf8_lossy(&raw);
    assert_public_artifact_clean(&artifact);
}

/// F-ANON-3 effect court (refuse-not-strip): private residue under the flag
/// refuses via RedactionCourt — never strip-and-ship through CLI.
#[test]
fn f_anon_3_leaky_subject_refuses_redaction_via_real_brand() {
    let brand = brand_toml();
    assert!(brand.is_file(), "brand twin missing: {}", brand.display());
    let brand_s = brand.to_string_lossy();
    let (code, _, stdout, stderr) = run(&[
        "publish",
        "post",
        "--subject",
        "t at /Users/sac/private/client and leak@example.com",
        "--public-anonymize",
        "--brand",
        brand_s.as_ref(),
    ]);
    assert_eq!(
        code, 2,
        "leaky subject must exit 2 (refuse-not-strip): stdout={stdout} stderr={stderr}"
    );
    let json = parse_json_envelope(&stdout, &stderr);
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert_eq!(json["code"], "REFUSED_REDACTION");
    let inspected = format!("{json}");
    assert!(
        !inspected.contains("/Users/sac"),
        "refusal envelope must not echo home path: {inspected}"
    );
    assert!(
        !inspected.contains("leak@example.com"),
        "refusal envelope must not echo injected email: {inspected}"
    );
}

/// F-ANON-4 — twin flagged runs under the same brand are byte-identical.
#[test]
fn f_anon_4_twin_flagged_runs_are_byte_identical() {
    let brand = brand_toml();
    assert!(brand.is_file(), "brand twin missing: {}", brand.display());
    let brand_s = brand.to_string_lossy();
    let args = [
        "publish",
        "post",
        "--subject",
        "t",
        "--public-anonymize",
        "--brand",
        brand_s.as_ref(),
    ];

    let (code1, raw1, stdout1, stderr1) = run(&args);
    let (code2, raw2, stdout2, stderr2) = run(&args);

    assert_eq!(
        code1, 0,
        "twin run A must exit 0: stdout={stdout1} stderr={stderr1}"
    );
    assert_eq!(
        code2, 0,
        "twin run B must exit 0: stdout={stdout2} stderr={stderr2}"
    );
    assert_eq!(
        raw1, raw2,
        "F-ANON-4 twin outputs must be byte-identical\nA={stdout1}\nB={stdout2}"
    );

    let json = parse_json_envelope(&stdout1, &stderr1);
    assert_eq!(json["ok"], true);
    assert_eq!(json["brand_id"], "fourth-watch");
    assert_eq!(json["redaction_court"]["standing"], "ALIVE");
    assert_public_artifact_clean(&stdout1);
}
