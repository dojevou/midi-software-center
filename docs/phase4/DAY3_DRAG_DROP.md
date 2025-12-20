# Phase 4, Day 3: Drag-and-Drop (Optional)

**Time Estimate:** 2-3 hours
**Prerequisite:** Complete DAY2_DOUBLE_CLICK.md
**Status:** Optional but recommended

## Overview

This part implements drag-and-drop functionality, allowing users to drag file cards from VIP3 browser and drop them directly onto the DAW sequencer timeline. Files can be dropped at specific time positions for precise placement.

**What we're building:**
- Draggable file cards with HTML5 Drag API
- Drop zones in sequencer timeline
- Visual feedback during drag (cursor, drop indicator)
- Position calculation from drop coordinates
- Multi-file drag support

**Why this matters:**
- More intuitive than double-click for power users
- Allows precise positioning in timeline
- Professional DAW workflow expectation
- Supports multi-file drag operations

---

## Step 1: Drag Utility Functions

### File: `app/src/lib/utils/dragDrop.ts`

```typescript
export interface DragData {
  type: 'midi-file' | 'multiple-files';
  fileId?: number;
  fileIds?: number[];
  fileName?: string;
}

export class DragDropHelper {
  private static readonly MIME_TYPE = 'application/x-midi-file';

  /**
   * Set drag data for a single file
   */
  static setFileDragData(event: DragEvent, fileId: number, fileName: string) {
    if (!event.dataTransfer) return;

    const data: DragData = {
      type: 'midi-file',
      fileId,
      fileName,
    };

    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData(this.MIME_TYPE, JSON.stringify(data));
    event.dataTransfer.setData('text/plain', fileName);
  }

  /**
   * Set drag data for multiple files
   */
  static setMultipleFilesDragData(event: DragEvent, fileIds: number[], fileNames: string[]) {
    if (!event.dataTransfer) return;

    const data: DragData = {
      type: 'multiple-files',
      fileIds,
    };

    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData(this.MIME_TYPE, JSON.stringify(data));
    event.dataTransfer.setData('text/plain', `${fileIds.length} files`);
  }

  /**
   * Get drag data from drop event
   */
  static getFileDragData(event: DragEvent): DragData | null {
    if (!event.dataTransfer) return null;

    try {
      const jsonData = event.dataTransfer.getData(this.MIME_TYPE);
      if (!jsonData) return null;

      return JSON.parse(jsonData) as DragData;
    } catch (error) {
      console.error('Failed to parse drag data:', error);
      return null;
    }
  }

  /**
   * Check if drag event contains MIDI file(s)
   */
  static hasMidiFileData(event: DragEvent): boolean {
    if (!event.dataTransfer) return false;
    return event.dataTransfer.types.includes(this.MIME_TYPE);
  }

  /**
   * Calculate timeline position from drop coordinates
   * @param dropX - X coordinate of drop event
   * @param timelineWidth - Width of timeline in pixels
   * @param timelineDuration - Duration of timeline in ticks
   * @returns Position in ticks
   */
  static calculateDropPosition(
    dropX: number,
    timelineWidth: number,
    timelineDuration: number
  ): number {
    const ratio = dropX / timelineWidth;
    return Math.floor(ratio * timelineDuration);
  }

  /**
   * Snap position to nearest beat/bar
   * @param positionTicks - Position in ticks
   * @param ppq - Pulses per quarter note
   * @param snapTo - 'beat' | 'bar' | 'half-beat' | 'none'
   * @param timeSignature - Time signature (default 4/4)
   * @returns Snapped position in ticks
   */
  static snapPosition(
    positionTicks: number,
    ppq: number,
    snapTo: 'beat' | 'bar' | 'half-beat' | 'none' = 'beat',
    timeSignature: [number, number] = [4, 4]
  ): number {
    if (snapTo === 'none') return positionTicks;

    let snapInterval: number;

    switch (snapTo) {
      case 'half-beat':
        snapInterval = ppq / 2;
        break;
      case 'beat':
        snapInterval = ppq;
        break;
      case 'bar':
        snapInterval = ppq * timeSignature[0];
        break;
    }

    return Math.round(positionTicks / snapInterval) * snapInterval;
  }
}
```

---

## Step 2: Make File Cards Draggable

### Update: `app/src/lib/components/VIP3/FileCard.svelte`

```svelte
<script lang="ts">
  import { DragDropHelper } from '$lib/utils/dragDrop';
  // ... existing imports ...

  export let file: VIP3File;
  export let draggable = true; // Option to disable dragging
  // ... existing props ...

  let isDragging = false;

  function handleDragStart(event: DragEvent) {
    if (!draggable) {
      event.preventDefault();
      return;
    }

    isDragging = true;

    // Set drag data
    DragDropHelper.setFileDragData(event, file.id, file.original_filename);

    // Optional: Set drag image (custom appearance while dragging)
    if (event.dataTransfer) {
      const dragImage = event.currentTarget as HTMLElement;
      const clone = dragImage.cloneNode(true) as HTMLElement;
      clone.style.opacity = '0.8';
      clone.style.transform = 'rotate(-2deg)';
      document.body.appendChild(clone);
      event.dataTransfer.setDragImage(clone, 20, 20);

      // Clean up cloned element after drag
      setTimeout(() => {
        document.body.removeChild(clone);
      }, 0);
    }
  }

  function handleDragEnd() {
    isDragging = false;
  }

  // ... existing handlers ...
</script>

<div
  class="file-card"
  class:selected
  class:loading={isLoading}
  class:dragging={isDragging}
  {draggable}
  on:dragstart={handleDragStart}
  on:dragend={handleDragEnd}
  on:click={handleClick}
  on:dblclick={handleDoubleClick}
  role="button"
  tabindex="0"
>
  <!-- ... existing content ... -->

  <div class="drag-indicator" class:visible={isDragging}>
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
      <path d="M10 3a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm0 4a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm0 4a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm-4-8a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm0 4a1 1 0 1 1 2 0 1 1 0 0 1-2 0zm0 4a1 1 0 1 1 2 0 1 1 0 0 1-2 0z"/>
    </svg>
  </div>
</div>

<style>
  /* ... existing styles ... */

  .file-card.dragging {
    opacity: 0.5;
    cursor: grabbing;
  }

  .file-card[draggable='true'] {
    cursor: grab;
  }

  .drag-indicator {
    position: absolute;
    top: 8px;
    right: 8px;
    color: #4a9eff;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .file-card:hover .drag-indicator {
    opacity: 0.6;
  }

  .drag-indicator.visible {
    opacity: 1;
  }
</style>
```

---

## Step 3: Sequencer Drop Zone

### Update: `app/src/lib/components/DAW/Sequencer.svelte`

```svelte
<script lang="ts">
  import { IntegrationApi } from '$lib/api/integrationApi';
  import { notifications } from '$lib/stores/notificationStore';
  import { DragDropHelper } from '$lib/utils/dragDrop';
  // ... existing imports ...

  let timelineElement: HTMLElement;
  let isDragOver = false;
  let dropIndicatorPosition: number | null = null; // Position in pixels

  const ppq = 480; // Pulses per quarter note
  const timelineDuration = 19200; // 40 bars at 480 PPQ
  const snapMode = 'beat'; // 'beat' | 'bar' | 'half-beat' | 'none'

  function handleDragOver(event: DragEvent) {
    if (!DragDropHelper.hasMidiFileData(event)) return;

    event.preventDefault();
    isDragOver = true;

    // Show drop indicator at mouse position
    if (event.currentTarget instanceof HTMLElement) {
      const rect = event.currentTarget.getBoundingClientRect();
      dropIndicatorPosition = event.clientX - rect.left;
    }

    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDragLeave(event: DragEvent) {
    // Only hide if actually leaving the drop zone
    if (event.currentTarget === event.target) {
      isDragOver = false;
      dropIndicatorPosition = null;
    }
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
    dropIndicatorPosition = null;

    const dragData = DragDropHelper.getFileDragData(event);
    if (!dragData) {
      notifications.error('Invalid drag data');
      return;
    }

    // Calculate drop position
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const dropX = event.clientX - rect.left;
    const timelineWidth = rect.width;

    let positionTicks = DragDropHelper.calculateDropPosition(
      dropX,
      timelineWidth,
      timelineDuration
    );

    // Snap to grid
    positionTicks = DragDropHelper.snapPosition(positionTicks, ppq, snapMode);

    // Load file(s) at position
    try {
      if (dragData.type === 'midi-file' && dragData.fileId) {
        // Single file
        const trackId = await IntegrationApi.loadFileToDaw(dragData.fileId);

        // Set track position
        await setTrackPosition(trackId, positionTicks);

        notifications.success(
          `Loaded "${dragData.fileName}" at ${formatPosition(positionTicks)}`
        );
      } else if (dragData.type === 'multiple-files' && dragData.fileIds) {
        // Multiple files
        const results = await IntegrationApi.loadFilesToDaw(dragData.fileIds);

        let successCount = 0;
        let currentPosition = positionTicks;

        for (const result of results) {
          if (result.ok) {
            await setTrackPosition(result.value as number, currentPosition);
            currentPosition += ppq * 4; // Space files 1 bar apart
            successCount++;
          }
        }

        notifications.success(`Loaded ${successCount} file(s) starting at ${formatPosition(positionTicks)}`);
      }
    } catch (error) {
      notifications.error(`Failed to load file: ${error}`);
      console.error('Drop error:', error);
    }
  }

  async function setTrackPosition(trackId: number, positionTicks: number) {
    // Call your existing set_track_position command
    await invoke('set_track_position', { trackId, positionTicks });
  }

  function formatPosition(ticks: number): string {
    const bar = Math.floor(ticks / (ppq * 4)) + 1;
    const beat = Math.floor((ticks % (ppq * 4)) / ppq) + 1;
    return `Bar ${bar}, Beat ${beat}`;
  }
</script>

<div
  class="sequencer-timeline"
  class:drag-over={isDragOver}
  bind:this={timelineElement}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="region"
  aria-label="Sequencer timeline drop zone"
>
  <!-- Existing timeline content -->

  {#if dropIndicatorPosition !== null}
    <div class="drop-indicator" style="left: {dropIndicatorPosition}px">
      <div class="drop-line"></div>
      <div class="drop-label">Drop here</div>
    </div>
  {/if}

  <!-- ... existing tracks, playhead, etc. ... -->
</div>

<style>
  .sequencer-timeline {
    position: relative;
    width: 100%;
    height: 100%;
    background: #1a1a1a;
    transition: background 0.2s;
  }

  .sequencer-timeline.drag-over {
    background: #2a2a3a;
    outline: 2px dashed #4a9eff;
    outline-offset: -4px;
  }

  .drop-indicator {
    position: absolute;
    top: 0;
    bottom: 0;
    pointer-events: none;
    z-index: 1000;
  }

  .drop-line {
    width: 2px;
    height: 100%;
    background: #4a9eff;
    box-shadow: 0 0 8px rgba(74, 158, 255, 0.6);
  }

  .drop-label {
    position: absolute;
    top: 8px;
    left: 4px;
    padding: 4px 8px;
    background: #4a9eff;
    color: white;
    font-size: 11px;
    border-radius: 4px;
    white-space: nowrap;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }
</style>
```

---

## Step 4: Multi-File Drag Support

### Update: `app/src/lib/components/VIP3/VIP3Results.svelte`

Add multi-file drag support when files are selected:

```svelte
<script lang="ts">
  import { DragDropHelper } from '$lib/utils/dragDrop';
  // ... existing imports ...

  export let selectedFiles: Set<number> = new Set();

  function handleBatchDragStart(event: DragEvent) {
    if (selectedFiles.size === 0) {
      event.preventDefault();
      return;
    }

    const fileIds = Array.from(selectedFiles);
    const fileNames = files
      .filter((f) => selectedFiles.has(f.id))
      .map((f) => f.original_filename);

    DragDropHelper.setMultipleFilesDragData(event, fileIds, fileNames);
  }
</script>

<div class="vip3-results">
  {#if selectedFiles.size > 0}
    <div
      class="batch-drag-handle"
      draggable="true"
      on:dragstart={handleBatchDragStart}
    >
      <span class="drag-icon">⋮⋮</span>
      <span>Drag {selectedFiles.size} selected file(s) to DAW</span>
    </div>
  {/if}

  <!-- ... existing file grid ... -->
</div>

<style>
  .batch-drag-handle {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    background: #2a2a2a;
    border: 2px dashed #4a9eff;
    border-radius: 8px;
    cursor: grab;
    transition: all 0.2s;
  }

  .batch-drag-handle:hover {
    background: #333;
    border-color: #5aafff;
  }

  .batch-drag-handle:active {
    cursor: grabbing;
  }

  .drag-icon {
    color: #4a9eff;
    font-weight: bold;
  }
</style>
```

---

## Step 5: Drop Zones in Track Lanes

Add drop zones to individual track lanes for re-ordering:

### Create: `app/src/lib/components/DAW/TrackLane.svelte`

```svelte
<script lang="ts">
  import { DragDropHelper } from '$lib/utils/dragDrop';
  import type { Track } from '$lib/types/daw';

  export let track: Track;
  export let index: number;

  let isDragOver = false;

  function handleDragOver(event: DragEvent) {
    if (!DragDropHelper.hasMidiFileData(event)) return;

    event.preventDefault();
    isDragOver = true;

    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;

    const dragData = DragDropHelper.getFileDragData(event);
    if (!dragData || dragData.type !== 'midi-file' || !dragData.fileId) {
      return;
    }

    // Replace this track with dropped file
    try {
      await invoke('replace_track', {
        trackId: track.id,
        fileId: dragData.fileId,
      });

      notifications.success(`Replaced track ${index + 1} with "${dragData.fileName}"`);
    } catch (error) {
      notifications.error(`Failed to replace track: ${error}`);
    }
  }
</script>

<div
  class="track-lane"
  class:drag-over={isDragOver}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
>
  <div class="track-header">
    <span class="track-number">{index + 1}</span>
    <span class="track-name">{track.name}</span>
  </div>

  <!-- Track content, waveform, MIDI events, etc. -->

  {#if isDragOver}
    <div class="drop-overlay">
      <span>Drop to replace track</span>
    </div>
  {/if}
</div>

<style>
  .track-lane {
    position: relative;
    height: 80px;
    border-bottom: 1px solid #333;
    background: #1a1a1a;
    transition: background 0.2s;
  }

  .track-lane.drag-over {
    background: #2a2a3a;
    outline: 2px solid #4a9eff;
    outline-offset: -2px;
  }

  .track-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    background: #222;
  }

  .track-number {
    width: 24px;
    text-align: center;
    color: #666;
    font-size: 12px;
  }

  .track-name {
    flex: 1;
    color: white;
    font-weight: 500;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(74, 158, 255, 0.1);
    color: #4a9eff;
    font-weight: 500;
    pointer-events: none;
  }
</style>
```

---

## Step 6: Drag Feedback and Cursor

Add custom cursor and visual feedback:

### File: `app/src/lib/styles/drag.css`

```css
/* Drag cursors */
[draggable='true'] {
  cursor: grab;
  user-select: none;
}

[draggable='true']:active {
  cursor: grabbing;
}

/* Drag ghost (element being dragged) */
.dragging {
  opacity: 0.5 !important;
}

/* Drop zones */
.drop-zone {
  transition: background 0.2s, outline 0.2s;
}

.drop-zone.drag-over {
  outline: 2px dashed #4a9eff;
  outline-offset: -4px;
}

/* Drop indicator line */
.drop-indicator {
  position: absolute;
  width: 2px;
  background: #4a9eff;
  box-shadow: 0 0 8px rgba(74, 158, 255, 0.8);
  pointer-events: none;
  z-index: 9999;
}

/* Drop not allowed */
.drop-not-allowed {
  cursor: not-allowed;
}

.drop-not-allowed * {
  cursor: not-allowed !important;
}
```

Import in `app/src/lib/styles/global.css`:

```css
@import './drag.css';
```

---

## Verification Checklist

### Manual Testing

1. **Test single file drag:**
   - Drag file card from VIP3
   - Drop on sequencer timeline
   - Verify file loads at drop position
   - Verify position snaps to beat

2. **Test multi-file drag:**
   - Select 3 files in VIP3
   - Drag batch handle to sequencer
   - Drop on timeline
   - Verify all files load with spacing

3. **Test drop indicator:**
   - Drag file over timeline
   - Verify drop indicator line appears
   - Verify position label shows bar/beat
   - Move mouse and verify indicator follows

4. **Test track lane drop:**
   - Drag file to specific track lane
   - Drop on lane
   - Verify replaces track content

5. **Test drag feedback:**
   - Start dragging file
   - Verify cursor changes to grabbing
   - Verify card opacity reduces
   - Verify drag icon appears

6. **Test snap modes:**
   - Set snap to 'beat'
   - Drop file at various positions
   - Verify snaps to nearest beat
   - Test 'bar', 'half-beat', 'none' modes

---

## Troubleshooting

### Issue: Drag not working

**Symptom:**
File card not draggable

**Solution:**
1. Verify `draggable="true"` attribute
2. Check `handleDragStart` is attached
3. Verify drag data is set correctly
4. Check browser console for errors

---

### Issue: Drop not working

**Symptom:**
Drop event doesn't fire

**Solution:**
1. Must call `event.preventDefault()` in `dragover`
2. Verify `on:drop` handler attached
3. Check drop zone has correct event handlers
4. Test with Chrome DevTools drag events

---

### Issue: Drop indicator not appearing

**Symptom:**
No visual feedback during drag over timeline

**Solution:**
1. Check `isDragOver` reactivity
2. Verify `dropIndicatorPosition` calculation
3. Check CSS z-index on indicator
4. Verify timeline element has correct dimensions

---

### Issue: Position calculation incorrect

**Symptom:**
File loads at wrong position

**Solution:**
1. Check `timelineWidth` is accurate
2. Verify `timelineDuration` matches sequencer
3. Test snap function with console.log
4. Check for scrolling offset issues

---

## What's Next?

You've completed Day 3! Drag-and-drop is now functional.

**Next Steps:**
1. Move to [Day 4: Integration Testing](./DAY4_TESTING.md)
2. Test complete VIP3 → DAW workflows
3. Verify all edge cases

**What you've built:**
- ✅ Draggable file cards
- ✅ Drop zones in timeline
- ✅ Drop position calculation with snapping
- ✅ Visual drag feedback (cursor, indicator)
- ✅ Multi-file drag support
- ✅ Track lane drop zones

**What's coming:**
- End-to-end integration tests
- Performance verification
- Edge case handling
- Complete workflow validation
