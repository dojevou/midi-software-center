<script lang="ts">
  /**
   * DatabaseWindow - MIDI database browser
   *
   * Task-O-Matic: Search, filter, preview, and import MIDI files.
   */

  import { onMount } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    databaseStore,
    databaseActions,
    hasResults,
    totalPages,
    hasNextPage,
    hasPreviousPage,
    activeFiltersCount
  } from '$lib/stores/databaseStore';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/api/dialog';

  // Local state
  let searchInput = '';
  let showFilters = false;
  let previewPlaying = false;

  // Filter state
  let bpmMin = '';
  let bpmMax = '';
  let selectedKeys: string[] = [];
  let selectedCategories: string[] = [];
  let selectedTags: string[] = [];
  let favoriteOnly = false;

  // Available options
  let availableKeys = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
  let availableCategories: string[] = [];
  let availableTags: string[] = [];

  // Reactive state
  $: searchResults = $databaseStore.searchResults;
  $: selectedFile = $databaseStore.selectedFile;
  $: loading = $databaseStore.loading;
  $: error = $databaseStore.error;
  $: currentPage = $databaseStore.currentPage;
  $: totalResults = $databaseStore.totalResults;

  // Search
  async function handleSearch() {
    await databaseActions.search(searchInput);
  }

  function handleSearchKeypress(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSearch();
    }
  }

  // Filters
  async function applyFilters() {
    const filters: any = {
      favoriteOnly
    };

    if (bpmMin) filters.bpmMin = parseInt(bpmMin);
    if (bpmMax) filters.bpmMax = parseInt(bpmMax);
    if (selectedKeys.length > 0) filters.keys = selectedKeys;
    if (selectedCategories.length > 0) filters.categories = selectedCategories;
    if (selectedTags.length > 0) filters.tags = selectedTags;

    await databaseActions.setFilters(filters);
  }

  async function clearFilters() {
    bpmMin = '';
    bpmMax = '';
    selectedKeys = [];
    selectedCategories = [];
    selectedTags = [];
    favoriteOnly = false;
    await databaseActions.clearFilters();
  }

  // File selection
  function handleSelectFile(fileId: string) {
    databaseActions.selectFile(fileId);
  }

  async function handleToggleFavorite(fileId: string) {
    await databaseActions.toggleFavorite(fileId);
  }

  // Preview
  async function handlePreview() {
    if (!selectedFile) return;

    try {
      if (previewPlaying) {
        await invoke('stop_preview');
        previewPlaying = false;
      } else {
        await invoke('start_preview', { filepath: selectedFile.filepath });
        previewPlaying = true;
      }
    } catch (err) {
      console.error('Preview failed:', err);
    }
  }

  // Import
  async function handleImportFile() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'MIDI Files',
        extensions: ['mid', 'midi']
      }]
    });

    if (selected && typeof selected === 'string') {
      try {
        await databaseActions.importFile(selected);
      } catch (err) {
        console.error('Import failed:', err);
      }
    }
  }

  async function handleImportFolder() {
    const selected = await open({
      directory: true,
      multiple: false
    });

    if (selected && typeof selected === 'string') {
      try {
        await databaseActions.importDirectory(selected);
      } catch (err) {
        console.error('Import failed:', err);
      }
    }
  }

  // Pagination
  async function handlePreviousPage() {
    await databaseActions.previousPage();
  }

  async function handleNextPage() {
    await databaseActions.nextPage();
  }

  // Format helpers
  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }

  // Load available options
  onMount(async () => {
    availableCategories = await databaseActions.getCategories();
    availableTags = await databaseActions.getTags();
    await handleSearch(); // Initial search
  });
</script>

<WindowBase windowId="database" title="Database">
  <div class="database-window">
    <!-- Search Bar -->
    <div class="search-bar">
      <input
        type="text"
        class="search-input"
        placeholder="Search MIDI files..."
        bind:value={searchInput}
        on:keypress={handleSearchKeypress}
      />
      <button class="search-btn" on:click={handleSearch} disabled={loading}>
        {loading ? '...' : '🔍'}
      </button>
      <button
        class="filter-btn"
        class:active={showFilters}
        on:click={() => showFilters = !showFilters}
      >
        Filters {#if $activeFiltersCount > 0}({$activeFiltersCount}){/if}
      </button>
      <button class="import-btn" on:click={handleImportFile}>
        Import File
      </button>
      <button class="import-btn" on:click={handleImportFolder}>
        Import Folder
      </button>
    </div>

    <!-- Filters Panel -->
    {#if showFilters}
      <div class="filters-panel">
        <div class="filter-group">
          <label class="filter-label">BPM Range:</label>
          <div class="filter-inputs">
            <input
              type="number"
              class="filter-input"
              placeholder="Min"
              bind:value={bpmMin}
            />
            <span>-</span>
            <input
              type="number"
              class="filter-input"
              placeholder="Max"
              bind:value={bpmMax}
            />
          </div>
        </div>

        <div class="filter-group">
          <label class="filter-label">Keys:</label>
          <div class="filter-checkboxes">
            {#each availableKeys as key}
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  value={key}
                  bind:group={selectedKeys}
                />
                {key}
              </label>
            {/each}
          </div>
        </div>

        <div class="filter-group">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={favoriteOnly} />
            Favorites Only
          </label>
        </div>

        <div class="filter-actions">
          <button class="apply-btn" on:click={applyFilters}>Apply</button>
          <button class="clear-btn" on:click={clearFilters}>Clear</button>
        </div>
      </div>
    {/if}

    {#if error}
      <div class="error-message">
        <span class="error-icon">⚠️</span>
        {error}
      </div>
    {/if}

    <!-- Main Content -->
    <div class="main-content">
      <!-- Results List -->
      <div class="results-list">
        <div class="results-header">
          <span class="results-count">
            {totalResults} {totalResults === 1 ? 'result' : 'results'}
          </span>
        </div>

        <div class="results-container">
          {#if loading}
            <div class="loading-state">
              <div class="spinner" />
              <p>Loading...</p>
            </div>
          {:else if searchResults.length === 0}
            <div class="empty-state">
              <p>No results found</p>
              <p class="hint">Try adjusting your search or filters</p>
            </div>
          {:else}
            {#each searchResults as file (file.id)}
              <div
                class="result-item"
                class:selected={selectedFile?.id === file.id}
                on:click={() => handleSelectFile(file.id)}
                role="button"
                tabindex="0"
              >
                <div class="result-info">
                  <div class="result-filename">{file.filename}</div>
                  <div class="result-details">
                    {#if file.bpm}
                      <span class="detail-badge">{file.bpm} BPM</span>
                    {/if}
                    {#if file.key}
                      <span class="detail-badge">{file.key}</span>
                    {/if}
                    <span class="detail-badge">{formatDuration(file.duration)}</span>
                    <span class="detail-badge">{file.trackCount} tracks</span>
                  </div>
                </div>
                <button
                  class="favorite-btn"
                  class:favorited={file.favorite}
                  on:click|stopPropagation={() => handleToggleFavorite(file.id)}
                  title={file.favorite ? 'Remove from favorites' : 'Add to favorites'}
                >
                  {file.favorite ? '★' : '☆'}
                </button>
              </div>
            {/each}
          {/if}
        </div>

        <!-- Pagination -->
        {#if $hasResults && $totalPages > 1}
          <div class="pagination">
            <button
              class="page-btn"
              on:click={handlePreviousPage}
              disabled={!$hasPreviousPage || loading}
            >
              Previous
            </button>
            <span class="page-info">
              Page {currentPage} of {$totalPages}
            </span>
            <button
              class="page-btn"
              on:click={handleNextPage}
              disabled={!$hasNextPage || loading}
            >
              Next
            </button>
          </div>
        {/if}
      </div>

      <!-- File Details -->
      <div class="file-details">
        {#if selectedFile}
          <div class="details-header">
            <h3 class="details-title">File Details</h3>
            <button
              class="preview-btn"
              on:click={handlePreview}
            >
              {previewPlaying ? 'Stop' : 'Preview'}
            </button>
          </div>

          <div class="details-content">
            <div class="detail-row">
              <span class="detail-label">Filename:</span>
              <span class="detail-value">{selectedFile.filename}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Path:</span>
              <span class="detail-value path">{selectedFile.filepath}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Duration:</span>
              <span class="detail-value">{formatDuration(selectedFile.duration)}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">BPM:</span>
              <span class="detail-value">{selectedFile.bpm || 'N/A'}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Key:</span>
              <span class="detail-value">{selectedFile.key || 'N/A'}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Time Signature:</span>
              <span class="detail-value">{selectedFile.timeSignature || 'N/A'}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Tracks:</span>
              <span class="detail-value">{selectedFile.trackCount}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Notes:</span>
              <span class="detail-value">{selectedFile.noteCount.toLocaleString()}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">File Size:</span>
              <span class="detail-value">{formatFileSize(selectedFile.fileSize)}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Imported:</span>
              <span class="detail-value">{formatDate(selectedFile.importedAt)}</span>
            </div>

            {#if selectedFile.tags.length > 0}
              <div class="detail-row">
                <span class="detail-label">Tags:</span>
                <div class="tags-container">
                  {#each selectedFile.tags as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {:else}
          <div class="no-selection">
            <p>Select a file to view details</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</WindowBase>

<style>
  .database-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Search Bar */
  .search-bar {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .search-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    font-size: 14px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .search-btn,
  .filter-btn,
  .import-btn {
    padding: 8px 16px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s ease;
  }

  .search-btn:hover,
  .filter-btn:hover,
  .import-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .filter-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .search-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Filters Panel */
  .filters-panel {
    padding: 16px;
    background: var(--panel-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .filter-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .filter-inputs {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-input {
    width: 80px;
    padding: 6px 10px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
  }

  .filter-checkboxes {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
  }

  .filter-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .apply-btn,
  .clear-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .apply-btn {
    background: var(--accent-color, #0078d4);
    color: white;
  }

  .clear-btn {
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
  }

  /* Error Message */
  .error-message {
    padding: 12px 16px;
    background: rgba(232, 17, 35, 0.1);
    color: #e81123;
    border-bottom: 1px solid #e81123;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* Main Content */
  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* Results List */
  .results-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .results-header {
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .results-count {
    font-size: 13px;
    color: var(--text-secondary, #888);
  }

  .results-container {
    flex: 1;
    overflow-y: auto;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary, #888);
    gap: 12px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--border-color, #3e3e3e);
    border-top-color: var(--accent-color, #0078d4);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .hint {
    font-size: 13px;
    color: var(--text-tertiary, #666);
  }

  .result-item {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    transition: background 0.15s ease;
  }

  .result-item:hover {
    background: var(--item-hover, #252525);
  }

  .result-item.selected {
    background: var(--item-selected, #2a2a2a);
    border-left: 3px solid var(--accent-color, #0078d4);
  }

  .result-info {
    flex: 1;
  }

  .result-filename {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 4px;
  }

  .result-details {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .detail-badge {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--badge-bg, #3a3a3a);
    color: var(--text-secondary, #888);
    border-radius: 4px;
  }

  .favorite-btn {
    font-size: 20px;
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    cursor: pointer;
    padding: 4px;
  }

  .favorite-btn.favorited {
    color: #f7dc6f;
  }

  /* Pagination */
  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .page-btn {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 13px;
    color: var(--text-secondary, #888);
  }

  /* File Details */
  .file-details {
    width: 350px;
    background: var(--details-bg, #252525);
    display: flex;
    flex-direction: column;
  }

  .details-header {
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .details-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .preview-btn {
    padding: 6px 12px;
    background: var(--accent-color, #0078d4);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .details-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .detail-row {
    display: flex;
    margin-bottom: 12px;
    gap: 8px;
  }

  .detail-label {
    font-size: 13px;
    color: var(--text-secondary, #888);
    min-width: 100px;
  }

  .detail-value {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    word-break: break-word;
  }

  .detail-value.path {
    font-family: monospace;
    font-size: 11px;
  }

  .tags-container {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag {
    font-size: 11px;
    padding: 4px 8px;
    background: var(--accent-color, #0078d4);
    color: white;
    border-radius: 4px;
  }

  .no-selection {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary, #888);
  }

  /* Scrollbars */
  .results-container::-webkit-scrollbar,
  .details-content::-webkit-scrollbar {
    width: 12px;
  }

  .results-container::-webkit-scrollbar-track,
  .details-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .results-container::-webkit-scrollbar-thumb,
  .details-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
