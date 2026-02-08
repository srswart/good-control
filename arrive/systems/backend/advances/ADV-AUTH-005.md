---
advance:
  id: "ADV-AUTH-005"
  title: "Admin mock data reload"
  system: "backend"
  primary_component: "persistence"
  components: ["persistence", "graphql", "auth"]
  started_at: "2026-02-08T13:30:00Z"
  implementation_completed_at: "2026-02-08T14:55:00Z"
  pr_links: []
  reviewability_score: 34
  risk_flags: ["data", "admin_api"]
  evidence: []
  status: complete
---

## Objective

Provide an admin-only mock data reload capability to seed demo data on Fly.io.

## Behavioral Change

After this advance:
- Admins can trigger a truncate-and-reseed of mock data via GraphQL.
- Mock data includes inventory records and in-memory warehouse tasks for demos.

## Planned Implementation Tasks

- [x] tidy: add mock data module in persistence
- [x] test: add GraphQL tests for reload permissions
- [x] feat: add reload mutation with summary counts

## Component Impact

```yaml
components: [persistence, graphql, auth]
```

- **persistence**: mock data seeding and truncate logic
- **graphql**: admin-only mutation to trigger reload
- **auth**: permission check for admin access

## Risk + Rollback

- Risk: Destructive reload wipes real data
- Mitigation: Admin-only mutation with explicit warning
- Rollback: Remove the mutation and mock data module

## Evidence

- [ ] test:integration - Admin mock data reload tests passing
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-08 - feat: admin mock data reload
- `crates/backend/persistence/src/mock_data.rs`: Added truncate + seed mock data routine
- `crates/backend/graphql/src/mutation.rs`: Added `reloadMockData` mutation with task seeding
- `crates/backend/graphql/src/types.rs`: Added mock data summary type
- `crates/backend/graphql/tests/integration_test.rs`: Added reload permission tests
### 2026-02-08 - feat: expand mock data volume
- `crates/backend/persistence/src/mock_data.rs`: Expanded seed data to ~20 rows per type
- `crates/backend/graphql/src/mutation.rs`: Expanded task seeding to 20 tasks per type
