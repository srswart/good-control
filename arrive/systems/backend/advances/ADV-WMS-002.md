---
advance:
  id: "ADV-WMS-002"
  title: "Receiving workflows with discrepancy capture, QC outcomes, and quarantine/release transitions"
  system: "backend"
  primary_component: "warehouse"
  components: ["warehouse", "inventory"]
  started_at: "2026-01-26T08:00:00Z"
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
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
- [ ] tidy: Create `crates/domain/warehouse` crate structure
- [ ] test: Define receiving task state machine tests
- [ ] feat: Implement `ReceivingTask` entity with status lifecycle
- [ ] test: Define discrepancy capture model tests
- [ ] feat: Implement `ReceivingDiscrepancy` entity
- [ ] test: Define QC outcome model tests
- [ ] feat: Implement `QCOutcome` enum and `QCResult` entity
- [ ] test: Define quarantine/release transition tests
- [ ] feat: Implement quarantine state management

### Phase 2: State Transitions
- [ ] test: Test receiving task creation from purchase order
- [ ] feat: Implement receiving task creation logic
- [ ] test: Test quantity discrepancy capture (over/under/short)
- [ ] feat: Implement discrepancy recording and validation
- [ ] test: Test QC outcome recording (pass/fail/rework)
- [ ] feat: Implement QC outcome workflow
- [ ] test: Test quarantine placement and release transitions
- [ ] feat: Implement quarantine/release state machine

### Phase 3: Inventory Integration
- [ ] test: Test inventory ledger updates on receiving completion
- [ ] feat: Integrate receiving completion with inventory service
- [ ] test: Test inventory updates for QC-passed items
- [ ] feat: Implement inventory updates for QC outcomes
- [ ] test: Test quarantine state prevents inventory updates
- [ ] feat: Ensure quarantine items don't update inventory until released
- [ ] test: Test release transition updates inventory ledger
- [ ] feat: Implement inventory update on quarantine release

### Phase 4: Error Handling & Validation
- [ ] test: Test invalid state transitions are rejected
- [ ] feat: Add state transition validation
- [ ] test: Test discrepancy validation (cannot exceed reasonable thresholds)
- [ ] feat: Add discrepancy validation rules
- [ ] test: Test QC outcome required before completion
- [ ] feat: Enforce QC outcome requirement

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

- [ ] tidy:preparatory - Crate structure and domain boundaries established
- [ ] tdd:red-green - Tests written before implementation for all state transitions
- [ ] tests:unit - Unit tests for all receiving workflow entities and state machines
- [ ] tests:integration - Integration tests for receiving → inventory ledger flow
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
