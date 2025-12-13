<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SavedSearch, VIP3Filters } from '$lib/stores/vip3Store';

  export let savedSearches: SavedSearch[] = [];
  export let recentSearches: VIP3Filters[] = [];
  export let currentFilters: VIP3Filters | null = null;
  export let collapsed: boolean = false;

  let showSaveDialog = false;
  let newSearchName = '';
  let editingSearchId: number | null = null;

  const dispatch = createEventDispatcher<{
    save: { name: string };
    load: { id: number };
    delete: { id: number };
    loadRecent: { filters: VIP3Filters };
    toggle: { collapsed: boolean };
  }>();

  function handleSave() {
    if (newSearchName.trim()) {
      dispatch('save', { name: newSearchName.trim() });
      newSearchName = '';
      showSaveDialog = false;
    }
  }

  function handleLoad(id: number) {
    dispatch('load', { id });
  }

  function handleDelete(id: number, event: MouseEvent) {
    event.stopPropagation();
    if (confirm('Delete this saved search?')) {
      dispatch('delete', { id });
    }
  }

  function handleLoadRecent(filters: VIP3Filters) {
    dispatch('loadRecent', { filters });
  }

  function handleToggle() {
    dispatch('toggle', { collapsed: !collapsed });
  }

  function formatFiltersPreview(filters: VIP3Filters): string {
    const parts: string[] = [];
    if (filters.searchQuery) parts.push(`"${filters.searchQuery}"`);
    if (filters.timbreIds.length) parts.push(`${filters.timbreIds.length} timbres`);
    if (filters.styleIds.length) parts.push(`${filters.styleIds.length} styles`);
    if (filters.articulationIds.length) parts.push(`${filters.articulationIds.length} articulations`);
    if (filters.bpmRangeId) parts.push('BPM');
    if (filters.keyIds.length) parts.push(`${filters.keyIds.length} keys`);
    return parts.length > 0 ? parts.join(' \u2022 ') : 'All files';
  }

  $: hasActiveFilters = currentFilters && (
    currentFilters.searchQuery ||
    currentFilters.timbreIds.length > 0 ||
    currentFilters.styleIds.length > 0 ||
    currentFilters.articulationIds.length > 0 ||
    currentFilters.bpmRangeId !== null ||
    currentFilters.keyIds.length > 0
  );
</script>

<div class="saved-searches" class:collapsed>
  <button type="button" class="panel-header" on:click={handleToggle}>
    <span class="header-title">
      <span class="header-icon">💾</span>
      <span class="header-text">Saved Searches</span>
    </span>
    {#if savedSearches.length > 0}
      <span class="count-badge">{savedSearches.length}</span>
    {/if}
    <span class="collapse-icon">{collapsed ? '▶' : '▼'}</span>
  </button>

  {#if !collapsed}
    <div class="panel-content">
      <!-- Save Current Search Button -->
      {#if hasActiveFilters && !showSaveDialog}
        <button
          type="button"
          class="save-current-btn"
          on:click={() => { showSaveDialog = true; }}
        >
          <span class="btn-icon">➕</span>
          <span class="btn-text">Save Current Search</span>
        </button>
      {/if}

      <!-- Save Dialog -->
      {#if showSaveDialog}
        <div class="save-dialog">
          <input
            type="text"
            class="save-input"
            placeholder="Enter search name..."
            bind:value={newSearchName}
            on:keydown={(e) => {
              if (e.key === 'Enter') handleSave();
              if (e.key === 'Escape') { showSaveDialog = false; newSearchName = ''; }
            }}
          />
          <div class="save-actions">
            <button type="button" class="action-btn save" on:click={handleSave}>
              Save
            </button>
            <button
              type="button"
              class="action-btn cancel"
              on:click={() => { showSaveDialog = false; newSearchName = ''; }}
            >
              Cancel
            </button>
          </div>
        </div>
      {/if}

      <!-- Saved Searches List -->
      {#if savedSearches.length > 0}
        <div class="searches-section">
          <span class="section-label">Saved</span>
          <ul class="searches-list">
            {#each savedSearches as search (search.id)}
              <li class="search-item">
                <button
                  type="button"
                  class="search-btn"
                  on:click={() => handleLoad(search.id)}
                  title={formatFiltersPreview(search.filters)}
                >
                  <span class="search-icon">🔖</span>
                  <span class="search-name">{search.name}</span>
                </button>
                <button
                  type="button"
                  class="delete-btn"
                  on:click={(e) => handleDelete(search.id, e)}
                  title="Delete search"
                >
                  ✕
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      <!-- Recent Searches -->
      {#if recentSearches.length > 0}
        <div class="searches-section">
          <span class="section-label">Recent</span>
          <ul class="searches-list">
            {#each recentSearches.slice(0, 5) as filters, i (i)}
              <li class="search-item">
                <button
                  type="button"
                  class="search-btn recent"
                  on:click={() => handleLoadRecent(filters)}
                >
                  <span class="search-icon">🕐</span>
                  <span class="search-name">{formatFiltersPreview(filters)}</span>
                </button>
              </li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if savedSearches.length === 0 && recentSearches.length === 0}
        <div class="empty-state">
          <span class="empty-icon">📭</span>
          <span class="empty-text">No saved searches yet</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .saved-searches {
    display: flex;
    flex-direction: column;
    background: var(--gray-900, #18181b);
    border: 1px solid var(--gray-700, #3f3f46);
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .saved-searches.collapsed {
    border-radius: 8px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--gray-800, #27272a);
    border: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    color: inherit;
    transition: background-color 0.15s ease;
  }

  .panel-header:hover {
    background: var(--gray-700, #3f3f46);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
  }

  .header-icon {
    font-size: 12px;
  }

  .header-text {
    font-size: 12px;
    font-weight: 600;
    color: var(--gray-200, #e4e4e7);
  }

  .count-badge {
    background: var(--primary-600, #2563eb);
    color: white;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 10px;
  }

  .collapse-icon {
    font-size: 10px;
    color: var(--gray-500, #71717a);
  }

  .panel-content {
    padding: 8px;
  }

  .save-current-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 8px 10px;
    background: var(--gray-800, #27272a);
    border: 1px dashed var(--gray-600, #52525b);
    border-radius: 6px;
    color: var(--gray-400, #a1a1aa);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
    margin-bottom: 8px;
  }

  .save-current-btn:hover {
    background: var(--gray-700, #3f3f46);
    border-color: var(--primary-500, #3b82f6);
    color: var(--gray-200, #e4e4e7);
  }

  .btn-icon {
    font-size: 12px;
  }

  .save-dialog {
    padding: 8px;
    background: var(--gray-800, #27272a);
    border-radius: 6px;
    margin-bottom: 8px;
  }

  .save-input {
    width: 100%;
    padding: 8px 10px;
    background: var(--gray-700, #3f3f46);
    border: 1px solid var(--gray-600, #52525b);
    border-radius: 4px;
    color: var(--gray-100, #f4f4f5);
    font-size: 12px;
    margin-bottom: 8px;
  }

  .save-input:focus {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }

  .save-input::placeholder {
    color: var(--gray-500, #71717a);
  }

  .save-actions {
    display: flex;
    gap: 6px;
  }

  .action-btn {
    flex: 1;
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .action-btn.save {
    background: var(--primary-600, #2563eb);
    color: white;
  }

  .action-btn.save:hover {
    background: var(--primary-500, #3b82f6);
  }

  .action-btn.cancel {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-300, #d4d4d8);
  }

  .action-btn.cancel:hover {
    background: var(--gray-600, #52525b);
  }

  .searches-section {
    margin-bottom: 8px;
  }

  .searches-section:last-child {
    margin-bottom: 0;
  }

  .section-label {
    display: block;
    font-size: 10px;
    font-weight: 600;
    color: var(--gray-500, #71717a);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 4px 8px;
    margin-bottom: 4px;
  }

  .searches-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .search-item {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 2px;
  }

  .search-btn {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--gray-300, #d4d4d8);
    font-size: 11px;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.15s ease;
    overflow: hidden;
  }

  .search-btn:hover {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-100, #f4f4f5);
  }

  .search-icon {
    font-size: 11px;
    flex-shrink: 0;
  }

  .search-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-btn.recent {
    color: var(--gray-500, #71717a);
  }

  .search-btn.recent:hover {
    color: var(--gray-300, #d4d4d8);
  }

  .delete-btn {
    background: transparent;
    border: none;
    color: var(--gray-600, #52525b);
    padding: 4px 6px;
    font-size: 10px;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
    opacity: 0;
  }

  .search-item:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    background: var(--red-900, #7f1d1d);
    color: var(--red-400, #f87171);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    color: var(--gray-500, #71717a);
  }

  .empty-icon {
    font-size: 24px;
    opacity: 0.5;
  }

  .empty-text {
    font-size: 11px;
  }
</style>
