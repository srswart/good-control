# Solution Intent — Backend
**Date:** 2026-01-25  
**System:** backend  
**Stack:** Rust (Axum), GraphQL, SQLx, SQLite  
**Languages:** English + Chinese (no Arabic)

## 1) Outcomes
- Provide a **single source of truth** for inventory, warehouse execution, logistics, compliance approvals, and asset/battery lifecycle.
- Enforce **traceability** (serial/lot) and **auditability** (immutable event trail for regulated workflows).
- Support **jurisdiction-based compliance gating** that can block operations (shipments, transfers, releases).
- Provide a stable, evolvable **GraphQL API** for desktop/mobile UI and CLI tooling.
- Enable optional **Kingdee Cloud financial** integration (master data + transactions + reconciliation).

## 2) In Scope
- Domain capabilities:
  - Master data (items/SKUs, suppliers/customers, locations)
  - Inventory ledger (available/reserved/in-transit/quarantine/repair/scrap)
  - Serial + lot traceability and genealogy
  - WMS task flows (receive/inspect/putaway/replenish/pick/pack/ship)
  - Drone asset registry + configuration control
  - Battery lifecycle states, health/cycle tracking, grounding/retirement
  - Compliance rules engine (per jurisdiction) + approvals + document packs + gating
  - Logistics (shipments, tracking events, 3PL abstractions)
  - Procurement, orders/fulfillment, MRO/RMA
  - Reporting/query endpoints (dashboards + exports)
- Cross-cutting:
  - AuthN/AuthZ (RBAC), audit hooks, structured logging/tracing
  - i18n primitives (EN+ZH) for display strings and document generation
  - Integrations: Kingdee connector + sync ledger + reconciliation

## 3) Out of Scope (initially)
- Full-blown MRP/forecasting optimization (can be added as later domain crates).
- Deep carrier-rate shopping and dynamic routing.
- Full offline warehouse execution (limited caching ok; authoritative writes go to backend).

## 4) Architecture Approach
### 4.1 Layering
- **Transport layer**: Axum HTTP server + GraphQL endpoint(s)
- **Application layer**: orchestrates use-cases (commands/queries), validates invariants, enforces gates
- **Domain layer** (crate-per-capability): pure rules/state transitions, no SQL/HTTP
- **Persistence layer**: SQLx repositories + migrations, transactional boundaries
- **Integration layer**: Kingdee client, mapping tables, sync jobs, reconciliation

### 4.2 Crate-based modularity (Cargo workspace)
**Backend system crates (wiring):**
- `crates/backend/foundation` — config, tracing, error taxonomy, server startup
- `crates/backend/auth` — sessions/tokens, RBAC, permission checks, audit hooks
- `crates/backend/graphql` — schema/resolvers; maps to application commands/queries
- `crates/backend/persistence` — SQLx pool, repository implementations, migrations helpers

**Domain crates (pure business logic):**
- `crates/domain/inventory`
- `crates/domain/warehouse`
- `crates/domain/logistics`
- `crates/domain/compliance`
- `crates/domain/assets`
- `crates/domain/battery`
- `crates/domain/procurement`
- `crates/domain/orders`
- `crates/domain/mro`
- `crates/domain/reporting`

**Integration crates:**
- `crates/integration/kingdee`

**Shared crates:**
- `crates/shared/i18n` — locale tags, translation selection, bilingual doc primitives
- (optional later) `crates/shared/ids` — typed IDs, ULID/UUID policies

## 5) Data & Persistence Intent (SQLite + SQLx)
- SQLite as the primary DB for the initial phase, with:
  - strict foreign keys, WAL mode, transactional writes
  - migration discipline (up/down where feasible)
- Tables organized by bounded context with **typed IDs** and **immutable events** where auditability matters:
  - inventory ledger entries (append-only), plus current balances (derived/materialized)
  - status/state tables for assets and batteries with event history
  - compliance approvals and document pack tables with versioning and expiry tracking
- Concurrency: optimistic patterns where possible; use transactions to protect multi-step operations (allocate + reserve + ship).

## 6) API Intent (GraphQL)
- GraphQL as the primary contract:
  - Queries for dashboards, inventory positions, trace views, shipment status, approvals state
  - Mutations that align to use-cases (ReceiveShipment, ReleaseQuarantine, AllocateOrder, ApproveDocPack, etc.)
- Versioning approach:
  - additive-first schema evolution
  - deprecate fields with migration window for clients
- Bulk operations:
  - prefer dedicated mutations with server-side validation and per-row errors

## 7) Compliance & Jurisdiction Model
- Model jurisdiction requirements as **rule templates** + **evaluation engine**:
  - required approvals/documents for operations (ship, export, transfer, release)
  - validity windows and expiry alerts
- Gating:
  - compliance engine returns structured “block reasons” for UI and CLI to show
- Document packs:
  - pack generation produces a stable pack ID + attachments + language selection rules (EN/CH where applicable)

## 8) Security & Audit
- RBAC permissions mapped to ARRIVE component boundaries:
  - inventory, warehouse, compliance, integration actions separated
- Audit trail:
  - record actor, time, request id, before/after summary for sensitive actions
- Operational logs:
  - tracing spans per request, correlation IDs propagated to sync jobs and CLI

## 9) Kingdee Cloud Integration Intent (optional)
- Connector responsibilities:
  - mapping tables (material codes, org/warehouse codes, dimensions)
  - idempotent posting and retry handling
  - sync status ledger + reconciliation reports (qty/value mismatches)
- Integration pattern:
  - background job runner (in-process initially), evolving to separate worker if needed

## 10) Quality Strategy
- Tests:
  - domain unit tests for invariants/state transitions
  - persistence integration tests (SQLite)
  - API tests for GraphQL contract and authorization gates
- Observability:
  - health endpoint, structured logs, trace ids, metrics hooks

## 11) First Moves (mapped to ARRIVE Advances)
- Implement backend **foundation** crate: workspace, config, tracing, minimal Axum server.
- Establish SQLx + SQLite migration baseline in **persistence**.
- Create first vertical slice: inventory item CRUD + stock ledger append + query.
- Add auth boundary and one permission-gated mutation.
