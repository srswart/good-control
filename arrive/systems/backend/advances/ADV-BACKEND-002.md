---
advance:
  id: "ADV-BACKEND-002"
  title: "Deploy inventory backend to Fly.io sprite"
  system: "backend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-02-07T12:00:00Z"
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: ["network"]
  evidence: []
  status: planned
---

## Objective

Deploy the inventory backend to Fly.io using a new sprite named `good-control-api`, with repeatable configuration and a documented rollout/rollback path.

## Behavioral Change

After this advance:
- The backend runs on Fly.io as a managed service
- The deployment is reproducible from repo configuration
- Health checks and environment configuration are enforced by the platform
- Operators have a defined rollback path for failed deploys

## Planned Implementation Tasks

- [x] tidy: audit backend build/run entrypoints and document required env vars
- [x] test: define a deploy smoke check against `/health`
- [x] feat: add Fly.io app configuration (`fly.toml`) for the backend sprite
- [x] feat: add Dockerfile/build config for the backend server binary
- [x] feat: configure Fly.io secrets and runtime env (db path, JWT, GraphQL)
- [x] feat: provision storage/volume for SQLite persistence if needed
- [x] feat: add deploy helper script
- [x] fix: create app if missing during deploy
- [ ] test: run smoke checks post-deploy and capture evidence

## Component Impact

```yaml
components: [foundation]
```

- **foundation**: Fly.io deployment configuration for the backend server

## Risk + Rollback

- Risk: Misconfigured secrets or volumes could prevent startup or cause data loss
- Mitigation: Health checks, staged deploy, and explicit volume mount settings
- Rollback: Revert to previous Fly.io release and reattach volume

## Evidence

- [ ] tidy:preparatory - Backend entrypoints and env vars documented
- [ ] tdd:red-green - Smoke checks defined before deploy wiring
- [ ] tests:integration - Health check passes in deployed environment
- [ ] ci:passed - Build and deploy pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - tidy: document backend deploy entrypoints
- `apps/server/README.md`: Documented runtime env vars and Fly.io deploy steps

### 2026-02-07 - test: define deploy smoke check
- `apps/server/README.md`: Added `/health` smoke check command

### 2026-02-07 - feat: add Fly.io deployment config
- `apps/server/fly.toml`: Added Fly app config with health checks and volume mount
- `apps/server/Dockerfile`: Added multi-stage image build for `inventory-server`
- `.dockerignore`: Added Docker build ignore rules for faster builds

### 2026-02-07 - feat: allow DB config via env
- `crates/backend/persistence/src/pool.rs`: Added env overrides for DB config with tests
- `apps/server/src/main.rs`: Load database config from environment

### 2026-02-07 - feat: add deploy helper script
- `scripts/deploy-backend-fly.sh`: Added Fly deploy script with volume and smoke check
- `apps/server/README.md`: Updated deploy instructions to use the script

### 2026-02-07 - fix: ensure app creation during deploy
- `scripts/deploy-backend-fly.sh`: Added `--create-app` option with org support
- `apps/server/README.md`: Documented app creation in deploy command

### 2026-02-07 - fix: reduce Docker build context
- `.dockerignore`: Excluded UI/apps/tests/docs/arrive from backend build context

### 2026-02-07 - fix: include domain crates in build context
- `.dockerignore`: Restored `crates/domain` to ensure workspace builds succeed

### 2026-02-07 - fix: include CLI crates in build context
- `.dockerignore`: Restored `crates/cli` for workspace manifest consistency

### 2026-02-07 - fix: include UI crates in build context
- `.dockerignore`: Restored `crates/ui` for workspace manifest consistency

### 2026-02-07 - fix: restore full build context
- `.dockerignore`: Removed workspace exclusions to avoid missing crates/apps

### 2026-02-07 - feat: add context size check
- `scripts/deploy-backend-fly.sh`: Added Docker context size and target/ check

### 2026-02-07 - fix: avoid blocking on context size check
- `scripts/deploy-backend-fly.sh`: Added timeout guard for buildx size check

### 2026-02-07 - fix: exclude local virtualenv from Docker context
- `.dockerignore`: Added `.venv` to reduce context size

### 2026-02-07 - feat: add context inspection report
- `scripts/docker-context-report.py`: Added Docker context size reporter
- `scripts/deploy-backend-fly.sh`: Added `--context-report` option

### 2026-02-07 - fix: exclude frontend/test node_modules from context
- `.dockerignore`: Excluded UI/test node_modules and Playwright report artifacts

### 2026-02-07 - fix: align Fly primary region with volume
- `apps/server/fly.toml`: Set `primary_region` to `sin`
- `apps/server/README.md`: Updated deploy example to match region

### 2026-02-07 - fix: avoid creating duplicate volumes
- `scripts/deploy-backend-fly.sh`: Skip volume creation when one exists
