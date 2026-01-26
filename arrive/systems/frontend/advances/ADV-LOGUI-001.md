---
advance:
  id: "ADV-LOGUI-001"
  title: "Logistics UI (shipments, tracking, cross-border pack review): initial implementation"
  system: "frontend"
  primary_component: "logistics-ui"
  components: ["logistics-ui"]
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

Create the first working slice of the logistics-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed logistics-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Implementation Summary

### Completed Features

1. **Logistics UI Crate Structure**
   - Created `crates/ui/logistics` crate with modular component structure
   - Separated into modules: `shipments`, `tracking`, `cross_border_pack_review`
   - Each module exports a main view component

2. **Shipments Component**
   - `ShipmentsView` component displaying shipments in a table
   - Shows Shipment ID, Order ID, Carrier, Tracking Number, Status, Route (origin → destination), and Created date
   - Placeholder data structure ready for GraphQL integration

3. **Tracking Component**
   - `TrackingView` component displaying tracking events
   - Shows Tracking Number, Event Type, Location, Timestamp, Description, and Status
   - Supports shipment tracking workflow

4. **Cross-Border Pack Review Component**
   - `CrossBorderPackReviewView` component displaying cross-border shipments requiring compliance review
   - Shows Pack ID, Order ID, Route (origin → destination countries), Compliance Status, Review Status, Reviewed By, and Created date
   - Supports compliance review workflow for international shipments

5. **Integration with App Shell**
   - Wired into `/logistics` route with tabbed interface
   - Tab navigation between Shipments, Tracking, and Cross-Border Pack Review
   - Integrated with i18n system for translations

6. **Test Coverage**
   - Unit tests for each component (3 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/logistics/Cargo.toml` - Crate manifest
- `crates/ui/logistics/src/lib.rs` - Library exports
- `crates/ui/logistics/src/shipments.rs` - Shipments view component and tests
- `crates/ui/logistics/src/tracking.rs` - Tracking view component and tests
- `crates/ui/logistics/src/cross_border_pack_review.rs` - Cross-border pack review view component and tests

### Files Modified

- `Cargo.toml` - Added ui-logistics to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Logistics route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added logistics UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-logistics dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations
- Cross-border pack review integrates with compliance workflows

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 3 unit tests passing for logistics components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
