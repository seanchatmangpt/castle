//! CASTLE-SPEC-26.11.20 publish noun/verb routes.
//!
//! Wires `castle publish --public-anonymize --brand <path>` to consume an
//! explicit brand manifest via `castle::publish::wire`. Omitting `--brand`
//! on a public target is `REFUSED_BRAND` (no soft default path).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Unified public target dispatch. Requires `--public-anonymize` and explicit `--brand <path>` (omit → REFUSED_BRAND; no soft default).
#[verb("run", "publish")]
fn publish_run(
    target: String,
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
    subject: Option<String>,
    out_dir: Option<String>,
) -> Result<serde_json::Value> {
    super::castle_261120_handlers::publish_run_handler(
        target,
        brand,
        public_anonymize,
        input_path,
        subject,
        out_dir,
    )
}

/// LinkedIn carousel (Typst). Requires `--public-anonymize` and explicit `--brand <path>` (omit → REFUSED_BRAND).
#[verb("carousel", "publish")]
fn publish_carousel(
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
    out_dir: Option<String>,
) -> Result<serde_json::Value> {
    super::castle_261120_handlers::publish_carousel_handler(
        brand,
        public_anonymize,
        input_path,
        out_dir,
    )
}

/// LinkedIn post. Requires `--public-anonymize` and explicit `--brand <path>` (omit → REFUSED_BRAND; no soft default).
#[verb("post", "publish")]
fn publish_post(
    subject: Option<String>,
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
) -> Result<serde_json::Value> {
    super::castle_261120_handlers::publish_post_handler(
        subject,
        brand,
        public_anonymize,
        input_path,
    )
}

/// LinkedIn newsletter. Requires `--public-anonymize`, `--input-path`, and explicit `--brand <path>` (omit → REFUSED_BRAND).
#[verb("newsletter", "publish")]
fn publish_newsletter(
    brand: Option<String>,
    public_anonymize: bool,
    input_path: String,
) -> Result<serde_json::Value> {
    super::castle_261120_handlers::publish_newsletter_handler(brand, public_anonymize, input_path)
}

#[linkme::distributed_slice(::clap_noun_verb::cli::registry::__NOUN_REGISTRY)]
static REGISTER_PUBLISH_NOUN: fn() = register_publish_noun;

fn register_publish_noun() {
    ::clap_noun_verb::cli::registry::CommandRegistry::register_noun(
        "publish",
        "Deterministic public publish (LinkedIn carousel/post/newsletter) with mandatory --public-anonymize redaction court and mandatory explicit --brand <path> (omit → REFUSED_BRAND; no soft default).",
    );
}
