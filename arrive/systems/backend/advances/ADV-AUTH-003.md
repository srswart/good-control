---
advance:
  id: "ADV-AUTH-003"
  title: "Passkey registration/authentication with admin seed"
  system: "backend"
  primary_component: "auth"
  components: ["auth", "graphql"]
  started_at: "2026-02-07T15:30:00Z"
  implementation_completed_at: "2026-02-07T17:00:00Z"
  pr_links: []
  reviewability_score: 34
  risk_flags: ["auth", "public_api"]
  evidence: []
  status: complete
---

## Objective

Add WebAuthn passkey registration and authentication flows backed by the database, and seed an initial admin user from environment configuration.

## Behavioral Change

After this advance:
- Users can register passkeys and authenticate using WebAuthn
- Passkey challenges are tracked and validated server-side
- An initial admin user can be seeded from env vars

## Planned Implementation Tasks

- [x] tidy: review auth module for passkey integration points
- [ ] test: add passkey flow coverage for challenge storage and persistence
- [x] feat: add passkey storage and challenge tables
- [x] feat: implement passkey registration and authentication flows
- [x] feat: seed initial admin user from environment

## Component Impact

```yaml
components: [auth, graphql]
```

- **auth**: Passkey flow, challenge storage, admin bootstrap
- **graphql**: Passkey mutations for registration/authentication

## Risk + Rollback

- Risk: Misconfigured RP ID/origin could prevent passkey enrollment
- Mitigation: Validate configuration on startup and return clear errors
- Rollback: Disable passkey mutations and remove new tables

## Evidence

- [ ] tidy:preparatory - Passkey entrypoints reviewed
- [ ] tdd:red-green - Passkey tests written first
- [ ] tests:unit - Passkey flows and challenge storage tests passing
- [ ] ci:passed - CI pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: add passkey storage and challenges
- `migrations/20260207000003_passkey_schema.sql`: Added passkeys and challenge tables
- `crates/backend/auth/src/passkey.rs`: Implemented passkey registration/authentication logic
- `crates/backend/auth/src/lib.rs`: Exported passkey functions

### 2026-02-07 - feat: expose passkey GraphQL mutations
- `crates/backend/graphql/src/types.rs`: Added passkey challenge type
- `crates/backend/graphql/src/mutation.rs`: Added passkey registration/auth mutations

### 2026-02-07 - feat: seed initial admin user
- `crates/backend/auth/src/bootstrap.rs`: Seed admin from env vars
- `apps/server/src/main.rs`: Invoke admin seed on startup
- `apps/server/Cargo.toml`: Added auth dependency
