---
advance:
  id: "ADV-{{COMPONENT_ABBREV}}-001"
  title: "{{ADVANCE_TITLE}}"
  system: "backend"
  primary_component: "assets"
  components: ["assets"]
  started_at: "{{STARTED_AT}}"
  implementation_completed_at: ~
  review_time_estimate_minutes: 0
  review_time_actual_minutes: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

{{OBJECTIVE}}

## Behavioral Change

After this advance:
- {{BEHAVIORAL_CHANGE}}

## Planned Implementation Tasks

- [ ] tidy: preparatory refactoring (no behavior change)
- [ ] test: add/adjust tests describing expected behavior
- [ ] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: TODO
- Rollback: TODO

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit

## CI Evidence Notes

- If CI jobs are enabled, link pipeline evidence (`ci:passed`) from PR/MR and default-branch runs.
- If CI jobs are temporarily disabled, run checks externally before merge:
  - `arrive pr check --strict --json`
  - `arrive evidence record --advance <ADV-ID> --status passed` (include provider/run metadata when available)
  - Optional combined flow when provider lookup is available:
    - `arrive pr check --provider-status --advance <ADV-ID> --record-ci-evidence --json`

