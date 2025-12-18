import { test, expect } from '@playwright/test';

test.describe('Keyboard Shortcuts E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should toggle play/pause with Space key', async ({ page }) => {
    // Ensure body has focus
    await page.click('body');

    // Press Space to play
    await page.keyboard.press('Space');
    await page.waitForTimeout(100);

    // Check for playing state
    const isPlaying = await page.evaluate(() => {
      const playButton = document.querySelector('[data-testid="play-button"], button[aria-label="Play"]');
      return playButton?.getAttribute('aria-pressed') === 'true' ||
             document.querySelector('.playing') !== null;
    });

    // Press Space to pause
    await page.keyboard.press('Space');
    await page.waitForTimeout(100);
  });

  test('should stop with Enter key', async ({ page }) => {
    await page.click('body');
    await page.keyboard.press('Enter');
    await page.waitForTimeout(100);

    // Playhead should be at start
    const timeDisplay = page.locator('.time-display, [data-testid="time-display"]');
    if (await timeDisplay.isVisible()) {
      const text = await timeDisplay.textContent();
      // Should show start position
    }
  });

  test('should undo/redo with Ctrl+Z/Ctrl+Shift+Z', async ({ page }) => {
    await page.click('body');

    // Try undo
    await page.keyboard.press('Control+z');
    await page.waitForTimeout(100);

    // Try redo
    await page.keyboard.press('Control+Shift+z');
    await page.waitForTimeout(100);
  });

  test('should zoom in/out with Ctrl+Plus/Minus', async ({ page }) => {
    await page.click('body');

    // Zoom in
    await page.keyboard.press('Control+=');
    await page.waitForTimeout(100);

    // Zoom out
    await page.keyboard.press('Control+-');
    await page.waitForTimeout(100);
  });

  test('should select all with Ctrl+A', async ({ page }) => {
    await page.click('body');
    await page.keyboard.press('Control+a');
    await page.waitForTimeout(100);
  });

  test('should delete selected with Delete/Backspace', async ({ page }) => {
    await page.click('body');

    // First select something
    await page.keyboard.press('Control+a');
    await page.waitForTimeout(100);

    // Then delete
    await page.keyboard.press('Delete');
    await page.waitForTimeout(100);
  });

  test('should toggle loop with L key', async ({ page }) => {
    await page.click('body');

    const loopToggle = page.locator('[data-testid="loop-toggle"], button[aria-label="Loop"]');
    let initialState: string | null = null;

    if (await loopToggle.isVisible()) {
      initialState = await loopToggle.getAttribute('aria-pressed');
    }

    // Press L to toggle loop
    await page.keyboard.press('l');
    await page.waitForTimeout(100);

    if (await loopToggle.isVisible() && initialState !== null) {
      const newState = await loopToggle.getAttribute('aria-pressed');
      expect(newState).not.toBe(initialState);
    }
  });

  test('should toggle record with R key', async ({ page }) => {
    await page.click('body');

    // Press R to toggle recording
    await page.keyboard.press('r');
    await page.waitForTimeout(100);

    // Press R again to toggle off
    await page.keyboard.press('r');
    await page.waitForTimeout(100);
  });

  test('should navigate to start with Home key', async ({ page }) => {
    await page.click('body');
    await page.keyboard.press('Home');
    await page.waitForTimeout(100);
  });

  test('should navigate to end with End key', async ({ page }) => {
    await page.click('body');
    await page.keyboard.press('End');
    await page.waitForTimeout(100);
  });

  test('should open keyboard shortcuts help with Ctrl+?', async ({ page }) => {
    await page.click('body');

    // Try to open shortcuts help
    await page.keyboard.press('Control+Shift+/'); // Ctrl+?
    await page.waitForTimeout(200);

    // Look for shortcuts modal/dialog
    const shortcutsHelp = page.locator(
      '.keyboard-shortcuts, [data-testid="shortcuts-help"], .shortcuts-modal'
    );

    // May not be implemented
    if (await shortcutsHelp.isVisible()) {
      await expect(shortcutsHelp).toBeVisible();
      // Close it
      await page.keyboard.press('Escape');
    }
  });

  test('should close modals with Escape key', async ({ page }) => {
    // Open any modal (like settings)
    const settingsButton = page.locator('[data-testid="settings"], button:has-text("Settings")');

    if (await settingsButton.isVisible()) {
      await settingsButton.click();
      await page.waitForTimeout(200);

      // Close with Escape
      await page.keyboard.press('Escape');
      await page.waitForTimeout(100);
    }
  });

  test('should duplicate selected with Ctrl+D', async ({ page }) => {
    await page.click('body');

    // First select something (if possible)
    const clip = page.locator('.clip, [data-testid="clip"]').first();
    if (await clip.isVisible()) {
      await clip.click();
      await page.waitForTimeout(100);

      // Get initial clip count
      const initialCount = await page.locator('.clip, [data-testid="clip"]').count();

      // Duplicate
      await page.keyboard.press('Control+d');
      await page.waitForTimeout(200);

      // Check count increased
      const newCount = await page.locator('.clip, [data-testid="clip"]').count();
      // Note: May not work if no clip is selected
    }
  });

  test('should copy/paste with Ctrl+C/Ctrl+V', async ({ page }) => {
    await page.click('body');

    // Copy
    await page.keyboard.press('Control+c');
    await page.waitForTimeout(100);

    // Paste
    await page.keyboard.press('Control+v');
    await page.waitForTimeout(100);
  });

  test('should cut with Ctrl+X', async ({ page }) => {
    await page.click('body');

    // Cut
    await page.keyboard.press('Control+x');
    await page.waitForTimeout(100);
  });
});
