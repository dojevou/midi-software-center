/**
 * Database Store - Search and file management
 *
 * Grown-up Script: Manages database search, filtering, and file selection.
 * Handles MIDI file search with multiple filter criteria.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export interface SearchResult {
  id: string;
  filename: string;
  filepath: string;
  duration: number;
  bpm: number | null;
  key: string | null;
  timeSignature: string | null;
  trackCount: number;
  noteCount: number;
  category: string | null;
  tags: string[];
  favorite: boolean;
  importedAt: string;
  fileSize: number;
}

export interface SearchFilters {
  bpmMin: number | null;
  bpmMax: number | null;
  keys: string[];
  categories: string[];
  tags: string[];
  durationMin: number | null; // seconds
  durationMax: number | null; // seconds
  trackCountMin: number | null;
  trackCountMax: number | null;
  favoriteOnly: boolean;
}

export interface DatabaseState {
  searchQuery: string;
  filters: SearchFilters;
  searchResults: SearchResult[];
  selectedFile: SearchResult | null;
  loading: boolean;
  error: string | null;
  totalResults: number;
  currentPage: number;
  pageSize: number;
  sortBy: 'filename' | 'bpm' | 'duration' | 'imported_at';
  sortOrder: 'asc' | 'desc';
}

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_FILTERS: SearchFilters = {
  bpmMin: null,
  bpmMax: null,
  keys: [],
  categories: [],
  tags: [],
  durationMin: null,
  durationMax: null,
  trackCountMin: null,
  trackCountMax: null,
  favoriteOnly: false
};

const DEFAULT_STATE: DatabaseState = {
  searchQuery: '',
  filters: DEFAULT_FILTERS,
  searchResults: [],
  selectedFile: null,
  loading: false,
  error: null,
  totalResults: 0,
  currentPage: 1,
  pageSize: 50,
  sortBy: 'imported_at',
  sortOrder: 'desc'
};

// ============================================================================
// Store
// ============================================================================

const { subscribe, set, update } = writable<DatabaseState>(DEFAULT_STATE);

// ============================================================================
// Actions
// ============================================================================

export const databaseActions = {
  /**
   * Search for MIDI files
   */
  async search(query: string = ''): Promise<void> {
    const state = get(databaseStore);

    update(s => ({ ...s, loading: true, error: null, searchQuery: query }));

    try {
      const response = await invoke<{
        results: SearchResult[];
        total: number;
      }>('search_files', {
        query,
        filters: state.filters,
        page: state.currentPage,
        pageSize: state.pageSize,
        sortBy: state.sortBy,
        sortOrder: state.sortOrder
      });

      update(s => ({
        ...s,
        searchResults: response.results,
        totalResults: response.total,
        loading: false
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Search failed';
      console.error('Search failed:', error);
      update(s => ({
        ...s,
        loading: false,
        error: errorMessage,
        searchResults: [],
        totalResults: 0
      }));
    }
  },

  /**
   * Set search filters
   */
  async setFilters(filters: Partial<SearchFilters>): Promise<void> {
    update(s => ({
      ...s,
      filters: { ...s.filters, ...filters },
      currentPage: 1 // Reset to first page when filters change
    }));

    const state = get(databaseStore);
    await this.search(state.searchQuery);
  },

  /**
   * Clear all filters
   */
  async clearFilters(): Promise<void> {
    update(s => ({
      ...s,
      filters: DEFAULT_FILTERS,
      currentPage: 1
    }));

    const state = get(databaseStore);
    await this.search(state.searchQuery);
  },

  /**
   * Select a file
   */
  async selectFile(fileId: string | null): Promise<void> {
    const state = get(databaseStore);

    if (fileId === null) {
      update(s => ({ ...s, selectedFile: null }));
      return;
    }

    const file = state.searchResults.find(f => f.id === fileId);

    if (file) {
      update(s => ({ ...s, selectedFile: file }));
    } else {
      // File not in current results, fetch from database
      try {
        const fetchedFile = await invoke<SearchResult>('get_file_by_id', { fileId });
        update(s => ({ ...s, selectedFile: fetchedFile }));
      } catch (error) {
        console.error('Failed to fetch file:', error);
        update(s => ({ ...s, error: 'Failed to load file' }));
      }
    }
  },

  /**
   * Toggle favorite status
   */
  async toggleFavorite(fileId: string): Promise<void> {
    const state = get(databaseStore);
    const file = state.searchResults.find(f => f.id === fileId);

    if (!file) {
      console.error('File not found:', fileId);
      return;
    }

    const newFavoriteState = !file.favorite;

    try {
      await invoke('set_favorite', { fileId, favorite: newFavoriteState });

      update(s => ({
        ...s,
        searchResults: s.searchResults.map(f =>
          f.id === fileId ? { ...f, favorite: newFavoriteState } : f
        ),
        selectedFile: s.selectedFile?.id === fileId
          ? { ...s.selectedFile, favorite: newFavoriteState }
          : s.selectedFile
      }));
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
      update(s => ({ ...s, error: 'Failed to update favorite status' }));
    }
  },

  /**
   * Import file to database
   */
  async importFile(filepath: string): Promise<void> {
    update(s => ({ ...s, loading: true, error: null }));

    try {
      await invoke('import_single_file', { filepath });
      update(s => ({ ...s, loading: false }));

      // Refresh search results
      const state = get(databaseStore);
      await this.search(state.searchQuery);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Import failed';
      console.error('Import failed:', error);
      update(s => ({ ...s, loading: false, error: errorMessage }));
    }
  },

  /**
   * Import directory to database
   */
  async importDirectory(dirpath: string): Promise<void> {
    update(s => ({ ...s, loading: true, error: null }));

    try {
      await invoke('import_directory', { dirpath });
      update(s => ({ ...s, loading: false }));

      // Refresh search results
      const state = get(databaseStore);
      await this.search(state.searchQuery);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Import failed';
      console.error('Import failed:', error);
      update(s => ({ ...s, loading: false, error: errorMessage }));
    }
  },

  /**
   * Set page number
   */
  async setPage(page: number): Promise<void> {
    update(s => ({ ...s, currentPage: page }));

    const state = get(databaseStore);
    await this.search(state.searchQuery);
  },

  /**
   * Set page size
   */
  async setPageSize(pageSize: number): Promise<void> {
    update(s => ({ ...s, pageSize, currentPage: 1 }));

    const state = get(databaseStore);
    await this.search(state.searchQuery);
  },

  /**
   * Set sort order
   */
  async setSortOrder(
    sortBy: DatabaseState['sortBy'],
    sortOrder: 'asc' | 'desc'
  ): Promise<void> {
    update(s => ({ ...s, sortBy, sortOrder }));

    const state = get(databaseStore);
    await this.search(state.searchQuery);
  },

  /**
   * Load next page
   */
  async nextPage(): Promise<void> {
    const state = get(databaseStore);
    const totalPages = Math.ceil(state.totalResults / state.pageSize);

    if (state.currentPage < totalPages) {
      await this.setPage(state.currentPage + 1);
    }
  },

  /**
   * Load previous page
   */
  async previousPage(): Promise<void> {
    const state = get(databaseStore);

    if (state.currentPage > 1) {
      await this.setPage(state.currentPage - 1);
    }
  },

  /**
   * Get available categories
   */
  async getCategories(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_categories');
    } catch (error) {
      console.error('Failed to fetch categories:', error);
      return [];
    }
  },

  /**
   * Get available tags
   */
  async getTags(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_tags');
    } catch (error) {
      console.error('Failed to fetch tags:', error);
      return [];
    }
  },

  /**
   * Reset to initial state
   */
  reset(): void {
    set(DEFAULT_STATE);
  }
};

// ============================================================================
// Derived Stores
// ============================================================================

export const hasResults = derived(
  databaseStore,
  $db => $db.searchResults.length > 0
);

export const totalPages = derived(
  databaseStore,
  $db => Math.ceil($db.totalResults / $db.pageSize)
);

export const hasNextPage = derived(
  [databaseStore, totalPages],
  ([$db, $totalPages]) => $db.currentPage < $totalPages
);

export const hasPreviousPage = derived(
  databaseStore,
  $db => $db.currentPage > 1
);

export const activeFiltersCount = derived(
  databaseStore,
  $db => {
    let count = 0;
    const f = $db.filters;

    if (f.bpmMin !== null || f.bpmMax !== null) count++;
    if (f.keys.length > 0) count++;
    if (f.categories.length > 0) count++;
    if (f.tags.length > 0) count++;
    if (f.durationMin !== null || f.durationMax !== null) count++;
    if (f.trackCountMin !== null || f.trackCountMax !== null) count++;
    if (f.favoriteOnly) count++;

    return count;
  }
);

// ============================================================================
// Export
// ============================================================================

export const databaseStore = {
  subscribe,
  set,
  update
};
