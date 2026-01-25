---
advance:
  id: "ADV-INV-001"
  title: "Inventory ledger, reservations, serial/lot traceability: initial implementation"
  system: "backend"
  primary_component: "inventory"
  components: ["inventory"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T21:10:00Z"
  pr_links: []
  reviewability_score: 24
  risk_flags: []
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Create the first working slice of the inventory component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed inventory component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Completed Implementation Tasks

- [x] tidy: prepare persistence interfaces for inventory
- [x] test: add integration tests for reserve/release/ship; concurrent reservations
- [x] feat: implement inventory schema, position updates, and movement ledger

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit
- [x] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-25 - feat: Inventory schema + service
- migrations/20260125230000_inventory_schema.sql: Items, locations, serial_units, lots, inventory_movements (append-only), inventory_positions, indexes, triggers.
- migrations/20260125230000_inventory_schema.down.sql: Rollback scripts.
- crates/backend/persistence/src/inventory_repository.rs: Added record types for items, locations, positions, movements.
- crates/backend/persistence/src/inventory_service.rs: Implemented `reserve_inventory`, `release_reservation`, `ship_reserved_inventory`, and `reserve_and_ship` with transaction boundaries and tests.
