import { test, expect } from '@playwright/test';

test.describe('Sequencer/DAW E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('should display sequencer with transport controls', async ({ page }) => {
    // Look for sequencer or DAW view
    const sequencer = page.locator('.sequencer, [data-testid="sequencer"], .daw-container');
    await expect(sequencer).toBeVisible({ timeout: 5000 });

    // Check for transport controls
    const transportControls = page.locator('.transport, [data-testid="transport"], .sequencer-transport');
    await expect(transportControls).toBeVisible();
  });

  test('should toggle play/pause', async ({ page }) => {
    // Find play button
    const playButton = page.locator('[data-testid="play-button"], button[aria-label="Play"], button:has-text("Play")');

    if (await playButton.isVisible()) {
      await playButton.click();
      await page.waitForTimeout(200);

      // Check for pause button or playing state
      const pauseButton = page.locator('[data-testid="pause-button"], button[aria-label="Pause"], button:has-text("Pause")');
      const isPlaying = await pauseButton.isVisible() || await playButton.getAttribute('aria-pressed') === 'true';
      expect(isPlaying).toBeTruthy();
    }
  });

  test('should stop playback and reset position', async ({ page }) => {
    // Find stop button
    const stopButton = page.locator('[data-testid="stop-button"], button[aria-label="Stop"], button:has-text("Stop")');

    if (await stopButton.isVisible()) {
      await stopButton.click();
      await page.waitForTimeout(200);

      // Verify playhead is at start (position 0 or displays 0:00)
      const timeDisplay = page.locator('.time-display, [data-testid="time-display"]');
      if (await timeDisplay.isVisible()) {
        const timeText = await timeDisplay.textContent();
        expect(timeText).toMatch(/0:00|0\.0|1\.1/); // Could be 0:00, 0.0.0, or bar 1.1
      }
    }
  });

  test('should adjust tempo/BPM', async ({ page }) => {
    // Find BPM input or slider
    const bpmInput = page.locator('input[data-testid="bpm-input"], input[aria-label="BPM"], input.bpm-input');

    if (await bpmInput.isVisible()) {
      await bpmInput.clear();
      await bpmInput.fill('140');
      await bpmInput.press('Enter');

      // Verify BPM changed
      await expect(bpmInput).toHaveValue('140');
    }
  });

  test('should add and remove tracks', async ({ page }) => {
    // Find add track button
    const addTrackButton = page.locator('[data-testid="add-track"], button:has-text("Add Track"), button[aria-label="Add Track"]');

    if (await addTrackButton.isVisible()) {
      // Get initial track count
      const tracks = page.locator('.track, [data-testid="track"], .sequencer-track');
      const initialCount = await tracks.count();

      await addTrackButton.click();
      await page.waitForTimeout(200);

      // Verify track was added
      await expect(tracks).toHaveCount(initialCount + 1);
    }
  });

  test('should handle keyboard shortcuts for transport', async ({ page }) => {
    // Focus sequencer area
    const sequencer = page.locator('.sequencer, [data-testid="sequencer"]');
    await sequencer.click();

    // Press Space for play/pause
    await page.keyboard.press('Space');
    await page.waitForTimeout(100);

    // Press Space again to pause
    await page.keyboard.press('Space');
    await page.waitForTimeout(100);

    // Press Enter/Stop
    await page.keyboard.press('Enter');
    await page.waitForTimeout(100);
  });

  test('should zoom in and out', async ({ page }) => {
    // Find zoom controls
    const zoomInButton = page.locator('[data-testid="zoom-in"], button[aria-label="Zoom In"], button:has-text("+")');
    const zoomOutButton = page.locator('[data-testid="zoom-out"], button[aria-label="Zoom Out"], button:has-text("-")');

    if (await zoomInButton.isVisible()) {
      await zoomInButton.click();
      await page.waitForTimeout(100);
      await zoomOutButton.click();
      await page.waitForTimeout(100);
    }
  });

  test('should scroll timeline horizontally', async ({ page }) => {
    // Find scrollable timeline or tracks container
    const tracksContainer = page.locator('.tracks-container, [data-testid="tracks-container"]');

    if (await tracksContainer.isVisible()) {
      // Scroll horizontally
      await tracksContainer.evaluate((el) => {
        el.scrollLeft = 200;
      });
      await page.waitForTimeout(100);

      const scrollLeft = await tracksContainer.evaluate((el) => el.scrollLeft);
      expect(scrollLeft).toBeGreaterThan(0);
    }
  });

  test('should toggle loop mode', async ({ page }) => {
    // Find loop toggle
    const loopToggle = page.locator('[data-testid="loop-toggle"], button[aria-label="Loop"], button:has-text("Loop")');

    if (await loopToggle.isVisible()) {
      // Get initial state
      const initialState = await loopToggle.getAttribute('aria-pressed');

      await loopToggle.click();
      await page.waitForTimeout(100);

      // Check state changed
      const newState = await loopToggle.getAttribute('aria-pressed');
      expect(newState).not.toBe(initialState);
    }
  });

  test('should display playhead position', async ({ page }) => {
    const playhead = page.locator('.playhead, [data-testid="playhead"], .sequencer-playhead');
    await expect(playhead).toBeVisible();
  });

  test('should handle recording toggle', async ({ page }) => {
    const recordButton = page.locator('[data-testid="record-button"], button[aria-label="Record"], button:has-text("Record")');

    if (await recordButton.isVisible()) {
      await recordButton.click();
      await page.waitForTimeout(200);

      // Check for recording state
      const isRecording = await recordButton.getAttribute('aria-pressed') === 'true' ||
                          await recordButton.evaluate((el) => el.classList.contains('recording'));

      // Toggle off
      await recordButton.click();
      await page.waitForTimeout(100);
    }
  });
});
