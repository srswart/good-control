import { Page, Locator } from '@playwright/test';
import { BasePage } from './BasePage';

/**
 * Page object for the Stock View (Inventory Positions)
 */
export class StockViewPage extends BasePage {
  // Filter controls
  readonly skuFilterInput: Locator;
  readonly statusFilterSelect: Locator;
  readonly searchButton: Locator;

  // Table elements
  readonly table: Locator;
  readonly tableHeaders: Locator;
  readonly tableRows: Locator;

  // State indicators
  readonly loadingIndicator: Locator;
  readonly errorMessage: Locator;
  readonly emptyState: Locator;
  readonly retryButton: Locator;

  // Position count
  readonly positionCount: Locator;

  constructor(page: Page) {
    super(page);
    
    // Filter controls
    this.skuFilterInput = page.locator('input[placeholder*="Filter by SKU"]');
    this.statusFilterSelect = page.locator('select').filter({ hasText: 'All Statuses' });
    this.searchButton = page.locator('button:has-text("Search")');

    // Table elements
    this.table = page.locator('table');
    this.tableHeaders = page.locator('thead th');
    this.tableRows = page.locator('tbody tr');

    // State indicators
    this.loadingIndicator = page.locator('.animate-spin');
    this.errorMessage = page.locator('.bg-red-50');
    this.emptyState = page.locator('text=No stock positions found');
    this.retryButton = page.locator('button:has-text("Retry")');

    // Position count
    this.positionCount = page.locator('text=/\\(\\d+ positions\\)/');
  }

  /**
   * Navigate to the stock view
   */
  async goto() {
    // Navigate to inventory page
    await this.page.goto('/inventory');
    await this.page.waitForLoadState('networkidle');

    // Click the Stock tab
    const stockTab = this.page.locator('button:has-text("Stock")');
    await stockTab.click();

    // Wait for the stock view to load
    await this.waitForPageLoad();
  }

  /**
   * Wait for the page to load completely
   */
  async waitForPageLoad() {
    // Wait for either the table or empty state to appear
    await Promise.race([
      this.table.waitFor({ state: 'visible', timeout: 10000 }),
      this.emptyState.waitFor({ state: 'visible', timeout: 10000 })
    ]).catch(() => {
      // If neither appears, that's okay - might be in error state
    });

    // Make sure loading is complete
    await this.loadingIndicator.waitFor({ state: 'hidden', timeout: 10000 }).catch(() => {
      // Loading indicator might not be visible
    });
  }

  /**
   * Filter by SKU
   */
  async filterBySku(sku: string) {
    await this.skuFilterInput.fill(sku);
  }

  /**
   * Filter by status
   */
  async filterByStatus(status: 'all' | 'available' | 'reserved' | 'quarantine' | 'damaged') {
    const statusMap = {
      'all': 'All Statuses',
      'available': 'Available',
      'reserved': 'Reserved',
      'quarantine': 'Quarantine',
      'damaged': 'Damaged'
    };
    await this.statusFilterSelect.selectOption({ label: statusMap[status] });
  }

  /**
   * Click the search button
   */
  async search() {
    await this.searchButton.click();
    await this.waitForLoading();
  }

  /**
   * Clear SKU filter
   */
  async clearSkuFilter() {
    await this.skuFilterInput.clear();
  }

  /**
   * Get the number of positions displayed
   */
  async getPositionCount(): Promise<number> {
    const rows = await this.tableRows.count();
    return rows;
  }

  /**
   * Get all SKUs from the table
   */
  async getAllSkus(): Promise<string[]> {
    const skus: string[] = [];
    const count = await this.tableRows.count();
    
    for (let i = 0; i < count; i++) {
      const sku = await this.tableRows.nth(i).locator('td').nth(0).textContent();
      if (sku) skus.push(sku.trim());
    }
    
    return skus;
  }

  /**
   * Get all statuses from the table
   */
  async getAllStatuses(): Promise<string[]> {
    const statuses: string[] = [];
    const count = await this.tableRows.count();
    
    for (let i = 0; i < count; i++) {
      const status = await this.tableRows.nth(i).locator('td').nth(2).textContent();
      if (status) statuses.push(status.trim());
    }
    
    return statuses;
  }

  /**
   * Check if table has specific SKU
   */
  async hasSkuInTable(sku: string): Promise<boolean> {
    const skus = await this.getAllSkus();
    return skus.some(s => s.includes(sku));
  }

  /**
   * Check if table has specific status
   */
  async hasStatusInTable(status: string): Promise<boolean> {
    const statuses = await this.getAllStatuses();
    return statuses.some(s => s.toLowerCase() === status.toLowerCase());
  }

  /**
   * Check if all rows have specific status
   */
  async allRowsHaveStatus(status: string): Promise<boolean> {
    const statuses = await this.getAllStatuses();
    return statuses.every(s => s.toLowerCase() === status.toLowerCase());
  }

  /**
   * Wait for loading to complete
   */
  async waitForLoading() {
    // Give a small delay for the loading state to potentially start
    await this.page.waitForTimeout(100);

    // Wait for loading indicator to disappear (if it's visible)
    await this.loadingIndicator.waitFor({ state: 'hidden', timeout: 10000 }).catch(() => {
      // Loading indicator might not be visible at all
    });

    // Wait for network to be idle
    await this.page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => {
      // Network might already be idle
    });
  }
}

