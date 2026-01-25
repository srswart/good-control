---
advance:
  id: "ADV-AUTH-001"
  title: "AuthN/AuthZ + RBAC + audit hooks: initial implementation"
  system: "backend"
  primary_component: "auth"
  components: ["auth"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T14:05:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: ["auth", "public_api"]
  evidence:
    - type: "tests:unit"
      description: "21 unit tests passing (user management, RBAC, sessions, audit, middleware)"
    - type: "tdd:red-green"
      description: "Tests written for all auth modules before implementation"
    - type: "database:migration"
      description: "Auth schema migration with roles, permissions, sessions, and audit tables"
  status: complete
---

## Objective

Create the first working slice of the auth component and establish its public contracts, tests, and wiring for the backend system.  Ensure the auth component does not impede development and has a clear mechanism to establish bootstrap auth while remaining secure. 

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed auth component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Completed Implementation Tasks

- [x] Created auth crate structure (`crates/backend/auth`)
- [x] Created auth database migration with users, roles, permissions, sessions, and audit_events tables
- [x] Implemented user model and repository with password hashing and authentication
- [x] Implemented RBAC models (Role, Permission) with permission checking logic
- [x] Implemented session/token management with SHA-256 token generation
- [x] Implemented audit hooks for recording security and business events
- [x] Implemented auth middleware for Axum request authentication
- [x] Added comprehensive unit tests (21 tests passing)
- [x] Updated workspace Cargo.toml with auth crate
- [x] Seeded default roles (admin, warehouse_operator, warehouse_supervisor, compliance_officer)
- [x] Seeded default permissions for inventory, warehouse, compliance, and admin operations

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tdd:red-green - Tests written for all auth modules (user, RBAC, session, audit, middleware)
- [x] tests:unit - 21 unit tests passing covering all auth functionality
- [x] database:migration - Auth schema migration applied successfully
- [x] Auth crate builds successfully with no errors
- [x] All auth modules functional with comprehensive test coverage
- [x] Default roles and permissions seeded in database
- [x] Password hashing with SHA-256
- [x] Token-based session management
- [x] Axum middleware integration working
