//! CASTLE-SPEC-26.11.20 publish handlers — brand + public-anonymize wire.
//!
//! Public / anonymize targets require an *explicit* `--brand <path>` —
//! omit → `REFUSED_BRAND`. There is no soft default to
//! `brands/fourth-watch.brand.toml`. Non-public paths never invent a brand.
//!
//! Optional `--ggen-layout-digest <64-hex>` threads an AotSealReceipt-shaped
//! pin into `seal_public_artifact` → `bind_into_mint`.

use castle::publish::wire::{publish_target, PublishWireError};
use clap_noun_verb::NounVerbError;
use serde_json::Value;

type Result<T> = std::result::Result<T, NounVerbError>;

fn refused(err: PublishWireError) -> NounVerbError {
    // Emit structured refused JSON; main::failure_json promotes it to a bare envelope + exit 2
    // (typed-refusal contract shared with REFUSED:/BLOCKED: sibling noun-verb handlers).
    NounVerbError::ExecutionError {
        message: err.to_json().to_string(),
    }
}

/// `castle publish run --target … --brand … --public-anonymize [--ggen-layout-digest …]`
pub fn publish_run_handler(
    target: String,
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
    subject: Option<String>,
    out_dir: Option<String>,
    ggen_layout_digest: Option<String>,
) -> Result<Value> {
    publish_target(
        &target,
        brand.as_deref(),
        public_anonymize,
        input_path.as_deref(),
        subject.as_deref(),
        out_dir.as_deref(),
        ggen_layout_digest.as_deref(),
    )
    .map_err(refused)
}

/// `castle publish carousel --brand … --public-anonymize [--ggen-layout-digest …]`
pub fn publish_carousel_handler(
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
    out_dir: Option<String>,
    ggen_layout_digest: Option<String>,
) -> Result<Value> {
    publish_run_handler(
        "linkedin-carousel".into(),
        brand,
        public_anonymize,
        input_path,
        None,
        out_dir,
        ggen_layout_digest,
    )
}

/// `castle publish post --subject t --brand … --public-anonymize [--ggen-layout-digest …]`
pub fn publish_post_handler(
    subject: Option<String>,
    brand: Option<String>,
    public_anonymize: bool,
    input_path: Option<String>,
    ggen_layout_digest: Option<String>,
) -> Result<Value> {
    publish_run_handler(
        "linkedin-post".into(),
        brand,
        public_anonymize,
        input_path,
        subject,
        None,
        ggen_layout_digest,
    )
}

/// `castle publish newsletter --brand … --public-anonymize --input-path …`
pub fn publish_newsletter_handler(
    brand: Option<String>,
    public_anonymize: bool,
    input_path: String,
    ggen_layout_digest: Option<String>,
) -> Result<Value> {
    publish_run_handler(
        "linkedin-newsletter".into(),
        brand,
        public_anonymize,
        Some(input_path),
        None,
        None,
        ggen_layout_digest,
    )
}
