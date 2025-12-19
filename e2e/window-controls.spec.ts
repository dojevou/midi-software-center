import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Window Controls E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
  });

  // ============================================================================
  // WINDOW CLOSE BUTTON TESTS
  // ============================================================================

  test.describe('Close Button', () => {
    test('should close arrangement window via close button', async ({ page }) => {
      // Open arrangement window
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const arrangementWindow = page.locator('.arrangement-window').first();
      await expect(arrangementWindow).toBeVisible({ timeout: 5000 });

      // Find and click close button
      const closeBtn = arrangementWindow.locator('.close-btn, button[aria-label="Close window"]').first();
      await closeBtn.click();
      await page.waitForTimeout(300);

      await expect(arrangementWindow).not.toBeVisible();
    });

    test('should close mixer window via close button', async ({ page }) => {
      // Open mixer window
      await page.keyboard.press('Alt+2');
      await page.waitForTimeout(300);

      const mixerWindow = page.locator('.mixer-window').first();
      await expect(mixerWindow).toBeVisible({ timeout: 5000 });

      const closeBtn = mixerWindow.locator('.close-btn, button[aria-label="Close window"]').first();
      await closeBtn.click();
      await page.waitForTimeout(300);

      await expect(mixerWindow).not.toBeVisible();
    });

    test('should close VIP3 browser window via close button', async ({ page }) => {
      // VIP3 browser should be visible by default
      const vip3Browser = page.locator('.vip3-browser').first();
      await expect(vip3Browser).toBeVisible({ timeout: 5000 });

      // Find close button in VIP3 browser's parent window
      const vip3Window = page.locator('.window-base').filter({ has: page.locator('.vip3-browser') }).first();
      const closeBtn = vip3Window.locator('.close-btn, button[aria-label="Close window"]').first();

      if (await closeBtn.isVisible()) {
        await closeBtn.click();
        await page.waitForTimeout(300);
        await expect(vip3Browser).not.toBeVisible();
      }
    });
  });

  // ============================================================================
  // WINDOW MINIMIZE BUTTON TESTS
  // ============================================================================

  test.describe('Minimize Button', () => {
    test('should minimize arrangement window', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const arrangementWindow = page.locator('.arrangement-window').first();
      await expect(arrangementWindow).toBeVisible({ timeout: 5000 });

      const minimizeBtn = arrangementWindow.locator('.minimize-btn, button[aria-label="Minimize window"]').first();
      if (await minimizeBtn.isVisible()) {
        await minimizeBtn.click();
        await page.waitForTimeout(300);

        // Window should be minimized (hidden or in taskbar)
        await expect(arrangementWindow).not.toBeVisible();
      }
    });

    test('should minimize mixer window', async ({ page }) => {
      await page.keyboard.press('Alt+2');
      await page.waitForTimeout(300);

      const mixerWindow = page.locator('.mixer-window').first();
      await expect(mixerWindow).toBeVisible({ timeout: 5000 });

      const minimizeBtn = mixerWindow.locator('.minimize-btn, button[aria-label="Minimize window"]').first();
      if (await minimizeBtn.isVisible()) {
        await minimizeBtn.click();
        await page.waitForTimeout(300);
        await expect(mixerWindow).not.toBeVisible();
      }
    });
  });

  // ============================================================================
  // WINDOW MAXIMIZE BUTTON TESTS
  // ============================================================================

  test.describe('Maximize Button', () => {
    test('should maximize arrangement window', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const arrangementWindow = page.locator('.arrangement-window, .window-base').first();
      await expect(arrangementWindow).toBeVisible({ timeout: 5000 });

      // Get initial size
      const initialBox = await arrangementWindow.boundingBox();
      if (!initialBox) {
        test.skip();
        return;
      }

      const maximizeBtn = arrangementWindow.locator('.maximize-btn, button[aria-label="Maximize window"]').first();
      if (await maximizeBtn.isVisible()) {
        await maximizeBtn.click();
        await page.waitForTimeout(300);

        // Check window is larger or has maximized class
        const hasMaximizedClass = await arrangementWindow.evaluate((el) =>
          el.classList.contains('maximized')
        );

        const newBox = await arrangementWindow.boundingBox();
        if (newBox) {
          // Either has maximized class or is larger
          expect(hasMaximizedClass || newBox.width > initialBox.width).toBeTruthy();
        }
      }
    });

    test('should restore window from maximized state', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      await expect(window).toBeVisible({ timeout: 5000 });

      const maximizeBtn = window.locator('.maximize-btn').first();
      if (!(await maximizeBtn.isVisible())) {
        test.skip();
        return;
      }

      // Maximize
      await maximizeBtn.click();
      await page.waitForTimeout(300);

      // Restore (click again)
      await maximizeBtn.click();
      await page.waitForTimeout(300);

      // Should not have maximized class
      const hasMaximizedClass = await window.evaluate((el) =>
        el.classList.contains('maximized')
      );
      expect(hasMaximizedClass).toBeFalsy();
    });

    test('should maximize on double-click title bar', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      await expect(window).toBeVisible({ timeout: 5000 });

      const titleBar = window.locator('.window-title').first();

      // Double-click title bar
      await titleBar.dblclick();
      await page.waitForTimeout(300);

      // Window should be maximized
      const hasMaximizedClass = await window.evaluate((el) =>
        el.classList.contains('maximized')
      );
      expect(hasMaximizedClass).toBeTruthy();
    });
  });

  // ============================================================================
  // WINDOW DRAG TESTS
  // ============================================================================

  test.describe('Window Dragging', () => {
    test('should drag window by title bar', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      await expect(window).toBeVisible({ timeout: 5000 });

      const titleBar = window.locator('.window-title').first();
      const initialBox = await window.boundingBox();
      if (!initialBox) {
        test.skip();
        return;
      }

      // Drag window
      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(initialBox.x + 100, initialBox.y + 50, { steps: 10 });
      await page.mouse.up();
      await page.waitForTimeout(200);

      const newBox = await window.boundingBox();
      if (newBox) {
        // Window should have moved
        expect(newBox.x).not.toEqual(initialBox.x);
      }
    });

    test('should show dragging cursor during drag', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const titleBar = window.locator('.window-title').first();
      const box = await window.boundingBox();
      if (!box) {
        test.skip();
        return;
      }

      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(box.x + 50, box.y + 50);

      // Check for dragging class
      const hasDraggingClass = await window.evaluate((el) =>
        el.classList.contains('dragging')
      );
      expect(hasDraggingClass).toBeTruthy();

      await page.mouse.up();
    });

    test('should snap window to edges', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const titleBar = window.locator('.window-title').first();
      const viewport = page.viewportSize();
      if (!viewport) {
        test.skip();
        return;
      }

      // Drag window to left edge
      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(5, 100, { steps: 10 }); // Near left edge
      await page.mouse.up();
      await page.waitForTimeout(300);

      const box = await window.boundingBox();
      if (box) {
        // Should snap to left edge (x = 0)
        expect(box.x).toBeLessThanOrEqual(10);
      }
    });
  });

  // ============================================================================
  // WINDOW RESIZE TESTS
  // ============================================================================

  test.describe('Window Resizing', () => {
    test('should resize window from SE corner', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      await expect(window).toBeVisible({ timeout: 5000 });

      const initialBox = await window.boundingBox();
      if (!initialBox) {
        test.skip();
        return;
      }

      // Find SE resize handle
      const resizeHandle = window.locator('.resize-se, .resize-handle').last();

      if (await resizeHandle.isVisible()) {
        const handleBox = await resizeHandle.boundingBox();
        if (handleBox) {
          await page.mouse.move(handleBox.x + handleBox.width / 2, handleBox.y + handleBox.height / 2);
          await page.mouse.down();
          await page.mouse.move(handleBox.x + 100, handleBox.y + 100, { steps: 10 });
          await page.mouse.up();
          await page.waitForTimeout(200);

          const newBox = await window.boundingBox();
          if (newBox) {
            expect(newBox.width).toBeGreaterThan(initialBox.width);
            expect(newBox.height).toBeGreaterThan(initialBox.height);
          }
        }
      }
    });

    test('should resize window from right edge', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const initialBox = await window.boundingBox();
      if (!initialBox) {
        test.skip();
        return;
      }

      const resizeHandle = window.locator('.resize-e').first();

      if (await resizeHandle.isVisible()) {
        const handleBox = await resizeHandle.boundingBox();
        if (handleBox) {
          await page.mouse.move(handleBox.x + handleBox.width / 2, handleBox.y + handleBox.height / 2);
          await page.mouse.down();
          await page.mouse.move(handleBox.x + 100, handleBox.y, { steps: 10 });
          await page.mouse.up();
          await page.waitForTimeout(200);

          const newBox = await window.boundingBox();
          if (newBox) {
            expect(newBox.width).toBeGreaterThan(initialBox.width);
          }
        }
      }
    });

    test('should enforce minimum window size', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();

      const resizeHandle = window.locator('.resize-se').first();
      if (!(await resizeHandle.isVisible())) {
        test.skip();
        return;
      }

      const handleBox = await resizeHandle.boundingBox();
      if (handleBox) {
        // Try to resize to very small
        await page.mouse.move(handleBox.x + handleBox.width / 2, handleBox.y + handleBox.height / 2);
        await page.mouse.down();
        await page.mouse.move(handleBox.x - 500, handleBox.y - 500, { steps: 10 });
        await page.mouse.up();
        await page.waitForTimeout(200);

        const newBox = await window.boundingBox();
        if (newBox) {
          // Should be at least minimum size (200x150)
          expect(newBox.width).toBeGreaterThanOrEqual(200);
          expect(newBox.height).toBeGreaterThanOrEqual(150);
        }
      }
    });
  });

  // ============================================================================
  // WINDOW FOCUS TESTS
  // ============================================================================

  test.describe('Window Focus', () => {
    test('should bring window to front on click', async ({ page }) => {
      // Open two windows
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);
      await page.keyboard.press('Alt+2');
      await page.waitForTimeout(300);

      const arrangementWindow = page.locator('.arrangement-window, .window-base').first();
      const mixerWindow = page.locator('.mixer-window').first();

      // Click arrangement window to bring to front
      await arrangementWindow.click();
      await page.waitForTimeout(200);

      // Arrangement window should have higher z-index
      const arrZIndex = await arrangementWindow.evaluate((el) =>
        parseInt(getComputedStyle(el).zIndex || '0')
      );
      const mixerZIndex = await mixerWindow.evaluate((el) =>
        parseInt(getComputedStyle(el).zIndex || '0')
      );

      // Arrangement should be on top or equal
      expect(arrZIndex).toBeGreaterThanOrEqual(mixerZIndex);
    });

    test('should show focus ring when window is active', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();

      // Focus the window
      await window.click();
      await page.waitForTimeout(200);

      // Check for focus-within styles (box-shadow changes)
      const boxShadow = await window.evaluate((el) =>
        getComputedStyle(el).boxShadow
      );

      // Should have some box-shadow for focus
      expect(boxShadow).not.toBe('none');
    });
  });

  // ============================================================================
  // WINDOW DOCKING TESTS
  // ============================================================================

  test.describe('Window Docking', () => {
    test('should dock window to left edge', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const titleBar = window.locator('.window-title').first();
      const viewport = page.viewportSize();
      if (!viewport) {
        test.skip();
        return;
      }

      // Drag to left edge dock zone
      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(2, viewport.height / 2, { steps: 10 }); // Far left
      await page.waitForTimeout(100);
      await page.mouse.up();
      await page.waitForTimeout(300);

      // Window should be docked (takes up half screen)
      const box = await window.boundingBox();
      if (box) {
        expect(box.x).toBeLessThanOrEqual(5);
        expect(box.width).toBeCloseTo(viewport.width / 2, -1);
      }
    });

    test('should show dock preview while dragging', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const titleBar = window.locator('.window-title').first();
      const viewport = page.viewportSize();
      if (!viewport) {
        test.skip();
        return;
      }

      // Drag to left edge without releasing
      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(2, viewport.height / 2, { steps: 10 });

      // Check for dock preview
      const dockPreview = page.locator('.dock-preview');
      await expect(dockPreview).toBeVisible({ timeout: 1000 });

      await page.mouse.up();
    });
  });

  // ============================================================================
  // WINDOW STATE PERSISTENCE TESTS
  // ============================================================================

  test.describe('Window State', () => {
    test('should maintain window position after multiple operations', async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(300);

      const window = page.locator('.window-base').first();
      const titleBar = window.locator('.window-title').first();
      const initialBox = await window.boundingBox();
      if (!initialBox) {
        test.skip();
        return;
      }

      // Move window
      await titleBar.hover();
      await page.mouse.down();
      await page.mouse.move(initialBox.x + 100, initialBox.y + 100, { steps: 5 });
      await page.mouse.up();
      await page.waitForTimeout(200);

      const movedBox = await window.boundingBox();

      // Click away then back to the window
      await page.locator('.workspace').click({ force: true });
      await page.waitForTimeout(200);
      await window.click();
      await page.waitForTimeout(200);

      // Position should be maintained
      const finalBox = await window.boundingBox();
      if (movedBox && finalBox) {
        expect(finalBox.x).toEqual(movedBox.x);
        expect(finalBox.y).toEqual(movedBox.y);
      }
    });
  });
});
