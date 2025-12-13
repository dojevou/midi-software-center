/**
 * Saved Searches API
 *
 * CRUD operations for saved search configurations
 * Backend: pipeline/src-tauri/src/commands/saved_searches.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type { SavedSearch, RecentSearch, VIP3BrowserFilters, VIP3BrowserResults } from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const SAVED_SEARCH_COMMANDS = {
  GET_ALL: 'get_all_saved_searches',
  GET_BY_ID: 'get_saved_search_by_id',
  GET_FAVORITES: 'get_favorite_saved_searches',
  CREATE: 'create_saved_search',
  UPDATE: 'update_saved_search',
  DELETE: 'delete_saved_search',
  EXECUTE: 'execute_saved_search',
  TOGGLE_FAVORITE: 'toggle_saved_search_favorite',
  DUPLICATE: 'duplicate_saved_search',
  GET_RECENT: 'get_recent_searches',
  CLEAR_RECENT: 'clear_recent_searches',
  SAVE_RECENT_AS: 'save_recent_search_as',
} as const;

/**
 * Saved search creation parameters
 */
export interface CreateSavedSearchParams {
  name: string;
  description?: string;
  filters: VIP3BrowserFilters;
  is_favorite?: boolean;
}

/**
 * Saved search update parameters
 */
export interface UpdateSavedSearchParams {
  id: number;
  name?: string;
  description?: string;
  filters?: VIP3BrowserFilters;
  is_favorite?: boolean;
}

/**
 * Saved Searches API module
 */
export const savedSearchesApi = {
  /**
   * Get all saved searches
   */
  getAll: async (): Promise<SavedSearch[]> => {
    try {
      return await invoke<SavedSearch[]>(SAVED_SEARCH_COMMANDS.GET_ALL);
    } catch (error) {
      console.error('[savedSearchesApi] Failed to get all saved searches:', error);
      throw error;
    }
  },

  /**
   * Get saved search by ID
   */
  getById: async (id: number): Promise<SavedSearch | null> => {
    try {
      return await invoke<SavedSearch | null>(SAVED_SEARCH_COMMANDS.GET_BY_ID, { id });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to get saved search ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get favorite saved searches
   */
  getFavorites: async (): Promise<SavedSearch[]> => {
    try {
      return await invoke<SavedSearch[]>(SAVED_SEARCH_COMMANDS.GET_FAVORITES);
    } catch (error) {
      console.error('[savedSearchesApi] Failed to get favorite saved searches:', error);
      throw error;
    }
  },

  /**
   * Create a new saved search
   */
  create: async (params: CreateSavedSearchParams): Promise<SavedSearch> => {
    try {
      return await invoke<SavedSearch>(SAVED_SEARCH_COMMANDS.CREATE, { params });
    } catch (error) {
      console.error('[savedSearchesApi] Failed to create saved search:', error);
      throw error;
    }
  },

  /**
   * Update an existing saved search
   */
  update: async (params: UpdateSavedSearchParams): Promise<SavedSearch> => {
    try {
      return await invoke<SavedSearch>(SAVED_SEARCH_COMMANDS.UPDATE, { params });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to update saved search ${params.id}:`, error);
      throw error;
    }
  },

  /**
   * Delete a saved search
   */
  delete: async (id: number): Promise<void> => {
    try {
      await invoke(SAVED_SEARCH_COMMANDS.DELETE, { id });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to delete saved search ${id}:`, error);
      throw error;
    }
  },

  /**
   * Execute a saved search and return results
   */
  execute: async (id: number): Promise<VIP3BrowserResults> => {
    try {
      return await invoke<VIP3BrowserResults>(SAVED_SEARCH_COMMANDS.EXECUTE, { id });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to execute saved search ${id}:`, error);
      throw error;
    }
  },

  /**
   * Toggle favorite status of a saved search
   */
  toggleFavorite: async (id: number): Promise<SavedSearch> => {
    try {
      return await invoke<SavedSearch>(SAVED_SEARCH_COMMANDS.TOGGLE_FAVORITE, { id });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to toggle favorite for saved search ${id}:`, error);
      throw error;
    }
  },

  /**
   * Duplicate a saved search
   */
  duplicate: async (id: number, newName?: string): Promise<SavedSearch> => {
    try {
      return await invoke<SavedSearch>(SAVED_SEARCH_COMMANDS.DUPLICATE, {
        id,
        new_name: newName,
      });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to duplicate saved search ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get recent searches (auto-saved)
   */
  getRecent: async (limit?: number): Promise<RecentSearch[]> => {
    try {
      return await invoke<RecentSearch[]>(SAVED_SEARCH_COMMANDS.GET_RECENT, { limit });
    } catch (error) {
      console.error('[savedSearchesApi] Failed to get recent searches:', error);
      throw error;
    }
  },

  /**
   * Clear recent search history
   */
  clearRecent: async (): Promise<void> => {
    try {
      await invoke(SAVED_SEARCH_COMMANDS.CLEAR_RECENT);
    } catch (error) {
      console.error('[savedSearchesApi] Failed to clear recent searches:', error);
      throw error;
    }
  },

  /**
   * Save a recent search as a named saved search
   */
  saveRecentAs: async (recentId: number, name: string, description?: string): Promise<SavedSearch> => {
    try {
      return await invoke<SavedSearch>(SAVED_SEARCH_COMMANDS.SAVE_RECENT_AS, {
        recent_id: recentId,
        name,
        description,
      });
    } catch (error) {
      console.error(`[savedSearchesApi] Failed to save recent search ${recentId}:`, error);
      throw error;
    }
  },
};

export default savedSearchesApi;
