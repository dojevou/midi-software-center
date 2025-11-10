<script lang="ts">
  import { onMount } from 'svelte';
  import { databaseStore, databaseActions, totalPages } from '$lib/stores/databaseStore';
  import { projectStore, projectActions } from '$lib/stores/projectStore';
  import { formatBPM } from '$lib/utils/formatters';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import type { FileDetails } from '$lib/types';

  let searchQuery = '';
  let searchTimeout: NodeJS.Timeout;
  let selectedFile: FileDetails | null = null;
  let isLoading = false;

  // Reactive
  $: searchResults = $databaseStore.searchResults;
  $: currentPage = $databaseStore.currentPage;
  $: isLoading = $databaseStore.isLoading;

  // Debounced search
  $: if (searchQuery) {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(async () => {
      await databaseActions.search({ search_text: searchQuery.trim() });
    }, 300);
  } else {
    clearTimeout(searchTimeout);
    databaseActions.clearSearch();
  }

  onMount(async () => {
    // Initial load
    await databaseActions.search();
  });

  async function handleSearch(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      clearTimeout(searchTimeout);
      await databaseActions.search({ search_text: searchQuery.trim() });
    }
  }

  async function handleDoubleClick(file: FileDetails) {
    selectedFile = file;
    // Load into sequencer - assign next channel or default 0
    const channel = $projectStore.tracks.length; // Simple auto-channel
    await projectActions.addTrack(file.id, channel);
  }

  async function nextPage() {
    await databaseActions.nextPage();
  }

  async function previousPage() {
    await databaseActions.previousPage();
  }

  function selectFile(file: FileDetails) {
    selectedFile = file;
  }
</script>

<WindowBase windowId="database" title="Database" width={800} height={600} resizable={true}>
  <div class="database-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
    <!-- Search Bar -->
    <div class="search-bar dark:bg-menu p-3 rounded mb-4">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search MIDI files..."
        on:keydown={handleSearch}
        class="search-input dark:bg-input dark:text-app-text px-4 py-2 rounded w-full"
        disabled={isLoading}
      />
      {#if isLoading}
        <div class="loading dark:text-gray-400 mt-2">Searching...</div>
      {/if}
    </div>

    <!-- Results List -->
    <div class="results-list flex-1 overflow-auto dark:bg-window-subtle rounded border dark:border-window-border mb-4">
      {#if searchResults.length === 0 && !isLoading}
        <div class="no-results dark:text-gray-400 p-4 text-center">No results found</div>
      {:else}
        {#each searchResults as file (file.id)}
          <div
            class="result-item dark:bg-window hover:dark:bg-hover p-3 border-b dark:border-window-border cursor-pointer flex items-center space-x-4"
            class:selected={selectedFile?.id === file.id}
            on:click={() => selectFile(file)}
            on:dblclick={() => handleDoubleClick(file)}
          >
            <div class="file-icon dark:text-primary">🎹</div>
            <div class="file-info flex-1">
              <div class="file-name font-medium dark:text-app-text">{file.file_name}</div>
              <div class="file-meta dark:text-gray-400 text-sm">
                BPM: {formatBPM(file.bpm)} | Key: {file.key || 'N/A'}
              </div>
            </div>
            {#if file.is_favorite}
              <div class="favorite dark:text-primary">★</div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <!-- Pagination -->
    {#if $totalPages > 1}
      <div class="pagination dark:bg-menu p-3 rounded flex items-center justify-between">
        <div class="page-info dark:text-gray-300">
          Page {currentPage + 1} of {$totalPages}
        </div>
        <div class="controls flex space-x-2">
          <button
            class="prev-btn dark:bg-secondary dark:text-white px-4 py-2 rounded hover:dark:bg-secondary-dark"
            on:click={previousPage}
            disabled={currentPage === 0 || isLoading}
          >
            Previous
          </button>
          <button
            class="next-btn dark:bg-primary dark:text-white px-4 py-2 rounded hover:dark:bg-primary-dark"
            on:click={nextPage}
            disabled={currentPage === $totalPages - 1 || isLoading}
          >
            Next
          </button>
        </div>
      </div>
    {/if}
  </div>

  <style>
    .database-window {
      height: 100%;
    }
    .selected {
      background-color: var(--primary-color) !important;
      color: white !important;
    }
    .prev-btn:disabled, .next-btn:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    .search-input:disabled {
      opacity: 0.7;
      cursor: wait;
    }
  </style>
</WindowBase>