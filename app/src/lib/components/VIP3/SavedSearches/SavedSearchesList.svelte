<script lang="ts">
  import { onMount } from 'svelte';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { SavedSearchResponse } from '$lib/types/vip3';
  import SavedSearchItem from './SavedSearchItem.svelte';
  import SaveSearchDialog from './SaveSearchDialog.svelte';

  export let onLoadSearch: (search: SavedSearchResponse) => void = () => {};

  let searches: SavedSearchResponse[] = [];
  let loading = true;
  let error: string | null = null;
  let showSaveDialog = false;

  async function loadSavedSearches() {
    try {
      loading = true;
      error = null;
      searches = await Vip3BrowserApi.getSavedSearches();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load saved searches';
      console.error('Error loading saved searches:', e);
    } finally {
      loading = false;
    }
  }

  async function handleLoadSearch(search: SavedSearchResponse) {
    try {
      // Load the search (increments use count)
      const updatedSearch = await Vip3BrowserApi.loadSavedSearch(search.id);

      // Update the local list
      searches = searches.map(s => s.id === updatedSearch.id ? updatedSearch : s);

      // Notify parent to apply filters
      onLoadSearch(updatedSearch);
    } catch (e) {
      console.error('Error loading search:', e);
      alert(`Failed to load search: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  async function handleDeleteSearch(searchId: number) {
    if (!confirm('Are you sure you want to delete this saved search?')) {
      return;
    }

    try {
      await Vip3BrowserApi.deleteSavedSearch(searchId);
      searches = searches.filter(s => s.id !== searchId);
    } catch (e) {
      console.error('Error deleting search:', e);
      alert(`Failed to delete search: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  async function handleTogglePin(searchId: number) {
    try {
      const newPinStatus = await Vip3BrowserApi.toggleSavedSearchPin(searchId);
      searches = searches.map(s =>
        s.id === searchId ? { ...s, is_pinned: newPinStatus } : s
      );
    } catch (e) {
      console.error('Error toggling pin:', e);
      alert(`Failed to toggle pin: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  function handleSaveSuccess() {
    showSaveDialog = false;
    loadSavedSearches(); // Reload list
  }

  onMount(() => {
    loadSavedSearches();
  });

  // Sort: pinned first, then by last used
  $: sortedSearches = [...searches].sort((a, b) => {
    if (a.is_pinned !== b.is_pinned) {
      return a.is_pinned ? -1 : 1;
    }
    const aTime = a.last_used ? new Date(a.last_used).getTime() : 0;
    const bTime = b.last_used ? new Date(b.last_used).getTime() : 0;
    return bTime - aTime;
  });
</script>

<div class="saved-searches-list">
  <div class="header">
    <h3>Saved Searches</h3>
    <button class="btn-new" on:click={() => showSaveDialog = true}>
      + New Search
    </button>
  </div>

  {#if loading}
    <div class="loading">Loading saved searches...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={loadSavedSearches}>Retry</button>
    </div>
  {:else if sortedSearches.length === 0}
    <div class="empty">
      <p>No saved searches yet</p>
      <p class="hint">Save your current filters to quickly access them later</p>
    </div>
  {:else}
    <div class="searches">
      {#each sortedSearches as search (search.id)}
        <SavedSearchItem
          {search}
          onLoad={() => handleLoadSearch(search)}
          onDelete={() => handleDeleteSearch(search.id)}
          onTogglePin={() => handleTogglePin(search.id)}
        />
      {/each}
    </div>
  {/if}
</div>

{#if showSaveDialog}
  <SaveSearchDialog
    onClose={() => showSaveDialog = false}
    onSave={handleSaveSuccess}
  />
{/if}

<style>
  .saved-searches-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 8px;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border, #3a3a3a);
  }

  h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text, #ffffff);
  }

  .btn-new {
    padding: 8px 16px;
    background: var(--color-primary, #007aff);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-new:hover {
    background: var(--color-primary-hover, #0056b3);
  }

  .loading,
  .error,
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    color: var(--color-text-secondary, #999);
  }

  .error {
    color: var(--color-error, #ff3b30);
  }

  .error button {
    margin-top: 12px;
    padding: 8px 16px;
    background: var(--color-primary, #007aff);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .empty .hint {
    margin-top: 8px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .searches {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
</style>
