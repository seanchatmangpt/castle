//! LinkedIn deterministic publishers + the Copy & Framing Lint Court
//! (CASTLE-SPEC-26.11.20).
//!
//! Three pure publishers, one court:
//!
//! - [`publish_carousel`] — 8–10 slides rendered as a 1080x1350 Typst deck,
//!   body text >= 24pt, 50px safe margin, strictly < 60 words per slide, an
//!   action-first hazard hook on Slide 1, the 5-Question Executive Check,
//!   and exactly one `/drop/:lease_id` CTA on the final slide.
//! - [`publish_post`] — 150–250 words, hook < 140 chars, zero external
//!   links in the body; every link surfaces only in the first-comment block.
//! - [`publish_newsletter`] — Markdown + HTML projection of one sectioned
//!   document.
//!
//! The Copy & Framing Lint Court ([`lint_copy`] plus the per-publisher
//! rule sets [`lint_carousel`] / [`lint_post`] / [`lint_newsletter`])
//! refuses soft/subordinate scripts ("Would it be crazy", "totally fine if
//! not", ...) with typed `REFUSED_LINT_*` findings before any artifact is
//! minted. A refusal carries `broken_term = "mu_on_O"`: publishing from copy
//! the court has not admitted is manufacture from unadmitted input.
//!
//! Determinism law: no I/O, no wall clock, no randomness. Every byte of an
//! artifact is a pure function of its input struct, and the artifact digest
//! is `blake3(canonical JSON payload)` — identical inputs replay
//! byte-identical artifacts (asserted by test).
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::json;

// ---------------------------------------------------------------------------
// Law constants (CASTLE-SPEC-26.11.20)
// ---------------------------------------------------------------------------

/// Carousel canvas width in px (LinkedIn portrait carousel).
pub const CAROUSEL_WIDTH_PX: u32 = 1080;
/// Carousel canvas height in px (LinkedIn portrait carousel).
pub const CAROUSEL_HEIGHT_PX: u32 = 1350;
/// Minimum body font size in pt on the carousel canvas.
pub const MIN_BODY_PT: f64 = 24.0;
/// Safe margin in px that slide copy must never cross.
pub const SAFE_MARGIN_PX: u32 = 50;
/// Per-slide word budget: strictly fewer than this many words.
pub const MAX_SLIDE_WORDS: usize = 60;
/// Minimum slides per carousel deck (inclusive).
pub const MIN_SLIDES: usize = 8;
/// Maximum slides per carousel deck (inclusive).
pub const MAX_SLIDES: usize = 10;
/// Minimum words per post (inclusive).
pub const POST_MIN_WORDS: usize = 150;
/// Maximum words per post (inclusive).
pub const POST_MAX_WORDS: usize = 250;
/// Hook length budget: strictly fewer than this many chars.
pub const MAX_HOOK_CHARS: usize = 140;
/// Font size of the injected CTA line in the Typst deck.
pub const CTA_TEXT_PT: f64 = 28.0;
/// Chatman failure-taxonomy term carried by every refusal of this module.
pub const BROKEN_TERM: &str = "mu_on_O";

// ---------------------------------------------------------------------------
// Typed refusal + rule ids
// ---------------------------------------------------------------------------

pub const REFUSED_LINT_CAROUSEL: &str = "REFUSED_LINT_CAROUSEL";
pub const REFUSED_LINT_POST: &str = "REFUSED_LINT_POST";
pub const REFUSED_LINT_NEWSLETTER: &str = "REFUSED_LINT_NEWSLETTER";

pub const RULE_SOFT_SCRIPT: &str = "COPY_SOFT_SCRIPT";
pub const RULE_CAROUSEL_SLIDE_COUNT: &str = "CAROUSEL_SLIDE_COUNT";
pub const RULE_CAROUSEL_HEADLINE_REQUIRED: &str = "CAROUSEL_HEADLINE_REQUIRED";
pub const RULE_CAROUSEL_SLIDE_WORD_BUDGET: &str = "CAROUSEL_SLIDE_WORD_BUDGET";
pub const RULE_CAROUSEL_BODY_PT: &str = "CAROUSEL_BODY_PT";
pub const RULE_CAROUSEL_SAFE_MARGIN: &str = "CAROUSEL_SAFE_MARGIN";
pub const RULE_CAROUSEL_HOOK_ACTION_FIRST: &str = "CAROUSEL_HOOK_ACTION_FIRST";
pub const RULE_CAROUSEL_HOOK_HAZARD: &str = "CAROUSEL_HOOK_HAZARD";
pub const RULE_CAROUSEL_LEASE_ID: &str = "CAROUSEL_LEASE_ID";
pub const RULE_CAROUSEL_CTA_UNIQUE: &str = "CAROUSEL_CTA_UNIQUE";
pub const RULE_CAROUSEL_EXECUTIVE_CHECK: &str = "CAROUSEL_EXECUTIVE_CHECK";
pub const RULE_POST_HOOK_REQUIRED: &str = "POST_HOOK_REQUIRED";
pub const RULE_POST_HOOK_SINGLE_LINE: &str = "POST_HOOK_SINGLE_LINE";
pub const RULE_POST_HOOK_LENGTH: &str = "POST_HOOK_LENGTH";
pub const RULE_POST_WORD_BUDGET: &str = "POST_WORD_BUDGET";
pub const RULE_POST_EXTERNAL_LINK: &str = "POST_EXTERNAL_LINK";
pub const RULE_POST_FIRST_COMMENT: &str = "POST_FIRST_COMMENT";
pub const RULE_NEWSLETTER_TITLE_REQUIRED: &str = "NEWSLETTER_TITLE_REQUIRED";
pub const RULE_NEWSLETTER_SECTIONS_REQUIRED: &str = "NEWSLETTER_SECTIONS_REQUIRED";
pub const RULE_NEWSLETTER_SECTION_SHAPE: &str = "NEWSLETTER_SECTION_SHAPE";

// ---------------------------------------------------------------------------
// Lexicons
// ---------------------------------------------------------------------------

/// Soft/subordinate scripts. Any occurrence (case-insensitive, on word
/// boundaries, whitespace-normalized) is a refusal: the court publishes
/// action, not apology.
pub const SOFT_SCRIPT_PHRASES: &[&str] = &[
    "would it be crazy",
    "totally fine if not",
    "no worries if not",
    "sorry to bother",
    "sorry in advance",
    "sorry if",
    "if you don't mind",
    "hope that's okay",
    "hope this is okay",
    "if it's not too much trouble",
    "just wondering",
    "i was wondering if",
    "when you get a chance",
    "do you have a sec",
    "quick favor",
    "just checking in",
    "correct me if i'm wrong",
    "i could be wrong",
    "just a thought",
    "not sure if",
    "i think maybe",
    "it might be nice",
    "if that makes sense",
    "does that make sense",
    "hope this helps",
    "feel free to ignore",
    "gently nudging",
    "just following up",
];

/// Imperative lead verbs admitting a hook as action-first. The first word of
/// the Slide 1 headline must be one of these (punctuation stripped,
/// case-insensitive).
pub const ACTION_LEXICON: &[&str] = &[
    "act", "always", "audit", "ban", "block", "break", "build", "burn", "close", "count", "cut",
    "dead", "delete", "deploy", "demand", "drop", "end", "erase", "expire", "fix", "freeze",
    "falsify", "gate", "guard", "halt", "kill", "measure", "migrate", "move", "never", "open",
    "own", "patch", "preserve", "purge", "rebuild", "refuse", "remove", "replace", "restore",
    "revoke", "rewrite", "rollback", "run", "seal", "ship", "slash", "start", "stop", "tear",
    "test", "track", "trim", "unblock", "verify", "write",
];

/// Hazard stems. A hook names the hazard: some headline word must start with
/// one of these stems (so plurals and inflections count: "leaking", "losses").
pub const HAZARD_STEMS: &[&str] = &[
    "mistake", "risk", "failure", "fail", "breach", "loss", "lose", "leak", "bug", "outage",
    "debt", "hazard", "danger", "wrong", "incident", "attack", "threat", "expos", "regress",
    "drift", "corrupt", "collaps", "downtime", "exploit", "vulnerab", "silent", "hidden", "dead",
    "dying", "broke", "broken", "bleed", "burn", "cripp", "doom", "toxic",
];

// ---------------------------------------------------------------------------
// Court types
// ---------------------------------------------------------------------------

/// One typed lint-court finding.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LintFinding {
    /// Stable rule id, e.g. [`RULE_SOFT_SCRIPT`].
    pub rule: &'static str,
    /// Human-readable evidence naming the violated law.
    pub detail: String,
}

/// A typed refusal to publish. The candidate copy is unadmitted input;
/// manufacturing from it would be `mu_on_O`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PublishRefusal {
    /// `REFUSED_LINT_*` code naming the refused publisher.
    pub code: &'static str,
    /// Chatman failure-taxonomy term ([`BROKEN_TERM`]).
    pub broken_term: &'static str,
    /// Every rule the candidate failed; never empty.
    pub findings: Vec<LintFinding>,
}

impl std::fmt::Display for PublishRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}[{}]: {} finding(s)",
            self.code,
            self.broken_term,
            self.findings.len()
        )
    }
}

impl std::error::Error for PublishRefusal {}

/// One question of the 5-Question Executive Check.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutiveQuestion {
    /// The question, verbatim.
    pub question: &'static str,
    /// Whether the published deck answers it with a witnessed pass.
    pub pass: bool,
}

/// The 5-Question Executive Check carried on every carousel artifact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExecutiveCheck {
    /// Exactly five questions, in publication order.
    pub questions: [ExecutiveQuestion; 5],
}

impl ExecutiveCheck {
    /// True only when all five questions pass.
    pub fn all_pass(&self) -> bool {
        self.questions.iter().all(|q| q.pass)
    }
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// One carousel slide: headline + body copy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideInput {
    /// Slide headline (Slide 1 doubles as the action-first hazard hook).
    pub headline: String,
    /// Slide body copy; together with the headline it must stay strictly
    /// under [`MAX_SLIDE_WORDS`] words.
    pub body: String,
}

/// Input to [`publish_carousel`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarouselInput {
    /// Lease id minted into the `/drop/:lease_id` CTA.
    pub lease_id: String,
    /// Body font size in pt; must be >= [`MIN_BODY_PT`] and finite.
    pub body_pt: f64,
    /// Safe margin in px; must be >= [`SAFE_MARGIN_PX`].
    pub safe_margin_px: u32,
    /// 8–10 slides.
    pub slides: Vec<SlideInput>,
}

/// Input to [`publish_post`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostInput {
    /// Single-line hook; strictly < [`MAX_HOOK_CHARS`] chars.
    pub hook: String,
    /// Post body; together with the hook it must land in
    /// `POST_MIN_WORDS..=POST_MAX_WORDS` words and carry zero external links.
    pub body: String,
    /// Optional lease id surfaced as `/drop/:lease_id` in the first comment.
    pub lease_id: Option<String>,
    /// Optional external link; admissible only inside the first-comment block.
    pub cta_link: Option<String>,
}

/// One newsletter section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsletterSection {
    /// Section heading.
    pub heading: String,
    /// At least one non-empty paragraph.
    pub paragraphs: Vec<String>,
    /// Optional bullets.
    pub bullets: Vec<String>,
}

/// Input to [`publish_newsletter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsletterInput {
    /// Newsletter title.
    pub title: String,
    /// Optional subtitle.
    pub subtitle: Option<String>,
    /// At least one well-formed section.
    pub sections: Vec<NewsletterSection>,
}

// ---------------------------------------------------------------------------
// Artifacts
// ---------------------------------------------------------------------------

/// Deterministic carousel artifact: Typst source + receipt identity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CarouselArtifact {
    pub slide_count: usize,
    pub width_px: u32,
    pub height_px: u32,
    pub safe_margin_px: u32,
    pub body_pt: f64,
    pub lease_id: String,
    /// The `/drop/:lease_id` CTA path injected exactly once.
    pub cta: String,
    /// Full Typst source of the 1080x1350 deck.
    pub typst: String,
    /// The 5-Question Executive Check, all witnessed passes.
    pub executive_check: ExecutiveCheck,
    /// `blake3(canonical JSON payload)` — the replay identity.
    pub digest: String,
}

/// Deterministic post artifact: text + first-comment block + receipt identity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PostArtifact {
    pub hook: String,
    pub body: String,
    /// `hook + "\n\n" + body` — the published surface, link-free by law.
    pub full_text: String,
    /// The first-comment block where every link lives.
    pub first_comment: String,
    pub word_count: usize,
    pub hook_chars: usize,
    pub digest: String,
}

/// Deterministic newsletter artifact: Markdown + HTML + receipt identity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NewsletterArtifact {
    pub title: String,
    pub markdown: String,
    pub html: String,
    pub digest: String,
}

// ---------------------------------------------------------------------------
// Scanning helpers (deterministic, allocation-light)
// ---------------------------------------------------------------------------

/// Lowercase, map curly apostrophes to ASCII, collapse whitespace runs.
fn normalize_text(text: &str) -> String {
    let replaced = text.replace('’', "'");
    replaced
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Count whitespace-separated words.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Word-boundary-aware substring search over already-normalized text.
fn contains_phrase_with_boundaries(hay_norm: &str, needle_norm: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(rel) = hay_norm[search_from..].find(needle_norm) {
        let start = search_from + rel;
        let end = start + needle_norm.len();
        let before_blocks = hay_norm[..start]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric());
        let after_blocks = hay_norm[end..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric());
        if !before_blocks && !after_blocks {
            return true;
        }
        search_from = end;
    }
    false
}

/// Every soft/subordinate script present in `text`, in lexicon order.
fn matched_soft_phrases(text: &str) -> Vec<&'static str> {
    let hay = normalize_text(text);
    SOFT_SCRIPT_PHRASES
        .iter()
        .copied()
        .filter(|phrase| contains_phrase_with_boundaries(&hay, &normalize_text(phrase)))
        .collect()
}

/// First external-link marker (`https://`, `http://`, `www.`) on a word
/// boundary, if any.
fn external_link_marker(text: &str) -> Option<&'static str> {
    const MARKERS: [&str; 3] = ["https://", "http://", "www."];
    let lower = text.to_lowercase();
    for marker in MARKERS {
        let mut search_from = 0usize;
        while let Some(rel) = lower[search_from..].find(marker) {
            let abs = search_from + rel;
            let boundary_free = !lower[..abs]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '.');
            if boundary_free {
                return Some(marker);
            }
            search_from = abs + marker.len();
        }
    }
    None
}

/// Lowercased, punctuation-stripped first word of `text`.
fn first_word_lower(text: &str) -> String {
    text.split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Action-first: the hook opens on an imperative verb.
fn is_action_first(headline: &str) -> bool {
    ACTION_LEXICON.contains(&first_word_lower(headline).as_str())
}

/// Hazard-carrying: some headline word starts with a hazard stem.
fn has_hazard_cue(headline: &str) -> bool {
    headline.split_whitespace().any(|word| {
        let stripped = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        HAZARD_STEMS.iter().any(|stem| stripped.starts_with(stem))
    })
}

/// Shape law for a lease id minted into a `/drop/:lease_id` path.
fn lease_id_problem(lease_id: &str) -> Option<&'static str> {
    if lease_id.trim().is_empty() {
        return Some("lease_id is empty");
    }
    if !lease_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Some("lease_id must be ASCII alphanumeric with '-' or '_' only");
    }
    None
}

/// `blake3` over the canonical JSON payload (serde_json sorts object keys).
fn blake3_digest(payload: &serde_json::Value) -> String {
    blake3::hash(payload.to_string().as_bytes()).to_hex().to_string()
}

fn refusal(code: &'static str, findings: Vec<LintFinding>) -> PublishRefusal {
    PublishRefusal {
        code,
        broken_term: BROKEN_TERM,
        findings,
    }
}

// ---------------------------------------------------------------------------
// The Copy & Framing Lint Court
// ---------------------------------------------------------------------------

/// Core copy court: refuses soft/subordinate scripts anywhere in `text`.
pub fn lint_copy(text: &str) -> Vec<LintFinding> {
    matched_soft_phrases(text)
        .into_iter()
        .map(|phrase| LintFinding {
            rule: RULE_SOFT_SCRIPT,
            detail: format!("soft/subordinate script refused: {phrase:?}"),
        })
        .collect()
}

/// Framing court for the carousel: slide count, word budget, canvas law
/// (24pt body, 50px safe margin), Slide 1 action-first hazard hook, lease
/// shape, CTA uniqueness, and the copy court over every slide.
pub fn lint_carousel(input: &CarouselInput) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    if input.slides.len() < MIN_SLIDES || input.slides.len() > MAX_SLIDES {
        findings.push(LintFinding {
            rule: RULE_CAROUSEL_SLIDE_COUNT,
            detail: format!(
                "deck carries {} slides; the law is {}-{}",
                input.slides.len(),
                MIN_SLIDES,
                MAX_SLIDES
            ),
        });
    }

    for (idx, slide) in input.slides.iter().enumerate() {
        if slide.headline.trim().is_empty() {
            findings.push(LintFinding {
                rule: RULE_CAROUSEL_HEADLINE_REQUIRED,
                detail: format!("slide {} has an empty headline", idx + 1),
            });
        }
        let words = count_words(&slide.headline) + count_words(&slide.body);
        if words >= MAX_SLIDE_WORDS {
            findings.push(LintFinding {
                rule: RULE_CAROUSEL_SLIDE_WORD_BUDGET,
                detail: format!(
                    "slide {} carries {} words; budget is strictly < {}",
                    idx + 1,
                    words,
                    MAX_SLIDE_WORDS
                ),
            });
        }
    }

    if !input.body_pt.is_finite() || input.body_pt < MIN_BODY_PT {
        findings.push(LintFinding {
            rule: RULE_CAROUSEL_BODY_PT,
            detail: format!("body font is {}pt; the floor is {MIN_BODY_PT}pt", input.body_pt),
        });
    }

    if input.safe_margin_px < SAFE_MARGIN_PX {
        findings.push(LintFinding {
            rule: RULE_CAROUSEL_SAFE_MARGIN,
            detail: format!(
                "safe margin is {}px; the floor is {SAFE_MARGIN_PX}px",
                input.safe_margin_px
            ),
        });
    }

    if let Some(first) = input.slides.first() {
        if !first.headline.trim().is_empty() {
            if !is_action_first(&first.headline) {
                findings.push(LintFinding {
                    rule: RULE_CAROUSEL_HOOK_ACTION_FIRST,
                    detail: format!(
                        "Slide 1 must open on an imperative verb; first word is {:?}",
                        first_word_lower(&first.headline)
                    ),
                });
            }
            if !has_hazard_cue(&first.headline) {
                findings.push(LintFinding {
                    rule: RULE_CAROUSEL_HOOK_HAZARD,
                    detail: "Slide 1 hook must name the hazard it stops".to_string(),
                });
            }
        }
    }

    if let Some(problem) = lease_id_problem(&input.lease_id) {
        findings.push(LintFinding {
            rule: RULE_CAROUSEL_LEASE_ID,
            detail: problem.to_string(),
        });
    } else {
        let cta = drop_cta(&input.lease_id);
        if input
            .slides
            .iter()
            .any(|s| s.headline.contains(&cta) || s.body.contains(&cta))
        {
            findings.push(LintFinding {
                rule: RULE_CAROUSEL_CTA_UNIQUE,
                detail: "the /drop CTA appears in slide copy; the template injects it exactly once"
                    .to_string(),
            });
        }
    }

    for (idx, slide) in input.slides.iter().enumerate() {
        for finding in lint_copy(&format!("{}\n{}", slide.headline, slide.body)) {
            findings.push(LintFinding {
                rule: finding.rule,
                detail: format!("slide {}: {}", idx + 1, finding.detail),
            });
        }
    }

    findings
}

/// Framing court for the post: hook presence/shape/length, word budget,
/// zero external links in the body, first-comment placement, and the copy
/// court over the full text.
pub fn lint_post(input: &PostInput) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    if input.hook.trim().is_empty() {
        findings.push(LintFinding {
            rule: RULE_POST_HOOK_REQUIRED,
            detail: "post hook is empty".to_string(),
        });
    }
    if input.hook.contains('\n') {
        findings.push(LintFinding {
            rule: RULE_POST_HOOK_SINGLE_LINE,
            detail: "post hook must be a single line".to_string(),
        });
    }

    let hook_chars = input.hook.chars().count();
    if hook_chars >= MAX_HOOK_CHARS {
        findings.push(LintFinding {
            rule: RULE_POST_HOOK_LENGTH,
            detail: format!(
                "hook is {hook_chars} chars; the ceiling is strictly < {MAX_HOOK_CHARS}"
            ),
        });
    }

    let full_text = post_full_text(input);
    let words = count_words(&full_text);
    if !(POST_MIN_WORDS..=POST_MAX_WORDS).contains(&words) {
        findings.push(LintFinding {
            rule: RULE_POST_WORD_BUDGET,
            detail: format!(
                "post carries {words} words; the law is {POST_MIN_WORDS}-{POST_MAX_WORDS}"
            ),
        });
    }

    if let Some(marker) = external_link_marker(&full_text) {
        findings.push(LintFinding {
            rule: RULE_POST_EXTERNAL_LINK,
            detail: format!(
                "external link marker {marker:?} in the body; links surface only in the first comment"
            ),
        });
    }

    if let Some(link) = input.cta_link.as_deref() {
        if !link.trim().is_empty() && !build_first_comment(input).contains(link) {
            findings.push(LintFinding {
                rule: RULE_POST_FIRST_COMMENT,
                detail: "cta_link missing from the first-comment block".to_string(),
            });
        }
    }

    findings.extend(lint_copy(&full_text));
    findings
}

/// Court for the newsletter: title/section shape and the copy court over the
/// full document.
pub fn lint_newsletter(input: &NewsletterInput) -> Vec<LintFinding> {
    let mut findings = Vec::new();

    if input.title.trim().is_empty() {
        findings.push(LintFinding {
            rule: RULE_NEWSLETTER_TITLE_REQUIRED,
            detail: "newsletter title is empty".to_string(),
        });
    }
    if input.sections.is_empty() {
        findings.push(LintFinding {
            rule: RULE_NEWSLETTER_SECTIONS_REQUIRED,
            detail: "newsletter carries no sections".to_string(),
        });
    }

    for (idx, section) in input.sections.iter().enumerate() {
        if section.heading.trim().is_empty() {
            findings.push(LintFinding {
                rule: RULE_NEWSLETTER_SECTION_SHAPE,
                detail: format!("section {} has an empty heading", idx + 1),
            });
        }
        if section.paragraphs.is_empty() {
            findings.push(LintFinding {
                rule: RULE_NEWSLETTER_SECTION_SHAPE,
                detail: format!("section {} carries no paragraphs", idx + 1),
            });
        }
        for (p_idx, paragraph) in section.paragraphs.iter().enumerate() {
            if paragraph.trim().is_empty() {
                findings.push(LintFinding {
                    rule: RULE_NEWSLETTER_SECTION_SHAPE,
                    detail: format!("section {} paragraph {} is empty", idx + 1, p_idx + 1),
                });
            }
        }
    }

    let mut all = String::new();
    all.push_str(&input.title);
    all.push('\n');
    if let Some(subtitle) = input.subtitle.as_deref() {
        all.push_str(subtitle);
        all.push('\n');
    }
    for section in &input.sections {
        all.push_str(&section.heading);
        all.push('\n');
        for paragraph in &section.paragraphs {
            all.push_str(paragraph);
            all.push('\n');
        }
        for bullet in &section.bullets {
            all.push_str(bullet);
            all.push('\n');
        }
    }
    findings.extend(lint_copy(&all));
    findings
}

// ---------------------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------------------

/// The `/drop/:lease_id` CTA path.
pub fn drop_cta(lease_id: &str) -> String {
    format!("/drop/{lease_id}")
}

/// Escape Typst markup-active characters; escape a line-leading `=`/`-`/`+`/`/`
/// so slide copy can never mint headings, lists, or terms of its own.
fn escape_typst(text: &str) -> String {
    fn escape_line(line: &str) -> String {
        let mut out = String::with_capacity(line.len() + 8);
        let mut leader_done = false;
        for c in line.chars() {
            if !leader_done {
                if c.is_whitespace() {
                    out.push(c);
                    continue;
                }
                if matches!(c, '=' | '-' | '+' | '/') {
                    out.push('\\');
                }
                leader_done = true;
            }
            match c {
                '\\' | '#' | '[' | ']' | '*' | '_' | '`' | '@' | '<' | '>' | '"' => {
                    out.push('\\');
                    out.push(c);
                }
                _ => out.push(c),
            }
        }
        out
    }
    text.lines().map(escape_line).collect::<Vec<_>>().join("\n")
}

/// Render the deterministic 1080x1350 Typst deck: one page per slide, the
/// page/margin/typography laws baked into the header, and the `/drop` CTA
/// injected exactly once on the final slide.
fn render_carousel_typst(input: &CarouselInput) -> String {
    let mut typst = String::new();
    typst.push_str(
        "// Generated by castle::publish::linkedin (CASTLE-SPEC-26.11.20). Deterministic projection; do not hand-edit.\n",
    );
    typst.push_str(&format!(
        "#set page(width: {}px, height: {}px, margin: {}px)\n",
        CAROUSEL_WIDTH_PX,
        CAROUSEL_HEIGHT_PX,
        input.safe_margin_px
    ));
    typst.push_str(&format!("#set text(size: {}pt)\n", input.body_pt));
    typst.push_str("#set align(left)\n");

    let last = input.slides.len().saturating_sub(1);
    for (idx, slide) in input.slides.iter().enumerate() {
        if idx > 0 {
            typst.push_str("\n#pagebreak()\n\n");
        }
        typst.push_str(&format!(
            "= {}\n\n{}\n",
            escape_typst(&slide.headline),
            escape_typst(&slide.body)
        ));
        if idx == last && !input.slides.is_empty() {
            typst.push_str(&format!(
                "\n#align(center)[#text(size: {}pt)[{}]]\n",
                CTA_TEXT_PT,
                drop_cta_line(&input.lease_id)
            ));
        }
    }
    typst
}

fn drop_cta_line(lease_id: &str) -> String {
    format!("Get the drop: `{}`", drop_cta(lease_id))
}

fn post_full_text(input: &PostInput) -> String {
    format!("{}\n\n{}", input.hook, input.body)
}

/// The first-comment block: every link the post intends lives here and only
/// here.
fn build_first_comment(input: &PostInput) -> String {
    let mut lines = Vec::new();
    if let Some(link) = input.cta_link.as_deref() {
        if !link.trim().is_empty() {
            lines.push(format!("Full breakdown: {link}"));
        }
    }
    if let Some(lease_id) = input.lease_id.as_deref() {
        if lease_id_problem(lease_id).is_none() {
            lines.push(format!("Get the drop: {}", drop_cta(lease_id)));
        }
    }
    lines.join("\n")
}

/// Render the newsletter as Markdown (`#`/`##` headings, `*subtitle*`,
/// `- ` bullets).
pub fn render_newsletter_markdown(input: &NewsletterInput) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {}\n", input.title));
    if let Some(subtitle) = input.subtitle.as_deref() {
        if !subtitle.trim().is_empty() {
            markdown.push_str(&format!("\n*{subtitle}*\n"));
        }
    }
    for section in &input.sections {
        markdown.push_str(&format!("\n## {}\n", section.heading));
        for paragraph in &section.paragraphs {
            markdown.push_str(&format!("\n{paragraph}\n"));
        }
        for bullet in &section.bullets {
            markdown.push_str(&format!("\n- {bullet}\n"));
        }
    }
    markdown
}

/// Escape the five HTML-significant characters.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render the newsletter as HTML (`<article>` with h1/h2/p/ul-li), fully
/// escaped copy.
pub fn render_newsletter_html(input: &NewsletterInput) -> String {
    let mut html = String::from("<article>\n");
    html.push_str(&format!("<h1>{}</h1>\n", html_escape(&input.title)));
    if let Some(subtitle) = input.subtitle.as_deref() {
        if !subtitle.trim().is_empty() {
            html.push_str(&format!(
                "<p class=\"subtitle\">{}</p>\n",
                html_escape(subtitle)
            ));
        }
    }
    for section in &input.sections {
        html.push_str(&format!("<h2>{}</h2>\n", html_escape(&section.heading)));
        for paragraph in &section.paragraphs {
            html.push_str(&format!("<p>{}</p>\n", html_escape(paragraph)));
        }
        if !section.bullets.is_empty() {
            html.push_str("<ul>\n");
            for bullet in &section.bullets {
                html.push_str(&format!("<li>{}</li>\n", html_escape(bullet)));
            }
            html.push_str("</ul>\n");
        }
    }
    html.push_str("</article>\n");
    html
}

// ---------------------------------------------------------------------------
// Publishers
// ---------------------------------------------------------------------------

/// Compute the 5-Question Executive Check against the input and its rendered
/// deck.
fn executive_check(input: &CarouselInput, typst: &str) -> ExecutiveCheck {
    let q1 = input.slides.first().is_some_and(|slide| {
        !slide.headline.trim().is_empty()
            && is_action_first(&slide.headline)
            && has_hazard_cue(&slide.headline)
    });
    let q2 = input
        .slides
        .iter()
        .all(|slide| count_words(&slide.headline) + count_words(&slide.body) < MAX_SLIDE_WORDS);
    let q3 = input.body_pt.is_finite()
        && input.body_pt >= MIN_BODY_PT
        && input.safe_margin_px >= SAFE_MARGIN_PX
        && typst.contains(&format!("width: {CAROUSEL_WIDTH_PX}px"))
        && typst.contains(&format!("height: {CAROUSEL_HEIGHT_PX}px"));
    let q4 = lease_id_problem(&input.lease_id).is_none()
        && typst.matches(&drop_cta(&input.lease_id)).count() == 1;
    let q5 = input.slides.iter().all(|slide| {
        lint_copy(&format!("{}\n{}", slide.headline, slide.body)).is_empty()
    });

    ExecutiveCheck {
        questions: [
            ExecutiveQuestion {
                question: "1. Does Slide 1 open with an action-first hazard hook?",
                pass: q1,
            },
            ExecutiveQuestion {
                question: "2. Does every slide stay strictly under 60 words?",
                pass: q2,
            },
            ExecutiveQuestion {
                question: "3. Is body text at least 24pt on the 1080x1350 canvas inside the 50px safe margin?",
                pass: q3,
            },
            ExecutiveQuestion {
                question: "4. Does the final slide carry exactly one /drop/:lease_id CTA?",
                pass: q4,
            },
            ExecutiveQuestion {
                question: "5. Is the deck free of soft and subordinate scripts?",
                pass: q5,
            },
        ],
    }
}

/// Publish the carousel: run the framing + copy courts, then mint the
/// deterministic Typst deck with its Executive Check and digest. Refuses
/// with [`REFUSED_LINT_CAROUSEL`] on any finding.
pub fn publish_carousel(input: &CarouselInput) -> Result<CarouselArtifact, PublishRefusal> {
    let typst = render_carousel_typst(input);
    let check = executive_check(input, &typst);

    let mut findings = lint_carousel(input);
    if !check.all_pass() && findings.is_empty() {
        findings.push(LintFinding {
            rule: RULE_CAROUSEL_EXECUTIVE_CHECK,
            detail: "5-Question Executive Check failed without a named rule finding".to_string(),
        });
    }
    if !findings.is_empty() || !check.all_pass() {
        return Err(refusal(REFUSED_LINT_CAROUSEL, findings));
    }

    let cta = drop_cta(&input.lease_id);
    let payload = json!({
        "kind": "linkedin-carousel",
        "lease_id": input.lease_id,
        "width_px": CAROUSEL_WIDTH_PX,
        "height_px": CAROUSEL_HEIGHT_PX,
        "safe_margin_px": input.safe_margin_px,
        "body_pt": input.body_pt,
        "slide_count": input.slides.len(),
        "cta": cta,
        "executive_check_all_pass": check.all_pass(),
        "typst": typst,
    });

    Ok(CarouselArtifact {
        slide_count: input.slides.len(),
        width_px: CAROUSEL_WIDTH_PX,
        height_px: CAROUSEL_HEIGHT_PX,
        safe_margin_px: input.safe_margin_px,
        body_pt: input.body_pt,
        lease_id: input.lease_id.clone(),
        cta,
        typst,
        executive_check: check,
        digest: blake3_digest(&payload),
    })
}

/// Publish the post: run the framing + copy courts, then mint the link-free
/// body with its first-comment block and digest. Refuses with
/// [`REFUSED_LINT_POST`] on any finding.
pub fn publish_post(input: &PostInput) -> Result<PostArtifact, PublishRefusal> {
    let findings = lint_post(input);
    if !findings.is_empty() {
        return Err(refusal(REFUSED_LINT_POST, findings));
    }

    let hook = input.hook.clone();
    let body = input.body.clone();
    let full_text = post_full_text(input);
    let first_comment = build_first_comment(input);
    let word_count = count_words(&full_text);
    let hook_chars = hook.chars().count();

    let payload = json!({
        "kind": "linkedin-post",
        "hook": hook,
        "body": body,
        "first_comment": first_comment,
        "word_count": word_count,
        "hook_chars": hook_chars,
    });

    Ok(PostArtifact {
        hook,
        body,
        full_text,
        first_comment,
        word_count,
        hook_chars,
        digest: blake3_digest(&payload),
    })
}

/// Publish the newsletter: run the shape + copy courts, then mint the
/// Markdown/HTML pair with its digest. Refuses with
/// [`REFUSED_LINT_NEWSLETTER`] on any finding.
pub fn publish_newsletter(input: &NewsletterInput) -> Result<NewsletterArtifact, PublishRefusal> {
    let findings = lint_newsletter(input);
    if !findings.is_empty() {
        return Err(refusal(REFUSED_LINT_NEWSLETTER, findings));
    }

    let markdown = render_newsletter_markdown(input);
    let html = render_newsletter_html(input);
    let payload = json!({
        "kind": "linkedin-newsletter",
        "title": input.title,
        "markdown": markdown,
        "html": html,
    });

    Ok(NewsletterArtifact {
        title: input.title.clone(),
        markdown,
        html,
        digest: blake3_digest(&payload),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn slide(headline: &str, body: &str) -> SlideInput {
        SlideInput {
            headline: headline.to_string(),
            body: body.to_string(),
        }
    }

    fn good_carousel() -> CarouselInput {
        CarouselInput {
            lease_id: "lease-abc123".to_string(),
            body_pt: 24.0,
            safe_margin_px: 50,
            slides: (0..8)
                .map(|i| {
                    if i == 0 {
                        slide(
                            "Stop leaking receipts",
                            "Every unverified write is a silent loss. Seal the chain first.",
                        )
                    } else {
                        slide(
                            &format!("Slide {n} headline", n = i + 1),
                            &format!("Slide {n} keeps the copy short and concrete.", n = i + 1),
                        )
                    }
                })
                .collect(),
        }
    }

    fn words(count: usize, prefix: &str) -> String {
        (0..count)
            .map(|i| format!("{prefix}{i}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Hook contributes exactly 7 words; body fills the rest.
    fn post_with_total_words(total: usize) -> PostInput {
        PostInput {
            hook: "Ship the ledger before it ships you.".to_string(),
            body: words(total.saturating_sub(7), "w"),
            lease_id: Some("lease-abc123".to_string()),
            cta_link: None,
        }
    }

    fn good_newsletter() -> NewsletterInput {
        NewsletterInput {
            title: "Castle Weekly 001".to_string(),
            subtitle: Some("Deterministic publishing, receipted.".to_string()),
            sections: vec![
                NewsletterSection {
                    heading: "This week in the castle".to_string(),
                    paragraphs: vec![
                        "The chain sealed eight receipts and replayed them byte-identical."
                            .to_string(),
                    ],
                    bullets: vec!["Carousel court shipped".to_string()],
                },
                NewsletterSection {
                    heading: "Next week".to_string(),
                    paragraphs: vec!["The gate learns to refuse harder.".to_string()],
                    bullets: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn carousel_publishes_full_template_with_cta_and_check() {
        let artifact = publish_carousel(&good_carousel()).expect("clean deck must publish");
        assert_eq!(artifact.slide_count, 8);
        assert_eq!(
            (artifact.width_px, artifact.height_px, artifact.safe_margin_px),
            (CAROUSEL_WIDTH_PX, CAROUSEL_HEIGHT_PX, SAFE_MARGIN_PX)
        );
        assert!(artifact.body_pt >= MIN_BODY_PT);
        assert!(artifact
            .typst
            .contains("#set page(width: 1080px, height: 1350px, margin: 50px)"));
        assert!(artifact.typst.contains("#set text(size: 24pt)"));
        assert_eq!(artifact.typst.matches("#pagebreak()").count(), 7);
        assert_eq!(artifact.typst.matches("/drop/lease-abc123").count(), 1);
        assert!(artifact.typst.contains("Get the drop: `/drop/lease-abc123`"));
        assert!(artifact.executive_check.all_pass());
        assert_eq!(artifact.executive_check.questions.len(), 5);
        assert_eq!(artifact.digest.len(), 64);
        assert!(artifact.digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn carousel_replays_byte_identical_and_diverges_on_input_change() {
        let first = publish_carousel(&good_carousel()).unwrap();
        let second = publish_carousel(&good_carousel()).unwrap();
        assert_eq!(first, second);

        let mut changed = good_carousel();
        changed.slides[3].body = "A different sentence entirely.".to_string();
        let third = publish_carousel(&changed).unwrap();
        assert_ne!(first.digest, third.digest);
        assert_ne!(first.typst, third.typst);
    }

    #[test]
    fn carousel_slide_count_boundaries() {
        for count in [7usize, 11] {
            let mut input = good_carousel();
            while input.slides.len() < count {
                input.slides.push(slide("Filler headline", "Filler body."));
            }
            input.slides.truncate(count);
            let err = publish_carousel(&input)
                .expect_err("deck outside 8-10 slides must refuse");
            assert_eq!(err.code, REFUSED_LINT_CAROUSEL);
            assert_eq!(err.broken_term, BROKEN_TERM);
            assert!(err
                .findings
                .iter()
                .any(|f| f.rule == RULE_CAROUSEL_SLIDE_COUNT));
        }
        for count in [8usize, 9, 10] {
            let mut input = good_carousel();
            while input.slides.len() < count {
                input.slides.push(slide("Filler headline", "Filler body."));
            }
            assert!(publish_carousel(&input).is_ok(), "{count} slides must pass");
        }
    }

    #[test]
    fn carousel_word_budget_boundary_is_strict() {
        // Slide 3 headline contributes 3 words; 57 + 3 = 60 must refuse.
        let mut over = good_carousel();
        over.slides[2].body = words(57, "w");
        let err = publish_carousel(&over).expect_err("60 words must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_SLIDE_WORD_BUDGET));

        // 55 + 3 = 58 words must pass.
        let mut under = good_carousel();
        under.slides[2].body = words(55, "w");
        assert!(publish_carousel(&under).is_ok());
    }

    #[test]
    fn carousel_requires_action_first_hazard_hook() {
        let mut not_action = good_carousel();
        not_action.slides[0].headline = "Here is what broke my pipeline".to_string();
        let err = publish_carousel(&not_action).expect_err("non-imperative hook must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_HOOK_ACTION_FIRST));
        assert!(err
            .findings
            .iter()
            .all(|f| f.rule != RULE_CAROUSEL_HOOK_HAZARD));

        let mut no_hazard = good_carousel();
        no_hazard.slides[0].headline = "Stop shipping on Fridays".to_string();
        let err = publish_carousel(&no_hazard).expect_err("hazard-free hook must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_HOOK_HAZARD));
        assert!(err
            .findings
            .iter()
            .all(|f| f.rule != RULE_CAROUSEL_HOOK_ACTION_FIRST));
    }

    #[test]
    fn carousel_refuses_soft_scripts() {
        let mut input = good_carousel();
        input.slides[4].body =
            "Would it be crazy to audit weekly? Totally fine if not.".to_string();
        let err = publish_carousel(&input).expect_err("soft scripts must refuse");
        assert_eq!(err.code, REFUSED_LINT_CAROUSEL);
        let hits: Vec<_> = err
            .findings
            .iter()
            .filter(|f| f.rule == RULE_SOFT_SCRIPT)
            .collect();
        assert_eq!(hits.len(), 2, "both soft phrases must be caught");
    }

    #[test]
    fn carousel_refuses_invalid_lease_and_duplicate_cta() {
        let mut bad_lease = good_carousel();
        bad_lease.lease_id = "lease/abc".to_string();
        let err = publish_carousel(&bad_lease).expect_err("slash in lease id must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_LEASE_ID));

        let mut duplicate = good_carousel();
        duplicate.slides[6].body = "Review the /drop/lease-abc123 page.".to_string();
        let err = publish_carousel(&duplicate).expect_err("CTA in slide copy must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_CTA_UNIQUE));
    }

    #[test]
    fn carousel_refuses_substrate_laws() {
        let mut small_type = good_carousel();
        small_type.body_pt = 23.5;
        let err = publish_carousel(&small_type).expect_err("sub-24pt body must refuse");
        assert!(err.findings.iter().any(|f| f.rule == RULE_CAROUSEL_BODY_PT));

        let mut thin_margin = good_carousel();
        thin_margin.safe_margin_px = 49;
        let err = publish_carousel(&thin_margin).expect_err("sub-50px margin must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_CAROUSEL_SAFE_MARGIN));
    }

    #[test]
    fn post_publishes_within_budget_with_first_comment() {
        let artifact = publish_post(&post_with_total_words(150))
            .expect("in-budget post must publish");
        assert_eq!(artifact.word_count, 150);
        assert!(artifact.hook.chars().count() < MAX_HOOK_CHARS);
        assert!(!artifact.full_text.contains("http"));
        assert_eq!(artifact.first_comment, "Get the drop: /drop/lease-abc123");
        assert_eq!(artifact.digest.len(), 64);
        assert_eq!(
            artifact,
            publish_post(&post_with_total_words(150)).unwrap()
        );
    }

    #[test]
    fn post_word_budget_boundaries() {
        for (total, ok) in [(149usize, false), (150, true), (250, true), (251, false)] {
            let result = publish_post(&post_with_total_words(total));
            assert_eq!(result.is_ok(), ok, "post of {total} words");
            if let Err(err) = result {
                assert!(err
                    .findings
                    .iter()
                    .any(|f| f.rule == RULE_POST_WORD_BUDGET));
            }
        }
    }

    #[test]
    fn post_hook_length_boundary_is_strict() {
        // 140-char hook (1 word) + 149-word body = 150 total: only the hook rule fires.
        let mut long = post_with_total_words(150);
        long.hook = "x".repeat(140);
        long.body = words(149, "w");
        let err = publish_post(&long).expect_err("140-char hook must refuse");
        assert_eq!(err.code, REFUSED_LINT_POST);
        let rules: Vec<_> = err.findings.iter().map(|f| f.rule).collect();
        assert!(rules.contains(&RULE_POST_HOOK_LENGTH));
        assert!(!rules.contains(&RULE_POST_WORD_BUDGET));

        let mut fits = long;
        fits.hook = "x".repeat(139);
        let artifact = publish_post(&fits).expect("139-char hook must pass");
        assert_eq!(artifact.hook_chars, 139);
    }

    #[test]
    fn post_refuses_external_link_in_body_but_allows_first_comment() {
        let mut linked = post_with_total_words(150);
        linked.body = format!("{} and see https://example.com/receipts", linked.body);
        let err = publish_post(&linked).expect_err("body link must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_POST_EXTERNAL_LINK));

        let mut commented = post_with_total_words(150);
        commented.cta_link = Some("https://example.com/receipts".to_string());
        let artifact = publish_post(&commented).expect("link in first comment must pass");
        assert!(artifact.first_comment.contains("https://example.com/receipts"));
        assert!(!artifact.full_text.contains("http"));
    }

    #[test]
    fn post_refuses_multiline_hook_and_soft_scripts() {
        let multiline = PostInput {
            hook: "Ship it\ntoday".to_string(),
            body: words(147, "w"),
            lease_id: Some("lease-abc123".to_string()),
            cta_link: None,
        };
        let err = publish_post(&multiline).expect_err("multiline hook must refuse");
        // 3 hook words + 147 body words = 150, so the budget passes and the
        // single-line rule is isolated.
        assert_eq!(err.findings.len(), 1);
        assert_eq!(err.findings[0].rule, RULE_POST_HOOK_SINGLE_LINE);
    }

    #[test]
    fn post_refuses_soft_script_copies() {
        let mut input = post_with_total_words(150);
        input.body = format!("{} No worries if not.", words(140, "w"));
        let err = publish_post(&input).expect_err("soft script must refuse");
        assert!(err.findings.iter().any(|f| f.rule == RULE_SOFT_SCRIPT));
    }

    #[test]
    fn newsletter_publishes_markdown_and_html() {
        let artifact = publish_newsletter(&good_newsletter())
            .expect("well-formed newsletter must publish");
        assert!(artifact.markdown.starts_with("# Castle Weekly 001\n"));
        assert!(artifact.markdown.contains("## This week in the castle"));
        assert!(artifact.markdown.contains("- Carousel court shipped"));
        assert!(artifact.html.contains("<h1>Castle Weekly 001</h1>"));
        assert!(artifact
            .html
            .contains("<p class=\"subtitle\">Deterministic publishing, receipted.</p>"));
        assert!(artifact.html.contains("<h2>This week in the castle</h2>"));
        assert!(artifact.html.contains("<li>Carousel court shipped</li>"));
        assert_eq!(artifact.digest.len(), 64);
        assert_eq!(
            artifact,
            publish_newsletter(&good_newsletter()).unwrap()
        );
    }

    #[test]
    fn newsletter_escapes_html_significant_copy() {
        let mut input = good_newsletter();
        input.title = "<Scripts & \"Quotes\">".to_string();
        let artifact = publish_newsletter(&input).expect("escaping happens after the court");
        assert!(artifact
            .html
            .contains("<h1>&lt;Scripts &amp; &quot;Quotes&quot;&gt;</h1>"));
        assert!(!artifact.html.contains("<h1><Scripts"));
    }

    #[test]
    fn newsletter_refuses_soft_script_and_empty_title() {
        let mut soft = good_newsletter();
        soft.sections[0].paragraphs[0] =
            "Just wondering if the chain held. It did.".to_string();
        let err = publish_newsletter(&soft).expect_err("soft script must refuse");
        assert_eq!(err.code, REFUSED_LINT_NEWSLETTER);
        assert!(err.findings.iter().any(|f| f.rule == RULE_SOFT_SCRIPT));

        let mut untitled = good_newsletter();
        untitled.title = "   ".to_string();
        let err = publish_newsletter(&untitled).expect_err("empty title must refuse");
        assert!(err
            .findings
            .iter()
            .any(|f| f.rule == RULE_NEWSLETTER_TITLE_REQUIRED));
    }

    #[test]
    fn lint_copy_flags_every_blacklisted_phrase() {
        for phrase in SOFT_SCRIPT_PHRASES {
            let text = format!("Team update. {phrase} We ship anyway.");
            let findings = lint_copy(&text);
            assert_eq!(findings.len(), 1, "phrase {phrase:?} not caught exactly once");
            assert_eq!(findings[0].rule, RULE_SOFT_SCRIPT);
            assert!(findings[0].detail.contains(phrase));
        }
    }

    #[test]
    fn lint_copy_passes_clean_copy_with_boundary_lookalikes() {
        let clean = "Crazy velocity this week. The pipeline is totally fine and audits run \
                     clean. Ship the drop.";
        assert!(lint_copy(clean).is_empty());
    }

    #[test]
    fn helpers_measure_deterministically() {
        assert_eq!(count_words("a b  c\n d"), 4);
        assert_eq!(count_words(""), 0);
        assert!(is_action_first("Stop leaking receipts"));
        assert!(!is_action_first("Here is the thing"));
        assert!(has_hazard_cue("Stop leaking receipts"));
        assert!(!has_hazard_cue("Stop shipping on Fridays"));
        assert_eq!(drop_cta("lease-9"), "/drop/lease-9");
        assert_eq!(
            lease_id_problem("lease-9_A"),
            None,
            "alnum, dash and underscore are lawful"
        );
        assert!(lease_id_problem("").is_some());
        assert!(lease_id_problem("lease/9").is_some());
        assert_eq!(external_link_marker("see www.example.com"), Some("www."));
        assert_eq!(external_link_marker("no links here"), None);
    }

    #[test]
    fn typst_escaping_neutralizes_markup() {
        assert_eq!(escape_typst("#set page"), "\\#set page");
        assert_eq!(escape_typst("- item"), "\\- item");
        assert_eq!(escape_typst("= heading"), "\\= heading");
        assert_eq!(escape_typst("a*b_c"), "a\\*b\\_c");
        assert_eq!(escape_typst("plain text"), "plain text");

        let mut injected = good_carousel();
        injected.slides[5].body = "#set page(width: 1px)".to_string();
        let artifact = publish_carousel(&injected).expect("markup is escaped, not refused");
        assert!(artifact.typst.contains("\\#set page(width: 1px)"));
        assert_eq!(artifact.typst.matches("#set page(width: 1080px").count(), 1);
    }
}
