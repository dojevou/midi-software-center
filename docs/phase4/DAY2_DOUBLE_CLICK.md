# Phase 4, Day 2: Double-Click Integration

**Time Estimate:** 2 hours
**Prerequisite:** Complete DAY1_FILE_LOADER.md

## Overview

This part integrates the file loader command with the VIP3 browser UI. Users will be able to double-click any file card in VIP3 results to instantly load it into the DAW sequencer.

**What we're building:**
- Double-click event handlers on file cards
- Toast notifications for success/error
- Optional: Auto-switch to DAW tab after loading
- Loading state indicators

**Why this matters:**
- Primary user workflow for VIP3 → DAW
- Intuitive interaction (double-click is discoverable)
- Immediate feedback for user actions

---

## Step 1: Toast Notification System

First, let's create a notification system for user feedback.

### File: `app/src/lib/stores/notificationStore.ts`

```typescript
import { writable } from 'svelte/store';

export type NotificationType = 'success' | 'error' | 'info' | 'warning';

export interface Notification {
  id: string;
  type: NotificationType;
  message: string;
  duration?: number; // milliseconds, default 3000
}

function createNotificationStore() {
  const { subscribe, update } = writable<Notification[]>([]);

  let nextId = 0;

  return {
    subscribe,

    add(type: NotificationType, message: string, duration = 3000) {
      const id = `notification-${nextId++}`;
      const notification: Notification = { id, type, message, duration };

      update((notifications) => [...notifications, notification]);

      if (duration > 0) {
        setTimeout(() => {
          this.remove(id);
        }, duration);
      }

      return id;
    },

    success(message: string, duration = 3000) {
      return this.add('success', message, duration);
    },

    error(message: string, duration = 5000) {
      return this.add('error', message, duration);
    },

    info(message: string, duration = 3000) {
      return this.add('info', message, duration);
    },

    warning(message: string, duration = 4000) {
      return this.add('warning', message, duration);
    },

    remove(id: string) {
      update((notifications) => notifications.filter((n) => n.id !== id));
    },

    clear() {
      update(() => []);
    },
  };
}

export const notifications = createNotificationStore();
```

---

## Step 2: Toast Component

### File: `app/src/lib/components/common/Toast.svelte`

```svelte
<script lang="ts">
  import { notifications, type Notification } from '$lib/stores/notificationStore';
  import { fade, fly } from 'svelte/transition';

  let toasts: Notification[] = [];

  notifications.subscribe((value) => {
    toasts = value;
  });

  function close(id: string) {
    notifications.remove(id);
  }

  function getIcon(type: string): string {
    switch (type) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
        return 'ℹ';
      default:
        return '';
    }
  }

  function getColorClass(type: string): string {
    switch (type) {
      case 'success':
        return 'bg-green-500';
      case 'error':
        return 'bg-red-500';
      case 'warning':
        return 'bg-yellow-500';
      case 'info':
        return 'bg-blue-500';
      default:
        return 'bg-gray-500';
    }
  }
</script>

<div class="toast-container">
  {#each toasts as toast (toast.id)}
    <div
      class="toast {getColorClass(toast.type)}"
      in:fly={{ y: -20, duration: 300 }}
      out:fade={{ duration: 200 }}
    >
      <span class="toast-icon">{getIcon(toast.type)}</span>
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" on:click={() => close(toast.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 20px;
    right: 20px;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    min-width: 300px;
    max-width: 500px;
    pointer-events: auto;
  }

  .toast-icon {
    font-size: 18px;
    font-weight: bold;
  }

  .toast-message {
    flex: 1;
  }

  .toast-close {
    background: none;
    border: none;
    color: white;
    font-size: 20px;
    cursor: pointer;
    padding: 0 4px;
    opacity: 0.8;
  }

  .toast-close:hover {
    opacity: 1;
  }
</style>
```

---

## Step 3: Update VIP3 FileCard Component

### Update: `app/src/lib/components/VIP3/FileCard.svelte`

Add double-click handler:

```svelte
<script lang="ts">
  import type { VIP3File } from '$lib/types/vip3';
  import { IntegrationApi } from '$lib/api/integrationApi';
  import { notifications } from '$lib/stores/notificationStore';
  import { goto } from '$app/navigation';

  export let file: VIP3File;
  export let selected = false;
  export let autoSwitchToDaw = true; // Option to auto-switch to DAW tab

  let isLoading = false;

  async function handleDoubleClick() {
    if (isLoading) return;

    isLoading = true;

    try {
      // Load file to DAW
      const trackId = await IntegrationApi.loadFileToDaw(file.id);

      // Show success notification
      notifications.success(`Loaded "${file.original_filename}" to DAW (Track ${trackId})`);

      // Optional: Switch to DAW tab
      if (autoSwitchToDaw) {
        goto('/daw');
      }
    } catch (error) {
      // Show error notification
      notifications.error(`Failed to load file: ${error}`);
      console.error('Load file error:', error);
    } finally {
      isLoading = false;
    }
  }

  function handleClick() {
    // Single click for selection (existing behavior)
    // Your existing click handler code
  }

  function formatDuration(ticks: number, ppq: number = 480): string {
    const seconds = (ticks / ppq) * 0.5; // Assuming 120 BPM
    const minutes = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div
  class="file-card"
  class:selected
  class:loading={isLoading}
  on:click={handleClick}
  on:dblclick={handleDoubleClick}
  role="button"
  tabindex="0"
  on:keydown={(e) => {
    if (e.key === 'Enter') handleDoubleClick();
  }}
>
  {#if isLoading}
    <div class="loading-overlay">
      <div class="spinner"></div>
      <span>Loading to DAW...</span>
    </div>
  {/if}

  <div class="file-card-content">
    <div class="file-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
      </svg>
    </div>

    <div class="file-info">
      <div class="file-name">{file.original_filename}</div>
      <div class="file-meta">
        {#if file.bpm}
          <span class="meta-item">{file.bpm.toFixed(0)} BPM</span>
        {/if}
        {#if file.key_signature}
          <span class="meta-item">{file.key_signature}</span>
        {/if}
        {#if file.duration_ticks}
          <span class="meta-item">{formatDuration(file.duration_ticks)}</span>
        {/if}
      </div>
    </div>
  </div>

  <div class="file-card-hint">
    Double-click to load to DAW
  </div>
</div>

<style>
  .file-card {
    position: relative;
    background: #2a2a2a;
    border: 2px solid transparent;
    border-radius: 8px;
    padding: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .file-card:hover {
    border-color: #4a9eff;
    background: #333;
  }

  .file-card.selected {
    border-color: #4a9eff;
    background: #3a3a3a;
  }

  .file-card.loading {
    opacity: 0.6;
    pointer-events: none;
  }

  .loading-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.7);
    border-radius: 8px;
    gap: 8px;
    color: white;
    font-size: 14px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .file-card-content {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .file-icon {
    color: #4a9eff;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-weight: 500;
    color: white;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    display: flex;
    gap: 8px;
    margin-top: 4px;
    font-size: 12px;
    color: #999;
  }

  .meta-item {
    display: flex;
    align-items: center;
  }

  .file-card-hint {
    margin-top: 8px;
    font-size: 11px;
    color: #666;
    text-align: center;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .file-card:hover .file-card-hint {
    opacity: 1;
  }
</style>
```

---

## Step 4: Add Toast Container to Layout

### Update: `app/src/routes/+layout.svelte`

```svelte
<script lang="ts">
  import Toast from '$lib/components/common/Toast.svelte';
  import '$lib/styles/global.css';
</script>

<div class="app-layout">
  <slot />
</div>

<!-- Toast notifications -->
<Toast />

<style>
  .app-layout {
    width: 100%;
    height: 100vh;
    overflow: hidden;
  }
</style>
```

---

## Step 5: Optional Tab Navigation

### Create: `app/src/lib/stores/navigationStore.ts`

```typescript
import { writable } from 'svelte/store';

export type TabId = 'pipeline' | 'vip3' | 'daw';

function createNavigationStore() {
  const { subscribe, set, update } = writable<TabId>('vip3');

  return {
    subscribe,
    setTab(tab: TabId) {
      set(tab);
    },
    goToDaw() {
      set('daw');
    },
    goToVip3() {
      set('vip3');
    },
    goToPipeline() {
      set('pipeline');
    },
  };
}

export const navigation = createNavigationStore();
```

### Update FileCard to use navigation store:

```svelte
<script lang="ts">
  import { navigation } from '$lib/stores/navigationStore';

  // ... existing code ...

  async function handleDoubleClick() {
    if (isLoading) return;

    isLoading = true;

    try {
      const trackId = await IntegrationApi.loadFileToDaw(file.id);
      notifications.success(`Loaded "${file.original_filename}" to DAW (Track ${trackId})`);

      if (autoSwitchToDaw) {
        navigation.goToDaw(); // Use store instead of goto()
      }
    } catch (error) {
      notifications.error(`Failed to load file: ${error}`);
      console.error('Load file error:', error);
    } finally {
      isLoading = false;
    }
  }
</script>
```

---

## Step 6: Batch Loading Support

Add ability to load multiple selected files:

### Update: `app/src/lib/components/VIP3/VIP3Results.svelte`

```svelte
<script lang="ts">
  import { IntegrationApi } from '$lib/api/integrationApi';
  import { notifications } from '$lib/stores/notificationStore';
  import FileCard from './FileCard.svelte';
  import type { VIP3File } from '$lib/types/vip3';

  export let files: VIP3File[] = [];
  export let selectedFiles: Set<number> = new Set();

  let isLoadingBatch = false;

  async function loadSelectedToDaw() {
    if (selectedFiles.size === 0) {
      notifications.warning('No files selected');
      return;
    }

    if (isLoadingBatch) return;

    isLoadingBatch = true;

    try {
      const fileIds = Array.from(selectedFiles);
      const results = await IntegrationApi.loadFilesToDaw(fileIds);

      const successCount = results.filter((r) => r.ok).length;
      const errorCount = results.length - successCount;

      if (errorCount === 0) {
        notifications.success(`Loaded ${successCount} file(s) to DAW`);
      } else {
        notifications.warning(`Loaded ${successCount} file(s), ${errorCount} failed`);
      }

      // Clear selection after loading
      selectedFiles.clear();
      selectedFiles = selectedFiles; // Trigger reactivity
    } catch (error) {
      notifications.error(`Failed to load files: ${error}`);
      console.error('Batch load error:', error);
    } finally {
      isLoadingBatch = false;
    }
  }
</script>

<div class="vip3-results">
  {#if selectedFiles.size > 0}
    <div class="batch-actions">
      <span class="selected-count">{selectedFiles.size} file(s) selected</span>
      <button class="btn-load-batch" on:click={loadSelectedToDaw} disabled={isLoadingBatch}>
        {#if isLoadingBatch}
          Loading...
        {:else}
          Load to DAW
        {/if}
      </button>
    </div>
  {/if}

  <div class="file-grid">
    {#each files as file (file.id)}
      <FileCard {file} selected={selectedFiles.has(file.id)} />
    {/each}
  </div>
</div>

<style>
  .vip3-results {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .batch-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: #2a2a2a;
    border-radius: 8px;
  }

  .selected-count {
    color: #4a9eff;
    font-weight: 500;
  }

  .btn-load-batch {
    padding: 8px 16px;
    background: #4a9eff;
    color: white;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-load-batch:hover:not(:disabled) {
    background: #3a8eef;
  }

  .btn-load-batch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }
</style>
```

---

## Step 7: Keyboard Shortcuts

Add Enter key to load selected file:

### Update: `app/src/lib/components/VIP3/VIP3Results.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';

  // ... existing code ...

  function handleKeydown(event: KeyboardEvent) {
    // Enter key loads selected files
    if (event.key === 'Enter' && selectedFiles.size > 0) {
      loadSelectedToDaw();
    }

    // Escape key clears selection
    if (event.key === 'Escape') {
      selectedFiles.clear();
      selectedFiles = selectedFiles;
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>
```

---

## Verification Checklist

### Manual Testing

1. **Test double-click load:**
   - Open VIP3 browser
   - Search for files
   - Double-click a file card
   - Verify success notification appears
   - Verify file loads to DAW (check DAW tab)

2. **Test error handling:**
   - Modify database to have invalid file path
   - Double-click file
   - Verify error notification appears

3. **Test loading indicator:**
   - Double-click file
   - Verify loading spinner appears
   - Verify card is disabled during load

4. **Test batch loading:**
   - Select multiple files (Ctrl+Click)
   - Click "Load to DAW" button
   - Verify all files load
   - Verify success notification

5. **Test keyboard shortcuts:**
   - Select files
   - Press Enter
   - Verify files load
   - Press Escape
   - Verify selection clears

6. **Test auto-switch to DAW:**
   - Double-click file in VIP3
   - Verify automatically switches to DAW tab
   - Verify track appears in sequencer

---

## Troubleshooting

### Issue: Double-click not triggering

**Symptom:**
Double-clicking file card does nothing

**Solution:**
1. Check browser console for errors
2. Verify `on:dblclick` handler is attached
3. Test in Chrome DevTools with breakpoint
4. Verify `IntegrationApi` is imported correctly

---

### Issue: Notification not appearing

**Symptom:**
File loads but no toast notification

**Solution:**
1. Verify `Toast` component is in layout
2. Check `notificationStore` is imported
3. Check browser console for errors
4. Verify CSS z-index on toast container

---

### Issue: Loading state stuck

**Symptom:**
Loading spinner doesn't disappear

**Solution:**
1. Check `finally` block executes
2. Verify `isLoading` is set to false
3. Check for uncaught promise rejections
4. Add timeout fallback:
```typescript
setTimeout(() => {
  isLoading = false;
}, 5000); // 5 second timeout
```

---

### Issue: Auto-switch not working

**Symptom:**
File loads but doesn't switch to DAW tab

**Solution:**
1. Check `navigationStore` is imported
2. Verify `navigation.goToDaw()` is called
3. Check router is configured correctly
4. Test with `autoSwitchToDaw = false` to isolate issue

---

## What's Next?

You've completed Day 2! Double-click integration is now functional.

**Next Steps:**
1. Move to [Day 3: Drag-and-Drop (Optional)](./DAY3_DRAG_DROP.md)
2. Implement drag-and-drop for even more intuitive workflow
3. Add drop zones and visual feedback

**What you've built:**
- ✅ Double-click file cards to load
- ✅ Toast notifications for success/error
- ✅ Loading state indicators
- ✅ Batch loading support
- ✅ Keyboard shortcuts (Enter, Escape)
- ✅ Auto-switch to DAW tab

**What's coming:**
- Drag file cards from VIP3 to sequencer
- Drop at specific positions in timeline
- Visual drag feedback and drop zones
