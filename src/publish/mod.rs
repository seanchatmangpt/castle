//! CASTLE publish surface.
//!
//! Honest boundary:
//! - **Typst source emit** for LinkedIn carousels is ALIVE (`linkedin`, `wire`).
//! - **PDF compile** is best-effort via host `typst` (`typst_pdf`); when the
//!   binary is missing the wire documents a `pdf.standing = gap` — never a
//!   fake ALIVE PDF claim.
//! - **Ggen layout digest-bind** (`digest_bind`) is wired through
//!   `wire::seal_public_artifact` when `--ggen-layout-digest` / an API pin is
//!   supplied (`crown_wire: WIRED`). Omit pin → key absent (optional).
//!   Wire bind ≠ Foundry / zero-HITL crown `ALIVE`.
//!
//! Public targets require `--public-anonymize` and an *explicit* `--brand
//! <path>` (e.g. `brands/fourth-watch.brand.toml`) — omitting `--brand` is
//! `REFUSED_BRAND`; there is no soft default path.

pub mod anonymize;
pub mod brand;
pub mod digest_bind;
pub mod linkedin;
pub mod typst_pdf;
pub mod wire;

pub use anonymize::{
    require_public_anonymize, AnonymizeRefusal, RedactionPolicy, REFUSED_NO_PUBLIC_ANONYMIZE,
};
pub use brand::BrandManifest;
pub use digest_bind::{
    admit_ggen_layout_digest, assert_mint_binds_digest, bind_into_mint, DigestBindReceipt,
    DigestBindRefusal, GgenLayoutDigestPin, CLI_FLAG_GGEN_LAYOUT_DIGEST, MINT_KEY_GGEN_LAYOUT_DIGEST,
    REFUSED_GGEN_DIGEST,
};
pub use typst_pdf::{try_compile_pdf, PdfCompileStatus};
pub use wire::{publish_target, require_brand_path, PublishWireError, PUBLIC_TARGETS};
