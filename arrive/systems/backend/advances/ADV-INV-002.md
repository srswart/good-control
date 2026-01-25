---
advance:
  id: "ADV-INV-002"
  title: "Inventory ledger append-only invariants + current-position query"
  system: "backend"
  primary_component: "inventory"
  components: ["inventory", "persistence"]
  started_at: "2026-01-26T02:00:00Z"
  implementation_completed_at: "2026-01-26T03:00:00Z"
  pr_links: []
  reviewability_score: 20
  risk_flags: ["migration"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Implement and validate the append-only inventory movement ledger and provide a current-position query by SKU/location/status spanning the inventory domain and persistence layer.

## Behavioral Change

After this advance:
- Inventory movements are enforced as append-only (updates/deletes abort).
- Persistence exposes a `get_current_position(sku, location_id, status)` query for callers.
- Tests cover ledger invariants and current-position lookups against SQLite.

## Completed Implementation Tasks

- [x] tidy: added repository module structure for inventory records
- [x] test: unit/integration tests for append-only triggers and position query
- [x] feat: added `get_current_position` helper to persistence

## Risk + Rollback

- Risk: none beyond existing schema; uses current migration; tests rely on SQLite triggers.
- Rollback: remove helper function and tests; no schema changes introduced by this advance.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit (trigger invariant tests)
- [x] tests:integration (position query end-to-end)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - feat: Position query and ledger invariants tests
- `crates/backend/persistence/src/inventory_repository.rs`: Added `get_current_position` and tests. Added record structs previously.
- Tests insert positions and verify query by SKU/location/status. Ledger tests assert update/delete on `inventory_movements` abort due to triggers.

### 2026-01-26 - test: SQLite tuning for persistence tests
- Tests use file-backed SQLite with `WAL` and `busy_timeout` to improve reliability under concurrent access.

