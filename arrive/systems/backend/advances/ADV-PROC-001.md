---
advance:
  id: "ADV-PROC-001"
  title: "Procurement + suppliers + inbound docs: initial implementation"
  system: "backend"
  primary_component: "procurement"
  components: ["procurement"]
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

Create the first working slice of the procurement component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed procurement component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Implementation Summary

### Completed Features

1. **Procurement Domain Crate Structure**
   - Created `crates/domain/procurement` crate with modular domain models
   - Separated into modules: `error`, `supplier`, `purchase_order`, `inbound_document`
   - Each module contains domain entities and business logic

2. **Error Module**
   - `ProcurementError` enum with domain-specific error types
   - Error types for supplier, purchase order, and inbound document operations
   - Result type alias for convenience

3. **Supplier Module**
   - `Supplier` entity with bilingual names (English/Chinese)
   - Contact information management (email, phone, address)
   - Active/inactive status management
   - Full test coverage for supplier operations

4. **Purchase Order Module**
   - `PurchaseOrder` entity with status lifecycle (Draft → Pending → Approved → Sent → Received)
   - `PurchaseOrderLine` entity for line items with quantity tracking
   - State transition validation (prevents invalid status changes)
   - Receipt tracking with validation (cannot receive more than ordered)
   - Total amount calculation
   - Full test coverage including workflow tests

5. **Inbound Document Module**
   - `InboundDocument` entity with multiple document types
   - Document status management (Pending → Approved/Rejected → Archived)
   - Association with purchase orders
   - File path and metadata support
   - Full test coverage for document operations

6. **Test Coverage**
   - 21 unit tests total (all passing)
   - Tests cover entity creation, state transitions, validation, and serialization
   - Tests verify business logic and invariants

### Files Created

- `crates/domain/procurement/Cargo.toml` - Crate manifest
- `crates/domain/procurement/src/lib.rs` - Library exports
- `crates/domain/procurement/src/error.rs` - Error types
- `crates/domain/procurement/src/supplier.rs` - Supplier domain model and tests
- `crates/domain/procurement/src/purchase_order.rs` - Purchase order domain model and tests
- `crates/domain/procurement/src/inbound_document.rs` - Inbound document domain model and tests

### Files Modified

- `Cargo.toml` - Added procurement to workspace members and dependencies

### Architecture Notes

- Pure domain logic (no SQL, no HTTP) following domain-driven design principles
- State machines with validation for purchase orders and documents
- Bilingual support (English/Chinese) for supplier names
- Ready for repository layer implementation in future advances
- Follows same pattern as logistics domain crate
- All entities are serializable for GraphQL integration

## Evidence

- [x] tidy:preparatory - Crate structure prepared without changing behavior
- [x] tdd:red-green - Tests written before implementation, then domain models implemented
- [x] tests:unit - 21 unit tests passing for procurement domain models
- [ ] tests:integration - Integration tests deferred (requires repository layer)
- [ ] ci:passed - CI verification pending
- [ ] review:owner - Code review pending
