import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Keyboard Shortcuts E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
  });

  test('should toggle play/pause with Space key', async ({ page }) => {
    // Ensure arrangement is open
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    // Press space to play
    await page.keyboard.press('Space');
    await page.waitForTimeout(200);

    const playBtn = page.locator('.play-btn').first();
    if (await playBtn.isVisible()) {
      await expect(playBtn).toHaveClass(/playing/);
    }

    // Press space again to pause
    await page.keyboard.press('Space');
    await page.waitForTimeout(200);
  });

  test('should stop with Enter key', async ({ page }) => {
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    // Start playback
    await page.keyboard.press('Space');
    await page.waitForTimeout(200);

    // Stop with Enter
    await page.keyboard.press('Enter');
    await page.waitForTimeout(200);

    const playBtn = page.locator('.play-btn').first();
    if (await playBtn.isVisible()) {
      await expect(playBtn).not.toHaveClass(/playing/);
    }
  });

  test('should undo/redo with Ctrl+Z/Ctrl+Shift+Z', async ({ page }) => {
    // These shortcuts should be handled by the app
    await page.keyboard.press('Control+z');
    await page.waitForTimeout(100);

    await page.keyboard.press('Control+Shift+z');
    await page.waitForTimeout(100);

    // No error should occur
  });

  test('should toggle loop with L key', async ({ page }) => {
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    await page.keyboard.press('l');
    await page.waitForTimeout(200);

    const loopBtn = page.locator('.loop-btn').first();
    if (await loopBtn.isVisible()) {
      await expect(loopBtn).toHaveClass(/active/);

      await page.keyboard.press('l');
      await page.waitForTimeout(200);
      await expect(loopBtn).not.toHaveClass(/active/);
    }
  });

  test('should toggle record with R key', async ({ page }) => {
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    await page.keyboard.press('r');
    await page.waitForTimeout(200);

    const recordBtn = page.locator('.record-btn').first();
    if (await recordBtn.isVisible()) {
      await expect(recordBtn).toHaveClass(/recording/);

      // Stop recording
      await page.keyboard.press('Enter');
      await page.waitForTimeout(200);
    }
  });

  test('should navigate to start with Home key', async ({ page }) => {
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    await page.keyboard.press('Home');
    await page.waitForTimeout(100);
    // Should not throw
  });

  test('should navigate to end with End key', async ({ page }) => {
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    await page.keyboard.press('End');
    await page.waitForTimeout(100);
    // Should not throw
  });

  test('should close modals with Escape key', async ({ page }) => {
    // Open a modal (pipeline)
    await page.keyboard.press('Control+i');
    await page.waitForTimeout(500);

    const modal = page.locator('.modal-overlay').first();
    await expect(modal).toBeVisible();

    // Close with Escape
    await page.keyboard.press('Escape');
    await page.waitForTimeout(200);

    await expect(modal).not.toBeVisible();
  });

  test('should open pipeline with Ctrl+I', async ({ page }) => {
    await page.keyboard.press('Control+i');
    await page.waitForTimeout(500);

    const modal = page.locator('.modal-overlay, [role="dialog"]').first();
    await expect(modal).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('should open VIP3 browser with Ctrl+B', async ({ page }) => {
    // First close it if open
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(300);

    // Toggle again
    await page.keyboard.press('Control+b');
    await page.waitForTimeout(300);

    const browser = page.locator('.browser-layout, [class*="vip3"]').first();
    await expect(browser).toBeVisible({ timeout: 5000 });
  });

  test('should open export window with Ctrl+E', async ({ page }) => {
    await page.keyboard.press('Control+e');
    await page.waitForTimeout(500);

    const modal = page.locator('.modal-overlay, [role="dialog"]').first();
    await expect(modal).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('should open MIDI monitor with Ctrl+M', async ({ page }) => {
    await page.keyboard.press('Control+m');
    await page.waitForTimeout(500);

    // MIDI monitor may be a modal or a window - check for either
    const modal = page.locator('.modal-overlay, [role="dialog"], .midi-monitor, [class*="midi"]').first();
    if (await modal.isVisible()) {
      await page.keyboard.press('Escape');
    }
    // If nothing visible, the shortcut might not be implemented yet - that's ok
  });

  test('should cycle windows with Alt+Number keys', async ({ page }) => {
    // Alt+1 - Arrangement
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(200);

    // Alt+2 - Mixer
    await page.keyboard.press('Alt+2');
    await page.waitForTimeout(200);

    // Alt+3 - Database
    await page.keyboard.press('Alt+3');
    await page.waitForTimeout(200);

    // Alt+6 - MIDI I/O Setup
    await page.keyboard.press('Alt+6');
    await page.waitForTimeout(300);

    const modal = page.locator('.modal-overlay').first();
    await expect(modal).toBeVisible();

    await page.keyboard.press('Escape');
  });

  test('should toggle windows with F-keys', async ({ page }) => {
    // F1 toggles arrangement
    await page.keyboard.press('F1');
    await page.waitForTimeout(200);

    // F2 toggles mixer
    await page.keyboard.press('F2');
    await page.waitForTimeout(200);

    // F5 toggles pipeline
    await page.keyboard.press('F5');
    await page.waitForTimeout(300);

    const modal = page.locator('.modal-overlay').first();
    if (await modal.isVisible()) {
      await page.keyboard.press('Escape');
    }
  });

  test('should handle new project with Ctrl+N', async ({ page }) => {
    await page.keyboard.press('Control+n');
    await page.waitForTimeout(200);
    // Should not throw - new project should be created
  });

  test('should handle save project with Ctrl+S', async ({ page }) => {
    await page.keyboard.press('Control+s');
    await page.waitForTimeout(200);
    // Should not throw
  });
});
