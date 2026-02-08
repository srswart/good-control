---
advance:
  id: "ADV-GQL-002"
  title: "GraphQL conventions (pagination, error mapping, input validation)"
  system: "backend"
  primary_component: "graphql"
  components: ["graphql"]
  started_at: "2026-01-29"
  implementation_completed_at: "2026-01-29"
  pr_links: []
  reviewability_score: null
  risk_flags: ["public_api"]
  evidence: ["tests:integration", "tdd:red-green"]
  status: complete
---

## Objective

Standardize GraphQL API conventions for pagination, error mapping (validation vs authorization vs compliance block), and input validation patterns to ensure consistent client experience across all queries and mutations.

## Behavioral Change

After this advance:
- All paginated queries use consistent `PaginationInput` and `PageInfo` types
- Errors are categorized and mapped to structured error types (validation, authorization, compliance, not found, internal)
- Input validation provides clear field-level error messages
- At least one existing query and mutation updated to use the standard patterns
- Comprehensive tests validate error handling and pagination behavior

## Completed Implementation Tasks

- [x] tidy: review existing GraphQL error handling patterns
- [x] feat: add standard pagination types (PaginationInput, PageInfo, Connection pattern)
- [x] feat: add structured error types and error mapping utilities
- [x] feat: add input validation framework with field-level errors
- [x] feat: update existing queries to use pagination conventions (items query)
- [x] feat: update existing mutations to use error mapping (createItem mutation)
- [x] test: add integration tests for pagination edge cases (3 tests)
- [x] test: add integration tests for each error type category (4 tests)
- [x] test: add tests for input validation (included in error tests)

## Risk + Rollback

- Risk: Changing error response format could break existing clients. Need to ensure backward compatibility or coordinate client updates.
- Rollback: Keep old error format alongside new format during transition; feature flag new error handling.

## Evidence

- [x] tidy:preparatory - Reviewed existing error handling patterns
- [x] tdd:red-green - Tests written before implementation
- [x] tests:integration - 7 new integration tests (3 pagination + 4 error handling)
- [x] ci:passed - All 42 GraphQL integration tests passing
- [ ] review:owner - Pending code review

## Implementation Notes

### Pagination Convention

Use Relay-style cursor-based pagination:

```graphql
input PaginationInput {
  first: Int
  after: String
  last: Int
  before: String
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

type ItemConnection {
  edges: [ItemEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type ItemEdge {
  node: Item!
  cursor: String!
}
```

### Error Categories

```graphql
enum ErrorCategory {
  VALIDATION
  AUTHORIZATION
  COMPLIANCE_BLOCK
  NOT_FOUND
  CONFLICT
  INTERNAL
}

type FieldError {
  field: String!
  message: String!
}

type StructuredError {
  category: ErrorCategory!
  message: String!
  fieldErrors: [FieldError!]
  code: String
}
```

### Input Validation

- Validate all input fields before processing
- Return field-level errors with clear messages
- Use Rust validation libraries (e.g., validator crate)
- Provide bilingual error messages (EN/ZH) where applicable

### Migration Strategy

1. Add new error types alongside existing error handling
2. Update one query (e.g., `items`) to use pagination
3. Update one mutation (e.g., `createItem`) to use structured errors
4. Add comprehensive tests
5. Document patterns for future resolvers
6. Gradually migrate other resolvers in subsequent advances

