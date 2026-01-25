---
advance:
  id: "ADV-BACKEND-001"
  title: "Backend Foundation (workspace, Axum server, config, tracing): initial implementation"
  system: "backend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T14:30:00Z"
  pr_links: []
  reviewability_score: 25
  risk_flags: ["new_dependency", "public_api"]
  evidence: ["tests:unit"]
  status: complete
---

## Objective

Create the first working slice of the foundation component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed foundation component boundary, with the minimal scaffolding needed to support subsequent feature advances.
- Cargo workspace established with `crates/backend/foundation`, `crates/backend/server`, and `crates/shared`
- Configuration loading from environment variables and optional config file
- Tracing/logging initialization with configurable log levels
- Axum HTTP server with health check endpoint at `/health`
- 8 unit tests covering config, tracing, router, and health endpoints

## Planned Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change) - N/A for greenfield
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tests:unit (8 tests passing)
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-25 - feat: Create Cargo workspace structure
- `Cargo.toml`: Root workspace with shared dependencies
- `crates/shared/Cargo.toml`: Shared types crate
- `crates/shared/src/lib.rs`: Shared module exports
- `crates/shared/src/error.rs`: Common error types

### 2026-01-25 - feat: Add foundation crate with config and tracing
- `crates/backend/foundation/Cargo.toml`: Foundation crate dependencies
- `crates/backend/foundation/src/lib.rs`: Foundation module exports
- `crates/backend/foundation/src/config.rs`: AppConfig and ServerConfig with tests
- `crates/backend/foundation/src/tracing_init.rs`: Tracing initialization with tests

### 2026-01-25 - feat: Add server crate with Axum and health endpoint
- `crates/backend/server/Cargo.toml`: Server crate dependencies
- `crates/backend/server/src/lib.rs`: Server run functions
- `crates/backend/server/src/router.rs`: Router configuration with tests
- `crates/backend/server/src/health.rs`: Health check endpoint with tests
