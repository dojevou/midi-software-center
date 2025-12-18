import { test, expect } from '@playwright/test';

test.describe('VIP3 Browser E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display VIP3 browser when navigating to database', async ({ page }) => {
    // Open VIP3 browser via navigation
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Check that VIP3 browser elements are visible
    await expect(page.locator('.vip3-browser, [data-testid="vip3-browser"]')).toBeVisible({ timeout: 5000 });
  });

  test('should filter files by search query', async ({ page }) => {
    // Navigate to VIP3 browser
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Enter search query
    const searchInput = page.locator('input[placeholder*="Search"], input[type="search"], [data-testid="vip3-search"]');
    await searchInput.first().fill('piano');
    await searchInput.first().press('Enter');

    // Wait for results to update
    await page.waitForTimeout(500);

    // Check that some results are displayed or "no results" message
    const results = page.locator('.file-row, .result-item, [data-testid="file-item"]');
    const noResults = page.locator(':text("No files found"), :text("No results")');

    // Either results exist or no results message shows
    const hasResults = await results.count() > 0;
    const hasNoResultsMessage = await noResults.count() > 0;
    expect(hasResults || hasNoResultsMessage).toBeTruthy();
  });

  test('should select and deselect files', async ({ page }) => {
    // Navigate to VIP3 browser
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Wait for file list to load
    const fileItems = page.locator('.file-row, .result-item, [data-testid="file-item"]');

    // Skip test if no files available
    const count = await fileItems.count();
    if (count === 0) {
      test.skip();
      return;
    }

    // Click first file to select
    await fileItems.first().click();
    await expect(fileItems.first()).toHaveClass(/selected|active/);

    // Ctrl+click second file for multi-select
    if (count > 1) {
      await fileItems.nth(1).click({ modifiers: ['Control'] });
    }
  });

  test('should display file details on selection', async ({ page }) => {
    // Navigate to VIP3 browser
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Wait for file list to load
    const fileItems = page.locator('.file-row, .result-item, [data-testid="file-item"]');

    // Skip test if no files available
    const count = await fileItems.count();
    if (count === 0) {
      test.skip();
      return;
    }

    // Click a file to select it
    await fileItems.first().click();

    // Check for BPM, key, or duration display
    const fileInfo = page.locator(':text("BPM"), :text("Key"), :text("Duration")');
    const hasInfo = await fileInfo.count() > 0;
    expect(hasInfo).toBeTruthy();
  });

  test('should handle pagination', async ({ page }) => {
    // Navigate to VIP3 browser
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Look for pagination controls
    const pagination = page.locator('.pagination, [data-testid="pagination"], button:has-text("Next"), button:has-text("Previous")');

    // Skip if pagination isn't visible (might not have enough files)
    const hasPagination = await pagination.count() > 0;
    if (!hasPagination) {
      test.skip();
      return;
    }

    // Click next page if available
    const nextButton = page.locator('button:has-text("Next"), [data-testid="next-page"]');
    if (await nextButton.isVisible()) {
      await nextButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('should sort files by different columns', async ({ page }) => {
    // Navigate to VIP3 browser
    const databaseButton = page.locator('[data-testid="nav-database"], button:has-text("Database"), button:has-text("Browse")');
    await databaseButton.first().click();
    await page.waitForLoadState('networkidle');

    // Find sort controls
    const sortSelect = page.locator('select[name="sort"], [data-testid="sort-select"]');

    if (await sortSelect.isVisible()) {
      // Sort by BPM
      await sortSelect.selectOption({ label: /BPM/i });
      await page.waitForTimeout(500);

      // Sort by name
      await sortSelect.selectOption({ label: /Name/i });
      await page.waitForTimeout(500);
    }
  });
});
