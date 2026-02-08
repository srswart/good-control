---
advance:
  id: "ADV-FE-008"
  title: "Admin mock data reload UI"
  system: "frontend"
  primary_component: "app_shell"
  components: ["app_shell", "ui"]
  started_at: "2026-02-08T13:31:00Z"
  implementation_completed_at: "2026-02-08T14:55:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: ["admin_ui"]
  evidence: []
  status: complete
---

## Objective

Expose an admin-only control to reload mock data in the admin area.

## Behavioral Change

After this advance:
- Admins can trigger mock data reload from the admin area.
- The UI shows a destructive warning and displays summary counts.

## Planned Implementation Tasks

- [x] tidy: add admin mock data panel
- [x] test: add render coverage for the panel
- [x] feat: wire GraphQL mutation and results

## Component Impact

```yaml
components: [app_shell, ui]
```

- **app_shell**: admin area placement
- **ui**: mock data reload panel

## Risk + Rollback

- Risk: Accidental data loss if triggered unintentionally
- Mitigation: Explicit destructive warning in UI
- Rollback: Remove panel and mutation usage

## Evidence

- [ ] test:ui - Admin mock data panel render test passing
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-08 - feat: admin mock data reload panel
- `crates/ui/admin/src/users.rs`: Added mock data reload section and summary UI
