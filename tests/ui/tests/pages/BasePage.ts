import { Page, Locator, expect } from '@playwright/test';

/**
 * Base page object with common functionality for all pages
 */
export class BasePage {
  readonly page: Page;
  readonly navigation: Locator;
  readonly loadingIndicator: Locator;
  readonly errorMessage: Locator;
  readonly successMessage: Locator;

  constructor(page: Page) {
    this.page = page;
    this.navigation = page.locator('nav');
    this.loadingIndicator = page.locator('text=/Loading|加载中/');
    this.errorMessage = page.locator('[role="alert"], .error-message');
    this.successMessage = page.locator('[role="status"], .success-message');
  }

  /**
   * Wait for the page to finish loading
   */
  async waitForLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Wait for loading indicator to disappear
   */
  async waitForLoadingToComplete() {
    await expect(this.loadingIndicator).not.toBeVisible({ timeout: 10000 });
  }

  /**
   * Assert that navigation is visible
   */
  async assertNavigationVisible() {
    await expect(this.navigation).toBeVisible();
  }

  /**
   * Assert that no errors are visible
   */
  async assertNoErrors() {
    await expect(this.errorMessage).not.toBeVisible();
  }

  /**
   * Assert that an error is visible
   */
  async assertErrorVisible(errorText?: string) {
    await expect(this.errorMessage).toBeVisible();
    if (errorText) {
      await expect(this.errorMessage).toContainText(errorText);
    }
  }

  /**
   * Assert that a success message is visible
   */
  async assertSuccessVisible(successText?: string) {
    await expect(this.successMessage).toBeVisible();
    if (successText) {
      await expect(this.successMessage).toContainText(successText);
    }
  }

  /**
   * Assert that the page is not a 404
   */
  async assertNotFound() {
    const notFound = this.page.locator('text=/404|Not Found|未找到页面/');
    await expect(notFound).not.toBeVisible();
  }

  /**
   * Get the current URL
   */
  async getCurrentUrl(): Promise<string> {
    return this.page.url();
  }

  /**
   * Take a screenshot
   */
  async screenshot(name: string) {
    await this.page.screenshot({ path: `test-results/${name}.png` });
  }

  /**
   * Reload the page
   */
  async reload() {
    await this.page.reload();
    await this.waitForLoad();
  }

  /**
   * Go back in browser history
   */
  async goBack() {
    await this.page.goBack();
    await this.waitForLoad();
  }

  /**
   * Go forward in browser history
   */
  async goForward() {
    await this.page.goForward();
    await this.waitForLoad();
  }
}

