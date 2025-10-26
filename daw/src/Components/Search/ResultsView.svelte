<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { writable } from 'svelte/store';

  import FileCard from './FileCard.svelte';
  import { viewMode } from '../../lib/stores';

  export let searchQuery: string;
  export let filters: any;

  interface MidiFile {
    id: number;
    file_name: string;
    file_path: string;
    category: string | null;
    bpm: number | null;
    key_signature: string | null;
    time_signature: string | null;
    duration_seconds: number;
    instruments: string[];
    created_at: string;
  }

  let results: MidiFile[] = [];
  let loading = false;
  let error = '';
  let totalResults = 0;
  let currentPage = 1;
  let pageSize = 50;

  async function performSearch() {
    loading = true;
    error = '';

    try {
      // Convert page to offset for backend
      const offset = (currentPage - 1) * pageSize;

      const response = await invoke<{ files: MidiFile[], total: number }>('search_files', {
        filters: {
          ...filters,
          search_text: searchQuery || null,
          limit: pageSize,
          offset: offset
        }
      });

      results = response.files;
      totalResults = response.total;
    } catch (err) {
      console.error('Search failed:', err);
      error = `Search failed: ${err}`;
      results = [];
      totalResults = 0;
    } finally {
      loading = false;
    }
  }

  function handleFileSelect(event: CustomEvent<{ fileId: number }>) {
    console.log('File selected:', event.detail.fileId);
    // TODO: Handle file selection
  }

  function handleFilePlay(event: CustomEvent<{ fileId: number }>) {
    console.log('File play:', event.detail.fileId);
    // TODO: Trigger MIDI playback
  }

  function handlePageChange(page: number) {
    currentPage = page;
    performSearch();
  }

  onMount(() => {
    performSearch();
  });

  $: {
    // React to search query or filter changes
    if (searchQuery !== undefined || filters !== undefined) {
      currentPage = 1;
      performSearch();
    }
  }
</script>

<div class="results-view">
  <div class="results-header">
    <div class="results-info">
      {#if loading}
        <span class="info-text">Searching...</span>
      {:else if error}
        <span class="info-text error">{error}</span>
      {:else}
        <span class="info-text">
          {totalResults.toLocaleString()} result{totalResults !== 1 ? 's' : ''}
        </span>
      {/if}
    </div>

    <div class="view-controls">
      <button
        class="view-btn"
        class:active={$viewMode === 'grid'}
        on:click={() => $viewMode = 'grid'}
        title="Grid view"
      >
        ⊞
      </button>
      <button
        class="view-btn"
        class:active={$viewMode === 'list'}
        on:click={() => $viewMode = 'list'}
        title="List view"
      >
        ☰
      </button>
    </div>
  </div>

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Searching files...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <div class="error-icon">⚠️</div>
      <p>{error}</p>
      <button class="btn-retry" on:click={performSearch}>Retry</button>
    </div>
  {:else if results.length === 0}
    <div class="empty-state">
      <div class="empty-icon">🔍</div>
      <h3>No results found</h3>
      <p>Try adjusting your search or filters</p>
    </div>
  {:else}
    <div class="results-container" class:grid-view={$viewMode === 'grid'} class:list-view={$viewMode === 'list'}>
      {#each results as file (file.id)}
        <FileCard
          {file}
          viewMode={$viewMode}
          on:select={handleFileSelect}
          on:play={handleFilePlay}
        />
      {/each}
    </div>

    {#if totalResults > pageSize}
      <div class="pagination">
        <button
          class="page-btn"
          disabled={currentPage === 1}
          on:click={() => handlePageChange(currentPage - 1)}
        >
          ← Previous
        </button>

        <span class="page-info">
          Page {currentPage} of {Math.ceil(totalResults / pageSize)}
        </span>

        <button
          class="page-btn"
          disabled={currentPage >= Math.ceil(totalResults / pageSize)}
          on:click={() => handlePageChange(currentPage + 1)}
        >
          Next →
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .results-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background: #1e1e1e;
    border-bottom: 1px solid #3d3d3d;
  }

  .results-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .info-text {
    font-size: 14px;
    color: #b0b0b0;
    font-weight: 500;
  }

  .info-text.error {
    color: #ff6b6b;
  }

  .view-controls {
    display: flex;
    gap: 4px;
  }

  .view-btn {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #b0b0b0;
    font-size: 18px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-btn:hover {
    background: #2d2d2d;
    border-color: #4a9eff;
    color: #4a9eff;
  }

  .view-btn.active {
    background: #4a9eff;
    border-color: #4a9eff;
    color: white;
  }

  .loading-state,
  .error-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: #b0b0b0;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid #3d3d3d;
    border-top-color: #4a9eff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-icon,
  .empty-icon {
    font-size: 64px;
  }

  .empty-state h3 {
    margin: 0;
    font-size: 20px;
    color: #e0e0e0;
  }

  .empty-state p {
    margin: 0;
    font-size: 14px;
  }

  .btn-retry {
    padding: 10px 20px;
    background: #4a9eff;
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-retry:hover {
    background: #357abd;
  }

  .results-container {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .results-container.grid-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 20px;
    align-content: start;
  }

  .results-container.list-view {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: #1e1e1e;
    border-top: 1px solid #3d3d3d;
  }

  .page-btn {
    padding: 8px 16px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .page-btn:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #4a9eff;
  }

  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 14px;
    color: #b0b0b0;
  }

  /* Custom scrollbar */
  .results-container::-webkit-scrollbar {
    width: 8px;
  }

  .results-container::-webkit-scrollbar-track {
    background: #1a1a1a;
  }

  .results-container::-webkit-scrollbar-thumb {
    background: #3d3d3d;
    border-radius: 4px;
  }
</style>
