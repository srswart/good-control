---
advance:
  id: "ADV-FE-001"
  title: "Frontend Foundation (Dioxus 0.7.x, Tailwind pipeline, desktop packaging): initial implementation"
  system: "frontend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: ["new_dependency", "public_api"]
  evidence: []
  status: planned
---

## Objective

Create the first working slice of the foundation component and establish its public contracts, tests, and wiring for the frontend system.

## Behavioral Change

After this advance:
- The frontend system has a defined, test-backed foundation component boundary, with the minimal scaffolding needed to support subsequent feature advances.

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
