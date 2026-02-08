# ADV-UITEST-005: CI/CD Integration for UI Tests

**Status:** Not Started  
**Component:** ui-automation  
**Depends on:** ADV-UITEST-004 (inventory items tests)  
**Blocks:** None

## Objective
Integrate automated UI tests into CI/CD pipeline to run on every commit and pull request. Ensure tests run reliably in CI environment and provide clear feedback on failures.

## Success Criteria
- [ ] UI tests run automatically on pull requests
- [ ] UI tests run on main branch commits
- [ ] Test results reported in CI
- [ ] Screenshots and videos uploaded as artifacts
- [ ] Test failures block merging
- [ ] Tests run in reasonable time (< 5 minutes)
- [ ] Flaky tests identified and fixed

## Implementation Steps

### 1. Create GitHub Actions Workflow
**`.github/workflows/ui-tests.yml`:**
```yaml
name: UI Tests

on:
  pull_request:
    branches: [main]
    paths:
      - 'crates/ui/**'
      - 'apps/web/**'
      - 'apps/desktop/**'
      - 'tests/ui/**'
      - 'assets/ui/**'
  push:
    branches: [main]
    paths:
      - 'crates/ui/**'
      - 'apps/web/**'
      - 'tests/ui/**'

jobs:
  ui-tests:
    name: Run UI Tests
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: |
            assets/ui/package-lock.json
            tests/ui/package-lock.json

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          cache-on-failure: true

      - name: Install Tailwind dependencies
        working-directory: assets/ui
        run: npm ci

      - name: Build Tailwind CSS
        working-directory: assets/ui
        run: npm run build:css

      - name: Build web app
        run: cargo build --package inventory-web --release

      - name: Install Playwright dependencies
        working-directory: tests/ui
        run: |
          npm ci
          npx playwright install --with-deps chromium

      - name: Run database migrations
        run: |
          cargo run --package inventory-cli -- db migrate

      - name: Seed test data
        run: |
          cargo run --package inventory-cli -- db seed

      - name: Run UI tests
        working-directory: tests/ui
        env:
          CI: true
        run: npm test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: tests/ui/test-results/
          retention-days: 30

      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-screenshots
          path: tests/ui/test-results/**/*.png
          retention-days: 7

      - name: Upload videos
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-videos
          path: tests/ui/test-results/**/*.webm
          retention-days: 7

      - name: Comment PR with test results
        if: github.event_name == 'pull_request' && failure()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const resultsPath = 'tests/ui/test-results/results.json';
            
            if (fs.existsSync(resultsPath)) {
              const results = JSON.parse(fs.readFileSync(resultsPath, 'utf8'));
              const failed = results.suites.reduce((acc, suite) => 
                acc + suite.specs.filter(s => s.tests.some(t => t.status === 'failed')).length, 0
              );
              
              github.rest.issues.createComment({
                issue_number: context.issue.number,
                owner: context.repo.owner,
                repo: context.repo.repo,
                body: `## ❌ UI Tests Failed\n\n${failed} test(s) failed. Check the artifacts for screenshots and videos.`
              });
            }
```

### 2. Create Pre-commit Hook
**`.githooks/pre-push`:**
```bash
#!/usr/bin/env bash
set -e

echo "Running UI tests before push..."

# Check if UI files changed
if git diff --name-only origin/main...HEAD | grep -qE '^(crates/ui|apps/web|tests/ui)'; then
  echo "UI changes detected, running tests..."
  ./scripts/test-ui.sh
else
  echo "No UI changes detected, skipping tests"
fi
```

Make executable: `chmod +x .githooks/pre-push`

### 3. Create Test Performance Monitoring
**`tests/ui/tests/performance.spec.ts`:**
```typescript
import { test, expect } from '@playwright/test';
import { ItemsListPage } from './pages/ItemsListPage';

test.describe('Performance Tests', () => {
  test('items page should load within 3 seconds', async ({ page }) => {
    const startTime = Date.now();
    
    const itemsPage = new ItemsListPage(page);
    await itemsPage.goto();
    await itemsPage.assertPageLoaded();
    
    const loadTime = Date.now() - startTime;
    expect(loadTime).toBeLessThan(3000);
  });

  test('search should complete within 1 second', async ({ page }) => {
    const itemsPage = new ItemsListPage(page);
    await itemsPage.goto();
    
    const startTime = Date.now();
    await itemsPage.search('DJI');
    const searchTime = Date.now() - startTime;
    
    expect(searchTime).toBeLessThan(1000);
  });
});
```

### 4. Create Flaky Test Detection
**`scripts/detect-flaky-tests.sh`:**
```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../tests/ui"

echo "Running tests 10 times to detect flaky tests..."

FAILURES=0
for i in {1..10}; do
  echo "Run $i/10..."
  if ! npm test > /dev/null 2>&1; then
    FAILURES=$((FAILURES + 1))
  fi
done

echo ""
echo "Results: $FAILURES failures out of 10 runs"

if [ $FAILURES -gt 0 ] && [ $FAILURES -lt 10 ]; then
  echo "⚠️  WARNING: Flaky tests detected!"
  exit 1
elif [ $FAILURES -eq 10 ]; then
  echo "❌ All runs failed - tests are broken"
  exit 1
else
  echo "✅ All runs passed - tests are stable"
  exit 0
fi
```

Make executable: `chmod +x scripts/detect-flaky-tests.sh`

### 5. Create Test Coverage Report
**`scripts/ui-test-coverage.sh`:**
```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../tests/ui"

echo "Generating UI test coverage report..."

# Run tests with coverage
npm test -- --reporter=json > test-results/coverage.json

# Parse results
node -e "
const fs = require('fs');
const results = JSON.parse(fs.readFileSync('test-results/coverage.json', 'utf8'));

const total = results.suites.reduce((acc, suite) => 
  acc + suite.specs.length, 0
);
const passed = results.suites.reduce((acc, suite) => 
  acc + suite.specs.filter(s => s.tests.every(t => t.status === 'passed')).length, 0
);

console.log('');
console.log('UI Test Coverage Report');
console.log('======================');
console.log('Total tests:', total);
console.log('Passed:', passed);
console.log('Failed:', total - passed);
console.log('Coverage:', ((passed / total) * 100).toFixed(2) + '%');
"
```

Make executable: `chmod +x scripts/ui-test-coverage.sh`

## Testing Approach
- Run workflow locally with `act` (GitHub Actions local runner)
- Test on actual PR
- Verify artifacts uploaded correctly
- Verify test failures block merging
- Check performance of CI run

## Documentation
- Document CI/CD setup
- Document how to view test results in CI
- Document how to debug CI failures
- Add badge to README showing test status

## Notes
- Tests run only when UI files change
- Screenshots and videos uploaded on failure
- Test results retained for 30 days
- Pre-push hook prevents pushing broken code
- Flaky test detection helps maintain reliability

