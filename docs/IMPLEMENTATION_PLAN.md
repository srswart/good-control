# Implementation Plan: Industrial Drone Inventory Management

> **Project**: drone-inventory  
> **Description**: Inventory + WMS + compliance workflow system for industrial drones (EN+ZH), with optional Kingdee Cloud financial integration.  
> **Created**: 2026-01-25

## Overview

This plan organizes advances across Backend, CLI, and Frontend into logical phases based on dependencies. It is updated to reflect completed work and to surface the most direct path to a functioning, end-to-end system.

## Current Status (as of 2026-01-26)

- **Backend**: foundation, auth, GraphQL baseline, i18n, persistence baseline, and inventory domain (ADV-INV-001..005) are complete. Procurement domain is complete. Warehouse receiving/QC/quarantine with inventory integration is complete (ADV-WMS-002).
- **Frontend**: foundation, design system, app shell, i18n, and feature-area UI scaffolds (inventory, warehouse, assets, compliance, procurement, logistics) are complete but **not wired to backend GraphQL** (placeholder data).
- **CLI**: foundation and migrations are complete; operational tools are still planned.

**Primary gap**: UI-to-backend wiring (GraphQL conventions + API endpoints + client integration).

---

## Phase 0: Foundation Layer (Infrastructure)
**Goal**: Establish the core infrastructure that all other components depend on.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 0.1 | ADV-BACKEND-001 | backend | foundation | None | ✅ Complete |
| 0.2 | ADV-DB-001 | backend | persistence | foundation | ✅ Complete |
| 0.3 | ADV-I18N-001 | backend | i18n | foundation | ✅ Complete |
| 0.4 | ADV-CLI-001 | cli | foundation | backend/foundation | ✅ Complete |
| 0.5 | ADV-MIG-001 | cli | db-migrations | cli/foundation, persistence | ✅ Complete |
| 0.6 | ADV-PERSIST-002 | backend | persistence | foundation | ✅ Complete |

**Rationale**: 
- Backend foundation provides config, tracing, and server infrastructure
- Persistence is required by all domain components
- i18n is crosscutting and needed early for bilingual support
- CLI foundation enables operational tooling
- Migrations enable schema management

---

## Phase 1: Security & API Layer
**Goal**: Establish authentication, authorization, and the API surface.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 1.1 | ADV-AUTH-001 | backend | auth | foundation, persistence | ✅ Complete |
| 1.2 | ADV-GQL-001 | backend | graphql | foundation, auth | ✅ Complete |
| 1.3 | ADV-LOG-001 | backend | logging/audit | auth | ✅ Complete |

**Rationale**:
- Auth must exist before exposing any API
- GraphQL is the primary API surface for frontend
- Audit logging integrates with auth for compliance

---

## Phase 2: Core Domain - Assets & Inventory
**Goal**: Implement the core business domain for drone asset management.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 2.1 | ADV-ASSET-001 | backend | assets | persistence, graphql | Planned |
| 2.2 | ADV-BATT-001 | backend | battery | assets | Planned |
| 2.3 | ADV-INV-001 | backend | inventory | assets, persistence | ✅ Complete |
| 2.4 | ADV-INV-002 | backend | inventory | inventory | ✅ Complete |
| 2.5 | ADV-INV-003 | backend | inventory | inventory | ✅ Complete |
| 2.6 | ADV-INV-004 | backend | inventory | inventory | ✅ Complete |
| 2.7 | ADV-INV-005 | backend | inventory | inventory | ✅ Complete |
| 2.8 | ADV-COMP-001 | backend | compliance | assets, inventory | Planned |

**Rationale**:
- Assets (drones) are the primary entities
- Batteries have lifecycle tied to assets
- Inventory tracks stock levels and serial/lot traceability
- Compliance gates depend on asset and inventory data

---

## Phase 3: Warehouse & Logistics
**Goal**: Implement warehouse management and logistics operations.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 3.1 | ADV-WMS-001 | backend | warehouse | inventory, assets | Planned |
| 3.2 | ADV-WMS-002 | backend | warehouse | inventory | ✅ Complete |
| 3.3 | ADV-ORD-001 | backend | orders | inventory, warehouse | Planned |
| 3.4 | ADV-PROC-001 | backend | procurement | inventory, orders | ✅ Complete |
| 3.5 | ADV-MRO-001 | backend | mro | assets, inventory, orders | Planned |

**Rationale**:
- Warehouse operations (receive/putaway/pick/pack/ship) depend on inventory
- Orders drive allocation and fulfillment
- Procurement handles inbound supply chain
- MRO (maintenance/repair/returns) spans assets and orders

---

## Phase 4: Reporting & Integration
**Goal**: Add reporting capabilities and external system integration.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 4.1 | ADV-RPT-001 | backend | reporting | All domain components | Planned |
| 4.2 | ADV-KDC-001 | backend | integration-kingdee | orders, inventory | Planned |
| 4.3 | ADV-EXPORT-001 | cli | export-report | reporting | Planned |
| 4.4 | ADV-KDC-001 (cli) | cli | kingdee-sync | integration-kingdee | Planned |

**Rationale**:
- Reporting aggregates data from all domains
- Kingdee integration is optional but valuable for financial reconciliation
- CLI tools provide operational access to reports and sync

---

## Phase 5: Frontend Foundation
**Goal**: Establish the desktop application infrastructure.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 5.1 | ADV-FE-001 | frontend | foundation | backend/graphql | ✅ Complete |
| 5.2 | ADV-DS-001 | frontend | design-system | frontend/foundation | ✅ Complete |
| 5.3 | ADV-SHELL-001 | frontend | app-shell | design-system | ✅ Complete |
| 5.4 | ADV-I18N-001 (fe) | frontend | i18n | foundation | ✅ Complete |

**Rationale**:
- Frontend foundation sets up Dioxus 0.7.x and Tailwind
- Design system provides reusable components
- App shell provides navigation and layout
- i18n enables EN+ZH language switching

---

## Phase 6: Frontend Feature Areas
**Goal**: Implement the UI for each domain area.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 6.1 | ADV-ASSETUI-001 | frontend | assets-ui | app-shell, backend/assets | 🟡 Scaffolded (backend wiring pending) |
| 6.2 | ADV-INVUI-001 | frontend | inventory-ui | app-shell, backend/inventory | 🟡 Scaffolded (backend wiring pending) |
| 6.3 | ADV-WMSUI-001 | frontend | warehouse-ui | app-shell, backend/warehouse | 🟡 Scaffolded (backend wiring pending) |
| 6.4 | ADV-POUI-001 | frontend | procurement-orders-ui | app-shell, backend/orders | 🟡 Scaffolded (backend wiring pending) |
| 6.5 | ADV-LOGUI-001 | frontend | logistics-ui | app-shell, backend/logistics | 🟡 Scaffolded (backend wiring pending) |
| 6.6 | ADV-COMPUI-001 | frontend | compliance-ui | app-shell, backend/compliance | 🟡 Scaffolded (backend wiring pending) |
| 6.7 | ADV-RPTUI-001 | frontend | reporting-ui | app-shell, backend/reporting | Planned |
| 6.8 | ADV-INTUI-001 | frontend | integration-ui | app-shell, backend/integration-kingdee | Planned |

**Rationale**:
- Each UI area maps to a backend domain component
- All depend on app-shell for navigation
- Can be developed in parallel once app-shell is complete

---

## Phase 6.5: API + UI Wiring (Bridge)
**Goal**: Connect completed UI scaffolds to backend GraphQL APIs with consistent error handling.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 6.5.1 | ADV-GQL-002 | backend | graphql | auth, graphql baseline | Planned |
| 6.5.2 | ADV-GQL-003 | backend | graphql | ADV-GQL-002 | Planned |
| 6.5.3 | **NEW: ADV-GQL-004** | backend | graphql | ADV-INV-001..005, ADV-GQL-002 | New (create advance) |
| 6.5.4 | **NEW: ADV-GQL-005** | backend | graphql | ADV-WMS-002, ADV-GQL-002 | New (create advance) |
| 6.5.5 | ADV-FE-002 | frontend | foundation | ADV-GQL-002 | Planned |
| 6.5.6 | ADV-INVUI-002 | frontend | inventory-ui | ADV-GQL-004, ADV-FE-002 | Planned |
| 6.5.7 | ADV-WMSUI-002 | frontend | warehouse-ui | ADV-GQL-005, ADV-FE-002 | Planned |

**Rationale**:
- Backend APIs must stabilize before wiring UI
- GraphQL conventions and client error mapping prevent UI rework
- Inventory + receiving are the first end-to-end slice

---

## Phase 7: CLI Operations & Tooling
**Goal**: Complete operational CLI tools.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 7.1 | ADV-SEED-001 | cli | seed-import | persistence | Planned |
| 7.2 | ADV-OPS-001 | cli | ops-diagnostics | All backend components | Planned |
| 7.3 | ADV-COMP-001 (cli) | cli | compliance-tools | compliance | Planned |

**Rationale**:
- Seed/import enables initial data loading
- Ops diagnostics help with troubleshooting
- Compliance tools support audit workflows

---

## Dependency Graph

```
                    ┌─────────────────────────────────────────────────────────────┐
                    │                     PHASE 0: FOUNDATION                      │
                    └─────────────────────────────────────────────────────────────┘
                                                │
                    ┌───────────────────────────┼───────────────────────────┐
                    ▼                           ▼                           ▼
            ┌──────────────┐           ┌──────────────┐           ┌──────────────┐
            │ ADV-BACKEND  │           │  ADV-DB-001  │           │ ADV-I18N-001 │
            │     -001     │──────────▶│ persistence  │           │     i18n     │
            │  foundation  │           └──────────────┘           └──────────────┘
            └──────────────┘                   │
                    │                          │
                    ▼                          ▼
            ┌──────────────┐           ┌──────────────┐
            │ ADV-CLI-001  │           │ ADV-MIG-001  │
            │ cli/foundation│◀─────────│ migrations   │
            └──────────────┘           └──────────────┘
                                               │
                    ┌──────────────────────────┘
                    ▼
            ┌─────────────────────────────────────────────────────────────┐
            │                     PHASE 1: SECURITY & API                  │
            └─────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    ▼                          ▼                          ▼
            ┌──────────────┐           ┌──────────────┐           ┌──────────────┐
            │ ADV-AUTH-001 │──────────▶│ ADV-GQL-001  │           │ ADV-LOG-001  │
            │     auth     │           │   graphql    │           │    audit     │
            └──────────────┘           └──────────────┘           └──────────────┘
                                               │
                    ┌──────────────────────────┘
                    ▼
            ┌─────────────────────────────────────────────────────────────┐
            │                  PHASE 2: CORE DOMAIN                        │
            └─────────────────────────────────────────────────────────────┘
                                               │
            ┌──────────────┬───────────────────┼───────────────────┬──────────────┐
            ▼              ▼                   ▼                   ▼              ▼
     ┌────────────┐ ┌────────────┐     ┌────────────┐     ┌────────────┐ ┌────────────┐
     │ADV-ASSET   │ │ADV-BATT    │     │ADV-INV-001 │     │ADV-COMP    │ │ADV-WMS-001 │
     │   -001     │▶│   -001     │     │ inventory  │     │   -001     │ │ warehouse  │
     │  assets    │ │  battery   │     └────────────┘     │ compliance │ └────────────┘
     └────────────┘ └────────────┘                        └────────────┘
                                               │
                    ┌──────────────────────────┘
                    ▼
            ┌─────────────────────────────────────────────────────────────┐
            │              PHASE 3-4: LOGISTICS, REPORTING, INTEGRATION    │
            └─────────────────────────────────────────────────────────────┘
                                               │
                    ┌──────────────────────────┘
                    ▼
            ┌─────────────────────────────────────────────────────────────┐
            │                    PHASE 5-6: FRONTEND                       │
            └─────────────────────────────────────────────────────────────┘
```

---

## Implementation Guidelines

### Branch Naming
Per ARRIVE rules, use: `{system}/{advance_id}`
- Example: `backend/ADV-DB-001`

### Commit Sequence
Follow Tidy First + TDD:
1. `tidy: <description>` - Preparatory refactoring
2. `test: <description>` - Write failing tests
3. `feat: <description>` or `fix: <description>` - Implementation

### Evidence Requirements
Each advance must capture:
- `tests:unit` - Unit tests passing
- `tests:integration` - Integration tests (where applicable)
- `tdd:red-green` - Tests written before implementation
- `ci:passed` - CI pipeline green

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 0: Foundation | 1 week | Week 1 |
| Phase 1: Security & API | 1 week | Week 2 |
| Phase 2: Core Domain | 2 weeks | Week 4 |
| Phase 3: Warehouse & Logistics | 2 weeks | Week 6 |
| Phase 4: Reporting & Integration | 1 week | Week 7 |
| Phase 5: Frontend Foundation | 1 week | Week 8 |
| Phase 6: Frontend Features | 3 weeks | Week 11 |
| Phase 7: CLI Tooling | 1 week | Week 12 |

**Total Estimated Duration**: 12 weeks

---

## Next Steps

1. **Immediate**: Create advances for **API wiring** (NEW: ADV-GQL-004, ADV-GQL-005) and implement GraphQL conventions + client layer (ADV-GQL-002, ADV-FE-002).
2. **This Week**: Wire inventory + receiving UI to backend (ADV-INVUI-002, ADV-WMSUI-002) to deliver the first end-to-end workflow (receive → inventory update).
3. **Next**: Expand core domain coverage (ADV-ASSET-001, ADV-BATT-001, ADV-COMP-001, ADV-ORD-001) and continue WMS execution (ADV-WMS-003/004/005).

---

## Advance Prompts (incorporated)
The prompts below are pulled into the implementation plan so the execution path is explicit. The full catalog remains in `docs/project-brief.md` (Section 10).

### Immediate: API + UI Wiring
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
- **NEW: ADV-GQL-004 — Inventory API surface (positions, movements, reservations)**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/graphql exposing inventory queries (positions, movements) and mutations (reserve/release/adjust) using the new GraphQL conventions. Include tests for pagination, validation, and error mapping.
    ```
- **NEW: ADV-GQL-005 — Warehouse API surface (receiving/QC/quarantine)**
  - Prompt:
    ```text
    Create an ARRIVE advance for backend/graphql exposing warehouse receiving workflows (create task, record receipt, record QC, quarantine/release) and link them to inventory updates. Include tests for gating, validation, and error mapping.
    ```
- **ADV-FE-002 — GraphQL client layer + error mapping**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/foundation adding a typed GraphQL client layer with standardized error handling (auth, validation, compliance blocks) and request correlation IDs. Add a small demo query in the inventory UI and tests for error mapping.
    ```
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

### Near-term: Core Domain Expansion
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
- **ADV-ASSET-002 — Drone registry + readiness states**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/assets implementing a drone registry with serial identity, configuration metadata, and readiness states. Include tests for state transitions and persistence integration.
    ```
- **ADV-BATT-002 — Battery registry + lifecycle states**
  - Prompt:
    ```text
    Create an ARRIVE advance for domain/battery implementing battery registry, lifecycle states (ready/training-only/grounded/retired), and basic policy checks. Integrate with inventory/asset linking and add tests.
    ```

### Gaps from Product Brief (new advances to add)
- **NEW: ADV-WMSUI-003 — Pick/pack/ship scan workflows**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/warehouse-ui implementing scan-first pick, pack, and ship-staging workflows with exception handling and clear block reasons. Add tests for scan routing and error recovery.
    ```
- **NEW: ADV-LOGUI-002 — Shipment timeline + tracking events**
  - Prompt:
    ```text
    Create an ARRIVE advance for frontend/logistics-ui implementing shipment detail views with tracking timeline and package contents. Add tests for empty/error/loading states.
    ```
- **NEW: ADV-FE-MOBILE-001 — Mobile scan-first subset (optional)**
  - Prompt:
    ```text
    Create an ARRIVE advance for a mobile subset (frontend) that supports scan-first receiving, putaway, and pick flows. Keep scope minimal and reuse shared UI patterns where possible. Add smoke tests for scan routing.
    ```

## Risk Considerations

| Risk | Mitigation |
|------|------------|
| Schema evolution | Keep migrations reversible; use feature flags |
| Auth complexity | Start with simple RBAC; iterate |
| Kingdee integration | Mark as optional; implement last |
| Dioxus 0.7.x stability | Pin versions; test on target platforms early |
| Bilingual complexity | Implement i18n early; test with both locales |
