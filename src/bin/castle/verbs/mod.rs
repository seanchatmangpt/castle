//! `verbs` module — the noun-verb CLI routes plus their handlers.
//!
//! `routes.rs` is a thin `#[verb]`-annotated wrapper layer with no business
//! logic of its own — every route delegates to `handlers::*`, which calls
//! into the `castle` library crate. This mirrors the ecosystem's own
//! generated-route / hand-written-handler seam (see
//! `~/ggen/examples/clap-noun-verb-cli/src/verbs/`), authored directly here
//! since castle has no `cnv:` ontology of its own yet to drive `ggen sync`.

pub mod handlers;
pub mod routes;
