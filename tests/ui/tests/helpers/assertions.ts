import { Page, expect, Locator } from '@playwright/test';

/**
 * Assert that a table contains specific text
 */
export async function assertTableContainsText(page: Page, text: string) {
  const table = page.locator('table');
  await expect(table).toContainText(text);
}

/**
 * Assert that a table has a specific number of rows
 */
export async function assertTableRowCount(page: Page, expectedCount: number) {
  const rows = await page.locator('table tbody tr').count();
  expect(rows).toBe(expectedCount);
}

/**
 * Assert that a table has at least a minimum number of rows
 */
export async function assertTableHasMinRows(page: Page, minCount: number) {
  const rows = await page.locator('table tbody tr').count();
  expect(rows).toBeGreaterThanOrEqual(minCount);
}

/**
 * Assert that no loading indicator is visible
 */
export async function assertNoLoadingIndicator(page: Page) {
  const loading = page.locator('text=/Loading|加载中/');
  await expect(loading).not.toBeVisible();
}

/**
 * Assert that no error messages are visible
 */
export async function assertNoErrors(page: Page) {
  const error = page.locator('[role="alert"], .error-message');
  await expect(error).not.toBeVisible();
}

/**
 * Assert that an error message is visible
 */
export async function assertErrorVisible(page: Page, errorText?: string) {
  const error = page.locator('[role="alert"], .error-message');
  await expect(error).toBeVisible();
  
  if (errorText) {
    await expect(error).toContainText(errorText);
  }
}

/**
 * Assert that a success message is visible
 */
export async function assertSuccessVisible(page: Page, successText?: string) {
  const success = page.locator('[role="status"], .success-message');
  await expect(success).toBeVisible();
  
  if (successText) {
    await expect(success).toContainText(successText);
  }
}

/**
 * Assert that an empty state is visible
 */
export async function assertEmptyState(page: Page, emptyText?: string) {
  const emptyState = page.locator('text=/No.*found|未找到|Empty|空/');
  await expect(emptyState).toBeVisible();
  
  if (emptyText) {
    await expect(emptyState).toContainText(emptyText);
  }
}

/**
 * Assert that a button is enabled
 */
export async function assertButtonEnabled(button: Locator) {
  await expect(button).toBeEnabled();
}

/**
 * Assert that a button is disabled
 */
export async function assertButtonDisabled(button: Locator) {
  await expect(button).toBeDisabled();
}

/**
 * Assert that navigation is visible
 */
export async function assertNavigationVisible(page: Page) {
  const nav = page.locator('nav');
  await expect(nav).toBeVisible();
}

/**
 * Assert that a specific page title is visible
 */
export async function assertPageTitle(page: Page, title: string | RegExp) {
  const heading = page.locator('h1, h2').first();
  if (typeof title === 'string') {
    await expect(heading).toContainText(title);
  } else {
    await expect(heading).toHaveText(title);
  }
}

/**
 * Assert that the page has loaded successfully (no 404, no errors)
 */
export async function assertPageLoaded(page: Page) {
  // Check that we're not on a 404 page
  const notFound = page.locator('text=/404|Not Found|未找到页面/');
  await expect(notFound).not.toBeVisible();
  
  // Check that navigation is visible
  await assertNavigationVisible(page);
  
  // Check that no errors are visible
  await assertNoErrors(page);
}

/**
 * Assert that a form field has a specific validation error
 */
export async function assertFieldError(page: Page, fieldName: string, errorText: string) {
  const field = page.locator(`[name="${fieldName}"]`);
  const errorMessage = field.locator('..').locator('.error, [role="alert"]');
  await expect(errorMessage).toContainText(errorText);
}

/**
 * Assert that pagination controls are visible
 */
export async function assertPaginationVisible(page: Page) {
  const pagination = page.locator('button').filter({ hasText: /Previous|Next|上一页|下一页/ });
  await expect(pagination.first()).toBeVisible();
}

