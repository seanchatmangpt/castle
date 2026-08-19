//! `verbs` module — the noun-verb CLI routes plus their handlers.
//!
//! `routes.rs` is a thin `#[verb]`-annotated wrapper layer with no business
//! logic of its own — every route delegates to a handler module, which calls
//! into the `castle` library crate. DfCM closure handlers are separated only to
//! keep the existing release-line handler file bounded.

pub mod dfcm_handlers;
pub mod handlers;
pub mod routes;
