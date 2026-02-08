import { Page, Locator, expect } from '@playwright/test';

/**
 * Page object for the Inventory Items List page
 */
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
    // Support both English and Chinese text
    this.pageTitle = page.locator('h2').filter({ hasText: /Items|物品/ });
    this.searchInput = page.locator('input[placeholder*="Search"], input[placeholder*="搜索"]');
    this.searchButton = page.locator('button').filter({ hasText: /^Search$|^搜索$/ });
    this.itemsTable = page.locator('table');
    this.previousButton = page.locator('button').filter({ hasText: /Previous|上一页/ });
    this.nextButton = page.locator('button').filter({ hasText: /Next|下一页/ });
    this.loadingIndicator = page.locator('text=/Loading|加载中/');
    this.errorMessage = page.locator('[role="alert"], .error-message');
    this.emptyState = page.locator('text=/No items found|未找到物品|No results|没有结果/');
  }

  /**
   * Navigate to the items list page
   */
  async goto() {
    await this.page.goto('/inventory');
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Search for items using the search button
   */
  async search(query: string) {
    await this.searchInput.fill(query);
    await this.searchButton.click();
    // Wait for search to complete
    await this.page.waitForTimeout(500);
  }

  /**
   * Search for items using the Enter key
   */
  async searchWithEnter(query: string) {
    await this.searchInput.fill(query);
    await this.searchInput.press('Enter');
    await this.page.waitForTimeout(500);
  }

  /**
   * Clear the search input and search again
   */
  async clearSearch() {
    await this.searchInput.clear();
    await this.searchButton.click();
    await this.page.waitForTimeout(500);
  }

  /**
   * Navigate to the next page of results
   */
  async goToNextPage() {
    await this.nextButton.click();
    await this.page.waitForTimeout(500);
  }

  /**
   * Navigate to the previous page of results
   */
  async goToPreviousPage() {
    await this.previousButton.click();
    await this.page.waitForTimeout(500);
  }

  /**
   * Get the number of rows in the table
   */
  async getTableRowCount(): Promise<number> {
    const rows = await this.itemsTable.locator('tbody tr').count();
    return rows;
  }

  /**
   * Get the text content of a specific table cell
   */
  async getTableCellText(row: number, column: number): Promise<string> {
    const cell = this.itemsTable.locator(`tbody tr:nth-child(${row}) td:nth-child(${column})`);
    return await cell.textContent() || '';
  }

  /**
   * Get all text from a specific column
   */
  async getColumnText(column: number): Promise<string[]> {
    const cells = this.itemsTable.locator(`tbody tr td:nth-child(${column})`);
    const count = await cells.count();
    const texts: string[] = [];
    
    for (let i = 0; i < count; i++) {
      const text = await cells.nth(i).textContent();
      if (text) {
        texts.push(text.trim());
      }
    }
    
    return texts;
  }

  /**
   * Assert that the page has loaded successfully
   */
  async assertPageLoaded() {
    await expect(this.pageTitle).toBeVisible();
    await expect(this.searchInput).toBeVisible();
  }

  /**
   * Assert that the table has results
   */
  async assertHasResults() {
    await expect(this.itemsTable).toBeVisible();
    const rowCount = await this.getTableRowCount();
    expect(rowCount).toBeGreaterThan(0);
  }

  /**
   * Assert that no results are shown
   */
  async assertNoResults() {
    await expect(this.emptyState).toBeVisible();
  }

  /**
   * Assert that the next button is enabled
   */
  async assertNextButtonEnabled() {
    await expect(this.nextButton).toBeEnabled();
  }

  /**
   * Assert that the next button is disabled
   */
  async assertNextButtonDisabled() {
    await expect(this.nextButton).toBeDisabled();
  }

  /**
   * Assert that the previous button is enabled
   */
  async assertPreviousButtonEnabled() {
    await expect(this.previousButton).toBeEnabled();
  }

  /**
   * Assert that the previous button is disabled
   */
  async assertPreviousButtonDisabled() {
    await expect(this.previousButton).toBeDisabled();
  }
}

