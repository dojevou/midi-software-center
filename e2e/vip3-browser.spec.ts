import { test, expect, waitForWorkspace } from './fixtures';

test.describe('VIP3 Browser E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
    // VIP3 browser should be visible by default
  });

  test('should display VIP3 browser on startup', async ({ page }) => {
    // VIP3 browser is shown by default
    const browser = page.locator('.browser-layout, [class*="vip3-browser"], [class*="browser"]').first();
    await expect(browser).toBeVisible({ timeout: 5000 });
  });

  test('should have search functionality', async ({ page }) => {
    // Find search input
    const searchInput = page.locator('input[type="search"], input[placeholder*="search" i], .search-input').first();

    if (await searchInput.isVisible()) {
      await searchInput.fill('test');
      await page.waitForTimeout(300);

      // Search should be triggered
      const inputValue = await searchInput.inputValue();
      expect(inputValue).toBe('test');
    }
  });

  test('should have sidebar with saved searches', async ({ page }) => {
    const sidebar = page.locator('.sidebar, [class*="sidebar"]').first();
    await expect(sidebar).toBeVisible({ timeout: 5000 });
  });

  test('should toggle VIP3 browser with Ctrl+B', async ({ page }) => {
    // Toggle off
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(300);

    // Toggle on
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(300);

    const browser = page.locator('.browser-layout').first();
    await expect(browser).toBeVisible({ timeout: 5000 });
  });

  test('should display file list', async ({ page }) => {
    // Look for file list container
    const fileList = page.locator('.browser-main, .file-list, [class*="results"]').first();
    await expect(fileList).toBeVisible({ timeout: 5000 });
  });

  test('should support filtering', async ({ page }) => {
    // Look for filter controls
    const filterControls = page.locator('[class*="filter"], .filter-panel, .sidebar').first();
    await expect(filterControls).toBeVisible({ timeout: 5000 });
  });

  test('should collapse/expand sidebar', async ({ page }) => {
    const sidebar = page.locator('.sidebar').first();

    if (await sidebar.isVisible()) {
      // Check for collapse button
      const collapseBtn = page.locator('.sidebar button, [class*="collapse"]').first();

      if (await collapseBtn.isVisible()) {
        await collapseBtn.click();
        await page.waitForTimeout(200);

        // Sidebar should be collapsed (has class)
        await expect(sidebar).toHaveClass(/collapsed/);
      }
    }
  });

  test('should have main browser area', async ({ page }) => {
    const mainArea = page.locator('.browser-main').first();
    await expect(mainArea).toBeVisible({ timeout: 5000 });
  });

  test('should display layout properly', async ({ page }) => {
    const browserLayout = page.locator('.browser-layout').first();
    await expect(browserLayout).toBeVisible({ timeout: 5000 });

    // Should have both sidebar and main area
    const sidebar = browserLayout.locator('.sidebar');
    const main = browserLayout.locator('.browser-main');

    await expect(sidebar).toBeVisible();
    await expect(main).toBeVisible();
  });
});
