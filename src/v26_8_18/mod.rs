//! CASTLE v26.8.18 Fortune-5 global deployment runtime.
//!
//! The release adds a cellular deployment constitution, O -> O* admission,
//! transport-neutral CLI/API/MCP/A2A intent contracts, air-gapped CONSTRUCT,
//! BRCE-wrapped real provider execution, and global standing without creating
//! any alternate DO edge.

mod airgap;
mod brce;
mod protocol;
mod topology;

pub const RELEASE_VERSION: &str = "26.8.18";
pub const RELEASE_KIND: &str = "CASTLE_FORTUNE5_GLOBAL_V1";

pub use airgap::*;
pub use brce::*;
pub use protocol::*;
pub use topology::*;
