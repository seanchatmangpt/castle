//! Library surface for `castle-ggen-rdf`.
//!
//! Previously bin-only; the strata receipt-ingest lane added a `lib` target
//! so the ggen `ggen-receipt/v2` ingest calculus is consumable as a library
//! (and testable via `tests/strata_ingest.rs`) without spawning the binary.

pub mod strata;
