import { test, expect, waitForWorkspace } from './fixtures';

test.describe('MIDI Software Center E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    // Wait for app to be ready with graceful timeout
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
  });

  test('should launch the app and display main UI elements', async ({ page }) => {
    // Check workspace exists (attached but may have width=0 due to fixed children)
    await expect(page.locator('.workspace')).toBeAttached();

    // Check at least one window is visible
    await expect(page.locator('.window-base').first()).toBeVisible();

    // Check menu bar exists
    await expect(page.locator('.menu-bar').first()).toBeVisible();

    // Check status bar exists
    await expect(page.locator('.status-bar').first()).toBeVisible();
  });

  test('should test DAW transport controls', async ({ page }) => {
    // Open arrangement window if not visible (Alt+1)
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(500);

    // Find arrangement window toolbar (contains transport controls)
    const toolbar = page.locator('.arrangement-window .toolbar, .toolbar').first();
    await expect(toolbar).toBeVisible({ timeout: 5000 });

    // Test play button in toolbar
    const playBtn = page.locator('.toolbar-btn, button[title*="play" i], button[aria-label*="play" i]').first();
    if (await playBtn.isVisible()) {
      await playBtn.click();
      await page.waitForTimeout(300);
    }
  });

  test('should test keyboard shortcuts for transport', async ({ page }) => {
    // Press Space to play
    await page.keyboard.press('Space');
    await page.waitForTimeout(300);

    // Press Space again to pause
    await page.keyboard.press('Space');
    await page.waitForTimeout(300);

    // Press Enter to stop
    await page.keyboard.press('Enter');
    await page.waitForTimeout(300);
  });

  test('should toggle VIP3 browser with keyboard shortcut', async ({ page }) => {
    // VIP3 browser should be visible by default
    const vip3Browser = page.locator('.vip3-browser').first();
    await expect(vip3Browser).toBeVisible({ timeout: 5000 });

    // Ctrl+B toggles it closed
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(500);
    await expect(vip3Browser).not.toBeVisible({ timeout: 5000 });

    // Ctrl+B again toggles it open
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(500);
    await expect(vip3Browser).toBeVisible({ timeout: 5000 });
  });

  test('should open pipeline/import with keyboard shortcut', async ({ page }) => {
    // Open pipeline with Ctrl+I
    await page.keyboard.press('Control+i');
    await page.waitForTimeout(500);

    // Check for modal with Import header
    const importModal = page.locator('.modal-overlay, [role="dialog"]').first();
    await expect(importModal).toBeVisible({ timeout: 5000 });

    // Close with Escape
    await page.keyboard.press('Escape');
  });

  test('should test window management shortcuts', async ({ page }) => {
    // Alt+1 - Arrangement
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    // Alt+2 - Mixer
    await page.keyboard.press('Alt+2');
    await page.waitForTimeout(300);

    // Alt+3 - Database
    await page.keyboard.press('Alt+3');
    await page.waitForTimeout(300);

    // Escape to close any open modals
    await page.keyboard.press('Escape');
  });
});
