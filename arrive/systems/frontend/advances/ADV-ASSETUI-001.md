---
advance:
  id: "ADV-ASSETUI-001"
  title: "Assets UI (drones + batteries lifecycle views): initial implementation"
  system: "frontend"
  primary_component: "assets-ui"
  components: ["assets-ui"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-26T00:00:00Z"
  pr_links: []
  reviewability_score: 22
  risk_flags: []
  evidence:
    - tidy:preparatory
    - tdd:red-green
    - tests:unit
  status: completed
---

## Objective

Create the first working slice of the assets-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed assets-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Implementation Summary

### Completed Features

1. **Assets UI Crate Structure**
   - Created `crates/ui/assets` crate with modular component structure
   - Separated into modules: `drones`, `batteries`
   - Each module exports a main view component

2. **Drones Component**
   - `DronesView` component displaying drone assets in a table
   - Shows Serial, Model, Configuration Version, Firmware, Readiness State, and Location
   - Placeholder data structure ready for GraphQL integration
   - Supports lifecycle states: ready, in_repair, grounded

3. **Batteries Component**
   - `BatteriesView` component displaying batteries in a table
   - Shows Serial, Model, Cycles, Last Test Date, Lifecycle State, Assignment Pool, and Location
   - Ready for backend integration
   - Supports lifecycle states: ready, grounded, retired

4. **Integration with App Shell**
   - Wired into `/assets` route with tabbed interface
   - Tab navigation between Drones and Batteries
   - Integrated with i18n system for translations

5. **Test Coverage**
   - Unit tests for each component (2 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/assets/Cargo.toml` - Crate manifest
- `crates/ui/assets/src/lib.rs` - Library exports
- `crates/ui/assets/src/drones.rs` - Drones view component and tests
- `crates/ui/assets/src/batteries.rs` - Batteries view component and tests

### Files Modified

- `Cargo.toml` - Added ui-assets to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Assets route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added assets UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-assets dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations
- Lifecycle states align with domain model (ready, grounded, in_repair, retired)

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 2 unit tests passing for assets components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
