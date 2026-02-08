---
advance:
  id: "ADV-AUTH-002"
  title: "Wire GraphQL authentication context"
  system: "backend"
  primary_component: "auth"
  components: ["auth", "graphql"]
  started_at: "2026-02-07T15:00:00Z"
  implementation_completed_at: "2026-02-07T17:00:00Z"
  pr_links: []
  reviewability_score: 26
  risk_flags: ["auth", "public_api"]
  evidence: []
  status: complete
---

## Objective

Ensure GraphQL requests participate in authentication by extracting Bearer sessions and exposing the current user in the GraphQL context so permission checks are enforced.

## Behavioral Change

After this advance:
- GraphQL requests can include Bearer tokens to authenticate
- `require_user` and `require_permission` are enforced from GraphQL context
- Unauthenticated requests continue for public operations but fail when auth is required

## Planned Implementation Tasks

- [x] tidy: confirm GraphQL handler wiring and auth middleware entrypoints
- [x] test: validate optional auth middleware behavior for GraphQL routes
- [x] feat: add optional auth middleware and attach CurrentUser to GraphQL context

## Component Impact

```yaml
components: [auth, graphql]
```

- **auth**: Optional auth middleware for GraphQL requests
- **graphql**: Context now includes authenticated user when present

## Risk + Rollback

- Risk: Authentication failures could block previously accessible operations
- Mitigation: Use optional auth middleware; only enforce auth where required
- Rollback: Remove auth middleware wiring from GraphQL router

## Evidence

- [ ] tidy:preparatory - Middleware wiring reviewed
- [ ] tdd:red-green - Middleware behavior tests added
- [ ] tests:unit - Middleware tests passing
- [ ] ci:passed - CI pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: wire optional auth into GraphQL requests
- `crates/backend/auth/src/middleware.rs`: Added optional auth middleware for Bearer tokens
- `crates/backend/auth/src/lib.rs`: Exported optional auth middleware
- `crates/backend/server/src/graphql_router.rs`: Attached auth layer and CurrentUser to GraphQL context
