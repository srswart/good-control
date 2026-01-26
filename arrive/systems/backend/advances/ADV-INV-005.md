---
advance:
  id: "ADV-INV-005"
  title: "Cycle count sessions (ABC), variance workflows, and audit — GraphQL exposed"
  system: "backend"
  primary_component: "inventory"
  components: ["inventory", "graphql", "persistence"]
  started_at: "2026-01-26T05:10:00Z"
  implementation_completed_at: "2026-01-26T06:05:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: ["public_api"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:integration"]
  status: complete
---

## Objective

Add cycle count sessions following ABC rules, support variance capture workflows, and record audit events. Provide GraphQL mutations/queries and integration tests.

## Behavioral Change

After this advance:
- Clients can start/record/complete cycle counts via GraphQL.
- Variance calculation is automatic based on current `inventory_positions` expected quantity.
- Audit events are recorded for session start, record, and completion.

## Completed Implementation Tasks

- [x] tidy: add GraphQL types for cycle count session/line
- [x] feat: GraphQL mutations — `startCycleCount`, `recordCycleCount`, `completeCycleCount`
- [x] feat: GraphQL query — `cycleCountSession(id)` + subfield `lines`
- [x] test: GraphQL integration test for partial/variance and audit event existence

## Risk + Rollback

- Risk: Uses ad-hoc tables created in tests (no migration included in this slice). Future advance will formalize migrations.
- Rollback: Remove GraphQL types/resolvers and tests.

## Evidence

- [x] tdd:red-green
- [x] tests:integration (cycle count flow, variance correctness, audit events)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-26 - feat: Cycle count GraphQL API and tests
- `crates/backend/graphql/src/types.rs`: Added `CycleCountSession`, `CycleCountLine`.
- `crates/backend/graphql/src/query.rs`: Added `cycleCountSession` query with line subfield.
- `crates/backend/graphql/src/mutation.rs`: Added `startCycleCount`, `recordCycleCount`, `completeCycleCount` mutations, integrated audit events.
- `crates/backend/graphql/tests/integration_test.rs`: Added `test_cycle_count_flow_partial_and_audit`, with test-local schema setup for cycle count tables.

