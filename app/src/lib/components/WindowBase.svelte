<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { uiStore, uiActions } from '$lib/stores/uiStore';
  import type { WindowId, WindowPosition } from '$lib/types';
  import { DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT } from '$lib/utils/constants';

  export let windowId: WindowId;
  export let title: string;
  export let width: number = DEFAULT_WINDOW_WIDTH;
  export let height: number = DEFAULT_WINDOW_HEIGHT;
  export let minWidth: number = MIN_WINDOW_WIDTH;
  export let minHeight: number = MIN_WINDOW_HEIGHT;
  export let resizable: boolean = true;
  export let closable: boolean = true;

  let position: WindowPosition;
  let isDragging = false;
  let isResizing = false;
  let dragStart: { x: number; y: number; offsetX: number; offsetY: number } | null = null;
  let resizeStart: { x: number; y: number; offsetWidth: number; offsetHeight: number } | null = null;
  let currentZIndex = 0;

  // Subscribe to uiStore for this window
  $: position = $uiStore.windows[windowId] || { x: 50, y: 50, width, height, z_index: 1, visible: true };
  $: currentZIndex = position.z_index;

  // Update store on position/size changes
  $: if (position) {
    uiActions.setWindowPosition(windowId, position.x, position.y);
    uiActions.setWindowSize(windowId, position.width, position.height);
  }

  // Bring to front on mount or click
  onMount(() => {
    uiActions.bringToFront(windowId);
    uiActions.showWindow(windowId);
  });

  function handleMouseDownTitle(event: MouseEvent) {
    if (event.button !== 0) return; // Left click only
    isDragging = true;
    dragStart = {
      x: event.clientX,
      y: event.clientY,
      offsetX: event.clientX - position.x,
      offsetY: event.clientY - position.y
    };
    uiActions.bringToFront(windowId);
    event.preventDefault();
  }

  function handleMouseDownResize(event: MouseEvent) {
    if (event.button !== 0) return;
    isResizing = true;
    resizeStart = {
      x: event.clientX,
      y: event.clientY,
      offsetWidth: position.width,
      offsetHeight: position.height
    };
    event.preventDefault();
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging && !isResizing) return;

    if (isDragging && dragStart) {
      let newX = event.clientX - dragStart.offsetX;
      let newY = event.clientY - dragStart.offsetY;

      // Clamp to viewport
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      newX = Math.max(0, Math.min(newX, viewportWidth - position.width));
      newY = Math.max(0, Math.min(newY, viewportHeight - position.height));

      position.x = newX;
      position.y = newY;
    }

    if (isResizing && resizeStart) {
      const deltaX = event.clientX - resizeStart.x;
      const deltaY = event.clientY - resizeStart.y;

      let newWidth = resizeStart.offsetWidth + deltaX;
      let newHeight = resizeStart.offsetHeight + deltaY;

      // Enforce min sizes
      newWidth = Math.max(minWidth, newWidth);
      newHeight = Math.max(minHeight, newHeight);

      position.width = newWidth;
      position.height = newHeight;
    }
  }

  function handleMouseUp() {
    isDragging = false;
    isResizing = false;
    dragStart = null;
    resizeStart = null;
  }

  // Global event listeners for drag/resize
  onMount(() => {
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };
  });

  function closeWindow() {
    uiActions.hideWindow(windowId);
  }

  function minimizeWindow() {
    // Toggle visibility for minimize
    position.visible = !position.visible;
    uiActions.showWindow(windowId); // or hide based on logic
  }

  function maximizeWindow() {
    // Implement maximize logic if needed
    console.log('Maximize not fully implemented');
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<div
  class="window-base dark:bg-window dark:border-window-border dark:text-app-text"
  class:dragging={isDragging}
  class:resizing={isResizing}
  style="
    left: {position.x}px;
    top: {position.y}px;
    width: {position.width}px;
    height: {position.height}px;
    z-index: {currentZIndex};
    display: {position.visible ? 'block' : 'none'};
  "
  on:click={() => uiActions.bringToFront(windowId)}
>
  <!-- Title Bar -->
  <div
    class="window-title dark:bg-menu flex items-center justify-between px-3 py-2 cursor-move select-none"
    on:mousedown={handleMouseDownTitle}
  >
    <span class="font-medium">{title}</span>
    <div class="flex space-x-1">
      <button
        class="minimize-btn dark:bg-transparent dark:text-gray-400 hover:dark:text-app-text w-6 h-6 flex items-center justify-center rounded"
        on:click|stopPropagation={minimizeWindow}
        title="Minimize"
      >
        ─
      </button>
      <button
        class="maximize-btn dark:bg-transparent dark:text-gray-400 hover:dark:text-app-text w-6 h-6 flex items-center justify-center rounded"
        on:click|stopPropagation={maximizeWindow}
        title="Maximize"
      >
        ⬜
      </button>
      {#if closable}
        <button
          class="close-btn dark:bg-transparent dark:text-gray-400 hover:dark:text-red-400 w-6 h-6 flex items-center justify-center rounded"
          on:click|stopPropagation={closeWindow}
          title="Close"
        >
          ×
        </button>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div class="window-content dark:bg-window p-2 overflow-auto flex-1">
    <slot />
  </div>

  <!-- Resize Handle -->
  {#if resizable}
    <div
      class="resize-handle dark:bg-window-border absolute bottom-0 right-0 w-4 h-4 cursor-se-resize"
      on:mousedown={handleMouseDownResize}
    />
  {/if}
</div>

<style>
  .window-base {
    position: fixed;
    border: 1px solid var(--window-border);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-width: 200px;
    min-height: 150px;
  }

  .window-title {
    -webkit-user-select: none;
    -moz-user-select: none;
    user-select: none;
    border-bottom: 1px solid var(--window-border);
  }

  .window-content {
    flex: 1;
    min-height: 0;
  }

  .dragging,
  .resizing {
    user-select: none;
  }

  .resize-handle {
    background-image: linear-gradient(-45deg, transparent 0%, transparent 46%, var(--window-border) 46%, var(--window-border) 50%, transparent 50%, transparent 100%);
    background-size: 8px 8px;
  }

  .minimize-btn:hover,
  .maximize-btn:hover,
  .close-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }
</style>