---
advance:
  id: "ADV-FE-002"
  title: "GraphQL client layer + error mapping"
  system: "frontend"
  primary_component: "foundation"
  components: ["foundation", "inventory-ui"]
  started_at: "2026-01-29"
  implementation_completed_at: "2026-01-29"
  pr_links: []
  reviewability_score: null
  risk_flags: ["public_api", "network"]
  evidence: ["tests:unit", "tdd:red-green"]
  status: complete
---

## Objective

Add a typed GraphQL client layer to the frontend foundation with standardized error handling (auth, validation, compliance blocks) and request correlation IDs. Demonstrate the client with a small query in the inventory UI.

## Behavioral Change

After this advance:
- Frontend has a reusable GraphQL client module that handles HTTP requests to the backend
- Errors from the backend are properly mapped to frontend error types with categories
- All GraphQL requests include correlation IDs for tracing
- Inventory UI can query items from the backend using the typed client

## Completed Implementation Tasks

- [x] tidy: review existing frontend error types and HTTP client patterns
- [x] feat: add GraphQL client dependencies (reqwest, serde, uuid)
- [x] feat: create GraphQL client module with request/response handling
- [x] feat: add error mapping types matching backend StructuredError
- [x] feat: add correlation ID generation and header injection
- [x] feat: create demo items query in inventory UI
- [x] test: add tests for error category mapping (11 tests)
- [x] test: add tests for field-level error parsing
- [x] test: add client creation test

## Risk + Rollback

- Risk: Network errors, authentication failures, or malformed responses could crash the UI
- Mitigation: Comprehensive error handling with fallback UI states
- Rollback: Feature-flag GraphQL client usage, fall back to mock data if needed

## Evidence

- [x] tidy:preparatory - Reviewed existing error types and patterns
- [x] tdd:red-green - Tests written before implementation
- [x] tests:unit - 12 unit tests passing (11 error mapping + 1 client creation)
- [x] ci:passed - All builds successful (foundation, inventory-ui, app-shell, desktop)
- [ ] review:owner - Pending code review

## Changes Made

### 2026-01-29 - feat: Add GraphQL client dependencies
- crates/ui/foundation/Cargo.toml: Added reqwest 0.12 with json features, uuid 1.11 with v4 and serde features

### 2026-01-29 - feat: Create error mapping types
- crates/ui/foundation/src/graphql/error.rs: Created ErrorCategory enum matching backend
- crates/ui/foundation/src/graphql/error.rs: Created FieldError and StructuredError types
- crates/ui/foundation/src/graphql/error.rs: Created GraphQLError enum with helper methods
- crates/ui/foundation/src/graphql/error.rs: Added 11 comprehensive unit tests

### 2026-01-29 - feat: Create GraphQL client module
- crates/ui/foundation/src/graphql/client.rs: Implemented GraphQLClient with query/mutate methods
- crates/ui/foundation/src/graphql/client.rs: Added correlation ID generation using UUID v4
- crates/ui/foundation/src/graphql/client.rs: Added X-Correlation-ID header injection
- crates/ui/foundation/src/graphql/client.rs: Implemented structured error parsing from responses
- crates/ui/foundation/src/graphql/mod.rs: Created module exports

### 2026-01-29 - feat: Wire GraphQL client into foundation
- crates/ui/foundation/src/lib.rs: Exported GraphQL client types and error types

### 2026-01-29 - feat: Create demo query in inventory UI
- crates/ui/inventory/src/graphql_demo.rs: Created GraphQLDemo component
- crates/ui/inventory/src/graphql_demo.rs: Implemented items query with pagination support
- crates/ui/inventory/src/graphql_demo.rs: Added comprehensive error handling UI
- crates/ui/inventory/src/lib.rs: Exported GraphQLDemo component

### 2026-01-29 - feat: Wire demo into app shell
- crates/ui/app_shell/src/routes.rs: Added GraphQLDemo import
- crates/ui/app_shell/src/routes.rs: Added "GraphQL Demo" tab to inventory route
- crates/ui/app_shell/src/routes.rs: Wired GraphQLDemo component into tab content

