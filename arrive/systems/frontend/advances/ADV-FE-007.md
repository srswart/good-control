---
advance:
  id: "ADV-FE-007"
  title: "Admin user management UI"
  system: "frontend"
  primary_component: "app_shell"
  components: ["app_shell", "ui"]
  started_at: "2026-02-07T18:35:00Z"
  implementation_completed_at: "2026-02-07T19:10:00Z"
  pr_links: []
  reviewability_score: 32
  risk_flags: ["auth", "ui"]
  evidence: []
  status: complete
---

## Objective

Provide an admin-only UI to view users, manage roles, and toggle active status.

## Behavioral Change

After this advance:
- Admins can navigate to `/admin/users` to manage users and roles.
- The admin route is hidden and blocked for non-admin users.

## Planned Implementation Tasks

- [x] tidy: add admin route and nav entry point
- [x] test: add render checks for admin UI
- [x] feat: implement admin users UI with role assignment
- [x] feat: wire GraphQL queries/mutations for admin actions

## Component Impact

```yaml
components: [app_shell, ui]
```

- **app_shell**: route + navigation + access gating
- **ui**: admin users view components and GraphQL wiring

## Risk + Rollback

- Risk: Admin UI reachable by non-admin users
- Mitigation: Route gating checks admin role; GraphQL enforces permissions
- Rollback: Remove admin route and UI module

## Evidence

- [ ] test:ui - Admin view render test passing
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: admin user management UI
- `crates/ui/admin/`: Added admin UI crate with user management view
- `crates/ui/app_shell/src/routes.rs`: Added `/admin/users` route
- `crates/ui/app_shell/src/nav.rs`: Added admin nav item with role gating
- `crates/ui/app_shell/src/session.rs`: Added admin route access gate
- `crates/ui/app_shell/src/i18n.rs`: Added admin route label translations
