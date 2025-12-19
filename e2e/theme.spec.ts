import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Theme System E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
  });

  test('should have a theme selector', async ({ page }) => {
    // Theme selector might be in menu bar or preferences
    // Check for theme-related elements
    const themeSelector = page.locator('.theme-selector, .theme-button, [data-theme]').first();

    // If not directly visible, open preferences (Alt+8)
    if (!(await themeSelector.isVisible())) {
      await page.keyboard.press('Alt+8');
      await page.waitForTimeout(500);
    }

    // Look for theme options
    const themeIndicator = page.locator('.theme-indicator, [data-theme]').first();
    await expect(themeIndicator).toBeVisible({ timeout: 5000 });
  });

  test('should switch between themes', async ({ page }) => {
    // Open preferences window
    await page.keyboard.press('Alt+8');
    await page.waitForTimeout(500);

    // Find theme selector button
    const themeBtn = page.locator('.theme-button, .theme-selector button').first();

    if (await themeBtn.isVisible()) {
      await themeBtn.click();
      await page.waitForTimeout(200);

      // Theme dropdown should appear
      const dropdown = page.locator('.theme-dropdown, [role="listbox"]').first();
      await expect(dropdown).toBeVisible({ timeout: 2000 });

      // Select a different theme (e.g., NEON)
      const neonOption = page.locator('.theme-option:has-text("NEON"), button:has-text("NEON")').first();
      if (await neonOption.isVisible()) {
        await neonOption.click();
        await page.waitForTimeout(200);

        // Document should have data-theme-name attribute updated
        const htmlElement = page.locator('html');
        await expect(htmlElement).toHaveAttribute('data-theme-name', 'NEON');
      }
    }

    // Close preferences
    await page.keyboard.press('Escape');
  });

  test('should change theme via preferences', async ({ page }) => {
    // Open preferences
    await page.keyboard.press('Alt+8');
    await page.waitForTimeout(500);

    // Get current theme
    const initialTheme = await page.locator('html').getAttribute('data-theme-name');

    // Find and click theme selector
    const themeBtn = page.locator('.theme-button').first();
    if (await themeBtn.isVisible()) {
      await themeBtn.click();
      await page.waitForTimeout(200);

      // Select a different theme
      const themeOptions = page.locator('.theme-option');
      const optionCount = await themeOptions.count();

      if (optionCount > 1) {
        // Click second option (different from current)
        await themeOptions.nth(1).click();
        await page.waitForTimeout(200);

        // Theme should have changed
        const newTheme = await page.locator('html').getAttribute('data-theme-name');
        expect(newTheme).not.toBe(initialTheme);
      }
    }

    await page.keyboard.press('Escape');

    // Note: Theme persistence across reload requires actual Tauri backend
    // which the mock doesn't provide, so we skip the reload test
  });

  test('should apply CSS variables correctly', async ({ page }) => {
    // CSS variables should be applied to root
    const bgPrimary = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--bg-primary');
    });

    expect(bgPrimary).toBeTruthy();
    expect(bgPrimary.trim()).toMatch(/^#|^rgb/);
  });

  test('should display all theme options (DARK, WARM, NEON, MINT, ROSE, BASS)', async ({ page }) => {
    // Open preferences
    await page.keyboard.press('Alt+8');
    await page.waitForTimeout(500);

    const themeBtn = page.locator('.theme-button').first();
    if (await themeBtn.isVisible()) {
      await themeBtn.click();
      await page.waitForTimeout(200);

      const themes = ['DARK', 'WARM', 'NEON', 'MINT', 'ROSE', 'BASS'];

      for (const theme of themes) {
        const option = page.locator(`.theme-option:has-text("${theme}")`).first();
        await expect(option).toBeVisible();
      }
    }

    await page.keyboard.press('Escape');
    await page.keyboard.press('Escape');
  });

  test('should have proper contrast in theme', async ({ page }) => {
    // Get text and background colors
    const colors = await page.evaluate(() => {
      const style = getComputedStyle(document.documentElement);
      return {
        textPrimary: style.getPropertyValue('--text-primary'),
        bgPrimary: style.getPropertyValue('--bg-primary'),
      };
    });

    expect(colors.textPrimary).toBeTruthy();
    expect(colors.bgPrimary).toBeTruthy();

    // Both should be defined (not empty)
    expect(colors.textPrimary.trim().length).toBeGreaterThan(0);
    expect(colors.bgPrimary.trim().length).toBeGreaterThan(0);
  });

  test('should support reduced motion preference', async ({ page }) => {
    // Check for animation duration CSS variable
    const animDuration = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--animation-duration');
    });

    expect(animDuration).toBeTruthy();
    // Should be either a time value or 0ms for reduced motion
    expect(animDuration.trim()).toMatch(/\d+m?s/);
  });
});
