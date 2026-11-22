//! Chicago court: ggen `AotSealReceipt` / layout digest-bind into CASTLE mint.
//!
//! Arrange → Act → Assert on real `castle::publish::{digest_bind,wire,brand}`
//! collaborators. No mock digests that skip hex-shape law.
//!
//! Standing honesty:
//! - Admit / bind API tests below are expected **green**.
//! - `mint_receipt_must_bind_admitted_ggen_layout_digest` requires production
//!   `publish_subject_post` to thread a pin into `seal_public_artifact` →
//!   `bind_into_mint`. Wire bind **WIRED** ≠ Foundry / zero-HITL crown ALIVE.

use castle::publish::digest_bind::{
    admit_ggen_layout_digest, assert_mint_binds_digest, bind_into_mint, GgenLayoutDigestPin,
    MINT_KEY_DIGEST_BIND, MINT_KEY_GGEN_LAYOUT_DIGEST, REFUSED_GGEN_DIGEST,
};
use castle::publish::brand::BrandManifest;
use castle::publish::wire::{publish_subject_post, seal_public_artifact};
use castle::publish::anonymize::RedactionPolicy;
use serde_json::json;

/// Fixture digest shaped like ggen `AotSealReceipt.digest` (64 ASCII hex).
/// Not a live Taffy recompute — castle admits identity, does not re-seal layout.
fn fixture_aot_digest() -> String {
    // Distinct non-trivial hex (not all-zero) for mismatch courts.
    format!("{}{}", "ab".repeat(16), "cd".repeat(16))
}

#[test]
fn stub_admit_accepts_aot_seal_shaped_digest() {
    // Arrange
    let pin = GgenLayoutDigestPin::from_aot_seal_digest(fixture_aot_digest());

    // Act
    let receipt = admit_ggen_layout_digest(Some(&pin)).expect("stub admit");

    // Assert
    assert_eq!(receipt.standing, "ADMITTED");
    assert_eq!(receipt.ggen_layout_digest, fixture_aot_digest());
    assert_eq!(receipt.to_json()["crown_wire"], "UNKNOWN");
}

#[test]
fn stub_admit_refuses_missing_and_malformed_pins() {
    let err = admit_ggen_layout_digest(None).expect_err("missing pin");
    assert_eq!(err.code, REFUSED_GGEN_DIGEST);

    let empty = GgenLayoutDigestPin::from_aot_seal_digest("   ");
    let err = admit_ggen_layout_digest(Some(&empty)).expect_err("empty");
    assert_eq!(err.code, REFUSED_GGEN_DIGEST);

    let short = GgenLayoutDigestPin::from_aot_seal_digest("deadbeef");
    let err = admit_ggen_layout_digest(Some(&short)).expect_err("short");
    assert_eq!(err.code, REFUSED_GGEN_DIGEST);
}

#[test]
fn stub_bind_into_mint_attaches_digest_keys() {
    // Arrange — synthetic sealed mint (redaction already ALIVE)
    let mut mint = json!({
        "ok": true,
        "kind": "linkedin-post",
        "redaction_court": {
            "standing": "ALIVE",
            "collaborator": "RedactionCourt",
        },
    });
    let pin = GgenLayoutDigestPin::from_aot_seal_digest(fixture_aot_digest());

    // Act
    bind_into_mint(&mut mint, &pin).expect("bind_into_mint");

    // Assert
    assert_eq!(mint[MINT_KEY_GGEN_LAYOUT_DIGEST], fixture_aot_digest());
    assert_eq!(mint[MINT_KEY_DIGEST_BIND]["collaborator"], "GgenDigestBind");
    assert_eq!(mint[MINT_KEY_DIGEST_BIND]["crown_wire"], "WIRED");
    assert_mint_binds_digest(&mint, Some(&fixture_aot_digest())).expect("assert bind");
}

/// Crown-wire falsifier: production subject-post with a threaded pin must bind.
#[test]
fn mint_receipt_must_bind_admitted_ggen_layout_digest() {
    // Arrange — real brand twin + AotSealReceipt-shaped pin
    let brand = BrandManifest::load_path("brands/fourth-watch.brand.toml")
        .expect("castle brands/fourth-watch.brand.toml");
    let pin_digest = fixture_aot_digest();
    let pin = GgenLayoutDigestPin::from_aot_seal_digest(pin_digest.clone());

    // Act — production subject-post mint with digest pin threaded
    let mint = publish_subject_post("spine-digest-bind-probe", &brand, true, Some(&pin))
        .expect("clean subject must mint under redaction/brand + digest-bind courts");

    // Assert — mint carries bound ggen layout digest (wire WIRED).
    assert_eq!(
        mint["redaction_court"]["standing"], "ALIVE",
        "precondition: redaction court must remain ALIVE"
    );
    assert_mint_binds_digest(&mint, Some(&pin_digest)).unwrap_or_else(|err| {
        panic!(
            "CROWN DIGEST-BIND MISSING on wired seal: {} — pin={}",
            err, pin_digest
        );
    });
    assert_eq!(mint[MINT_KEY_DIGEST_BIND]["crown_wire"], "WIRED");
    assert_eq!(mint[MINT_KEY_DIGEST_BIND]["collaborator"], "GgenDigestBind");
}

/// Omit pin → seal stays unbound (optional digest-bind; sibling courts green).
#[test]
fn seal_public_artifact_without_pin_omits_ggen_layout_digest() {
    let policy = RedactionPolicy::fourth_watch_defaults();
    let (sealed, court) = seal_public_artifact(
        json!({"ok": true, "body": "Audit the receipt chain before the next breach window closes."}),
        &policy,
        None,
    )
    .expect("clean artifact admits at redaction court");
    assert_eq!(court["standing"], "ALIVE");
    assert_eq!(court["collaborator"], "RedactionCourt");

    let err = assert_mint_binds_digest(&sealed, None).expect_err("unwired seal without pin");
    assert_eq!(err.code, REFUSED_GGEN_DIGEST);
    assert!(sealed.get(MINT_KEY_GGEN_LAYOUT_DIGEST).is_none());
}

/// Supply pin → seal binds digest on the production helper.
#[test]
fn seal_public_artifact_with_pin_binds_ggen_layout_digest() {
    let policy = RedactionPolicy::fourth_watch_defaults();
    let pin = GgenLayoutDigestPin::from_aot_seal_digest(fixture_aot_digest());
    let (sealed, court) = seal_public_artifact(
        json!({"ok": true, "body": "Audit the receipt chain before the next breach window closes."}),
        &policy,
        Some(&pin),
    )
    .expect("clean artifact admits + binds");
    assert_eq!(court["standing"], "ALIVE");
    assert_mint_binds_digest(&sealed, Some(&fixture_aot_digest())).expect("bound");
    assert_eq!(sealed[MINT_KEY_DIGEST_BIND]["crown_wire"], "WIRED");
}
