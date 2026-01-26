---
advance:
  id: "ADV-POUI-001"
  title: "Procurement + Orders UI (POs, allocations, fulfillment): initial implementation"
  system: "frontend"
  primary_component: "procurement-orders-ui"
  components: ["procurement-orders-ui"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-26T00:00:00Z"
  pr_links: []
  reviewability_score: 25
  risk_flags: []
  evidence:
    - tidy:preparatory
    - tdd:red-green
    - tests:unit
  status: completed
---

## Objective

Create the first working slice of the procurement-orders-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed procurement-orders-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Implementation Summary

### Completed Features

1. **Procurement + Orders UI Crate Structure**
   - Created `crates/ui/procurement_orders` crate with modular component structure
   - Separated into modules: `purchase_orders`, `allocations`, `fulfillment`
   - Each module exports a main view component

2. **Purchase Orders Component**
   - `PurchaseOrdersView` component displaying purchase orders in a table
   - Shows PO Number, Supplier, Status, Total Amount, Created date, and Expected Delivery
   - Placeholder data structure ready for GraphQL integration

3. **Allocations Component**
   - `AllocationsView` component displaying order allocations
   - Shows Order ID, SKU, Requested/Allocated quantities, Status, From Location, and Created date
   - Supports inventory allocation tracking

4. **Fulfillment Component**
   - `FulfillmentView` component displaying order fulfillment status
   - Shows Order ID, Customer, Items count, Status, Fulfillment Date, and Created date
   - Supports order fulfillment workflow

5. **Integration with App Shell**
   - Wired into `/orders` route with tabbed interface
   - Tab navigation between Purchase Orders, Allocations, and Fulfillment
   - Integrated with i18n system for translations

6. **Test Coverage**
   - Unit tests for each component (3 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/procurement_orders/Cargo.toml` - Crate manifest
- `crates/ui/procurement_orders/src/lib.rs` - Library exports
- `crates/ui/procurement_orders/src/purchase_orders.rs` - Purchase orders view component and tests
- `crates/ui/procurement_orders/src/allocations.rs` - Allocations view component and tests
- `crates/ui/procurement_orders/src/fulfillment.rs` - Fulfillment view component and tests

### Files Modified

- `Cargo.toml` - Added ui-procurement-orders to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Orders route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added procurement-orders UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-procurement-orders dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations
- Allocations component integrates with inventory and warehouse workflows

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 3 unit tests passing for procurement-orders components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
