---
advance:
  id: "ADV-FE-MOBILE-001"
  title: "Mobile scan-first subset (optional)"
  system: "frontend"
  primary_component: "mobile"
  components: ["mobile", "warehouse-ui", "foundation"]
  started_at: null
  implementation_completed_at: null
  pr_links: []
  reviewability_score: null
  risk_flags: ["new_platform", "optional"]
  evidence: []
  status: planned
---

## Objective

Create a mobile subset that supports scan-first receiving, putaway, and pick flows. Keep scope minimal and reuse shared UI patterns where possible. Add smoke tests for scan routing.

## Behavioral Change

After this advance:
- Mobile application supports core scan-first workflows (receiving, putaway, pick)
- UI is optimized for mobile devices with large touch targets and simplified navigation
- Shared UI patterns and components are reused from desktop warehouse-ui where possible
- Scan routing works correctly on mobile platform
- Smoke tests verify basic workflow functionality

## Implementation Tasks

- [ ] tidy: review existing warehouse-ui components for mobile reusability
- [ ] test: add smoke tests for scan routing on mobile
- [ ] feat: create mobile application scaffold with navigation
- [ ] feat: implement mobile receiving flow reusing warehouse-ui patterns
- [ ] feat: implement mobile putaway flow with touch-optimized controls
- [ ] feat: implement mobile pick flow with scan verification
- [ ] feat: optimize UI for mobile screen sizes and touch input
- [ ] test: add smoke tests for core workflows

## Risk + Rollback

- Risk: New platform introduces additional complexity, testing burden, and potential platform-specific bugs
- Mitigation: Keep scope minimal (receiving/putaway/pick only), reuse existing patterns, comprehensive smoke tests, mark as optional
- Rollback: Mobile subset is optional and can be deferred or removed without impacting desktop workflows

## Evidence

- [ ] tidy:preparatory - Review existing components for reusability
- [ ] tdd:red-green - Tests written before implementation
- [ ] tests:unit - Smoke tests for scan routing and core workflows
- [ ] ci:passed - All builds successful (including mobile platform)
- [ ] review:owner - Code review completed

## Dependencies

- ADV-WMSUI-002 (Scan-first receiving + putaway)
- ADV-WMSUI-003 (Pick/pack/ship scan workflows)

## Changes Made

(To be filled during implementation)

