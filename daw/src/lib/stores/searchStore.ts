// searchStore.ts - Manager for search-related I/O operations
// Archetype: Grown-up Script (manages state AND does I/O)

import { writable, derived, type Writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types
export interface SearchSuggestion {
  text: string;
  type: 'category' | 'instrument' | 'key' | 'bpm';
  count: number;
}

export interface SearchState {
  query: string;
  suggestions: SearchSuggestion[];
  loading: boolean;
  error: string | null;
}

// Create the store
function createSearchStore() {
  const { subscribe, set, update }: Writable<SearchState> = writable<SearchState>({
    query: '',
    suggestions: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    // Set query without loading suggestions
    setQuery: (query: string) => {
      update(state => ({ ...state, query, error: null }));
    },

    // Load suggestions from backend (I/O operation)
    loadSuggestions: async (searchText: string) => {
      if (searchText.length < 2) {
        update(state => ({ ...state, suggestions: [] }));
        return;
      }

      update(state => ({ ...state, loading: true, error: null }));

      try {
        const suggestions = await invoke<SearchSuggestion[]>('get_search_suggestions', {
          query: searchText
        });

        update(state => ({
          ...state,
          suggestions,
          loading: false
        }));
      } catch (error) {
        console.error('Failed to load suggestions:', error);
        update(state => ({
          ...state,
          suggestions: [],
          loading: false,
          error: `Failed to load suggestions: ${error}`
        }));
      }
    },

    // Clear suggestions
    clearSuggestions: () => {
      update(state => ({ ...state, suggestions: [] }));
    },

    // Reset store
    reset: () => {
      set({
        query: '',
        suggestions: [],
        loading: false,
        error: null
      });
    }
  };
}

// Export singleton instance
export const searchStore = createSearchStore();

// Derived stores for convenience
export const searchQuery = derived(searchStore, $store => $store.query);
export const searchSuggestions = derived(searchStore, $store => $store.suggestions);
export const searchLoading = derived(searchStore, $store => $store.loading);
