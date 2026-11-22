//! Brand manifest loader for `castle publish --brand`.
//!
//! Consumes Fourth Watch `*.brand.toml` (messaging + redaction_court + DTCG
//! token tables + honesty/commercial floors). Schema matches the intentional
//! slim twins:
//! - `castle/brands/fourth-watch.brand.toml`
//! - `zcode-cli/brands/fourth-watch.brand.toml`
//!
//! SoT (full BrandTheme pack) lives at
//! `ggen-marketplace/packs/mobile-ui-layout-pack/brand/fourth-watch.brand.toml`.
//! Twins project castle-facing floors (`[messaging.cta]`, `[messaging.commercial]`,
//! `[messaging.redaction_court]`, `[extensions.honesty]` incl. Jobs
//! `blast_unlock_primary` / `low_durable_status`, top-level
//! `[redaction_court].require_public_anonymize`) and omit BrandTheme-only
//! `[meta]` / `primitive` / `semantic` / `component` / `extensions.ggen` tables.
//! Intentional twin drift: `tokens.typography.scale_body_px` stays ≥24 (LinkedIn
//! carousel court); SoT BrandTheme body is 18. Unknown SoT tables are ignored
//! on parse (no `deny_unknown_fields`). Policy actuation reads top-level
//! `[redaction_court]` only; `[messaging.redaction_court]` is the GTM mirror.
#![forbid(unsafe_code)]

use serde::Deserialize;
use std::path::Path;

use super::anonymize::{require_public_anonymize, AnonymizeRefusal, RedactionPolicy};

/// Top-level brand identity.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BrandIdentity {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub tagline: Option<String>,
    pub domain: Option<String>,
    pub intake_airlock: Option<String>,
    pub intake_policy: Option<String>,
    #[serde(default)]
    pub discovery_calls: bool,
    #[serde(default)]
    pub target_verticals: Vec<String>,
}

/// Sentinel / Steward / Shepherd triad copy.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct MessagingTriad {
    pub sentinel: Option<String>,
    pub steward: Option<String>,
    pub shepherd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct MessagingVoice {
    pub register: Option<String>,
    #[serde(default)]
    pub prohibited_patterns: Vec<String>,
    pub required_cta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct MessagingCta {
    pub featured_host: Option<String>,
    pub featured_path: Option<String>,
    pub featured_url_template: Option<String>,
    pub intake_policy: Option<String>,
    #[serde(default)]
    pub discovery_calls: bool,
    #[serde(default)]
    pub forbidden_hosts: Vec<String>,
    pub drop_gated_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct MessagingCommercial {
    pub custom_redline_duty_usd: Option<u64>,
    pub weekly_cell_usd: Option<u64>,
    pub custody_pct: Option<f64>,
    pub transaction_rent_pct: Option<f64>,
    #[serde(default)]
    pub target_verticals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct Messaging {
    #[serde(default)]
    pub triad: MessagingTriad,
    #[serde(default)]
    pub voice: MessagingVoice,
    #[serde(default)]
    pub cta: MessagingCta,
    #[serde(default)]
    pub commercial: MessagingCommercial,
    /// GTM mirror of top-level `[redaction_court]` (SoT + slim twins).
    /// Actuation still uses [`BrandManifest::redaction_court`].
    #[serde(default)]
    pub redaction_court: RedactionCourtToml,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct RedactionCourtToml {
    #[serde(default = "default_true")]
    pub forbid_private_client_names: bool,
    #[serde(default = "default_true")]
    pub forbid_local_user_paths: bool,
    #[serde(default = "default_true")]
    pub forbid_bearer_tokens: bool,
    #[serde(default = "default_true")]
    pub forbid_external_links_in_post_body: bool,
    #[serde(default = "default_true")]
    pub forbid_calendar_links: bool,
    /// When true, public publish targets refuse without `--public-anonymize`.
    #[serde(default = "default_true")]
    pub require_public_anonymize: bool,
    /// Optional denylist of private client / speaker names.
    #[serde(default)]
    pub private_client_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct HonestyExtensions {
    pub webgpu_120hz_status: Option<String>,
    pub zero_dom_status: Option<String>,
    pub raid_4cloud_status: Option<String>,
    pub linkedin_graph_count_status: Option<String>,
    pub intake_host_of_record: Option<String>,
    /// Jobs primary blast unlock: /readyz 200 ∧ 2× durable LOW-only — not a shipping claim.
    pub blast_unlock_primary: Option<String>,
    /// 2× LOW durable is a required second gate (AND with /readyz); status only — not unlock alone.
    pub low_durable_status: Option<String>,
    #[serde(default)]
    pub forbid_shipping_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct BrandExtensions {
    #[serde(default)]
    pub honesty: HonestyExtensions,
}

fn default_true() -> bool {
    true
}

/// DTCG-ish `{ value = "…", type = "…" }` leaf used by color/type/spacing tables.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TokenLeaf {
    pub value: toml::Value,
    #[serde(rename = "type")]
    pub token_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct ColorTokens {
    pub bg_obsidian: Option<TokenLeaf>,
    pub bg_slate: Option<TokenLeaf>,
    pub border_steel: Option<TokenLeaf>,
    pub text_primary: Option<TokenLeaf>,
    pub text_muted: Option<TokenLeaf>,
    pub accent_brass: Option<TokenLeaf>,
    pub status_alive: Option<TokenLeaf>,
    pub status_refused: Option<TokenLeaf>,
    pub status_partial: Option<TokenLeaf>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct DtcgMeta {
    pub spec_version: Option<String>,
    pub surface_target: Option<String>,
    pub surface_target_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct TypographyTokens {
    pub font_display: Option<TokenLeaf>,
    pub font_mono: Option<TokenLeaf>,
    pub scale_h1_px: Option<TokenLeaf>,
    pub scale_h2_px: Option<TokenLeaf>,
    pub scale_body_px: Option<TokenLeaf>,
    pub scale_caption_px: Option<TokenLeaf>,
    pub base_min_px: Option<TokenLeaf>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct SpacingTokens {
    pub unit_px: Option<TokenLeaf>,
    pub touch_target_min_px: Option<TokenLeaf>,
    pub touch_target_comfortable_px: Option<TokenLeaf>,
    pub carousel_width_px: Option<TokenLeaf>,
    pub carousel_height_px: Option<TokenLeaf>,
    pub safe_margin_px: Option<TokenLeaf>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct Tokens {
    #[serde(default)]
    pub dtcg: DtcgMeta,
    #[serde(default)]
    pub color: ColorTokens,
    #[serde(default)]
    pub typography: TypographyTokens,
    #[serde(default)]
    pub spacing: SpacingTokens,
}

/// Full Fourth Watch brand manifest.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BrandManifest {
    pub brand: BrandIdentity,
    #[serde(default)]
    pub messaging: Messaging,
    #[serde(default)]
    pub redaction_court: RedactionCourtToml,
    #[serde(default)]
    pub extensions: BrandExtensions,
    #[serde(default)]
    pub tokens: Tokens,
}

/// Typed brand load failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrandLoadError {
    pub detail: String,
}

impl std::fmt::Display for BrandLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "REFUSED_BRAND: {}", self.detail)
    }
}

impl std::error::Error for BrandLoadError {}

impl BrandManifest {
    /// Parse brand TOML from UTF-8 bytes.
    pub fn parse_toml(bytes: &str) -> Result<Self, BrandLoadError> {
        toml::from_str(bytes).map_err(|e| BrandLoadError {
            detail: format!("invalid brand.toml: {e}"),
        })
    }

    /// Load from a filesystem path (I/O admitted at CLI boundary only).
    pub fn load_path(path: impl AsRef<Path>) -> Result<Self, BrandLoadError> {
        let path = path.as_ref();
        let bytes = std::fs::read_to_string(path).map_err(|e| BrandLoadError {
            detail: format!("unreadable brand path {}: {e}", path.display()),
        })?;
        Self::parse_toml(&bytes)
    }

    /// Project `[redaction_court]` into the anonymize policy.
    /// Empty `private_client_names` under forbid-on loads the default denylist.
    pub fn redaction_policy(&self) -> RedactionPolicy {
        RedactionPolicy {
            forbid_private_client_names: self.redaction_court.forbid_private_client_names,
            forbid_local_user_paths: self.redaction_court.forbid_local_user_paths,
            forbid_bearer_tokens: self.redaction_court.forbid_bearer_tokens,
            forbid_external_links_in_post_body: self
                .redaction_court
                .forbid_external_links_in_post_body,
            forbid_calendar_links: self.redaction_court.forbid_calendar_links,
            private_names: self.redaction_court.private_client_names.clone(),
        }
        .with_default_denylist_if_empty()
    }

    /// Whether public publish targets must see `--public-anonymize`.
    ///
    /// Sourced from `[redaction_court].require_public_anonymize` (SoT default true).
    pub fn requires_public_anonymize(&self) -> bool {
        self.redaction_court.require_public_anonymize
    }

    /// Fail closed when the brand floor requires anonymize and the CLI flag is absent.
    pub fn enforce_public_anonymize(&self, public_anonymize: bool) -> Result<(), AnonymizeRefusal> {
        if self.requires_public_anonymize() {
            require_public_anonymize(public_anonymize)
        } else {
            Ok(())
        }
    }

    /// Featured CTA host of record (`[messaging.cta].featured_host`).
    pub fn featured_cta_host(&self) -> Option<&str> {
        self.messaging.cta.featured_host.as_deref()
    }

    /// Prefer brand spacing tokens; fall back to LinkedIn court floors.
    ///
    /// Dual-surface honesty: marketplace SoT `scale_body_px=18` is the BrandTheme
    /// UI/CSS surface. Carousel publish always clamps to ≥24pt (LinkedIn court);
    /// slim twins therefore project `scale_body_px=24` rather than copying SoT's 18.
    pub fn carousel_geometry(&self) -> (u32, u32, u32, f64) {
        let width = token_u32(&self.tokens.spacing.carousel_width_px).unwrap_or(1080);
        let height = token_u32(&self.tokens.spacing.carousel_height_px).unwrap_or(1350);
        let margin = token_u32(&self.tokens.spacing.safe_margin_px).unwrap_or(72);
        let body_pt = token_f64(&self.tokens.typography.scale_body_px).unwrap_or(24.0);
        (width, height, margin, body_pt.max(24.0))
    }

    /// Accent / canvas colors for Typst theming (hex strings when present).
    pub fn typst_palette(&self) -> TypstPalette {
        TypstPalette {
            bg: token_str(&self.tokens.color.bg_obsidian).unwrap_or_else(|| "#0B0F14".into()),
            surface: token_str(&self.tokens.color.bg_slate).unwrap_or_else(|| "#131A22".into()),
            text: token_str(&self.tokens.color.text_primary).unwrap_or_else(|| "#F3F6FA".into()),
            accent: token_str(&self.tokens.color.accent_brass).unwrap_or_else(|| "#C89B3C".into()),
            muted: token_str(&self.tokens.color.text_muted).unwrap_or_else(|| "#9AA8B8".into()),
        }
    }
}

/// Colors injected into branded Typst decks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypstPalette {
    pub bg: String,
    pub surface: String,
    pub text: String,
    pub accent: String,
    pub muted: String,
}

fn token_str(leaf: &Option<TokenLeaf>) -> Option<String> {
    leaf.as_ref().and_then(|t| match &t.value {
        toml::Value::String(s) => Some(s.clone()),
        other => Some(other.to_string().trim_matches('"').to_string()),
    })
}

fn token_u32(leaf: &Option<TokenLeaf>) -> Option<u32> {
    leaf.as_ref().and_then(|t| match &t.value {
        toml::Value::Integer(i) if *i >= 0 && *i <= u32::MAX as i64 => Some(*i as u32),
        toml::Value::String(s) => s.trim_end_matches("px").parse().ok(),
        _ => None,
    })
}

fn token_f64(leaf: &Option<TokenLeaf>) -> Option<f64> {
    leaf.as_ref().and_then(|t| match &t.value {
        toml::Value::Integer(i) => Some(*i as f64),
        toml::Value::Float(f) => Some(*f),
        toml::Value::String(s) => s.trim_end_matches("px").parse().ok(),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r##"
[brand]
id = "fourth-watch"
name = "Fourth Watch AI Cybersecurity"
short_name = "Fourth Watch"

[messaging.triad]
sentinel = "watch"
steward = "own"
shepherd = "guide"

[redaction_court]
forbid_private_client_names = true
forbid_local_user_paths = true
private_client_names = ["Acme"]

[tokens.color]
accent_brass = { value = "#C89B3C", type = "color" }

[tokens.spacing]
carousel_width_px = { value = 1080, type = "dimension" }
carousel_height_px = { value = 1350, type = "dimension" }
safe_margin_px = { value = 72, type = "dimension" }
"##;

    #[test]
    fn parses_fourth_watch_shape() {
        let m = BrandManifest::parse_toml(MINIMAL).expect("parse");
        assert_eq!(m.brand.id, "fourth-watch");
        let policy = m.redaction_policy();
        assert!(policy.forbid_local_user_paths);
        assert_eq!(policy.private_names, vec!["Acme".to_string()]);
        let (w, h, margin, _) = m.carousel_geometry();
        assert_eq!((w, h, margin), (1080, 1350, 72));
        assert_eq!(m.typst_palette().accent, "#C89B3C");
    }

    #[test]
    fn loads_castle_twin_and_marketplace_sot() {
        let castle = BrandManifest::load_path("brands/fourth-watch.brand.toml")
            .expect("castle brands twin");
        assert_eq!(castle.brand.id, "fourth-watch");
        assert!(
            castle
                .messaging
                .triad
                .sentinel
                .as_deref()
                .unwrap_or("")
                .contains("Claims vs Code"),
            "sentinel triad required"
        );
        assert!(
            !castle
                .messaging
                .triad
                .shepherd
                .as_deref()
                .unwrap_or("")
                .contains("120 Hz"),
            "shepherd must not claim 120 Hz shipping"
        );
        assert_shared_floors(&castle, "castle twin");

        let sot = BrandManifest::load_path(
            "/Users/sac/ggen-marketplace/packs/mobile-ui-layout-pack/brand/fourth-watch.brand.toml",
        )
        .expect("marketplace SoT");
        assert_eq!(sot.brand.id, "fourth-watch");
        assert!(sot.redaction_policy().forbid_private_client_names);
        assert!(
            sot.brand
                .intake_airlock
                .as_deref()
                .unwrap_or("")
                .contains("cos.fnox.io"),
            "intake must be cos.fnox.io"
        );
        assert_shared_floors(&sot, "marketplace SoT");
    }

    fn assert_shared_floors(m: &BrandManifest, label: &str) {
        assert_eq!(
            m.messaging.cta.featured_host.as_deref(),
            Some("cos.fnox.io"),
            "{label}: messaging.cta.featured_host"
        );
        assert!(
            m.messaging
                .cta
                .forbidden_hosts
                .iter()
                .any(|h| h == "revops.fnox.io"),
            "{label}: messaging.cta.forbidden_hosts"
        );
        let gate = m.messaging.cta.drop_gated_note.as_deref().unwrap_or("");
        assert!(
            gate.contains("SlotLease") && gate.contains("/readyz"),
            "{label}: messaging.cta.drop_gated_note must surface SlotLease/`/readyz`"
        );
        assert_eq!(
            m.messaging.commercial.weekly_cell_usd,
            Some(5000),
            "{label}: messaging.commercial.weekly_cell_usd"
        );
        assert_eq!(
            m.messaging.commercial.custom_redline_duty_usd,
            Some(25000),
            "{label}: messaging.commercial.custom_redline_duty_usd"
        );
        assert_eq!(
            m.messaging.commercial.custody_pct,
            Some(0.0),
            "{label}: messaging.commercial.custody_pct"
        );
        assert!(
            m.redaction_court.require_public_anonymize,
            "{label}: redaction_court.require_public_anonymize"
        );
        assert!(
            m.messaging.redaction_court.require_public_anonymize,
            "{label}: messaging.redaction_court.require_public_anonymize"
        );
        assert!(
            m.enforce_public_anonymize(false).is_err(),
            "{label}: enforce_public_anonymize refuses without flag"
        );
        assert!(
            m.enforce_public_anonymize(true).is_ok(),
            "{label}: enforce_public_anonymize accepts flag"
        );
        assert_eq!(
            m.extensions.honesty.intake_host_of_record.as_deref(),
            Some("cos.fnox.io"),
            "{label}: extensions.honesty.intake_host_of_record"
        );
        assert_eq!(
            m.extensions.honesty.raid_4cloud_status.as_deref(),
            Some("target-not-proven"),
            "{label}: extensions.honesty.raid_4cloud_status"
        );
        assert_eq!(
            m.extensions.honesty.webgpu_120hz_status.as_deref(),
            Some("doc-only-not-shipping"),
            "{label}: extensions.honesty.webgpu_120hz_status"
        );
        let unlock = m
            .extensions
            .honesty
            .blast_unlock_primary
            .as_deref()
            .unwrap_or("");
        assert!(
            unlock.contains("/readyz") && unlock.contains("2× durable LOW-only"),
            "{label}: extensions.honesty.blast_unlock_primary AND-gate (/readyz ∧ 2× LOW)"
        );
        let low = m
            .extensions
            .honesty
            .low_durable_status
            .as_deref()
            .unwrap_or("");
        assert!(
            low.contains("required-second-gate") && !low.contains("void-as-unlock"),
            "{label}: extensions.honesty.low_durable_status required second gate (not void-as-unlock)"
        );
        assert!(
            m.extensions
                .honesty
                .forbid_shipping_claims
                .iter()
                .any(|c| c == "@fnox/* GA"),
            "{label}: extensions.honesty.forbid_shipping_claims includes @fnox/* GA"
        );
        let cta = m.messaging.voice.required_cta.as_deref().unwrap_or("");
        assert!(
            !cta.to_lowercase().contains("https://")
                && !cta.to_lowercase().contains("http://")
                && !cta.to_lowercase().contains("www."),
            "{label}: required_cta must be body-safe prose (no link markers); URL belongs in messaging.cta.featured_url_template"
        );
        assert!(
            cta.contains("Featured") && cta.contains("cos.fnox.io"),
            "{label}: required_cta must name Featured host in prose"
        );
        assert!(
            m.messaging
                .cta
                .featured_url_template
                .as_deref()
                .unwrap_or("")
                .starts_with("https://cos.fnox.io/"),
            "{label}: featured_url_template keeps honest https Featured URL off the body"
        );
        assert!(
            cta.contains("SlotLease") && cta.contains("/readyz"),
            "{label}: required_cta must surface SlotLease /readyz gate"
        );
        for pattern in ["@fnox/* GA", "2× LOW cleared", "code bar cleared"] {
            assert!(
                m.messaging
                    .voice
                    .prohibited_patterns
                    .iter()
                    .any(|p| p == pattern),
                "{label}: prohibited_patterns missing {pattern}"
            );
        }
    }

    #[test]
    fn dual_surface_body_18_sot_vs_24_twin_is_intentional() {
        let twin = BrandManifest::load_path("brands/fourth-watch.brand.toml")
            .expect("castle brands twin");
        let sot = BrandManifest::load_path(
            "/Users/sac/ggen-marketplace/packs/mobile-ui-layout-pack/brand/fourth-watch.brand.toml",
        )
        .expect("marketplace SoT");
        let twin_body = token_f64(&twin.tokens.typography.scale_body_px).expect("twin body");
        let sot_body = token_f64(&sot.tokens.typography.scale_body_px).expect("sot body");
        assert_eq!(
            sot_body, 18.0,
            "SoT BrandTheme / UI CSS surface body stays 18"
        );
        assert_eq!(
            twin_body, 24.0,
            "slim twin carousel court body stays 24 (intentional dual-surface, not drift)"
        );
        let (_, _, _, twin_clamped) = twin.carousel_geometry();
        assert_eq!(twin_clamped, 24.0, "carousel_geometry keeps twin body at ≥24");
        let (_, _, _, sot_clamped) = sot.carousel_geometry();
        assert_eq!(
            sot_clamped, 24.0,
            "carousel_geometry clamps SoT UI body 18 up to carousel floor 24"
        );
    }

    #[test]
    fn castle_and_zcode_twins_are_byte_identical() {
        let castle = std::fs::read("brands/fourth-watch.brand.toml").expect("castle twin bytes");
        let zcode_path = Path::new("/Users/sac/zcode-cli/brands/fourth-watch.brand.toml");
        if !zcode_path.is_file() {
            // CI / machines without the sibling checkout skip the cross-repo pin.
            return;
        }
        let zcode = std::fs::read(zcode_path).expect("zcode twin bytes");
        assert_eq!(
            castle, zcode,
            "intentional twin sync: castle/brands ≡ zcode-cli/brands (SoT hash may differ)"
        );
        let header = String::from_utf8_lossy(&castle);
        assert!(
            header.contains("Dual-surface") || header.contains("dual-surface"),
            "twin header must document dual-surface LinkedIn ≥24 vs SoT UI 18"
        );
    }
}
