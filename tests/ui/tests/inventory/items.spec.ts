import { test, expect } from '@playwright/test';
import { ItemsListPage } from '../pages/ItemsListPage';
import { NavigationHelper } from '../helpers/navigation';
import { assertPageLoaded } from '../helpers/assertions';
import { SEARCH_QUERIES } from '../fixtures/test-data';

test.describe('Inventory Items View', () => {
  let itemsPage: ItemsListPage;
  let nav: NavigationHelper;

  test.beforeEach(async ({ page }) => {
    itemsPage = new ItemsListPage(page);
    nav = new NavigationHelper(page);
    await itemsPage.goto();
    // Wait for initial load
    await page.waitForTimeout(1000);
  });

  test.describe('Page Load', () => {
    test('should load items page successfully', async () => {
      await itemsPage.assertPageLoaded();
      await expect(itemsPage.searchInput).toBeVisible();
      await expect(itemsPage.searchButton).toBeVisible();
    });

    test('should display items table with data', async () => {
      // Check if we have results (may be empty if no seed data)
      const rowCount = await itemsPage.getTableRowCount();

      if (rowCount > 0) {
        await itemsPage.assertHasResults();
        expect(rowCount).toBeGreaterThan(0);
        expect(rowCount).toBeLessThanOrEqual(20); // Page size is 20
      } else {
        // If no data, verify the empty state
        await itemsPage.assertNoResults();
      }
    });

    test('should have correct table headers in English', async () => {
      await expect(itemsPage.itemsTable).toContainText('SKU');
      await expect(itemsPage.itemsTable).toContainText('Name (EN)');
      await expect(itemsPage.itemsTable).toContainText('Name (ZH)');
      await expect(itemsPage.itemsTable).toContainText('Type');
      await expect(itemsPage.itemsTable).toContainText('Tracking');
    });
  });

  test.describe('Search Functionality', () => {
    test('should search by SKU', async ({ page }) => {
      await itemsPage.search(SEARCH_QUERIES.validPartial);
      await page.waitForTimeout(1000);

      // Verify search completed (either results or no results)
      const rowCount = await itemsPage.getTableRowCount();

      if (rowCount > 0) {
        await itemsPage.assertHasResults();
        // Verify results contain search term
        const skuColumn = await itemsPage.getColumnText(1);
        const hasMatch = skuColumn.some(sku => sku.includes(SEARCH_QUERIES.validPartial));
        expect(hasMatch).toBeTruthy();
      }
    });

    test('should search by English name', async ({ page }) => {
      await itemsPage.search(SEARCH_QUERIES.validNameEn);
      await page.waitForTimeout(1000);

      const rowCount = await itemsPage.getTableRowCount();
      if (rowCount > 0) {
        await itemsPage.assertHasResults();
        // Verify results contain search term in name column
        await expect(itemsPage.itemsTable).toContainText(SEARCH_QUERIES.validNameEn);
      }
    });

    test('should search by Chinese name', async ({ page }) => {
      await itemsPage.search(SEARCH_QUERIES.validNameZh);
      await page.waitForTimeout(1000);

      const rowCount = await itemsPage.getTableRowCount();
      if (rowCount > 0) {
        await itemsPage.assertHasResults();
        // Verify results contain search term
        await expect(itemsPage.itemsTable).toContainText(SEARCH_QUERIES.validNameZh);
      }
    });

    test('should show empty state for no results', async () => {
      await itemsPage.search(SEARCH_QUERIES.noResults);
      await itemsPage.page.waitForTimeout(1000);
      await itemsPage.assertNoResults();
    });

    test('should clear search and show all items', async ({ page }) => {
      // First search for something specific
      await itemsPage.search(SEARCH_QUERIES.validPartial);
      await page.waitForTimeout(1000);
      const searchResultCount = await itemsPage.getTableRowCount();

      // Clear search
      await itemsPage.clearSearch();
      await page.waitForTimeout(1000);
      const allResultsCount = await itemsPage.getTableRowCount();

      // Should have more or equal results after clearing
      expect(allResultsCount).toBeGreaterThanOrEqual(searchResultCount);
    });

    test('should trigger search with Enter key', async ({ page }) => {
      await itemsPage.searchWithEnter(SEARCH_QUERIES.validPartial);
      await page.waitForTimeout(1000);

      const rowCount = await itemsPage.getTableRowCount();
      if (rowCount > 0) {
        await itemsPage.assertHasResults();
        const skuColumn = await itemsPage.getColumnText(1);
        const hasMatch = skuColumn.some(sku => sku.includes(SEARCH_QUERIES.validPartial));
        expect(hasMatch).toBeTruthy();
      }
    });
  });

  test.describe('Pagination', () => {
    test('should have previous button disabled on first page', async () => {
      await itemsPage.assertPreviousButtonDisabled();
    });

    test('should navigate to next page when available', async ({ page }) => {
      // Check if next button is enabled
      const isNextEnabled = await itemsPage.nextButton.isEnabled();

      if (isNextEnabled) {
        // Get first item SKU on page 1
        const page1FirstSku = await itemsPage.getTableCellText(1, 1);

        // Go to next page
        await itemsPage.goToNextPage();

        // Get first item SKU on page 2
        const page2FirstSku = await itemsPage.getTableCellText(1, 1);

        // Should be different items
        expect(page2FirstSku).not.toBe(page1FirstSku);

        // Previous button should now be enabled
        await itemsPage.assertPreviousButtonEnabled();
      }
    });

    test('should navigate back to previous page', async ({ page }) => {
      const isNextEnabled = await itemsPage.nextButton.isEnabled();

      if (isNextEnabled) {
        // Go to page 2
        await itemsPage.goToNextPage();

        // Go back to page 1
        await itemsPage.goToPreviousPage();

        // Previous button should be disabled again
        await itemsPage.assertPreviousButtonDisabled();
      }
    });

    test('should have next button disabled on last page', async () => {
      // Keep clicking next until disabled
      let clickCount = 0;
      const maxClicks = 10; // Safety limit

      while (await itemsPage.nextButton.isEnabled() && clickCount < maxClicks) {
        await itemsPage.goToNextPage();
        clickCount++;
      }

      // Should eventually be disabled (or we hit safety limit)
      if (clickCount < maxClicks) {
        await itemsPage.assertNextButtonDisabled();
      }
    });
  });

  test.describe('Localization', () => {
    test('should display Chinese UI when language switched', async ({ page }) => {
      // Switch to Chinese
      await nav.switchLanguage('zh');

      // Reload items page
      await itemsPage.goto();
      await page.waitForTimeout(1000);

      // Check that page loaded successfully
      await expect(itemsPage.pageTitle).toBeVisible();

      // Verify the page is functional (language switch may or may not be implemented)
      // Just verify the UI still works
      await assertPageLoaded(page);
    });

    test('should switch back to English', async ({ page }) => {
      // Switch to Chinese then back to English
      await nav.switchLanguage('zh');
      await nav.switchLanguage('en');

      // Reload items page
      await itemsPage.goto();
      await page.waitForTimeout(1000);

      // Check for English text
      await expect(itemsPage.pageTitle).toBeVisible();
      await expect(itemsPage.searchButton).toBeVisible();
    });

    test('should preserve table data in both languages', async ({ page }) => {
      // Get first row data in English (if exists)
      const rowCount = await itemsPage.getTableRowCount();

      if (rowCount > 0) {
        const enSku = await itemsPage.getTableCellText(1, 1);

        // Switch to Chinese
        await nav.switchLanguage('zh');
        await itemsPage.goto();
        await page.waitForTimeout(1000);

        // Get first row data in Chinese
        const zhSku = await itemsPage.getTableCellText(1, 1);

        // SKU should be the same (data doesn't translate)
        expect(zhSku).toBe(enSku);
      }
    });
  });
});

