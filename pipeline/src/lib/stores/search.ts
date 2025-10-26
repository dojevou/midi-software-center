/**
 * Search state management
 * Archetype: Grown-up Script (store with I/O operations)
 */

import { writable } from 'svelte/store';
import { searchFiles } from '$lib/api/search';
import type { SearchFilters, FileMetadata } from '$lib/api/types';

interface SearchState {
  query: string;
  filters: SearchFilters;
  results: FileMetadata[];
  total: number;
  loading: boolean;
  error: string | null;
  page: number;
  pageSize: number;
}

function createSearchStore() {
  const { subscribe, update } = writable<SearchState>({
    query: '',
    filters: {},
    results: [],
    total: 0,
    loading: false,
    error: null,
    page: 1,
    pageSize: 50,
  });

  return {
    subscribe,
    search: async (query: string, filters: SearchFilters = {}) => {
      update(s => ({ ...s, query, filters, loading: true, error: null, page: 1 }));
      try {
        const result = await searchFiles(query, filters, 1, 50);
        update(s => ({
          ...s,
          results: result.files,
          total: result.totalCount,
          page: result.page,
          loading: false,
        }));
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : String(error),
          loading: false,
          results: [],
          total: 0,
        }));
      }
    },
    clear: () => update(s => ({
      ...s,
      query: '',
      filters: {},
      results: [],
      total: 0,
      error: null,
      page: 1,
    })),
  };
}

export const searchStore = createSearchStore();
