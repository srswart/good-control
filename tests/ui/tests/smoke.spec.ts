import { test, expect } from '@playwright/test';
import { NavigationHelper } from './helpers/navigation';
import { assertNavigationVisible, assertPageLoaded } from './helpers/assertions';
import { BasePage } from './pages/BasePage';

test.describe('Application Smoke Tests', () => {
  test('should load home page', async ({ page }) => {
    const nav = new NavigationHelper(page);

    await nav.goToHome();

    // Check for app title
    await expect(page).toHaveTitle(/Controlled Goods SCM/);

    // Check for navigation using helper
    await assertNavigationVisible(page);
  });

  test('should navigate to inventory page', async ({ page }) => {
    const nav = new NavigationHelper(page);
    const basePage = new BasePage(page);

    // Navigate to inventory page
    await nav.goToInventoryItems();

    // Verify page loaded successfully
    await assertPageLoaded(page);

    // Verify navigation is visible
    await basePage.assertNavigationVisible();
  });

  test('should switch language', async ({ page }) => {
    const nav = new NavigationHelper(page);

    await nav.goToHome();

    // Try to switch to Chinese
    await nav.switchLanguage('zh');

    // Verify the page still works
    await assertNavigationVisible(page);

    // Switch back to English
    await nav.switchLanguage('en');

    // Verify the page still works
    await assertNavigationVisible(page);
  });
});

