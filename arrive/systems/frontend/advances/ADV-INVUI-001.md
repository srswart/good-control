---
advance:
  id: "ADV-INVUI-001"
  title: "Inventory UI (items, stock, traceability, cycle counts): initial implementation"
  system: "frontend"
  primary_component: "inventory-ui"
  components: ["inventory-ui"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-26T00:00:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: []
  evidence:
    - tidy:preparatory
    - tdd:red-green
    - tests:unit
  status: completed
---

## Objective

Create the first working slice of the inventory-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed inventory-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Implementation Summary

### Completed Features

1. **Inventory UI Crate Structure**
   - Created `crates/ui/inventory` crate with modular component structure
   - Separated into modules: `items`, `stock`, `traceability`, `cycle_counts`
   - Each module exports a main view component

2. **Items Component**
   - `ItemsList` component displaying inventory items in a table
   - Shows SKU, Name, Type, and Tracking mode
   - Placeholder data structure ready for GraphQL integration

3. **Stock Component**
   - `StockView` component displaying inventory positions
   - Shows SKU, Location, Status, On Hand, and Available quantities
   - Ready for backend integration

4. **Traceability Component**
   - `TraceabilityView` component displaying movement history
   - Shows Timestamp, SKU, Reason, From/To locations, Quantity, and Actor
   - Supports traceability requirements

5. **Cycle Counts Component**
   - `CycleCountsView` component displaying cycle count tasks
   - Shows Location, Scheduled Date, Status, and Counted By
   - Ready for warehouse integration

6. **Integration with App Shell**
   - Wired into `/inventory` route with tabbed interface
   - Tab navigation between Items, Stock, Traceability, and Cycle Counts
   - Integrated with i18n system for translations

7. **Test Coverage**
   - Unit tests for each component (4 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/inventory/Cargo.toml` - Crate manifest
- `crates/ui/inventory/src/lib.rs` - Library exports
- `crates/ui/inventory/src/items.rs` - Items list component and tests
- `crates/ui/inventory/src/stock.rs` - Stock view component and tests
- `crates/ui/inventory/src/traceability.rs` - Traceability view component and tests
- `crates/ui/inventory/src/cycle_counts.rs` - Cycle counts view component and tests

### Files Modified

- `Cargo.toml` - Added ui-inventory to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Inventory route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added inventory UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-inventory dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 4 unit tests passing for inventory components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
