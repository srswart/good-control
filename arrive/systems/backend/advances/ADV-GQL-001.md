---
advance:
  id: "ADV-GQL-001"
  title: "GraphQL schema + resolvers (API surface): initial implementation"
  system: "backend"
  primary_component: "graphql"
  components: ["graphql"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T18:30:00Z"
  pr_links: []
  reviewability_score: 22
  risk_flags: ["new_dependency", "public_api"]
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: complete
---

## Objective

Create the first working slice of the graphql component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed graphql component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Completed Implementation Tasks

- [x] tidy: preparatory refactoring (crate boundaries, module layout)
- [x] test: add GraphQL integration tests (health, auth flows, permissions)
- [x] feat: implement schema, Query/Mutation resolvers, Axum routing

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit (resolver helpers)
- [x] tests:integration (health, login, currentUser, permissions)
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### 2026-01-25 - feat: GraphQL schema + resolvers
- crates/backend/graphql/src/{lib.rs,query.rs,mutation.rs,context.rs,schema.rs}: Implemented Query (health, currentUser, users/user) and Mutation (login/logout/user mgmt), context wiring with auth and pool.
- crates/backend/server/src/graphql_router.rs: Added GraphQL routes and playground.
- crates/backend/graphql/tests/integration_test.rs: Added integration tests for health, login, auth-required queries, roles/permissions.
