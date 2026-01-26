---
advance:
  id: "ADV-INV-003"
  title: "Reservations/allocations with partial allocation + conflict prevention (GraphQL exposed)"
  system: "backend"
  primary_component: "inventory"
  components: ["inventory", "graphql", "persistence"]
  started_at: "2026-01-26T03:05:00Z"
  implementation_completed_at: "2026-01-26T04:10:00Z"
  pr_links: []
  reviewability_score: 26
  risk_flags: ["concurrency", "public_api"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Add reservation/allocation capability for inventory, including partial allocation support and conflict prevention under concurrent access. Expose GraphQL mutations and add tests for rollback behavior and conflict scenarios.

## Behavioral Change

After this advance:
- Clients can reserve inventory via GraphQL with optional partial allocation.
- Persistence service prevents over-allocation and performs atomic updates.
- Tests validate partial allocation outcomes and conflicting requests.

## Completed Implementation Tasks

- [x] tidy: prepared persistence service surface for allocation
- [x] test: GraphQL integration tests for partial allocation and conflict prevention
- [x] feat: `reserve_inventory_with_partial` in persistence; `reserveInventory` GraphQL mutation

## Risk + Rollback

- Risk: Concurrency-related flakiness on SQLite under high contention; mitigated with WAL and busy_timeout; race-stress tests deferred.
- Rollback: Remove GraphQL fields and persistence method; no schema changes.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit (persistence-level)
- [x] tests:integration (GraphQL: partial allocation, conflict)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - feat: Reservation with partial allocation + GraphQL exposure
- `crates/backend/persistence/src/inventory_service.rs`: Added `reserve_inventory_with_partial` returning movement id and allocated qty; atomic transaction semantics.
- `crates/backend/graphql/src/types.rs`: Added `ReserveResponse` type.
- `crates/backend/graphql/src/mutation.rs`: Added `reserveInventory` mutation with `allowPartial` flag.
- `crates/backend/graphql/tests/integration_test.rs`: Added two tests covering partial allocation and conflict prevention.
