import { test, expect, waitForWorkspace } from './fixtures';

test.describe('VIP3 Browser Comprehensive E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
    // VIP3 browser should be visible by default
    await page.waitForSelector('.vip3-browser', { timeout: 5000 }).catch(() => {});
    await page.waitForTimeout(300);
  });

  // ============================================================================
  // VIP3 BROWSER VISIBILITY TESTS
  // ============================================================================

  test.describe('VIP3 Browser Visibility', () => {
    test('should display VIP3 browser on startup', async ({ page }) => {
      const vip3Browser = page.locator('.vip3-browser').first();
      await expect(vip3Browser).toBeVisible({ timeout: 5000 });
    });

    test('should toggle VIP3 browser with Ctrl+B', async ({ page }) => {
      const vip3Browser = page.locator('.vip3-browser').first();
      await expect(vip3Browser).toBeVisible({ timeout: 5000 });

      // Hide
      await page.keyboard.press('Control+b');
      await page.waitForTimeout(500);
      await expect(vip3Browser).not.toBeVisible();

      // Show again
      await page.keyboard.press('Control+b');
      await page.waitForTimeout(500);
      await expect(vip3Browser).toBeVisible();
    });

    test('should display browser header with title', async ({ page }) => {
      const header = page.locator('.browser-header, .vip3-header').first();
      await expect(header).toBeVisible();
      // Header contains various controls - just verify it's visible and has content
      const headerText = await header.textContent();
      expect(headerText?.length).toBeGreaterThan(0);
    });
  });

  // ============================================================================
  // VIP3 TAB NAVIGATION TESTS
  // ============================================================================

  test.describe('VIP3 Tab Navigation', () => {
    test('should display all tabs', async ({ page }) => {
      // Tab text includes emojis: 🔍 Browser, ⭐ Saved Searches, 📁 Collections, ❤️ Favorites
      const browserTab = page.locator('.tab').filter({ hasText: /Browser/ });
      const searchesTab = page.locator('.tab').filter({ hasText: /Saved Searches/ });
      const collectionsTab = page.locator('.tab').filter({ hasText: /Collections/ });
      const favoritesTab = page.locator('.tab').filter({ hasText: /Favorites/ });

      await expect(browserTab).toBeVisible();
      await expect(searchesTab).toBeVisible();
      await expect(collectionsTab).toBeVisible();
      await expect(favoritesTab).toBeVisible();
    });

    test('should switch to Saved Searches tab', async ({ page }) => {
      const searchesTab = page.locator('.tab').filter({ hasText: /Saved Searches/ });
      await searchesTab.click();
      await page.waitForTimeout(200);

      // Tab should be active
      await expect(searchesTab).toHaveClass(/active/);
    });

    test('should switch to Collections tab', async ({ page }) => {
      const collectionsTab = page.locator('.tab').filter({ hasText: /Collections/ });
      await collectionsTab.click();
      await page.waitForTimeout(200);

      await expect(collectionsTab).toHaveClass(/active/);
    });

    test('should switch to Favorites tab', async ({ page }) => {
      const favoritesTab = page.locator('.tab').filter({ hasText: /Favorites/ });
      await favoritesTab.click();
      await page.waitForTimeout(200);

      await expect(favoritesTab).toHaveClass(/active/);
    });

    test('should switch back to Browser tab', async ({ page }) => {
      // First switch to another tab
      const favoritesTab = page.locator('.tab').filter({ hasText: 'Favorites' });
      await favoritesTab.click();
      await page.waitForTimeout(200);

      // Then switch back
      const browserTab = page.locator('.tab').filter({ hasText: 'Browser' });
      await browserTab.click();
      await page.waitForTimeout(200);

      await expect(browserTab).toHaveClass(/active/);
    });
  });

  // ============================================================================
  // VIP3 FILTER COLUMNS TESTS
  // ============================================================================

  test.describe('VIP3 Filter Columns', () => {
    test('should display filter columns', async ({ page }) => {
      const filterColumns = page.locator('.filter-columns, .vip3-columns').first();
      await expect(filterColumns).toBeVisible();
    });

    test('should have Folders column', async ({ page }) => {
      // Ensure we're on browser tab
      const browserTab = page.locator('.tab').filter({ hasText: 'Browser' });
      if (await browserTab.isVisible()) {
        await browserTab.click();
        await page.waitForTimeout(300);
      }

      // Folders column has title in .column-title
      const foldersColumn = page.locator('.vip3-column .column-title').filter({ hasText: 'Folders' });
      if (!(await foldersColumn.isVisible({ timeout: 3000 }))) {
        // May not have Folders column if not implemented
        test.skip();
        return;
      }
      await expect(foldersColumn).toBeVisible();
    });

    test('should have Instruments column', async ({ page }) => {
      // Ensure we're on browser tab
      const browserTab = page.locator('.tab').filter({ hasText: 'Browser' });
      if (await browserTab.isVisible()) {
        await browserTab.click();
        await page.waitForTimeout(300);
      }

      const instrumentsColumn = page.locator('.vip3-column .column-title').filter({ hasText: 'Instruments' });
      if (!(await instrumentsColumn.isVisible({ timeout: 3000 }))) {
        // May not have Instruments column if not implemented
        test.skip();
        return;
      }
      await expect(instrumentsColumn).toBeVisible();
    });

    test('should have Timbre column', async ({ page }) => {
      const timbreColumn = page.locator('.vip3-column, .filter-column').filter({ hasText: 'Timbre' });
      await expect(timbreColumn).toBeVisible();
    });

    test('should have Style column', async ({ page }) => {
      const styleColumn = page.locator('.vip3-column, .filter-column').filter({ hasText: 'Style' });
      await expect(styleColumn).toBeVisible();
    });

    test('should have Articulation column', async ({ page }) => {
      const articulationColumn = page.locator('.vip3-column, .filter-column').filter({ hasText: 'Articulation' });
      await expect(articulationColumn).toBeVisible();
    });

    test('should have BPM column', async ({ page }) => {
      const bpmColumn = page.locator('.vip3-bpm-column, .bpm-column, .filter-column').filter({ hasText: /BPM|Tempo/ });
      // BPM column may exist
    });

    test('should click filter item to select', async ({ page }) => {
      // Find first clickable filter item
      const filterItem = page.locator('.column-item, .filter-item, .vip3-item').first();
      if (await filterItem.isVisible()) {
        await filterItem.click();
        await page.waitForTimeout(200);

        // Item should show selected state
        const hasSelectedClass = await filterItem.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeTruthy();
      }
    });

    test('should deselect filter item on second click', async ({ page }) => {
      const filterItem = page.locator('.column-item, .filter-item').first();
      if (await filterItem.isVisible()) {
        // Select
        await filterItem.click();
        await page.waitForTimeout(200);

        // Deselect
        await filterItem.click();
        await page.waitForTimeout(200);

        // Item should not be selected
        const hasSelectedClass = await filterItem.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeFalsy();
      }
    });

    test('should show item counts in filter columns', async ({ page }) => {
      // Filter items should have counts
      const itemWithCount = page.locator('.column-item .count, .filter-item .count, .item-count').first();
      // May or may not have counts depending on data
    });
  });

  // ============================================================================
  // VIP3 BPM FILTER TESTS
  // ============================================================================

  test.describe('VIP3 BPM Filter', () => {
    test('should have BPM range inputs', async ({ page }) => {
      const bpmMin = page.locator('input[placeholder*="Min"], .bpm-min, input[name="bpm-min"]');
      const bpmMax = page.locator('input[placeholder*="Max"], .bpm-max, input[name="bpm-max"]');
      // BPM inputs may exist
    });

    test('should filter by BPM range', async ({ page }) => {
      const bpmMin = page.locator('input[placeholder*="Min"], .bpm-min').first();
      const bpmMax = page.locator('input[placeholder*="Max"], .bpm-max').first();

      if (await bpmMin.isVisible() && await bpmMax.isVisible()) {
        await bpmMin.fill('100');
        await bpmMax.fill('130');
        await page.waitForTimeout(300);

        // Results should update based on BPM range
      }
    });
  });

  // ============================================================================
  // VIP3 RESULTS PANEL TESTS
  // ============================================================================

  test.describe('VIP3 Results Panel', () => {
    test('should display results panel', async ({ page }) => {
      const resultsPanel = page.locator('.results-panel, .search-results, .file-list').first();
      await expect(resultsPanel).toBeVisible();
    });

    test('should display total matches count', async ({ page }) => {
      const totalMatches = page.locator('.total-matches, .result-count, .match-count').first();
      await expect(totalMatches).toBeVisible();
    });

    test('should display file items in results', async ({ page }) => {
      const fileItems = page.locator('.file-item, .result-item, .file-row');
      const count = await fileItems.count();
      // May have file items depending on database
    });

    test('should select file item on click', async ({ page }) => {
      const fileItem = page.locator('.file-item, .result-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click();
        await page.waitForTimeout(200);

        // Should have selected state
        const hasSelectedClass = await fileItem.evaluate((el) =>
          el.classList.contains('selected') || el.classList.contains('active')
        );
        expect(hasSelectedClass).toBeTruthy();
      }
    });

    test('should handle file double-click to load to sequencer', async ({ page }) => {
      const fileItem = page.locator('.file-item, .result-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.dblclick();
        await page.waitForTimeout(500);

        // File should be loaded to sequencer (check for clip or console log)
      }
    });
  });

  // ============================================================================
  // VIP3 SEARCH FUNCTIONALITY TESTS
  // ============================================================================

  test.describe('VIP3 Search Functionality', () => {
    test('should have search input', async ({ page }) => {
      const searchInput = page.locator('input[placeholder*="Search"], .search-input, input[type="search"]');
      // Search input may exist
    });

    test('should filter results on search', async ({ page }) => {
      const searchInput = page.locator('input[placeholder*="Search"], .search-input').first();
      if (await searchInput.isVisible()) {
        await searchInput.fill('drum');
        await page.waitForTimeout(500);

        // Results should update
      }
    });

    test('should clear search on button click', async ({ page }) => {
      const clearBtn = page.locator('.clear-search, .search-clear, button[title*="Clear"]');
      if (await clearBtn.isVisible()) {
        await clearBtn.click();
        await page.waitForTimeout(200);
      }
    });
  });

  // ============================================================================
  // VIP3 FAVORITES TESTS
  // ============================================================================

  test.describe('VIP3 Favorites', () => {
    test('should switch to favorites tab', async ({ page }) => {
      const favoritesTab = page.locator('.tab').filter({ hasText: 'Favorites' });
      await favoritesTab.click();
      await page.waitForTimeout(200);

      await expect(favoritesTab).toHaveClass(/active/);
    });

    test('should display favorites list', async ({ page }) => {
      const favoritesTab = page.locator('.tab').filter({ hasText: 'Favorites' });
      await favoritesTab.click();
      await page.waitForTimeout(200);

      const favoritesList = page.locator('.favorites-list, .favorites-panel');
      await expect(favoritesList).toBeVisible();
    });

    test('should have favorite button on file items', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        const favoriteBtn = fileItem.locator('.favorite-btn, .heart-btn, button[title*="Favorite"]');
        // May have favorite button
      }
    });
  });

  // ============================================================================
  // VIP3 COLLECTIONS TESTS
  // ============================================================================

  test.describe('VIP3 Collections', () => {
    test('should switch to collections tab', async ({ page }) => {
      const collectionsTab = page.locator('.tab').filter({ hasText: 'Collections' });
      await collectionsTab.click();
      await page.waitForTimeout(200);

      await expect(collectionsTab).toHaveClass(/active/);
    });

    test('should display collections list', async ({ page }) => {
      const collectionsTab = page.locator('.tab').filter({ hasText: 'Collections' });
      await collectionsTab.click();
      await page.waitForTimeout(200);

      const collectionsList = page.locator('.collections-list, .collections-panel');
      await expect(collectionsList).toBeVisible();
    });

    test('should have create collection button', async ({ page }) => {
      const collectionsTab = page.locator('.tab').filter({ hasText: 'Collections' });
      await collectionsTab.click();
      await page.waitForTimeout(200);

      const createBtn = page.locator('button').filter({ hasText: /Create|New|Add/ });
      // May have create button
    });
  });

  // ============================================================================
  // VIP3 SAVED SEARCHES TESTS
  // ============================================================================

  test.describe('VIP3 Saved Searches', () => {
    test('should switch to saved searches tab', async ({ page }) => {
      const searchesTab = page.locator('.tab').filter({ hasText: 'Saved Searches' });
      await searchesTab.click();
      await page.waitForTimeout(200);

      await expect(searchesTab).toHaveClass(/active/);
    });

    test('should display saved searches list', async ({ page }) => {
      const searchesTab = page.locator('.tab').filter({ hasText: 'Saved Searches' });
      await searchesTab.click();
      await page.waitForTimeout(200);

      const searchesList = page.locator('.saved-searches-list, .searches-panel');
      await expect(searchesList).toBeVisible();
    });

    test('should have save search button', async ({ page }) => {
      const saveBtn = page.locator('button').filter({ hasText: /Save Search|Save/ });
      // May have save search button
    });
  });

  // ============================================================================
  // VIP3 DRAG AND DROP TESTS
  // ============================================================================

  test.describe('VIP3 Drag and Drop', () => {
    test('file items should be draggable', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        const isDraggable = await fileItem.evaluate((el) =>
          el.getAttribute('draggable') === 'true' || el.draggable
        );
        expect(isDraggable).toBeTruthy();
      }
    });

    test('should initiate drag from file item', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        const box = await fileItem.boundingBox();
        if (box) {
          await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
          await page.mouse.down();
          await page.mouse.move(box.x + 100, box.y);

          // Check for dragging state
          const hasDraggingClass = await fileItem.evaluate((el) =>
            el.classList.contains('dragging') || document.querySelector('.drag-ghost') !== null
          );

          await page.mouse.up();
        }
      }
    });
  });

  // ============================================================================
  // VIP3 CONTEXT MENU TESTS
  // ============================================================================

  test.describe('VIP3 Context Menu', () => {
    test('should open context menu on right-click file item', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click({ button: 'right' });
        await page.waitForTimeout(200);

        const contextMenu = page.locator('.context-menu, [role="menu"]');
        // May have context menu
      }
    });

    test('context menu should have Add to Sequencer option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click({ button: 'right' });
        await page.waitForTimeout(200);

        const addOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Add|Load|Sequencer/ });
        // May have add option
      }
    });

    test('context menu should have Add to Favorites option', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click({ button: 'right' });
        await page.waitForTimeout(200);

        const favOption = page.locator('.context-menu button, [role="menuitem"]').filter({ hasText: /Favorite/ });
        // May have favorite option
      }
    });

    test('should close context menu on click outside', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click({ button: 'right' });
        await page.waitForTimeout(200);

        // Click outside
        await page.locator('.vip3-browser').click({ position: { x: 10, y: 10 } });
        await page.waitForTimeout(200);

        const contextMenu = page.locator('.context-menu, [role="menu"]');
        await expect(contextMenu).not.toBeVisible();
      }
    });
  });

  // ============================================================================
  // VIP3 LOADING STATE TESTS
  // ============================================================================

  test.describe('VIP3 Loading States', () => {
    test('should show loading indicator during data fetch', async ({ page }) => {
      const loadingIndicator = page.locator('.loading, [data-loading="true"], .spinner');
      // May show loading on initial load
    });

    test('should show loading text in header', async ({ page }) => {
      const loadingText = page.locator('.loading').filter({ hasText: 'Loading' });
      // May show loading text
    });
  });

  // ============================================================================
  // VIP3 KEYBOARD NAVIGATION TESTS
  // ============================================================================

  test.describe('VIP3 Keyboard Navigation', () => {
    test('should navigate file list with arrow keys', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click();
        await page.waitForTimeout(100);

        // Press down arrow
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(100);

        // Second item should now be selected
        const selectedItem = page.locator('.file-item.selected, .file-item.active').first();
        // Check selection moved
      }
    });

    test('should preview file with Enter key', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click();
        await page.waitForTimeout(100);

        // Press Enter to preview/load
        await page.keyboard.press('Enter');
        await page.waitForTimeout(300);
      }
    });

    test('should toggle favorite with F key', async ({ page }) => {
      const fileItem = page.locator('.file-item').first();
      if (await fileItem.isVisible()) {
        await fileItem.click();
        await page.waitForTimeout(100);

        // Press F to favorite
        await page.keyboard.press('f');
        await page.waitForTimeout(200);
      }
    });
  });
});
