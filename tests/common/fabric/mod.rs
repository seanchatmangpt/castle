//! Test-side harness mirroring `Xaas.Fabric.*` (~/xaas/lib/xaas/fabric): runtime composition
//! (registry, resolver, orchestrator, replay) belongs to XaaS. CASTLE owns only the envelope
//! (`castle::operation_envelope`); this harness lets CASTLE's own payment providers be exercised
//! by the CASTLE ALIVE test until a XaaS-to-CASTLE transport exists.

pub mod contract;
pub mod orchestrator;
pub mod registry;

pub use contract::{Capability, CapabilityContract, CapError, CapResult, Facts, FailureClass};
pub use castle::operation_envelope::{EffectClass, OperationEnvelope};
pub use orchestrator::{Orchestrator, Outcome, ReplayVerdict, Standing, Stage};
pub use registry::Registry;

pub const URI_PROJECTION: &str = "capability://projection/map";
pub const URI_PROCESS: &str = "capability://process/conformance";
pub const URI_LAW: &str = "capability://law/admit";
pub const URI_EVIDENCE: &str = "capability://evidence/attest";
pub const URI_ACTUATION: &str = "capability://agent/actuate";
