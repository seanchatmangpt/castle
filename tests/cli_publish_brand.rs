//! Chicago CLI courts for mandatory explicit `--brand` on public publish.
//!
//! Soft-default overclaim closed: routes/docs said `--brand` was mandatory
//! while handlers once fell back to `brands/fourth-watch.brand.toml`. Real
//! binary, real exit codes, real JSON envelopes.

use std::process::Command;

fn castle_bin() -> &'static str {
    env!("CARGO_BIN_EXE_castle")
}

fn run(args: &[&str]) -> (i32, serde_json::Value, String) {
    let output = Command::new(castle_bin())
        .args(args)
        .output()
        .expect("spawn castle");
    let stdout = String::from_utf8(output.stdout).expect("stdout UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr UTF-8");
    let code = output.status.code().unwrap_or(-1);
    let parsed = if stdout.trim().is_empty() {
        serde_json::Value::Null
    } else {
        // failure_json prints refused envelope on stdout; success may also be JSON.
        let start = stdout.find('{').unwrap_or(0);
        serde_json::from_str(&stdout[start..]).unwrap_or_else(|e| {
            panic!("stdout not JSON ({e}): {stdout}\nstderr:{stderr}")
        })
    };
    (code, parsed, stderr)
}

/// F-ANON-2 preserved: omitting `--public-anonymize` still refuses anonymize
/// even when `--brand` is also absent (anonymize court fires first).
#[test]
fn publish_post_without_anonymize_refuses_anonymize_before_brand() {
    let (code, json, _err) = run(&["publish", "post", "--subject", "t"]);
    assert_eq!(code, 2, "typed refusal must exit 2: {json}");
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert!(
        json["code"]
            .as_str()
            .unwrap_or("")
            .contains("ANONYMIZE"),
        "expected anonymize refusal, got {json}"
    );
}

/// Soft `--brand` default closed: anonymize alone is not enough.
#[test]
fn publish_post_with_anonymize_without_brand_refuses_brand() {
    let (code, json, _err) = run(&[
        "publish",
        "post",
        "--subject",
        "t",
        "--public-anonymize",
    ]);
    assert_eq!(code, 2, "typed refusal must exit 2: {json}");
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert_eq!(json["code"], "REFUSED_BRAND");
    let detail = json["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("--brand") || detail.contains("no soft default"),
        "detail must name --brand / soft default: {json}"
    );
}

/// Same F-BRAND-1 court on `publish run --target` (public path, no soft default).
#[test]
fn publish_run_public_target_without_brand_refuses_brand() {
    let (code, json, _err) = run(&[
        "publish",
        "run",
        "--target",
        "linkedin-post",
        "--subject",
        "t",
        "--public-anonymize",
    ]);
    assert_eq!(code, 2, "typed refusal must exit 2: {json}");
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert_eq!(json["code"], "REFUSED_BRAND");
}

/// Route prose names mandatory `--brand` / REFUSED_BRAND (aligned with wire court).
#[test]
fn publish_help_names_mandatory_brand() {
    let output = Command::new(castle_bin())
        .args(["publish", "--help"])
        .output()
        .expect("spawn castle");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(
        help.contains("--brand") && help.contains("REFUSED_BRAND"),
        "publish --help must name mandatory --brand / REFUSED_BRAND: {help}"
    );
    let post = Command::new(castle_bin())
        .args(["publish", "post", "--help"])
        .output()
        .expect("spawn castle post --help");
    let post_help = String::from_utf8_lossy(&post.stdout);
    assert!(
        post_help.contains("--brand")
            && (post_help.contains("REFUSED_BRAND") || post_help.contains("mandatory")),
        "publish post --help must align brand mandate prose: {post_help}"
    );
}

/// Bad brand path still refuses (load court intact).
#[test]
fn publish_post_with_bad_brand_path_refuses_brand() {
    let (code, json, _err) = run(&[
        "publish",
        "post",
        "--subject",
        "t",
        "--public-anonymize",
        "--brand",
        "/no/such/castle-brand.toml",
    ]);
    assert_eq!(code, 2, "typed refusal must exit 2: {json}");
    assert_eq!(json["ok"], false);
    assert_eq!(json["class"], "refused");
    assert_eq!(json["code"], "REFUSED_BRAND");
    assert!(
        json["detail"]
            .as_str()
            .unwrap_or("")
            .contains("unreadable"),
        "bad-path refuse must cite unreadable: {json}"
    );
}

/// Happy path / F-ANON-3: explicit brand + anonymize admits a sealed post.
/// Featured CTA honesty: body prose names cos.fnox.io without https markers;
/// the Featured URL may live only in first_comment via cta_link.
#[test]
fn publish_post_with_explicit_brand_and_anonymize_succeeds() {
    let (code, json, err) = run(&[
        "publish",
        "post",
        "--subject",
        "t",
        "--public-anonymize",
        "--brand",
        "brands/fourth-watch.brand.toml",
    ]);
    assert_eq!(
        code, 0,
        "explicit brand + anonymize must succeed: {json}\nstderr:{err}"
    );
    assert_eq!(json["ok"], true);
    let s = json.to_string();
    assert!(!s.contains("/Users/"), "public artifact must not leak home paths");

    let body = json["body"].as_str().unwrap_or("");
    let full = json["full_text"].as_str().unwrap_or("");
    for surface in [body, full] {
        let lower = surface.to_lowercase();
        assert!(
            !lower.contains("https://") && !lower.contains("http://") && !lower.contains("www."),
            "post body/full_text must not carry external link markers: {surface}"
        );
        assert!(
            surface.contains("cos.fnox.io") || surface.contains("Featured"),
            "body must keep Featured CTA honesty in prose: {surface}"
        );
    }
    let first = json["first_comment"].as_str().unwrap_or("");
    assert!(
        first.contains("https://cos.fnox.io/drop/lease-public"),
        "Featured URL must surface in first_comment, not body: {first}"
    );
}
