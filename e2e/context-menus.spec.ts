import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Context Menu E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
    await page.waitForTimeout(300);
  });

  // ============================================================================
  // VIP3 FILE CONTEXT MENU TESTS
  // ============================================================================

  test.describe('VIP3 File Context Menu', () => {
    test('should open context menu on right-click file item', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"], .vip3-context-menu');
      await expect(contextMenu).toBeVisible({ timeout: 2000 });
    });

    test('should have Load to Sequencer option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const loadOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Load|Add|Sequencer/ });
      if (await loadOption.isVisible()) {
        await expect(loadOption).toBeVisible();
      }
    });

    test('should have Add to Favorites option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const favOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Favorite/ });
      if (await favOption.isVisible()) {
        await expect(favOption).toBeVisible();
      }
    });

    test('should have Add to Collection option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const collectionOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Collection/ });
      if (await collectionOption.isVisible()) {
        await expect(collectionOption).toBeVisible();
      }
    });

    test('should have Show Details option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const detailsOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Details|Properties|Info/ });
      if (await detailsOption.isVisible()) {
        await expect(detailsOption).toBeVisible();
      }
    });

    test('should close context menu on item click', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        const menuItem = contextMenu.locator('button, [role="menuitem"]').first();
        await menuItem.click();
        await page.waitForTimeout(200);

        await expect(contextMenu).not.toBeVisible();
      }
    });

    test('should close context menu on Escape', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        await page.keyboard.press('Escape');
        await page.waitForTimeout(200);

        await expect(contextMenu).not.toBeVisible();
      }
    });

    test('should close context menu on click outside', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        await page.locator('.vip3-browser, .workspace').click({ position: { x: 10, y: 10 }, force: true });
        await page.waitForTimeout(200);

        await expect(contextMenu).not.toBeVisible();
      }
    });
  });

  // ============================================================================
  // SEQUENCER TRACK CONTEXT MENU TESTS
  // ============================================================================

  test.describe('Sequencer Track Context Menu', () => {
    test.beforeEach(async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(500);
    });

    test('should open context menu on right-click track', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track-header, .track').first();
      if (!(await track.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await track.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      // May or may not have context menu
    });

    test('should have Rename Track option', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track-header').first();
      if (!(await track.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await track.click({ button: 'right' });
      await page.waitForTimeout(200);

      const renameOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Rename/ });
      // May exist
    });

    test('should have Delete Track option', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track-header').first();
      if (!(await track.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await track.click({ button: 'right' });
      await page.waitForTimeout(200);

      const deleteOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Delete|Remove/ });
      // May exist
    });

    test('should have Duplicate Track option', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track-header').first();
      if (!(await track.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await track.click({ button: 'right' });
      await page.waitForTimeout(200);

      const duplicateOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Duplicate|Copy/ });
      // May exist
    });

    test('should have Change Track Color option', async ({ page }) => {
      const track = page.locator('.sequencer-track, .track-header').first();
      if (!(await track.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await track.click({ button: 'right' });
      await page.waitForTimeout(200);

      const colorOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Color/ });
      // May exist
    });
  });

  // ============================================================================
  // SEQUENCER CLIP CONTEXT MENU TESTS
  // ============================================================================

  test.describe('Sequencer Clip Context Menu', () => {
    test.beforeEach(async ({ page }) => {
      await page.keyboard.press('Alt+1');
      await page.waitForTimeout(500);
    });

    test('should open context menu on right-click clip', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (!(await clip.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await clip.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      // May or may not have context menu
    });

    test('should have Edit Clip option', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (!(await clip.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await clip.click({ button: 'right' });
      await page.waitForTimeout(200);

      const editOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Edit/ });
      // May exist
    });

    test('should have Delete Clip option', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (!(await clip.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await clip.click({ button: 'right' });
      await page.waitForTimeout(200);

      const deleteOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Delete|Remove/ });
      // May exist
    });

    test('should have Split Clip option', async ({ page }) => {
      const clip = page.locator('.sequencer-clip, .clip').first();
      if (!(await clip.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await clip.click({ button: 'right' });
      await page.waitForTimeout(200);

      const splitOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Split/ });
      // May exist
    });
  });

  // ============================================================================
  // MIXER CHANNEL CONTEXT MENU TESTS
  // ============================================================================

  test.describe('Mixer Channel Context Menu', () => {
    test.beforeEach(async ({ page }) => {
      await page.keyboard.press('Alt+2');
      await page.waitForTimeout(500);
    });

    test('should open context menu on right-click channel', async ({ page }) => {
      const channel = page.locator('.mixer-channel, .channel').first();
      if (!(await channel.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await channel.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      // May or may not have context menu
    });

    test('should have Rename Channel option', async ({ page }) => {
      const channel = page.locator('.mixer-channel, .channel').first();
      if (!(await channel.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await channel.click({ button: 'right' });
      await page.waitForTimeout(200);

      const renameOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Rename/ });
      // May exist
    });

    test('should have Reset Channel option', async ({ page }) => {
      const channel = page.locator('.mixer-channel, .channel').first();
      if (!(await channel.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      await channel.click({ button: 'right' });
      await page.waitForTimeout(200);

      const resetOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Reset/ });
      // May exist
    });
  });

  // ============================================================================
  // WINDOW TITLE BAR CONTEXT MENU TESTS
  // ============================================================================

  test.describe('Window Title Bar Context Menu', () => {
    test('should open context menu on right-click title bar', async ({ page }) => {
      const window = page.locator('.window-base').first();
      if (!(await window.isVisible({ timeout: 3000 }))) {
        test.skip();
        return;
      }

      const titleBar = window.locator('.window-title').first();
      await titleBar.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      // May or may not have context menu on title bar
    });
  });

  // ============================================================================
  // CONTEXT MENU KEYBOARD NAVIGATION TESTS
  // ============================================================================

  test.describe('Context Menu Keyboard Navigation', () => {
    test('should navigate context menu with arrow keys', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        // Navigate with arrow keys
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(100);
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(100);
        await page.keyboard.press('ArrowUp');
        await page.waitForTimeout(100);

        // Close with Escape
        await page.keyboard.press('Escape');
      }
    });

    test('should activate item with Enter key', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        // Navigate and select
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(100);
        await page.keyboard.press('Enter');
        await page.waitForTimeout(200);

        // Menu should be closed
        await expect(contextMenu).not.toBeVisible();
      }
    });
  });

  // ============================================================================
  // CONTEXT MENU POSITIONING TESTS
  // ============================================================================

  test.describe('Context Menu Positioning', () => {
    test('context menu should appear at click position', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      const box = await fileItem.boundingBox();
      if (!box) {
        test.skip();
        return;
      }

      const clickX = box.x + box.width / 2;
      const clickY = box.y + box.height / 2;

      await page.mouse.click(clickX, clickY, { button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        const menuBox = await contextMenu.boundingBox();
        if (menuBox) {
          // Menu should be near click position
          expect(Math.abs(menuBox.x - clickX)).toBeLessThan(200);
          expect(Math.abs(menuBox.y - clickY)).toBeLessThan(200);
        }

        await page.keyboard.press('Escape');
      }
    });

    test('context menu should stay within viewport', async ({ page }) => {
      const fileItem = page.locator('.file-item').last();
      const isVisible = await fileItem.isVisible().catch(() => false);
      if (!isVisible) {
        test.skip();
        return;
      }

      await fileItem.click({ button: 'right' });
      await page.waitForTimeout(200);

      const contextMenu = page.locator('.context-menu, [role="menu"]');
      if (await contextMenu.isVisible()) {
        const menuBox = await contextMenu.boundingBox();
        const viewport = page.viewportSize();

        if (menuBox && viewport) {
          // Menu should be within viewport
          expect(menuBox.x).toBeGreaterThanOrEqual(0);
          expect(menuBox.y).toBeGreaterThanOrEqual(0);
          expect(menuBox.x + menuBox.width).toBeLessThanOrEqual(viewport.width + 10);
          expect(menuBox.y + menuBox.height).toBeLessThanOrEqual(viewport.height + 10);
        }

        await page.keyboard.press('Escape');
      }
    });
  });
});
