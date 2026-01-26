---
advance:
  id: "ADV-FE-001"
  title: "Frontend Foundation (Dioxus 0.7.x, Tailwind pipeline, desktop packaging): initial implementation"
  system: "frontend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-26T00:00:00Z"
  pr_links: []
  reviewability_score: 0
  risk_flags: ["new_dependency", "public_api"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit"]
  status: complete
---

## Objective

Create the first working slice of the foundation component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed foundation component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Planned Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - tidy: Create crates/ui/foundation crate structure
- Cargo.toml: Added ui-foundation crate to workspace members and dependencies
- crates/ui/foundation/Cargo.toml: Created foundation crate with Dioxus 0.7.x dependency
- crates/ui/foundation/src/lib.rs: Created foundation library with error_boundary module
- crates/ui/foundation/src/error_boundary.rs: Created error boundary component structure

### 2026-01-26 - tidy: Update Dioxus from 0.4 to 0.7.x in desktop app
- apps/desktop/Cargo.toml: Updated dioxus and dioxus-desktop to 0.7, added ui-foundation dependency
- apps/desktop/src/main.rs: Migrated from Dioxus 0.4 API (Scope-based) to 0.7 API (direct Element return)
- apps/desktop/src/main.rs: Updated test to use Dioxus 0.7 VirtualDom API with Mutations

### 2026-01-26 - tidy: Set up Tailwind pipeline
- assets/ui/package.json: Created npm package with Tailwind build scripts
- assets/ui/tailwind.config.cjs: Updated content paths to include crates/ui/** source files
- assets/ui/dist/: Created dist directory for compiled CSS output
- Cargo.toml: Updated time crate to compatible version (0.3.45) for Rust 1.87

### 2026-01-26 - test: Add tests for foundation component
- crates/ui/foundation/src/error_boundary.rs: Added test for ErrorBoundary component
- apps/desktop/src/main.rs: Fixed existing test to work with Dioxus 0.7 API

### 2026-01-26 - feat: Wire Tailwind CSS into desktop app
- apps/desktop/Dioxus.toml: Created Dioxus config with asset_dir = "assets"
- apps/desktop/assets/tailwind.css: Copied compiled Tailwind CSS from assets/ui/dist/
- apps/desktop/src/main.rs: Added Tailwind CSS loading via asset!() macro and document::Stylesheet

### 2026-01-26 - feat: Create foundation component with app bootstrap
- crates/ui/foundation/src/error_boundary.rs: Implemented minimal ErrorBoundary component
- crates/ui/foundation/src/lib.rs: Exported BoundaryError and ErrorBoundary types
