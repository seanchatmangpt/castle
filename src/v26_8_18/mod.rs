//! CASTLE v26.8.18 Fortune-5 global deployment runtime and DfCM closure.
//!
//! The release line provides cellular deployment constitution, O -> O* admission,
//! transport-neutral CLI/API/MCP/A2A intent contracts, air-gapped CONSTRUCT,
//! BRCE-wrapped real provider execution, durable evidence, dual identity/crypto,
//! cross-cell reconciliation, chaos qualification, real PQC self-tests, bounded
//! live-provider observation, and global standing without creating an alternate DO edge.

mod airgap;
mod brce;
mod chaos;
mod crypto;
mod dfcm;
mod evidence;
mod protocol;
mod replication;
mod runtime;
mod topology;

pub const RELEASE_VERSION: &str = "26.8.18+dfcm.1";
pub const RELEASE_KIND: &str = "CASTLE_FORTUNE5_GLOBAL_V1";

pub use airgap::*;
pub use brce::*;
pub use chaos::*;
pub use crypto::*;
pub use dfcm::*;
pub use evidence::*;
pub use protocol::*;
pub use replication::*;
pub use runtime::*;
pub use topology::*;
