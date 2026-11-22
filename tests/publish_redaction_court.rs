//! Chicago redaction court: real `castle::publish::anonymize` collaborator.
//! Asserts final refuse/admit state — never call counts. Refusal inspect must
//! not re-leak the private material the court just blocked.

use castle::publish::anonymize::{
    RedactionCourt, RedactionPolicy, BROKEN_TERM, DEFAULT_PRIVATE_CLIENT_NAMES, REDACTED_TOKEN,
    REFUSED_REDACTION, RULE_BEARER_TOKEN, RULE_CALENDAR_LINK, RULE_EMAIL, RULE_EXTERNAL_LINK,
    RULE_PRIVATE_CLIENT_NAME, RULE_PRIVATE_HOME_PATH, RULE_UNREDACTED_CITATION,
};
use castle::publish::brand::BrandManifest;
use castle::publish::wire::{publish_subject_post, seal_public_artifact};
use serde_json::json;

#[test]
fn redaction_refuses_private_client_names() {
    let client = "NorthwindLogisticsInc";
    let body = format!(
        "Seal the Monday prepaid rail before {client} sees the outbound carousel."
    );
    let err = RedactionCourt::with_private_names([client])
        .admit(&body)
        .expect_err("private client name must refuse");

    assert_eq!(err.code, REFUSED_REDACTION);
    assert_eq!(err.broken_term, BROKEN_TERM);
    assert!(
        err.findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_CLIENT_NAME),
        "expected PRIVATE_CLIENT_NAME, got {:?}",
        err.findings
    );

    let inspected = format!("{err:?}");
    assert!(
        !inspected.contains(client),
        "refusal Debug must not echo private client name: {inspected}"
    );
    assert!(err.findings.iter().any(|f| f.detail.contains(REDACTED_TOKEN)));
}

#[test]
fn redaction_refuses_default_active_clients() {
    assert!(!DEFAULT_PRIVATE_CLIENT_NAMES.is_empty());
    for name in DEFAULT_PRIVATE_CLIENT_NAMES {
        // Prefer the longest denylist form so short stems like "Vince" are
        // tested via the explicit "Vince Pilot" / "Courtroom.ai" rows too.
        if *name == "Vince" || *name == "Courtroom" {
            continue;
        }
        let body = format!("Board packet mentions {name} by name.");
        let err = RedactionCourt::with_default_clients()
            .admit(&body)
            .expect_err("default denylist must refuse private client name");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_PRIVATE_CLIENT_NAME));
        assert!(
            !format!("{err:?}").contains(name),
            "must not echo {name}"
        );
    }
}

#[test]
fn redaction_refuses_users_paths() {
    let path = "/Users/sac/fnox/cos/docs/gtm/YEAR-END-SOW-1PAGE.md";
    let body = format!("Draft SOW path: {path}");
    let err = RedactionCourt::with_private_names(std::iter::empty::<&str>())
        .admit(&body)
        .expect_err("/Users/ path must refuse");

    assert!(err
        .findings
        .iter()
        .any(|f| f.rule == RULE_PRIVATE_HOME_PATH));
    let inspected = format!("{err:?}");
    assert!(
        !inspected.contains("/Users/sac"),
        "refusal must not echo home path: {inspected}"
    );
    assert!(!inspected.contains("YEAR-END-SOW"));
}

#[test]
fn redaction_refuses_unredacted_citations() {
    let cite = "lib/fnox_cos/revops/ocel.ex:55";
    let body = format!("OCEL durable root check cited at {cite}.");
    let err = RedactionCourt::with_private_names(std::iter::empty::<&str>())
        .admit(&body)
        .expect_err("unredacted file:line citation must refuse");

    assert!(err
        .findings
        .iter()
        .any(|f| f.rule == RULE_UNREDACTED_CITATION));
    let inspected = format!("{err:?}");
    assert!(
        !inspected.contains(cite),
        "refusal must not echo citation: {inspected}"
    );
    assert!(
        !inspected.contains("ocel.ex"),
        "refusal must not echo citation path stem: {inspected}"
    );
}

#[test]
fn redaction_admits_clean_public_copy() {
    let body = concat!(
        "Stop the 80% Ready Mirage. Seal receipts before outbound. ",
        "CTA: Featured /drop/:lease_id on cos.fnox.io only."
    );
    assert!(RedactionCourt::with_default_clients().admit(body).is_ok());
}

#[test]
fn redaction_refuses_calendar_bearer_external() {
    let court = RedactionCourt::with_default_clients();
    assert!(court
        .admit("Book https://calendly.com/acme/intro")
        .unwrap_err()
        .findings
        .iter()
        .any(|f| f.rule == RULE_CALENDAR_LINK));
    assert!(court
        .admit("Authorization: Bearer sk-live-abcdefghijklmnopqrstuvwxyz")
        .unwrap_err()
        .findings
        .iter()
        .any(|f| f.rule == RULE_BEARER_TOKEN));
    assert!(court
        .admit("See https://evil.example/x")
        .unwrap_err()
        .findings
        .iter()
        .any(|f| f.rule == RULE_EXTERNAL_LINK));
}

#[test]
fn redaction_refuses_email_without_echo() {
    let addr = "ops@evil.example";
    let body = format!("Contact {addr} before the Featured drop.");
    let err = RedactionCourt::with_default_clients()
        .admit(&body)
        .expect_err("email must refuse at RedactionCourt");
    assert_eq!(err.code, REFUSED_REDACTION);
    assert!(
        err.findings.iter().any(|f| f.rule == RULE_EMAIL),
        "expected EMAIL, got {:?}",
        err.findings
    );
    let inspected = format!("{err:?}");
    assert!(
        !inspected.contains(addr),
        "refusal must not echo email: {inspected}"
    );
    assert!(err.findings.iter().any(|f| f.detail.contains(REDACTED_TOKEN)));
}

#[test]
fn wire_seal_refuses_home_path_without_strip() {
    let brand = BrandManifest::parse_toml(
        r#"
[brand]
id = "fourth-watch"
name = "Fourth Watch"

[redaction_court]
forbid_private_client_names = true
forbid_local_user_paths = true
forbid_bearer_tokens = true
forbid_calendar_links = true
forbid_external_links_in_post_body = true
"#,
    )
    .unwrap();
    let policy = brand.redaction_policy();
    let err = seal_public_artifact(
        json!({"body": "leak at /Users/sac/private/client.md"}),
        &policy, None)
    .expect_err("wire seal must refuse, not strip");
    let j = err.to_json();
    assert_eq!(j["code"], REFUSED_REDACTION);
    assert!(j["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_PRIVATE_HOME_PATH));
}

#[test]
fn wire_subject_post_records_alive_court_receipt() {
    let brand = BrandManifest::parse_toml(
        r#"
[brand]
id = "fourth-watch"
name = "Fourth Watch AI Cybersecurity"

[messaging.triad]
sentinel = "Pinned probes refuse silent failure before auditors arrive."
steward = "Client-owned hyperscaler RAID under your own EIN with zero rent."
shepherd = "Keyless shims wrap legacy screens so staff keep working."

[messaging.voice]
required_cta = "Drop a read-only snapshot at Featured /drop/lease-public for a board PDF."

[redaction_court]
forbid_private_client_names = true
forbid_local_user_paths = true
forbid_bearer_tokens = true
forbid_calendar_links = true
forbid_external_links_in_post_body = true
"#,
    )
    .unwrap();
    let out = publish_subject_post("t", &brand, true, None).expect("clean subject");
    assert_eq!(out["ok"], true);
    assert_eq!(out["redaction_court"]["collaborator"], "RedactionCourt");
    assert_eq!(out["redaction_court"]["standing"], "ALIVE");
    assert_eq!(out["redaction_court"]["code"], "ADMITTED");
}

#[test]
fn empty_brand_denylist_loads_defaults_on_policy() {
    let policy = RedactionPolicy {
        forbid_private_client_names: true,
        private_names: Vec::new(),
        ..RedactionPolicy::fourth_watch_defaults()
    };
    let policy = RedactionPolicy {
        private_names: Vec::new(),
        ..policy
    }
    .with_default_denylist_if_empty();
    assert!(
        !policy.private_names.is_empty(),
        "default denylist must fill"
    );
    assert!(policy
        .private_names
        .iter()
        .any(|n| n.eq_ignore_ascii_case("Courtroom.ai")));
}

#[test]
fn redaction_admits_allowlisted_intake_host() {
    let body = "First comment only: https://cos.fnox.io/drop/lease-public";
    assert!(RedactionCourt::with_default_clients().admit(body).is_ok());
}

#[test]
fn wire_seal_refuses_calendar_bearer_external_and_default_client() {
    let policy = RedactionPolicy::fourth_watch_defaults();

    let err = seal_public_artifact(
        json!({"body": "Book https://calendly.com/acme/intro"}),
        &policy, None)
    .expect_err("calendar floor on wire");
    assert!(err.to_json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_CALENDAR_LINK || f["rule"] == RULE_EXTERNAL_LINK));

    let err = seal_public_artifact(
        json!({"auth": "Authorization: Bearer sk-live-abcdefghijklmnopqrstuvwxyz"}),
        &policy, None)
    .expect_err("bearer residual refuse on wire");
    assert!(err.to_json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_BEARER_TOKEN));

    let err = seal_public_artifact(
        json!({"body": "See https://evil.example/leak"}),
        &policy, None)
    .expect_err("external floor on wire");
    assert!(err.to_json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_EXTERNAL_LINK));

    let err = seal_public_artifact(
        json!({"body": "Pilot update from Courtroom.ai this week."}),
        &policy, None)
    .expect_err("default denylist on wire");
    assert!(err.to_json()["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_PRIVATE_CLIENT_NAME));
}

#[test]
fn wire_real_brand_refuses_default_client_in_subject() {
    let brand = BrandManifest::load_path("brands/fourth-watch.brand.toml")
        .expect("castle brands twin");
    assert!(
        brand
            .redaction_policy()
            .private_names
            .iter()
            .any(|n| n.eq_ignore_ascii_case("Vince Pilot")),
        "empty brand denylist must wire DEFAULT_PRIVATE_CLIENT_NAMES"
    );
    let err = publish_subject_post("Vince Pilot breach window", &brand, true, None)
        .expect_err("default client in subject must refuse");
    let j = err.to_json();
    assert_eq!(j["code"], REFUSED_REDACTION);
    assert!(j["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["rule"] == RULE_PRIVATE_CLIENT_NAME));
}
