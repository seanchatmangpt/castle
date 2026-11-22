//! Ggen layout / brand digest-bind admit surface for CASTLE mint.
//!
//! **Standing honesty (Foundry crown wire):**
//! - This module admits a ggen
//!   [`AotSealReceipt`](https://github.com/seanchatmangpt/ggen)-shaped layout
//!   digest (64-hex BLAKE3) and binds it into a CASTLE mint receipt.
//! - Production publish wire (`wire::seal_public_artifact`) calls
//!   [`bind_into_mint`] when a pin is threaded (CLI `--ggen-layout-digest` or
//!   API `Option<&GgenLayoutDigestPin>`). Omit pin → mint omits the key
//!   (sibling redaction/brand paths stay green without a digest).
//! - Wire bind **WIRED** ≠ Foundry / zero-HITL crown `ALIVE`. Sibling
//!   redaction/brand courts ALIVE ≠ spine crown without a bound digest.
//!
//! Plug-in hooks (exact):
//! - After `RedactionCourt::admit` in `src/publish/wire.rs` →
//!   `seal_public_artifact` (optional pin → attach `ggen_layout_digest` +
//!   `digest_bind` receipt onto the sealed mint JSON).
//! - CLI: `castle publish … --ggen-layout-digest <64-hex>`
//!   ([`CLI_FLAG_GGEN_LAYOUT_DIGEST`]).
//! - Brand twin may later carry `extensions.ggen.layout_digest` (castle twin
//!   currently omits `extensions.ggen.*` by design).
//!
//! Chicago court: `tests/publish_digest_bind_court.rs` — all green when wire
//! threads a pin into `publish_subject_post` / `seal_public_artifact`.

#![forbid(unsafe_code)]

use serde_json::{json, Value};

/// CLI flag name wired into `castle publish …` clap-noun-verb routes.
pub const CLI_FLAG_GGEN_LAYOUT_DIGEST: &str = "--ggen-layout-digest";

/// Typed refusal when digest-bind admit fails.
pub const REFUSED_GGEN_DIGEST: &str = "REFUSED_GGEN_DIGEST";

/// Broken-term vocabulary for digest-bind refusals.
pub const BROKEN_DIGEST_BIND: &str = "digest_bind";

/// Collaborator name recorded on a successful bind receipt.
pub const COLLABORATOR: &str = "GgenDigestBind";

/// Mint JSON key for the bound ggen layout digest (AotSealReceipt.digest).
pub const MINT_KEY_GGEN_LAYOUT_DIGEST: &str = "ggen_layout_digest";

/// Mint JSON key for the digest-bind court receipt object.
pub const MINT_KEY_DIGEST_BIND: &str = "digest_bind";

/// Pin shaped like ggen `AotSealReceipt.digest` (domain-separated BLAKE3 hex).
///
/// Castle does not re-run Taffy / recompute the seal — identity is the pin;
/// integrity of the remote seal is the consumer's (`ui_layout` / zcode-cli).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GgenLayoutDigestPin {
    /// 64-char lowercase/uppercase ASCII hex digest.
    pub digest: String,
    /// Provenance label (e.g. `"AotSealReceipt"`, `"brand_rhythm"`).
    pub source: String,
}

impl GgenLayoutDigestPin {
    /// Construct a pin from an AotSealReceipt-shaped digest string.
    pub fn from_aot_seal_digest(digest: impl Into<String>) -> Self {
        Self {
            digest: digest.into(),
            source: "AotSealReceipt".to_string(),
        }
    }
}

/// Successful digest-bind admit receipt (CONSTRUCT-side; does not authorize DO).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestBindReceipt {
    pub standing: &'static str,
    pub code: &'static str,
    pub collaborator: &'static str,
    pub source: String,
    pub ggen_layout_digest: String,
    /// `"UNKNOWN"` for admit-only; `"WIRED"` after [`bind_into_mint`].
    /// Foundry / zero-HITL crown remains orthogonal.
    pub crown_wire: &'static str,
}

impl DigestBindReceipt {
    pub fn to_json(&self) -> Value {
        let note = if self.crown_wire == "WIRED" {
            "bind_into_mint attached ggen_layout_digest; Foundry/zero-HITL crown ≠ this wire"
        } else {
            "admit-only PRESENT; crown_wire stays UNKNOWN until bind_into_mint"
        };
        json!({
            "standing": self.standing,
            "code": self.code,
            "collaborator": self.collaborator,
            "source": self.source,
            "ggen_layout_digest": self.ggen_layout_digest,
            "crown_wire": self.crown_wire,
            "note": note,
        })
    }
}

/// Typed digest-bind refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestBindRefusal {
    pub code: &'static str,
    pub broken_term: &'static str,
    pub detail: String,
}

impl std::fmt::Display for DigestBindRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl DigestBindRefusal {
    pub fn to_json(&self) -> Value {
        json!({
            "ok": false,
            "class": "refused",
            "code": self.code,
            "broken_term": self.broken_term,
            "detail": self.detail,
        })
    }

    fn missing() -> Self {
        Self {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: "ggen layout digest pin required for digest-bind admit (ABSENT)".into(),
        }
    }

    fn empty() -> Self {
        Self {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: "ggen layout digest pin is empty".into(),
        }
    }

    fn bad_hex(observed: &str) -> Self {
        Self {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: format!(
                "ggen layout digest must be 64 ASCII hex chars (AotSealReceipt.digest); got len={} sample={}",
                observed.len(),
                observed.chars().take(16).collect::<String>()
            ),
        }
    }

    fn mismatch(expected: &str, observed: &str) -> Self {
        Self {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: format!(
                "ggen layout digest mismatch: expected_prefix={} observed_prefix={}",
                expected.chars().take(12).collect::<String>(),
                observed.chars().take(12).collect::<String>()
            ),
        }
    }

    fn unwired_on_mint() -> Self {
        Self {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: format!(
                "mint receipt lacks `{MINT_KEY_GGEN_LAYOUT_DIGEST}` — publish wire has not bound ggen digest (crown wire UNKNOWN)"
            ),
        }
    }
}

fn is_64_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Admit a ggen layout digest pin (fail closed).
///
/// Does not authorize LinkedIn send / DO. Pure SELECT/CONSTRUCT admit.
pub fn admit_ggen_layout_digest(
    pin: Option<&GgenLayoutDigestPin>,
) -> Result<DigestBindReceipt, DigestBindRefusal> {
    let pin = pin.ok_or_else(DigestBindRefusal::missing)?;
    let digest = pin.digest.trim();
    if digest.is_empty() {
        return Err(DigestBindRefusal::empty());
    }
    if !is_64_hex(digest) {
        return Err(DigestBindRefusal::bad_hex(digest));
    }
    Ok(DigestBindReceipt {
        standing: "ADMITTED",
        code: "ADMITTED",
        collaborator: COLLABORATOR,
        source: pin.source.clone(),
        ggen_layout_digest: digest.to_ascii_lowercase(),
        crown_wire: "UNKNOWN",
    })
}

/// Attach an admitted ggen layout digest onto a mint JSON object.
///
/// Production call site: `wire::seal_public_artifact` after redaction admit
/// when a ggen pin is threaded (CLI `--ggen-layout-digest` or API pin).
pub fn bind_into_mint(
    mint: &mut Value,
    pin: &GgenLayoutDigestPin,
) -> Result<DigestBindReceipt, DigestBindRefusal> {
    let mut receipt = admit_ggen_layout_digest(Some(pin))?;
    receipt.crown_wire = "WIRED";
    if let Some(obj) = mint.as_object_mut() {
        obj.insert(
            MINT_KEY_GGEN_LAYOUT_DIGEST.to_string(),
            json!(receipt.ggen_layout_digest),
        );
        obj.insert(MINT_KEY_DIGEST_BIND.to_string(), receipt.to_json());
    } else {
        return Err(DigestBindRefusal {
            code: REFUSED_GGEN_DIGEST,
            broken_term: BROKEN_DIGEST_BIND,
            detail: "mint payload must be a JSON object to bind ggen layout digest".into(),
        });
    }
    Ok(receipt)
}

/// Assert a mint receipt already carries a bound ggen layout digest.
///
/// Used by Chicago courts: production `publish_subject_post` /
/// `seal_public_artifact` with a pin must bind; omit pin → refuse.
pub fn assert_mint_binds_digest(
    mint: &Value,
    expected: Option<&str>,
) -> Result<(), DigestBindRefusal> {
    let observed = mint
        .get(MINT_KEY_GGEN_LAYOUT_DIGEST)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(DigestBindRefusal::unwired_on_mint)?;
    if !is_64_hex(observed) {
        return Err(DigestBindRefusal::bad_hex(observed));
    }
    if let Some(exp) = expected.map(str::trim).filter(|s| !s.is_empty()) {
        if !is_64_hex(exp) {
            return Err(DigestBindRefusal::bad_hex(exp));
        }
        if observed.to_ascii_lowercase() != exp.to_ascii_lowercase() {
            return Err(DigestBindRefusal::mismatch(exp, observed));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_digest() -> String {
        "a".repeat(64)
    }

    #[test]
    fn admits_valid_aot_seal_shaped_digest() {
        let pin = GgenLayoutDigestPin::from_aot_seal_digest(sample_digest());
        let receipt = admit_ggen_layout_digest(Some(&pin)).expect("admit");
        assert_eq!(receipt.standing, "ADMITTED");
        assert_eq!(receipt.ggen_layout_digest, sample_digest());
        assert_eq!(receipt.collaborator, COLLABORATOR);
        assert_eq!(receipt.crown_wire, "UNKNOWN");
        assert_eq!(receipt.to_json()["crown_wire"], "UNKNOWN");
    }

    #[test]
    fn refuses_missing_empty_and_bad_hex() {
        assert_eq!(
            admit_ggen_layout_digest(None).unwrap_err().code,
            REFUSED_GGEN_DIGEST
        );
        let empty = GgenLayoutDigestPin::from_aot_seal_digest("");
        assert_eq!(
            admit_ggen_layout_digest(Some(&empty)).unwrap_err().code,
            REFUSED_GGEN_DIGEST
        );
        let bad = GgenLayoutDigestPin::from_aot_seal_digest("deadbeef");
        assert_eq!(
            admit_ggen_layout_digest(Some(&bad)).unwrap_err().code,
            REFUSED_GGEN_DIGEST
        );
    }

    #[test]
    fn bind_into_mint_attaches_keys() {
        let mut mint = json!({"ok": true, "redaction_court": {"standing": "ALIVE"}});
        let pin = GgenLayoutDigestPin::from_aot_seal_digest(sample_digest());
        let receipt = bind_into_mint(&mut mint, &pin).expect("bind");
        assert_eq!(receipt.crown_wire, "WIRED");
        assert_eq!(mint[MINT_KEY_GGEN_LAYOUT_DIGEST], sample_digest());
        assert_eq!(mint[MINT_KEY_DIGEST_BIND]["collaborator"], COLLABORATOR);
        assert_eq!(mint[MINT_KEY_DIGEST_BIND]["crown_wire"], "WIRED");
        assert_mint_binds_digest(&mint, Some(&sample_digest())).expect("assert");
    }

    #[test]
    fn assert_mint_refuses_when_unwired() {
        let mint = json!({"ok": true, "redaction_court": {"standing": "ALIVE"}});
        let err = assert_mint_binds_digest(&mint, None).expect_err("unwired");
        assert_eq!(err.code, REFUSED_GGEN_DIGEST);
        assert!(err.detail.contains(MINT_KEY_GGEN_LAYOUT_DIGEST));
    }
}
