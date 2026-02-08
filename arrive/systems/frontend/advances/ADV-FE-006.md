---
advance:
  id: "ADV-FE-006"
  title: "Rename Fly app to good-control"
  system: "frontend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-02-08T05:00:00Z"
  implementation_completed_at: "2026-02-08T05:15:00Z"
  pr_links: []
  reviewability_score: 16
  risk_flags: ["network"]
  evidence: []
  status: complete
---

## Objective

Rename the frontend Fly app from `good-control-web` to `good-control` and update deploy tooling and documentation accordingly.

## Behavioral Change

After this advance:
- Frontend deploy configuration and docs refer to `good-control`
- Runtime GraphQL endpoint defaults to the renamed backend app hostname

## Planned Implementation Tasks

- [x] tidy: confirm frontend deploy config locations
- [x] feat: update frontend Fly app name, endpoint, and deploy scripts
- [x] docs: update frontend deploy documentation

## Component Impact

```yaml
components: [foundation]
```

## Risk + Rollback

- Risk: DNS or app mismatch after rename could break deploys
- Mitigation: Keep old app until the new one is verified
- Rollback: Revert config to old app name and redeploy

## Evidence

- [ ] tests:smoke - `/health` reachable on new app
- [ ] ci:passed - CI pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-08 - feat: rename frontend Fly app
- `apps/web/fly.toml`: Updated app name to `good-control` and GraphQL endpoint to `good-control-api`
- `scripts/deploy-frontend-fly.sh`: Updated default app name and help text
- `apps/web/README.md`: Updated deploy references and health URL
- `arrive/systems/frontend/advances/ADV-FE-003.md`: Updated sprite name reference
