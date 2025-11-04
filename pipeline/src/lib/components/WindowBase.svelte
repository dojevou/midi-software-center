<script lang="ts">
  /**
   * WindowBase - Reusable window component
   *
   * Task-O-Matic: Draggable, resizable window with minimize/restore.
   * Integrates with UI store for position persistence.
   */

  import { onMount, onDestroy } from 'svelte';
  import { uiStore, uiActions, type WindowId } from '$lib/stores/uiStore';

  // Props
  export let windowId: WindowId;
  export let title: string;
  export let onClose: (() => void) | null = null;

  // Local state
  let windowElement: HTMLDivElement;
  let isDragging = false;
  let isResizing = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let resizeStartWidth = 0;
  let resizeStartHeight = 0;

  // Reactive window state from store
  $: position = $uiStore.windowPositions.get(windowId);
  $: isVisible = $uiStore.windowVisibility.get(windowId) || false;
  $: isActive = $uiStore.activeWindow === windowId;
  $: isMinimized = position?.minimized || false;

  // Drag handlers
  function handleMouseDownTitle(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.window-controls')) {
      return; // Don't drag when clicking controls
    }

    isDragging = true;
    dragStartX = e.clientX - (position?.x || 0);
    dragStartY = e.clientY - (position?.y || 0);

    uiActions.setActiveWindow(windowId);

    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (isDragging && position) {
      const newX = e.clientX - dragStartX;
      const newY = e.clientY - dragStartY;
      uiActions.setWindowPosition(windowId, newX, newY);
    } else if (isResizing && position) {
      const newWidth = resizeStartWidth + (e.clientX - dragStartX);
      const newHeight = resizeStartHeight + (e.clientY - dragStartY);
      uiActions.setWindowSize(
        windowId,
        Math.max(400, newWidth),
        Math.max(300, newHeight)
      );
    }
  }

  function handleMouseUp() {
    isDragging = false;
    isResizing = false;
  }

  // Resize handlers
  function handleMouseDownResize(e: MouseEvent) {
    isResizing = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    resizeStartWidth = position?.width || 800;
    resizeStartHeight = position?.height || 600;

    e.preventDefault();
  }

  // Window controls
  function handleMinimize() {
    if (isMinimized) {
      uiActions.restoreWindow(windowId);
    } else {
      uiActions.minimizeWindow(windowId);
    }
  }

  function handleClose() {
    if (onClose) {
      onClose();
    } else {
      uiActions.hideWindow(windowId);
    }
  }

  function handleClickWindow() {
    if (!isActive) {
      uiActions.setActiveWindow(windowId);
    }
  }

  // Mount/cleanup
  onMount(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  });

  onDestroy(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  });
</script>

{#if isVisible && position}
  <div
    bind:this={windowElement}
    class="window-base"
    class:active={isActive}
    class:minimized={isMinimized}
    style="
      left: {position.x}px;
      top: {position.y}px;
      width: {position.width}px;
      height: {position.height}px;
      z-index: {position.zIndex};
    "
    on:mousedown={handleClickWindow}
    role="dialog"
    aria-label={title}
  >
    <!-- Title Bar -->
    <div
      class="window-titlebar"
      on:mousedown={handleMouseDownTitle}
      role="button"
      tabindex="0"
    >
      <div class="window-title">{title}</div>
      <div class="window-controls">
        <button
          class="window-control minimize"
          on:click={handleMinimize}
          aria-label={isMinimized ? 'Restore' : 'Minimize'}
        >
          {isMinimized ? '□' : '–'}
        </button>
        <button
          class="window-control close"
          on:click={handleClose}
          aria-label="Close"
        >
          ×
        </button>
      </div>
    </div>

    <!-- Content -->
    {#if !isMinimized}
      <div class="window-content">
        <slot />
      </div>

      <!-- Resize Handle -->
      <div
        class="resize-handle"
        on:mousedown={handleMouseDownResize}
        role="button"
        tabindex="0"
        aria-label="Resize"
      />
    {/if}
  </div>
{/if}

<style>
  .window-base {
    position: fixed;
    background: var(--window-bg, #1e1e1e);
    border: 1px solid var(--window-border, #3e3e3e);
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transition: opacity 0.2s ease;
  }

  .window-base.minimized {
    height: auto !important;
  }

  .window-base.active {
    border-color: var(--accent-color, #0078d4);
  }

  .window-titlebar {
    background: var(--titlebar-bg, #2d2d2d);
    color: var(--titlebar-text, #e0e0e0);
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    cursor: move;
    user-select: none;
    border-bottom: 1px solid var(--window-border, #3e3e3e);
  }

  .window-base.active .window-titlebar {
    background: var(--titlebar-active-bg, #3a3a3a);
  }

  .window-title {
    font-size: 13px;
    font-weight: 500;
    flex: 1;
  }

  .window-controls {
    display: flex;
    gap: 8px;
  }

  .window-control {
    background: none;
    border: none;
    color: var(--titlebar-text, #e0e0e0);
    font-size: 16px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s ease;
  }

  .window-control:hover {
    background: var(--control-hover, rgba(255, 255, 255, 0.1));
  }

  .window-control.close:hover {
    background: #e81123;
    color: white;
  }

  .window-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
    background: var(--window-bg, #1e1e1e);
  }

  .resize-handle {
    position: absolute;
    bottom: 0;
    right: 0;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    background: linear-gradient(
      135deg,
      transparent 0%,
      transparent 50%,
      var(--resize-handle, #666) 50%,
      var(--resize-handle, #666) 60%,
      transparent 60%,
      transparent 100%
    );
  }

  .resize-handle:hover {
    background: linear-gradient(
      135deg,
      transparent 0%,
      transparent 50%,
      var(--accent-color, #0078d4) 50%,
      var(--accent-color, #0078d4) 60%,
      transparent 60%,
      transparent 100%
    );
  }

  /* Scrollbar styling */
  .window-content::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .window-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .window-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }

  .window-content::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-thumb-hover, #5e5e5e);
  }
</style>
