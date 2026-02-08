---
advance:
  id: "ADV-FE-003"
  title: "Deploy Dioxus server to Fly.io sprite"
  system: "frontend"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-02-07T13:30:00Z"
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: ["network"]
  evidence: []
  status: planned
---

## Objective

Deploy the Dioxus web server to Fly.io as a separate sprite from the backend called "good-control", with environment-driven API targeting and a documented rollout/rollback path. Ensure the UI is connected to the backend and the app can be accessed.

## Behavioral Change

After this advance:
- The Dioxus web server runs on Fly.io as its own service
- Deployments are reproducible from repo configuration
- The UI can target the backend sprite via configured API base URL
- Operators have a defined rollback path for failed deploys

## Planned Implementation Tasks

- [x] tidy: confirm Dioxus web build entrypoint and runtime requirements
- [x] test: define a deploy smoke check for the root route
- [x] feat: add Fly.io app configuration (`fly.toml`) for the Dioxus sprite
- [x] feat: add Dockerfile/build config for the Dioxus server
- [x] feat: configure runtime env for API base URL and assets
- [x] feat: add deploy helper script
- [ ] test: run smoke checks post-deploy and capture evidence

## Component Impact

```yaml
components: [foundation]
```

- **foundation**: Fly.io deployment configuration for the Dioxus web server

## Risk + Rollback

- Risk: Incorrect API base URL breaks data fetching
- Mitigation: Health checks plus a staging deploy with real data
- Rollback: Revert to previous Fly.io release

## Evidence

- [ ] tidy:preparatory - Dioxus runtime requirements documented
- [ ] tdd:red-green - Smoke checks defined before deploy wiring
- [ ] tests:integration - Root route passes in deployed environment
- [ ] ci:passed - Build and deploy pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - tidy: document Dioxus deploy entrypoints
- `apps/web/README.md`: Documented build-time GraphQL endpoint and Fly deploy steps

### 2026-02-07 - test: define deploy smoke check
- `apps/web/nginx.conf`: Added `/health` endpoint for smoke checks

### 2026-02-07 - feat: add Fly.io deployment config
- `apps/web/fly.toml`: Added Fly app config with build args and health checks
- `apps/web/Dockerfile`: Added multi-stage build and Nginx runtime
- `scripts/deploy-frontend-fly.sh`: Added Fly deploy script for the web sprite
- `.dockerignore`: Excluded `apps/web/dist` from Docker context

### 2026-02-07 - feat: configure GraphQL endpoint targeting
- `crates/ui/foundation/src/graphql/endpoint.rs`: Added endpoint resolution helper
- `crates/ui/foundation/src/graphql/mod.rs`: Exported endpoint helper
- `crates/ui/foundation/src/lib.rs`: Re-exported endpoint helper
- `crates/ui/inventory/src/items.rs`: Use configured GraphQL endpoint
- `crates/ui/inventory/src/stock.rs`: Use configured GraphQL endpoint
- `crates/ui/inventory/src/traceability.rs`: Use configured GraphQL endpoint
- `crates/ui/inventory/src/graphql_demo.rs`: Use configured GraphQL endpoint
- `crates/ui/warehouse/src/receiving.rs`: Use configured GraphQL endpoint
- `crates/ui/warehouse/src/pick.rs`: Use configured GraphQL endpoint
- `crates/ui/warehouse/src/pack.rs`: Use configured GraphQL endpoint
- `crates/ui/warehouse/src/ship.rs`: Use configured GraphQL endpoint

### 2026-02-07 - feat: add runtime config for web deploy
- `apps/web/assets/config.js`: Added runtime config script (overwritten at startup)
- `apps/web/entrypoint.sh`: Writes config.js from `APP__GRAPHQL_ENDPOINT`
- `apps/web/Dockerfile`: Runs entrypoint before Nginx
- `apps/web/fly.toml`: Set runtime env for GraphQL endpoint
- `apps/web/src/main.rs`: Loads runtime config script
- `crates/ui/foundation/Cargo.toml`: Added wasm dependencies for window config
- `crates/ui/foundation/src/graphql/endpoint.rs`: Reads window config for wasm

### 2026-02-07 - fix: update Rust toolchain for Dioxus CLI
- `apps/web/Dockerfile`: Bumped builder image to Rust 1.88

### 2026-02-07 - fix: ensure web build output directory
- `apps/web/Dioxus.toml`: Set `out_dir` to `dist`

### 2026-02-07 - fix: normalize Dioxus build output for Docker
- `apps/web/Dockerfile`: Copy build output into `dist` when CLI writes elsewhere

### 2026-02-07 - fix: runtime config injection for deployed web app
- `apps/web/src/main.rs`: Removed asset!() config.js (bypassed by asset hashing)
- `apps/web/entrypoint.sh`: Write config.js to fixed path; inject script into index.html
- `apps/web/nginx.conf`: Serve config.js with no-cache headers
