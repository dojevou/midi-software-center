<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';
  import { uiActions, uiStore } from '$lib/stores/uiStore';
  import type { WindowId, WindowPosition } from '$lib/types';
  import {
    DEFAULT_WINDOW_HEIGHT,
    DEFAULT_WINDOW_WIDTH,
    MIN_WINDOW_HEIGHT,
    MIN_WINDOW_WIDTH,
  } from '$lib/utils/constants';

  export let windowId: WindowId;
  export let title: string;
  export let width: number = DEFAULT_WINDOW_WIDTH;
  export let height: number = DEFAULT_WINDOW_HEIGHT;
  export let minWidth: number = MIN_WINDOW_WIDTH;
  export let minHeight: number = MIN_WINDOW_HEIGHT;
  export let resizable: boolean = true;
  export let closable: boolean = true;
  export let minimizable: boolean = true;
  export let maximizable: boolean = true;
  export let snappable: boolean = true;
  export let dockable: boolean = true;

  const dispatch = createEventDispatcher<{
    minimize: { windowId: WindowId };
    maximize: { windowId: WindowId };
    close: { windowId: WindowId };
    dock: { windowId: WindowId; zone: DockZone };
    focus: { windowId: WindowId };
    tabRequest: { sourceWindowId: WindowId; targetWindowId: WindowId };
  }>();

  // Snap and dock configuration
  const SNAP_THRESHOLD = 20; // pixels
  const DOCK_ZONE_SIZE = 50; // pixels from edge for dock detection

  type DockZone = 'none' | 'left' | 'right' | 'top' | 'bottom' | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  type ResizeDirection = 'se' | 'sw' | 'ne' | 'nw' | 'n' | 's' | 'e' | 'w';

  let position: WindowPosition;
  let isDragging = false;
  let isResizing = false;
  let resizeDirection: ResizeDirection = 'se';
  let dragStart: { x: number; y: number; offsetX: number; offsetY: number } | null = null;
  let resizeStart: {
    x: number;
    y: number;
    windowX: number;
    windowY: number;
    windowWidth: number;
    windowHeight: number
  } | null = null;
  let currentZIndex = 0;
  let isMaximized = false;
  let isMinimized = false;
  let savedPosition: WindowPosition | null = null;
  let currentDockZone: DockZone = 'none';
  let showDockPreview = false;
  let dockPreviewZone: DockZone = 'none';
  let lastClickTime = 0;
  let showTabDropZone = false;
  let potentialTabTarget: WindowId | null = null;

  // Subscribe to uiStore for this window
  $: position = $uiStore.windows[windowId] || {
    x: 50,
    y: 50,
    width,
    height,
    z_index: 1,
    visible: true,
  };
  $: currentZIndex = position.z_index;

  // Update store on position/size changes
  $: if (position && !isMinimized) {
    uiActions.setWindowPosition(windowId, position.x, position.y);
    uiActions.setWindowSize(windowId, position.width, position.height);
  }

  // Bring to front on mount
  onMount(() => {
    uiActions.bringToFront(windowId);
    uiActions.showWindow(windowId);
  });

  // Get all other window positions for snapping
  function getOtherWindowBounds(): Array<{ x: number; y: number; width: number; height: number }> {
    const bounds: Array<{ x: number; y: number; width: number; height: number }> = [];
    for (const [id, pos] of Object.entries($uiStore.windows)) {
      if (id !== windowId && pos.visible) {
        bounds.push({ x: pos.x, y: pos.y, width: pos.width, height: pos.height });
      }
    }
    return bounds;
  }

  // Calculate snap position
  function calculateSnap(x: number, y: number, w: number, h: number): { x: number; y: number } {
    if (!snappable) {return { x, y };}

    let snappedX = x;
    let snappedY = y;

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const otherWindows = getOtherWindowBounds();

    // Snap to viewport edges
    if (Math.abs(x) < SNAP_THRESHOLD) {snappedX = 0;}
    if (Math.abs(y) < SNAP_THRESHOLD) {snappedY = 0;}
    if (Math.abs(x + w - viewportWidth) < SNAP_THRESHOLD) {snappedX = viewportWidth - w;}
    if (Math.abs(y + h - viewportHeight) < SNAP_THRESHOLD) {snappedY = viewportHeight - h;}

    // Snap to other windows
    for (const other of otherWindows) {
      // Snap to left edge of other window
      if (Math.abs(x + w - other.x) < SNAP_THRESHOLD) {snappedX = other.x - w;}
      // Snap to right edge of other window
      if (Math.abs(x - (other.x + other.width)) < SNAP_THRESHOLD) {snappedX = other.x + other.width;}
      // Snap to top edge of other window
      if (Math.abs(y + h - other.y) < SNAP_THRESHOLD) {snappedY = other.y - h;}
      // Snap to bottom edge of other window
      if (Math.abs(y - (other.y + other.height)) < SNAP_THRESHOLD) {snappedY = other.y + other.height;}

      // Align edges
      if (Math.abs(x - other.x) < SNAP_THRESHOLD) {snappedX = other.x;}
      if (Math.abs(y - other.y) < SNAP_THRESHOLD) {snappedY = other.y;}
      if (Math.abs(x + w - (other.x + other.width)) < SNAP_THRESHOLD) {snappedX = other.x + other.width - w;}
      if (Math.abs(y + h - (other.y + other.height)) < SNAP_THRESHOLD) {snappedY = other.y + other.height - h;}
    }

    return { x: snappedX, y: snappedY };
  }

  // Detect dock zone based on mouse position
  function detectDockZone(clientX: number, clientY: number): DockZone {
    if (!dockable) {return 'none';}

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    const isLeft = clientX < DOCK_ZONE_SIZE;
    const isRight = clientX > viewportWidth - DOCK_ZONE_SIZE;
    const isTop = clientY < DOCK_ZONE_SIZE;
    const isBottom = clientY > viewportHeight - DOCK_ZONE_SIZE;

    if (isTop && isLeft) {return 'top-left';}
    if (isTop && isRight) {return 'top-right';}
    if (isBottom && isLeft) {return 'bottom-left';}
    if (isBottom && isRight) {return 'bottom-right';}
    if (isLeft) {return 'left';}
    if (isRight) {return 'right';}
    if (isTop) {return 'top';}
    if (isBottom) {return 'bottom';}

    return 'none';
  }

  // Apply dock position
  function applyDock(zone: DockZone) {
    if (zone === 'none') {return;}

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const menuBarHeight = 32; // Account for menu bar
    const statusBarHeight = 24; // Account for status bar
    const availableHeight = viewportHeight - menuBarHeight - statusBarHeight;

    // Save current position before docking
    if (!savedPosition) {
      savedPosition = { ...position };
    }

    switch (zone) {
      case 'left':
        position.x = 0;
        position.y = menuBarHeight;
        position.width = viewportWidth / 2;
        position.height = availableHeight;
        break;
      case 'right':
        position.x = viewportWidth / 2;
        position.y = menuBarHeight;
        position.width = viewportWidth / 2;
        position.height = availableHeight;
        break;
      case 'top':
        position.x = 0;
        position.y = menuBarHeight;
        position.width = viewportWidth;
        position.height = availableHeight / 2;
        break;
      case 'bottom':
        position.x = 0;
        position.y = menuBarHeight + availableHeight / 2;
        position.width = viewportWidth;
        position.height = availableHeight / 2;
        break;
      case 'top-left':
        position.x = 0;
        position.y = menuBarHeight;
        position.width = viewportWidth / 2;
        position.height = availableHeight / 2;
        break;
      case 'top-right':
        position.x = viewportWidth / 2;
        position.y = menuBarHeight;
        position.width = viewportWidth / 2;
        position.height = availableHeight / 2;
        break;
      case 'bottom-left':
        position.x = 0;
        position.y = menuBarHeight + availableHeight / 2;
        position.width = viewportWidth / 2;
        position.height = availableHeight / 2;
        break;
      case 'bottom-right':
        position.x = viewportWidth / 2;
        position.y = menuBarHeight + availableHeight / 2;
        position.width = viewportWidth / 2;
        position.height = availableHeight / 2;
        break;
    }

    currentDockZone = zone;
    dispatch('dock', { windowId, zone });
  }

  // Get dock preview bounds
  function getDockPreviewBounds(zone: DockZone): { x: number; y: number; width: number; height: number } | null {
    if (zone === 'none') {return null;}

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const menuBarHeight = 32;
    const statusBarHeight = 24;
    const availableHeight = viewportHeight - menuBarHeight - statusBarHeight;

    switch (zone) {
      case 'left':
        return { x: 0, y: menuBarHeight, width: viewportWidth / 2, height: availableHeight };
      case 'right':
        return { x: viewportWidth / 2, y: menuBarHeight, width: viewportWidth / 2, height: availableHeight };
      case 'top':
        return { x: 0, y: menuBarHeight, width: viewportWidth, height: availableHeight / 2 };
      case 'bottom':
        return { x: 0, y: menuBarHeight + availableHeight / 2, width: viewportWidth, height: availableHeight / 2 };
      case 'top-left':
        return { x: 0, y: menuBarHeight, width: viewportWidth / 2, height: availableHeight / 2 };
      case 'top-right':
        return { x: viewportWidth / 2, y: menuBarHeight, width: viewportWidth / 2, height: availableHeight / 2 };
      case 'bottom-left':
        return { x: 0, y: menuBarHeight + availableHeight / 2, width: viewportWidth / 2, height: availableHeight / 2 };
      case 'bottom-right':
        return { x: viewportWidth / 2, y: menuBarHeight + availableHeight / 2, width: viewportWidth / 2, height: availableHeight / 2 };
      default:
        return null;
    }
  }

  function handleMouseDownTitle(event: MouseEvent) {
    if (event.button !== 0) {return;}

    // Double-click to maximize
    const now = Date.now();
    if (now - lastClickTime < 300) {
      maximizeWindow();
      lastClickTime = 0;
      return;
    }
    lastClickTime = now;

    isDragging = true;
    dragStart = {
      x: event.clientX,
      y: event.clientY,
      offsetX: event.clientX - position.x,
      offsetY: event.clientY - position.y,
    };
    uiActions.bringToFront(windowId);
    dispatch('focus', { windowId });
    event.preventDefault();
  }

  function handleMouseDownResize(event: MouseEvent, direction: ResizeDirection) {
    if (event.button !== 0) {return;}
    isResizing = true;
    resizeDirection = direction;
    resizeStart = {
      x: event.clientX,
      y: event.clientY,
      windowX: position.x,
      windowY: position.y,
      windowWidth: position.width,
      windowHeight: position.height,
    };
    event.preventDefault();
    event.stopPropagation();
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging && !isResizing) {return;}

    if (isDragging && dragStart) {
      let newX = event.clientX - dragStart.offsetX;
      let newY = event.clientY - dragStart.offsetY;

      // Detect tab target (another window's title bar)
      const tabTarget = detectTabTarget(event.clientX, event.clientY);
      if (tabTarget) {
        showTabDropZone = true;
        potentialTabTarget = tabTarget;
        showDockPreview = false;
        dockPreviewZone = 'none';
      } else {
        showTabDropZone = false;
        potentialTabTarget = null;

        // Detect dock zone
        const zone = detectDockZone(event.clientX, event.clientY);
        if (zone !== 'none') {
          showDockPreview = true;
          dockPreviewZone = zone;
        } else {
          showDockPreview = false;
          dockPreviewZone = 'none';
        }
      }

      // Undock if currently docked and dragging away
      if (currentDockZone !== 'none' && dockPreviewZone === 'none' && !showTabDropZone) {
        currentDockZone = 'none';
        if (savedPosition) {
          position.width = savedPosition.width;
          position.height = savedPosition.height;
        }
      }

      // Apply snapping
      const snapped = calculateSnap(newX, newY, position.width, position.height);
      newX = snapped.x;
      newY = snapped.y;

      // Clamp to viewport
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      newX = Math.max(0, Math.min(newX, viewportWidth - 100)); // Keep at least 100px visible
      newY = Math.max(0, Math.min(newY, viewportHeight - 50));

      position.x = newX;
      position.y = newY;
    }

    if (isResizing && resizeStart) {
      const deltaX = event.clientX - resizeStart.x;
      const deltaY = event.clientY - resizeStart.y;

      let newX = resizeStart.windowX;
      let newY = resizeStart.windowY;
      let newWidth = resizeStart.windowWidth;
      let newHeight = resizeStart.windowHeight;

      // Handle different resize directions
      switch (resizeDirection) {
        case 'se':
          newWidth = resizeStart.windowWidth + deltaX;
          newHeight = resizeStart.windowHeight + deltaY;
          break;
        case 'sw':
          newX = resizeStart.windowX + deltaX;
          newWidth = resizeStart.windowWidth - deltaX;
          newHeight = resizeStart.windowHeight + deltaY;
          break;
        case 'ne':
          newY = resizeStart.windowY + deltaY;
          newWidth = resizeStart.windowWidth + deltaX;
          newHeight = resizeStart.windowHeight - deltaY;
          break;
        case 'nw':
          newX = resizeStart.windowX + deltaX;
          newY = resizeStart.windowY + deltaY;
          newWidth = resizeStart.windowWidth - deltaX;
          newHeight = resizeStart.windowHeight - deltaY;
          break;
        case 'n':
          newY = resizeStart.windowY + deltaY;
          newHeight = resizeStart.windowHeight - deltaY;
          break;
        case 's':
          newHeight = resizeStart.windowHeight + deltaY;
          break;
        case 'e':
          newWidth = resizeStart.windowWidth + deltaX;
          break;
        case 'w':
          newX = resizeStart.windowX + deltaX;
          newWidth = resizeStart.windowWidth - deltaX;
          break;
      }

      // Enforce min sizes
      if (newWidth < minWidth) {
        if (resizeDirection.includes('w')) {
          newX = resizeStart.windowX + resizeStart.windowWidth - minWidth;
        }
        newWidth = minWidth;
      }
      if (newHeight < minHeight) {
        if (resizeDirection.includes('n')) {
          newY = resizeStart.windowY + resizeStart.windowHeight - minHeight;
        }
        newHeight = minHeight;
      }

      position.x = newX;
      position.y = newY;
      position.width = newWidth;
      position.height = newHeight;
    }
  }

  function handleMouseUp(event: MouseEvent) {
    if (isDragging) {
      // Handle tab drop
      if (showTabDropZone && potentialTabTarget) {
        handleTabDrop(potentialTabTarget);
      }
      // Handle dock drop
      else if (showDockPreview && dockPreviewZone !== 'none') {
        applyDock(dockPreviewZone);
      }
    }

    isDragging = false;
    isResizing = false;
    dragStart = null;
    resizeStart = null;
    showDockPreview = false;
    dockPreviewZone = 'none';
    showTabDropZone = false;
    potentialTabTarget = null;
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
    dispatch('close', { windowId });
    uiActions.hideWindow(windowId);
  }

  function minimizeWindow() {
    isMinimized = true;
    dispatch('minimize', { windowId });
    uiActions.minimizeWindow(windowId);
  }

  function maximizeWindow() {
    if (isMaximized) {
      // Restore to previous size
      if (savedPosition) {
        position.x = savedPosition.x;
        position.y = savedPosition.y;
        position.width = savedPosition.width;
        position.height = savedPosition.height;
        savedPosition = null;
        isMaximized = false;
        currentDockZone = 'none';
      }
    } else {
      // Save current position and maximize
      savedPosition = {
        x: position.x,
        y: position.y,
        width: position.width,
        height: position.height,
        z_index: position.z_index,
        visible: position.visible,
      };

      // Full screen accounting for menu/status bars
      const menuBarHeight = 32;
      const statusBarHeight = 24;
      position.x = 0;
      position.y = menuBarHeight;
      position.width = window.innerWidth;
      position.height = window.innerHeight - menuBarHeight - statusBarHeight;
      isMaximized = true;
    }
    dispatch('maximize', { windowId });
  }

  function handleWindowClick() {
    uiActions.bringToFront(windowId);
    dispatch('focus', { windowId });
  }

  // Get dock preview bounds for rendering
  $: dockPreviewBounds = getDockPreviewBounds(dockPreviewZone);

  // Detect if dragging over another window's title bar for tabbing
  function detectTabTarget(clientX: number, clientY: number): WindowId | null {
    const titleBarHeight = 40; // Approximate title bar height
    for (const [id, pos] of Object.entries($uiStore.windows)) {
      if (id === windowId || !pos.visible) continue;
      // Check if cursor is over this window's title bar area
      if (
        clientX >= pos.x &&
        clientX <= pos.x + pos.width &&
        clientY >= pos.y &&
        clientY <= pos.y + titleBarHeight
      ) {
        return id as WindowId;
      }
    }
    return null;
  }

  // Handle tab drop
  function handleTabDrop(targetWindowId: WindowId) {
    dispatch('tabRequest', { sourceWindowId: windowId, targetWindowId });
    uiActions.createTabbedGroup([targetWindowId, windowId]);
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<!-- Dock Preview Overlay -->
{#if showDockPreview && dockPreviewBounds}
  <div
    class="dock-preview"
    style="
      left: {dockPreviewBounds.x}px;
      top: {dockPreviewBounds.y}px;
      width: {dockPreviewBounds.width}px;
      height: {dockPreviewBounds.height}px;
    "
  />
{/if}

<!-- Tab Drop Zone Indicator -->
{#if showTabDropZone && potentialTabTarget && $uiStore.windows[potentialTabTarget]}
  {@const targetPos = $uiStore.windows[potentialTabTarget]}
  <div
    class="tab-drop-preview"
    style="
      left: {targetPos.x}px;
      top: {targetPos.y}px;
      width: {targetPos.width}px;
      height: 40px;
    "
  >
    <span class="tab-drop-text">Drop to create tab group</span>
  </div>
{/if}

<div
  class="window-base"
  class:dragging={isDragging}
  class:resizing={isResizing}
  class:maximized={isMaximized}
  class:docked={currentDockZone !== 'none'}
  style="
    left: {position.x}px;
    top: {position.y}px;
    width: {position.width}px;
    height: {position.height}px;
    z-index: {currentZIndex};
    display: {position.visible ? 'flex' : 'none'};
  "
  on:click={handleWindowClick}
  on:keydown={(e) => e.key === 'Escape' && closeWindow()}
  role="dialog"
  aria-label={title}
  tabindex="-1"
>
  <!-- Title Bar -->
  <div
    class="window-title"
    on:mousedown={handleMouseDownTitle}
    role="banner"
  >
    <span class="window-title-text">{title}</span>
    <div class="window-controls">
      {#if minimizable}
        <button
          class="window-btn minimize-btn"
          on:click|stopPropagation={minimizeWindow}
          title="Minimize"
          aria-label="Minimize window"
        >
          <svg viewBox="0 0 12 12" width="12" height="12">
            <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      {/if}
      {#if maximizable}
        <button
          class="window-btn maximize-btn"
          class:active={isMaximized}
          on:click|stopPropagation={maximizeWindow}
          title={isMaximized ? 'Restore' : 'Maximize'}
          aria-label={isMaximized ? 'Restore window' : 'Maximize window'}
        >
          {#if isMaximized}
            <svg viewBox="0 0 12 12" width="12" height="12">
              <rect x="2" y="4" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M4 4V2h6v6h-2" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          {:else}
            <svg viewBox="0 0 12 12" width="12" height="12">
              <rect x="2" y="2" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          {/if}
        </button>
      {/if}
      {#if closable}
        <button
          class="window-btn close-btn"
          on:click|stopPropagation={closeWindow}
          title="Close"
          aria-label="Close window"
        >
          <svg viewBox="0 0 12 12" width="12" height="12">
            <line x1="2" y1="2" x2="10" y2="10" stroke="currentColor" stroke-width="1.5"/>
            <line x1="10" y1="2" x2="2" y2="10" stroke="currentColor" stroke-width="1.5"/>
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div class="window-content">
    <slot />
  </div>

  <!-- Resize Handles (all 8 directions) -->
  {#if resizable && !isMaximized}
    <div class="resize-handle resize-n" on:mousedown={(e) => handleMouseDownResize(e, 'n')} role="separator" aria-orientation="horizontal"/>
    <div class="resize-handle resize-s" on:mousedown={(e) => handleMouseDownResize(e, 's')} role="separator" aria-orientation="horizontal"/>
    <div class="resize-handle resize-e" on:mousedown={(e) => handleMouseDownResize(e, 'e')} role="separator" aria-orientation="vertical"/>
    <div class="resize-handle resize-w" on:mousedown={(e) => handleMouseDownResize(e, 'w')} role="separator" aria-orientation="vertical"/>
    <div class="resize-handle resize-nw" on:mousedown={(e) => handleMouseDownResize(e, 'nw')} role="separator"/>
    <div class="resize-handle resize-ne" on:mousedown={(e) => handleMouseDownResize(e, 'ne')} role="separator"/>
    <div class="resize-handle resize-sw" on:mousedown={(e) => handleMouseDownResize(e, 'sw')} role="separator"/>
    <div class="resize-handle resize-se" on:mousedown={(e) => handleMouseDownResize(e, 'se')} role="separator"/>
  {/if}
</div>

<style>
  .window-base {
    position: fixed;
    background-color: var(--window, #252525);
    border: 1px solid var(--window-border, #444);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4), 0 0 1px rgba(255, 255, 255, 0.1);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-width: 200px;
    min-height: 150px;
    transition: box-shadow 0.2s ease;
  }

  .window-base:focus-within {
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5), 0 0 0 1px var(--primary, #00d4aa);
  }

  .window-base.maximized {
    border-radius: 0;
  }

  .window-base.docked {
    border-radius: 0;
  }

  .window-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background-color: var(--bg-secondary, #1e1e1e);
    border-bottom: 1px solid var(--window-border, #444);
    cursor: move;
    user-select: none;
    -webkit-user-select: none;
  }

  .window-title-text {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary, #fff);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .window-controls {
    display: flex;
    gap: 4px;
  }

  .window-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-secondary, #888);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .window-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: var(--text-primary, #fff);
  }

  .window-btn.active {
    color: var(--primary, #00d4aa);
  }

  .close-btn:hover {
    background-color: #e81123;
    color: white;
  }

  .window-content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    background-color: var(--window, #252525);
  }

  .dragging,
  .resizing {
    user-select: none;
    cursor: grabbing !important;
  }

  .dragging .window-title {
    cursor: grabbing;
  }

  /* Resize Handles */
  .resize-handle {
    position: absolute;
    background: transparent;
  }

  .resize-n {
    top: 0;
    left: 8px;
    right: 8px;
    height: 4px;
    cursor: ns-resize;
  }

  .resize-s {
    bottom: 0;
    left: 8px;
    right: 8px;
    height: 4px;
    cursor: ns-resize;
  }

  .resize-e {
    right: 0;
    top: 8px;
    bottom: 8px;
    width: 4px;
    cursor: ew-resize;
  }

  .resize-w {
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 4px;
    cursor: ew-resize;
  }

  .resize-nw {
    top: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nwse-resize;
  }

  .resize-ne {
    top: 0;
    right: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }

  .resize-sw {
    bottom: 0;
    left: 0;
    width: 8px;
    height: 8px;
    cursor: nesw-resize;
  }

  .resize-se {
    bottom: 0;
    right: 0;
    width: 12px;
    height: 12px;
    cursor: nwse-resize;
    background-image: linear-gradient(
      -45deg,
      transparent 0%,
      transparent 40%,
      var(--window-border, #444) 40%,
      var(--window-border, #444) 45%,
      transparent 45%,
      transparent 55%,
      var(--window-border, #444) 55%,
      var(--window-border, #444) 60%,
      transparent 60%,
      transparent 100%
    );
    background-size: 6px 6px;
    background-position: bottom right;
    background-repeat: no-repeat;
  }

  /* Dock Preview */
  .dock-preview {
    position: fixed;
    background-color: var(--primary, #00d4aa);
    opacity: 0.2;
    border: 2px dashed var(--primary, #00d4aa);
    border-radius: 4px;
    pointer-events: none;
    z-index: 9998;
    animation: dock-pulse 1s ease-in-out infinite;
  }

  @keyframes dock-pulse {
    0%, 100% { opacity: 0.15; }
    50% { opacity: 0.3; }
  }

  /* Tab Drop Preview */
  .tab-drop-preview {
    position: fixed;
    background-color: var(--accent, #ff9500);
    opacity: 0.4;
    border: 2px solid var(--accent, #ff9500);
    border-radius: 8px 8px 0 0;
    pointer-events: none;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: tab-pulse 0.8s ease-in-out infinite;
  }

  .tab-drop-text {
    color: white;
    font-size: 12px;
    font-weight: 600;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  }

  @keyframes tab-pulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 0.5; }
  }
</style>
