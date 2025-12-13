/**
 * Timbres API
 *
 * CRUD operations for timbre categories (sound classification)
 * Backend: pipeline/src-tauri/src/commands/timbres.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type { Timbre, BulkOperationResult } from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const TIMBRE_COMMANDS = {
  GET_ALL: 'get_all_timbres',
  GET_BY_ID: 'get_timbre_by_id',
  CREATE: 'create_timbre',
  UPDATE: 'update_timbre',
  DELETE: 'delete_timbre',
  GET_FOR_FILE: 'get_timbres_for_file',
  ASSIGN_TO_FILE: 'assign_timbre_to_file',
  REMOVE_FROM_FILE: 'remove_timbre_from_file',
  ASSIGN_BULK: 'assign_timbres_bulk',
  GET_COUNTS: 'get_timbre_counts',
} as const;

/**
 * Timbre creation parameters
 */
export interface CreateTimbreParams {
  name: string;
  description?: string;
  icon?: string;
  sort_order?: number;
}

/**
 * Timbre update parameters
 */
export interface UpdateTimbreParams {
  id: number;
  name?: string;
  description?: string;
  icon?: string;
  sort_order?: number;
}

/**
 * Timbre counts by ID
 */
export type TimbreCounts = Record<number, number>;

/**
 * Timbres API module
 */
export const timbresApi = {
  /**
   * Get all timbres
   */
  getAll: async (): Promise<Timbre[]> => {
    try {
      return await invoke<Timbre[]>(TIMBRE_COMMANDS.GET_ALL);
    } catch (error) {
      console.error('[timbresApi] Failed to get all timbres:', error);
      throw error;
    }
  },

  /**
   * Get timbre by ID
   */
  getById: async (id: number): Promise<Timbre | null> => {
    try {
      return await invoke<Timbre | null>(TIMBRE_COMMANDS.GET_BY_ID, { id });
    } catch (error) {
      console.error(`[timbresApi] Failed to get timbre ${id}:`, error);
      throw error;
    }
  },

  /**
   * Create a new timbre
   */
  create: async (params: CreateTimbreParams): Promise<Timbre> => {
    try {
      return await invoke<Timbre>(TIMBRE_COMMANDS.CREATE, { params });
    } catch (error) {
      console.error('[timbresApi] Failed to create timbre:', error);
      throw error;
    }
  },

  /**
   * Update an existing timbre
   */
  update: async (params: UpdateTimbreParams): Promise<Timbre> => {
    try {
      return await invoke<Timbre>(TIMBRE_COMMANDS.UPDATE, { params });
    } catch (error) {
      console.error(`[timbresApi] Failed to update timbre ${params.id}:`, error);
      throw error;
    }
  },

  /**
   * Delete a timbre
   */
  delete: async (id: number): Promise<void> => {
    try {
      await invoke(TIMBRE_COMMANDS.DELETE, { id });
    } catch (error) {
      console.error(`[timbresApi] Failed to delete timbre ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get timbres assigned to a file
   */
  getForFile: async (fileId: number): Promise<Timbre[]> => {
    try {
      return await invoke<Timbre[]>(TIMBRE_COMMANDS.GET_FOR_FILE, { file_id: fileId });
    } catch (error) {
      console.error(`[timbresApi] Failed to get timbres for file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Assign a timbre to a file
   */
  assignToFile: async (fileId: number, timbreId: number): Promise<void> => {
    try {
      await invoke(TIMBRE_COMMANDS.ASSIGN_TO_FILE, { file_id: fileId, timbre_id: timbreId });
    } catch (error) {
      console.error(`[timbresApi] Failed to assign timbre ${timbreId} to file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Remove a timbre from a file
   */
  removeFromFile: async (fileId: number, timbreId: number): Promise<void> => {
    try {
      await invoke(TIMBRE_COMMANDS.REMOVE_FROM_FILE, { file_id: fileId, timbre_id: timbreId });
    } catch (error) {
      console.error(`[timbresApi] Failed to remove timbre ${timbreId} from file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Bulk assign timbres to multiple files
   */
  assignBulk: async (
    fileIds: number[],
    timbreIds: number[]
  ): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(TIMBRE_COMMANDS.ASSIGN_BULK, {
        file_ids: fileIds,
        timbre_ids: timbreIds,
      });
    } catch (error) {
      console.error('[timbresApi] Failed to bulk assign timbres:', error);
      throw error;
    }
  },

  /**
   * Get timbre counts (number of files per timbre)
   */
  getCounts: async (): Promise<TimbreCounts> => {
    try {
      return await invoke<TimbreCounts>(TIMBRE_COMMANDS.GET_COUNTS);
    } catch (error) {
      console.error('[timbresApi] Failed to get timbre counts:', error);
      throw error;
    }
  },
};

export default timbresApi;
