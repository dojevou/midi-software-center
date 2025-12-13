import { test, expect } from '@playwright/test';

test.describe('MIDI Software Center E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('http://localhost:1420');
    await page.waitForLoadState('networkidle');
  });

  test('should launch the app and test DAW transport controls', async ({ page }) => {
    const playButton = page.locator('button:has-text("▶")');
    await expect(playButton).toBeVisible();

    await playButton.click();
    await expect(page.locator('button:has-text("⏸")')).toBeVisible();

    const pauseButton = page.locator('button:has-text("⏸")');
    await pauseButton.click();
    await expect(playButton).toBeVisible();

    const stopButton = page.locator('button:has-text("⏹")');
    await stopButton.click();
    await expect(playButton).toBeVisible();
  });

  test('should test mixer volume adjustment', async ({ page }) => {
    const mixerButton = page.locator('button:has-text("Mixer")');
    await mixerButton.click();
    await page.waitForLoadState('networkidle');

    const volumeSlider = page.locator('input[type="range"][min="0"][max="1"]');
    await expect(volumeSlider).toBeVisible();

    await volumeSlider.fill('0.5');
    await expect(volumeSlider).toHaveValue('0.5');
  });

  test('should test database search and load to DAW', async ({ page }) => {
    const databaseButton = page.locator('button:has-text("Database")');
    await databaseButton.click();
    await page.waitForLoadState('networkidle');

    const searchInput = page.locator('input[placeholder*="Search"]');
    await searchInput.fill('test');
    await searchInput.press('Enter');

    // Wait for results
    await page.waitForSelector('.result-item', { timeout: 5000 });

    // Double-click to load
    const firstResult = page.locator('.result-item').first();
    await firstResult.dblclick();

    // Verify loaded in DAW (switch back to DAW window)
    const dawButton = page.locator('button:has-text("DAW")');
    await dawButton.click();
    await expect(page.locator('.track-row')).toHaveCount(1);
  });

  test('should test pipeline import', async ({ page }) => {
    const pipelineButton = page.locator('button:has-text("Pipeline")');
    await pipelineButton.click();
    await page.waitForLoadState('networkidle');

    const importSelect = page.locator('select').first();
    await importSelect.selectOption('import');

    const startButton = page.locator('button:has-text("Start Import")');
    await startButton.click();

    await page.waitForSelector('.progress-fill', { state: 'visible', timeout: 10000 });

    await expect(page.locator('.progress-info')).toBeVisible();
  });
});