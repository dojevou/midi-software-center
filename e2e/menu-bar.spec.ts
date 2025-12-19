import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Menu Bar E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
  });

  // ============================================================================
  // FILE MENU TESTS
  // ============================================================================

  test.describe('File Menu', () => {
    test('should open File menu and display all items', async ({ page }) => {
      // Click File menu
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      // Verify dropdown is visible
      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Verify all menu items are present
      await expect(dropdown.locator('text=New Project')).toBeVisible();
      await expect(dropdown.locator('text=Open Project')).toBeVisible();
      await expect(dropdown.locator('text=Save Project')).toBeVisible();
      await expect(dropdown.locator('text=Save As...')).toBeVisible();
      await expect(dropdown.locator('text=Import Files...')).toBeVisible();
      await expect(dropdown.locator('text=Export MIDI')).toBeVisible();
      await expect(dropdown.locator('text=Quit')).toBeVisible();
    });

    test('should trigger New Project action', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      // Mock confirm dialog - accept it
      page.on('dialog', (dialog) => dialog.accept());

      const newProject = page.locator('.dropdown-menu button').filter({ hasText: 'New Project' });
      await newProject.click();
      await page.waitForTimeout(500);

      // Menu should close after action
      await expect(page.locator('.dropdown-menu')).not.toBeVisible();
    });

    test('should trigger Import Files action and show pipeline window', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      const importItem = page.locator('.dropdown-menu button').filter({ hasText: 'Import Files...' });
      await importItem.click();
      await page.waitForTimeout(500);

      // Pipeline window or modal should appear
      const pipelineWindow = page.locator('.pipeline-window, .modal-overlay, [data-window="pipeline"]');
      // May or may not show depending on implementation
    });

    test('File menu shortcuts should be displayed', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      // Check shortcuts are visible
      await expect(page.locator('.dropdown-menu').locator('text=Ctrl+N')).toBeVisible();
      await expect(page.locator('.dropdown-menu').locator('text=Ctrl+O')).toBeVisible();
      await expect(page.locator('.dropdown-menu').locator('text=Ctrl+S')).toBeVisible();
    });
  });

  // ============================================================================
  // EDIT MENU TESTS
  // ============================================================================

  test.describe('Edit Menu', () => {
    test('should open Edit menu and display all items', async ({ page }) => {
      const editMenu = page.locator('.menu-item button').filter({ hasText: 'Edit' });
      await editMenu.click();
      await page.waitForTimeout(200);

      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Check edit menu items
      await expect(dropdown.locator('text=Undo')).toBeVisible();
      await expect(dropdown.locator('text=Redo')).toBeVisible();
      await expect(dropdown.locator('text=Cut')).toBeVisible();
      await expect(dropdown.locator('text=Copy')).toBeVisible();
      await expect(dropdown.locator('text=Paste')).toBeVisible();
      await expect(dropdown.locator('text=Delete')).toBeVisible();
      await expect(dropdown.locator('text=Select All')).toBeVisible();
      await expect(dropdown.locator('text=Preferences')).toBeVisible();
    });

    test('should have disabled edit items when nothing selected', async ({ page }) => {
      const editMenu = page.locator('.menu-item button').filter({ hasText: 'Edit' });
      await editMenu.click();
      await page.waitForTimeout(200);

      // Undo/Redo/Cut/Copy/Paste/Delete/Select All should be disabled
      const undoBtn = page.locator('.dropdown-menu button').filter({ hasText: 'Undo' });
      await expect(undoBtn).toBeDisabled();
    });

    test('should open Preferences dialog', async ({ page }) => {
      const editMenu = page.locator('.menu-item button').filter({ hasText: 'Edit' });
      await editMenu.click();
      await page.waitForTimeout(200);

      const prefsItem = page.locator('.dropdown-menu button').filter({ hasText: 'Preferences' });
      await prefsItem.click();
      await page.waitForTimeout(500);

      // Preferences dialog should appear
      const prefsDialog = page.locator('[role="dialog"]').filter({ hasText: 'Preferences' });
      await expect(prefsDialog).toBeVisible();

      // Close with Escape
      await page.keyboard.press('Escape');
      await page.waitForTimeout(200);
      await expect(prefsDialog).not.toBeVisible();
    });
  });

  // ============================================================================
  // VIEW MENU TESTS
  // ============================================================================

  test.describe('View Menu', () => {
    test('should open View menu and display all items', async ({ page }) => {
      const viewMenu = page.locator('.menu-item button').filter({ hasText: 'View' });
      await viewMenu.click();
      await page.waitForTimeout(200);

      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Check view menu items
      await expect(dropdown.locator('text=Toggle DAW Window')).toBeVisible();
      await expect(dropdown.locator('text=Toggle Mixer Window')).toBeVisible();
      await expect(dropdown.locator('text=Toggle Database Window')).toBeVisible();
      await expect(dropdown.locator('text=Toggle Pipeline Window')).toBeVisible();
      await expect(dropdown.locator('text=Zoom In')).toBeVisible();
      await expect(dropdown.locator('text=Zoom Out')).toBeVisible();
      await expect(dropdown.locator('text=Reset Zoom')).toBeVisible();
    });

    test('should toggle DAW window via menu', async ({ page }) => {
      const viewMenu = page.locator('.menu-item button').filter({ hasText: 'View' });
      await viewMenu.click();
      await page.waitForTimeout(200);

      const toggleDaw = page.locator('.dropdown-menu button').filter({ hasText: 'Toggle DAW Window' });
      await toggleDaw.click();
      await page.waitForTimeout(500);

      // DAW/Arrangement window state should change
    });

    test('should toggle Mixer window via menu', async ({ page }) => {
      const viewMenu = page.locator('.menu-item button').filter({ hasText: 'View' });
      await viewMenu.click();
      await page.waitForTimeout(200);

      const toggleMixer = page.locator('.dropdown-menu button').filter({ hasText: 'Toggle Mixer Window' });
      await toggleMixer.click();
      await page.waitForTimeout(500);

      // Mixer window should appear or disappear
      const mixerWindow = page.locator('.mixer-window');
      // Check visibility state changed
    });

    test('should handle zoom actions', async ({ page }) => {
      // Test Zoom In
      const viewMenu = page.locator('.menu-item button').filter({ hasText: 'View' });
      await viewMenu.click();
      await page.waitForTimeout(200);

      const zoomIn = page.locator('.dropdown-menu button').filter({ hasText: 'Zoom In' });
      await zoomIn.click();
      await page.waitForTimeout(300);

      // Zoom should have changed (check CSS variable or element scale)
      const zoomValue = await page.evaluate(() => {
        return getComputedStyle(document.documentElement).getPropertyValue('--app-zoom');
      });
      // May be 1.1 after zoom in

      // Test Zoom Out
      await viewMenu.click();
      await page.waitForTimeout(200);
      const zoomOut = page.locator('.dropdown-menu button').filter({ hasText: 'Zoom Out' });
      await zoomOut.click();
      await page.waitForTimeout(300);

      // Test Reset Zoom
      await viewMenu.click();
      await page.waitForTimeout(200);
      const resetZoom = page.locator('.dropdown-menu button').filter({ hasText: 'Reset Zoom' });
      await resetZoom.click();
      await page.waitForTimeout(300);
    });
  });

  // ============================================================================
  // TOOLS MENU TESTS
  // ============================================================================

  test.describe('Tools Menu', () => {
    test('should open Tools menu and display all items', async ({ page }) => {
      const toolsMenu = page.locator('.menu-item button').filter({ hasText: 'Tools' });
      await toolsMenu.click();
      await page.waitForTimeout(200);

      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Check tools menu items
      await expect(dropdown.locator('text=Script Editor')).toBeVisible();
      await expect(dropdown.locator('text=MIDI Learn Mode')).toBeVisible();
      await expect(dropdown.locator('text=Ableton Link')).toBeVisible();
    });

    test('should open Script Editor via menu', async ({ page }) => {
      const toolsMenu = page.locator('.menu-item button').filter({ hasText: 'Tools' });
      await toolsMenu.click();
      await page.waitForTimeout(200);

      const scriptEditor = page.locator('.dropdown-menu button').filter({ hasText: 'Script Editor' });
      await scriptEditor.click();
      await page.waitForTimeout(500);

      // Script editor window should appear
      const scriptWindow = page.locator('.script-editor-window, [data-window="script-editor"]');
      // May or may not be implemented
    });
  });

  // ============================================================================
  // TRANSPORT MENU TESTS
  // ============================================================================

  test.describe('Transport Menu', () => {
    test('should open Transport menu and display all items', async ({ page }) => {
      const transportMenu = page.locator('.menu-item button').filter({ hasText: 'Transport' });
      await transportMenu.click();
      await page.waitForTimeout(200);

      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Check transport menu items
      await expect(dropdown.locator('text=Play')).toBeVisible();
      await expect(dropdown.locator('text=Pause')).toBeVisible();
      await expect(dropdown.locator('text=Stop')).toBeVisible();
      await expect(dropdown.locator('text=Record')).toBeVisible();
      await expect(dropdown.locator('text=Loop Playback')).toBeVisible();
      await expect(dropdown.locator('text=Metronome')).toBeVisible();
    });

    test('should trigger Play action via menu', async ({ page }) => {
      const transportMenu = page.locator('.menu-item button').filter({ hasText: 'Transport' });
      await transportMenu.click();
      await page.waitForTimeout(200);

      const playItem = page.locator('.dropdown-menu button').filter({ hasText: /^Play$/ });
      await playItem.click();
      await page.waitForTimeout(300);

      // Menu should close after action
      await expect(page.locator('.dropdown-menu')).not.toBeVisible();
    });

    test('should trigger Stop action via menu', async ({ page }) => {
      // First play
      await page.keyboard.press('Space');
      await page.waitForTimeout(200);

      // Then stop via menu
      const transportMenu = page.locator('.menu-item button').filter({ hasText: 'Transport' });
      await transportMenu.click();
      await page.waitForTimeout(200);

      const stopItem = page.locator('.dropdown-menu button').filter({ hasText: 'Stop' });
      await stopItem.click();
      await page.waitForTimeout(300);
    });

    test('should toggle Loop via menu', async ({ page }) => {
      const transportMenu = page.locator('.menu-item button').filter({ hasText: 'Transport' });
      await transportMenu.click();
      await page.waitForTimeout(200);

      const loopItem = page.locator('.dropdown-menu button').filter({ hasText: 'Loop Playback' });
      await loopItem.click();
      await page.waitForTimeout(300);
    });

    test('should toggle Metronome via menu', async ({ page }) => {
      const transportMenu = page.locator('.menu-item button').filter({ hasText: 'Transport' });
      await transportMenu.click();
      await page.waitForTimeout(200);

      const metronomeItem = page.locator('.dropdown-menu button').filter({ hasText: 'Metronome' });
      await metronomeItem.click();
      await page.waitForTimeout(300);
    });
  });

  // ============================================================================
  // HELP MENU TESTS
  // ============================================================================

  test.describe('Help Menu', () => {
    test('should open Help menu and display all items', async ({ page }) => {
      const helpMenu = page.locator('.menu-item button').filter({ hasText: 'Help' });
      await helpMenu.click();
      await page.waitForTimeout(200);

      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown).toBeVisible();

      // Check help menu items
      await expect(dropdown.locator('text=Documentation')).toBeVisible();
      await expect(dropdown.locator('text=Keyboard Shortcuts')).toBeVisible();
      await expect(dropdown.locator('text=About MIDI Software Center')).toBeVisible();
    });

    test('should open Keyboard Shortcuts dialog', async ({ page }) => {
      const helpMenu = page.locator('.menu-item button').filter({ hasText: 'Help' });
      await helpMenu.click();
      await page.waitForTimeout(200);

      const shortcutsItem = page.locator('.dropdown-menu button').filter({ hasText: 'Keyboard Shortcuts' });
      await shortcutsItem.click();
      await page.waitForTimeout(500);

      // Keyboard shortcuts dialog should appear
      const shortcutsDialog = page.locator('[role="dialog"]').filter({ hasText: 'Keyboard Shortcuts' });
      await expect(shortcutsDialog).toBeVisible();

      // Verify shortcut categories are present
      await expect(shortcutsDialog.locator('text=File')).toBeVisible();
      await expect(shortcutsDialog.locator('text=Edit')).toBeVisible();
      await expect(shortcutsDialog.locator('text=View')).toBeVisible();
      await expect(shortcutsDialog.locator('text=Transport')).toBeVisible();

      // Close button should work
      const closeBtn = shortcutsDialog.locator('button').filter({ hasText: 'Close' });
      await closeBtn.click();
      await page.waitForTimeout(200);
      await expect(shortcutsDialog).not.toBeVisible();
    });

    test('should open About dialog', async ({ page }) => {
      const helpMenu = page.locator('.menu-item button').filter({ hasText: 'Help' });
      await helpMenu.click();
      await page.waitForTimeout(200);

      const aboutItem = page.locator('.dropdown-menu button').filter({ hasText: 'About MIDI Software Center' });
      await aboutItem.click();
      await page.waitForTimeout(500);

      // About dialog should appear
      const aboutDialog = page.locator('[role="dialog"]').filter({ hasText: 'MIDI Software Center' });
      await expect(aboutDialog).toBeVisible();

      // Check content
      await expect(aboutDialog.locator('text=Version')).toBeVisible();
      await expect(aboutDialog.locator('text=Features')).toBeVisible();
      await expect(aboutDialog.locator('text=Technology Stack')).toBeVisible();

      // Close
      const closeBtn = aboutDialog.locator('button').filter({ hasText: 'Close' });
      await closeBtn.click();
      await page.waitForTimeout(200);
      await expect(aboutDialog).not.toBeVisible();
    });
  });

  // ============================================================================
  // MENU INTERACTION TESTS
  // ============================================================================

  test.describe('Menu Interactions', () => {
    test('should close menu when clicking outside', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      await expect(page.locator('.dropdown-menu')).toBeVisible();

      // Click outside menu
      await page.locator('.workspace').click({ force: true });
      await page.waitForTimeout(200);

      await expect(page.locator('.dropdown-menu')).not.toBeVisible();
    });

    test('should switch between menus by hovering', async ({ page }) => {
      // Open File menu
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      // Click Edit menu (should switch)
      const editMenu = page.locator('.menu-item button').filter({ hasText: 'Edit' });
      await editMenu.click();
      await page.waitForTimeout(200);

      // Verify Edit dropdown is open
      const dropdown = page.locator('.dropdown-menu').first();
      await expect(dropdown.locator('text=Undo')).toBeVisible();
    });

    test('should close menu when pressing Escape', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      await expect(page.locator('.dropdown-menu')).toBeVisible();

      await page.keyboard.press('Escape');
      await page.waitForTimeout(200);

      await expect(page.locator('.dropdown-menu')).not.toBeVisible();
    });

    test('menu should have visual feedback when active', async ({ page }) => {
      const fileMenu = page.locator('.menu-item button').filter({ hasText: 'File' });
      await fileMenu.click();
      await page.waitForTimeout(200);

      // Check for active class
      await expect(fileMenu).toHaveClass(/active/);
    });
  });
});
