---
advance:
  id: "ADV-INVUI-001"
  title: "Inventory UI (items, stock, traceability, cycle counts): initial implementation"
  system: "frontend"
  primary_component: "inventory-ui"
  components: ["inventory-ui"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Create the first working slice of the inventory-ui component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed inventory-ui component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Planned Implementation Tasks

- [ ] tidy: preparatory refactoring (no behavior change)
- [ ] test: add/adjust tests describing expected behavior
- [ ] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner
