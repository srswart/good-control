---
advance:
  id: "ADV-ORD-003"
  title: "Fulfillment workflow"
  system: "backend"
  primary_component: "orders"
  components: ["orders", "warehouse"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Integrate pick/pack/ship completion to order fulfillment status updates. Add tests that validate order states match warehouse outcomes.

## Behavioral Change

After this advance:
- Pick task completion updates order fulfillment status
- Pack verification updates order packaging status
- Ship dispatch updates order shipment status
- Order status accurately reflects warehouse operation outcomes
- Comprehensive tests validate order state synchronization

## Planned Implementation Tasks

- [ ] tidy: review existing warehouse task completion and order status requirements
- [ ] test: define pick completion → order update tests
- [ ] feat: implement pick completion integration with order status updates
- [ ] test: define pack completion → order update tests
- [ ] feat: implement pack completion integration with order status updates
- [ ] test: define ship dispatch → order update tests
- [ ] feat: implement ship dispatch integration with order status updates
- [ ] test: define partial fulfillment tests
- [ ] feat: implement partial fulfillment status tracking
- [ ] test: define exception handling tests (short pick, damaged, cancelled)
- [ ] feat: implement exception handling with order status updates
- [ ] test: add integration tests for complete fulfillment lifecycle
- [ ] test: add integration tests for partial fulfillment scenarios

## Risk + Rollback

**Risks:**
- **Status synchronization**: Incorrect status updates could confuse operators and customers
- **Partial fulfillment**: Complex partial fulfillment logic may introduce bugs
- **Exception handling**: Incorrect exception handling could leave orders in inconsistent state
- **Integration failures**: Warehouse service failures could prevent status updates

**Rollback Plan:**
- Feature flag fulfillment workflow integration
- Database migrations are reversible
- Manual order status corrections available via admin tools
- Status update events can be replayed if synchronization fails

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Order Status Lifecycle
- `Pending` → `Allocated` → `Picking` → `Picked` → `Packing` → `Packed` → `Shipping` → `Shipped` → `Completed`
- Partial fulfillment: `PartiallyPicked`, `PartiallyPacked`, `PartiallyShipped`
- Exceptions: `PickException`, `PackException`, `ShipException`, `Cancelled`

### Pick Completion Integration
- Pick task completion triggers order status update
- Full pick: Order status → `Picked`
- Partial pick: Order status → `PartiallyPicked`, backorder created
- Pick exception: Order status → `PickException`, requires resolution

### Pack Completion Integration
- Pack verification completion triggers order status update
- Full pack: Order status → `Packed`
- Partial pack: Order status → `PartiallyPacked`
- Pack exception: Order status → `PackException`, requires resolution

### Ship Dispatch Integration
- Ship dispatch confirmation triggers order status update
- Full shipment: Order status → `Shipped`
- Partial shipment: Order status → `PartiallyShipped`, remaining items backordered
- Ship exception: Order status → `ShipException`, requires resolution

### Fulfillment Tracking
- Track quantities at each stage (allocated, picked, packed, shipped)
- Track timestamps for each status transition
- Track operator/user for each operation
- Maintain audit trail of all status changes

### Exception Handling
- **Short Pick**: Insufficient quantity found, create backorder, update status
- **Damaged**: Item damaged during pick/pack, quarantine and replace
- **Cancelled**: Order cancelled, release reservations, update status
- All exceptions logged with operator notes and resolution actions

### Integration Points
- **Warehouse**: Pick/pack/ship task completion events
- **Inventory**: Reservation release and position updates
- **Logistics**: Shipment record creation
- **Notifications**: Customer notifications on status changes (if configured)

