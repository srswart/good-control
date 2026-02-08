# UI Automation Testing Plan

## Overview
This document outlines the complete plan for implementing automated UI testing for the inventory management system. The approach uses **Playwright** for browser automation with **Dioxus running in web mode**.

## Why Web Mode?
- **Desktop apps are hard to automate**: Native desktop automation tools are limited and platform-specific
- **Web automation is mature**: Playwright provides excellent browser automation with reliable selectors, screenshots, videos, and debugging
- **Dioxus supports both**: Our Dioxus 0.7 app can run as both desktop (production) and web (testing)
- **Best of both worlds**: Ship desktop app to users, test with web automation

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Production (Desktop)                     │
│  apps/desktop → Dioxus Desktop → Native Window              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    Testing (Web + Playwright)                │
│  apps/web → Dioxus Web → Browser ← Playwright Tests         │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Web Build Setup (ADV-UITEST-001)
**Goal**: Enable Dioxus to run as web application

**Deliverables**:
- `apps/web/` - New web app entry point
- `scripts/run-web.sh` - Script to run web app
- Web app accessible at `http://localhost:8080`

**Timeline**: 1-2 days

---

### Phase 2: Playwright Infrastructure (ADV-UITEST-002)
**Goal**: Install and configure Playwright test framework

**Deliverables**:
- `tests/ui/` - Playwright test directory
- `playwright.config.ts` - Test configuration
- `scripts/test-ui.sh` - Test runner script
- Basic smoke tests passing

**Timeline**: 1 day

---

### Phase 3: Test Utilities (ADV-UITEST-003)
**Goal**: Create reusable test helpers and page objects

**Deliverables**:
- Page object models (e.g., `ItemsListPage`)
- Navigation helpers
- Assertion helpers
- Test data fixtures
- Database seeding utilities

**Timeline**: 2 days

---

### Phase 4: Inventory Items Tests (ADV-UITEST-004)
**Goal**: Comprehensive tests for inventory items view

**Test Coverage**:
- ✅ Page load and initial state
- ✅ Search by SKU, name_en, name_zh
- ✅ Search with Enter key
- ✅ Pagination (next, previous, disabled states)
- ✅ Localization (English ↔ Chinese)
- ✅ Loading states
- ✅ Error states
- ✅ Empty states

**Timeline**: 2-3 days

---

### Phase 5: CI/CD Integration (ADV-UITEST-005)
**Goal**: Automate test execution in CI/CD pipeline

**Deliverables**:
- GitHub Actions workflow
- Pre-push hooks
- Test result reporting
- Screenshot/video artifacts
- Flaky test detection

**Timeline**: 1-2 days

---

## Total Timeline: 7-10 days

## Test Development Workflow

### When Developing New UI Features:

1. **Build the UI component** (e.g., in `crates/ui/inventory/`)
2. **Create page object** (e.g., `tests/ui/tests/pages/NewFeaturePage.ts`)
3. **Write tests** (e.g., `tests/ui/tests/inventory/new-feature.spec.ts`)
4. **Run tests locally**:
   ```bash
   ./scripts/test-ui.sh headed  # Watch tests run
   ```
5. **Fix issues** found by tests
6. **Commit code** - tests run automatically in CI

### When Bugs Are Found:

1. **Write failing test** that reproduces the bug
2. **Fix the bug** in the UI code
3. **Verify test passes**
4. **Commit** - test prevents regression

## Benefits

### For Development:
- **Catch bugs early**: Tests run on every commit
- **Prevent regressions**: Old bugs can't come back
- **Faster debugging**: Screenshots/videos show exactly what failed
- **Confidence in refactoring**: Tests ensure nothing breaks

### For Quality:
- **Consistent testing**: Same tests run every time
- **Multi-locale testing**: Automatically test EN and ZH
- **Cross-browser testing**: Can add Firefox, Safari later
- **Performance monitoring**: Track page load times

### For Team:
- **Living documentation**: Tests show how features work
- **Onboarding**: New developers can see expected behavior
- **Collaboration**: Tests define acceptance criteria
- **Reduced manual testing**: Automation handles repetitive checks

## Example Test

```typescript
test('should search inventory items by SKU', async ({ page }) => {
  // Arrange
  const itemsPage = new ItemsListPage(page);
  await itemsPage.goto();
  
  // Act
  await itemsPage.search('DJI-M300');
  
  // Assert
  await itemsPage.assertHasResults();
  const firstRowSku = await itemsPage.getTableCellText(1, 1);
  expect(firstRowSku).toContain('DJI-M300');
});
```

## Running Tests

```bash
# Run all tests (headless)
./scripts/test-ui.sh

# Run with browser visible (for debugging)
./scripts/test-ui.sh headed

# Run specific test file
cd tests/ui
npm test -- items.spec.ts

# Debug mode (step through tests)
./scripts/test-ui.sh debug

# View test report
cd tests/ui && npm run test:report
```

## Next Steps

1. **Review this plan** - Ensure it meets your needs
2. **Start with ADV-UITEST-001** - Set up web build
3. **Implement advances in order** - Each builds on the previous
4. **Expand test coverage** - Add tests for other UI features
5. **Integrate into workflow** - Make testing part of development process

## Questions to Consider

- Should we add Firefox/Safari testing? (Can add later)
- Do we need visual regression testing? (Screenshots comparison)
- Should tests run on every commit or just PRs? (Configurable)
- How do we handle test data? (Seed scripts or fixtures)

## Resources

- [Playwright Documentation](https://playwright.dev)
- [Dioxus Web Platform](https://dioxuslabs.com/learn/0.7/getting_started/web)
- [Page Object Pattern](https://playwright.dev/docs/pom)

