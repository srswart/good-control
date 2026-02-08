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
mod bootstrap;
mod error;
mod middleware;
mod passkey;
mod rbac;
mod session;
mod user;

pub use audit::{AuditEvent, AuditEventType, record_audit_event};
pub use bootstrap::seed_admin_user;
pub use error::{AuthError, Result};
pub use middleware::{AuthLayer, CurrentUser, optional_auth_middleware};
pub use passkey::{
    begin_passkey_authentication, begin_passkey_registration, finish_passkey_authentication,
    finish_passkey_registration,
};
pub use rbac::{Permission, Role, check_permission, require_permission, get_user_permissions, get_user_roles, assign_role, remove_role, find_role_by_name};
pub use session::{Session, SessionManager, create_session, validate_session, authenticate};
pub use user::{User, UserRepository, SqliteUserRepository};

