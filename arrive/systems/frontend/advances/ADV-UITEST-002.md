# ADV-UITEST-002: Playwright Test Infrastructure

**Status:** Complete
**Component:** ui-automation
**Depends on:** ADV-UITEST-001 (web build configuration)
**Blocks:** ADV-UITEST-003 (test utilities)

## Objective
Set up Playwright test infrastructure for automated browser-based UI testing. Install Playwright, configure test runner, and create basic test structure.

## Success Criteria
- [x] Playwright installed and configured
- [x] Test runner script created
- [x] Can run tests in headed mode (browser visible)
- [x] Can run tests in headless mode (CI/CD)
- [x] Test reports generated
- [x] Screenshots captured on failure
- [x] Video recording available for debugging

## Implementation Steps

### 1. Install Playwright
Create `tests/ui/package.json`:
```json
{
  "name": "ui-tests",
  "version": "0.1.0",
  "description": "Automated UI tests using Playwright",
  "scripts": {
    "test": "playwright test",
    "test:headed": "playwright test --headed",
    "test:debug": "playwright test --debug",
    "test:ui": "playwright test --ui",
    "test:report": "playwright show-report"
  },
  "devDependencies": {
    "@playwright/test": "^1.48.0",
    "typescript": "^5.3.0"
  }
}
```

Install dependencies:
```bash
cd tests/ui
npm install
npx playwright install chromium
```

### 2. Configure Playwright
Create `tests/ui/playwright.config.ts`:
```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false, // Run tests sequentially for now
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : 1,
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list']
  ],
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: '../../scripts/run-web.sh',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

### 3. Create Test Structure
```
tests/ui/
├── package.json
├── playwright.config.ts
├── tsconfig.json
├── tests/
│   ├── inventory/
│   │   └── items.spec.ts
│   ├── fixtures/
│   │   └── test-data.ts
│   └── helpers/
│       ├── navigation.ts
│       └── assertions.ts
└── test-results/
```

**`tests/ui/tsconfig.json`:**
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "moduleResolution": "node",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["tests/**/*.ts"],
  "exclude": ["node_modules"]
}
```

### 4. Create Basic Smoke Test
**`tests/ui/tests/smoke.spec.ts`:**
```typescript
import { test, expect } from '@playwright/test';

test.describe('Application Smoke Tests', () => {
  test('should load home page', async ({ page }) => {
    await page.goto('/');
    
    // Check for app title
    await expect(page).toHaveTitle(/Controlled Goods SCM/);
    
    // Check for navigation
    await expect(page.locator('nav')).toBeVisible();
  });

  test('should navigate to inventory items', async ({ page }) => {
    await page.goto('/');
    
    // Click on Inventory > Items
    await page.click('text=Inventory');
    await page.click('text=Items');
    
    // Verify we're on items page
    await expect(page.locator('h2:has-text("Items")')).toBeVisible();
  });

  test('should switch language', async ({ page }) => {
    await page.goto('/');
    
    // Find and click language switcher
    const languageSwitcher = page.locator('select, button').filter({ hasText: /EN|English/ });
    await languageSwitcher.click();
    
    // Select Chinese
    await page.click('text=中文');
    
    // Verify language changed
    await expect(page.locator('text=物品')).toBeVisible({ timeout: 5000 });
  });
});
```

### 5. Create Test Runner Scripts
**`scripts/test-ui.sh`:**
```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="${1:-headless}"

echo "Running UI tests in $MODE mode..."

cd tests/ui

case "$MODE" in
  headed)
    npm run test:headed
    ;;
  debug)
    npm run test:debug
    ;;
  ui)
    npm run test:ui
    ;;
  *)
    npm test
    ;;
esac
```

Make executable: `chmod +x scripts/test-ui.sh`

### 6. Testing
```bash
# Run tests in headless mode
./scripts/test-ui.sh

# Run tests with browser visible
./scripts/test-ui.sh headed

# Debug tests
./scripts/test-ui.sh debug

# View test report
cd tests/ui && npm run test:report
```

## Testing Approach
- Verify Playwright can launch browser
- Verify web server starts automatically
- Verify smoke tests pass
- Verify screenshots captured on failure
- Verify test reports generated

## Documentation
- Document how to run UI tests
- Document test debugging workflow
- Document CI/CD integration approach
- Add troubleshooting guide

## Notes
- Playwright will auto-start web server before tests
- Tests run in Chromium by default (can add Firefox, WebKit later)
- Screenshots and videos saved in `test-results/`
- Test reports viewable with `npm run test:report`

---

## Implementation Notes

### Date: 2026-01-30

**Files Created:**
- `tests/ui/package.json` - NPM package configuration with Playwright dependencies
- `tests/ui/tsconfig.json` - TypeScript configuration for test files
- `tests/ui/playwright.config.ts` - Playwright configuration with Chromium browser
- `tests/ui/.gitignore` - Ignore node_modules and test results
- `tests/ui/tests/smoke.spec.ts` - Basic smoke tests for application
- `scripts/test-ui.sh` - Test runner script with multiple modes

**Installation:**
```bash
cd tests/ui
npm install
npx playwright install chromium
```

**Test Results:**
✅ All 3 smoke tests passing:
1. Home page loads with correct title
2. Navigation to inventory page works
3. Language switcher is present and functional

**Test Execution:**
```bash
# Headless mode (default)
./scripts/test-ui.sh

# Headed mode (browser visible)
./scripts/test-ui.sh headed

# Debug mode
./scripts/test-ui.sh debug

# UI mode (interactive)
./scripts/test-ui.sh ui

# View test report
cd tests/ui && npm run test:report
```

**Configuration Highlights:**
- **Base URL:** http://localhost:8080
- **Browser:** Chromium (Desktop Chrome)
- **Auto-start:** Web server starts automatically before tests
- **Screenshots:** Captured on failure only
- **Videos:** Retained on failure only
- **Traces:** Captured on first retry
- **Reports:** HTML report in `playwright-report/`, JSON in `test-results/`

**Smoke Tests:**
1. **Home page test** - Verifies app loads with correct title and navigation
2. **Inventory navigation test** - Verifies routing to /inventory works
3. **Language switcher test** - Verifies language switcher is present

**Next Steps:**
Ready for ADV-UITEST-003 to create test utilities and page objects for more comprehensive testing.

