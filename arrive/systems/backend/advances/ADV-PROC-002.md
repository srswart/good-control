---
advance:
  id: "ADV-PROC-002"
  title: "Purchase orders + expected receipts"
  system: "backend"
  primary_component: "procurement"
  components: ["procurement", "warehouse", "inventory"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement purchase orders and expected receipts, and integrate receiving to close PO lines. Add tests for partial receipts and discrepancies.

## Behavioral Change

After this advance:
- Purchase orders can be created with multiple line items
- Expected receipts are generated from purchase orders
- Receiving tasks are linked to purchase orders
- PO lines are automatically closed when fully received
- Partial receipts and discrepancies are tracked
- Comprehensive tests validate PO lifecycle and receipt scenarios

## Planned Implementation Tasks

- [ ] tidy: review existing procurement and receiving integration points
- [ ] test: define purchase order entity tests
- [ ] feat: implement purchase order entity with line items
- [ ] test: define expected receipt generation tests
- [ ] feat: implement expected receipt generation from PO
- [ ] test: define receiving → PO line closure tests
- [ ] feat: implement PO line closure on receiving completion
- [ ] test: define partial receipt tests
- [ ] feat: implement partial receipt handling with PO line updates
- [ ] test: define discrepancy handling tests
- [ ] feat: implement discrepancy capture and PO line adjustment
- [ ] test: add integration tests for PO → receipt → inventory flow
- [ ] test: add integration tests for partial receipt scenarios

## Risk + Rollback

**Risks:**
- **PO accuracy**: Incorrect PO line closure could cause procurement issues
- **Discrepancy handling**: Complex discrepancy logic may introduce bugs
- **Partial receipts**: Incorrect partial receipt tracking could confuse procurement
- **Integration failures**: Receiving service failures could prevent PO updates

**Rollback Plan:**
- Feature flag PO and expected receipt functionality
- Database migrations are reversible
- Manual PO corrections available via admin tools
- PO line status can be manually adjusted if automation fails

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Purchase Order Entity
- PO header (supplier, order date, expected delivery date, terms)
- PO lines (item, quantity, unit price, delivery date)
- PO status (draft, submitted, confirmed, receiving, partially_received, received, closed)
- Line status (pending, receiving, partially_received, received, closed)

### Expected Receipt
- Generated from confirmed PO
- Contains expected items, quantities, and delivery date
- Used to create receiving tasks in warehouse
- Linked to PO for tracking and closure

### Receiving Integration
- Receiving task linked to PO line
- Receiving completion updates PO line received quantity
- Full receipt: PO line status → `Received`
- Partial receipt: PO line status → `PartiallyReceived`
- Over-receipt: Discrepancy captured, requires approval

### PO Line Closure Logic
- PO line closed when received quantity >= ordered quantity
- Partial receipts allowed with configurable tolerance
- Under-receipt requires approval to close line
- Over-receipt requires approval to accept excess

### Discrepancy Handling
- **Over-receipt**: Received > ordered, capture excess quantity
- **Under-receipt**: Received < ordered, capture shortage
- **Quality issues**: QC failures captured as discrepancies
- All discrepancies require acknowledgment before PO line closure

### Integration Points
- **Warehouse**: Receiving task creation and completion
- **Inventory**: Inventory position updates on receipt
- **Supplier**: Supplier performance tracking (on-time, quality, quantity accuracy)

