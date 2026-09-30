//! RFC reference-stack consumer projection.
//!
//! CASTLE does not own semantic law, Knowledge-Hook meaning, SA2A routing, or
//! trust-plane truth. This module consumes those admitted projections and binds
//! them to CASTLE's existing PreparedEffect/BRCE/receipt/OCEL machinery.

mod feedback;
mod fibo;
mod ownership;

pub use feedback::{project_beam4pm_feedback, Beam4PmFeedback};
pub use fibo::{
    bind_verification_receipt, project_fibo_payment, AuthorityEvidence, FiboPaymentProjection,
    KnowledgeHookCandidate, FIBO_PAYMENT_CAPABILITY, GRAPHLAW_HOOK_CONTRACT,
};
pub use ownership::{
    canonical_owners, ReferenceOwner, ReferenceStage, INHERITED_COURTS, REFERENCE_STAGES,
};
