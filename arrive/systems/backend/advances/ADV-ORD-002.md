---
advance:
  id: "ADV-ORD-002"
  title: "Orders + allocation"
  system: "backend"
  primary_component: "orders"
  components: ["orders", "inventory"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement order capture and allocation (reserve/allocate) with backorder behavior. Expose GraphQL and add tests for partial fulfillment.

## Behavioral Change

After this advance:
- Orders can be created with multiple line items
- Allocation logic reserves inventory for order fulfillment
- Partial fulfillment is supported with backorder creation
- GraphQL API exposes order and allocation operations
- Comprehensive tests validate allocation and backorder scenarios

## Planned Implementation Tasks

- [ ] tidy: review existing inventory reservation logic and order requirements
- [ ] test: define order entity tests
- [ ] feat: implement order entity with line items
- [ ] test: define allocation logic tests (full, partial, none)
- [ ] feat: implement allocation logic with inventory reservation
- [ ] test: define backorder creation tests
- [ ] feat: implement backorder creation for partial fulfillment
- [ ] test: define allocation priority tests (FIFO, expiry, location)
- [ ] feat: implement allocation priority logic
- [ ] test: define GraphQL mutation tests
- [ ] feat: expose GraphQL mutations for order operations
- [ ] test: add integration tests for order → allocation → inventory flow
- [ ] test: add integration tests for partial fulfillment scenarios

## Risk + Rollback

**Risks:**
- **Inventory accuracy**: Incorrect allocation could cause over-commitment
- **Allocation logic**: Complex priority rules may introduce bugs
- **Backorder handling**: Incorrect backorder creation could confuse fulfillment
- **Concurrency**: Multiple orders allocating same inventory could cause race conditions

**Rollback Plan:**
- Feature flag order allocation
- Database migrations are reversible
- Manual allocation corrections available via admin tools
- Inventory reservations can be released if allocation fails

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Order Entity
- Order header (customer, order date, required date, priority)
- Order lines (item, quantity, unit price)
- Order status (pending, allocated, partially_allocated, picking, packed, shipped, completed)
- Fulfillment tracking (allocated qty, picked qty, shipped qty)

### Allocation Logic
- **Full Allocation**: All line items can be fully allocated
- **Partial Allocation**: Some line items partially allocated, backorders created
- **No Allocation**: Insufficient inventory, entire order backordered
- Allocation respects inventory reservations and available quantities

### Allocation Priority Rules
- **FIFO**: First-in, first-out (oldest inventory first)
- **Expiry**: Nearest expiry date first (for perishable items)
- **Location**: Closest to shipping dock first (minimize travel)
- **Serial/Lot**: Specific serial/lot requirements honored

### Backorder Behavior
- Backorder created automatically for unfulfilled quantities
- Backorder linked to original order for tracking
- Backorder automatically allocated when inventory becomes available
- Customer notification on backorder creation (if configured)

### GraphQL API
```graphql
mutation createOrder(input: CreateOrderInput!): OrderResult!
mutation allocateOrder(orderId: ID!): AllocationResult!
mutation cancelOrder(orderId: ID!, reason: String): OrderResult!
query orders(filter: OrderFilter, pagination: PaginationInput): OrderConnection!
query order(id: ID!): Order
```

### Integration with Inventory
- Allocation creates inventory reservations
- Reservations prevent double-allocation
- Pick completion releases reservations and updates positions
- Ship completion removes inventory from positions

