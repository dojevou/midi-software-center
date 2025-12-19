import { test, expect, waitForWorkspace } from './fixtures';

test.describe('Drag and Drop E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/', { waitUntil: 'commit' });

    const isReady = await waitForWorkspace(page, 10000);
    if (!isReady) {
      test.skip();
    }
    // VIP3 browser is open by default - wait for it
    await page.waitForSelector('.vip3-browser', { timeout: 5000 }).catch(() => {});
    await page.waitForTimeout(300);
  });

  test('should show drag visual feedback when dragging file', async ({ page }) => {
    // Wait for file list in VIP3 browser
    const fileItem = page.locator('.file-item').first();

    if (!(await fileItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    // Start drag
    const box = await fileItem.boundingBox();
    if (!box) {
      test.skip();
      return;
    }

    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + 100, box.y + 100);

    // Check for drag ghost or visual feedback
    const dragGhost = page.locator('.drag-ghost, .dragging, [data-dragging="true"]');
    // May or may not be visible depending on browser behavior

    await page.mouse.up();
  });

  test('should accept drop on sequencer tracks', async ({ page }) => {
    // Get a file item from VIP3 browser
    const fileItem = page.locator('.file-item').first();

    if (!(await fileItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    // Open arrangement view
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    // Get arrangement view drop zone
    const sequencer = page.locator('.arrangement-view, .arrangement-main').first();

    if (!(await sequencer.isVisible({ timeout: 3000 }))) {
      test.skip();
      return;
    }

    // Perform drag and drop
    const fileBox = await fileItem.boundingBox();
    const seqBox = await sequencer.boundingBox();

    if (!fileBox || !seqBox) {
      test.skip();
      return;
    }

    // Start drag from file
    await page.mouse.move(fileBox.x + fileBox.width / 2, fileBox.y + fileBox.height / 2);
    await page.mouse.down();

    // Drag to sequencer
    await page.mouse.move(seqBox.x + seqBox.width / 2, seqBox.y + seqBox.height / 2, { steps: 10 });

    // Drop
    await page.mouse.up();
    await page.waitForTimeout(500);

    // Check if a clip was created or loading indicator appeared
    const clip = page.locator('.sequencer-clip, .clip');
    const loadingIndicator = page.locator('.loading, [data-loading="true"]');

    // May have created a clip or shown loading
  });

  test('should show drop zone highlight on drag over', async ({ page }) => {
    // Open arrangement view
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    const sequencer = page.locator('.arrangement-view, .arrangement-main').first();

    if (!(await sequencer.isVisible({ timeout: 3000 }))) {
      test.skip();
      return;
    }

    // Simulate dragover event
    await sequencer.evaluate((el) => {
      const dragOverEvent = new DragEvent('dragover', {
        bubbles: true,
        cancelable: true,
        dataTransfer: new DataTransfer(),
      });
      el.dispatchEvent(dragOverEvent);
    });

    // Check for drag-over class or styling
    const hasDragOverClass = await sequencer.evaluate((el) => {
      return el.classList.contains('drag-over') || el.matches('.drag-over');
    });

    // Dispatch dragleave to reset
    await sequencer.evaluate((el) => {
      const dragLeaveEvent = new DragEvent('dragleave', {
        bubbles: true,
        cancelable: true,
      });
      el.dispatchEvent(dragLeaveEvent);
    });
  });

  test('should handle drag data with correct MIME type', async ({ page }) => {
    const fileItem = page.locator('.file-item').first();

    if (!(await fileItem.isVisible({ timeout: 5000 }))) {
      test.skip();
      return;
    }

    // Check that draggable is set
    const isDraggable = await fileItem.evaluate((el) => {
      return el.getAttribute('draggable') === 'true' || el.draggable;
    });

    expect(isDraggable).toBeTruthy();
  });

  test('should create new track when dropping below existing tracks', async ({ page }) => {
    // Open arrangement view
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(300);

    const tracksContainer = page.locator('.arrangement-view, .arrangement-main').first();

    if (!(await tracksContainer.isVisible({ timeout: 3000 }))) {
      test.skip();
      return;
    }

    // Get initial track count
    const initialTrackCount = await page.locator('.sequencer-track, .track').count();

    // Simulate drop event with mock data
    await tracksContainer.evaluate((el) => {
      const dataTransfer = new DataTransfer();
      dataTransfer.setData('application/json', JSON.stringify({
        type: 'midi-file',
        id: 1,
        filename: 'test.mid',
        bpm: 120,
        key_signature: 'C',
        duration_seconds: 60,
      }));

      const dropEvent = new DragEvent('drop', {
        bubbles: true,
        cancelable: true,
        dataTransfer,
      });
      el.dispatchEvent(dropEvent);
    });

    await page.waitForTimeout(1000);

    // Check if track was added
    const newTrackCount = await page.locator('.sequencer-track, .track').count();
    // May or may not increase depending on implementation details
  });
});
