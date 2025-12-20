import { invoke } from '@tauri-apps/api/core';
import type {
  FilterCounts,
  Vip3Filters,
  Vip3SearchResponse,
  Timbre,
  Style,
  Articulation,
  SavedSearchResponse,
  CreateSavedSearchRequest,
  CollectionResponse,
  CreateCollectionRequest,
  UpdateCollectionRequest
} from '$lib/types/vip3';

/**
 * VIP3 Browser API
 * Handles all VIP3-related Tauri command invocations
 */
export class Vip3BrowserApi {
  /**
   * Get dynamic filter counts that update based on current filter selections
   * @param filters Current filter selections
   * @returns Counts for all filter categories (updates based on active filters)
   */
  static async getDynamicFilterCounts(filters: Vip3Filters): Promise<FilterCounts> {
    try {
      const start = performance.now();

      const counts = await invoke<FilterCounts>('get_vip3_dynamic_filter_counts', {
        filters
      });

      const elapsed = performance.now() - start;

      if (elapsed > 50) {
        console.warn(`Dynamic filter counts took ${elapsed.toFixed(0)}ms (target: <50ms)`);
      } else {
        console.log(`Dynamic filter counts loaded in ${elapsed.toFixed(0)}ms`);
      }

      return counts;
    } catch (error) {
      console.error('Failed to get dynamic filter counts:', error);
      throw new Error(`Filter count error: ${error}`);
    }
  }

  /**
   * Get static filter counts (legacy, doesn't update with filters)
   * @deprecated Use getDynamicFilterCounts instead
   * @param filters Current filter selections
   * @returns Counts for all filter categories
   */
  static async getFilterCounts(filters: Vip3Filters): Promise<FilterCounts> {
    try {
      const start = performance.now();

      const counts = await invoke<FilterCounts>('get_vip3_filter_counts', {
        filters
      });

      const elapsed = performance.now() - start;

      if (elapsed > 50) {
        console.warn(`Filter counts took ${elapsed.toFixed(0)}ms (target: <50ms)`);
      } else {
        console.log(`Filter counts loaded in ${elapsed.toFixed(0)}ms`);
      }

      return counts;
    } catch (error) {
      console.error('Failed to get filter counts:', error);
      throw new Error(`Filter count error: ${error}`);
    }
  }

  /**
   * Search files with VIP3 filters
   * @param filters Filter selections
   * @param page Page number (0-indexed)
   * @param pageSize Number of results per page
   * @returns Search results with pagination info
   */
  static async searchFiles(
    filters: Vip3Filters,
    page: number = 0,
    pageSize: number = 50
  ): Promise<Vip3SearchResponse> {
    try {
      const response = await invoke<Vip3SearchResponse>('search_files_vip3', {
        filters,
        limit: pageSize,
        offset: page * pageSize
      });

      return {
        ...response,
        page,
        page_size: pageSize,
        has_more: response.total_count > (page + 1) * pageSize
      };
    } catch (error) {
      console.error('Failed to search files:', error);
      throw new Error(`Search error: ${error}`);
    }
  }

  /**
   * Get all available timbres
   */
  static async getTimbres(): Promise<Timbre[]> {
    try {
      return await invoke<Timbre[]>('get_all_timbres');
    } catch (error) {
      console.error('Failed to get timbres:', error);
      return [];
    }
  }

  /**
   * Get all available styles
   */
  static async getStyles(): Promise<Style[]> {
    try {
      return await invoke<Style[]>('get_all_styles');
    } catch (error) {
      console.error('Failed to get styles:', error);
      return [];
    }
  }

  /**
   * Get all available articulations
   */
  static async getArticulations(): Promise<Articulation[]> {
    try {
      return await invoke<Articulation[]>('get_all_articulations');
    } catch (error) {
      console.error('Failed to get articulations:', error);
      return [];
    }
  }

  // ========================================================================
  // Category List Fetching (for VIP3 Browser Initialization)
  // ========================================================================

  /**
   * Get all unique folders from the database
   * @returns Array of folder paths from files.parent_folder
   */
  static async getFolders(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_folders');
    } catch (error) {
      console.error('Failed to get folders:', error);
      return [];
    }
  }

  /**
   * Get all instruments (tags with category='instrument')
   * @returns Array of instrument names
   */
  static async getInstruments(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_instruments');
    } catch (error) {
      console.error('Failed to get instruments:', error);
      return [];
    }
  }

  /**
   * Get all timbre names (simple string list)
   * @returns Array of timbre names from timbres table
   */
  static async getTimbreNames(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_timbres');
    } catch (error) {
      console.error('Failed to get timbre names:', error);
      return [];
    }
  }

  /**
   * Get all style names (simple string list)
   * @returns Array of style names from styles table
   */
  static async getStyleNames(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_styles');
    } catch (error) {
      console.error('Failed to get style names:', error);
      return [];
    }
  }

  /**
   * Get all articulation names (simple string list)
   * @returns Array of articulation names from articulations table
   */
  static async getArticulationNames(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_articulations');
    } catch (error) {
      console.error('Failed to get articulation names:', error);
      return [];
    }
  }

  /**
   * Get all manufacturers
   * @returns Array of manufacturer names from files.manufacturer
   */
  static async getManufacturers(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_vip3_manufacturers');
    } catch (error) {
      console.error('Failed to get manufacturers:', error);
      return [];
    }
  }

  // ========================================================================
  // Combined Categories Fetching (for efficient VIP3 Browser initialization)
  // ========================================================================

  /**
   * All VIP3 categories response from backend
   */
  /**
   * Get all VIP3 categories in a single call (more efficient than 6 separate calls)
   * Uses parallel database queries for optimal performance
   * @returns All categories: folders, instruments, timbres, styles, articulations, manufacturers
   */
  static async getAllCategories(): Promise<AllVip3Categories> {
    try {
      const start = performance.now();

      const result = await invoke<AllVip3Categories>('get_all_vip3_categories');

      const elapsed = performance.now() - start;
      console.log(`✓ Loaded all VIP3 categories in ${elapsed.toFixed(0)}ms`);
      console.log(`  Folders: ${result.folders.length}, Instruments: ${result.instruments.length}`);
      console.log(`  Timbres: ${result.timbres.length}, Styles: ${result.styles.length}, Articulations: ${result.articulations.length}`);

      return result;
    } catch (error) {
      console.error('Failed to get all categories:', error);
      throw new Error(`Failed to get VIP3 categories: ${error}`);
    }
  }

  /**
   * Load a file into the DAW sequencer
   * @param fileId Database file ID
   * @returns Track ID in sequencer
   */
  static async loadFileToDaw(fileId: number): Promise<number> {
    try {
      const trackId = await invoke<number>('load_file_to_daw', {
        fileId
      });

      console.log(`Loaded file ${fileId} to DAW as track ${trackId}`);
      return trackId;
    } catch (error) {
      console.error('Failed to load file to DAW:', error);
      throw new Error(`DAW load error: ${error}`);
    }
  }

  // ========================================================================
  // Favorites Management
  // ========================================================================

  /**
   * Toggle favorite status for a file
   * @param fileId File ID to toggle
   * @returns New favorite status (true/false)
   */
  static async toggleFavorite(fileId: number): Promise<boolean> {
    try {
      return await invoke<boolean>('toggle_favorite', { fileId });
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
      throw new Error(`Toggle favorite error: ${error}`);
    }
  }

  /**
   * Set favorite status for a file
   * @param fileId File ID to update
   * @param favorite New favorite status
   */
  static async setFavorite(fileId: number, favorite: boolean): Promise<void> {
    try {
      await invoke('set_favorite', { fileId, favorite });
    } catch (error) {
      console.error('Failed to set favorite:', error);
      throw new Error(`Set favorite error: ${error}`);
    }
  }

  /**
   * Get all favorited files
   * @param page Page number
   * @param pageSize Number of results per page
   * @returns Paginated search results
   */
  static async getFavorites(page: number = 1, pageSize: number = 50): Promise<Vip3SearchResponse> {
    try {
      return await invoke<Vip3SearchResponse>('get_favorites', { page, pageSize });
    } catch (error) {
      console.error('Failed to get favorites:', error);
      throw new Error(`Get favorites error: ${error}`);
    }
  }

  /**
   * Get count of favorited files
   * @returns Number of favorites
   */
  static async getFavoriteCount(): Promise<number> {
    try {
      return await invoke<number>('get_favorite_count');
    } catch (error) {
      console.error('Failed to get favorite count:', error);
      throw new Error(`Get favorite count error: ${error}`);
    }
  }

  // ========================================================================
  // Saved Searches
  // ========================================================================

  /**
   * Save a search with filters
   * @param request Saved search details
   * @returns ID of created saved search
   */
  static async saveSearch(request: CreateSavedSearchRequest): Promise<number> {
    try {
      return await invoke<number>('save_search', request);
    } catch (error) {
      console.error('Failed to save search:', error);
      throw new Error(`Save search error: ${error}`);
    }
  }

  /**
   * Get all saved searches
   * @returns Array of saved searches
   */
  static async getSavedSearches(): Promise<SavedSearchResponse[]> {
    try {
      return await invoke<SavedSearchResponse[]>('get_saved_searches');
    } catch (error) {
      console.error('Failed to get saved searches:', error);
      throw new Error(`Get saved searches error: ${error}`);
    }
  }

  /**
   * Load a saved search (increments use count)
   * @param searchId ID of saved search
   * @returns Saved search details with filters
   */
  static async loadSavedSearch(searchId: number): Promise<SavedSearchResponse> {
    try {
      return await invoke<SavedSearchResponse>('load_saved_search', { searchId });
    } catch (error) {
      console.error('Failed to load saved search:', error);
      throw new Error(`Load saved search error: ${error}`);
    }
  }

  /**
   * Delete a saved search
   * @param searchId ID of saved search to delete
   */
  static async deleteSavedSearch(searchId: number): Promise<void> {
    try {
      await invoke('delete_saved_search', { searchId });
    } catch (error) {
      console.error('Failed to delete saved search:', error);
      throw new Error(`Delete saved search error: ${error}`);
    }
  }

  /**
   * Toggle pin status for a saved search
   * @param searchId ID of saved search
   * @returns New pinned status
   */
  static async toggleSavedSearchPin(searchId: number): Promise<boolean> {
    try {
      return await invoke<boolean>('toggle_saved_search_pin', { searchId });
    } catch (error) {
      console.error('Failed to toggle saved search pin:', error);
      throw new Error(`Toggle pin error: ${error}`);
    }
  }

  // ========================================================================
  // Collections
  // ========================================================================

  /**
   * Create a new collection
   * @param request Collection details
   * @returns ID of created collection
   */
  static async createCollection(request: CreateCollectionRequest): Promise<number> {
    try {
      return await invoke<number>('create_collection', request);
    } catch (error) {
      console.error('Failed to create collection:', error);
      throw new Error(`Create collection error: ${error}`);
    }
  }

  /**
   * Get all collections
   * @returns Array of collections with file counts
   */
  static async getCollections(): Promise<CollectionResponse[]> {
    try {
      return await invoke<CollectionResponse[]>('get_collections');
    } catch (error) {
      console.error('Failed to get collections:', error);
      throw new Error(`Get collections error: ${error}`);
    }
  }

  /**
   * Get a single collection by ID
   * @param collectionId Collection ID
   * @returns Collection details
   */
  static async getCollection(collectionId: number): Promise<CollectionResponse> {
    try {
      return await invoke<CollectionResponse>('get_collection', { collectionId });
    } catch (error) {
      console.error('Failed to get collection:', error);
      throw new Error(`Get collection error: ${error}`);
    }
  }

  /**
   * Update collection details
   * @param collectionId Collection ID
   * @param updates Fields to update
   */
  static async updateCollection(
    collectionId: number,
    updates: Partial<UpdateCollectionRequest>
  ): Promise<void> {
    try {
      await invoke('update_collection', { collectionId, ...updates });
    } catch (error) {
      console.error('Failed to update collection:', error);
      throw new Error(`Update collection error: ${error}`);
    }
  }

  /**
   * Delete a collection
   * @param collectionId Collection ID to delete
   */
  static async deleteCollection(collectionId: number): Promise<void> {
    try {
      await invoke('delete_collection', { collectionId });
    } catch (error) {
      console.error('Failed to delete collection:', error);
      throw new Error(`Delete collection error: ${error}`);
    }
  }

  /**
   * Add a file to a collection
   * @param collectionId Collection ID
   * @param fileId File ID to add
   */
  static async addFileToCollection(collectionId: number, fileId: number): Promise<void> {
    try {
      await invoke('add_file_to_collection', { collectionId, fileId });
    } catch (error) {
      console.error('Failed to add file to collection:', error);
      throw new Error(`Add file error: ${error}`);
    }
  }

  /**
   * Remove a file from a collection
   * @param collectionId Collection ID
   * @param fileId File ID to remove
   */
  static async removeFileFromCollection(collectionId: number, fileId: number): Promise<void> {
    try {
      await invoke('remove_file_from_collection', { collectionId, fileId });
    } catch (error) {
      console.error('Failed to remove file from collection:', error);
      throw new Error(`Remove file error: ${error}`);
    }
  }

  /**
   * Get files in a collection
   * @param collectionId Collection ID
   * @param page Page number
   * @param pageSize Number of results per page
   * @returns Paginated search results
   */
  static async getCollectionFiles(
    collectionId: number,
    page: number = 1,
    pageSize: number = 50
  ): Promise<Vip3SearchResponse> {
    try {
      return await invoke<Vip3SearchResponse>('get_collection_files', { collectionId, page, pageSize });
    } catch (error) {
      console.error('Failed to get collection files:', error);
      throw new Error(`Get collection files error: ${error}`);
    }
  }

  /**
   * Get files in a smart collection (auto-filter based)
   * Smart collections dynamically execute saved filters instead of storing file references
   * @param collectionId Collection ID (must be a smart collection)
   * @param page Page number
   * @param pageSize Number of results per page
   * @returns Paginated search results based on smart filters
   */
  static async getSmartCollectionFiles(
    collectionId: number,
    page: number = 1,
    pageSize: number = 50
  ): Promise<Vip3SearchResponse> {
    try {
      return await invoke<Vip3SearchResponse>('get_smart_collection_files', { collectionId, page, pageSize });
    } catch (error) {
      console.error('Failed to get smart collection files:', error);
      throw new Error(`Get smart collection files error: ${error}`);
    }
  }

  /**
   * Batch add multiple files to a collection
   * @param collectionId Collection ID
   * @param fileIds Array of file IDs to add
   * @returns Number of files added
   */
  static async batchAddFilesToCollection(collectionId: number, fileIds: number[]): Promise<number> {
    try {
      return await invoke<number>('batch_add_files_to_collection', { collectionId, fileIds });
    } catch (error) {
      console.error('Failed to batch add files:', error);
      throw new Error(`Batch add error: ${error}`);
    }
  }

  /**
   * Batch remove multiple files from a collection
   * @param collectionId Collection ID
   * @param fileIds Array of file IDs to remove
   * @returns Number of files removed
   */
  static async batchRemoveFilesFromCollection(collectionId: number, fileIds: number[]): Promise<number> {
    try {
      return await invoke<number>('batch_remove_files_from_collection', { collectionId, fileIds });
    } catch (error) {
      console.error('Failed to batch remove files:', error);
      throw new Error(`Batch remove error: ${error}`);
    }
  }

  /**
   * Clear all files from a collection
   * @param collectionId Collection ID
   * @returns Number of files removed
   */
  static async clearCollection(collectionId: number): Promise<number> {
    try {
      return await invoke<number>('clear_collection', { collectionId });
    } catch (error) {
      console.error('Failed to clear collection:', error);
      throw new Error(`Clear collection error: ${error}`);
    }
  }

  /**
   * Reorder files in a collection
   * @param collectionId Collection ID
   * @param fileIdsInOrder Array of file IDs in desired order
   */
  static async reorderCollectionFiles(collectionId: number, fileIdsInOrder: number[]): Promise<void> {
    try {
      await invoke('reorder_collection_files', { collectionId, fileIdsInOrder });
    } catch (error) {
      console.error('Failed to reorder collection files:', error);
      throw new Error(`Reorder files error: ${error}`);
    }
  }
}

/**
 * All VIP3 categories in a single response
 */
export interface AllVip3Categories {
  folders: string[];
  instruments: string[];
  timbres: string[];
  styles: string[];
  articulations: string[];
  manufacturers: string[];
}
