---
advance:
  id: "ADV-INV-004"
  title: "Serial and lot tracking with genealogy queries (where-used, where-shipped)"
  system: "backend"
  primary_component: "inventory"
  components: ["inventory", "persistence"]
  started_at: "2026-01-26T04:15:00Z"
  implementation_completed_at: "2026-01-26T05:00:00Z"
  pr_links: []
  reviewability_score: 22
  risk_flags: []
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Implement serial and lot tracking genealogy queries to answer where-used (full movement history) and where-shipped (shipment events) for serial numbers and lot numbers. Provide tests that validate correct lineage across transfers and shipments.

## Behavioral Change

After this advance:
- Persistence exposes genealogy helpers:
  - `get_serial_genealogy(serial)` and `get_serial_shipments(serial)`
  - `get_lot_genealogy_by_number(lot_number)` and `get_lot_shipments_by_number(lot_number)`
- Tests assert correct order and contents of movement histories across receipt → transfer → shipment.

## Completed Implementation Tasks

- [x] tidy: augment inventory repository with record access helpers
- [x] test: unit/integration tests for serial and lot genealogy
- [x] feat: add genealogy query functions for serials and lots

## Risk + Rollback

- Risk: Queries use LIKE on JSON array stored as text for serial matching; acceptable for current scope, may evolve to normalized schema later.
- Rollback: Remove helpers and tests; no schema changes introduced in this advance.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit (trigger invariants reused; order-controlled inserts)
- [x] tests:integration (serial and lot genealogy and shipment extraction)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - feat: Genealogy queries and tests
- `crates/backend/persistence/src/inventory_repository.rs`:
  - Added `get_serial_genealogy`, `get_serial_shipments`, `get_lot_genealogy_by_number`, `get_lot_shipments_by_number`.
  - Added tests that seed items/locations/lots and insert receipt/transfer/shipment movements with explicit timestamps for deterministic ordering.

