//! @arrive component: compliance-ui
//! Compliance UI: approvals, doc packs, blocking reasons.
//!
//! This crate provides UI components for compliance management:
//! - Approvals workflow
//! - Document packs
//! - Blocking reasons

pub mod approvals;
pub mod doc_packs;
pub mod blocking_reasons;

pub use approvals::ApprovalsView;
pub use doc_packs::DocPacksView;
pub use blocking_reasons::BlockingReasonsView;
