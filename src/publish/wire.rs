//! Orchestration for `castle publish --public-anonymize --brand <path>`.
//!
//! Pipeline: require anonymize flag → require explicit `--brand` (no soft
//! default) → load brand → mint artifact (Typst source for carousels) →
//! [`RedactionCourt::admit`] seal (refuse, never strip-and-ship) → optional
//! PDF compile (gap-honest).
#![forbid(unsafe_code)]

use serde_json::{json, Value};
use std::path::Path;

use super::anonymize::{
    require_public_anonymize, AnonymizeRefusal, RedactionCourt, RedactionPolicy, RedactionRefusal,
};
use super::brand::{BrandLoadError, BrandManifest};
use super::linkedin::{
    publish_carousel, publish_newsletter, publish_post, CarouselInput, NewsletterInput, PostInput,
    PublishRefusal, SlideInput,
};
use super::typst_pdf::{try_compile_pdf, PdfCompileStatus};

/// Public publish targets that mandate `--public-anonymize`.
pub const PUBLIC_TARGETS: &[&str] = &[
    "linkedin-carousel",
    "linkedin-newsletter",
    "linkedin-post",
    "gatekeeper-pack",
    "carousel",
    "newsletter",
    "post",
];

/// Unified publish refusal surfaced to CLI as structured JSON.
#[derive(Debug, Clone)]
pub enum PublishWireError {
    Anonymize(AnonymizeRefusal),
    Redaction(RedactionRefusal),
    Brand(BrandLoadError),
    Lint(PublishRefusal),
    Input(String),
}

impl PublishWireError {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Anonymize(e) => json!({
                "ok": false,
                "class": e.class,
                "code": e.code,
                "broken_term": e.broken_term,
                "detail": e.detail,
            }),
            Self::Redaction(e) => json!({
                "ok": false,
                "class": "refused",
                "code": e.code,
                "broken_term": e.broken_term,
                "findings": e.findings,
            }),
            Self::Brand(e) => json!({
                "ok": false,
                "class": "refused",
                "code": "REFUSED_BRAND",
                "detail": e.detail,
            }),
            Self::Lint(e) => json!({
                "ok": false,
                "class": "refused",
                "code": e.code,
                "broken_term": e.broken_term,
                "findings": e.findings,
            }),
            Self::Input(detail) => json!({
                "ok": false,
                "class": "refused",
                "code": "REFUSED_INPUT",
                "detail": detail,
            }),
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Anonymize(e) => e.to_string(),
            Self::Redaction(e) => e.to_string(),
            Self::Brand(e) => e.to_string(),
            Self::Lint(e) => e.to_string(),
            Self::Input(d) => format!("REFUSED_INPUT: {d}"),
        }
    }
}

fn target_requires_anonymize(target: &str) -> bool {
    PUBLIC_TARGETS
        .iter()
        .any(|t| target == *t || target.contains(t))
}

/// Explicit `--brand <path>` required for public publish (no soft default).
///
/// Omit / empty → `REFUSED_BRAND` (CLI exit 2 via handler envelope). Orthogonal to
/// `--public-anonymize` and to F-ANON-3 redaction/CTA courts.
pub fn require_brand_path(brand: Option<&str>) -> Result<&str, PublishWireError> {
    brand
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .ok_or_else(|| {
            PublishWireError::Brand(BrandLoadError {
                detail: "public publish requires --brand <path> (no soft default)".into(),
            })
        })
}

/// Load brand from `--brand` path (required for branded public publish).
pub fn load_brand(brand_path: &str) -> Result<BrandManifest, PublishWireError> {
    BrandManifest::load_path(brand_path).map_err(PublishWireError::Brand)
}

/// Fail-closed seal: [`RedactionCourt::admit`] is the wired collaborator.
///
/// Law: refuse private residue — never strip-and-ship. On success the artifact
/// bytes are unchanged and the court receipt records an ALIVE admit.
pub fn seal_public_artifact(
    artifact: Value,
    policy: &RedactionPolicy,
) -> Result<(Value, Value), PublishWireError> {
    let court = RedactionCourt::from_policy(policy);
    let flat = artifact.to_string();
    court.admit(&flat).map_err(PublishWireError::Redaction)?;
    let receipt = json!({
        "public_anonymize": true,
        "standing": "ALIVE",
        "code": "ADMITTED",
        "collaborator": "RedactionCourt",
    });
    Ok((artifact, receipt))
}

/// Publish a LinkedIn carousel: Typst source ALIVE; PDF optional/gap.
pub fn publish_branded_carousel(
    input: &CarouselInput,
    brand: &BrandManifest,
    public_anonymize: bool,
    out_dir: Option<&Path>,
) -> Result<Value, PublishWireError> {
    brand
        .enforce_public_anonymize(public_anonymize)
        .map_err(PublishWireError::Anonymize)?;
    let policy = brand.redaction_policy();

    // Apply brand geometry floors without violating LinkedIn court mins.
    // Copy is not stripped here — seal_public_artifact refuses via RedactionCourt.
    let (_w, _h, brand_margin, brand_body_pt) = brand.carousel_geometry();
    let mut branded = input.clone();
    branded.safe_margin_px = branded.safe_margin_px.max(brand_margin).max(50);
    branded.body_pt = branded.body_pt.max(brand_body_pt).max(24.0);

    let artifact = publish_carousel(&branded).map_err(PublishWireError::Lint)?;
    let palette = brand.typst_palette();
    let branded_typst = inject_brand_theme(&artifact.typst, brand, &palette);

    let mut payload = json!({
        "ok": true,
        "kind": "linkedin-carousel",
        "brand_id": brand.brand.id,
        "brand_name": brand.brand.name,
        "lease_id": artifact.lease_id,
        "slide_count": artifact.slide_count,
        "width_px": artifact.width_px,
        "height_px": artifact.height_px,
        "safe_margin_px": artifact.safe_margin_px,
        "body_pt": artifact.body_pt,
        "cta": artifact.cta,
        "digest": artifact.digest,
        "executive_check": artifact.executive_check,
        "typst": branded_typst,
        "typst_source_standing": "ALIVE",
        "messaging": {
            "sentinel": brand.messaging.triad.sentinel,
            "steward": brand.messaging.triad.steward,
            "shepherd": brand.messaging.triad.shepherd,
            "required_cta": brand.messaging.voice.required_cta,
        },
    });

    let pdf_status = match out_dir {
        Some(dir) => {
            let _ = std::fs::create_dir_all(dir);
            let pdf = dir.join(format!("{}-carousel.pdf", brand.brand.id));
            let typ = dir.join(format!("{}-carousel.typ", brand.brand.id));
            let _ = std::fs::write(&typ, payload["typst"].as_str().unwrap_or(""));
            try_compile_pdf(
                payload["typst"].as_str().unwrap_or(""),
                &pdf,
                Some(&typ),
            )
        }
        None => PdfCompileStatus::Gap {
            reason: "no --out-dir; Typst source emit OK; PDF compile skipped".to_string(),
            typst_source_emitted: true,
        },
    };
    payload["pdf"] = serde_json::to_value(&pdf_status).unwrap_or(json!({}));

    let (sealed, court) = seal_public_artifact(payload, &policy)?;
    let mut sealed = sealed;
    sealed["redaction_court"] = court;
    Ok(sealed)
}

/// Publish a LinkedIn post under brand + anonymize courts.
pub fn publish_branded_post(
    input: &PostInput,
    brand: &BrandManifest,
    public_anonymize: bool,
) -> Result<Value, PublishWireError> {
    brand
        .enforce_public_anonymize(public_anonymize)
        .map_err(PublishWireError::Anonymize)?;
    let policy = brand.redaction_policy();
    // No strip-and-ship: lint mints on raw copy; seal refuses via RedactionCourt.
    let artifact = publish_post(input).map_err(PublishWireError::Lint)?;
    let payload = json!({
        "ok": true,
        "kind": "linkedin-post",
        "brand_id": brand.brand.id,
        "brand_name": brand.brand.name,
        "hook": artifact.hook,
        "body": artifact.body,
        "full_text": artifact.full_text,
        "first_comment": artifact.first_comment,
        "word_count": artifact.word_count,
        "digest": artifact.digest,
        "typst_source_standing": "N/A",
        "pdf": PdfCompileStatus::Gap {
            reason: "post target has no Typst/PDF surface".to_string(),
            typst_source_emitted: false,
        },
    });
    let (mut sealed, court) = seal_public_artifact(payload, &policy)?;
    sealed["redaction_court"] = court;
    Ok(sealed)
}

/// Publish a newsletter under brand + anonymize courts.
pub fn publish_branded_newsletter(
    input: &NewsletterInput,
    brand: &BrandManifest,
    public_anonymize: bool,
) -> Result<Value, PublishWireError> {
    brand
        .enforce_public_anonymize(public_anonymize)
        .map_err(PublishWireError::Anonymize)?;
    let policy = brand.redaction_policy();
    // No strip-and-ship: seal_public_artifact is the refuse court.
    let artifact = publish_newsletter(input).map_err(PublishWireError::Lint)?;
    let payload = json!({
        "ok": true,
        "kind": "linkedin-newsletter",
        "brand_id": brand.brand.id,
        "brand_name": brand.brand.name,
        "title": artifact.title,
        "markdown": artifact.markdown,
        "html": artifact.html,
        "digest": artifact.digest,
        "typst_source_standing": "N/A",
        "pdf": PdfCompileStatus::Gap {
            reason: "newsletter target emits Markdown/HTML; Typst/PDF GAP by design".to_string(),
            typst_source_emitted: false,
        },
    });
    let (mut sealed, court) = seal_public_artifact(payload, &policy)?;
    sealed["redaction_court"] = court;
    Ok(sealed)
}

/// Subject-only post court used by F-ANON probes (`publish post --subject t`).
///
/// Builds a lint-admissible post from brand triad copy + subject, then seals.
pub fn publish_subject_post(
    subject: &str,
    brand: &BrandManifest,
    public_anonymize: bool,
) -> Result<Value, PublishWireError> {
    brand
        .enforce_public_anonymize(public_anonymize)
        .map_err(PublishWireError::Anonymize)?;
    // Subject is admitted raw at seal — private residue refuses (no strip).

    let sentinel = brand
        .messaging
        .triad
        .sentinel
        .clone()
        .unwrap_or_else(|| "Pinned probes refuse silent failure.".into());
    let steward = brand
        .messaging
        .triad
        .steward
        .clone()
        .unwrap_or_else(|| "Client-owned RAID under your EIN.".into());
    let shepherd = brand
        .messaging
        .triad
        .shepherd
        .clone()
        .unwrap_or_else(|| "Keyless shims keep staff working.".into());
    // Body uses host-of-record (no scheme). Absolute Featured URL rides
    // cta_link → first_comment only (POST_EXTERNAL_LINK + redaction_court).
    let lease_id = "lease-public";
    let body_cta = body_safe_featured_cta(brand, lease_id);
    let cta_link = brand
        .messaging
        .cta
        .featured_url_template
        .as_ref()
        .map(|t| t.replace(":lease_id", lease_id))
        .or_else(|| {
            brand
                .messaging
                .voice
                .required_cta
                .as_deref()
                .and_then(extract_first_absolute_url)
                .map(|u| u.replace(":lease_id", lease_id))
        });

    // Build ≥150 words of action copy without soft scripts or external links.
    let mut body = format!(
        "{subject}. {sentinel} {steward} {shepherd} {body_cta} \
         Audit the receipt chain before the next breach window closes. \
         Measure every claim against pinned evidence. Refuse hollow demos. \
         Seal the lease at the airlock. Track drift until the standing is ALIVE. \
         Cut the silent failure mode that bleeds trust. Guard the drop path. \
         Verify the board PDF. Own the hyperscaler cells. Stop the rent. \
         Preserve client custody. Rewrite the unverified path. Deploy the court."
    );
    while count_words_approx(&body) < 150 {
        body.push_str(" Verify again. Seal again. Refuse soft scripts.");
    }

    let hook = {
        let mut h = format!("Stop the {subject} breach window");
        if h.chars().count() >= 140 {
            h = "Stop the breach window now".to_string();
        }
        h
    };

    let input = PostInput {
        hook,
        body,
        lease_id: Some(lease_id.to_string()),
        cta_link,
    };
    publish_branded_post(&input, brand, true)
}

/// Host-of-record CTA for LinkedIn post body (never embeds `https://`).
fn body_safe_featured_cta(brand: &BrandManifest, lease_id: &str) -> String {
    if let Some(host) = brand
        .messaging
        .cta
        .featured_host
        .as_deref()
        .map(str::trim)
        .filter(|h| !h.is_empty())
    {
        let path = brand
            .messaging
            .cta
            .featured_path
            .as_deref()
            .unwrap_or("/drop/:lease_id")
            .replace(":lease_id", lease_id);
        let mut body = format!(
            "Drop a read-only snapshot at Featured {host}{path} for a binary Claims vs Code board PDF."
        );
        if let Some(note) = brand
            .messaging
            .cta
            .drop_gated_note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
        {
            // Notes stay body-surface: strip schemes so a gated-note URL cannot
            // re-poison POST_EXTERNAL_LINK (Featured absolute URL is first_comment).
            body.push(' ');
            body.push_str(
                &strip_external_link_schemes(note).replace(":lease_id", lease_id),
            );
        }
        return collapse_whitespace(&strip_external_link_schemes(&body));
    }

    let raw = brand
        .messaging
        .voice
        .required_cta
        .as_deref()
        .unwrap_or(
            "Drop a read-only snapshot at Featured /drop/:lease_id on cos.fnox.io for a binary Claims vs Code board PDF.",
        );
    collapse_whitespace(
        &strip_external_link_schemes(raw).replace(":lease_id", lease_id),
    )
}

fn extract_first_absolute_url(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for scheme in ["https://", "http://"] {
        let mut search_from = 0usize;
        while let Some(rel) = lower[search_from..].find(scheme) {
            let start = search_from + rel;
            let boundary_ok = start == 0
                || !lower[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '.' || c == '/');
            if boundary_ok {
                let rest = &text[start..];
                let end = rest
                    .find(|c: char| {
                        c.is_whitespace()
                            || matches!(c, ')' | ']' | '"' | '\'' | '<' | '>' | '(')
                    })
                    .unwrap_or(rest.len());
                let mut url = rest[..end].to_string();
                while url
                    .chars()
                    .next_back()
                    .is_some_and(|c| matches!(c, '.' | ',' | ';' | ':' | '!' | '?'))
                {
                    url.pop();
                }
                if url.len() > scheme.len() {
                    return Some(url);
                }
            }
            search_from = start + scheme.len();
        }
    }
    None
}

fn strip_external_link_schemes(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let byte_idx = chars[i].0;
        let try_strip = |marker: &str| -> Option<usize> {
            if !lower[byte_idx..].starts_with(marker) {
                return None;
            }
            let boundary_ok = i == 0
                || (!chars[i - 1].1.is_alphanumeric()
                    && chars[i - 1].1 != '.'
                    && chars[i - 1].1 != '/');
            if boundary_ok {
                Some(i + marker.chars().count())
            } else {
                None
            }
        };
        if let Some(next) = try_strip("https://")
            .or_else(|| try_strip("http://"))
            .or_else(|| try_strip("www."))
        {
            i = next;
            continue;
        }
        out.push(chars[i].1);
        i += 1;
    }
    out
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Dispatch by target name (zcode `--target` compatibility).
///
/// Public / anonymize targets: `--brand <path>` is mandatory — `None` / empty
/// is `REFUSED_BRAND` (no soft default to `brands/fourth-watch.brand.toml`).
/// Anonymize court still fires first when `--public-anonymize` is absent.
///
/// Non-public / unknown targets: do not invent a soft brand path; refuse the
/// unknown target (or load an *explicit* `--brand` only when the caller
/// supplied one for a future private rail).
pub fn publish_target(
    target: &str,
    brand_path: Option<&str>,
    public_anonymize: bool,
    input_path: Option<&str>,
    subject: Option<&str>,
    out_dir: Option<&str>,
) -> Result<Value, PublishWireError> {
    let is_public = target_requires_anonymize(target);
    if is_public {
        // Pre-load gate: public targets always need the CLI flag before brand I/O.
        require_public_anonymize(public_anonymize).map_err(PublishWireError::Anonymize)?;
    }

    let brand = if is_public {
        let path = require_brand_path(brand_path)?;
        let brand = load_brand(path)?;
        // Brand floor may still require anonymize even after the target gate.
        brand
            .enforce_public_anonymize(public_anonymize)
            .map_err(PublishWireError::Anonymize)?;
        Some(brand)
    } else if let Some(path) = brand_path.map(str::trim).filter(|p| !p.is_empty()) {
        // Private/non-public: explicit `--brand` only — never soft-default.
        let brand = load_brand(path)?;
        brand
            .enforce_public_anonymize(public_anonymize)
            .map_err(PublishWireError::Anonymize)?;
        Some(brand)
    } else {
        None
    };
    let out = out_dir.map(Path::new);

    match target {
        "linkedin-carousel" | "carousel" => {
            let brand = brand.expect("public target always loads brand");
            let input = read_carousel_input(input_path)?;
            publish_branded_carousel(&input, &brand, public_anonymize, out)
        }
        "linkedin-post" | "post" => {
            let brand = brand.expect("public target always loads brand");
            if let Some(path) = input_path {
                let input = read_json_input::<PostInput>(path)?;
                publish_branded_post(&input, &brand, public_anonymize)
            } else if let Some(subj) = subject {
                publish_subject_post(subj, &brand, public_anonymize)
            } else {
                Err(PublishWireError::Input(
                    "post requires --input-path or --subject".into(),
                ))
            }
        }
        "linkedin-newsletter" | "newsletter" => {
            let brand = brand.expect("public target always loads brand");
            let input = read_json_input::<NewsletterInput>(input_path.ok_or_else(|| {
                PublishWireError::Input("newsletter requires --input-path".into())
            })?)?;
            publish_branded_newsletter(&input, &brand, public_anonymize)
        }
        "gatekeeper-pack" => {
            let brand = brand.expect("public target always loads brand");
            // Gatekeeper pack: anonymize seal over a JSON pack file (or empty envelope).
            let raw = match input_path {
                Some(p) => std::fs::read_to_string(p).map_err(|e| {
                    PublishWireError::Input(format!("unreadable input {p}: {e}"))
                })?,
                None => "{}".to_string(),
            };
            let value: Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({ "raw": raw }));
            let mut envelope = json!({
                "ok": true,
                "kind": "gatekeeper-pack",
                "brand_id": brand.brand.id,
                "pack": value,
            });
            let (sealed, court) = seal_public_artifact(envelope, &brand.redaction_policy())?;
            envelope = sealed;
            envelope["redaction_court"] = court;
            Ok(envelope)
        }
        other => Err(PublishWireError::Input(format!(
            "unknown publish target '{other}'"
        ))),
    }
}

fn inject_brand_theme(
    typst: &str,
    brand: &BrandManifest,
    palette: &super::brand::TypstPalette,
) -> String {
    let header = format!(
        "// brand: {} ({})\n// tagline: {}\n#let brand_accent = rgb(\"{}\")\n#let brand_bg = rgb(\"{}\")\n#let brand_text = rgb(\"{}\")\n",
        brand.brand.name,
        brand.brand.id,
        brand.brand.tagline.as_deref().unwrap_or(""),
        ensure_hash(&palette.accent),
        ensure_hash(&palette.bg),
        ensure_hash(&palette.text),
    );
    // Insert after the first generated comment line if present.
    if let Some(pos) = typst.find('\n') {
        let mut out = String::new();
        out.push_str(&typst[..=pos]);
        out.push_str(&header);
        out.push_str(&typst[pos + 1..]);
        out
    } else {
        format!("{header}{typst}")
    }
}

fn read_carousel_input(input_path: Option<&str>) -> Result<CarouselInput, PublishWireError> {
    match input_path {
        Some(path) => read_json_input(path),
        // Demo carousel *copy* only — not a soft `--brand` path default.
        None => Ok(default_carousel_input()),
    }
}

fn default_carousel_input() -> CarouselInput {
    let slides: Vec<SlideInput> = (0..8)
        .map(|i| {
            if i == 0 {
                SlideInput {
                    headline: "Stop leaking unverified claims".into(),
                    body: "Pinned CASTLE probes refuse hollow demos before auditors arrive."
                        .into(),
                }
            } else {
                SlideInput {
                    headline: format!("Seal step {}", i + 1),
                    body: "Measure the receipt. Own the cell. Guard the drop path.".into(),
                }
            }
        })
        .collect();
    CarouselInput {
        lease_id: "lease-public".into(),
        body_pt: 24.0,
        safe_margin_px: 72,
        slides,
    }
}

fn read_json_input<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, PublishWireError> {
    let bytes = std::fs::read_to_string(path)
        .map_err(|e| PublishWireError::Input(format!("unreadable input {path}: {e}")))?;
    serde_json::from_str(&bytes)
        .map_err(|e| PublishWireError::Input(format!("invalid input JSON {path}: {e}")))
}

fn count_words_approx(text: &str) -> usize {
    text.split_whitespace().count()
}

fn ensure_hash(color: &str) -> String {
    if color.starts_with('#') {
        color.to_string()
    } else {
        format!("#{color}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brand_toml() -> BrandManifest {
        BrandManifest::parse_toml(
            r#"
[brand]
id = "fourth-watch"
name = "Fourth Watch AI Cybersecurity"
tagline = "Sentinel Steward Shepherd"

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
private_client_names = ["TotallyPrivateClient"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn refuses_without_brand_path() {
        let err = require_brand_path(None).unwrap_err();
        let j = err.to_json();
        assert_eq!(j["ok"], false);
        assert_eq!(j["class"], "refused");
        assert_eq!(j["code"], "REFUSED_BRAND");
        assert!(
            j["detail"]
                .as_str()
                .unwrap_or("")
                .contains("requires --brand"),
            "detail must name --brand: {j}"
        );
        let empty = require_brand_path(Some("  ")).unwrap_err();
        assert_eq!(empty.to_json()["code"], "REFUSED_BRAND");
        assert_eq!(require_brand_path(Some("brands/x.brand.toml")).unwrap(), "brands/x.brand.toml");
    }

    #[test]
    fn refuses_without_anonymize() {
        let brand = brand_toml();
        let err = publish_subject_post("t", &brand, false).unwrap_err();
        let j = err.to_json();
        assert_eq!(j["ok"], false);
        assert_eq!(j["class"], "refused");
        assert!(j["code"].as_str().unwrap().contains("ANONYMIZE"));
    }

    #[test]
    fn publish_target_anonymize_before_brand() {
        // Missing both flags: anonymize court wins (F-ANON-2 contract).
        let err = publish_target("linkedin-post", None, false, None, Some("t"), None)
            .unwrap_err();
        let j = err.to_json();
        assert!(j["code"].as_str().unwrap().contains("ANONYMIZE"));

        // Anonymize present, brand omitted: soft-default closed.
        let err = publish_target("linkedin-post", None, true, None, Some("t"), None)
            .unwrap_err();
        let j = err.to_json();
        assert_eq!(j["code"], "REFUSED_BRAND");
        assert!(j["detail"].as_str().unwrap().contains("no soft default"));
    }

    #[test]
    fn non_public_unknown_target_without_brand_is_input_not_brand() {
        // Private/non-public: omitting --brand must not soft-default, and must
        // not pretend the miss was a public REFUSED_BRAND court.
        let err = publish_target("private-draft", None, false, None, None, None).unwrap_err();
        let j = err.to_json();
        assert_eq!(j["code"], "REFUSED_INPUT");
        assert!(
            j["detail"]
                .as_str()
                .unwrap_or("")
                .contains("unknown publish target"),
            "non-public miss names the target, not brand: {j}"
        );
    }

    #[test]
    fn publish_target_refuses_bad_brand_path() {
        let err = publish_target(
            "linkedin-post",
            Some("/no/such/brand.toml"),
            true,
            None,
            Some("t"),
            None,
        )
        .unwrap_err();
        let j = err.to_json();
        assert_eq!(j["code"], "REFUSED_BRAND");
        assert!(j["detail"].as_str().unwrap().contains("unreadable"));
    }

    #[test]
    fn subject_post_refuses_users_paths_via_court() {
        let brand = brand_toml();
        let err = publish_subject_post("incident at /Users/sac/private/client", &brand, true)
            .expect_err("home path must refuse at RedactionCourt seal");
        let j = err.to_json();
        assert_eq!(j["ok"], false);
        assert_eq!(j["code"], "REFUSED_REDACTION");
        let findings = j["findings"].as_array().expect("findings");
        assert!(
            findings
                .iter()
                .any(|f| f["rule"] == "PRIVATE_HOME_PATH"),
            "expected PRIVATE_HOME_PATH, got {j}"
        );
        let inspected = format!("{err:?}");
        assert!(
            !inspected.contains("/Users/sac"),
            "refusal must not echo home path: {inspected}"
        );
    }

    #[test]
    fn seal_admits_clean_artifact_with_court_receipt() {
        let brand = brand_toml();
        let policy = brand.redaction_policy();
        let (sealed, court) = seal_public_artifact(
            json!({"ok": true, "body": "Seal the ledger then publish the receipt."}),
            &policy,
        )
        .expect("clean artifact admits");
        assert_eq!(sealed["ok"], true);
        assert_eq!(court["collaborator"], "RedactionCourt");
        assert_eq!(court["standing"], "ALIVE");
        assert_eq!(court["code"], "ADMITTED");
    }

    #[test]
    fn seal_refuses_default_denylist_when_brand_names_empty() {
        let brand = BrandManifest::parse_toml(
            r#"
[brand]
id = "fourth-watch"
name = "Fourth Watch AI Cybersecurity"

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
        assert!(
            policy
                .private_names
                .iter()
                .any(|n| n.eq_ignore_ascii_case("Courtroom.ai")),
            "empty brand denylist must load defaults: {:?}",
            policy.private_names
        );
        let err = seal_public_artifact(
            json!({"body": "Pilot update from Courtroom.ai this week."}),
            &policy,
        )
        .expect_err("default denylist must refuse");
        let j = err.to_json();
        assert_eq!(j["code"], "REFUSED_REDACTION");
        assert!(j["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "PRIVATE_CLIENT_NAME"));
    }

    #[test]
    fn seal_refuses_calendar_bearer_and_external_floors() {
        let brand = brand_toml();
        let policy = brand.redaction_policy();

        let cal = seal_public_artifact(
            json!({"body": "Book via https://calendly.com/fourth-watch/intro"}),
            &policy,
        )
        .expect_err("calendar link must refuse");
        assert!(cal.to_json()["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "CALENDAR_LINK" || f["rule"] == "EXTERNAL_LINK"));

        let bearer = seal_public_artifact(
            json!({"auth": "Authorization: Bearer sk-live-abcdefghijklmnopqrstuvwxyz"}),
            &policy,
        )
        .expect_err("bearer residual must refuse");
        assert!(bearer.to_json()["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "BEARER_TOKEN"));

        let ext = seal_public_artifact(
            json!({"body": "See https://evil.example/leak for details"}),
            &policy,
        )
        .expect_err("disallowed external link must refuse");
        assert!(ext.to_json()["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "EXTERNAL_LINK"));
    }

    #[test]
    fn subject_post_admits_clean_and_records_court() {
        let brand = brand_toml();
        let out = publish_subject_post("t", &brand, true).expect("clean subject admits");
        assert_eq!(out["ok"], true);
        assert_eq!(out["redaction_court"]["collaborator"], "RedactionCourt");
        assert_eq!(out["redaction_court"]["standing"], "ALIVE");
    }

    /// F-ANON-3 against the real twin: https Featured URL must not poison body.
    #[test]
    fn real_brand_subject_post_admits_without_body_https() {
        let brand = BrandManifest::load_path("brands/fourth-watch.brand.toml")
            .expect("castle brands twin");
        let out = publish_subject_post("t", &brand, true).expect("real brand must admit");
        assert_eq!(out["ok"], true);
        assert_eq!(out["redaction_court"]["collaborator"], "RedactionCourt");
        let body = out["body"].as_str().unwrap_or("");
        let full = out["full_text"].as_str().unwrap_or("");
        for surface in [body, full] {
            let lower = surface.to_lowercase();
            assert!(
                !lower.contains("https://")
                    && !lower.contains("http://")
                    && !lower.contains("www."),
                "body must stay link-marker free: {surface}"
            );
        }
        let first = out["first_comment"].as_str().unwrap_or("");
        assert!(
            first.contains("https://cos.fnox.io/drop/lease-public"),
            "Featured URL belongs in first_comment: {first}"
        );
    }

    /// Legacy brands that still embed https in required_cta must still admit:
    /// body schemes are stripped; absolute URL rides cta_link → first_comment.
    #[test]
    fn legacy_https_required_cta_anonymizes_into_first_comment() {
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
required_cta = "Drop a read-only snapshot at https://cos.fnox.io/drop/:lease_id for a board PDF."

[messaging.cta]
featured_url_template = "https://cos.fnox.io/drop/:lease_id"

[redaction_court]
forbid_private_client_names = true
forbid_local_user_paths = true
forbid_bearer_tokens = true
forbid_calendar_links = true
forbid_external_links_in_post_body = true
require_public_anonymize = true
"#,
        )
        .unwrap();
        let out = publish_subject_post("t", &brand, true).expect("legacy https CTA must admit");
        assert_eq!(out["ok"], true);
        let body = out["body"].as_str().unwrap_or("");
        let lower = body.to_ascii_lowercase();
        assert!(
            !lower.contains("https://") && !lower.contains("http://") && !lower.contains("www."),
            "legacy required_cta https must not survive into body: {body}"
        );
        assert!(
            body.contains("cos.fnox.io"),
            "host-of-record prose must remain after scheme strip: {body}"
        );
        let first = out["first_comment"].as_str().unwrap_or("");
        assert!(
            first.contains("https://cos.fnox.io/drop/lease-public"),
            "Featured URL must still ship in first_comment: {first}"
        );
    }
}
