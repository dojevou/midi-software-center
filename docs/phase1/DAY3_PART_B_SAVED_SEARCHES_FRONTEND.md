# Day 3, Part 3B: Saved Searches Frontend

**Duration:** 1.5 hours
**Prerequisites:** Day 3 Part 3A completed
**Files to create/modify:** 2

---

## Overview

You've completed the saved searches backend (Part 3A). Now you'll:
1. Create savedSearchesApi.ts
2. Build VIP3SavedSearches component
3. Add "Save Current Search" button
4. Show popular searches in sidebar

---

## Step 1: Saved Searches API (20 min)

Create `app/src/lib/api/savedSearchesApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Vip3Filters } from '$lib/types/vip3';

export interface SavedSearch {
  id: number;
  name: string;
  description?: string;
  filters: Vip3Filters;
  use_count: number;
  created_at: string;
  updated_at: string;
}

export class SavedSearchesApi {
  /**
   * Get all saved searches
   */
  static async getAll(): Promise<SavedSearch[]> {
    try {
      return await invoke<SavedSearch[]>('get_saved_searches');
    } catch (error) {
      console.error('Failed to get saved searches:', error);
      throw new Error(`Failed to load saved searches: ${error}`);
    }
  }

  /**
   * Get a saved search by ID
   */
  static async getById(id: number): Promise<SavedSearch> {
    try {
      return await invoke<SavedSearch>('get_saved_search', { id });
    } catch (error) {
      console.error('Failed to get saved search:', error);
      throw new Error(`Failed to load saved search: ${error}`);
    }
  }

  /**
   * Create a new saved search
   */
  static async create(
    name: string,
    filters: Vip3Filters,
    description?: string
  ): Promise<SavedSearch> {
    try {
      return await invoke<SavedSearch>('create_saved_search', {
        name,
        description,
        filters
      });
    } catch (error) {
      console.error('Failed to create saved search:', error);
      throw new Error(`Failed to save search: ${error}`);
    }
  }

  /**
   * Update a saved search
   */
  static async update(
    id: number,
    name?: string,
    description?: string,
    filters?: Vip3Filters
  ): Promise<SavedSearch> {
    try {
      return await invoke<SavedSearch>('update_saved_search', {
        id,
        name,
        description,
        filters
      });
    } catch (error) {
      console.error('Failed to update saved search:', error);
      throw new Error(`Failed to update search: ${error}`);
    }
  }

  /**
   * Delete a saved search
   */
  static async delete(id: number): Promise<void> {
    try {
      await invoke('delete_saved_search', { id });
    } catch (error) {
      console.error('Failed to delete saved search:', error);
      throw new Error(`Failed to delete search: ${error}`);
    }
  }

  /**
   * Load a saved search (increments use_count)
   * @returns The filter state to apply
   */
  static async load(id: number): Promise<Vip3Filters> {
    try {
      const filters = await invoke<Vip3Filters>('load_saved_search', { id });
      console.log(`Loaded saved search ${id}`);
      return filters;
    } catch (error) {
      console.error('Failed to load saved search:', error);
      throw new Error(`Failed to load search: ${error}`);
    }
  }

  /**
   * Get popular saved searches (by use_count)
   */
  static async getPopular(limit: number = 10): Promise<SavedSearch[]> {
    try {
      return await invoke<SavedSearch[]>('get_popular_saved_searches', {
        limit
      });
    } catch (error) {
      console.error('Failed to get popular searches:', error);
      return [];
    }
  }
}
```

---

## Step 2: VIP3SavedSearches Component (1 hour)

Create `app/src/lib/components/VIP3/VIP3SavedSearches.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { SavedSearchesApi, type SavedSearch } from '$lib/api/savedSearchesApi';
  import { vip3Actions, vip3Filters, hasActiveFilters } from '$lib/stores/vip3Store';

  let savedSearches: SavedSearch[] = [];
  let popularSearches: SavedSearch[] = [];
  let loading = false;
  let showSaveDialog = false;
  let saveName = '';
  let saveDescription = '';
  let saveError = '';

  onMount(async () => {
    await loadSavedSearches();
  });

  async function loadSavedSearches() {
    loading = true;
    try {
      [savedSearches, popularSearches] = await Promise.all([
        SavedSearchesApi.getAll(),
        SavedSearchesApi.getPopular(5)
      ]);
    } catch (error) {
      console.error('Failed to load saved searches:', error);
    } finally {
      loading = false;
    }
  }

  async function loadSearch(id: number) {
    try {
      const filters = await SavedSearchesApi.load(id);

      // Apply filters to VIP3 store
      vip3Filters.set(filters);

      // Refresh counts and results
      await vip3Actions.refreshCounts();
      await vip3Actions.search();

      // Reload searches to update use_count
      await loadSavedSearches();
    } catch (error) {
      alert(`Failed to load search: ${error}`);
    }
  }

  async function deleteSearch(id: number, name: string) {
    if (!confirm(`Delete saved search "${name}"?`)) return;

    try {
      await SavedSearchesApi.delete(id);
      await loadSavedSearches();
    } catch (error) {
      alert(`Failed to delete search: ${error}`);
    }
  }

  async function saveCurrentSearch() {
    if (!saveName.trim()) {
      saveError = 'Name is required';
      return;
    }

    try {
      await SavedSearchesApi.create(saveName, $vip3Filters, saveDescription || undefined);

      // Reset form
      saveName = '';
      saveDescription = '';
      showSaveDialog = false;
      saveError = '';

      // Reload list
      await loadSavedSearches();
    } catch (error) {
      saveError = String(error);
    }
  }

  function openSaveDialog() {
    showSaveDialog = true;
    saveError = '';
  }
</script>

<div class="saved-searches">
  <div class="header">
    <h3>Saved Searches</h3>
    <button
      class="save-btn"
      on:click={openSaveDialog}
      disabled={!$hasActiveFilters}
      title={$hasActiveFilters ? 'Save current search' : 'Apply filters first'}
    >
      + Save Current
    </button>
  </div>

  {#if loading}
    <p class="loading">Loading...</p>
  {:else}
    <!-- Popular Searches -->
    {#if popularSearches.length > 0}
      <div class="section">
        <h4 class="section-title">Popular</h4>
        <div class="search-list">
          {#each popularSearches as search (search.id)}
            <div class="search-item">
              <button class="search-load" on:click={() => loadSearch(search.id)}>
                <div class="search-info">
                  <span class="search-name">{search.name}</span>
                  {#if search.description}
                    <span class="search-desc">{search.description}</span>
                  {/if}
                </div>
                <span class="use-count">{search.use_count} uses</span>
              </button>
              <button
                class="delete-btn"
                on:click|stopPropagation={() => deleteSearch(search.id, search.name)}
              >
                ×
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- All Searches -->
    <div class="section">
      <h4 class="section-title">All Searches ({savedSearches.length})</h4>
      <div class="search-list">
        {#each savedSearches as search (search.id)}
          <div class="search-item">
            <button class="search-load" on:click={() => loadSearch(search.id)}>
              <div class="search-info">
                <span class="search-name">{search.name}</span>
                {#if search.description}
                  <span class="search-desc">{search.description}</span>
                {/if}
              </div>
              <span class="use-count">{search.use_count}</span>
            </button>
            <button
              class="delete-btn"
              on:click|stopPropagation={() => deleteSearch(search.id, search.name)}
            >
              ×
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Save Dialog -->
  {#if showSaveDialog}
    <div class="dialog-overlay" on:click={() => (showSaveDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Save Current Search</h3>

        <label>
          Name *
          <input
            type="text"
            bind:value={saveName}
            placeholder="e.g., High Energy Drums"
            maxlength="100"
          />
        </label>

        <label>
          Description
          <textarea
            bind:value={saveDescription}
            placeholder="Optional description..."
            rows="3"
            maxlength="500"
          />
        </label>

        {#if saveError}
          <p class="error">{saveError}</p>
        {/if}

        <div class="dialog-actions">
          <button class="btn-cancel" on:click={() => (showSaveDialog = false)}>
            Cancel
          </button>
          <button class="btn-save" on:click={saveCurrentSearch}>Save</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .saved-searches {
    padding: 16px;
    background: #1a1a1a;
    border-right: 1px solid #333;
    min-width: 280px;
    max-width: 320px;
    overflow-y: auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .save-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .save-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .save-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .section {
    margin-bottom: 24px;
  }

  .section-title {
    font-size: 13px;
    color: #999;
    text-transform: uppercase;
    margin: 0 0 8px 0;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .search-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .search-item {
    display: flex;
    align-items: stretch;
    background: #252525;
    border-radius: 4px;
    overflow: hidden;
  }

  .search-load {
    flex: 1;
    padding: 10px 12px;
    background: transparent;
    border: none;
    color: #ccc;
    cursor: pointer;
    text-align: left;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .search-load:hover {
    background: #2a2a2a;
  }

  .search-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .search-name {
    font-size: 14px;
    color: #fff;
    font-weight: 500;
  }

  .search-desc {
    font-size: 12px;
    color: #999;
  }

  .use-count {
    font-size: 12px;
    color: #666;
    font-weight: 500;
  }

  .delete-btn {
    padding: 0 12px;
    background: transparent;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
  }

  .delete-btn:hover {
    background: #991b1b;
    color: #fff;
  }

  .loading {
    color: #999;
    text-align: center;
    padding: 24px;
  }

  /* Dialog styles */
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
    background: #2a2a2a;
    padding: 24px;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .dialog h3 {
    margin: 0 0 20px 0;
    color: #fff;
    font-size: 18px;
  }

  .dialog label {
    display: block;
    margin-bottom: 16px;
    color: #ccc;
    font-size: 14px;
  }

  .dialog input,
  .dialog textarea {
    width: 100%;
    margin-top: 6px;
    padding: 8px 12px;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #fff;
    font-size: 14px;
    font-family: inherit;
  }

  .dialog input:focus,
  .dialog textarea:focus {
    outline: none;
    border-color: #3b82f6;
  }

  .error {
    color: #ef4444;
    font-size: 13px;
    margin: 8px 0;
  }

  .dialog-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .btn-cancel,
  .btn-save {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
  }

  .btn-cancel {
    background: #444;
    color: #fff;
  }

  .btn-cancel:hover {
    background: #555;
  }

  .btn-save {
    background: #3b82f6;
    color: #fff;
  }

  .btn-save:hover {
    background: #2563eb;
  }
</style>
```

---

## Step 3: Update VIP3Browser to Include Saved Searches (10 min)

Update `app/src/lib/components/VIP3/VIP3Browser.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import VIP3Column from './VIP3Column.svelte';
  import VIP3BpmColumn from './VIP3BpmColumn.svelte';
  import VIP3SavedSearches from './VIP3SavedSearches.svelte';
  import {
    vip3Actions,
    timbres,
    styles,
    articulations,
    loadingCounts,
    totalMatches
  } from '$lib/stores/vip3Store';

  // ... existing folder/instrument mocks ...

  onMount(async () => {
    await vip3Actions.initialize();
  });
</script>

<div class="vip3-browser">
  <div class="browser-header">
    <h2>VIP3 Browser</h2>
    {#if $loadingCounts}
      <span class="loading">Loading counts...</span>
    {:else}
      <span class="total-matches">
        {$totalMatches.toLocaleString()} files
      </span>
    {/if}
  </div>

  <div class="browser-content">
    <!-- Saved Searches Sidebar -->
    <VIP3SavedSearches />

    <!-- Filter Columns -->
    <div class="filter-columns">
      <VIP3Column
        title="Folders"
        items={folders}
        filterKey="folder_ids"
        countKey="folder_counts"
      />

      <VIP3Column
        title="Instruments"
        items={instruments}
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

    <!-- Results Panel -->
    <div class="results-panel">
      <p class="placeholder">Search results will appear here</p>
    </div>
  </div>
</div>

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
    background: #1f1f1f;
  }

  .browser-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .filter-columns {
    display: flex;
    border-right: 1px solid #333;
    background: #1a1a1a;
    overflow-x: auto;
  }

  .results-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  /* ... rest of styles ... */
</style>
```

---

## Verification (10 min)

### Test Flow

```bash
make dev
```

1. **Save a Search:**
   - Apply some filters in VIP3 browser
   - Click "+ Save Current" button
   - Enter name "Test Search"
   - Click Save
   - Verify search appears in "All Searches" list

2. **Load a Search:**
   - Click on a saved search
   - Verify filters are applied
   - Verify filter counts update
   - Verify use_count increments (check in database or reload UI)

3. **Delete a Search:**
   - Click "×" button on a search
   - Confirm deletion
   - Verify search removed from list

4. **Popular Searches:**
   - Load same search multiple times
   - Verify it appears in "Popular" section
   - Verify use_count increases

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Save Current" button disabled | Apply filters first (folder, instrument, etc.) |
| Duplicate name error | Each search name must be unique |
| Filters not applying on load | Check `vip3Filters.set()` is called in `loadSearch()` |
| Use count not updating | Verify `load_saved_search` command increments count |

---

## What's Next?

✅ **Day 3 Complete! Saved searches system implemented:**
- ✅ Part 3A: Backend models, repository, commands
- ✅ Part 3B: Frontend API, UI component, save/load flow

**Next:** [Day 4, Part 4A: Collections Backend](./DAY4_PART_A_COLLECTIONS_BACKEND.md)
- Create Collection and CollectionFile models
- Implement CollectionRepository with ordering
- Add Tauri commands for collections management
- Support drag-and-drop file ordering

**Current Features:**
- Save current filter combinations ✓
- Load saved searches ✓
- Track usage with use_count ✓
- Show popular searches ✓
- Delete unwanted searches ✓
