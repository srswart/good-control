# Implementation Plan: Industrial Drone Inventory Management

> **Project**: drone-inventory  
> **Description**: Inventory + WMS + compliance workflow system for industrial drones (EN+ZH), with optional Kingdee Cloud financial integration.  
> **Created**: 2026-01-25

## Overview

This plan organizes the 35 planned advances across 3 systems (Backend, CLI, Frontend) into logical phases based on dependencies. Each phase builds on the previous, ensuring a stable foundation before adding complexity.

---

## Phase 0: Foundation Layer (Infrastructure)
**Goal**: Establish the core infrastructure that all other components depend on.

| Order | Advance | System | Component | Dependencies | Status |
|-------|---------|--------|-----------|--------------|--------|
| 0.1 | ADV-BACKEND-001 | backend | foundation | None | ✅ Complete |
| 0.2 | ADV-DB-001 | backend | persistence | foundation | Planned |
| 0.3 | ADV-I18N-001 | backend | i18n | foundation | Planned |
| 0.4 | ADV-CLI-001 | cli | foundation | backend/foundation | Planned |
| 0.5 | ADV-MIG-001 | cli | db-migrations | cli/foundation, persistence | Planned |

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
| 1.1 | ADV-AUTH-001 | backend | auth | foundation, persistence | Planned |
| 1.2 | ADV-GQL-001 | backend | graphql | foundation, auth | Planned |
| 1.3 | ADV-LOG-001 | backend | logging/audit | auth | Planned |

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
| 2.3 | ADV-INV-001 | backend | inventory | assets, persistence | Planned |
| 2.4 | ADV-COMP-001 | backend | compliance | assets, inventory | Planned |

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
| 3.2 | ADV-ORD-001 | backend | orders | inventory, warehouse | Planned |
| 3.3 | ADV-PROC-001 | backend | procurement | inventory, orders | Planned |
| 3.4 | ADV-MRO-001 | backend | mro | assets, inventory, orders | Planned |

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
| 5.1 | ADV-FE-001 | frontend | foundation | backend/graphql | Planned |
| 5.2 | ADV-DS-001 | frontend | design-system | frontend/foundation | Planned |
| 5.3 | ADV-SHELL-001 | frontend | app-shell | design-system | Planned |
| 5.4 | ADV-I18N-001 (fe) | frontend | i18n | foundation | Planned |

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
| 6.1 | ADV-ASSETUI-001 | frontend | assets-ui | app-shell, backend/assets | Planned |
| 6.2 | ADV-INVUI-001 | frontend | inventory-ui | app-shell, backend/inventory | Planned |
| 6.3 | ADV-WMSUI-001 | frontend | warehouse-ui | app-shell, backend/warehouse | Planned |
| 6.4 | ADV-POUI-001 | frontend | procurement-orders-ui | app-shell, backend/orders | Planned |
| 6.5 | ADV-LOGUI-001 | frontend | logistics-ui | app-shell, backend/logistics | Planned |
| 6.6 | ADV-COMPUI-001 | frontend | compliance-ui | app-shell, backend/compliance | Planned |
| 6.7 | ADV-RPTUI-001 | frontend | reporting-ui | app-shell, backend/reporting | Planned |
| 6.8 | ADV-INTUI-001 | frontend | integration-ui | app-shell, backend/integration-kingdee | Planned |

**Rationale**:
- Each UI area maps to a backend domain component
- All depend on app-shell for navigation
- Can be developed in parallel once app-shell is complete

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

1. **Immediate**: Complete Phase 0 (ADV-DB-001, ADV-I18N-001, ADV-CLI-001, ADV-MIG-001)
2. **This Week**: Establish persistence layer and CLI foundation
3. **Next Week**: Begin Phase 1 with auth and GraphQL

---

## Risk Considerations

| Risk | Mitigation |
|------|------------|
| Schema evolution | Keep migrations reversible; use feature flags |
| Auth complexity | Start with simple RBAC; iterate |
| Kingdee integration | Mark as optional; implement last |
| Dioxus 0.7.x stability | Pin versions; test on target platforms early |
| Bilingual complexity | Implement i18n early; test with both locales |
