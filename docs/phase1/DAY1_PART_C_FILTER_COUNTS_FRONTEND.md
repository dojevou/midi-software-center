# Day 1, Part 1C: Filter Counts Frontend API & Store

**Duration:** 2 hours
**Prerequisites:** Parts 1A and 1B completed
**Files to create/modify:** 3

---

## Overview

You've completed the Rust backend (Parts A & B). Now you'll:
1. Create TypeScript types matching Rust structs
2. Implement the VIP3 browser API
3. Create a Svelte store for filter state management
4. Add reactive filter count loading

---

## Step 1: TypeScript Types (30 min)

Create `app/src/lib/types/vip3.ts`:

```typescript
/**
 * VIP3 filter counts matching Rust FilterCounts struct
 */
export interface FilterCounts {
  folder_counts: Record<number, number>;
  instrument_counts: Record<number, number>;
  timbre_counts: Record<number, number>;
  style_counts: Record<number, number>;
  articulation_counts: Record<number, number>;
  bpm_counts: Record<string, number>; // "60-80", "100-120", etc.
  key_counts: Record<string, number>; // "C", "Am", etc.
  channel_counts: Record<string, number>; // "1", "2-4", etc.
  total_matches: number;
}

/**
 * VIP3 filter state matching Rust Vip3Filters struct
 */
export interface Vip3Filters {
  folder_ids?: number[];
  instrument_ids?: number[];
  timbre_ids?: number[];
  style_ids?: number[];
  articulation_ids?: number[];
  bpm_min?: number;
  bpm_max?: number;
  key_signatures?: string[];
  time_signatures?: string[];
  channel_count_min?: number;
  channel_count_max?: number;
  has_drums?: boolean;
  tags?: string[];
}

/**
 * VIP3 category types
 */
export interface Timbre {
  id: number;
  name: string;
  description?: string;
}

export interface Style {
  id: number;
  name: string;
  description?: string;
}

export interface Articulation {
  id: number;
  name: string;
  description?: string;
}

/**
 * VIP3 search results
 */
export interface Vip3SearchResult {
  id: number;
  file_name: string;
  file_path: string;
  folder_id: number;
  bpm?: number;
  key_signature?: string;
  time_signature?: string;
  duration_seconds?: number;
  instruments: string[];
  timbres: string[];
  styles: string[];
  articulations: string[];
}

/**
 * VIP3 search response with pagination
 */
export interface Vip3SearchResponse {
  results: Vip3SearchResult[];
  total_count: number;
  page: number;
  page_size: number;
  has_more: boolean;
}
```

---

## Step 2: VIP3 Browser API (45 min)

Create `app/src/lib/api/vip3BrowserApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type {
  FilterCounts,
  Vip3Filters,
  Vip3SearchResponse,
  Timbre,
  Style,
  Articulation
} from '$lib/types/vip3';

/**
 * VIP3 Browser API
 * Handles all VIP3-related Tauri command invocations
 */
export class Vip3BrowserApi {
  /**
   * Get filter counts for current filter state
   * @param filters Current filter selections
   * @returns Counts for all filter categories
   */
  static async getFilterCounts(filters: Vip3Filters): Promise<FilterCounts> {
    try {
      const start = performance.now();

      const counts = await invoke<FilterCounts>('get_vip3_filter_counts', {
        filters
      });

      const elapsed = performance.now() - start;

      if (elapsed > 50) {
        console.warn(`Filter counts took ${elapsed.toFixed(0)}ms (target: <50ms)`);
      } else {
        console.log(`Filter counts loaded in ${elapsed.toFixed(0)}ms`);
      }

      return counts;
    } catch (error) {
      console.error('Failed to get filter counts:', error);
      throw new Error(`Filter count error: ${error}`);
    }
  }

  /**
   * Search files with VIP3 filters
   * @param filters Filter selections
   * @param page Page number (0-indexed)
   * @param pageSize Number of results per page
   * @returns Search results with pagination info
   */
  static async searchFiles(
    filters: Vip3Filters,
    page: number = 0,
    pageSize: number = 50
  ): Promise<Vip3SearchResponse> {
    try {
      const response = await invoke<Vip3SearchResponse>('search_files_vip3', {
        filters,
        limit: pageSize,
        offset: page * pageSize
      });

      return {
        ...response,
        page,
        page_size: pageSize,
        has_more: response.total_count > (page + 1) * pageSize
      };
    } catch (error) {
      console.error('Failed to search files:', error);
      throw new Error(`Search error: ${error}`);
    }
  }

  /**
   * Get all available timbres
   */
  static async getTimbres(): Promise<Timbre[]> {
    try {
      return await invoke<Timbre[]>('get_all_timbres');
    } catch (error) {
      console.error('Failed to get timbres:', error);
      return [];
    }
  }

  /**
   * Get all available styles
   */
  static async getStyles(): Promise<Style[]> {
    try {
      return await invoke<Style[]>('get_all_styles');
    } catch (error) {
      console.error('Failed to get styles:', error);
      return [];
    }
  }

  /**
   * Get all available articulations
   */
  static async getArticulations(): Promise<Articulation[]> {
    try {
      return await invoke<Articulation[]>('get_all_articulations');
    } catch (error) {
      console.error('Failed to get articulations:', error);
      return [];
    }
  }

  /**
   * Load a file into the DAW sequencer
   * @param fileId Database file ID
   * @returns Track ID in sequencer
   */
  static async loadFileToDaw(fileId: number): Promise<number> {
    try {
      const trackId = await invoke<number>('load_file_to_daw', {
        fileId
      });

      console.log(`Loaded file ${fileId} to DAW as track ${trackId}`);
      return trackId;
    } catch (error) {
      console.error('Failed to load file to DAW:', error);
      throw new Error(`DAW load error: ${error}`);
    }
  }
}
```

---

## Step 3: Svelte Store for VIP3 State (45 min)

Create `app/src/lib/stores/vip3Store.ts`:

```typescript
import { writable, derived, get } from 'svelte/store';
import type {
  Vip3Filters,
  FilterCounts,
  Vip3SearchResponse,
  Timbre,
  Style,
  Articulation
} from '$lib/types/vip3';
import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';

/**
 * VIP3 Browser Store
 * Manages filter state, counts, and search results
 */

// Filter state
export const vip3Filters = writable<Vip3Filters>({});

// Filter counts (updated when filters change)
export const filterCounts = writable<FilterCounts | null>(null);

// Search results
export const searchResults = writable<Vip3SearchResponse | null>(null);

// Loading states
export const loadingCounts = writable<boolean>(false);
export const loadingResults = writable<boolean>(false);

// Category lists
export const timbres = writable<Timbre[]>([]);
export const styles = writable<Style[]>([]);
export const articulations = writable<Articulation[]>([]);

// Current page for pagination
export const currentPage = writable<number>(0);
export const pageSize = writable<number>(50);

/**
 * Derived store: Is any filter active?
 */
export const hasActiveFilters = derived(vip3Filters, ($filters) => {
  return Object.keys($filters).length > 0;
});

/**
 * Derived store: Total matches from filter counts
 */
export const totalMatches = derived(filterCounts, ($counts) => {
  return $counts?.total_matches ?? 0;
});

/**
 * Actions
 */
export const vip3Actions = {
  /**
   * Initialize VIP3 browser (load categories)
   */
  async initialize() {
    try {
      const [timbreList, styleList, articulationList] = await Promise.all([
        Vip3BrowserApi.getTimbres(),
        Vip3BrowserApi.getStyles(),
        Vip3BrowserApi.getArticulations()
      ]);

      timbres.set(timbreList);
      styles.set(styleList);
      articulations.set(articulationList);

      // Load initial counts
      await this.refreshCounts();
    } catch (error) {
      console.error('Failed to initialize VIP3 browser:', error);
    }
  },

  /**
   * Update a filter value
   * @param key Filter key
   * @param value Filter value (use undefined to remove filter)
   */
  setFilter<K extends keyof Vip3Filters>(key: K, value: Vip3Filters[K]) {
    vip3Filters.update((filters) => {
      const updated = { ...filters };

      if (value === undefined || value === null) {
        delete updated[key];
      } else {
        updated[key] = value;
      }

      return updated;
    });

    // Reset to first page when filters change
    currentPage.set(0);

    // Refresh counts and results
    this.refreshCounts();
    this.search();
  },

  /**
   * Toggle a folder in folder filter
   */
  toggleFolder(folderId: number) {
    vip3Filters.update((filters) => {
      const current = filters.folder_ids ?? [];
      const updated = current.includes(folderId)
        ? current.filter((id) => id !== folderId)
        : [...current, folderId];

      return {
        ...filters,
        folder_ids: updated.length > 0 ? updated : undefined
      };
    });

    currentPage.set(0);
    this.refreshCounts();
    this.search();
  },

  /**
   * Toggle an instrument in instrument filter
   */
  toggleInstrument(instrumentId: number) {
    vip3Filters.update((filters) => {
      const current = filters.instrument_ids ?? [];
      const updated = current.includes(instrumentId)
        ? current.filter((id) => id !== instrumentId)
        : [...current, instrumentId];

      return {
        ...filters,
        instrument_ids: updated.length > 0 ? updated : undefined
      };
    });

    currentPage.set(0);
    this.refreshCounts();
    this.search();
  },

  /**
   * Clear all filters
   */
  clearFilters() {
    vip3Filters.set({});
    currentPage.set(0);
    this.refreshCounts();
    this.search();
  },

  /**
   * Refresh filter counts based on current filter state
   */
  async refreshCounts() {
    loadingCounts.set(true);

    try {
      const filters = get(vip3Filters);
      const counts = await Vip3BrowserApi.getFilterCounts(filters);
      filterCounts.set(counts);
    } catch (error) {
      console.error('Failed to refresh filter counts:', error);
      filterCounts.set(null);
    } finally {
      loadingCounts.set(false);
    }
  },

  /**
   * Search files with current filters
   */
  async search() {
    loadingResults.set(true);

    try {
      const filters = get(vip3Filters);
      const page = get(currentPage);
      const size = get(pageSize);

      const results = await Vip3BrowserApi.searchFiles(filters, page, size);
      searchResults.set(results);
    } catch (error) {
      console.error('Failed to search files:', error);
      searchResults.set(null);
    } finally {
      loadingResults.set(false);
    }
  },

  /**
   * Go to next page
   */
  async nextPage() {
    const results = get(searchResults);
    if (results && results.has_more) {
      currentPage.update((p) => p + 1);
      await this.search();
    }
  },

  /**
   * Go to previous page
   */
  async previousPage() {
    const page = get(currentPage);
    if (page > 0) {
      currentPage.update((p) => p - 1);
      await this.search();
    }
  },

  /**
   * Load a file into the DAW
   */
  async loadToDaw(fileId: number) {
    try {
      const trackId = await Vip3BrowserApi.loadFileToDaw(fileId);
      console.log(`File ${fileId} loaded as track ${trackId}`);
      return trackId;
    } catch (error) {
      console.error('Failed to load file to DAW:', error);
      throw error;
    }
  }
};
```

---

## Verification (10 min)

### 1. Type Check

```bash
cd app
npm run check
```

**Expected:** No TypeScript errors

### 2. Test the Store

Create a test component `app/src/routes/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    vip3Filters,
    filterCounts,
    loadingCounts,
    totalMatches,
    vip3Actions
  } from '$lib/stores/vip3Store';

  onMount(async () => {
    await vip3Actions.initialize();
  });

  function toggleTestFolder() {
    vip3Actions.toggleFolder(1);
  }
</script>

<div class="p-4">
  <h1 class="text-2xl mb-4">VIP3 Filter Test</h1>

  <div class="mb-4">
    <button
      on:click={toggleTestFolder}
      class="px-4 py-2 bg-blue-500 text-white rounded"
    >
      Toggle Folder 1
    </button>
  </div>

  <div class="mb-4">
    <h2 class="text-xl">Current Filters:</h2>
    <pre>{JSON.stringify($vip3Filters, null, 2)}</pre>
  </div>

  {#if $loadingCounts}
    <p>Loading counts...</p>
  {:else if $filterCounts}
    <div class="mb-4">
      <h2 class="text-xl">Total Matches: {$totalMatches}</h2>
      <h3 class="text-lg mt-2">Folder Counts:</h3>
      <pre>{JSON.stringify($filterCounts.folder_counts, null, 2)}</pre>
      <h3 class="text-lg mt-2">BPM Counts:</h3>
      <pre>{JSON.stringify($filterCounts.bpm_counts, null, 2)}</pre>
    </div>
  {/if}
</div>
```

Run the dev server:

```bash
make dev
```

Open http://localhost:5173 and:
1. Click "Toggle Folder 1" - should see filter counts update
2. Check browser console for timing logs
3. Verify counts change when filter is toggled

**Expected Console Output:**
```
Filter counts loaded in 42ms
VIP3 browser initialized
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| TypeScript errors on invoke | Ensure `@tauri-apps/api` is installed: `npm install @tauri-apps/api` |
| "Command not found" | Verify Part 1B command registration completed |
| Store updates don't trigger | Check you're using `$` prefix in Svelte: `$vip3Filters` |
| Counts null after initialize | Check database has data, verify Part 1B command works |
| Performance >50ms | Continue to Part 2 (Database Optimization) for indexes |

---

## What's Next?

✅ **You've completed:**
- TypeScript types matching Rust structs
- VIP3BrowserApi with error handling and timing
- Svelte store with reactive filter management
- Actions for filter manipulation and search

**Next:** [Part 1D: UI Components](./DAY1_PART_D_FILTER_COUNTS_UI.md)
- Update VIP3Column component to display counts
- Add visual indicators for active filters
- Disable filters with 0 results
- Real-time count updates on filter changes

**Testing Note:** The store is now ready to use. Part 1D will integrate it into the VIP3Browser UI.
