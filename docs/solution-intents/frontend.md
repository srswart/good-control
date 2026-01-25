# Solution Intent — Frontend
**Date:** 2026-01-25  
**System:** frontend  
**Stack:** Dioxus 0.7.x, Tailwind (desktop-first), selective mobile (iOS/Android)  
**Languages:** English + Chinese (no Arabic)

## 1) Outcomes
- Provide a fast, operator-friendly desktop UI for inventory + warehouse + compliance + assets.
- Support scan-first flows (warehouse tasks) and clear explanations for compliance blocks.
- Keep UI modular: crate-per-area with a shared design system.
- Maintain a stable client contract through GraphQL with strong typing and predictable caching.

## 2) In Scope
- Desktop application as primary surface:
  - Inventory, WMS, logistics, compliance, procurement/orders, assets/batteries, reporting, integrations
- Mobile surface (subset):
  - scan-first receiving/picking/packing where needed
- i18n:
  - English + Chinese UI
  - multilingual item names/descriptions displayed correctly

## 3) Out of Scope (initially)
- Full offline mode for mobile warehouse execution (allow limited cached reads; writes require connectivity).
- Deep “BI” charting; start with actionable dashboards and export links.

## 4) Architecture Approach
### 4.1 Crate layout
- `crates/ui/foundation` — app bootstrap, theme, Tailwind integration, error boundary
- `crates/ui/i18n` — translation loading, language switcher, locale formatting
- `crates/ui/design_system` — buttons/forms/tables/modals/toasts/validation patterns
- `crates/ui/app_shell` — routing, nav, session, permission-aware rendering
- Feature-area UI crates:
  - `crates/ui/inventory`
  - `crates/ui/warehouse` (includes scan-first patterns)
  - `crates/ui/assets`
  - `crates/ui/compliance`
  - `crates/ui/procurement_orders`
  - `crates/ui/logistics`
  - `crates/ui/reporting`
  - `crates/ui/integrations`
- Apps:
  - `apps/desktop` — desktop packaging and entrypoint
  - `apps/mobile` — mobile entrypoint(s) for subset workflows

### 4.2 Client state & data
- GraphQL client layer:
  - typed query/mutation modules per UI area
  - standardized error mapping (auth, validation, compliance gate blocks)
- State strategy:
  - local component state for UI interaction
  - shared app state for session + language + permissions + notifications
- Caching:
  - conservative caching for reference data (items, locations)
  - task lists and approvals should prefer freshness with explicit refresh

## 5) UX Intent (operator-focused)
- Scan-first patterns for warehouse:
  - “big target” UI, minimal typing, clear next action, audible/visual feedback
- Compliance:
  - always show *why* an operation is blocked and what evidence is missing/expired
- Traceability:
  - fast lookup by SKU/serial/lot with a clean “where used / where shipped” view

## 6) i18n Intent (EN + ZH)
- UI language per user profile with quick switch.
- Translation keys centralized; glossary enforced for regulated terms.
- Support mixed-language display in lists (e.g., EN name + ZH name).
- Locale formatting for dates/numbers/currency based on user preference/site.

## 7) Desktop + Mobile Intent
- Desktop:
  - primary for planning, approvals, reporting, integration setup
- Mobile:
  - warehouse execution subset (receiving/picking/packing), optimized for camera scanning
- Keep shared UI logic in crates; keep platform-specific code in app entrypoints.

## 8) Quality Strategy
- Component-level tests for critical UI logic and validation.
- Contract tests for GraphQL query shape (keep breakage visible).
- Visual consistency via design system.

## 9) First Moves (mapped to ARRIVE Advances)
- Frontend **foundation**: Dioxus 0.7.x + Tailwind pipeline + desktop packaging.
- App shell + auth/session flow (login, token handling, permission gating).
- First vertical slice: inventory item search + stock view + trace lookup.
