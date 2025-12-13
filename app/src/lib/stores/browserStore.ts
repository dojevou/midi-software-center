/**
 * Browser Store
 *
 * Manages VIP3-style browser state including filters, results, and UI state
 * Follows the pattern from databaseStore.ts
 */

import { derived, writable, get } from 'svelte/store';
import { vip3BrowserApi } from '../api/vip3Browser';
import type {
  VIP3BrowserState,
  VIP3BrowserFilters,
  VIP3BrowserResults,
  VIP3ColumnConfig,
  FileDetails,
  Timbre,
  Style,
  Articulation,
  BpmRange,
  MusicalKey,
} from '../types';

console.log('🔧 browserStore.ts: Store module loading');

// Default column configuration for VIP3 browser
const defaultColumns: VIP3ColumnConfig[] = [
  { id: 'search', label: 'Search', type: 'search', width: 200, visible: true, sortOrder: 0 },
  { id: 'timbres', label: 'Timbres', type: 'category', width: 150, visible: true, sortOrder: 1 },
  { id: 'styles', label: 'Styles', type: 'category', width: 150, visible: true, sortOrder: 2 },
  { id: 'articulations', label: 'Articulations', type: 'category', width: 150, visible: true, sortOrder: 3 },
  { id: 'bpm', label: 'BPM', type: 'range', width: 120, visible: true, sortOrder: 4 },
  { id: 'key', label: 'Key', type: 'category', width: 120, visible: true, sortOrder: 5 },
  { id: 'tags', label: 'Tags', type: 'tags', width: 150, visible: true, sortOrder: 6 },
  { id: 'collections', label: 'Collections', type: 'collections', width: 150, visible: true, sortOrder: 7 },
];

// Initial state
const initialState: VIP3BrowserState = {
  // Filter state
  filters: {},
  activeFilters: [],

  // Results
  results: null,
  isLoading: false,
  error: null,

  // Selection
  selectedFileIds: [],
  lastSelectedId: null,

  // View options
  viewMode: 'list',
  columns: defaultColumns,
  previewEnabled: true,
  previewFileId: null,

  // Cached category data
  timbres: [],
  styles: [],
  articulations: [],
  bpmRanges: [],
  musicalKeys: [],
};

// Create writable store
const { subscribe, set, update } = writable<VIP3BrowserState>(initialState);

console.log('🔧 browserStore.ts: Store created with initial state');

// Read-only store export
export const browserStore = { subscribe };

// Derived stores for common selections
export const browserResults = derived(browserStore, ($store) => $store.results);
export const browserFilters = derived(browserStore, ($store) => $store.filters);
export const browserIsLoading = derived(browserStore, ($store) => $store.isLoading);
export const browserError = derived(browserStore, ($store) => $store.error);
export const selectedFiles = derived(browserStore, ($store) => $store.selectedFileIds);
export const previewFile = derived(browserStore, ($store) => {
  if (!$store.previewFileId || !$store.results) {return null;}
  return $store.results.files.find((f) => f.id === $store.previewFileId) || null;
});

// Actions
export const browserActions = {
  /**
   * Perform a search with the given filters
   */
  search: async (filters?: VIP3BrowserFilters): Promise<void> => {
    console.log('🔧 browserActions.search called with filters:', filters);

    update((state) => ({
      ...state,
      isLoading: true,
      error: null,
      filters: filters || state.filters,
    }));

    try {
      const searchFilters = filters || get(browserStore).filters;
      const results = await vip3BrowserApi.search(searchFilters);

      update((state) => ({
        ...state,
        results,
        isLoading: false,
        activeFilters: computeActiveFilters(searchFilters),
      }));

      console.log('🔧 browserActions.search: Search completed with', results.total, 'results');
    } catch (error) {
      console.error('❌ browserActions.search: Search failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  /**
   * Update filters and optionally trigger search
   */
  setFilters: async (filters: Partial<VIP3BrowserFilters>, autoSearch = true): Promise<void> => {
    const currentState = get(browserStore);
    const newFilters = { ...currentState.filters, ...filters };

    update((state) => ({
      ...state,
      filters: newFilters,
      activeFilters: computeActiveFilters(newFilters),
    }));

    if (autoSearch) {
      await browserActions.search(newFilters);
    }
  },

  /**
   * Clear all filters and reset to initial state
   */
  clearFilters: async (autoSearch = true): Promise<void> => {
    update((state) => ({
      ...state,
      filters: {},
      activeFilters: [],
    }));

    if (autoSearch) {
      await browserActions.search({});
    }
  },

  /**
   * Toggle a timbre filter
   */
  toggleTimbre: async (timbreId: number): Promise<void> => {
    const currentState = get(browserStore);
    const currentIds = currentState.filters.timbre_ids || [];
    const newIds = currentIds.includes(timbreId)
      ? currentIds.filter((id) => id !== timbreId)
      : [...currentIds, timbreId];

    await browserActions.setFilters({ timbre_ids: newIds.length > 0 ? newIds : undefined });
  },

  /**
   * Toggle a style filter
   */
  toggleStyle: async (styleId: number): Promise<void> => {
    const currentState = get(browserStore);
    const currentIds = currentState.filters.style_ids || [];
    const newIds = currentIds.includes(styleId)
      ? currentIds.filter((id) => id !== styleId)
      : [...currentIds, styleId];

    await browserActions.setFilters({ style_ids: newIds.length > 0 ? newIds : undefined });
  },

  /**
   * Toggle an articulation filter
   */
  toggleArticulation: async (articulationId: number): Promise<void> => {
    const currentState = get(browserStore);
    const currentIds = currentState.filters.articulation_ids || [];
    const newIds = currentIds.includes(articulationId)
      ? currentIds.filter((id) => id !== articulationId)
      : [...currentIds, articulationId];

    await browserActions.setFilters({ articulation_ids: newIds.length > 0 ? newIds : undefined });
  },

  /**
   * Set BPM range filter
   */
  setBpmRange: async (minBpm?: number, maxBpm?: number): Promise<void> => {
    await browserActions.setFilters({ min_bpm: minBpm, max_bpm: maxBpm });
  },

  /**
   * Toggle a key signature filter
   */
  toggleKey: async (keySignature: string): Promise<void> => {
    const currentState = get(browserStore);
    const currentKeys = currentState.filters.key_signatures || [];
    const newKeys = currentKeys.includes(keySignature)
      ? currentKeys.filter((k) => k !== keySignature)
      : [...currentKeys, keySignature];

    await browserActions.setFilters({ key_signatures: newKeys.length > 0 ? newKeys : undefined });
  },

  /**
   * Toggle a tag filter
   */
  toggleTag: async (tag: string, include = true): Promise<void> => {
    const currentState = get(browserStore);

    if (include) {
      const currentTags = currentState.filters.include_tags || [];
      const newTags = currentTags.includes(tag)
        ? currentTags.filter((t) => t !== tag)
        : [...currentTags, tag];
      await browserActions.setFilters({ include_tags: newTags.length > 0 ? newTags : undefined });
    } else {
      const currentTags = currentState.filters.exclude_tags || [];
      const newTags = currentTags.includes(tag)
        ? currentTags.filter((t) => t !== tag)
        : [...currentTags, tag];
      await browserActions.setFilters({ exclude_tags: newTags.length > 0 ? newTags : undefined });
    }
  },

  /**
   * Set text search
   */
  setSearchText: async (text: string): Promise<void> => {
    await browserActions.setFilters({ search_text: text || undefined });
  },

  /**
   * Set sort options
   */
  setSort: async (sortBy: VIP3BrowserFilters['sort_by'], sortDesc = true): Promise<void> => {
    await browserActions.setFilters({ sort_by: sortBy, sort_desc: sortDesc });
  },

  /**
   * Go to next page
   */
  nextPage: async (): Promise<void> => {
    const currentState = get(browserStore);
    if (!currentState.results) {return;}

    const currentPage = currentState.results.page;
    const totalPages = currentState.results.total_pages;

    if (currentPage < totalPages) {
      const pageSize = currentState.filters.limit || 50;
      await browserActions.setFilters({ offset: currentPage * pageSize });
    }
  },

  /**
   * Go to previous page
   */
  previousPage: async (): Promise<void> => {
    const currentState = get(browserStore);
    if (!currentState.results) {return;}

    const currentPage = currentState.results.page;
    const pageSize = currentState.filters.limit || 50;

    if (currentPage > 1) {
      await browserActions.setFilters({ offset: (currentPage - 2) * pageSize });
    }
  },

  /**
   * Go to a specific page
   */
  goToPage: async (page: number): Promise<void> => {
    const currentState = get(browserStore);
    const pageSize = currentState.filters.limit || 50;
    await browserActions.setFilters({ offset: (page - 1) * pageSize });
  },

  /**
   * Select a file
   */
  selectFile: (fileId: number, multiSelect = false, rangeSelect = false): void => {
    update((state) => {
      const currentState = get(browserStore);

      if (rangeSelect && currentState.lastSelectedId && currentState.results) {
        // Range selection
        const files = currentState.results.files;
        const lastIndex = files.findIndex((f) => f.id === currentState.lastSelectedId);
        const currentIndex = files.findIndex((f) => f.id === fileId);

        if (lastIndex !== -1 && currentIndex !== -1) {
          const start = Math.min(lastIndex, currentIndex);
          const end = Math.max(lastIndex, currentIndex);
          const rangeIds = files.slice(start, end + 1).map((f) => f.id);
          const newSelection = multiSelect
            ? [...new Set([...state.selectedFileIds, ...rangeIds])]
            : rangeIds;

          return { ...state, selectedFileIds: newSelection };
        }
      }

      if (multiSelect) {
        // Toggle selection
        const newSelection = state.selectedFileIds.includes(fileId)
          ? state.selectedFileIds.filter((id) => id !== fileId)
          : [...state.selectedFileIds, fileId];
        return { ...state, selectedFileIds: newSelection, lastSelectedId: fileId };
      }

      // Single selection
      return { ...state, selectedFileIds: [fileId], lastSelectedId: fileId };
    });
  },

  /**
   * Clear selection
   */
  clearSelection: (): void => {
    update((state) => ({
      ...state,
      selectedFileIds: [],
      lastSelectedId: null,
    }));
  },

  /**
   * Select all files in current results
   */
  selectAll: (): void => {
    update((state) => {
      if (!state.results) {return state;}
      return {
        ...state,
        selectedFileIds: state.results.files.map((f) => f.id),
      };
    });
  },

  /**
   * Set preview file
   */
  setPreviewFile: (fileId: number | null): void => {
    update((state) => ({
      ...state,
      previewFileId: fileId,
    }));
  },

  /**
   * Toggle preview panel
   */
  togglePreview: (): void => {
    update((state) => ({
      ...state,
      previewEnabled: !state.previewEnabled,
    }));
  },

  /**
   * Set view mode
   */
  setViewMode: (mode: 'list' | 'grid' | 'compact'): void => {
    update((state) => ({
      ...state,
      viewMode: mode,
    }));
  },

  /**
   * Update column configuration
   */
  setColumns: (columns: VIP3ColumnConfig[]): void => {
    update((state) => ({
      ...state,
      columns,
    }));
  },

  /**
   * Toggle column visibility
   */
  toggleColumn: (columnId: string): void => {
    update((state) => ({
      ...state,
      columns: state.columns.map((col) =>
        col.id === columnId ? { ...col, visible: !col.visible } : col
      ),
    }));
  },

  /**
   * Load category data (timbres, styles, etc.)
   */
  loadCategories: async (): Promise<void> => {
    console.log('🔧 browserActions.loadCategories: Loading categories');

    try {
      const categories = await vip3BrowserApi.getAllCategories();

      update((state) => ({
        ...state,
        timbres: categories.timbres,
        styles: categories.styles,
        articulations: categories.articulations,
        bpmRanges: categories.bpmRanges,
        musicalKeys: categories.musicalKeys,
      }));

      console.log('🔧 browserActions.loadCategories: Categories loaded');
    } catch (error) {
      console.error('❌ browserActions.loadCategories: Failed to load categories:', error);
    }
  },

  /**
   * Reset store to initial state
   */
  reset: (): void => {
    set(initialState);
  },
};

/**
 * Compute list of active filter keys for UI indication
 */
function computeActiveFilters(filters: VIP3BrowserFilters): string[] {
  const active: string[] = [];

  if (filters.search_text) {active.push('search');}
  if (filters.timbre_ids?.length) {active.push('timbres');}
  if (filters.style_ids?.length) {active.push('styles');}
  if (filters.articulation_ids?.length) {active.push('articulations');}
  if (filters.min_bpm !== undefined || filters.max_bpm !== undefined) {active.push('bpm');}
  if (filters.key_signatures?.length || filters.key_ids?.length) {active.push('key');}
  if (filters.include_tags?.length || filters.exclude_tags?.length) {active.push('tags');}
  if (filters.collection_ids?.length) {active.push('collections');}
  if (filters.has_drums !== undefined) {active.push('drums');}
  if (filters.is_favorite !== undefined) {active.push('favorites');}

  return active;
}

console.log('🔧 browserStore.ts: Store module loaded');
