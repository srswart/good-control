---
advance:
  id: "ADV-BACKEND-003"
  title: "Rename Fly app to good-control-api"
  system: "backend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-02-08T05:00:00Z"
  implementation_completed_at: "2026-02-08T05:15:00Z"
  pr_links: []
  reviewability_score: 14
  risk_flags: ["network"]
  evidence: []
  status: complete
---

## Objective

Rename the backend Fly app from `good-control-backend` to `good-control-api` and update deploy tooling and documentation accordingly.

## Behavioral Change

After this advance:
- Backend deploy configuration and docs refer to `good-control-api`
- Health check and smoke checks target the new hostname

## Planned Implementation Tasks

- [x] tidy: confirm backend deploy config locations
- [x] feat: update backend Fly app name and deploy scripts
- [x] docs: update backend deploy documentation

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

### 2026-02-08 - feat: rename backend Fly app
- `apps/server/fly.toml`: Updated app name to `good-control-api`
- `scripts/deploy-backend-fly.sh`: Updated default app name and help text
- `apps/server/README.md`: Updated deploy references and health URL
- `arrive/systems/backend/advances/ADV-BACKEND-002.md`: Updated sprite name reference
