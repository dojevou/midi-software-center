/**
 * Category Store
 *
 * Manages cached category data (timbres, styles, articulations, BPM ranges, keys)
 * This is a lightweight cache store that can be shared across components
 */

import { derived, writable, get } from 'svelte/store';
import { timbresApi } from '../api/timbres';
import { stylesApi, type StyleWithChildren } from '../api/styles';
import { articulationsApi } from '../api/articulations';
import { vip3BrowserApi } from '../api/vip3Browser';
import type {
  Timbre,
  Style,
  Articulation,
  BpmRange,
  MusicalKey,
  CategoryCounts,
} from '../types';

console.log('🔧 categoryStore.ts: Store module loading');

/**
 * Category store state
 */
export interface CategoryState {
  // Category data
  timbres: Timbre[];
  styles: Style[];
  styleHierarchy: StyleWithChildren[];
  articulations: Articulation[];
  bpmRanges: BpmRange[];
  musicalKeys: MusicalKey[];

  // Counts (file count per category)
  counts: CategoryCounts | null;

  // Loading states
  isLoading: boolean;
  isLoadingCounts: boolean;

  // Error state
  error: string | null;

  // Cache metadata
  lastLoaded: Date | null;
  lastCountsLoaded: Date | null;
}

// Initial state
const initialState: CategoryState = {
  timbres: [],
  styles: [],
  styleHierarchy: [],
  articulations: [],
  bpmRanges: [],
  musicalKeys: [],
  counts: null,
  isLoading: false,
  isLoadingCounts: false,
  error: null,
  lastLoaded: null,
  lastCountsLoaded: null,
};

// Cache TTL in milliseconds (5 minutes)
const CACHE_TTL = 5 * 60 * 1000;

// Create writable store
const { subscribe, set, update } = writable<CategoryState>(initialState);

console.log('🔧 categoryStore.ts: Store created with initial state');

// Read-only store export
export const categoryStore = { subscribe };

// Derived stores for individual categories
export const timbres = derived(categoryStore, ($store) => $store.timbres);
export const styles = derived(categoryStore, ($store) => $store.styles);
export const styleHierarchy = derived(categoryStore, ($store) => $store.styleHierarchy);
export const articulations = derived(categoryStore, ($store) => $store.articulations);
export const bpmRanges = derived(categoryStore, ($store) => $store.bpmRanges);
export const musicalKeys = derived(categoryStore, ($store) => $store.musicalKeys);
export const categoryCounts = derived(categoryStore, ($store) => $store.counts);
export const categoryIsLoading = derived(categoryStore, ($store) => $store.isLoading);
export const categoryError = derived(categoryStore, ($store) => $store.error);

// Derived stores for sorted/filtered categories
export const timbresSorted = derived(categoryStore, ($store) =>
  [...$store.timbres].sort((a, b) => a.sort_order - b.sort_order)
);

export const stylesSorted = derived(categoryStore, ($store) =>
  [...$store.styles].sort((a, b) => a.sort_order - b.sort_order)
);

export const articulationsSorted = derived(categoryStore, ($store) =>
  [...$store.articulations].sort((a, b) => a.sort_order - b.sort_order)
);

export const majorKeys = derived(categoryStore, ($store) =>
  $store.musicalKeys.filter((k) => k.mode === 'major')
);

export const minorKeys = derived(categoryStore, ($store) =>
  $store.musicalKeys.filter((k) => k.mode === 'minor')
);

// Actions
export const categoryActions = {
  /**
   * Load all categories (with caching)
   */
  loadAll: async (force = false): Promise<void> => {
    const currentState = get(categoryStore);

    // Check if cache is still valid
    if (
      !force &&
      currentState.lastLoaded &&
      Date.now() - currentState.lastLoaded.getTime() < CACHE_TTL &&
      currentState.timbres.length > 0
    ) {
      console.log('🔧 categoryActions.loadAll: Using cached data');
      return;
    }

    console.log('🔧 categoryActions.loadAll: Loading all categories');

    update((state) => ({
      ...state,
      isLoading: true,
      error: null,
    }));

    try {
      // Load all categories in parallel
      const [categories, styleHierarchy] = await Promise.all([
        vip3BrowserApi.getAllCategories(),
        stylesApi.getHierarchy().catch(() => [] as StyleWithChildren[]),
      ]);

      update((state) => ({
        ...state,
        timbres: categories.timbres,
        styles: categories.styles,
        styleHierarchy,
        articulations: categories.articulations,
        bpmRanges: categories.bpmRanges,
        musicalKeys: categories.musicalKeys,
        isLoading: false,
        lastLoaded: new Date(),
      }));

      console.log('🔧 categoryActions.loadAll: Categories loaded successfully');
    } catch (error) {
      console.error('❌ categoryActions.loadAll: Failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  /**
   * Load category counts
   */
  loadCounts: async (force = false): Promise<void> => {
    const currentState = get(categoryStore);

    // Check if cache is still valid
    if (
      !force &&
      currentState.lastCountsLoaded &&
      Date.now() - currentState.lastCountsLoaded.getTime() < CACHE_TTL &&
      currentState.counts
    ) {
      console.log('🔧 categoryActions.loadCounts: Using cached counts');
      return;
    }

    console.log('🔧 categoryActions.loadCounts: Loading category counts');

    update((state) => ({
      ...state,
      isLoadingCounts: true,
    }));

    try {
      const counts = await vip3BrowserApi.getCategoryCounts();

      update((state) => ({
        ...state,
        counts,
        isLoadingCounts: false,
        lastCountsLoaded: new Date(),
      }));

      console.log('🔧 categoryActions.loadCounts: Counts loaded successfully');
    } catch (error) {
      console.error('❌ categoryActions.loadCounts: Failed:', error);
      update((state) => ({
        ...state,
        isLoadingCounts: false,
      }));
    }
  },

  /**
   * Load filtered counts based on current filters
   */
  loadFilteredCounts: async (filters: import('../types').VIP3BrowserFilters): Promise<void> => {
    console.log('🔧 categoryActions.loadFilteredCounts: Loading filtered counts');

    update((state) => ({
      ...state,
      isLoadingCounts: true,
    }));

    try {
      const counts = await vip3BrowserApi.getCategoryCounts(filters);

      update((state) => ({
        ...state,
        counts,
        isLoadingCounts: false,
      }));
    } catch (error) {
      console.error('❌ categoryActions.loadFilteredCounts: Failed:', error);
      update((state) => ({
        ...state,
        isLoadingCounts: false,
      }));
    }
  },

  /**
   * Refresh all data (force reload)
   */
  refresh: async (): Promise<void> => {
    await Promise.all([
      categoryActions.loadAll(true),
      categoryActions.loadCounts(true),
    ]);
  },

  /**
   * Get timbre by ID
   */
  getTimbreById: (id: number): Timbre | undefined => {
    const state = get(categoryStore);
    return state.timbres.find((t) => t.id === id);
  },

  /**
   * Get style by ID
   */
  getStyleById: (id: number): Style | undefined => {
    const state = get(categoryStore);
    return state.styles.find((s) => s.id === id);
  },

  /**
   * Get articulation by ID
   */
  getArticulationById: (id: number): Articulation | undefined => {
    const state = get(categoryStore);
    return state.articulations.find((a) => a.id === id);
  },

  /**
   * Get BPM range by ID
   */
  getBpmRangeById: (id: number): BpmRange | undefined => {
    const state = get(categoryStore);
    return state.bpmRanges.find((b) => b.id === id);
  },

  /**
   * Get musical key by ID
   */
  getMusicalKeyById: (id: number): MusicalKey | undefined => {
    const state = get(categoryStore);
    return state.musicalKeys.find((k) => k.id === id);
  },

  /**
   * Get timbre count
   */
  getTimbreCount: (id: number): number => {
    const state = get(categoryStore);
    return state.counts?.timbres[id] || 0;
  },

  /**
   * Get style count
   */
  getStyleCount: (id: number): number => {
    const state = get(categoryStore);
    return state.counts?.styles[id] || 0;
  },

  /**
   * Get articulation count
   */
  getArticulationCount: (id: number): number => {
    const state = get(categoryStore);
    return state.counts?.articulations[id] || 0;
  },

  /**
   * Search timbres by name
   */
  searchTimbres: (query: string): Timbre[] => {
    const state = get(categoryStore);
    const lower = query.toLowerCase();
    return state.timbres.filter(
      (t) => t.name.toLowerCase().includes(lower) || t.description?.toLowerCase().includes(lower)
    );
  },

  /**
   * Search styles by name
   */
  searchStyles: (query: string): Style[] => {
    const state = get(categoryStore);
    const lower = query.toLowerCase();
    return state.styles.filter(
      (s) => s.name.toLowerCase().includes(lower) || s.description?.toLowerCase().includes(lower)
    );
  },

  /**
   * Search articulations by name
   */
  searchArticulations: (query: string): Articulation[] => {
    const state = get(categoryStore);
    const lower = query.toLowerCase();
    return state.articulations.filter(
      (a) =>
        a.name.toLowerCase().includes(lower) ||
        a.description?.toLowerCase().includes(lower) ||
        a.abbreviation?.toLowerCase().includes(lower)
    );
  },

  /**
   * Clear error
   */
  clearError: (): void => {
    update((state) => ({
      ...state,
      error: null,
    }));
  },

  /**
   * Invalidate cache
   */
  invalidateCache: (): void => {
    update((state) => ({
      ...state,
      lastLoaded: null,
      lastCountsLoaded: null,
    }));
  },

  /**
   * Reset store to initial state
   */
  reset: (): void => {
    set(initialState);
  },
};

console.log('🔧 categoryStore.ts: Store module loaded');
