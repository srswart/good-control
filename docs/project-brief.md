# Project Brief — Industrial Drone Inventory Management Application (EN + ZH)

## 1) What we’re building
An operator-grade, desktop-first inventory + warehouse + compliance system for industrial drones and their supply chain assets. It tracks drones, parts, batteries, tools/equipment, and consumables, and supports multi-jurisdiction documentation and approvals that can block operations.

**Languages:** English + Chinese. (No Arabic support.)  
**Primary UI:** Desktop (Dioxus 0.7.x + Tailwind).  
**Selective mobile:** iOS/Android for scan-first warehouse tasks and quick lookup (subset).  
**Backend:** Rust (Axum), GraphQL, SQLx, SQLite.  
**Architecture:** crate-based modular decomposition aligned to ARRIVE components (backend, frontend, cli).

### What “done” looks like (plain language)
- Warehouse can **receive**, **quarantine/release**, **put away**, **pick**, **pack**, and **ship** with scan validation.
- Inventory is always explainable: **what changed**, **why**, **who did it**, and **where it is now**.
- Compliance can **block** shipping/release/transfer when required approvals or documents are missing/expired, and the UI shows **exactly what’s missing**.
- Drones and batteries have lifecycle status (“ready”, “grounded”, “in repair”) and can be traced to shipments and work orders.
- Optional: Kingdee Cloud finance sync runs reliably with mappings + reconciliation.

---

## 2) Users, roles, and permissions (RBAC)
### Personas
- **Warehouse operator:** scan-first flows; exceptions; cycle counts
- **Warehouse supervisor:** queue management; exception resolution; productivity
- **Compliance officer:** approvals; document packs; expiry renewals; audit readiness
- **Ops / asset manager:** drone & battery readiness; assignments; maintenance status
- **Procurement:** POs; supplier onboarding docs; inbound readiness
- **Logistics coordinator:** shipment prep; tracking; cross-border packs
- **Finance/ERP admin (optional):** Kingdee mappings; sync monitoring; reconciliation
- **System admin:** RBAC; global config; audit access

### Permission expectations
- Server-enforced RBAC on every mutation.
- Fine-grained permissions by domain area: inventory, warehouse, compliance, integrations, admin.
- Elevated privileges required for destructive operations (scrap, adjustments, force override).
- All sensitive actions produce audit events.

---

## 3) Systems and components (ARRIVE-aligned)

### System: Backend (`arrive/systems/backend/`)
**Stack:** Rust + Axum + GraphQL + SQLx + SQLite  
**Components**
- `foundation` — server/bootstrap, config, tracing, error taxonomy
- `auth` — AuthN/AuthZ, RBAC, audit hooks
- `graphql` — schema/resolvers (API surface)
- `persistence` — SQLx/SQLite, migrations, repositories, transactions
- `i18n` — EN+ZH localization primitives for master data + document selection
- Domain crates:
  - `inventory` — ledger/reservations/traceability/cycle counts
  - `warehouse` — receiving/QC/putaway/pick/pack/ship task model
  - `logistics` — shipments/tracking/3PL
  - `compliance` — jurisdiction rules/approvals/doc packs/gates
  - `assets` — drone registry/config control
  - `battery` — health/cycles/states/retirement/grounding
  - `procurement` — suppliers/POs/inbound docs
  - `orders` — order capture/allocation/fulfillment/backorders
  - `mro` — RMAs/repair work orders/recertification
  - `reporting` — KPI queries/exports primitives
- Integration:
  - `integration-kingdee` — Kingdee connector, mapping, reconciliation

### System: Frontend (`arrive/systems/frontend/`)
**Stack:** Dioxus 0.7.x + Tailwind (desktop-first)  
**Components**
- `foundation` — app bootstrap, Tailwind pipeline, desktop packaging
- `i18n` — EN+ZH translations, language switching, locale formatting
- `design-system` — reusable components/forms/validation patterns
- `app-shell` — routing/session/nav/permission-aware UI
- Feature-area UI crates:
  - `inventory-ui`, `warehouse-ui`, `assets-ui`, `compliance-ui`,
  - `procurement-orders-ui`, `logistics-ui`, `reporting-ui`, `integration-ui`

### System: CLI (`arrive/systems/cli/`)
**Stack:** Rust + clap  
**Components**
- `foundation` — CLI scaffolding, config, logging, output conventions
- `db-migrations` — migrate/verify/backup/restore
- `seed-import` — CSV/JSON seed + import tools
- `export-report` — audit extracts, snapshots, dataset dumps
- `compliance-tools` — validate gates, build doc packs
- `kingdee-sync` — sync trigger/retry/reconcile tooling
- `ops-diagnostics` — DB integrity, connectivity, health checks

---

## 4) Core domain model (buildable data concepts)

This section names the “things” the system must model and the invariants that keep them sane.

### 4.1 Master data
- **Item/SKU**
  - Canonical keys: `sku`, optional `manufacturer_part_no`
  - Flags: `tracking = serial | lot | none`
  - Type: `drone | battery | part | tool | consumable | packaging | service`
  - Bilingual fields: `name_en`, `name_zh`, `description_en`, `description_zh`
  - Optional: hazard class, shelf life, UoM, HS code (if later included)
- **Location**
  - `warehouse` → `zone` → `bin`
  - Special bins: quarantine, staging, repair, scrap
- **Party**
  - suppliers, customers, 3PLs (if needed), carriers (later)
- **BOM / Kit**
  - BOM versioning (drone model build), field kit BOM, maintenance kit BOM

### 4.2 Identity-tracked objects
- **SerialUnit**
  - Unique serial for drones, batteries, and any serial-tracked parts
  - Links to an Item/SKU, current status, current location, and lifecycle history
- **Lot**
  - Lot/batch identity for lot-tracked items, with optional expiry date

### 4.3 Inventory state
**Authoritative principle:** inventory changes are recorded as an **append-only movement ledger**, with derived current positions.
- **InventoryMovement (append-only)**
  - `id`, `timestamp`, `actor`, `reason`, `from_location`, `to_location`, `sku`, `qty`, optional `serial_ids` / `lot_id`, `status_before`, `status_after`
- **InventoryPosition (derived/materialized)**
  - by `sku` + `location` + `status` (+ `lot_id` where relevant)
- **Reservation / Allocation**
  - reserve inventory for a demand source (order, work order, deployment kit)
  - supports partial allocation and backorder

### 4.4 Warehouse execution objects
- **Receipt**
  - expected or ad-hoc; line items; discrepancy capture
- **Inspection / QC**
  - sampling or per-item; produces pass/fail and disposition
- **WarehouseTask**
  - typed tasks: receive, inspect, putaway, replenish, pick, pack, ship-stage, cycle-count
  - scan validation and exception states

### 4.5 Logistics objects
- **Shipment**
  - shipment header, packages, contents, destination, carrier metadata (later)
- **TrackingEvent**
  - status updates; time-stamped timeline

### 4.6 Compliance objects (multi-jurisdiction)
- **Jurisdiction**
  - e.g., UAE, KSA, and extendable list
- **OperationType**
  - ship, transfer, release_quarantine, deploy_to_field, etc.
- **ComplianceRuleTemplate**
  - per jurisdiction + operation type → required approvals/documents + validity rules
- **Approval**
  - workflow states; actor; timestamps; comments; linked artifacts
- **Document**
  - stored metadata with attachments; validity window; language; applicability scope
- **DocumentPack**
  - bundle for shipment/asset/operation; versioned; language selection EN/CH; expiry tracking
- **ComplianceEvaluation**
  - structured output: allowed/blocked + list of missing/expired requirements

### 4.7 Assets and batteries
- **DroneAsset**
  - serial, model, configuration version, firmware, payload list, readiness state
- **Battery**
  - serial, health metrics (cycles, last test), lifecycle state (ready/grounded/retired), assignment pool

### 4.8 Orders, procurement, MRO
- **PurchaseOrder** / **POReceipt**
  - integrate expected receipts with receiving
- **Order**
  - order lines, allocations, fulfillment states, backorders
- **RMA** / **RepairWorkOrder**
  - triage outcomes, parts consumption, recertification evidence

### 4.9 Reporting
- Derived views for:
  - low stock, blocked shipments, expiring docs, readiness, aging inventory
- Export bundles (CSV/JSON) for audits and operational reviews

---

## 5) Key invariants (what must always be true)
These invariants guide tests and keep the system safe under real-world operations.

### Inventory invariants
- InventoryMovement ledger is **append-only** (no edits; corrections are new movements).
- Derived InventoryPosition must never show negative available quantity.
- Reservations cannot exceed available quantity for a given position/status.
- Serial IDs are globally unique; a serial cannot be in two places at once.
- Lot-tracked movements must carry a lot_id; serial-tracked movements must carry serial_ids.

### Warehouse invariants
- Quarantine inventory is not allocatable/shippable until released.
- Pick requires allocation/reservation; pack requires pick completion (unless explicitly overridden by privileged role).

### Compliance invariants
- Operations tagged as “gated” must evaluate compliance before state change is committed.
- When blocked, the system must return deterministic “block reasons” (missing/expired approvals/documents).
- Approvals and document changes are fully audited.

### Asset/battery invariants
- A “grounded” battery cannot be allocated for shipping/deployment.
- Drone readiness states must reflect maintenance/repair outcomes and compliance status (as configured).

---

## 6) Product features (functional requirements)

### 6.1 Master data
- CRUD for items/SKUs with bilingual names/descriptions and tracking flags
- Location model with zone/bin scanning identifiers
- Parties management (suppliers/customers); optional import/export

### 6.2 Inventory ledger + traceability
- Movement capture for all operational actions
- Position queries by sku/location/status/lot
- Reservations and allocations with partial fulfillment
- Serial/lot genealogy (“where used / where shipped”)
- Cycle counting workflows and variance resolution

### 6.3 Warehouse execution (WMS)
- Receiving with discrepancy capture
- QC/inspection with quarantine and release transitions
- Putaway and replenishment tasks
- Pick/pack/ship with scan validation and exception handling

### 6.4 Logistics
- Shipment entity with packages and contents
- Tracking events and shipment status timeline
- Cross-border readiness check (compliance pack validity)

### 6.5 Compliance (gate-driven)
- Rule templates per jurisdiction and operation type
- Approvals workflow and audit trail
- Document repository with expiry tracking
- Document packs and “evidence bundle” generation
- Operation gating that blocks state changes if requirements not met

### 6.6 Assets and batteries
- Drone registry; configuration/version metadata; readiness states
- Battery lifecycle: cycles/health; state transitions; grounding/retirement rules
- Assignment pooling and readiness dashboards

### 6.7 Procurement, orders, MRO
- Purchase orders and expected receipts; receiving closes PO lines
- Orders with allocations, backorders, and fulfillment integration to WMS
- RMA intake, repair work orders, recertification gates

### 6.8 Reporting + exports
- Dashboards: low stock, blocked shipments, expiring docs, readiness, aging inventory
- Exports: snapshots, audit extracts, evidence lists

### 6.9 Optional: Kingdee Cloud financial integration
- Mapping tables + validation
- Sync ledger with idempotent postings and retries
- Reconciliation reports and error drilldown

---

## 7) Internationalization (EN + ZH)
- UI language per user; persisted in profile
- Bilingual master data fields stored and searchable
- Locale formatting: dates/numbers/currency; time zone per warehouse/site
- Document packs: deterministic language selection (EN/CH) and fallback rules

---

## 8) Architecture constraints (how we build it)

### 8.1 Backend architecture
- Axum hosts GraphQL endpoint and minimal operational endpoints (health).
- Domain crates contain pure business logic (no SQL, no HTTP).
- Persistence crate implements repositories and migrations; transactions define atomicity.
- GraphQL crate is the public contract surface; mutations map to use-cases.
- Auth crate enforces RBAC and writes audit events.
- Foundation provides tracing/correlation IDs and job-runner plumbing (for sync jobs).

### 8.2 Frontend architecture
- Dioxus desktop app as primary; mobile subset shares UI crates where possible.
- Design system crate owns controls and patterns, especially scan-first flows.
- GraphQL client layer centralizes error mapping and correlation IDs.
- App shell controls navigation, session, and permission-aware rendering.
- Feature-area crates own screens and workflows.

### 8.3 CLI architecture
- Script-first subcommands (no TUI initially).
- Business actions go through backend GraphQL where feasible (RBAC + gating).
- Direct SQLite access allowed only for migrations/backup/restore/integrity checks.
- Supports `--json` output mode for automation.

---

## 9) Delivery plan: thin vertical slices
Build in reviewable increments; each slice should produce usable functionality.

### Slice 0: Foundations
- Backend/Frontend/CLI scaffolding and crate layout (already in ADV-*-001 drafts)
- CI-ready baseline, test scaffolding, minimal health + hello query

### Slice 1: Inventory MVP
- Items + locations + ledger append + position query
- Desktop inventory search & stock view
- CLI import items & export inventory snapshot

### Slice 2: Receiving + QC + Putaway
- Receiving and inspection; quarantine/release
- Putaway tasks and scan validation (desktop + mobile subset)

### Slice 3: Pick/Pack/Ship + Compliance gating
- Picking and packing flows; shipping staging
- Compliance templates + evaluator; approvals + doc packs
- Hard gate shipping when requirements missing; show block reasons in UI

### Slice 4: Assets + Batteries readiness
- Drone registry, readiness states, configuration metadata
- Battery lifecycle (health, cycles, grounding) integrated with allocations

### Slice 5: Logistics + Reporting + Kingdee (optional)
- Shipment tracking timeline and cross-border readiness checks
- Dashboards + export bundles
- Kingdee mapping/sync/reconcile

---

## 10) Advance backlog (new Advances to create) — with prompts
Each component already has an initial `ADV-*-001`. The Advances below represent the next concrete outcomes to build the system.

### 10.1 Backend
#### `foundation`
- **ADV-BACKEND-002 — Runtime config + tracing standards**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/foundation to implement runtime configuration (env+file), structured tracing with correlation IDs, and graceful shutdown. Include tests for config parsing and a health endpoint. Keep changes reviewable and update evidence.
    ```
- **ADV-BACKEND-003 — Background job runner baseline**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/foundation adding an in-process job runner abstraction (scheduled + ad-hoc jobs) with tracing and retry hooks. No Kingdee specifics yet. Include unit tests for scheduling and execution.
    ```

#### `auth`
- **ADV-AUTH-002 — Auth model + RBAC policy tables**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/auth defining the RBAC model (users, roles, permissions), token/session strategy, and enforcement middleware. Add audit hooks for sensitive mutations. Include unit tests for permission checks and integration tests for a gated GraphQL mutation.
    ```
- **ADV-AUTH-003 — Audit trail persistence**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/auth + persistence to store audit events (actor, action, object, request_id, timestamp). Expose a query to retrieve audit logs with paging. Include tests for write/read and redaction of sensitive fields.
    ```

#### `graphql`
- **ADV-GQL-002 — GraphQL conventions (paging, errors, input validation)**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/graphql to standardize pagination, error mapping (validation vs authorization vs compliance block), and input validation patterns. Update at least one existing query and mutation to use the standard shape and add tests.
    ```
- **ADV-GQL-003 — Bulk mutation pattern**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/graphql introducing a bulk mutation pattern that returns per-row success/error results. Apply it to one inventory or warehouse operation. Include tests validating partial failure handling.
    ```

#### `persistence`
- **ADV-DB-002 — Core schema baseline (items, locations, inventory movements)**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/persistence to establish the core schema for items/SKUs (with EN+ZH fields), locations/bins, and an append-only inventory movement ledger with derived current positions. Include SQLx migrations and integration tests.
    ```
- **ADV-DB-003 — Repository + transaction boundaries**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/persistence introducing repository traits and a transaction helper to ensure multi-step operations (reserve+allocate+ship) are atomic. Add tests that simulate concurrent operations and validate consistency.
    ```
- **ADV-DB-004 — Backup/restore + integrity checks**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/persistence adding database backup/restore primitives and integrity checks compatible with CLI tooling. Include tests for backup creation and restore verification.
    ```

#### `i18n`
- **ADV-I18N-002 — Bilingual master data storage + search hooks**
  - Prompt:
    ```text
    Create an ARRIVE advance for shared/i18n + backend/persistence implementing bilingual storage for item names/descriptions (EN+ZH), locale selection rules, and search normalization hooks. Add tests for lookup and fallback behavior.
    ```
- **ADV-I18N-003 — Document pack language selection**
  - Prompt:
    ```text
    Create an ARRIVE advance for shared/i18n + domain/compliance to support language selection for document packs (EN/CH) with deterministic fallback rules and auditability. Add tests for selection behavior.
    ```

#### `inventory`
- **ADV-INV-002 — Inventory ledger + current position queries**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/inventory + persistence implementing an append-only inventory movement ledger and a current-position query by SKU/location/status. Include unit tests for ledger invariants and integration tests for persistence.
    ```
- **ADV-INV-003 — Reservations + allocations**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/inventory adding reservations/allocations with partial allocation support and conflict prevention. Expose GraphQL mutations and add tests for race conditions and rollback behavior.
    ```
- **ADV-INV-004 — Serial/lot traceability + genealogy**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/inventory implementing serial and lot tracking with genealogy queries (where-used, where-shipped). Include tests for correct lineage across transfers and shipments.
    ```
- **ADV-INV-005 — Cycle counting**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/inventory adding cycle count sessions (ABC rules), variance workflows, and audit records. Provide GraphQL mutations/queries and integration tests.
    ```

#### `warehouse`
- **ADV-WMS-002 — Receiving + QC + quarantine**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/warehouse implementing receiving workflows with discrepancy capture, QC outcomes, and quarantine/release transitions. Integrate with inventory ledger updates and add tests for each state transition.
    ```
- **ADV-WMS-003 — Putaway + replenishment tasks**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/warehouse adding putaway and replenishment task generation and completion flows with scan validation. Include integration tests ensuring inventory positions update correctly.
    ```
- **ADV-WMS-004 — Picking + packing**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/warehouse implementing pick tasks (wave/batch) and pack verification with cartonization primitives. Include tests for partial picks and exception handling.
    ```
- **ADV-WMS-005 — Shipping staging**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/warehouse adding shipping staging and dispatch confirmation, producing shipment records in logistics. Ensure compliance gates can block dispatch. Add tests for blocked vs allowed cases.
    ```

#### `logistics`
- **ADV-LOG-002 — Shipment model + tracking events**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/logistics implementing shipments, packages, and tracking events with a query for shipment status timeline. Include integration tests and GraphQL endpoints.
    ```
- **ADV-LOG-003 — 3PL handoff abstraction**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/logistics defining a 3PL handoff abstraction (create shipment, update status, receive events) without committing to a specific provider. Include a fake adapter and tests.
    ```

#### `compliance`
- **ADV-COMP-002 — Jurisdiction rule templates + evaluator**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/compliance to implement jurisdiction rule templates and an evaluator that returns structured requirements and block reasons per operation type. Add tests for UAE and KSA example templates.
    ```
- **ADV-COMP-003 — Approval workflow**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/compliance implementing submit/review/approve/reject flows with audit records and expiry tracking. Expose GraphQL endpoints and add tests for transitions and permissions.
    ```
- **ADV-COMP-004 — Document packs + gating integration**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/compliance to implement document packs (attachments, versioning, expiry) and wire compliance gating into shipping and quarantine-release operations. Add tests proving operations are blocked when evidence is missing/expired.
    ```

#### `assets`
- **ADV-ASSET-002 — Drone registry + readiness states**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/assets implementing a drone registry with serial identity, configuration metadata, and readiness states. Include tests for state transitions and persistence integration.
    ```
- **ADV-ASSET-003 — Configuration approval by jurisdiction**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/assets + compliance enabling configuration approval flags by jurisdiction/customer. Ensure compliance rules can reference configuration approval status. Add tests for gating behavior.
    ```

#### `battery`
- **ADV-BATT-002 — Battery registry + lifecycle states**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/battery implementing battery registry, lifecycle states (ready/training-only/grounded/retired), and basic policy checks. Integrate with inventory/asset linking and add tests.
    ```
- **ADV-BATT-003 — Health/cycle tracking + retirement policy**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/battery adding cycle count and health test records, plus retirement/grounding rules and alerts. Provide queries for readiness views and tests for policy enforcement.
    ```

#### `procurement`
- **ADV-PROC-002 — Purchase orders + expected receipts**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/procurement implementing purchase orders and expected receipts, and integrate receiving to close PO lines. Add tests for partial receipts and discrepancies.
    ```
- **ADV-PROC-003 — Supplier compliance documents**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/procurement + compliance to require supplier documentation for certain item classes and block receiving when missing. Add tests for required vs optional cases.
    ```

#### `orders`
- **ADV-ORD-002 — Orders + allocation**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/orders + inventory implementing order capture and allocation (reserve/allocate) with backorder behavior. Expose GraphQL and add tests for partial fulfillment.
    ```
- **ADV-ORD-003 — Fulfillment workflow**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/orders + warehouse integrating pick/pack/ship completion to order fulfillment status updates. Add tests that validate order states match warehouse outcomes.
    ```

#### `mro`
- **ADV-MRO-002 — RMA intake + triage**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/mro implementing RMA intake, triage outcomes, and inventory status transitions (return→quarantine/repair/scrap). Add tests for each triage route.
    ```
- **ADV-MRO-003 — Repair work orders + recertification**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/mro implementing repair work orders with parts consumption, test evidence attachment, and recertification gates that return assets to 'ready'. Add tests for consumption and state transitions.
    ```

#### `reporting`
- **ADV-RPT-002 — Operational dashboard queries**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/reporting adding query endpoints for low stock, blocked shipments, expiring docs, and readiness counts. Include integration tests for correctness and paging.
    ```
- **ADV-RPT-003 — Export bundles**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/reporting implementing export bundle generation (CSV/JSON) for inventory snapshots and audit extracts. Wire into CLI and add tests.
    ```

#### `integration-kingdee` (optional)
- **ADV-KDC-002 — Connector auth + mapping tables**
  - Prompt:
    ```text
    Create an ARRIVE advance for integration/kingdee implementing connector authentication, mapping tables (items/orgs/warehouses), and a validation command to detect missing mappings. Include tests and a fake Kingdee adapter.
    ```
- **ADV-KDC-003 — Master data sync**
  - Prompt:
    ```text
    Create an ARRIVE advance for integration/kingdee implementing master data sync (items, customers, suppliers, warehouses) with idempotency and retry. Add tests for retry and duplicate prevention.
    ```
- **ADV-KDC-004 — Transaction sync + reconciliation**
  - Prompt:
    ```text
    Create an ARRIVE advance for integration/kingdee implementing transactional postings (receipts/shipments/adjustments as configured) plus a reconciliation report comparing local ledger to remote postings. Add tests and rollback flags.
    ```

### 10.2 Frontend
#### `foundation`
- **ADV-FE-002 — GraphQL client layer + error mapping**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/foundation adding a typed GraphQL client layer with standardized error handling (auth, validation, compliance blocks) and request correlation IDs. Add a small demo query in the inventory UI and tests for error mapping.
    ```
- **ADV-FE-003 — Desktop packaging + environment config**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/foundation improving desktop packaging and environment configuration (backend URL, feature flags). Keep it minimal and testable (smoke tests or compile-time checks).
    ```

#### `i18n`
- **ADV-I18N-002 — EN+ZH translations + glossary enforcement**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/i18n implementing EN+ZH translation loading, language switching, locale formatting, and a glossary guard for regulated terms. Add tests for fallback behavior and key completeness.
    ```

#### `design-system`
- **ADV-DS-002 — Form/validation primitives**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/design-system adding form components, validation patterns, and consistent error presentation for scan-first and desktop workflows. Add component tests for validation rendering.
    ```

#### `app-shell`
- **ADV-SHELL-002 — Session + permission-aware navigation**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/app-shell implementing session management, login/logout flows, and permission-aware navigation/menu gating. Ensure blocked actions display clear reasons.
    ```

#### Feature UI crates
- **ADV-INVUI-002 — Inventory search + stock view**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/inventory-ui implementing item search, stock position view, and serial/lot trace lookup using the GraphQL standard conventions. Include UX for empty/error/loading states and tests for key interactions.
    ```
- **ADV-WMSUI-002 — Scan-first receiving + putaway**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/warehouse-ui implementing scan-first receiving and putaway flows (desktop + mobile subset). Provide fast feedback, large controls, and resilient UI behavior on transient failures. Add tests for scan routing.
    ```
- **ADV-COMPUI-002 — Approvals + block reasons UX**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/compliance-ui implementing approvals list/detail, document pack viewer, and clear compliance block reasons surfaced from the backend. Add tests for rendering and state transitions.
    ```
- **ADV-ASSETUI-002 — Drone/battery readiness views**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/assets-ui implementing drone and battery readiness dashboards, detail pages, and state transitions (where permitted). Add tests for state-based UI behavior.
    ```
- **ADV-INTUI-002 — Kingdee mapping + sync monitor (optional)**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/integration-ui implementing mapping management screens, sync status views, and error queue drilldown for Kingdee integration. Add tests for list/detail workflows.
    ```

### 10.3 CLI
#### `foundation`
- **ADV-CLI-002 — Standard output modes + auth**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/foundation implementing consistent exit codes, --json output mode, and backend auth handling (token/env/profile). Add tests for command parsing and output formatting.
    ```

#### `db-migrations`
- **ADV-MIG-002 — Backup/restore + verify**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/db-migrations implementing backup/restore/verify commands compatible with SQLite and backend expectations. Add integration tests that create a DB, back it up, restore it, and verify schema.
    ```

#### `seed-import`
- **ADV-SEED-002 — Import items/locations with bilingual fields**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/seed-import adding CSV/JSON import for items and locations with EN+ZH fields, dry-run support, and per-row error reporting. Add tests for parsing and validation.
    ```

#### `export-report`
- **ADV-EXPORT-002 — Inventory snapshots + audit extracts**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/export-report implementing inventory snapshot export and audit extract export (movements, approvals, blocked operations) with filters and --json/--csv outputs. Add tests for deterministic output ordering.
    ```

#### `compliance-tools`
- **ADV-COMP-002 — Validate gates + pack builder**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/compliance-tools implementing commands to evaluate compliance gates for a shipment/operation and to assemble a document pack bundle. Prefer calling backend GraphQL; add tests using a local test backend.
    ```

#### `kingdee-sync` (optional)
- **ADV-KDC-002 — Trigger/retry/reconcile**
  - Prompt:
    ```text
    Create an ARRIVE advance for cli/kingdee-sync implementing sync trigger, retry, and reconciliation commands with clear status output and --json mode. Add tests for idempotent retries and error categorization.
    ```

---

## 11) Governance expectations (ARRIVE)
- Identify impacted system(s) and component(s) before edits (use `system.yaml` roots + component selectors).
- Follow commit order: **tidy → test → implement**.
- Keep within reviewability budget; split advances when necessary.
- When an advance completes, update:
  - `implementation_completed_at`, `status: complete`
  - `pr_links`, `reviewability_score`, and `evidence`
  - append a **Changes Made** log

---

## 12) Definition of done (project-level)
A “buildable milestone” is reached when:
- Backend boots, serves GraphQL, persists to SQLite with migrations.
- Desktop UI can authenticate, browse inventory, and complete one workflow end-to-end (e.g., receive → putaway).
- CLI can migrate DB and export inventory snapshot.
- Compliance gating demonstrably blocks an operation and explains why.
- Audit trail exists for sensitive operations.

---

## 13) File locations (where to look)
- ARRIVE metadata: `arrive/`
- Solution intents: `docs/solution-intents/`
- Project brief: `project-brief.md` (this file)
