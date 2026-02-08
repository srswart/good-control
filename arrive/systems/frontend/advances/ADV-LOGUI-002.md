---
advance:
  id: "ADV-LOGUI-002"
  title: "Shipment timeline + tracking events"
  system: "frontend"
  primary_component: "logistics-ui"
  components: ["logistics-ui", "foundation"]
  started_at: null
  implementation_completed_at: null
  pr_links: []
  reviewability_score: null
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement shipment detail views with tracking timeline and package contents. Add tests for empty/error/loading states.

## Behavioral Change

After this advance:
- Logistics UI displays shipment detail views with comprehensive tracking information
- Tracking timeline shows chronological events (created, picked up, in transit, delivered, etc.)
- Package contents view shows items, quantities, and serial/lot numbers where applicable
- All views handle empty states (no tracking events), error states, and loading states gracefully
- State handling is covered by tests

## Implementation Tasks

- [ ] tidy: review existing logistics UI components and GraphQL client patterns
- [ ] test: add tests for empty/error/loading state handling
- [ ] feat: implement shipment detail view with GraphQL queries
- [ ] feat: implement tracking timeline component with event display
- [ ] feat: implement package contents view with item details
- [ ] feat: add empty/error/loading state UX for all views
- [ ] test: add tests for timeline rendering and state transitions

## Risk + Rollback

- Risk: GraphQL query errors, missing tracking data, or slow carrier API responses could degrade UX
- Mitigation: Comprehensive error handling with clear user feedback, loading indicators, graceful degradation for missing data
- Rollback: Feature-flag new shipment detail views, fall back to basic shipment list if needed

## Evidence

- [ ] tidy:preparatory - Review existing components and patterns
- [ ] tdd:red-green - Tests written before implementation
- [ ] tests:unit - Unit tests for timeline, package contents, and state handling
- [ ] ci:passed - All builds successful
- [ ] review:owner - Code review completed

## Dependencies

- ADV-WMS-005 (Shipping staging)
- ADV-FE-002 (GraphQL client layer + error mapping)

## Changes Made

(To be filled during implementation)

