//! CASTLE capability fabric (SRFC `docs/rfc/castle-fabric`): product intent is carried by an
//! [`OperationEnvelope`], resolved by URI through a [`Registry`] to independently qualified
//! [`Capability`] realizations, and driven by the [`Orchestrator`] through
//! OBSERVE -> SELECT -> CONSTRUCT -> DO -> RECEIPT -> REPLAY -> STANDING.
//!
//! Nothing here names a concrete realization: the fabric is plane-agnostic by construction.

pub mod contract;
pub mod envelope;
pub mod orchestrator;
pub mod registry;

pub use contract::{Capability, CapabilityContract, CapError, CapResult, Facts, FailureClass};
pub use envelope::{EffectClass, OperationEnvelope};
pub use orchestrator::{Orchestrator, Outcome, ReplayVerdict, Standing, Stage};
pub use registry::Registry;

pub const URI_PROJECTION: &str = "capability://projection/map";
pub const URI_PROCESS: &str = "capability://process/conformance";
pub const URI_LAW: &str = "capability://law/admit";
pub const URI_EVIDENCE: &str = "capability://evidence/attest";
pub const URI_ACTUATION: &str = "capability://agent/actuate";
