<script lang="ts">
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';
  import VIP3Column from './VIP3Column.svelte';
  import VIP3BpmColumn from './VIP3BpmColumn.svelte';
  import SavedSearchesList from './SavedSearches/SavedSearchesList.svelte';
  import CollectionsList from './Collections/CollectionsList.svelte';
  import CollectionViewer from './Collections/CollectionViewer.svelte';
  import FavoritesList from './Favorites/FavoritesList.svelte';
  import FavoritesButton from './Favorites/FavoritesButton.svelte';
  import FileContextMenu from './FileContextMenu.svelte';
  import {
    vip3Actions,
    vip3Store,
    timbres,
    styles,
    articulations,
    loadingCounts,
    totalMatches,
    hasActiveFilters
  } from '$lib/stores/vip3Store';
  import { browserActions } from '$lib/stores/browserStore';
  import { sequencerActions } from '$lib/stores/sequencerStore';
  import { toastStore } from '$lib/stores/toastStore';
  import { api } from '$lib/api';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { SavedSearchResponse, CollectionResponse } from '$lib/types/vip3';

  // Reactive stores for folders and instruments loaded from database
  const folders = writable<Array<{ id: number; name: string }>>([]);
  const instruments = writable<Array<{ id: number; name: string }>>([]);
  const loadingCategories = writable(true);

  // Load folders and instruments from database on mount
  async function loadLocalCategories() {
    try {
      loadingCategories.set(true);
      const categories = await Vip3BrowserApi.getAllCategories();

      // Convert folder strings to objects with IDs
      folders.set(categories.folders.map((name, idx) => ({
        id: idx + 1,
        name
      })));

      // Convert instrument strings to objects with IDs
      instruments.set(categories.instruments.map((name, idx) => ({
        id: idx + 1,
        name
      })));

      console.log(`✓ Loaded ${categories.folders.length} folders, ${categories.instruments.length} instruments`);
    } catch (error) {
      console.error('Failed to load VIP3 categories:', error);
      // Fallback to empty arrays
      folders.set([]);
      instruments.set([]);
    } finally {
      loadingCategories.set(false);
    }
  }

  type Tab = 'browser' | 'searches' | 'collections' | 'favorites';
  let activeTab: Tab = 'browser';
  let selectedCollection: CollectionResponse | null = null;
  let searchQuery = '';
  let searchDebounceTimer: ReturnType<typeof setTimeout>;

  // Reactive file list and state from store
  $: files = $vip3Store.files;
  $: isLoading = $vip3Store.isLoading;
  $: totalCount = $vip3Store.totalCount;
  $: currentPage = $vip3Store.page;
  $: pageSize = $vip3Store.pageSize;
  $: totalPages = Math.ceil(totalCount / pageSize);

  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;

    // Debounce search
    clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      vip3Actions.setSearchQuery(searchQuery);
    }, 300);
  }

  function clearSearch() {
    searchQuery = '';
    vip3Actions.setSearchQuery('');
  }

  function clearAllFilters() {
    searchQuery = '';
    vip3Actions.clearFilters();
    toastStore.info('All filters cleared');
  }

  // Context menu state
  let contextMenu = {
    visible: false,
    x: 0,
    y: 0,
    fileId: 0,
    filename: '',
    isFavorite: false
  };

  function handleContextMenu(event: MouseEvent, file: { id: number; filename: string; favorite?: boolean }) {
    event.preventDefault();

    // Position menu at mouse coordinates, adjusting if near edges
    const menuWidth = 220;
    const menuHeight = 250;
    let x = event.clientX;
    let y = event.clientY;

    if (x + menuWidth > window.innerWidth) {
      x = window.innerWidth - menuWidth - 10;
    }
    if (y + menuHeight > window.innerHeight) {
      y = window.innerHeight - menuHeight - 10;
    }

    contextMenu = {
      visible: true,
      x,
      y,
      fileId: file.id,
      filename: file.filename,
      isFavorite: file.favorite ?? false
    };
  }

  /**
   * Handle favorite toggle from file grid
   */
  async function handleFavoriteToggle(fileId: number, newState: boolean) {
    // Update the store's file list with new favorite state
    vip3Store.update((s) => ({
      ...s,
      files: s.files.map((f) =>
        f.id === fileId ? { ...f, favorite: newState } : f
      ),
    }));
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  function handleLoadSearch(search: SavedSearchResponse) {
    // Apply the saved search filters to the browser
    if (search.filters) {
      vip3Actions.setFilters(search.filters);
    }
    activeTab = 'browser';
  }

  function handleSelectCollection(collection: CollectionResponse) {
    selectedCollection = collection;
  }

  function handleCloseCollectionViewer() {
    selectedCollection = null;
  }

  /** Selected file ID for tracking selection state */
  let selectedFileId: number | null = null;

  /**
   * Handle file selection from favorites, collections, or search results.
   * - Single click: Select and preview the file
   * - The file can then be added to the sequencer via context menu or double-click
   */
  async function handleFileSelect(fileId: number) {
    console.log('File selected:', fileId);

    // Update selection state
    selectedFileId = fileId;

    // Set as preview file in browser store (enables preview panel if configured)
    browserActions.setPreviewFile(fileId);

    // Play the file for audio preview
    try {
      await api.transport.playFile(fileId);
    } catch (error) {
      console.error('Failed to preview file:', error);
    }
  }

  /**
   * Handle double-click to load file into sequencer
   */
  async function handleFileDoubleClick(fileId: number) {
    console.log('Loading file to sequencer:', fileId);

    try {
      // Get file details for the track name
      const fileDetails = await api.files.getFileDetails(fileId);
      const trackName = fileDetails?.filename || `Track ${fileId}`;

      // Add a new track to the sequencer
      const trackId = sequencerActions.addTrack(trackName);
      if (trackId) {
        // Add clip at start of timeline (1920 ticks = 1 bar at 480 PPQ)
        sequencerActions.addClip(trackId, 0, 1920, fileId, trackName);
        toastStore.success(`Added "${trackName}" to sequencer`);
      }
    } catch (error) {
      console.error('Failed to load file to sequencer:', error);
      toastStore.error('Failed to load file to sequencer');
    }
  }

  /**
   * Format duration in seconds to mm:ss format
   */
  function formatDuration(seconds: number | null): string {
    if (seconds === null) return '--:--';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  /**
   * Format BPM value
   */
  function formatBpm(bpm: number | null): string {
    if (bpm === null) return '--';
    return `${Math.round(bpm)} BPM`;
  }

  onMount(async () => {
    // Load local folder/instrument categories and initialize VIP3 store in parallel
    await Promise.all([
      loadLocalCategories(),
      vip3Actions.initialize()
    ]);
  });
</script>

<div class="vip3-browser">
  <div class="browser-header">
    <h2>VIP3 Browser</h2>
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === 'browser'}
        on:click={() => activeTab = 'browser'}
      >
        🔍 Browser
      </button>
      <button
        class="tab"
        class:active={activeTab === 'searches'}
        on:click={() => activeTab = 'searches'}
      >
        ⭐ Saved Searches
      </button>
      <button
        class="tab"
        class:active={activeTab === 'collections'}
        on:click={() => activeTab = 'collections'}
      >
        📁 Collections
      </button>
      <button
        class="tab"
        class:active={activeTab === 'favorites'}
        on:click={() => activeTab = 'favorites'}
      >
        ❤️ Favorites
      </button>
    </div>
    {#if activeTab === 'browser'}
      {#if $loadingCounts}
        <span class="loading">Loading counts...</span>
      {:else}
        <span class="total-matches">
          {$totalMatches.toLocaleString()} files
        </span>
      {/if}
    {/if}
  </div>

  {#if activeTab === 'browser'}
    <!-- Search Bar -->
    <div class="search-bar">
      <div class="search-input-wrapper">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="M21 21l-4.35-4.35"/>
        </svg>
        <input
          type="search"
          class="search-input"
          placeholder="Search files by name..."
          bind:value={searchQuery}
          on:input={handleSearchInput}
        />
        {#if searchQuery}
          <button class="clear-search-btn" on:click={clearSearch} title="Clear search">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        {/if}
      </div>
      {#if $hasActiveFilters}
        <button class="clear-filters-btn" on:click={clearAllFilters}>
          Clear All Filters
        </button>
      {/if}
    </div>

    <div class="filter-columns">
      <VIP3Column
        title="Folders"
        items={$folders}
        filterKey="folder_ids"
        countKey="folder_counts"
      />

      <VIP3Column
        title="Instruments"
        items={$instruments}
        filterKey="instrument_ids"
        countKey="instrument_counts"
      />

      <VIP3Column
        title="Timbre"
        items={$timbres}
        filterKey="timbre_ids"
        countKey="timbre_counts"
      />

      <VIP3Column
        title="Style"
        items={$styles}
        filterKey="style_ids"
        countKey="style_counts"
      />

      <VIP3Column
        title="Articulation"
        items={$articulations}
        filterKey="articulation_ids"
        countKey="articulation_counts"
      />

      <VIP3BpmColumn />
    </div>

    <!-- Results Panel with File Grid -->
    <div class="results-panel">
      {#if isLoading}
        <div class="loading-state">
          <div class="spinner"></div>
          <span>Loading files...</span>
        </div>
      {:else if files.length === 0}
        <div class="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
          <p>No files match your filters</p>
          <button class="reset-btn" on:click={clearAllFilters}>Reset Filters</button>
        </div>
      {:else}
        <!-- File Results Header -->
        <div class="results-header">
          <span class="results-count">{totalCount.toLocaleString()} files found</span>
          {#if totalPages > 1}
            <div class="pagination">
              <button
                class="page-btn"
                disabled={currentPage <= 1}
                on:click={() => vip3Actions.prevPage()}
              >
                Prev
              </button>
              <span class="page-info">Page {currentPage} of {totalPages}</span>
              <button
                class="page-btn"
                disabled={currentPage >= totalPages}
                on:click={() => vip3Actions.nextPage()}
              >
                Next
              </button>
            </div>
          {/if}
        </div>

        <!-- File Grid -->
        <div class="file-grid">
          {#each files as file (file.id)}
            <div
              class="file-item"
              class:selected={selectedFileId === file.id}
              role="button"
              tabindex="0"
              on:click={() => handleFileSelect(file.id)}
              on:dblclick={() => handleFileDoubleClick(file.id)}
              on:contextmenu={(e) => handleContextMenu(e, file)}
              on:keydown={(e) => e.key === 'Enter' && handleFileDoubleClick(file.id)}
            >
              <div class="file-icon">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M9 3v2H5v14h14V5h-4V3H9zm0 2h6v2H9V5zM7 9h10v2H7V9zm0 4h10v2H7v-2z"/>
                </svg>
              </div>
              <div class="file-info">
                <span class="file-name" title={file.filename}>{file.filename}</span>
                <div class="file-meta">
                  <span class="meta-item bpm">{formatBpm(file.bpm)}</span>
                  {#if file.key_signature}
                    <span class="meta-item key">{file.key_signature}</span>
                  {/if}
                  <span class="meta-item duration">{formatDuration(file.duration_seconds)}</span>
                </div>
              </div>
              <div class="file-actions">
                <FavoritesButton
                  fileId={file.id}
                  isFavorite={file.favorite}
                  size="small"
                  onToggle={(newState) => handleFavoriteToggle(file.id, newState)}
                />
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else if activeTab === 'searches'}
    <div class="panel-content">
      <SavedSearchesList onLoadSearch={handleLoadSearch} />
    </div>
  {:else if activeTab === 'collections'}
    <div class="panel-content">
      {#if selectedCollection}
        <CollectionViewer
          collection={selectedCollection}
          onClose={handleCloseCollectionViewer}
        />
      {:else}
        <CollectionsList onSelectCollection={handleSelectCollection} />
      {/if}
    </div>
  {:else if activeTab === 'favorites'}
    <div class="panel-content">
      <FavoritesList onFileSelect={handleFileSelect} />
    </div>
  {/if}
</div>

<!-- Context Menu -->
<FileContextMenu
  bind:visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  fileId={contextMenu.fileId}
  filename={contextMenu.filename}
  isFavorite={contextMenu.isFavorite}
  on:close={closeContextMenu}
  on:addToSequencer={(e) => handleFileDoubleClick(e.detail)}
/>

<style>
  .vip3-browser {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #151515;
    color: #fff;
  }

  .browser-header {
    padding: 16px 24px;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 24px;
    background: #1f1f1f;
  }

  .browser-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    white-space: nowrap;
  }

  .tabs {
    display: flex;
    gap: 8px;
    flex: 1;
  }

  .tab {
    padding: 8px 16px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #999;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .tab:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
  }

  .tab.active {
    background: #007aff;
    color: #fff;
  }

  .loading {
    font-size: 14px;
    color: #999;
    white-space: nowrap;
  }

  .total-matches {
    font-size: 14px;
    color: #60a5fa;
    font-weight: 500;
    white-space: nowrap;
  }

  .filter-columns {
    display: flex;
    border-bottom: 1px solid #333;
    background: #1a1a1a;
    overflow-x: auto;
  }

  .results-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .panel-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Search Bar Styles */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    background: #1a1a1a;
    border-bottom: 1px solid #333;
  }

  .search-input-wrapper {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    width: 18px;
    height: 18px;
    color: #666;
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 10px 40px;
    background: #252525;
    border: 1px solid #333;
    border-radius: 8px;
    color: #fff;
    font-size: 14px;
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
  }

  .search-input:focus {
    border-color: #007aff;
    box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.2);
  }

  .search-input::placeholder {
    color: #666;
  }

  .clear-search-btn {
    position: absolute;
    right: 8px;
    width: 24px;
    height: 24px;
    padding: 4px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    border-radius: 4px;
    transition: color 0.2s, background 0.2s;
  }

  .clear-search-btn:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
  }

  .clear-search-btn svg {
    width: 16px;
    height: 16px;
  }

  .clear-filters-btn {
    padding: 8px 16px;
    background: rgba(255, 59, 48, 0.1);
    border: 1px solid rgba(255, 59, 48, 0.3);
    border-radius: 6px;
    color: #ff3b30;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
    white-space: nowrap;
  }

  .clear-filters-btn:hover {
    background: rgba(255, 59, 48, 0.2);
  }

  /* Results Panel Styles */
  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-bottom: 1px solid #333;
    margin-bottom: 16px;
  }

  .results-count {
    font-size: 14px;
    color: #999;
  }

  .pagination {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .page-btn {
    padding: 6px 12px;
    background: #252525;
    border: 1px solid #333;
    border-radius: 4px;
    color: #fff;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .page-btn:hover:not(:disabled) {
    background: #333;
  }

  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 13px;
    color: #999;
  }

  /* Loading State */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 48px;
    color: #999;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #333;
    border-top-color: #007aff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 48px;
    color: #666;
    text-align: center;
  }

  .empty-state svg {
    width: 48px;
    height: 48px;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 0;
    font-size: 16px;
  }

  .reset-btn {
    padding: 8px 16px;
    background: #007aff;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 14px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .reset-btn:hover {
    background: #0056b3;
  }

  /* File Grid */
  .file-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
    padding-bottom: 24px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: #1f1f1f;
    border: 1px solid #333;
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: background 0.2s, border-color 0.2s, transform 0.1s;
  }

  .file-item:hover {
    background: #252525;
    border-color: #444;
  }

  .file-item.selected {
    background: rgba(0, 122, 255, 0.15);
    border-color: #007aff;
  }

  .file-item:active {
    transform: scale(0.98);
  }

  .file-icon {
    flex-shrink: 0;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 122, 255, 0.1);
    border-radius: 8px;
    color: #007aff;
  }

  .file-icon svg {
    width: 24px;
    height: 24px;
  }

  .file-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-name {
    font-size: 14px;
    font-weight: 500;
    color: #fff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .meta-item {
    font-size: 12px;
    color: #999;
    padding: 2px 6px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
  }

  .meta-item.bpm {
    color: #22c55e;
    background: rgba(34, 197, 94, 0.1);
  }

  .meta-item.key {
    color: #eab308;
    background: rgba(234, 179, 8, 0.1);
  }

  .meta-item.duration {
    color: #60a5fa;
    background: rgba(96, 165, 250, 0.1);
  }

  .file-actions {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .file-item:hover .file-actions,
  .file-item.selected .file-actions {
    opacity: 1;
  }
</style>
