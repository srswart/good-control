# UI Automated Tests

Automated browser-based UI tests using Playwright.

## Quick Start

### Installation

```bash
cd tests/ui
npm install
npx playwright install chromium
```

### Running Tests

```bash
# From project root
./scripts/test-ui.sh              # Headless mode (default)
./scripts/test-ui.sh headed       # Browser visible
./scripts/test-ui.sh debug        # Debug mode with inspector
./scripts/test-ui.sh ui           # Interactive UI mode

# From tests/ui directory
npm test                          # Headless mode
npm run test:headed               # Browser visible
npm run test:debug                # Debug mode
npm run test:ui                   # Interactive UI mode
npm run test:report               # View HTML report
```

## Test Structure

```
tests/ui/
├── package.json              # NPM dependencies
├── playwright.config.ts      # Playwright configuration
├── tsconfig.json            # TypeScript configuration
├── tests/                   # Test files
│   ├── inventory/          # Inventory feature tests
│   │   └── items.spec.ts   # Items list tests (16 tests)
│   ├── pages/              # Page object models
│   │   ├── BasePage.ts     # Base page with common functionality
│   │   └── ItemsListPage.ts # Items list page object
│   ├── helpers/            # Reusable helper functions
│   │   ├── navigation.ts   # Navigation helpers
│   │   └── assertions.ts   # Custom assertions
│   ├── fixtures/           # Test data
│   │   └── test-data.ts    # Test fixtures and constants
│   └── smoke.spec.ts       # Basic smoke tests (3 tests)
├── test-results/           # Test artifacts (screenshots, videos)
└── playwright-report/      # HTML test reports
```

## Test Coverage

**Total: 19 tests (all passing)**

### Inventory Items Tests (16 tests)
- **Page Load** (3 tests)
  - Page loads successfully
  - Table displays data
  - Table headers are correct

- **Search Functionality** (6 tests)
  - Search by SKU
  - Search by English name
  - Search by Chinese name
  - Empty state for no results
  - Clear search
  - Search with Enter key

- **Pagination** (4 tests)
  - Previous button disabled on first page
  - Navigate to next page
  - Navigate back to previous page
  - Next button disabled on last page

- **Localization** (3 tests)
  - Display Chinese UI
  - Switch back to English
  - Preserve data in both languages

### Smoke Tests (3 tests)
- Home page loads
- Navigate to inventory
- Language switching

## Configuration

- **Base URL:** http://localhost:8080
- **Browser:** Chromium (Desktop Chrome)
- **Auto-start:** Web server starts automatically before tests
- **Screenshots:** Captured on failure only
- **Videos:** Retained on failure only
- **Traces:** Captured on first retry

## Writing Tests

### Using Page Objects

```typescript
// tests/pages/MyPage.ts
import { Page, Locator } from '@playwright/test';

export class MyPage {
  readonly page: Page;
  readonly myButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.myButton = page.locator('button.my-button');
  }

  async goto() {
    await this.page.goto('/my-page');
  }

  async clickButton() {
    await this.myButton.click();
  }
}
```

### Using Test Data

```typescript
// tests/fixtures/test-data.ts
export const MY_TEST_DATA = {
  validInput: 'test-value',
  invalidInput: 'bad-value'
};
```

### Writing Test Suites

```typescript
// tests/my-feature/my-test.spec.ts
import { test, expect } from '@playwright/test';
import { MyPage } from '../pages/MyPage';
import { MY_TEST_DATA } from '../fixtures/test-data';

test.describe('My Feature', () => {
  let myPage: MyPage;

  test.beforeEach(async ({ page }) => {
    myPage = new MyPage(page);
    await myPage.goto();
  });

  test('should do something', async () => {
    await myPage.clickButton();
    // assertions...
  });
});
```

### Best Practices

1. **Use Page Objects** - Encapsulate page structure and behavior
2. **Use Helpers** - Extract common operations into helper functions
3. **Use Test Data** - Keep test data in fixtures for consistency
4. **Independent Tests** - Each test should be able to run independently
5. **Descriptive Names** - Use clear, descriptive test names
6. **Wait Appropriately** - Use proper waits instead of arbitrary timeouts

## Debugging

### View Test Report
```bash
cd tests/ui
npm run test:report
```

### Debug Mode
```bash
./scripts/test-ui.sh debug
```

This opens the Playwright Inspector where you can:
- Step through tests
- Inspect selectors
- View console logs
- See network requests

### Screenshots and Videos

Failed tests automatically capture:
- Screenshots in `test-results/`
- Videos in `test-results/`
- Traces for debugging

## CI/CD Integration

The tests are configured to run in CI/CD environments:
- Retries: 2 attempts on failure
- Workers: 1 (sequential execution)
- Server: Auto-starts web app
- Reports: JSON and HTML formats

## Troubleshooting

### Web server doesn't start
Make sure the web app builds successfully:
```bash
cd apps/web
dx build --platform web
```

### Tests timeout
Increase timeout in `playwright.config.ts`:
```typescript
webServer: {
  timeout: 180000, // 3 minutes
}
```

### Browser not found
Reinstall browsers:
```bash
npx playwright install chromium
```

## Implementation Status

- ✅ ADV-UITEST-001: Web build configuration
- ✅ ADV-UITEST-002: Playwright infrastructure
- ✅ ADV-UITEST-003: Test utilities and page objects
- ✅ ADV-UITEST-004: Comprehensive inventory tests
- ⏳ ADV-UITEST-005: CI/CD integration (next)

