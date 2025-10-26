// filterStore.ts - Manager for filter options I/O operations
// Archetype: Grown-up Script (manages state AND does I/O)

import { writable, derived, type Writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types
export interface FilterOption {
  value: string;
  label: string;
  count: number;
}

export interface SearchFilters {
  categories: string[];
  bpmRange: [number, number];
  keySignature: string[];
  timeSignature: string[];
  instruments: string[];
  duration: [number, number];
}

export interface FilterState {
  // Available options (loaded from backend)
  availableCategories: FilterOption[];
  availableKeys: FilterOption[];
  availableTimeSignatures: FilterOption[];
  availableInstruments: FilterOption[];

  // Current filter selections
  filters: SearchFilters;

  // Loading state
  loading: boolean;
  error: string | null;
}

// Default filters
const defaultFilters: SearchFilters = {
  categories: [],
  bpmRange: [0, 300],
  keySignature: [],
  timeSignature: [],
  instruments: [],
  duration: [0, 600]
};

// Create the store
function createFilterStore() {
  const { subscribe, set, update }: Writable<FilterState> = writable<FilterState>({
    availableCategories: [],
    availableKeys: [],
    availableTimeSignatures: [],
    availableInstruments: [],
    filters: { ...defaultFilters },
    loading: false,
    error: null
  });

  return {
    subscribe,

    // Load all filter options from backend (I/O operation)
    loadFilterOptions: async () => {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const [categories, keys, timeSignatures, instruments] = await Promise.all([
          invoke<FilterOption[]>('get_categories'),
          invoke<FilterOption[]>('get_key_signatures'),
          invoke<FilterOption[]>('get_time_signatures'),
          invoke<FilterOption[]>('get_instruments')
        ]);

        update(state => ({
          ...state,
          availableCategories: categories,
          availableKeys: keys,
          availableTimeSignatures: timeSignatures,
          availableInstruments: instruments,
          loading: false
        }));
      } catch (error) {
        console.error('Failed to load filter options:', error);
        update(state => ({
          ...state,
          loading: false,
          error: `Failed to load filter options: ${error}`
        }));
      }
    },

    // Update filter values
    setFilters: (filters: SearchFilters) => {
      update(state => ({ ...state, filters }));
    },

    // Toggle a filter value in an array
    toggleFilter: (filterKey: keyof Pick<SearchFilters, 'categories' | 'keySignature' | 'timeSignature' | 'instruments'>, value: string) => {
      update(state => {
        const currentArray = state.filters[filterKey];
        const index = currentArray.indexOf(value);

        const newArray = index >= 0
          ? currentArray.filter(v => v !== value)
          : [...currentArray, value];

        return {
          ...state,
          filters: {
            ...state.filters,
            [filterKey]: newArray
          }
        };
      });
    },

    // Update range filter
    setRangeFilter: (filterKey: 'bpmRange' | 'duration', range: [number, number]) => {
      update(state => ({
        ...state,
        filters: {
          ...state.filters,
          [filterKey]: range
        }
      }));
    },

    // Clear all filters
    clearAllFilters: () => {
      update(state => ({
        ...state,
        filters: { ...defaultFilters }
      }));
    },

    // Reset store
    reset: () => {
      set({
        availableCategories: [],
        availableKeys: [],
        availableTimeSignatures: [],
        availableInstruments: [],
        filters: { ...defaultFilters },
        loading: false,
        error: null
      });
    }
  };
}

// Export singleton instance
export const filterStore = createFilterStore();

// Derived stores for convenience
export const filters = derived(filterStore, $store => $store.filters);
export const availableCategories = derived(filterStore, $store => $store.availableCategories);
export const availableKeys = derived(filterStore, $store => $store.availableKeys);
export const availableTimeSignatures = derived(filterStore, $store => $store.availableTimeSignatures);
export const availableInstruments = derived(filterStore, $store => $store.availableInstruments);
export const filterLoading = derived(filterStore, $store => $store.loading);
