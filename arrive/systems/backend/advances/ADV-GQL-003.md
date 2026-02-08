---
advance:
  id: "ADV-GQL-003"
  title: "Bulk mutation pattern"
  system: "backend"
  primary_component: "graphql"
  components: ["graphql"]
  started_at: "2026-01-29"
  implementation_completed_at: "2026-01-29"
  pr_links: []
  reviewability_score: 0
  risk_flags: ["public_api"]
  evidence: ["tests:integration", "tdd:red-green", "ci:passed"]
  status: complete
---

## Objective

Introduce a bulk mutation pattern that returns per-row success/error results. Apply it to one inventory or warehouse operation. Include tests validating partial failure handling.

## Behavioral Change

After this advance:
- GraphQL API supports bulk mutations with per-row success/error results
- Clients can submit multiple operations in a single request
- Partial failures are handled gracefully with detailed error information per row
- At least one inventory or warehouse operation uses the bulk mutation pattern
- Comprehensive tests validate partial failure scenarios

## Completed Implementation Tasks

- [x] tidy: review existing mutation patterns and identify bulk operation candidates
- [x] feat: implement bulk mutation response types (BulkMutationResult, RowResult, AdjustInventoryInput)
- [x] feat: implement bulk inventory adjustment mutation (bulkAdjustInventory)
- [x] test: add integration tests for bulk operations (all success, all fail, partial failure)
- [x] feat: add per-row validation and error handling
- [x] feat: add audit logging for bulk operations

## Risk + Rollback

**Risks:**
- **Transaction boundaries**: Bulk operations may need careful transaction handling to ensure atomicity per row
- **Performance**: Large bulk operations could impact API performance
- **Error reporting**: Complex error structures may be difficult for clients to parse
- **Rollback complexity**: Partial failures may require compensation logic

**Rollback Plan:**
- Feature flag bulk mutation endpoints
- Keep individual mutation endpoints available as fallback
- Document transaction semantics clearly
- Add rate limiting for bulk operations if needed

## Evidence

- [x] tidy:preparatory - Reviewed existing mutation patterns (adjust_inventory identified as candidate)
- [x] tdd:red-green - Tests written before implementation
- [x] tests:integration - 3 new integration tests (all success, all fail, partial failure)
- [x] ci:passed - All 45 GraphQL integration tests passing
- [ ] review:owner - Pending code review

## Design Decisions

### Bulk Mutation Response Pattern

```graphql
type BulkMutationResult {
  successCount: Int!
  failureCount: Int!
  results: [RowResult!]!
}

type RowResult {
  index: Int!
  success: Boolean!
  data: MutationData
  error: StructuredError
}
```

### Transaction Semantics
- Each row operation is independent (no cross-row transactions)
- Failures in one row do not affect other rows
- All operations are attempted regardless of individual failures
- Results maintain input order for easy correlation

### Example Application
- Bulk inventory adjustment mutation for cycle count corrections
- Supports adjusting multiple items/locations in a single request
- Returns success/failure per adjustment with detailed error messages

## Changes Made

### 2026-01-29 - feat: Bulk mutation types and implementation

**Files Modified:**

1. **crates/backend/graphql/src/types.rs** (lines 847-887)
   - Added `BulkMutationResult<T>` - Generic result type with success/failure counts and per-row results
   - Added `RowResult<T>` - Per-row result with index, success flag, optional data, and optional error
   - Added `AdjustInventoryInput` - Input type for single inventory adjustment in bulk operation

2. **crates/backend/graphql/src/mutation.rs** (lines 7-14, 730-881)
   - Updated imports to include `AdjustInventoryInput`, `BulkMutationResult`, `RowResult`
   - Added `bulk_adjust_inventory` mutation resolver:
     - Accepts `Vec<AdjustInventoryInput>` as input
     - Checks `inventory.adjust` permission once upfront
     - Validates each input (SKU, location_id, reason not empty; quantity not zero)
     - Processes each adjustment independently in a loop
     - Builds `RowResult` for each with success/error details
     - Records audit event for the bulk operation
     - Returns `BulkMutationResult<InventoryMovement>`

3. **crates/backend/graphql/tests/integration_test.rs** (lines 3004-3393)
   - Added `test_bulk_adjust_inventory_all_success` - Tests 3 successful adjustments
   - Added `test_bulk_adjust_inventory_all_fail` - Tests 3 validation failures
   - Added `test_bulk_adjust_inventory_partial_failure` - Tests mix of 3 successes and 2 failures
   - All tests verify:
     - Correct success/failure counts
     - Per-row results maintain input order
     - Successful rows update inventory positions correctly
     - Failed rows don't affect database state
     - Error categories and messages are correct

**Key Implementation Details:**

- **Transaction semantics**: Each row operation is independent with its own transaction (via `InventoryService::adjust_inventory`)
- **Error handling**: Validation errors (empty fields, zero quantity) and service errors (negative inventory, non-existent inventory) are categorized appropriately
- **Partial failure support**: All operations are attempted regardless of individual failures
- **Result ordering**: Results array maintains same order as input array for easy correlation
- **Permission check**: Single permission check upfront rather than per-row
- **Audit logging**: Single audit event records overall bulk operation statistics

**Test Results:**
- All 45 GraphQL integration tests passing (42 existing + 3 new)
- Tests cover all success, all fail, and partial failure scenarios
- Tests verify inventory positions are updated correctly
- Tests verify error categorization and messages

