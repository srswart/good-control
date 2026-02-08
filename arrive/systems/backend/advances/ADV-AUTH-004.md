---
advance:
  id: "ADV-AUTH-004"
  title: "Admin user management via GraphQL"
  system: "backend"
  primary_component: "auth"
  components: ["auth", "graphql"]
  started_at: "2026-02-07T18:30:00Z"
  implementation_completed_at: "2026-02-07T19:10:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: ["auth", "public_api"]
  evidence: []
  status: complete
---

## Objective

Provide admin-only GraphQL operations for user and role management to support the user admin UI.

## Behavioral Change

After this advance:
- Admins can list users and roles via GraphQL.
- Admins can assign and remove roles for users via GraphQL.

## Planned Implementation Tasks

- [x] tidy: confirm existing RBAC helpers cover role assignment
- [x] test: add GraphQL integration tests for admin user/role ops
- [x] feat: add roles query
- [x] feat: add role assignment mutations

## Component Impact

```yaml
components: [auth, graphql]
```

- **auth**: role assignment/revocation used by GraphQL
- **graphql**: admin queries/mutations for user and role management

## Risk + Rollback

- Risk: Admin operations exposed without proper permission checks
- Mitigation: Enforce `admin.users` and `admin.roles` permissions in resolvers
- Rollback: Remove admin GraphQL fields and re-deploy backend

## Evidence

- [ ] test:integration - Admin user/role GraphQL tests passing
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: admin user/role GraphQL operations
- `crates/backend/graphql/src/query.rs`: Added `roles` query gated by `admin.roles`
- `crates/backend/graphql/src/mutation.rs`: Added `assignRole` and `removeRole` mutations with audit events
- `crates/backend/graphql/tests/integration_test.rs`: Added admin user/role GraphQL tests
