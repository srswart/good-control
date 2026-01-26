---
advance:
  id: "ADV-COMPUI-001"
  title: "Compliance UI (approvals, doc packs, blocking reasons): initial implementation"
  system: "frontend"
  primary_component: "compliance-ui"
  components: ["compliance-ui"]
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

Create the first working slice of the compliance-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed compliance-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Implementation Summary

### Completed Features

1. **Compliance UI Crate Structure**
   - Created `crates/ui/compliance` crate with modular component structure
   - Separated into modules: `approvals`, `doc_packs`, `blocking_reasons`
   - Each module exports a main view component

2. **Approvals Component**
   - `ApprovalsView` component displaying compliance approvals in a table
   - Shows Request ID, Type, Requester, Status, Submitted date, and Reviewed By
   - Placeholder data structure ready for GraphQL integration

3. **Document Packs Component**
   - `DocPacksView` component displaying compliance document packs
   - Shows Pack ID, Name, Jurisdiction, Status, Created date, and Document count
   - Ready for backend integration

4. **Blocking Reasons Component**
   - `BlockingReasonsView` component displaying compliance blocking reasons
   - Shows Entity Type, Entity ID, Reason, Severity, Created date, and Resolved date
   - Supports compliance blocking requirements

5. **Integration with App Shell**
   - Wired into `/compliance` route with tabbed interface
   - Tab navigation between Approvals, Document Packs, and Blocking Reasons
   - Integrated with i18n system for translations

6. **Test Coverage**
   - Unit tests for each component (3 tests total)
   - All tests passing
   - Tests verify component rendering

### Files Created

- `crates/ui/compliance/Cargo.toml` - Crate manifest
- `crates/ui/compliance/src/lib.rs` - Library exports
- `crates/ui/compliance/src/approvals.rs` - Approvals view component and tests
- `crates/ui/compliance/src/doc_packs.rs` - Document packs view component and tests
- `crates/ui/compliance/src/blocking_reasons.rs` - Blocking reasons view component and tests

### Files Modified

- `Cargo.toml` - Added ui-compliance to workspace members and dependencies
- `crates/ui/app_shell/src/routes.rs` - Updated Compliance route to use new components
- `crates/ui/app_shell/src/i18n.rs` - Added compliance UI translations
- `crates/ui/app_shell/Cargo.toml` - Added ui-compliance dependency

### Architecture Notes

- Components use placeholder data structures matching backend domain model
- Ready for GraphQL integration in future advances
- Follows modular crate-per-area pattern
- Uses design system components (Table) for consistency
- Supports i18n with English and Chinese translations

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then components implemented
- [x] tests:unit - 3 unit tests passing for compliance components
- [ ] tests:integration - Integration tests deferred (requires GraphQL setup)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
