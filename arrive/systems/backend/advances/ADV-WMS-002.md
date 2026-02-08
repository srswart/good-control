---
advance:
  id: "ADV-WMS-002"
  title: "Receiving workflows with discrepancy capture, QC outcomes, and quarantine/release transitions"
  system: "backend"
  primary_component: "warehouse"
  components: ["warehouse", "inventory"]
  started_at: "2026-01-26T08:00:00Z"
  implementation_completed_at: "2026-01-26T12:00:00Z"
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: ["tidy:preparatory", "tdd:red-green", "tests:unit", "tests:integration"]
  status: completed
---

## Objective

Implement receiving workflows in the warehouse domain component with support for discrepancy capture, quality control (QC) outcomes, and quarantine/release state transitions. Integrate receiving operations with inventory ledger updates to ensure accurate stock tracking.

## Behavioral Change

After this advance:
- Warehouse operators can create receiving tasks from purchase orders
- Receiving tasks track expected vs. received quantities with discrepancy capture
- QC outcomes (pass/fail/rework) can be recorded for received items
- Items can be placed in quarantine and later released or rejected
- All receiving state transitions update the inventory ledger automatically
- Each state transition has comprehensive test coverage

## Implementation Tasks

### Phase 1: Domain Model Foundation
- [x] tidy: Create `crates/domain/warehouse` crate structure
- [x] test: Define receiving task state machine tests
- [x] feat: Implement `ReceivingTask` entity with status lifecycle
- [x] test: Define discrepancy capture model tests
- [x] feat: Implement `ReceivingDiscrepancy` entity
- [x] test: Define QC outcome model tests
- [x] feat: Implement `QCOutcome` enum and `QCResult` entity
- [x] test: Define quarantine/release transition tests
- [x] feat: Implement quarantine state management

### Phase 2: State Transitions
- [x] test: Test receiving task creation from purchase order
- [x] feat: Implement receiving task creation logic
- [x] test: Test quantity discrepancy capture (over/under/short)
- [x] feat: Implement discrepancy recording and validation
- [x] test: Test QC outcome recording (pass/fail/rework)
- [x] feat: Implement QC outcome workflow
- [x] test: Test quarantine placement and release transitions
- [x] feat: Implement quarantine/release state machine

### Phase 3: Inventory Integration
- [x] test: Test inventory ledger updates on receiving completion
- [x] feat: Integrate receiving completion with inventory service
- [x] test: Test inventory updates for QC-passed items
- [x] feat: Implement inventory updates for QC outcomes
- [x] test: Test quarantine state prevents inventory updates
- [x] feat: Ensure quarantine items don't update inventory until released
- [x] test: Test release transition updates inventory ledger
- [x] feat: Implement inventory update on quarantine release

### Phase 4: Error Handling & Validation
- [x] test: Test invalid state transitions are rejected
- [x] feat: Add state transition validation
- [x] test: Test discrepancy validation (cannot exceed reasonable thresholds)
- [x] feat: Add discrepancy recording and validation
- [x] test: Test QC outcome required before completion
- [x] feat: Enforce QC outcome requirement

## Component Impact

```yaml
components: [warehouse, inventory]
```

- **warehouse**: New domain crate with receiving workflow models and state machines
- **inventory**: Integration points for ledger updates from receiving operations

## Risk + Rollback

**Risks:**
- **Data integrity**: Incorrect inventory ledger updates could cause stock discrepancies
- **State machine bugs**: Invalid transitions could corrupt receiving task state
- **Concurrency**: Multiple operators processing same receiving task could cause race conditions
- **Integration failures**: Inventory service failures during receiving could leave tasks in inconsistent state

**Rollback Plan:**
- Feature flag receiving workflows to disable if critical issues found
- Database migrations are reversible (no data loss on rollback)
- Receiving tasks can be manually corrected via admin tools
- Inventory ledger corrections can be applied via reconciliation process
- Keep purchase order status separate from receiving task status for isolation

## Evidence

- [x] tidy:preparatory - Crate structure and domain boundaries established
- [x] tdd:red-green - Tests written before implementation for all state transitions
- [x] tests:unit - 18 unit tests passing for all receiving workflow entities and state machines
- [x] tests:integration - 7 integration tests passing for receiving → inventory ledger flow
- [ ] ci:passed - All tests passing in CI
- [ ] review:owner - Code review completed by warehouse component owner

## Design Decisions

### Receiving Task States
- `Pending` → `InProgress` → `Completed` / `Cancelled`
- QC outcomes recorded separately but required before completion
- Quarantine is a separate state that can be entered from any QC outcome

### Discrepancy Handling
- Capture over-receipt, under-receipt, and short-receipt scenarios
- Discrepancies require approval/acknowledgment before task completion
- Discrepancy thresholds configurable per item category

### QC Workflow
- QC outcomes: `Pass`, `Fail`, `Rework`
- Failed items automatically enter quarantine
- Passed items proceed to inventory update
- Rework items require re-inspection

### Quarantine Management
- Quarantine state prevents inventory updates
- Release transitions: `ReleaseToInventory` or `Reject`
- Released items update inventory ledger
- Rejected items are removed from receiving (no inventory update)

### Inventory Integration
- Receiving completion triggers inventory position updates
- QC-passed items create inventory movements
- Quarantine release creates inventory movements with timestamp
- All inventory updates are transactional and reversible

## Changes Made

### 2026-01-26 - tidy: Create warehouse domain crate structure
- crates/domain/warehouse/Cargo.toml: Created warehouse domain crate manifest
- crates/domain/warehouse/src/lib.rs: Created library exports
- crates/domain/warehouse/src/error.rs: Created WarehouseError enum and Result type
- Cargo.toml: Added warehouse crate to workspace members and dependencies

### 2026-01-26 - test: Define receiving workflow tests
- crates/domain/warehouse/src/receiving.rs: Added comprehensive test suite (18 tests) covering:
  - Receiving task creation from purchase orders
  - State transitions (Pending → InProgress → Completed/Cancelled)
  - Receipt recording with discrepancy detection
  - QC outcome recording and workflow
  - Quarantine placement, release, and rejection
  - Discrepancy acknowledgment
  - Completion validation (QC required, discrepancies acknowledged)

### 2026-01-26 - feat: Implement receiving workflow domain models
- crates/domain/warehouse/src/receiving.rs: Implemented complete receiving workflow:
  - `ReceivingTask` entity with status lifecycle management
  - `ReceivingLine` entity for tracking individual line items
  - `ReceivingDiscrepancy` entity for capturing over/under/short-receipt scenarios
  - `QCOutcome` enum and `QCResult` entity for quality control
  - `QuarantineStatus` enum for quarantine state management
  - State transition validation and business rules
  - All 18 tests passing

### 2026-01-26 - feat: Add receive_inventory method to inventory service
- crates/backend/persistence/src/inventory_service.rs: Added `receive_inventory` method:
  - Creates or updates inventory positions atomically
  - Records receipt movements in the ledger
  - Handles existing positions with reserved quantities
  - 4 unit tests passing

### 2026-01-26 - feat: Create warehouse service for receiving → inventory integration
- crates/backend/persistence/src/warehouse_service.rs: Created `WarehouseService`:
  - `complete_receiving_task`: Updates inventory for completed receiving tasks
  - Skips quarantine and rejected items (no inventory update)
  - Only updates inventory for QC Pass outcomes
  - `release_from_quarantine_and_update_inventory`: Releases items from quarantine and updates inventory
  - Validates QC Pass required before release
  - 7 integration tests passing

### 2026-01-26 - test: Integration tests for receiving → inventory flow
- crates/backend/persistence/src/warehouse_service.rs: Added comprehensive integration tests:
  - Test receiving task completion updates inventory
  - Test quarantine items are skipped during completion
  - Test rejected items are skipped during completion
  - Test incomplete tasks cannot update inventory
  - Test quarantine release updates inventory
  - Test quarantine release validation (must be quarantined, must have QC Pass)
  - All 7 integration tests passing
