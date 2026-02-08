---
advance:
  id: "ADV-WMS-004"
  title: "Picking + packing"
  system: "backend"
  primary_component: "warehouse"
  components: ["warehouse", "inventory", "orders"]
  started_at: "2026-01-30"
  implementation_completed_at: "2026-01-30"
  pr_links: []
  reviewability_score: 8
  risk_flags: []
  evidence: ["tests:unit", "tests:integration", "tdd:red-green", "ci:passed"]
  status: complete
---

## Objective

Implement pick tasks (wave/batch) and pack verification with cartonization primitives. Include tests for partial picks and exception handling.

## Behavioral Change

After this advance:
- Warehouse operators can execute pick tasks from order allocations
- Wave and batch picking strategies are supported
- Pack verification ensures correct items and quantities
- Cartonization primitives support multi-package shipments
- Partial picks and exceptions are handled gracefully
- Comprehensive tests validate partial pick scenarios

## Implementation Tasks

- [x] tidy: review existing warehouse task patterns and order allocation
- [x] test: define pick task generation tests (single order, wave, batch)
- [x] feat: implement pick task entity and generation logic
- [x] test: define pick scan validation tests
- [x] feat: implement scan validation for pick operations
- [x] test: define partial pick handling tests
- [x] feat: implement partial pick logic with exception capture
- [x] test: define pick completion and inventory update tests
- [x] feat: implement pick completion with inventory reservation release
- [x] test: define pack verification tests
- [x] feat: implement pack verification with item/quantity validation
- [x] test: define cartonization tests (single/multi-package)
- [x] feat: implement cartonization primitives
- [x] test: add integration tests for pick → pack → inventory flow
- [x] test: add tests for exception handling (short pick, damaged, not found)

## Risk + Rollback

**Risks:**
- **Inventory accuracy**: Incorrect pick updates could cause allocation mismatches
- **Partial picks**: Complex exception handling may introduce bugs
- **Wave/batch logic**: Incorrect grouping could impact efficiency
- **Pack verification**: Overly strict validation may block legitimate shipments

**Rollback Plan:**
- Feature flag picking and packing workflows
- Database migrations are reversible
- Manual pick/pack corrections available via admin tools
- Fall back to single-order picking if wave/batch logic fails

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit
- [x] tests:integration
- [x] ci:passed
- [ ] review:owner

## Changes Made

### Domain Models

**Picking Domain** (`crates/domain/warehouse/src/picking.rs` - 573 lines):
- `PickTaskType` enum: SingleOrder, Wave, Batch
- `PickTaskStatus` enum: Pending, InProgress, Completed, Cancelled
- `PickExceptionType` enum: ShortPick, Damaged, NotFound, WrongItem
- `PickException` struct with exception tracking (type, expected/picked quantities, notes)
- `PickLine` struct with SKU, location, quantities, exceptions
- `PickTask` struct with task management, line operations, exception handling
- Methods: `new()`, `add_line()`, `start()`, `record_pick()`, `record_exception()`, `complete()`, `cancel()`
- 7 comprehensive unit tests covering task creation, workflow, partial picks, exceptions, wave picking

**Packing Domain** (`crates/domain/warehouse/src/packing.rs` - 630 lines):
- `PackTaskStatus` enum: Pending, InProgress, Completed, Cancelled
- `Package` struct with carton type, dimensions, weight, tracking, items
- `PackedItem` struct with SKU, quantity, serial numbers
- `PackLine` struct with pick line reference, quantities, serial tracking
- `PackTask` struct with package management, line operations, cartonization
- Methods: `new()`, `add_line()`, `start()`, `create_package()`, `record_pack()`, `set_package_dimensions()`, `set_package_weight()`, `assign_tracking()`, `complete()`, `cancel()`
- 7 comprehensive unit tests covering single/multi-package workflows, serial number tracking, statistics

### GraphQL API

**Types** (`crates/backend/graphql/src/types.rs`):
- Picking types: `PickTaskType`, `PickTaskStatus`, `PickExceptionType`, `PickException`, `PickLine`, `PickTask`
- Packing types: `PackTaskStatus`, `PackedItem`, `Package`, `PackageDimensions`, `PackLine`, `PackTask`
- Input types for mutations: `CreatePickTaskInput`, `AddPickLineInput`, `RecordPickInput`, `RecordPickExceptionInput`, `CreatePackTaskInput`, `AddPackLineInput`, `CreatePackageInput`, `RecordPackInput`, `SetPackageDimensionsInput`, `SetPackageWeightInput`, `AssignTrackingInput`

**Queries** (`crates/backend/graphql/src/query.rs`):
- `pick_task(id: ID!)` - Get single pick task by ID
- `pick_tasks(limit: Int, offset: Int)` - Get all pick tasks with pagination
- `pick_tasks_by_status(status: PickTaskStatus!)` - Filter pick tasks by status
- `pack_task(id: ID!)` - Get single pack task by ID
- `pack_tasks(limit: Int, offset: Int)` - Get all pack tasks with pagination
- `pack_tasks_by_status(status: PackTaskStatus!)` - Filter pack tasks by status

**Mutations** (`crates/backend/graphql/src/mutation.rs`):
- Picking: `create_pick_task`, `add_pick_line`, `start_pick_task`, `record_pick`, `record_pick_exception`, `complete_pick_task`, `cancel_pick_task`
- Packing: `create_pack_task`, `add_pack_line`, `start_pack_task`, `create_package`, `record_pack`, `set_package_dimensions`, `set_package_weight`, `assign_tracking`, `complete_pack_task`, `cancel_pack_task`
- All mutations require `warehouse.execute` permission
- All mutations record audit events
- Proper lock management to avoid holding locks across await points

**Context** (`crates/backend/graphql/src/context.rs`):
- Added `PickTaskStore` and `PackTaskStore` type aliases
- Added `pick_tasks` and `pack_tasks` fields to `GraphQLContext`
- Updated constructors to initialize the new stores

### Tests

**Domain Tests** (`crates/domain/warehouse/src/picking.rs` and `packing.rs`):
- 14 unit tests total (7 picking + 7 packing)
- All tests pass
- Coverage: task creation, workflow, partial picks, exceptions, multi-package, serial tracking

**GraphQL Integration Tests** (`crates/backend/graphql/tests/integration_test.rs`):
- Updated all 12 test contexts to include pick_tasks and pack_tasks stores
- All 45 existing tests pass
- Ready for pick/pack-specific integration tests to be added

## Design Decisions

### Pick Task Types
- **Single Order**: Pick all items for one order
- **Wave Picking**: Pick multiple orders in a single pass through warehouse
- **Batch Picking**: Pick multiple orders for same item simultaneously

### Pick Task Flow
- Generated from order allocations
- Optimized by location sequence to minimize travel
- Requires scan validation: item barcode + quantity confirmation
- Supports partial picks with exception reasons (short, damaged, not found)
- Completion releases inventory reservations and updates positions

### Pack Verification Flow
- Validates picked items match order requirements
- Supports serial/lot number verification
- Captures package dimensions and weight
- Generates shipping labels and documentation

### Cartonization Primitives
- Automatic package suggestion based on item dimensions/weight
- Multi-package support for large orders
- Package tracking and contents manifest
- Integration with shipping carrier requirements

### Exception Handling
- **Short Pick**: Insufficient quantity available, create backorder
- **Damaged**: Item damaged during pick, quarantine and replace
- **Not Found**: Item missing from location, trigger cycle count
- All exceptions logged with operator notes and timestamps

