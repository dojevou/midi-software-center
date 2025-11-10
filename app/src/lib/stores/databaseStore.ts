import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { FileDetails, SearchFilters, SearchResponse } from '$lib/types';

// ============================================================================
// DATABASE STATE
// ============================================================================

export interface DatabaseState {
  searchResults: FileDetails[];
  totalCount: number;
  currentPage: number;
  pageSize: number;
  filters: SearchFilters;
  selectedFile: FileDetails | null;
  favorites: FileDetails[];
  isLoading: boolean;
  searchQuery: string;
}

const initialState: DatabaseState = {
  searchResults: [],
  totalCount: 0,
  currentPage: 0,
  pageSize: 50,
  filters: {},
  selectedFile: null,
  favorites: [],
  isLoading: false,
  searchQuery: '',
};

export const databaseStore = writable<DatabaseState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const totalPages = derived(
  databaseStore,
  ($database) => Math.ceil($database.totalCount / $database.pageSize)
);

export const hasResults = derived(
  databaseStore,
  ($database) => $database.searchResults.length > 0
);

export const hasDatabaseSelection = derived(
  databaseStore,
  ($database) => $database.selectedFile !== null
);

// ============================================================================
// ACTIONS
// ============================================================================

export const databaseActions = {
  async search(filters?: SearchFilters, page?: number) {
    databaseStore.update(state => ({ ...state, isLoading: true }));

    try {
      const currentState = get(databaseStore);
      const searchFilters: SearchFilters = {
        ...currentState.filters,
        ...filters,
        limit: currentState.pageSize,
        offset: (page ?? currentState.currentPage) * currentState.pageSize,
      };

      const response: SearchResponse = await api.search.files(searchFilters);

      databaseStore.update(state => ({
        ...state,
        searchResults: response.files,
        totalCount: response.total,
        currentPage: page ?? state.currentPage,
        filters: searchFilters,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Search failed:', error);
      databaseStore.update(state => ({ ...state, isLoading: false }));
      throw error;
    }
  },

  async loadFile(fileId: number) {
    databaseStore.update(state => ({ ...state, isLoading: true }));

    try {
      const file = await api.search.getDetails(fileId);
      databaseStore.update(state => ({
        ...state,
        selectedFile: file,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load file:', error);
      databaseStore.update(state => ({ ...state, isLoading: false }));
      throw error;
    }
  },

  async addToFavorites(fileId: number) {
    try {
      await api.analysis.addFavorite(fileId);
      await databaseActions.loadFavorites();
    } catch (error) {
      console.error('Failed to add to favorites:', error);
      throw error;
    }
  },

  async removeFromFavorites(fileId: number) {
    try {
      await api.analysis.removeFavorite(fileId);
      await databaseActions.loadFavorites();
    } catch (error) {
      console.error('Failed to remove from favorites:', error);
      throw error;
    }
  },

  async loadFavorites() {
    try {
      const favorites = await api.analysis.getFavorites();
      databaseStore.update(state => ({ ...state, favorites }));
    } catch (error) {
      console.error('Failed to load favorites:', error);
      throw error;
    }
  },

  setFilters(filters: Partial<SearchFilters>) {
    databaseStore.update(state => ({
      ...state,
      filters: { ...state.filters, ...filters },
    }));
  },

  clearSearch() {
    databaseStore.update(state => ({
      ...state,
      searchResults: [],
      totalCount: 0,
      currentPage: 0,
      searchQuery: '',
    }));
  },

  resetFilters() {
    databaseStore.update(state => ({
      ...state,
      filters: {},
      searchQuery: '',
    }));
  },

  nextPage() {
    const state = get(databaseStore);
    if (state.currentPage < Math.ceil(state.totalCount / state.pageSize) - 1) {
      databaseActions.search(state.filters, state.currentPage + 1);
    }
  },

  previousPage() {
    const state = get(databaseStore);
    if (state.currentPage > 0) {
      databaseActions.search(state.filters, state.currentPage - 1);
    }
  },

  setSearchQuery(query: string) {
    databaseStore.update(state => ({ ...state, searchQuery: query }));
  },
};