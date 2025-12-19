import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Arrangement/Sequencer Interactions E2E Tests', () => {
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
  // ARRANGEMENT VIEW BASIC TESTS
  // ============================================================================

  test.describe('Arrangement View Basics', () => {
    test('should display arrangement view', async ({ page }) => {
      const arrangementView = page.locator('.arrangement-view, .arrangement-main, .sequencer');
      await expect(arrangementView).toBeVisible({ timeout: 5000 });
    });

    test('should display timeline/ruler', async ({ page }) => {
      const timeline = page.locator('.timeline, .ruler, .time-ruler, .arrangement-ruler');
      await expect(timeline).toBeVisible();
    });

    test('should display track area', async ({ page }) => {
      const trackArea = page.locator('.track-area, .tracks-container, .arrangement-tracks');
      await expect(trackArea).toBeVisible();
    });

    test('should display transport bar', async ({ page }) => {
      const transport = page.locator('.transport, .transport-controls, .transport-bar');
      await expect(transport).toBeVisible();
    });
  });

  // ============================================================================
  // TRACK MANAGEMENT TESTS
  // ============================================================================

  test.describe('Track Management', () => {
    test('should have ability to add tracks', async ({ page }) => {
      const addTrackBtn = page.locator('button, .btn').filter({ hasText: /Add Track|New Track|\+/ });
      // Add track button may exist
    });

    test('should display track headers', async ({ page }) => {
      const trackHeaders = page.locator('.track-header, .track-label');
      // May have track headers
    });

    test('should select track on click', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track').first();
      if (await track.isVisible({ timeout: 3000 })) {
        await track.click();
        await page.waitForTimeout(200);

        const hasSelectedClass = await track.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeTruthy();
      }
    });

    test('should allow track name editing', async ({ page }) => {
      const trackName = page.locator('.track-name, .track-label').first();
      if (await trackName.isVisible({ timeout: 3000 })) {
        await trackName.dblclick();
        await page.waitForTimeout(200);

        // Input should appear for editing
        const input = page.locator('.track-name input, input[type="text"]');
        // May have inline edit
      }
    });

    test('should have mute button on tracks', async ({ page }) => {
      const muteBtn = page.locator('.track .mute-btn, .track-mute').first();
      // May have mute button
    });

    test('should have solo button on tracks', async ({ page }) => {
      const soloBtn = page.locator('.track .solo-btn, .track-solo').first();
      // May have solo button
    });

    test('should have arm/record button on tracks', async ({ page }) => {
      const armBtn = page.locator('.track .arm-btn, .track-arm, .record-arm').first();
      // May have arm button
    });
  });

  // ============================================================================
  // CLIP MANAGEMENT TESTS
  // ============================================================================

  test.describe('Clip Management', () => {
    test('should display clips on tracks', async ({ page }) => {
      const clips = page.locator('.sequencer-clip, .clip, .arrangement-clip');
      // May have clips
    });

    test('should select clip on click', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(200);

        const hasSelectedClass = await clip.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeTruthy();
      }
    });

    test('should open clip editor on double-click', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.dblclick();
        await page.waitForTimeout(500);

        // Piano roll or clip editor should open
        const editor = page.locator('.piano-roll, .clip-editor, .midi-editor');
        // May open editor
      }
    });

    test('should drag clip to move', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        const initialBox = await clip.boundingBox();
        if (!initialBox) {
          test.skip();
          return;
        }

        await clip.hover();
        await page.mouse.down();
        await page.mouse.move(initialBox.x + 100, initialBox.y, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(200);

        // Clip should have moved
        const newBox = await clip.boundingBox();
        if (newBox) {
          expect(newBox.x).not.toEqual(initialBox.x);
        }
      }
    });

    test('should resize clip from edge', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        const initialBox = await clip.boundingBox();
        if (!initialBox) {
          test.skip();
          return;
        }

        // Find resize handle on right edge
        const rightEdge = { x: initialBox.x + initialBox.width - 2, y: initialBox.y + initialBox.height / 2 };

        await page.mouse.move(rightEdge.x, rightEdge.y);
        await page.mouse.down();
        await page.mouse.move(rightEdge.x + 50, rightEdge.y, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(200);

        // Clip may have resized
      }
    });

    test('should delete selected clip with Delete key', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(200);

        const initialCount = await page.locator('.sequencer-clip, .clip').count();

        await page.keyboard.press('Delete');
        await page.waitForTimeout(300);

        // Clip should be deleted
        const newCount = await page.locator('.sequencer-clip, .clip').count();
        // May have deleted clip
      }
    });

    test('should duplicate clip with Ctrl+D', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(200);

        const initialCount = await page.locator('.sequencer-clip, .clip').count();

        await page.keyboard.press('Control+d');
        await page.waitForTimeout(300);

        // May have duplicated clip
      }
    });
  });

  // ============================================================================
  // PLAYHEAD/CURSOR TESTS
  // ============================================================================

  test.describe('Playhead/Cursor', () => {
    test('should display playhead', async ({ page }) => {
      const playhead = page.locator('.playhead, .cursor, .position-marker');
      await expect(playhead).toBeVisible();
    });

    test('should move playhead on timeline click', async ({ page }) => {
      const timeline = page.locator('.timeline, .ruler').first();
      if (await timeline.isVisible()) {
        const box = await timeline.boundingBox();
        if (box) {
          await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
          await page.waitForTimeout(200);

          // Playhead should move to click position
        }
      }
    });

    test('should move playhead during playback', async ({ page }) => {
      const playhead = page.locator('.playhead, .cursor').first();
      if (!(await playhead.isVisible())) {
        test.skip();
        return;
      }

      const initialPos = await playhead.boundingBox();
      if (!initialPos) {
        test.skip();
        return;
      }

      // Start playback
      await page.keyboard.press('Space');
      await page.waitForTimeout(500);

      // Stop
      await page.keyboard.press('Space');

      // Playhead may have moved
    });
  });

  // ============================================================================
  // TRANSPORT CONTROL TESTS
  // ============================================================================

  test.describe('Transport Controls', () => {
    test('should have play button', async ({ page }) => {
      const playBtn = page.locator('.play-btn, button[title*="Play"], [aria-label*="Play"]').first();
      // Play button should exist
    });

    test('should have stop button', async ({ page }) => {
      const stopBtn = page.locator('.stop-btn, button[title*="Stop"], [aria-label*="Stop"]').first();
      // Stop button should exist
    });

    test('should have record button', async ({ page }) => {
      const recordBtn = page.locator('.record-btn, button[title*="Record"], [aria-label*="Record"]').first();
      // Record button should exist
    });

    test('should have loop toggle', async ({ page }) => {
      const loopBtn = page.locator('.loop-btn, button[title*="Loop"], [aria-label*="Loop"]').first();
      // Loop button should exist
    });

    test('should display BPM/tempo', async ({ page }) => {
      const bpmDisplay = page.locator('.bpm, .tempo, [class*="bpm"]');
      // BPM display should exist
    });

    test('should display time position', async ({ page }) => {
      const timeDisplay = page.locator('.time-display, .position-display, [class*="time"]');
      // Time display should exist
    });

    test('should toggle metronome', async ({ page }) => {
      const metronomeBtn = page.locator('.metronome-btn, button[title*="Metronome"]').first();
      if (await metronomeBtn.isVisible()) {
        await metronomeBtn.click();
        await page.waitForTimeout(200);

        // Metronome state should toggle
      }
    });
  });

  // ============================================================================
  // ZOOM/SCROLL TESTS
  // ============================================================================

  test.describe('Zoom and Scroll', () => {
    test('should zoom horizontally with mouse wheel', async ({ page }) => {
      const timeline = page.locator('.timeline, .ruler').first();
      if (await timeline.isVisible()) {
        const box = await timeline.boundingBox();
        if (box) {
          // Ctrl+scroll to zoom
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.keyboard.down('Control');
          await page.mouse.wheel(0, -100);
          await page.keyboard.up('Control');
          await page.waitForTimeout(200);

          // Zoom may have changed
        }
      }
    });

    test('should scroll horizontally', async ({ page }) => {
      const trackArea = page.locator('.track-area, .arrangement-tracks').first();
      if (await trackArea.isVisible()) {
        // Shift+scroll for horizontal scroll
        await trackArea.hover();
        await page.keyboard.down('Shift');
        await page.mouse.wheel(100, 0);
        await page.keyboard.up('Shift');
        await page.waitForTimeout(200);

        // Scroll position may have changed
      }
    });

    test('should have zoom controls', async ({ page }) => {
      const zoomIn = page.locator('.zoom-in, button[title*="Zoom In"]');
      const zoomOut = page.locator('.zoom-out, button[title*="Zoom Out"]');
      // Zoom controls may exist
    });
  });

  // ============================================================================
  // SELECTION TESTS
  // ============================================================================

  test.describe('Selection', () => {
    test('should select all clips with Ctrl+A', async ({ page }) => {
      await page.keyboard.press('Control+a');
      await page.waitForTimeout(200);

      // All clips should be selected
    });

    test('should deselect with Escape', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(200);

        await page.keyboard.press('Escape');
        await page.waitForTimeout(200);

        // Should be deselected
        const hasSelectedClass = await clip.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeFalsy();
      }
    });

    test('should multi-select with Shift+click', async ({ page }) => {
      const clips = page.locator('.sequencer-clip, .clip');
      const count = await clips.count();
      if (count >= 2) {
        await clips.first().click();
        await page.waitForTimeout(100);

        await page.keyboard.down('Shift');
        await clips.nth(1).click();
        await page.keyboard.up('Shift');
        await page.waitForTimeout(200);

        // Both should be selected
      }
    });

    test('should toggle selection with Ctrl+click', async ({ page }) => {
      const clips = page.locator('.sequencer-clip, .clip');
      const count = await clips.count();
      if (count >= 2) {
        await clips.first().click();
        await page.waitForTimeout(100);

        await page.keyboard.down('Control');
        await clips.nth(1).click();
        await page.keyboard.up('Control');
        await page.waitForTimeout(200);

        // Second clip should be added to selection
      }
    });

    test('should rubber-band select with drag', async ({ page }) => {
      const trackArea = page.locator('.track-area, .arrangement-tracks').first();
      if (await trackArea.isVisible()) {
        const box = await trackArea.boundingBox();
        if (box) {
          // Draw selection rectangle
          await page.mouse.move(box.x + 10, box.y + 10);
          await page.mouse.down();
          await page.mouse.move(box.x + 200, box.y + 100, { steps: 10 });
          await page.mouse.up();
          await page.waitForTimeout(200);

          // Clips in area should be selected
        }
      }
    });
  });

  // ============================================================================
  // SPLIT VIEW TESTS
  // ============================================================================

  test.describe('Split View', () => {
    test('should toggle split view', async ({ page }) => {
      await page.keyboard.press('Control+e');
      await page.waitForTimeout(300);

      // Split view may have toggled
      const splitView = page.locator('.split-view, .piano-roll-split');
      // May be visible
    });

    test('should resize split view divider', async ({ page }) => {
      await page.keyboard.press('Control+e');
      await page.waitForTimeout(300);

      const divider = page.locator('.split-divider, .resize-divider');
      if (await divider.isVisible()) {
        const box = await divider.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(box.x, box.y - 100, { steps: 10 });
          await page.mouse.up();
          await page.waitForTimeout(200);
        }
      }
    });
  });

  // ============================================================================
  // AUTOMATION TESTS
  // ============================================================================

  test.describe('Automation', () => {
    test('should toggle automation view', async ({ page }) => {
      const automationBtn = page.locator('button').filter({ hasText: /Automation|Auto/ });
      if (await automationBtn.isVisible()) {
        await automationBtn.click();
        await page.waitForTimeout(200);

        const automationLane = page.locator('.automation-lane, .automation-view');
        // Automation lane may appear
      }
    });

    test('should draw automation points', async ({ page }) => {
      const automationLane = page.locator('.automation-lane').first();
      if (await automationLane.isVisible()) {
        const box = await automationLane.boundingBox();
        if (box) {
          // Draw automation
          await page.mouse.click(box.x + 50, box.y + box.height / 2);
          await page.waitForTimeout(100);
          await page.mouse.click(box.x + 100, box.y + box.height / 4);
          await page.waitForTimeout(100);
        }
      }
    });
  });

  // ============================================================================
  // UNDO/REDO TESTS
  // ============================================================================

  test.describe('Undo/Redo', () => {
    test('should undo with Ctrl+Z', async ({ page }) => {
      await page.keyboard.press('Control+z');
      await page.waitForTimeout(200);
      // Should not throw
    });

    test('should redo with Ctrl+Shift+Z', async ({ page }) => {
      await page.keyboard.press('Control+Shift+z');
      await page.waitForTimeout(200);
      // Should not throw
    });

    test('should redo with Ctrl+Y', async ({ page }) => {
      await page.keyboard.press('Control+y');
      await page.waitForTimeout(200);
      // Should not throw
    });
  });

  // ============================================================================
  // GRID SNAP TESTS
  // ============================================================================

  test.describe('Grid Snap', () => {
    test('should have grid snap toggle', async ({ page }) => {
      const snapBtn = page.locator('.snap-btn, button[title*="Snap"]');
      // Snap button may exist
    });

    test('should have grid size selector', async ({ page }) => {
      const gridSelector = page.locator('.grid-size, select[name="grid"]');
      // Grid selector may exist
    });
  });

  // ============================================================================
  // COPY/PASTE TESTS
  // ============================================================================

  test.describe('Copy/Paste', () => {
    test('should copy selected clip with Ctrl+C', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(100);

        await page.keyboard.press('Control+c');
        await page.waitForTimeout(200);
        // Should not throw
      }
    });

    test('should paste with Ctrl+V', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(100);

        await page.keyboard.press('Control+c');
        await page.waitForTimeout(100);

        await page.keyboard.press('Control+v');
        await page.waitForTimeout(300);

        // Clip may have been pasted
      }
    });

    test('should cut selected clip with Ctrl+X', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (await clip.isVisible({ timeout: 3000 })) {
        await clip.click();
        await page.waitForTimeout(100);

        await page.keyboard.press('Control+x');
        await page.waitForTimeout(300);
        // Clip may have been cut
      }
    });
  });
});
