import { Page } from '@playwright/test';

/**
 * Navigation helper for common navigation patterns across the application.
 */
export class NavigationHelper {
  constructor(private page: Page) {}

  /**
   * Navigate to the home page
   */
  async goToHome() {
    await this.page.goto('/');
  }

  /**
   * Navigate to the inventory items page
   */
  async goToInventoryItems() {
    await this.page.goto('/inventory');
    // Wait for page to load
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Navigate to the inventory stock view
   */
  async goToInventoryStock() {
    await this.page.goto('/inventory');
    // Click on Stock tab if it exists
    const stockTab = this.page.locator('button:has-text("Stock"), a:has-text("Stock")');
    const exists = await stockTab.count() > 0;
    if (exists) {
      await stockTab.first().click();
    }
  }

  /**
   * Navigate to the warehouse receiving page
   */
  async goToWarehouseReceiving() {
    await this.page.goto('/warehouse');
    // Click on Receiving tab if it exists
    const receivingTab = this.page.locator('button:has-text("Receiving"), a:has-text("Receiving")');
    const exists = await receivingTab.count() > 0;
    if (exists) {
      await receivingTab.first().click();
    }
  }

  /**
   * Navigate to the warehouse picking page
   */
  async goToWarehousePicking() {
    await this.page.goto('/warehouse');
    // Click on Pick tab if it exists
    const pickTab = this.page.locator('button:has-text("Pick"), a:has-text("Pick")');
    const exists = await pickTab.count() > 0;
    if (exists) {
      await pickTab.first().click();
    }
  }

  /**
   * Navigate to the warehouse packing page
   */
  async goToWarehousePacking() {
    await this.page.goto('/warehouse');
    // Click on Pack tab if it exists
    const packTab = this.page.locator('button:has-text("Pack"), a:has-text("Pack")');
    const exists = await packTab.count() > 0;
    if (exists) {
      await packTab.first().click();
    }
  }

  /**
   * Navigate to the warehouse shipping page
   */
  async goToWarehouseShipping() {
    await this.page.goto('/warehouse');
    // Click on Ship tab if it exists
    const shipTab = this.page.locator('button:has-text("Ship"), a:has-text("Ship")');
    const exists = await shipTab.count() > 0;
    if (exists) {
      await shipTab.first().click();
    }
  }

  /**
   * Switch the application language
   */
  async switchLanguage(locale: 'en' | 'zh') {
    // Click the language switcher button (🌐 Language)
    const languageButton = this.page.locator('button:has-text("🌐 Language")');
    await languageButton.click();

    // Wait for modal to open
    await this.page.waitForTimeout(300);

    // Click the appropriate language option in the modal
    if (locale === 'zh') {
      const zhButton = this.page.locator('button:has-text("中文")');
      await zhButton.click();
    } else {
      const enButton = this.page.locator('button:has-text("English")');
      await enButton.click();
    }

    // Wait for modal to close and language to change
    await this.page.waitForTimeout(500);
  }

  /**
   * Wait for navigation to complete
   */
  async waitForNavigation() {
    await this.page.waitForLoadState('networkidle');
  }
}

