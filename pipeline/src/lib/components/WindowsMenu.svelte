<script lang="ts">
  import { onMount } from 'svelte';
  import { windowStore, type WindowInfo } from '$lib/stores/windowStore';

  // Local state
  let windows: WindowInfo[] = [];
  let visibleWindows: WindowInfo[] = [];
  let currentLayout: string = 'default';
  let availableLayouts: string[] = [];
  let loading: boolean = false;
  let error: string | null = null;
  let showLayoutDialog: boolean = false;
  let newLayoutName: string = '';

  // Subscribe to store state
  $: loading = $windowStore.loading;
  $: error = $windowStore.error;
  $: currentLayout = $windowStore.currentLayout;

  // Load initial data
  onMount(async () => {
    await refreshWindows();
    await refreshLayouts();
    await loadCurrentLayout();
  });

  // Refresh window list
  async function refreshWindows() {
    try {
      windows = await windowStore.getAllWindows();
      visibleWindows = await windowStore.getVisibleWindows();
    } catch (err) {
      console.error('Failed to refresh windows:', err);
    }
  }

  // Refresh layout list
  async function refreshLayouts() {
    try {
      availableLayouts = await windowStore.getLayoutList();
    } catch (err) {
      console.error('Failed to refresh layouts:', err);
    }
  }

  // Load current layout name
  async function loadCurrentLayout() {
    try {
      currentLayout = await windowStore.getCurrentLayout();
    } catch (err) {
      console.error('Failed to load current layout:', err);
    }
  }

  // Toggle window visibility
  async function handleToggleWindow(label: string) {
    try {
      await windowStore.toggleWindow(label);
      await refreshWindows();
    } catch (err) {
      console.error('Toggle window failed:', err);
    }
  }

  // Show window
  async function handleShowWindow(label: string) {
    try {
      await windowStore.showWindow(label);
      await refreshWindows();
    } catch (err) {
      console.error('Show window failed:', err);
    }
  }

  // Hide window
  async function handleHideWindow(label: string) {
    try {
      await windowStore.hideWindow(label);
      await refreshWindows();
    } catch (err) {
      console.error('Hide window failed:', err);
    }
  }

  // Arrange windows
  async function handleArrangeWindows(type: 'tile_h' | 'tile_v' | 'cascade') {
    try {
      await windowStore.arrangeWindows(type);
      await refreshWindows();
    } catch (err) {
      console.error('Arrange windows failed:', err);
    }
  }

  // Save layout
  async function handleSaveLayout() {
    if (!newLayoutName.trim()) {
      return;
    }

    try {
      await windowStore.saveLayout(newLayoutName.trim());
      await refreshLayouts();
      showLayoutDialog = false;
      newLayoutName = '';
    } catch (err) {
      console.error('Save layout failed:', err);
    }
  }

  // Load layout
  async function handleLoadLayout(name: string) {
    try {
      await windowStore.loadLayout(name);
      await refreshWindows();
    } catch (err) {
      console.error('Load layout failed:', err);
    }
  }

  // Delete layout
  async function handleDeleteLayout(name: string) {
    if (!confirm(`Delete layout "${name}"?`)) {
      return;
    }

    try {
      await windowStore.deleteLayout(name);
      await refreshLayouts();
    } catch (err) {
      console.error('Delete layout failed:', err);
    }
  }

  // Clear error
  function clearError() {
    windowStore.clearError();
  }
</script>

<div class="windows-menu">
  <!-- Header -->
  <div class="menu-header">
    <h3 class="menu-title">Windows</h3>
    <button
      class="refresh-btn"
      on:click={refreshWindows}
      disabled={loading}
      title="Refresh window list"
    >
      🔄
    </button>
  </div>

  <!-- Error Display -->
  {#if error}
    <div class="error-banner">
      <span class="error-text">{error}</span>
      <button class="error-close" on:click={clearError}>×</button>
    </div>
  {/if}

  <!-- Loading State -->
  {#if loading}
    <div class="loading">Loading...</div>
  {/if}

  <!-- Window List -->
  <div class="window-list">
    <h4 class="section-title">All Windows ({windows.length})</h4>
    {#if windows.length === 0}
      <p class="empty-message">No windows available</p>
    {:else}
      {#each windows as window (window.label)}
        <div class="window-item" class:visible={window.visible}>
          <div class="window-info">
            <span class="window-icon">
              {#if window.window_type === 'main'}🏠
              {:else if window.window_type === 'dockable'}📌
              {:else if window.window_type === 'floating'}🪟
              {:else if window.window_type === 'modal'}⚠️
              {:else if window.window_type === 'palette'}🎨
              {/if}
            </span>
            <div class="window-details">
              <span class="window-title">{window.title}</span>
              <span class="window-label">{window.label}</span>
            </div>
          </div>
          <div class="window-actions">
            {#if window.visible}
              <button
                class="action-btn hide-btn"
                on:click={() => handleHideWindow(window.label)}
                title="Hide window"
              >
                👁️‍🗨️
              </button>
            {:else}
              <button
                class="action-btn show-btn"
                on:click={() => handleShowWindow(window.label)}
                title="Show window"
              >
                👁️
              </button>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Arrangement Controls -->
  <div class="arrangement-section">
    <h4 class="section-title">Arrange Windows</h4>
    <div class="arrangement-buttons">
      <button
        class="arrange-btn"
        on:click={() => handleArrangeWindows('tile_h')}
        disabled={loading}
        title="Tile windows horizontally"
      >
        ⬌ Tile H
      </button>
      <button
        class="arrange-btn"
        on:click={() => handleArrangeWindows('tile_v')}
        disabled={loading}
        title="Tile windows vertically"
      >
        ⬍ Tile V
      </button>
      <button
        class="arrange-btn"
        on:click={() => handleArrangeWindows('cascade')}
        disabled={loading}
        title="Cascade windows"
      >
        📑 Cascade
      </button>
    </div>
  </div>

  <!-- Layout Management -->
  <div class="layout-section">
    <h4 class="section-title">
      Layouts
      <span class="current-layout">({currentLayout})</span>
    </h4>
    <div class="layout-buttons">
      <button
        class="layout-btn save-btn"
        on:click={() => showLayoutDialog = true}
        disabled={loading}
        title="Save current layout"
      >
        💾 Save Layout
      </button>
    </div>

    {#if availableLayouts.length > 0}
      <div class="layout-list">
        {#each availableLayouts as layout (layout)}
          <div class="layout-item" class:active={layout === currentLayout}>
            <span class="layout-name">{layout}</span>
            <div class="layout-actions">
              <button
                class="layout-action-btn load-btn"
                on:click={() => handleLoadLayout(layout)}
                disabled={loading || layout === currentLayout}
                title="Load this layout"
              >
                📂
              </button>
              <button
                class="layout-action-btn delete-btn"
                on:click={() => handleDeleteLayout(layout)}
                disabled={loading || layout === 'default'}
                title="Delete this layout"
              >
                🗑️
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <p class="empty-message">No saved layouts</p>
    {/if}
  </div>

  <!-- Save Layout Dialog -->
  {#if showLayoutDialog}
    <div class="dialog-overlay" on:click={() => showLayoutDialog = false}>
      <div class="dialog" on:click|stopPropagation>
        <h3 class="dialog-title">Save Layout</h3>
        <input
          type="text"
          class="dialog-input"
          placeholder="Enter layout name..."
          bind:value={newLayoutName}
          on:keydown={(e) => e.key === 'Enter' && handleSaveLayout()}
        />
        <div class="dialog-buttons">
          <button
            class="dialog-btn cancel-btn"
            on:click={() => showLayoutDialog = false}
          >
            Cancel
          </button>
          <button
            class="dialog-btn save-btn"
            on:click={handleSaveLayout}
            disabled={!newLayoutName.trim()}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .windows-menu {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md, 16px);
    padding: var(--spacing-md, 16px);
    background: var(--color-surface, #2d2d2d);
    border-radius: var(--radius-md, 6px);
    max-width: 400px;
  }

  /* Header */
  .menu-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: var(--spacing-sm, 8px);
    border-bottom: 1px solid var(--color-border, #404040);
  }

  .menu-title {
    font-size: var(--font-size-lg, 18px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    font-size: 16px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Error Banner */
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm, 8px);
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid #ef4444;
    border-radius: var(--radius-sm, 4px);
    color: #ef4444;
    font-size: var(--font-size-sm, 14px);
  }

  .error-text {
    flex: 1;
  }

  .error-close {
    background: none;
    border: none;
    color: #ef4444;
    font-size: 20px;
    cursor: pointer;
    padding: 0 4px;
  }

  /* Loading */
  .loading {
    text-align: center;
    padding: var(--spacing-md, 16px);
    color: var(--color-text-secondary, #a3a3a3);
    font-size: var(--font-size-sm, 14px);
  }

  /* Section Title */
  .section-title {
    font-size: var(--font-size-md, 16px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0 0 var(--spacing-sm, 8px) 0;
  }

  .current-layout {
    font-weight: 400;
    color: var(--color-primary, #3b82f6);
  }

  /* Window List */
  .window-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 4px);
  }

  .window-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    opacity: 0.6;
    transition: all 0.2s;
  }

  .window-item.visible {
    opacity: 1;
    border-color: var(--color-primary, #3b82f6);
  }

  .window-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    flex: 1;
    min-width: 0;
  }

  .window-icon {
    font-size: 20px;
    flex-shrink: 0;
  }

  .window-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .window-title {
    font-size: var(--font-size-sm, 14px);
    color: var(--color-text, #e5e5e5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .window-label {
    font-size: var(--font-size-xs, 12px);
    color: var(--color-text-secondary, #a3a3a3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .window-actions {
    display: flex;
    gap: var(--spacing-xs, 4px);
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
  }

  /* Arrangement Section */
  .arrangement-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm, 8px);
  }

  .arrangement-buttons {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-xs, 4px);
  }

  .arrange-btn {
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text, #e5e5e5);
    font-size: var(--font-size-sm, 14px);
    cursor: pointer;
    transition: all 0.2s;
  }

  .arrange-btn:hover:not(:disabled) {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
  }

  .arrange-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Layout Section */
  .layout-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm, 8px);
  }

  .layout-buttons {
    display: flex;
    gap: var(--spacing-xs, 4px);
  }

  .layout-btn {
    flex: 1;
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text, #e5e5e5);
    font-size: var(--font-size-sm, 14px);
    cursor: pointer;
    transition: all 0.2s;
  }

  .layout-btn:hover:not(:disabled) {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
  }

  .layout-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .layout-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 4px);
  }

  .layout-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    transition: all 0.2s;
  }

  .layout-item.active {
    border-color: var(--color-primary, #3b82f6);
    background: rgba(59, 130, 246, 0.1);
  }

  .layout-name {
    font-size: var(--font-size-sm, 14px);
    color: var(--color-text, #e5e5e5);
  }

  .layout-actions {
    display: flex;
    gap: var(--spacing-xs, 4px);
  }

  .layout-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .layout-action-btn:hover:not(:disabled) {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
  }

  .layout-action-btn.delete-btn:hover:not(:disabled) {
    background: #ef4444;
    border-color: #ef4444;
  }

  .layout-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Empty Message */
  .empty-message {
    text-align: center;
    padding: var(--spacing-md, 16px);
    color: var(--color-text-secondary, #a3a3a3);
    font-size: var(--font-size-sm, 14px);
    margin: 0;
  }

  /* Dialog */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--color-surface, #2d2d2d);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-md, 6px);
    padding: var(--spacing-lg, 24px);
    min-width: 300px;
    max-width: 90vw;
  }

  .dialog-title {
    font-size: var(--font-size-lg, 18px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0 0 var(--spacing-md, 16px) 0;
  }

  .dialog-input {
    width: 100%;
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text, #e5e5e5);
    font-size: var(--font-size-sm, 14px);
    margin-bottom: var(--spacing-md, 16px);
  }

  .dialog-input:focus {
    outline: none;
    border-color: var(--color-primary, #3b82f6);
  }

  .dialog-buttons {
    display: flex;
    gap: var(--spacing-sm, 8px);
    justify-content: flex-end;
  }

  .dialog-btn {
    padding: var(--spacing-sm, 8px) var(--spacing-md, 16px);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    font-size: var(--font-size-sm, 14px);
    cursor: pointer;
    transition: all 0.2s;
  }

  .dialog-btn.cancel-btn {
    background: var(--color-bg, #1a1a1a);
    color: var(--color-text, #e5e5e5);
  }

  .dialog-btn.cancel-btn:hover {
    background: var(--color-surface, #2d2d2d);
  }

  .dialog-btn.save-btn {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
    color: white;
  }

  .dialog-btn.save-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .dialog-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
