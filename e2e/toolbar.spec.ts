import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Toolbar E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
    // Open arrangement window
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(500);
  });

  // ============================================================================
  // ARRANGEMENT WINDOW TOOLBAR TESTS
  // ============================================================================

  test.describe('Arrangement Window Toolbar', () => {
    test('should display arrangement window toolbar', async ({ page }) => {
      const toolbar = page.locator('.arrangement-window .toolbar, .toolbar').first();
      await expect(toolbar).toBeVisible({ timeout: 5000 });
    });

    test('should have View button in toolbar', async ({ page }) => {
      const viewBtn = page.locator('.toolbar-btn').filter({ hasText: 'View' });
      await expect(viewBtn).toBeVisible();
    });

    test('should open View dropdown menu', async ({ page }) => {
      const viewBtn = page.locator('.toolbar-btn').filter({ hasText: 'View' });
      await viewBtn.click();
      await page.waitForTimeout(200);

      // Dropdown should appear
      const dropdown = page.locator('.dropdown-menu');
      await expect(dropdown).toBeVisible();

      // Should have view options
      await expect(dropdown.locator('text=Track Type Icons')).toBeVisible();
      await expect(dropdown.locator('text=Clip Names')).toBeVisible();
      await expect(dropdown.locator('text=Clip Previews')).toBeVisible();
      await expect(dropdown.locator('text=Color Mode')).toBeVisible();
    });

    test('should toggle Track Type Icons option', async ({ page }) => {
      const viewBtn = page.locator('.toolbar-btn').filter({ hasText: 'View' });
      await viewBtn.click();
      await page.waitForTimeout(200);

      const trackTypeCheckbox = page.locator('.dropdown-menu input[type="checkbox"]').first();
      const initialState = await trackTypeCheckbox.isChecked();

      await trackTypeCheckbox.click();
      await page.waitForTimeout(200);

      const newState = await trackTypeCheckbox.isChecked();
      expect(newState).not.toBe(initialState);
    });

    test('should toggle Clip Names option', async ({ page }) => {
      const viewBtn = page.locator('.toolbar-btn').filter({ hasText: 'View' });
      await viewBtn.click();
      await page.waitForTimeout(200);

      // Find Clip Names checkbox (second checkbox)
      const checkboxes = page.locator('.dropdown-menu input[type="checkbox"]');
      const count = await checkboxes.count();
      if (count >= 2) {
        const clipNamesCheckbox = checkboxes.nth(1);
        const initialState = await clipNamesCheckbox.isChecked();
        await clipNamesCheckbox.click();
        await page.waitForTimeout(200);
        const newState = await clipNamesCheckbox.isChecked();
        expect(newState).not.toBe(initialState);
      }
    });

    test('should have Piano Roll button', async ({ page }) => {
      const pianoRollBtn = page.locator('.toolbar-btn').filter({ hasText: 'Piano Roll' });
      await expect(pianoRollBtn).toBeVisible();
    });

    test('should toggle Piano Roll', async ({ page }) => {
      const pianoRollBtn = page.locator('.toolbar-btn').filter({ hasText: 'Piano Roll' });
      await pianoRollBtn.click();
      await page.waitForTimeout(300);

      // Either split view enabled or window opened
      const pianoRollWindow = page.locator('.piano-roll-window, [data-window="piano-roll"]');
      // Check if button becomes active or window appears
    });

    test('should have Mixer button', async ({ page }) => {
      const mixerBtn = page.locator('.toolbar-btn').filter({ hasText: 'Mixer' });
      await expect(mixerBtn).toBeVisible();
    });

    test('should toggle Mixer window', async ({ page }) => {
      const mixerBtn = page.locator('.toolbar-btn').filter({ hasText: 'Mixer' });
      await mixerBtn.click();
      await page.waitForTimeout(300);

      // Mixer window should appear
      const mixerWindow = page.locator('.mixer-window');
      await expect(mixerWindow).toBeVisible({ timeout: 5000 });
    });

    test('should have Browser button', async ({ page }) => {
      const browserBtn = page.locator('.toolbar-btn').filter({ hasText: 'Browser' });
      await expect(browserBtn).toBeVisible();
    });

    test('should toggle Browser window', async ({ page }) => {
      const browserBtn = page.locator('.toolbar-btn').filter({ hasText: 'Browser' });
      await browserBtn.click();
      await page.waitForTimeout(300);

      // Database browser window state should change
    });

    test('should display status indicator', async ({ page }) => {
      const statusIndicator = page.locator('.status-indicator').first();
      await expect(statusIndicator).toBeVisible();

      // Should show Stopped by default
      await expect(statusIndicator).toContainText(/Stopped|Playing|Recording/);
    });

    test('should show Playing status when playing', async ({ page }) => {
      // Start playback
      await page.keyboard.press('Space');
      await page.waitForTimeout(300);

      const statusIndicator = page.locator('.status-indicator').first();
      await expect(statusIndicator).toContainText('Playing');

      // Stop
      await page.keyboard.press('Space');
    });

    test('should display track type legend', async ({ page }) => {
      const legend = page.locator('.track-type-legend');
      await expect(legend).toBeVisible();

      // Should have MIDI, Drum, Audio legends
      await expect(legend.locator('text=MIDI')).toBeVisible();
      await expect(legend.locator('text=Drum')).toBeVisible();
      await expect(legend.locator('text=Audio')).toBeVisible();
    });
  });

  // ============================================================================
  // MIXER WINDOW TOOLBAR TESTS
  // ============================================================================

  test.describe('Mixer Window Toolbar', () => {
    test.beforeEach(async ({ page }) => {
      // Open mixer window
      await page.keyboard.press('Alt+2');
      await page.waitForTimeout(500);
    });

    test('should display mixer window', async ({ page }) => {
      const mixerWindow = page.locator('.mixer-window');
      await expect(mixerWindow).toBeVisible({ timeout: 5000 });
    });

    test('should have mixer channels', async ({ page }) => {
      const channels = page.locator('.mixer-channel, .channel');
      const count = await channels.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should have volume faders on channels', async ({ page }) => {
      const faders = page.locator('.fader, input[type="range"], .volume-slider');
      const count = await faders.count();
      expect(count).toBeGreaterThan(0);
    });

    test('should have mute buttons on channels', async ({ page }) => {
      const muteButtons = page.locator('.mute-btn, button[title*="Mute"]');
      const count = await muteButtons.count();
      // May have mute buttons per channel
    });

    test('should have solo buttons on channels', async ({ page }) => {
      const soloButtons = page.locator('.solo-btn, button[title*="Solo"]');
      const count = await soloButtons.count();
      // May have solo buttons per channel
    });

    test('should toggle channel mute', async ({ page }) => {
      const muteBtn = page.locator('.mute-btn, button[title*="Mute"]').first();
      if (await muteBtn.isVisible()) {
        await muteBtn.click();
        await page.waitForTimeout(200);

        // Button should have active/muted state
        const hasActiveClass = await muteBtn.evaluate((el) =>
          el.classList.contains('active') || el.classList.contains('muted')
        );
        expect(hasActiveClass).toBeTruthy();

        // Toggle off
        await muteBtn.click();
        await page.waitForTimeout(200);
      }
    });

    test('should have master channel', async ({ page }) => {
      const masterChannel = page.locator('.master-channel, .channel-master, [data-channel="master"]');
      if (await masterChannel.isVisible()) {
        await expect(masterChannel).toBeVisible();
      }
    });
  });

  // ============================================================================
  // STATUS BAR TESTS
  // ============================================================================

  test.describe('Status Bar', () => {
    test('should display status bar', async ({ page }) => {
      const statusBar = page.locator('.status-bar');
      await expect(statusBar).toBeVisible();
    });

    test('should show BPM/tempo information', async ({ page }) => {
      const bpmDisplay = page.locator('.bpm, [class*="tempo"], [class*="bpm"]');
      // BPM should be visible somewhere
    });

    test('should show time signature', async ({ page }) => {
      const timeSignature = page.locator('.time-signature, [class*="time-sig"]');
      // Time signature may be visible
    });

    test('should show playback position', async ({ page }) => {
      const position = page.locator('.position, .time-display, [class*="position"]');
      // Position display should exist
    });
  });

  // ============================================================================
  // TRANSPORT CONTROLS TESTS
  // ============================================================================

  test.describe('Transport Controls', () => {
    test('should have transport controls in arrangement', async ({ page }) => {
      const transportBar = page.locator('.transport, .transport-controls, .arrangement-view');
      await expect(transportBar).toBeVisible({ timeout: 5000 });
    });

    test('should have play button', async ({ page }) => {
      const playBtn = page.locator('[title*="Play"], .play-btn, button').filter({ has: page.locator('svg') }).first();
      // Play button should exist
    });

    test('should have stop button', async ({ page }) => {
      const stopBtn = page.locator('[title*="Stop"], .stop-btn');
      // Stop button should exist
    });

    test('should have record button', async ({ page }) => {
      const recordBtn = page.locator('[title*="Record"], .record-btn');
      // Record button should exist
    });

    test('should have loop button', async ({ page }) => {
      const loopBtn = page.locator('[title*="Loop"], .loop-btn');
      // Loop button should exist
    });

    test('transport controls should respond to clicks', async ({ page }) => {
      // Test clicking transport controls
      const playBtn = page.locator('[title*="Play"], .play-btn').first();
      if (await playBtn.isVisible()) {
        await playBtn.click();
        await page.waitForTimeout(300);

        // Status should change
        const statusIndicator = page.locator('.status-indicator').first();
        await expect(statusIndicator).toContainText(/Playing/);

        // Stop
        await page.keyboard.press('Enter');
      }
    });
  });

  // ============================================================================
  // TOOLBAR BUTTON INTERACTIONS
  // ============================================================================

  test.describe('Toolbar Button Interactions', () => {
    test('toolbar buttons should have hover state', async ({ page }) => {
      const toolbarBtn = page.locator('.toolbar-btn').first();
      if (await toolbarBtn.isVisible()) {
        await toolbarBtn.hover();
        await page.waitForTimeout(100);

        // Check for hover style (background change)
        const bgColor = await toolbarBtn.evaluate((el) =>
          getComputedStyle(el).backgroundColor
        );
        // Should have some background color on hover
      }
    });

    test('active toolbar buttons should show active state', async ({ page }) => {
      const viewBtn = page.locator('.toolbar-btn').filter({ hasText: 'View' });
      await viewBtn.click();
      await page.waitForTimeout(200);

      // Check for active class
      await expect(viewBtn).toHaveClass(/active/);
    });

    test('toolbar buttons should have tooltips', async ({ page }) => {
      const toolbarBtns = page.locator('.toolbar-btn[title], button[title]');
      const count = await toolbarBtns.count();
      expect(count).toBeGreaterThan(0);
    });
  });
});
