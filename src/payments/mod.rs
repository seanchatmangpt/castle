//! CASTLE SA2A Payments: an economic-actuation kernel.
//!
//! `Candidate != Authorized != Funded != Constructed != Settled != Final`.
//! Nothing here moves value except through the existing exclusive DO path
//! (`castle::execute_powl_with_gym_act`) wrapped in durable BRCE journaling.
//! An agent-originated `PreparedEffect` has zero financial power until
//! `admission::admit_payment` seals it as a `PaymentAdmission`.

pub mod adapter;
pub mod admission;
pub mod claim_store;
pub mod compliance;
pub mod counterparty;
pub mod dirlock;
pub mod effect;
pub mod event_receipt;
pub mod execute;
pub mod execute_rail;
pub mod experience;
pub mod fibo;
pub mod iso20022;
pub mod ledger;
pub mod money;
pub mod nonce;
pub mod obligation;
pub mod pee;
pub mod policy;
pub mod rail;
pub mod rail_sim;
pub mod reconcile;
pub mod refusal;
pub mod replay;
pub mod settlement;


pub use admission::{admit_payment, AdmissionContext, PaymentAdmission};
pub use claim_store::{Claim, ClaimState, ClaimStore};
pub use effect::{PaymentEffect, PAYMENT_CAPABILITY};
pub use execute::{build_construct, execute_payment, ExecutionContext, PaymentExecution, PaymentStanding};
pub use ledger::{ActuationToken, FileJournalLedger, LedgerEntry, LedgerError, LedgerPort};
pub use experience::{ClassRule, ExperienceStore, Knowledge, PaymentClass};
pub use reconcile::{reconcile, recover_journal, JournalRecovery, ReconcileResolution};
pub use money::{Currency, Money};
pub use nonce::DurableNonceFence;
pub use policy::{PrincipalPolicy, SpendPolicy};

/// Payments release line. Crate version is unchanged; this tags the closure layer.
pub const VERSION: &str = "26.9.29";
