/**
 * Collections API
 *
 * CRUD operations for user-created file collections
 * Backend: pipeline/src-tauri/src/commands/collections.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type { Collection, CollectionWithFiles, BulkOperationResult, VIP3BrowserFilters } from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const COLLECTION_COMMANDS = {
  GET_ALL: 'get_all_collections',
  GET_BY_ID: 'get_collection_by_id',
  CREATE: 'create_collection',
  UPDATE: 'update_collection',
  DELETE: 'delete_collection',
  ADD_FILE: 'add_file_to_collection',
  REMOVE_FILE: 'remove_file_from_collection',
  ADD_FILES_BULK: 'add_files_to_collection_bulk',
  REMOVE_FILES_BULK: 'remove_files_from_collection_bulk',
  GET_FILES: 'get_collection_files',
  GET_FOR_FILE: 'get_collections_for_file',
  CREATE_SMART: 'create_smart_collection',
  UPDATE_SMART_CRITERIA: 'update_smart_collection_criteria',
  REFRESH_SMART: 'refresh_smart_collection',
  DUPLICATE: 'duplicate_collection',
  MERGE: 'merge_collections',
  EXPORT: 'export_collection',
} as const;

/**
 * Collection creation parameters
 */
export interface CreateCollectionParams {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
}

/**
 * Smart collection creation parameters
 */
export interface CreateSmartCollectionParams {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  filters: VIP3BrowserFilters;
}

/**
 * Collection update parameters
 */
export interface UpdateCollectionParams {
  id: number;
  name?: string;
  description?: string;
  color?: string;
  icon?: string;
}

/**
 * Collection export options
 */
export interface CollectionExportOptions {
  format: 'json' | 'csv' | 'playlist';
  includePaths: boolean;
  includeMetadata: boolean;
}

/**
 * Collections API module
 */
export const collectionsApi = {
  /**
   * Get all collections
   */
  getAll: async (): Promise<Collection[]> => {
    try {
      return await invoke<Collection[]>(COLLECTION_COMMANDS.GET_ALL);
    } catch (error) {
      console.error('[collectionsApi] Failed to get all collections:', error);
      throw error;
    }
  },

  /**
   * Get collection by ID (with file IDs)
   */
  getById: async (id: number): Promise<CollectionWithFiles | null> => {
    try {
      return await invoke<CollectionWithFiles | null>(COLLECTION_COMMANDS.GET_BY_ID, { id });
    } catch (error) {
      console.error(`[collectionsApi] Failed to get collection ${id}:`, error);
      throw error;
    }
  },

  /**
   * Create a new collection
   */
  create: async (params: CreateCollectionParams): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.CREATE, { params });
    } catch (error) {
      console.error('[collectionsApi] Failed to create collection:', error);
      throw error;
    }
  },

  /**
   * Create a smart collection (auto-populated based on search criteria)
   */
  createSmart: async (params: CreateSmartCollectionParams): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.CREATE_SMART, { params });
    } catch (error) {
      console.error('[collectionsApi] Failed to create smart collection:', error);
      throw error;
    }
  },

  /**
   * Update an existing collection
   */
  update: async (params: UpdateCollectionParams): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.UPDATE, { params });
    } catch (error) {
      console.error(`[collectionsApi] Failed to update collection ${params.id}:`, error);
      throw error;
    }
  },

  /**
   * Update smart collection search criteria
   */
  updateSmartCriteria: async (id: number, filters: VIP3BrowserFilters): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.UPDATE_SMART_CRITERIA, {
        id,
        filters,
      });
    } catch (error) {
      console.error(`[collectionsApi] Failed to update smart collection ${id} criteria:`, error);
      throw error;
    }
  },

  /**
   * Delete a collection
   */
  delete: async (id: number): Promise<void> => {
    try {
      await invoke(COLLECTION_COMMANDS.DELETE, { id });
    } catch (error) {
      console.error(`[collectionsApi] Failed to delete collection ${id}:`, error);
      throw error;
    }
  },

  /**
   * Add a file to a collection
   */
  addFile: async (collectionId: number, fileId: number): Promise<void> => {
    try {
      await invoke(COLLECTION_COMMANDS.ADD_FILE, {
        collection_id: collectionId,
        file_id: fileId,
      });
    } catch (error) {
      console.error(
        `[collectionsApi] Failed to add file ${fileId} to collection ${collectionId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Remove a file from a collection
   */
  removeFile: async (collectionId: number, fileId: number): Promise<void> => {
    try {
      await invoke(COLLECTION_COMMANDS.REMOVE_FILE, {
        collection_id: collectionId,
        file_id: fileId,
      });
    } catch (error) {
      console.error(
        `[collectionsApi] Failed to remove file ${fileId} from collection ${collectionId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Add multiple files to a collection
   */
  addFilesBulk: async (
    collectionId: number,
    fileIds: number[]
  ): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(COLLECTION_COMMANDS.ADD_FILES_BULK, {
        collection_id: collectionId,
        file_ids: fileIds,
      });
    } catch (error) {
      console.error(
        `[collectionsApi] Failed to add files to collection ${collectionId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Remove multiple files from a collection
   */
  removeFilesBulk: async (
    collectionId: number,
    fileIds: number[]
  ): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(COLLECTION_COMMANDS.REMOVE_FILES_BULK, {
        collection_id: collectionId,
        file_ids: fileIds,
      });
    } catch (error) {
      console.error(
        `[collectionsApi] Failed to remove files from collection ${collectionId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Get file IDs in a collection
   */
  getFiles: async (collectionId: number): Promise<number[]> => {
    try {
      return await invoke<number[]>(COLLECTION_COMMANDS.GET_FILES, {
        collection_id: collectionId,
      });
    } catch (error) {
      console.error(`[collectionsApi] Failed to get files for collection ${collectionId}:`, error);
      throw error;
    }
  },

  /**
   * Get collections that contain a specific file
   */
  getForFile: async (fileId: number): Promise<Collection[]> => {
    try {
      return await invoke<Collection[]>(COLLECTION_COMMANDS.GET_FOR_FILE, { file_id: fileId });
    } catch (error) {
      console.error(`[collectionsApi] Failed to get collections for file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Refresh a smart collection (re-run search criteria)
   */
  refreshSmart: async (id: number): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.REFRESH_SMART, { id });
    } catch (error) {
      console.error(`[collectionsApi] Failed to refresh smart collection ${id}:`, error);
      throw error;
    }
  },

  /**
   * Duplicate a collection
   */
  duplicate: async (id: number, newName?: string): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.DUPLICATE, {
        id,
        new_name: newName,
      });
    } catch (error) {
      console.error(`[collectionsApi] Failed to duplicate collection ${id}:`, error);
      throw error;
    }
  },

  /**
   * Merge multiple collections into one
   */
  merge: async (sourceIds: number[], targetId: number): Promise<Collection> => {
    try {
      return await invoke<Collection>(COLLECTION_COMMANDS.MERGE, {
        source_ids: sourceIds,
        target_id: targetId,
      });
    } catch (error) {
      console.error('[collectionsApi] Failed to merge collections:', error);
      throw error;
    }
  },

  /**
   * Export a collection
   */
  export: async (id: number, options: CollectionExportOptions): Promise<string> => {
    try {
      return await invoke<string>(COLLECTION_COMMANDS.EXPORT, { id, options });
    } catch (error) {
      console.error(`[collectionsApi] Failed to export collection ${id}:`, error);
      throw error;
    }
  },
};

export default collectionsApi;
