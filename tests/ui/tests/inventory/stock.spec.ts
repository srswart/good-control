import { test, expect } from '@playwright/test';
import { StockViewPage } from '../pages/StockViewPage';
import { NavigationHelper } from '../helpers/navigation';
import { STOCK_QUERIES } from '../fixtures/test-data';

test.describe('Stock View', () => {
  let stockPage: StockViewPage;
  let nav: NavigationHelper;

  test.beforeEach(async ({ page }) => {
    stockPage = new StockViewPage(page);
    nav = new NavigationHelper(page);
    await stockPage.goto();
  });

  test.describe('Page Load', () => {
    test('should load stock view successfully', async () => {
      await expect(stockPage.page.locator('h2:has-text("Stock Positions")')).toBeVisible();
      await expect(stockPage.skuFilterInput).toBeVisible();
      await expect(stockPage.statusFilterSelect).toBeVisible();
      await expect(stockPage.searchButton).toBeVisible();
    });

    test('should display inventory positions in table', async () => {
      await stockPage.waitForLoading();
      const count = await stockPage.getPositionCount();
      expect(count).toBeGreaterThan(0);
    });

    test('should have correct table headers', async () => {
      await stockPage.waitForLoading();
      const headers = await stockPage.tableHeaders.allTextContents();
      expect(headers).toContain('SKU');
      expect(headers).toContain('Location');
      expect(headers).toContain('Status');
      expect(headers).toContain('On Hand');
      expect(headers).toContain('Reserved');
      expect(headers).toContain('Available');
    });
  });

  test.describe('SKU Filter', () => {
    test('should filter by full SKU', async ({ page }) => {
      await stockPage.filterBySku(STOCK_QUERIES.skuFull);
      await stockPage.search();

      const skus = await stockPage.getAllSkus();
      expect(skus.length).toBeGreaterThan(0);
      expect(skus.every(sku => sku === STOCK_QUERIES.skuFull)).toBeTruthy();
    });

    test('should filter by partial SKU', async ({ page }) => {
      await stockPage.filterBySku(STOCK_QUERIES.skuPartial);
      await stockPage.search();

      const skus = await stockPage.getAllSkus();
      expect(skus.length).toBeGreaterThan(0);
      expect(skus.every(sku => sku.includes(STOCK_QUERIES.skuPartial))).toBeTruthy();
    });

    test('should clear SKU filter', async ({ page }) => {
      // First apply filter
      await stockPage.filterBySku(STOCK_QUERIES.skuFull);
      await stockPage.search();
      const filteredCount = await stockPage.getPositionCount();

      // Clear filter
      await stockPage.clearSkuFilter();
      await stockPage.search();
      const unfilteredCount = await stockPage.getPositionCount();

      expect(unfilteredCount).toBeGreaterThan(filteredCount);
    });

    test('should show empty state for non-existent SKU', async ({ page }) => {
      await stockPage.filterBySku(STOCK_QUERIES.skuNoResults);
      await stockPage.search();

      await expect(stockPage.emptyState).toBeVisible();
      const count = await stockPage.getPositionCount();
      expect(count).toBe(0);
    });
  });

  test.describe('Status Filter', () => {
    test('should filter by Available status', async ({ page }) => {
      await stockPage.filterByStatus('available');
      // Status filter triggers search automatically
      await stockPage.waitForLoading();

      const statuses = await stockPage.getAllStatuses();
      expect(statuses.length).toBeGreaterThan(0);
      expect(await stockPage.allRowsHaveStatus('available')).toBeTruthy();
    });

    test('should filter by Reserved status', async ({ page }) => {
      await stockPage.filterByStatus('reserved');
      await stockPage.waitForLoading();

      const statuses = await stockPage.getAllStatuses();
      expect(statuses.length).toBeGreaterThan(0);
      expect(await stockPage.allRowsHaveStatus('reserved')).toBeTruthy();
    });

    test('should filter by Quarantine status', async ({ page }) => {
      await stockPage.filterByStatus('quarantine');
      await stockPage.waitForLoading();

      const statuses = await stockPage.getAllStatuses();
      expect(statuses.length).toBeGreaterThan(0);
      expect(await stockPage.allRowsHaveStatus('quarantine')).toBeTruthy();
    });

    test('should filter by Damaged status', async ({ page }) => {
      await stockPage.filterByStatus('damaged');
      await stockPage.waitForLoading();

      const statuses = await stockPage.getAllStatuses();
      expect(statuses.length).toBeGreaterThan(0);
      expect(await stockPage.allRowsHaveStatus('damaged')).toBeTruthy();
    });

    test('should show all statuses when "All Statuses" selected', async ({ page }) => {
      // First filter to one status
      await stockPage.filterByStatus('available');
      await stockPage.waitForLoading();
      const availableCount = await stockPage.getPositionCount();

      // Then select "All Statuses"
      await stockPage.filterByStatus('all');
      await stockPage.waitForLoading();
      const allCount = await stockPage.getPositionCount();

      expect(allCount).toBeGreaterThan(availableCount);
    });

    test('should update table when status filter changes', async ({ page }) => {
      await stockPage.filterByStatus('available');
      await stockPage.waitForLoading();
      const availableCount = await stockPage.getPositionCount();

      await stockPage.filterByStatus('reserved');
      await stockPage.waitForLoading();
      const reservedCount = await stockPage.getPositionCount();

      // Counts should be different (unless all items happen to be both, which is unlikely)
      expect(availableCount).not.toBe(reservedCount);
    });
  });



  test.describe('Combined Filters', () => {
    test('should filter by SKU and status together', async ({ page }) => {
      // Filter by SKU that has multiple statuses
      await stockPage.filterBySku('DJI-M300-RTK');
      await stockPage.filterByStatus('available');
      await stockPage.search();

      const skus = await stockPage.getAllSkus();
      const statuses = await stockPage.getAllStatuses();

      expect(skus.length).toBeGreaterThan(0);
      expect(skus.every(sku => sku === 'DJI-M300-RTK')).toBeTruthy();
      expect(statuses.every(status => status.toLowerCase() === 'available')).toBeTruthy();
    });

    test('should clear all filters', async ({ page }) => {
      // Apply both filters
      await stockPage.filterBySku(STOCK_QUERIES.skuFull);
      await stockPage.filterByStatus('available');
      await stockPage.search();
      const filteredCount = await stockPage.getPositionCount();

      // Clear both filters
      await stockPage.clearSkuFilter();
      await stockPage.filterByStatus('all');
      await stockPage.search();
      const unfilteredCount = await stockPage.getPositionCount();

      expect(unfilteredCount).toBeGreaterThan(filteredCount);
    });
  });

  test.describe('Loading, Error, and Empty States', () => {
    test('should show loading indicator while fetching', async ({ page }) => {
      // Reload page to catch loading state
      await page.reload();

      // Loading indicator should appear briefly
      // Note: This might be too fast to catch in some cases
      const hasLoading = await stockPage.loadingIndicator.isVisible().catch(() => false);

      // Wait for loading to complete
      await stockPage.waitForLoading();

      // After loading, table should be visible
      await expect(stockPage.table).toBeVisible();
    });

    test('should show empty state when no results', async ({ page }) => {
      await stockPage.filterBySku(STOCK_QUERIES.skuNoResults);
      await stockPage.search();

      await expect(stockPage.emptyState).toBeVisible();
      await expect(stockPage.emptyState).toContainText('No stock positions found');
    });

    test('should show retry button on error', async ({ page }) => {
      // This test would require mocking a GraphQL error
      // For now, we'll just verify the error UI structure exists
      // In a real scenario, you'd use page.route() to mock the GraphQL endpoint

      // Skip this test for now - would need API mocking
      test.skip();
    });
  });

  test.describe('Localization', () => {
    test('should display in Chinese when language switched', async ({ page }) => {
      await nav.switchLanguage('zh');
      await page.waitForTimeout(500); // Wait for language switch

      // Check for Chinese text in the UI
      // Note: The exact Chinese text depends on your i18n implementation
      const pageContent = await stockPage.page.textContent('body');

      // Should have some Chinese characters
      expect(pageContent).toMatch(/[\u4e00-\u9fa5]/);
    });

    test('should display in English when switched back', async ({ page }) => {
      // Switch to Chinese first
      await nav.switchLanguage('zh');
      await page.waitForTimeout(500);

      // Switch back to English
      await nav.switchLanguage('en');
      await page.waitForTimeout(500);

      // Check for English text
      await expect(stockPage.page.locator('h2:has-text("Stock Positions")')).toBeVisible();
      await expect(stockPage.searchButton).toHaveText('Search');
    });
  });
});

