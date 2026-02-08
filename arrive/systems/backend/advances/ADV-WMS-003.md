---
advance:
  id: "ADV-WMS-003"
  title: "Putaway + replenishment tasks"
  system: "backend"
  primary_component: "warehouse"
  components: ["warehouse", "inventory"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Add putaway and replenishment task generation and completion flows with scan validation. Include integration tests ensuring inventory positions update correctly.

## Behavioral Change

After this advance:
- Warehouse operators can generate putaway tasks from received inventory
- Replenishment tasks are generated when pick locations fall below thresholds
- Scan validation ensures correct item/location matching during task execution
- Task completion updates inventory positions atomically
- Comprehensive integration tests validate inventory position updates

## Planned Implementation Tasks

- [ ] tidy: review existing warehouse task patterns from ADV-WMS-002
- [ ] test: define putaway task generation tests
- [ ] feat: implement putaway task entity and generation logic
- [ ] test: define putaway scan validation tests
- [ ] feat: implement scan validation for putaway operations
- [ ] test: define putaway completion and inventory update tests
- [ ] feat: implement putaway completion with inventory position updates
- [ ] test: define replenishment task generation tests
- [ ] feat: implement replenishment task generation based on min/max thresholds
- [ ] test: define replenishment scan validation tests
- [ ] feat: implement scan validation for replenishment operations
- [ ] test: define replenishment completion and inventory update tests
- [ ] feat: implement replenishment completion with inventory position updates
- [ ] test: add integration tests for putaway → inventory position flow
- [ ] test: add integration tests for replenishment → inventory position flow

## Risk + Rollback

**Risks:**
- **Inventory accuracy**: Incorrect position updates could cause stock discrepancies
- **Scan validation**: Overly strict validation may block legitimate operations
- **Task generation logic**: Incorrect replenishment triggers could cause over/under stocking
- **Concurrency**: Multiple operators working on same locations could cause race conditions

**Rollback Plan:**
- Feature flag putaway and replenishment workflows
- Database migrations are reversible
- Manual inventory corrections available via admin tools
- Task generation can be disabled independently from task execution

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Putaway Task Flow
- Generated from completed receiving tasks
- Suggests optimal storage location based on item attributes and space availability
- Requires scan validation: item barcode + destination location barcode
- Completion moves inventory from receiving staging to storage location

### Replenishment Task Flow
- Generated when pick location quantity falls below minimum threshold
- Suggests source location with available stock
- Requires scan validation: item barcode + source location + destination location
- Completion moves inventory from bulk storage to pick location

### Scan Validation Rules
- Item barcode must match expected item on task
- Location barcode must match expected location on task
- Quantity must be within acceptable range
- Serial/lot numbers validated if applicable

### Inventory Position Updates
- Putaway: Decrease staging position, increase storage position
- Replenishment: Decrease bulk storage position, increase pick position
- All updates are transactional and atomic
- Movement ledger records all position changes

