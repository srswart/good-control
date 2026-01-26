---
advance:
  id: "ADV-WMSUI-001"
  title: "Warehouse UI (receiving, putaway, pick/pack/ship; scan-first): initial implementation"
  system: "frontend"
  primary_component: "warehouse-ui"
  components: ["warehouse-ui"]
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

Create the first working slice of the warehouse-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed warehouse-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Implementation Summary

### Completed Features

1. **Warehouse UI Crate Structure**
   - Created `crates/ui/warehouse` crate with modular component structure
   - Separated into modules: `receiving`, `putaway`, `pick`, `pack`, `ship`
   - Each module exports a main view component

2. **Receiving Component**
   - `ReceivingView` component displaying receiving tasks in a table
   - Shows Task ID, PO Number, Progress (received/expected), Status, Assigned To, and Created date
   - Placeholder data structure ready for GraphQL integration

3. **Putaway Component**
   - `PutawayView` component displaying putaway tasks
   - Shows Task ID, SKU, Quantity, From/To locations, Status, and Assigned To
   - Ready for backend integration

4. **Pick Component**
   - `PickView` component displaying pick tasks for order fulfillment
   - Shows Task ID, Order ID, SKU, Quantity, From Location, Status, and Assigned To
   - Supports order picking workflow

5. **Pack Component**
   - `PackView` component displaying pack tasks
   - Shows Task ID, Order ID, Items count, Status, Assigned To, and Created date
   - Ready for packaging workflow integration

6. **Ship Component**
   - `ShipView` component displaying ship tasks
   - Shows Task ID, Order ID, Carrier, Tracking Number, Status, and Assigned To
   - Supports shipping workflow with carrier and tracking information

7. **Integration with App Shell**
   - Wired into `/warehouse` route with tabbed interface
   - Tab navigation between Receiving, Putaway, Pick, Pack, and Ship
   - Integrated with i18n system for translations

8. **Test Coverage**
   - Unit tests for each component (5 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/warehouse/Cargo.toml` - Crate manifest
- `crates/ui/warehouse/src/lib.rs` - Library exports
- `crates/ui/warehouse/src/receiving.rs` - Receiving view component and tests
- `crates/ui/warehouse/src/putaway.rs` - Putaway view component and tests
- `crates/ui/warehouse/src/pick.rs` - Pick view component and tests
- `crates/ui/warehouse/src/pack.rs` - Pack view component and tests
- `crates/ui/warehouse/src/ship.rs` - Ship view component and tests

### Files Modified

- `Cargo.toml` - Added ui-warehouse to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Warehouse route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added warehouse UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-warehouse dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations
- Note: Scan-first patterns will be implemented in future advances (mobile-specific workflows)

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 5 unit tests passing for warehouse components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
