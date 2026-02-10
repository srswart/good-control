---
advance:
  id: "ADV-WMS-001"
  title: "Warehouse operations (receiving/putaway/pick/pack/ship): initial implementation"
  system: "backend"
  primary_component: "warehouse"
  components: ["warehouse"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-02-08T19:10:00Z"
  pr_links: []
  reviewability_score: 18
  risk_flags: []
  evidence: []
  status: complete
---

## Objective

Create the first working slice of the warehouse component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed warehouse component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Planned Implementation Tasks

- [x] tidy: preparatory refactoring (no behavior change)
- [x] test: add/adjust tests describing expected behavior
- [x] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory
- [ ] tdd:red-green
- [x] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-02-08 - feat: warehouse domain foundation

- `crates/domain/warehouse/Cargo.toml`: Warehouse domain crate definition
- `crates/domain/warehouse/src/lib.rs`: Module exports and component boundary
- `crates/domain/warehouse/src/receiving.rs`: Receiving task entities and state machine
- `crates/domain/warehouse/src/picking.rs`: Picking workflow models and transitions
- `crates/domain/warehouse/src/packing.rs`: Packing workflow models and transitions
- `crates/domain/warehouse/src/shipping.rs`: Shipping workflow models and transitions
