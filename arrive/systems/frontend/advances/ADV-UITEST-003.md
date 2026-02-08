# ADV-UITEST-003: Test Utilities and Page Objects

**Status:** Complete
**Component:** ui-automation
**Depends on:** ADV-UITEST-002 (Playwright infrastructure)
**Blocks:** ADV-UITEST-004 (inventory items tests)

## Objective
Create reusable test utilities, page object models, and helper functions to make UI tests maintainable and reduce code duplication.

## Success Criteria
- [x] Page object models created for common UI patterns
- [x] Helper functions for navigation
- [x] Helper functions for assertions
- [x] Test data fixtures created
- [x] Locale switching helpers
- [ ] GraphQL mock helpers (optional - deferred)
- [ ] Database seeding utilities (deferred - using existing seed data)

## Implementation Steps

### 1. Create Navigation Helpers
**`tests/ui/tests/helpers/navigation.ts`:**
```typescript
import { Page } from '@playwright/test';

export class NavigationHelper {
  constructor(private page: Page) {}

  async goToInventoryItems() {
    await this.page.goto('/');
    await this.page.click('text=Inventory');
    await this.page.click('text=Items');
    await this.page.waitForURL('**/inventory/items');
  }

  async goToInventoryStock() {
    await this.page.goto('/');
    await this.page.click('text=Inventory');
    await this.page.click('text=Stock');
    await this.page.waitForURL('**/inventory/stock');
  }

  async goToWarehouseReceiving() {
    await this.page.goto('/');
    await this.page.click('text=Warehouse');
    await this.page.click('text=Receiving');
    await this.page.waitForURL('**/warehouse/receiving');
  }

  async switchLanguage(locale: 'en' | 'zh') {
    const languageButton = this.page.locator('[data-testid="language-switcher"]');
    await languageButton.click();
    
    if (locale === 'zh') {
      await this.page.click('text=中文');
    } else {
      await this.page.click('text=English');
    }
    
    // Wait for language to change
    await this.page.waitForTimeout(500);
  }
}
```

### 2. Create Page Object for Items List
**`tests/ui/tests/pages/ItemsListPage.ts`:**
```typescript
import { Page, Locator, expect } from '@playwright/test';

export class ItemsListPage {
  readonly page: Page;
  readonly pageTitle: Locator;
  readonly searchInput: Locator;
  readonly searchButton: Locator;
  readonly itemsTable: Locator;
  readonly previousButton: Locator;
  readonly nextButton: Locator;
  readonly loadingIndicator: Locator;
  readonly errorMessage: Locator;
  readonly emptyState: Locator;

  constructor(page: Page) {
    this.page = page;
    this.pageTitle = page.locator('h2').filter({ hasText: /Items|物品/ });
    this.searchInput = page.locator('input[placeholder*="Search"], input[placeholder*="搜索"]');
    this.searchButton = page.locator('button').filter({ hasText: /^Search$|^搜索$/ });
    this.itemsTable = page.locator('table');
    this.previousButton = page.locator('button').filter({ hasText: /Previous|上一页/ });
    this.nextButton = page.locator('button').filter({ hasText: /Next|下一页/ });
    this.loadingIndicator = page.locator('text=/Loading|加载中/');
    this.errorMessage = page.locator('[role="alert"], .error-message');
    this.emptyState = page.locator('text=/No items found|未找到物品/');
  }

  async goto() {
    await this.page.goto('/inventory/items');
    await this.pageTitle.waitFor({ state: 'visible' });
  }

  async search(query: string) {
    await this.searchInput.fill(query);
    await this.searchButton.click();
    // Wait for search to complete
    await this.page.waitForTimeout(500);
  }

  async searchWithEnter(query: string) {
    await this.searchInput.fill(query);
    await this.searchInput.press('Enter');
    await this.page.waitForTimeout(500);
  }

  async clearSearch() {
    await this.searchInput.clear();
    await this.searchButton.click();
    await this.page.waitForTimeout(500);
  }

  async goToNextPage() {
    await this.nextButton.click();
    await this.page.waitForTimeout(500);
  }

  async goToPreviousPage() {
    await this.previousButton.click();
    await this.page.waitForTimeout(500);
  }

  async getTableRowCount(): Promise<number> {
    const rows = await this.itemsTable.locator('tbody tr').count();
    return rows;
  }

  async getTableCellText(row: number, column: number): Promise<string> {
    const cell = this.itemsTable.locator(`tbody tr:nth-child(${row}) td:nth-child(${column})`);
    return await cell.textContent() || '';
  }

  async assertPageLoaded() {
    await expect(this.pageTitle).toBeVisible();
    await expect(this.searchInput).toBeVisible();
  }

  async assertHasResults() {
    await expect(this.itemsTable).toBeVisible();
    const rowCount = await this.getTableRowCount();
    expect(rowCount).toBeGreaterThan(0);
  }

  async assertNoResults() {
    await expect(this.emptyState).toBeVisible();
  }

  async assertNextButtonEnabled() {
    await expect(this.nextButton).toBeEnabled();
  }

  async assertNextButtonDisabled() {
    await expect(this.nextButton).toBeDisabled();
  }

  async assertPreviousButtonEnabled() {
    await expect(this.previousButton).toBeEnabled();
  }

  async assertPreviousButtonDisabled() {
    await expect(this.previousButton).toBeDisabled();
  }
}
```

### 3. Create Test Data Fixtures
**`tests/ui/tests/fixtures/test-data.ts`:**
```typescript
export interface TestItem {
  sku: string;
  name_en: string;
  name_zh: string;
  item_type: string;
  tracking_mode: string;
}

export const TEST_ITEMS: TestItem[] = [
  {
    sku: 'DJI-M300-001',
    name_en: 'DJI Matrice 300 RTK',
    name_zh: 'DJI 经纬 M300 RTK',
    item_type: 'drone',
    tracking_mode: 'serial'
  },
  {
    sku: 'DJI-TB60-001',
    name_en: 'TB60 Intelligent Flight Battery',
    name_zh: 'TB60 智能飞行电池',
    item_type: 'battery',
    tracking_mode: 'serial'
  },
  {
    sku: 'PROP-9455-001',
    name_en: '9455S Quick-Release Propellers',
    name_zh: '9455S 快拆桨',
    item_type: 'part',
    tracking_mode: 'lot'
  }
];

export const SEARCH_QUERIES = {
  validSku: 'DJI-M300',
  validNameEn: 'Battery',
  validNameZh: '电池',
  noResults: 'NONEXISTENT-SKU-999'
};
```

### 4. Create Database Seeding Utility
**`tests/ui/tests/helpers/database.ts`:**
```typescript
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export class DatabaseHelper {
  async seedTestData() {
    // Run seed script
    await execAsync('../../scripts/seed-test-data.sh');
  }

  async clearTestData() {
    // Clear test data (if needed)
    await execAsync('../../scripts/clear-test-data.sh');
  }

  async resetDatabase() {
    // Reset to clean state
    await this.clearTestData();
    await this.seedTestData();
  }
}
```

### 5. Create Assertion Helpers
**`tests/ui/tests/helpers/assertions.ts`:**
```typescript
import { Page, expect } from '@playwright/test';

export async function assertTableContainsText(page: Page, text: string) {
  const table = page.locator('table');
  await expect(table).toContainText(text);
}

export async function assertTableRowCount(page: Page, expectedCount: number) {
  const rows = await page.locator('table tbody tr').count();
  expect(rows).toBe(expectedCount);
}

export async function assertNoLoadingIndicator(page: Page) {
  const loading = page.locator('text=/Loading|加载中/');
  await expect(loading).not.toBeVisible();
}

export async function assertNoErrors(page: Page) {
  const error = page.locator('[role="alert"], .error-message');
  await expect(error).not.toBeVisible();
}
```

## Testing Approach
- Create simple test using page objects
- Verify helpers work correctly
- Verify test data fixtures are accessible
- Test database seeding (if implemented)

## Documentation
- Document page object pattern
- Document how to create new page objects
- Document test data management
- Add examples of using helpers

## Notes
- Page objects encapsulate UI structure
- Helpers reduce code duplication
- Test data fixtures make tests predictable
- Database helpers ensure clean test state

## Implementation Notes

### Files Created

**Helper Functions:**
- `tests/ui/tests/helpers/navigation.ts` - NavigationHelper class for common navigation patterns
- `tests/ui/tests/helpers/assertions.ts` - Reusable assertion functions for common UI patterns

**Page Objects:**
- `tests/ui/tests/pages/BasePage.ts` - Base page object with common functionality
- `tests/ui/tests/pages/ItemsListPage.ts` - Page object for inventory items list

**Test Data:**
- `tests/ui/tests/fixtures/test-data.ts` - Test data fixtures and helper functions

**Tests:**
- `tests/ui/tests/inventory/items.spec.ts` - Comprehensive inventory items tests (8 tests)
- `tests/ui/tests/smoke.spec.ts` - Updated to use new utilities (3 tests)

### Test Results

All 11 tests passing:
- ✅ 8 inventory items tests
- ✅ 3 smoke tests

**Test Coverage:**
- Page display and loading
- Search functionality (button and Enter key)
- Clear search
- No results handling
- Pagination navigation
- Localization (English/Chinese)

### Usage Examples

**Using NavigationHelper:**
```typescript
const nav = new NavigationHelper(page);
await nav.goToInventoryItems();
await nav.switchLanguage('zh');
```

**Using Page Objects:**
```typescript
const itemsPage = new ItemsListPage(page);
await itemsPage.goto();
await itemsPage.search('DJI');
await itemsPage.assertHasResults();
```

**Using Assertion Helpers:**
```typescript
await assertPageLoaded(page);
await assertTableHasMinRows(page, 1);
await assertNoErrors(page);
```

**Using Test Data:**
```typescript
import { SEARCH_QUERIES, TEST_ITEMS } from '../fixtures/test-data';
await itemsPage.search(SEARCH_QUERIES.validPartial);
```

### Benefits Achieved

1. **Maintainability** - UI changes only require updating page objects
2. **Readability** - Tests read like user stories
3. **Reusability** - Common patterns extracted into helpers
4. **Consistency** - All tests use same patterns and utilities
5. **Localization Support** - Helpers work with both English and Chinese

### Next Steps

Ready for **ADV-UITEST-004: Inventory Items UI Tests** which will build on these utilities to create comprehensive test coverage for the inventory module.

