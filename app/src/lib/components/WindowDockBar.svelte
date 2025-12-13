<script lang="ts">
  import { uiStore, uiActions } from '$lib/stores/uiStore';
  import type { WindowId } from '$lib/types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    restore: { windowId: WindowId };
    close: { windowId: WindowId };
  }>();

  // Window icon mapping
  const windowIcons: Record<string, string> = {
    daw: '\u{1F3B9}',
    mixer: '\u{1F39A}',
    database: '\u{1F4C1}',
    pipeline: '\u{2699}',
    'midi-io-setup': '\u{1F50C}',
    'midi-monitor': '\u{1F4CA}',
    preferences: '\u{2699}',
    'gear-manager': '\u{1F527}',
    'presets-manager': '\u{1F4BE}',
    'piano-roll': '\u{1F3B9}',
    'tag-editor': '\u{1F3F7}',
    'vip3-browser': '\u{1F4C2}',
    favorites: '\u{2B50}',
    'file-details': '\u{1F4C4}',
    'loop-browser': '\u{1F501}',
    'project-browser': '\u{1F4C1}',
    'command-palette': '\u{2318}',
    export: '\u{1F4E4}',
    settings: '\u{2699}',
  };

  // Window display names
  const windowNames: Record<string, string> = {
    daw: 'DAW',
    mixer: 'Mixer',
    database: 'Database',
    pipeline: 'Pipeline',
    'midi-io-setup': 'I/O Setup',
    'midi-monitor': 'Monitor',
    preferences: 'Preferences',
    'gear-manager': 'Gear Manager',
    'presets-manager': 'Presets',
    'piano-roll': 'Piano Roll',
    'tag-editor': 'Tags',
    'vip3-browser': 'VIP3 Browser',
    favorites: 'Favorites',
    'file-details': 'Details',
    'loop-browser': 'Loops',
    'project-browser': 'Projects',
    'command-palette': 'Commands',
    export: 'Export',
    settings: 'Settings',
  };

  // Get minimized windows from store
  $: minimizedWindows = $uiStore.minimizedWindows || [];

  // Drag reorder state
  let draggedWindow: WindowId | null = null;
  let dragOverWindow: WindowId | null = null;

  function handleRestore(windowId: WindowId) {
    uiActions.restoreWindow(windowId);
    dispatch('restore', { windowId });
  }

  function handleClose(windowId: WindowId, event: MouseEvent) {
    event.stopPropagation();
    uiActions.closeMinimizedWindow(windowId);
    dispatch('close', { windowId });
  }

  function handleDragStart(event: DragEvent, windowId: WindowId) {
    draggedWindow = windowId;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', windowId);
    }
  }

  function handleDragOver(event: DragEvent, windowId: WindowId) {
    event.preventDefault();
    if (draggedWindow && draggedWindow !== windowId) {
      dragOverWindow = windowId;
    }
  }

  function handleDragLeave() {
    dragOverWindow = null;
  }

  function handleDrop(event: DragEvent, targetWindowId: WindowId) {
    event.preventDefault();
    if (draggedWindow && draggedWindow !== targetWindowId) {
      uiActions.reorderMinimizedWindows(draggedWindow, targetWindowId);
    }
    draggedWindow = null;
    dragOverWindow = null;
  }

  function handleDragEnd() {
    draggedWindow = null;
    dragOverWindow = null;
  }

  function getIcon(windowId: WindowId): string {
    return windowIcons[windowId] || '\u{1F5D4}';
  }

  function getName(windowId: WindowId): string {
    return windowNames[windowId] || windowId;
  }
</script>

{#if minimizedWindows.length > 0}
  <div class="dock-bar" role="toolbar" aria-label="Minimized windows dock">
    {#each minimizedWindows as windowId (windowId)}
      <button
        class="dock-item"
        class:drag-over={dragOverWindow === windowId}
        class:dragging={draggedWindow === windowId}
        draggable="true"
        on:click={() => handleRestore(windowId)}
        on:dragstart={(e) => handleDragStart(e, windowId)}
        on:dragover={(e) => handleDragOver(e, windowId)}
        on:dragleave={handleDragLeave}
        on:drop={(e) => handleDrop(e, windowId)}
        on:dragend={handleDragEnd}
        title="Click to restore {getName(windowId)}"
        aria-label="Restore {getName(windowId)} window"
      >
        <span class="dock-icon">{getIcon(windowId)}</span>
        <span class="dock-label">{getName(windowId)}</span>
        <button
          class="dock-close"
          on:click={(e) => handleClose(windowId, e)}
          title="Close {getName(windowId)}"
          aria-label="Close {getName(windowId)}"
        >
          <svg viewBox="0 0 10 10" width="10" height="10">
            <line x1="2" y1="2" x2="8" y2="8" stroke="currentColor" stroke-width="1.5"/>
            <line x1="8" y1="2" x2="2" y2="8" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      </button>
    {/each}
  </div>
{/if}

<style>
  .dock-bar {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 4px;
    padding: 6px 8px;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--window-border, #444);
    border-radius: 8px;
    box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.3);
    z-index: 9999;
    max-width: calc(100vw - 40px);
    overflow-x: auto;
  }

  .dock-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--window-border, #444);
    border-radius: 6px;
    color: var(--text-primary, #fff);
    cursor: pointer;
    transition: all 0.15s ease;
    white-space: nowrap;
    font-size: 12px;
  }

  .dock-item:hover {
    background: var(--bg-hover, #333);
    border-color: var(--primary, #00d4aa);
  }

  .dock-item:active {
    transform: scale(0.98);
  }

  .dock-item.dragging {
    opacity: 0.5;
  }

  .dock-item.drag-over {
    border-color: var(--primary, #00d4aa);
    box-shadow: 0 0 0 2px var(--primary, #00d4aa);
  }

  .dock-icon {
    font-size: 14px;
  }

  .dock-label {
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dock-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    margin-left: 4px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 3px;
    color: var(--text-secondary, #888);
    cursor: pointer;
    opacity: 0;
    transition: all 0.15s ease;
  }

  .dock-item:hover .dock-close {
    opacity: 1;
  }

  .dock-close:hover {
    background: #e81123;
    color: white;
  }

  .dock-bar::-webkit-scrollbar {
    height: 4px;
  }

  .dock-bar::-webkit-scrollbar-track {
    background: transparent;
  }

  .dock-bar::-webkit-scrollbar-thumb {
    background: var(--window-border, #444);
    border-radius: 2px;
  }
</style>
