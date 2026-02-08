---
advance:
  id: "ADV-FE-005"
  title: "Role-based module access control"
  system: "frontend"
  primary_component: "app-shell"
  components: ["app-shell"]
  started_at: "2026-02-07T16:15:00Z"
  implementation_completed_at: "2026-02-07T17:00:00Z"
  pr_links: []
  reviewability_score: 18
  risk_flags: []
  evidence: []
  status: complete
---

## Objective

Restrict navigation and route access based on user roles mapped to UI modules.

## Behavioral Change

After this advance:
- Navigation items are filtered by role
- Unauthorized routes show an access-denied screen

## Planned Implementation Tasks

- [x] feat: add role-to-route access map
- [x] feat: filter navigation items by session access
- [x] feat: guard routes in AppLayout

## Component Impact

```yaml
components: [app-shell]
```

## Risk + Rollback

- Risk: Incorrect mappings could hide required screens
- Mitigation: Admin role bypass and simple access-denied fallback
- Rollback: Remove route guard and nav filtering

## Evidence

- [ ] tests:unit - Route access checks
- [ ] ci:passed - CI pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: role-based module access
- `crates/ui/app_shell/src/session.rs`: Added role-to-route access map
- `crates/ui/app_shell/src/nav.rs`: Filtered default navigation by access
- `crates/ui/app_shell/src/layout.rs`: Added access-denied guard for routes
