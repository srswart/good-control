---
advance:
  id: "ADV-GQL-005"
  title: "GraphQL API wiring for warehouse domain (receiving, QC, quarantine workflows)"
  system: "backend"
  primary_component: "graphql"
  components: ["graphql", "persistence", "warehouse"]
  started_at: null
  implementation_completed_at: null
  pr_links: []
  reviewability_score: null
  risk_flags: ["public_api"]
  evidence: []
  status: planned
---

## Objective

Wire up GraphQL queries and mutations for the warehouse domain, exposing receiving tasks, QC workflows, and quarantine operations to the API surface.

## Behavioral Change

After this advance:
- GraphQL API exposes warehouse domain functionality:
  - **Queries**: `receivingTask`, `receivingTasks`, `receivingTasksByStatus`
  - **Mutations**: `createReceivingTask`, `recordQCOutcome`, `quarantineLine`, `releaseFromQuarantine`, `completeReceivingTask`
- Frontend can manage receiving workflows end-to-end
- QC outcomes and quarantine status are tracked and auditable
- All operations integrate with inventory ledger via WarehouseService

## Planned Implementation Tasks

- [ ] tidy: add GraphQL types for ReceivingTask, ReceivingLine, QCOutcome, DiscrepancyType, QuarantineStatus
- [ ] feat: add Query resolvers for receiving tasks (by ID, by status, list all)
- [ ] feat: add Mutation resolvers for receiving workflow (create, QC, quarantine, release, complete)
- [ ] test: GraphQL integration tests for receiving workflows and state transitions
- [ ] test: permission checks for warehouse operations (warehouse.receiving permission)

## Risk + Rollback

- Risk: Receiving workflows involve multi-step state transitions and inventory updates. Improper error handling could leave tasks in inconsistent states. Requires careful transaction boundaries.
- Rollback: Remove GraphQL types and resolvers; no schema or persistence changes in this advance.

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:integration (receiving workflows, QC, quarantine, inventory integration, permissions)
- [ ] ci:passed
- [ ] review:owner

## Implementation Notes

### GraphQL Types to Add

```graphql
type ReceivingTask {
  id: ID!
  reference: String!
  status: ReceivingTaskStatus!
  expectedDeliveryDate: String
  actualDeliveryDate: String
  lines: [ReceivingLine!]!
  createdAt: String!
  updatedAt: String!
}

enum ReceivingTaskStatus {
  PENDING
  IN_PROGRESS
  COMPLETED
  CANCELLED
}

type ReceivingLine {
  id: ID!
  taskId: ID!
  sku: String!
  expectedQuantity: Int!
  receivedQuantity: Int
  qcOutcome: QCOutcome
  quarantineStatus: QuarantineStatus!
  discrepancies: [ReceivingDiscrepancy!]!
}

type ReceivingDiscrepancy {
  id: ID!
  lineId: ID!
  discrepancyType: DiscrepancyType!
  description: String!
  quantity: Int
  recordedAt: String!
}

enum DiscrepancyType {
  QUANTITY_MISMATCH
  DAMAGE
  WRONG_ITEM
  QUALITY_ISSUE
  OTHER
}

enum QCOutcome {
  PASS
  FAIL
  PENDING
}

enum QuarantineStatus {
  NOT_QUARANTINED
  QUARANTINED
  RELEASED
}

input CreateReceivingTaskInput {
  reference: String!
  expectedDeliveryDate: String
  lines: [CreateReceivingLineInput!]!
}

input CreateReceivingLineInput {
  sku: String!
  expectedQuantity: Int!
}

input RecordQCOutcomeInput {
  taskId: ID!
  lineId: ID!
  outcome: QCOutcome!
  notes: String
}

input QuarantineLineInput {
  taskId: ID!
  lineId: ID!
  reason: String!
}

input ReleaseFromQuarantineInput {
  taskId: ID!
  lineId: ID!
  locationId: String!
}
```

### Query Resolvers

- `receivingTask(id: ID!): ReceivingTask`
- `receivingTasks(limit: Int, offset: Int): [ReceivingTask!]!`
- `receivingTasksByStatus(status: ReceivingTaskStatus!): [ReceivingTask!]!`

### Mutation Resolvers

- `createReceivingTask(input: CreateReceivingTaskInput!): ReceivingTask!` (requires `warehouse.receiving` permission)
- `recordQCOutcome(input: RecordQCOutcomeInput!): ReceivingLine!` (requires `warehouse.receiving` permission)
- `quarantineLine(input: QuarantineLineInput!): ReceivingLine!` (requires `warehouse.receiving` permission)
- `releaseFromQuarantine(input: ReleaseFromQuarantineInput!): ReceivingLine!` (requires `warehouse.receiving` permission)
- `completeReceivingTask(taskId: ID!, locationId: String!): ReceivingTask!` (requires `warehouse.receiving` permission)

### Integration with WarehouseService

All mutations should use `WarehouseService` from the persistence layer to ensure:
- Atomic operations with inventory ledger updates
- Proper transaction boundaries
- Audit trail via inventory movements
- Consistent state transitions

