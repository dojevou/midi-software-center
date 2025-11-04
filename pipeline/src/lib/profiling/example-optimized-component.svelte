<!--
  Example Optimized Component - Phase 7C Frontend Optimization

  Demonstrates all optimization techniques:
  - Render profiling
  - Virtual scrolling
  - Debounced updates
  - Memory management
  - Network optimization

  This is a reference implementation showing best practices.
-->

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { useFullProfiler } from '$lib/profiling/hooks';
  import { VirtualScrollHelper } from '$lib/profiling/performance';
  import { CleanupTracker } from '$lib/profiling/performance';
  import { debounce } from '$lib/utils/debounce';

  // ============================================================================
  // PROPS & STATE
  // ============================================================================

  export let title = 'Optimized Component';

  interface FileItem {
    id: string;
    name: string;
    size: number;
    bpm: number;
  }

  let files: FileItem[] = [];
  let searchQuery = '';
  let scrollTop = 0;
  let loading = false;
  let error: string | null = null;

  // ============================================================================
  // PROFILING SETUP
  // ============================================================================

  // Enable full profiling (render, FPS, memory, network)
  const { trackRequest, cleanup } = useFullProfiler('OptimizedComponent');

  // ============================================================================
  // VIRTUAL SCROLLING SETUP
  // ============================================================================

  const CONTAINER_HEIGHT = 600;
  const ITEM_HEIGHT = 50;

  const virtualScroll = new VirtualScrollHelper(
    CONTAINER_HEIGHT,
    ITEM_HEIGHT,
    0, // Will be updated reactively
    3  // Overscan 3 items above/below
  );

  // Reactive virtual scrolling calculations
  $: virtualScroll.updateTotalItems(files.length);
  $: visibleRange = virtualScroll.calculateVisibleRange(scrollTop);
  $: visibleItems = files.slice(visibleRange.start, visibleRange.end);
  $: totalHeight = virtualScroll.getTotalHeight();
  $: offsetY = virtualScroll.getOffsetY(visibleRange.start);

  // ============================================================================
  // NETWORK OPTIMIZATION
  // ============================================================================

  // Debounced search - waits 300ms after user stops typing
  const debouncedSearch = debounce(async (query: string) => {
    if (!query.trim()) {
      files = [];
      return;
    }

    loading = true;
    error = null;

    try {
      // Track network request for profiling
      const results = await trackRequest(
        async () => {
          return await invoke<FileItem[]>('search_files', {
            query: query.trim()
          });
        },
        // Estimate response size for tracking (optional)
        1024 * 10 // ~10KB estimate
      );

      files = results;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed';
      console.error('Search error:', e);
    } finally {
      loading = false;
    }
  }, 300);

  // ============================================================================
  // MEMORY MANAGEMENT
  // ============================================================================

  // Track all event listeners for cleanup
  const cleanupTracker = new CleanupTracker('OptimizedComponent');

  // Register keyboard shortcut
  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'f') {
      e.preventDefault();
      // Focus search input
    }
  }

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(() => {
    console.log('[OptimizedComponent] Mounted');

    // Register event listener for cleanup
    cleanupTracker.registerEventListener(
      window,
      'keydown',
      handleKeydown
    );

    // Load initial data
    loadInitialData();
  });

  onDestroy(() => {
    console.log('[OptimizedComponent] Destroyed');

    // Cleanup all event listeners
    cleanupTracker.cleanup();

    // Cleanup count for debugging
    const cleanupCount = cleanupTracker.getCleanupCount();
    console.log(`[OptimizedComponent] Cleaned up ${cleanupCount} resources`);
  });

  // ============================================================================
  // ACTIONS
  // ============================================================================

  async function loadInitialData() {
    loading = true;

    try {
      const results = await trackRequest(async () => {
        return await invoke<FileItem[]>('get_recent_files', { limit: 100 });
      });

      files = results;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load files';
    } finally {
      loading = false;
    }
  }

  function handleSearchInput() {
    // Debounced search will be called after 300ms of inactivity
    debouncedSearch(searchQuery);
  }

  function handleScroll(event: Event) {
    const target = event.currentTarget as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function handleFileClick(file: FileItem) {
    console.log('File clicked:', file.name);
    // Dispatch event or handle action
  }
</script>

<!-- ============================================================================
     TEMPLATE
     ============================================================================ -->

<div class="optimized-component">
  <!-- Header -->
  <div class="header">
    <h2>{title}</h2>

    <!-- Search Input -->
    <input
      type="text"
      class="search-input"
      placeholder="Search files..."
      bind:value={searchQuery}
      on:input={handleSearchInput}
    />
  </div>

  <!-- Content -->
  <div class="content">
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Loading...</p>
      </div>
    {:else if error}
      <div class="error">
        <p>Error: {error}</p>
        <button on:click={loadInitialData}>Retry</button>
      </div>
    {:else if files.length === 0}
      <div class="empty">
        <p>No files found</p>
      </div>
    {:else}
      <!-- Virtual Scrolling Container -->
      <div
        class="virtual-list"
        style="height: {CONTAINER_HEIGHT}px;"
        on:scroll={handleScroll}
      >
        <!-- Spacer for total height -->
        <div style="height: {totalHeight}px; position: relative;">
          <!-- Visible items with offset -->
          <div style="transform: translateY({offsetY}px);">
            {#each visibleItems as file (file.id)}
              <div
                class="file-item"
                style="height: {ITEM_HEIGHT}px;"
                on:click={() => handleFileClick(file)}
                on:keypress={(e) => e.key === 'Enter' && handleFileClick(file)}
                role="button"
                tabindex="0"
              >
                <div class="file-name">{file.name}</div>
                <div class="file-meta">
                  {(file.size / 1024).toFixed(2)} KB | {file.bpm} BPM
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Footer with stats -->
      <div class="footer">
        <span>
          Showing {visibleRange.start + 1}-{Math.min(visibleRange.end, files.length)} of {files.length} files
        </span>
      </div>
    {/if}
  </div>
</div>

<!-- ============================================================================
     STYLES
     ============================================================================ -->

<style>
  .optimized-component {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #ffffff);
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }

  .header h2 {
    margin: 0 0 12px 0;
    font-size: 20px;
    font-weight: 600;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: 4px;
    font-size: 14px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--primary-color, #007bff);
  }

  .content {
    flex: 1;
    overflow: hidden;
  }

  .loading,
  .error,
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 32px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border-color, #e0e0e0);
    border-top-color: var(--primary-color, #007bff);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error {
    color: var(--error-color, #dc3545);
  }

  .error button {
    margin-top: 16px;
    padding: 8px 16px;
    background: var(--primary-color, #007bff);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .error button:hover {
    opacity: 0.9;
  }

  .virtual-list {
    overflow-y: auto;
    overflow-x: hidden;
    /* Use GPU acceleration for smooth scrolling */
    will-change: scroll-position;
    -webkit-overflow-scrolling: touch;
  }

  .file-item {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .file-item:hover {
    background-color: var(--bg-hover, #f5f5f5);
  }

  .file-item:focus {
    outline: 2px solid var(--primary-color, #007bff);
    outline-offset: -2px;
  }

  .file-name {
    font-size: 14px;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .file-meta {
    font-size: 12px;
    color: var(--text-secondary, #666666);
  }

  .footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    font-size: 12px;
    color: var(--text-secondary, #666666);
    text-align: center;
  }
</style>
