---
advance:
  id: "ADV-GQL-004"
  title: "GraphQL API wiring for inventory domain (positions, movements, items, locations, genealogy)"
  system: "backend"
  primary_component: "graphql"
  components: ["graphql", "persistence", "inventory"]
  started_at: null
  implementation_completed_at: null
  pr_links: []
  reviewability_score: null
  risk_flags: ["public_api"]
  evidence: []
  status: planned
---

## Objective

Wire up GraphQL queries and mutations for the inventory domain, exposing inventory positions, movements, items, locations, and serial/lot genealogy queries to the API surface.

## Behavioral Change

After this advance:
- GraphQL API exposes inventory domain functionality:
  - **Queries**: `inventoryPosition`, `inventoryPositions`, `inventoryMovements`, `items`, `item`, `locations`, `location`, `serialGenealogy`, `lotGenealogy`
  - **Mutations**: `createItem`, `createLocation`, `receiveInventory`, `transferInventory`, `adjustInventory`
- Frontend and external clients can query current inventory positions and movement history
- Serial and lot traceability queries are available for compliance and auditing
- All operations are backed by existing persistence layer services and repositories

## Planned Implementation Tasks

- [ ] tidy: add GraphQL types for Item, Location, InventoryPosition, InventoryMovement, SerialGenealogy, LotGenealogy
- [ ] feat: add Query resolvers for inventory positions, movements, items, locations, and genealogy
- [ ] feat: add Mutation resolvers for item/location creation and inventory operations (receive, transfer, adjust)
- [ ] test: GraphQL integration tests for inventory queries and mutations
- [ ] test: permission checks for inventory operations (warehouse.inventory permission)

## Risk + Rollback

- Risk: Exposing inventory data via public API requires proper permission checks to prevent unauthorized access. Genealogy queries may be slow on large datasets without proper indexing.
- Rollback: Remove GraphQL types and resolvers; no schema or persistence changes in this advance.

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:integration (inventory queries, mutations, genealogy, permissions)
- [ ] ci:passed
- [ ] review:owner

## Implementation Notes

### GraphQL Types to Add

```graphql
type Item {
  id: ID!
  sku: String!
  name: String!
  nameZh: String
  description: String
  descriptionZh: String
  category: String
  uom: String!
  abcClass: String
  createdAt: String!
  updatedAt: String!
}

type Location {
  id: ID!
  locationId: String!
  name: String!
  nameZh: String
  locationType: String!
  parentLocationId: String
  createdAt: String!
  updatedAt: String!
}

type InventoryPosition {
  id: ID!
  sku: String!
  locationId: String!
  status: String!
  quantityOnHand: Int!
  quantityAvailable: Int!
  quantityReserved: Int!
  lotId: String
  updatedAt: String!
}

type InventoryMovement {
  id: ID!
  timestamp: String!
  actor: String!
  reason: String!
  fromLocationId: String
  toLocationId: String
  sku: String!
  quantity: Int!
  lotId: String
  serialNumbers: [String!]
  statusBefore: String!
  statusAfter: String!
  reference: String
}

type SerialGenealogyEntry {
  movementId: ID!
  timestamp: String!
  reason: String!
  fromLocation: String
  toLocation: String
  sku: String!
  statusBefore: String!
  statusAfter: String!
}

type LotGenealogyEntry {
  movementId: ID!
  timestamp: String!
  reason: String!
  fromLocation: String
  toLocation: String
  sku: String!
  quantity: Int!
  statusBefore: String!
  statusAfter: String!
}
```

### Query Resolvers

- `inventoryPosition(sku: String!, locationId: String!, status: String!): InventoryPosition`
- `inventoryPositions(sku: String, locationId: String, status: String): [InventoryPosition!]!`
- `inventoryMovements(sku: String, locationId: String, limit: Int): [InventoryMovement!]!`
- `items(category: String, abcClass: String): [Item!]!`
- `item(sku: String!): Item`
- `locations(locationType: String): [Location!]!`
- `location(locationId: String!): Location`
- `serialGenealogy(serialNumber: String!): [SerialGenealogyEntry!]!`
- `lotGenealogy(lotNumber: String!): [LotGenealogyEntry!]!`

### Mutation Resolvers

- `createItem(input: CreateItemInput!): Item!` (requires `warehouse.inventory` permission)
- `createLocation(input: CreateLocationInput!): Location!` (requires `warehouse.inventory` permission)
- `receiveInventory(sku: String!, locationId: String!, quantity: Int!, reference: String): InventoryMovement!` (requires `warehouse.receiving` permission)
- `transferInventory(sku: String!, fromLocationId: String!, toLocationId: String!, quantity: Int!, reference: String): InventoryMovement!` (requires `warehouse.inventory` permission)
- `adjustInventory(sku: String!, locationId: String!, quantity: Int!, reason: String!, reference: String): InventoryMovement!` (requires `warehouse.inventory` permission)

