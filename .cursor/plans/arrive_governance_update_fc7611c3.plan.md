---
name: ARRIVE Governance Update
overview: Create a CLI-system advance and update Cursor rules, ARRIVE templates, and workflow docs to reflect CI pipeline guidance and reviewability time tracking, plus document an `arrive upgrade` command for updating older app instances without losing local customizations.
todos:
  - id: advance-arrive
    content: Create ADV-ARRIVE-001 under cli system
    status: completed
  - id: rules-update
    content: Update Cursor rules for CI + time tracking
    status: completed
  - id: templates-update
    content: Add/update ARRIVE templates for new fields
    status: completed
  - id: upgrade-docs
    content: Document arrive upgrade command + preservation steps
    status: completed
  - id: validate
    content: Run arrive status/score and update evidence
    status: completed
isProject: false
---

# ARRIVE Governance Update Plan

## Advance + Branch

- Create new CLI-system advance `ADV-ARRIVE-001` under `[arrive/systems/cli/advances/]( /Users/shawn/Projects/035-inventory/arrive/systems/cli/advances/)` documenting governance updates.
- Create branch `cli/ADV-ARRIVE-001` per `[docs/DEVELOPMENT_WORKFLOW.md](/Users/shawn/Projects/035-inventory/docs/DEVELOPMENT_WORKFLOW.md)`.

## Cursor Rules Updates

- Update ARRIVE rules to include CI pipeline expectations and reviewability time tracking:
  - `[.cursor/rules/arrive-core.mdc](/Users/shawn/Projects/035-inventory/.cursor/rules/arrive-core.mdc)` – add explicit mention of CI pipeline steps and reviewability time tracking rules.
  - `[.cursor/rules/arrive-reviewability.mdc](/Users/shawn/Projects/035-inventory/.cursor/rules/arrive-reviewability.mdc)` – add guidance on capturing reviewability time in evidence (and how it’s calculated from registry).
  - `[.cursor/rules/arrive-advance-writing.mdc](/Users/shawn/Projects/035-inventory/.cursor/rules/arrive-advance-writing.mdc)` – add template fields/notes for reviewability time tracking and CI evidence.

## Templates

- Add or update ARRIVE templates to reflect the new CI + reviewability time tracking fields:
  - Create `[arrive/templates/advance.md](/Users/shawn/Projects/035-inventory/arrive/templates/advance.md)` (new) with frontmatter placeholders for `started_at`, `implementation_completed_at`, `reviewability_score`, and CI evidence.
  - If you want to avoid new templates, fold the same guidance into `[docs/DEVELOPMENT_WORKFLOW.md](/Users/shawn/Projects/035-inventory/docs/DEVELOPMENT_WORKFLOW.md)` and note it in the advance.

## Upgrade Command Documentation

- Add a short upgrade guide that preserves local customizations and highlights changes:
  - Update `[docs/DEVELOPMENT_WORKFLOW.md](/Users/shawn/Projects/035-inventory/docs/DEVELOPMENT_WORKFLOW.md)` (or add `docs/ARRIVE_UPGRADE.md`) with:
    - `arrive upgrade --dry-run --preserve-local --report`
    - Steps to review differences in rules/templates and a checklist for reconciling local edits.
    - Guidance on identifying impacted “operating parts” (rules, templates, CI docs).

## Validation

- Run `arrive status` / `arrive score` to confirm advance changes are recognized.
- Ensure new templates/rules are referenced from docs and the advance.

## Notes

- CLI system roots currently cover `apps/cli/**` and `crates/cli/**`. Because this work touches `.cursor/**`, `arrive/**`, and `docs/**`, the advance will explicitly record cross-system impact in its “Component Impact” section and list these paths in the change log.

