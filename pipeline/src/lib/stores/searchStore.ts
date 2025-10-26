import { writable } from 'svelte/store';
import { searchFiles } from '$lib/api/search';
import type { FileMetadata, SearchFilters } from '$lib/types';

interface SearchState {
  query: string;
  results: FileMetadata[];
  loading: boolean;
  error: string | null;
}

function createSearchStore() {
  const { subscribe, set, update } = writable<SearchState>({
    query: '',
    results: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    // MANAGER handles BOTH state AND I/O
    search: async (query: string, filters: SearchFilters = {}) => {
      update(state => ({ ...state, query, loading: true, error: null }));

      try {
        const searchResults = await searchFiles(query, filters, 1, 100);
        update(state => ({
          ...state,
          results: searchResults.files,
          loading: false
        }));
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Search failed';
        update(state => ({
          ...state,
          results: [],
          loading: false,
          error: errorMessage
        }));
      }
    },

    clearSearch: () => set({
      query: '',
      results: [],
      loading: false,
      error: null
    })
  };
}

export const searchStore = createSearchStore();
