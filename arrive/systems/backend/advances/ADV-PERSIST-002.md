---
advance:
  id: "ADV-PERSIST-002"
  title: "Transaction helper and atomic multi-step operations"
  system: "backend"
  primary_component: "persistence"
  components: ["persistence", "inventory"]
  started_at: "2026-01-26T00:00:00Z"
  implementation_completed_at: "2026-01-26T01:30:00Z"
  pr_links: []
  reviewability_score: 26
  risk_flags: ["concurrency"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Introduce transaction helper utilities and repository traits to ensure multi-step inventory operations (reserve → allocate → ship) are atomic and consistent under concurrent access.

## Behavioral Change

After this advance:
- Transaction helper wraps SQLx transactions with ergonomic API
- Repository operations support transactional execution
- Multi-step operations (reserve, allocate, ship) can be composed atomically
- Tests simulate concurrent operations (race conditions)
- Tests validate inventory consistency under concurrent load
- Transaction rollback on error prevents partial state
- Isolation ensures serializable operations

## Completed Implementation Tasks

- [x] tidy: preparatory refactoring (crate structure, module boundaries)
- [x] test: add tests for transaction helper + concurrent ops
- [x] feat/fix: implement transaction helper, repository trait, and inventory service ops

## Risk + Rollback

- Risk: Transaction overhead may impact performance. Deadlocks possible under high concurrency. Serializable isolation may cause retries. Long transactions could block other operations.
- Rollback: Remove transaction wrapper and revert to non-transactional operations. No schema changes, so rollback is clean.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit (transaction helper commit/rollback, explicit commit, concurrency)
- [x] tests:integration (reserve/release/ship flows against SQLite + migrations)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - feat: Transaction helper + repository traits
- crates/backend/persistence/src/transaction.rs: Added `TransactionContext`, `with_transaction`, `with_transaction_explicit`, `Transactional` trait, and unit tests including concurrency simulation.
- crates/backend/persistence/src/repository.rs: Introduced generic `Repository<T, ID>` async trait with compile-test.

### 2026-01-26 - feat: Inventory service atomic ops + tests
- crates/backend/persistence/src/inventory_service.rs: Implemented `InventoryService` with `reserve_inventory`, `release_reservation`, `ship_reserved_inventory`, and `reserve_and_ship` using transaction helper; added tests covering concurrent reservations and consistency checks.

### 2026-01-26 - test: Inventory schema in-place for integration
- migrations/20260125230000_inventory_schema.sql: Items, locations, serial_units, lots, inventory_movements (append-only), inventory_positions, indexes, and triggers.
- migrations/20260125230000_inventory_schema.down.sql: Corresponding rollback scripts.
