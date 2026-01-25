//! @arrive component: auth
//! Authentication, authorization, RBAC, and audit hooks.
//!
//! This crate provides:
//! - User management and authentication
//! - Role-Based Access Control (RBAC)
//! - Session/token management
//! - Audit event recording
//! - Authentication middleware for Axum

mod audit;
mod error;
mod middleware;
mod rbac;
mod session;
mod user;

pub use audit::{AuditEvent, AuditEventType, record_audit_event};
pub use error::{AuthError, Result};
pub use middleware::{AuthLayer, CurrentUser};
pub use rbac::{Permission, Role, check_permission};
pub use session::{Session, SessionManager, create_session, validate_session};
pub use user::{User, UserRepository};

