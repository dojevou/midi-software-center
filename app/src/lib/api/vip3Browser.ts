/**
 * VIP3 Browser API
 *
 * Combined filtering and browsing endpoint for VIP3-style database browser
 * Backend: pipeline/src-tauri/src/commands/vip3_browser.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  VIP3BrowserFilters,
  VIP3BrowserResults,
  Timbre,
  Style,
  Articulation,
  BpmRange,
  MusicalKey,
  CategoryCounts,
  FileCategoryAssignment,
  BulkOperationResult,
} from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const VIP3_BROWSER_COMMANDS = {
  SEARCH: 'vip3_browser_search',
  GET_CATEGORIES: 'vip3_get_all_categories',
  GET_CATEGORY_COUNTS: 'vip3_get_category_counts',
  ASSIGN_CATEGORIES: 'vip3_assign_categories',
  REMOVE_CATEGORIES: 'vip3_remove_categories',
  BATCH_ASSIGN: 'vip3_batch_assign_categories',
  GET_SUGGESTIONS: 'vip3_get_search_suggestions',
  GET_FILTER_OPTIONS: 'vip3_get_filter_options',
} as const;

/**
 * All category data for VIP3 browser
 */
export interface VIP3Categories {
  timbres: Timbre[];
  styles: Style[];
  articulations: Articulation[];
  bpmRanges: BpmRange[];
  musicalKeys: MusicalKey[];
}

/**
 * Search suggestions response
 */
export interface SearchSuggestions {
  filenames: string[];
  tags: string[];
  manufacturers: string[];
  collections: string[];
}

/**
 * Filter options with available values
 */
export interface VIP3FilterOptions {
  timeSignatures: string[];
  keySignatures: string[];
  manufacturers: string[];
  sources: string[];
  categories: string[];
}

/**
 * VIP3 Browser API module
 */
export const vip3BrowserApi = {
  /**
   * Perform a VIP3 browser search with filters
   * This is the main search endpoint for the browser
   */
  search: async (filters: VIP3BrowserFilters): Promise<VIP3BrowserResults> => {
    try {
      console.log('[vip3BrowserApi] Searching with filters:', filters);
      return await invoke<VIP3BrowserResults>(VIP3_BROWSER_COMMANDS.SEARCH, { filters });
    } catch (error) {
      console.error('[vip3BrowserApi] Search failed:', error);
      throw error;
    }
  },

  /**
   * Get all category data for filter columns
   * Returns timbres, styles, articulations, BPM ranges, and musical keys
   */
  getAllCategories: async (): Promise<VIP3Categories> => {
    try {
      return await invoke<VIP3Categories>(VIP3_BROWSER_COMMANDS.GET_CATEGORIES);
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to get categories:', error);
      throw error;
    }
  },

  /**
   * Get counts for all categories (for filter badges)
   * Optionally filtered by current search filters
   */
  getCategoryCounts: async (filters?: VIP3BrowserFilters): Promise<CategoryCounts> => {
    try {
      return await invoke<CategoryCounts>(VIP3_BROWSER_COMMANDS.GET_CATEGORY_COUNTS, {
        filters,
      });
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to get category counts:', error);
      throw error;
    }
  },

  /**
   * Assign categories to a file
   */
  assignCategories: async (assignment: FileCategoryAssignment): Promise<void> => {
    try {
      await invoke(VIP3_BROWSER_COMMANDS.ASSIGN_CATEGORIES, { assignment });
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to assign categories:', error);
      throw error;
    }
  },

  /**
   * Remove categories from a file
   */
  removeCategories: async (assignment: FileCategoryAssignment): Promise<void> => {
    try {
      await invoke(VIP3_BROWSER_COMMANDS.REMOVE_CATEGORIES, { assignment });
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to remove categories:', error);
      throw error;
    }
  },

  /**
   * Batch assign categories to multiple files
   */
  batchAssign: async (assignments: FileCategoryAssignment[]): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(VIP3_BROWSER_COMMANDS.BATCH_ASSIGN, {
        assignments,
      });
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to batch assign categories:', error);
      throw error;
    }
  },

  /**
   * Get search suggestions for autocomplete
   */
  getSuggestions: async (query: string, limit?: number): Promise<SearchSuggestions> => {
    try {
      return await invoke<SearchSuggestions>(VIP3_BROWSER_COMMANDS.GET_SUGGESTIONS, {
        query,
        limit: limit || 10,
      });
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to get suggestions:', error);
      throw error;
    }
  },

  /**
   * Get available filter options
   */
  getFilterOptions: async (): Promise<VIP3FilterOptions> => {
    try {
      return await invoke<VIP3FilterOptions>(VIP3_BROWSER_COMMANDS.GET_FILTER_OPTIONS);
    } catch (error) {
      console.error('[vip3BrowserApi] Failed to get filter options:', error);
      throw error;
    }
  },
};

export default vip3BrowserApi;
