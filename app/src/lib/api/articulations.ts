/**
 * Articulations API
 *
 * CRUD operations for articulation types (playing techniques)
 * Backend: pipeline/src-tauri/src/commands/articulations.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type { Articulation, BulkOperationResult } from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const ARTICULATION_COMMANDS = {
  GET_ALL: 'get_all_articulations',
  GET_BY_ID: 'get_articulation_by_id',
  CREATE: 'create_articulation',
  UPDATE: 'update_articulation',
  DELETE: 'delete_articulation',
  GET_FOR_FILE: 'get_articulations_for_file',
  ASSIGN_TO_FILE: 'assign_articulation_to_file',
  REMOVE_FROM_FILE: 'remove_articulation_from_file',
  ASSIGN_BULK: 'assign_articulations_bulk',
  GET_COUNTS: 'get_articulation_counts',
} as const;

/**
 * Articulation creation parameters
 */
export interface CreateArticulationParams {
  name: string;
  description?: string;
  abbreviation?: string;
  icon?: string;
  sort_order?: number;
}

/**
 * Articulation update parameters
 */
export interface UpdateArticulationParams {
  id: number;
  name?: string;
  description?: string;
  abbreviation?: string;
  icon?: string;
  sort_order?: number;
}

/**
 * Articulation counts by ID
 */
export type ArticulationCounts = Record<number, number>;

/**
 * Articulations API module
 */
export const articulationsApi = {
  /**
   * Get all articulations
   */
  getAll: async (): Promise<Articulation[]> => {
    try {
      return await invoke<Articulation[]>(ARTICULATION_COMMANDS.GET_ALL);
    } catch (error) {
      console.error('[articulationsApi] Failed to get all articulations:', error);
      throw error;
    }
  },

  /**
   * Get articulation by ID
   */
  getById: async (id: number): Promise<Articulation | null> => {
    try {
      return await invoke<Articulation | null>(ARTICULATION_COMMANDS.GET_BY_ID, { id });
    } catch (error) {
      console.error(`[articulationsApi] Failed to get articulation ${id}:`, error);
      throw error;
    }
  },

  /**
   * Create a new articulation
   */
  create: async (params: CreateArticulationParams): Promise<Articulation> => {
    try {
      return await invoke<Articulation>(ARTICULATION_COMMANDS.CREATE, { params });
    } catch (error) {
      console.error('[articulationsApi] Failed to create articulation:', error);
      throw error;
    }
  },

  /**
   * Update an existing articulation
   */
  update: async (params: UpdateArticulationParams): Promise<Articulation> => {
    try {
      return await invoke<Articulation>(ARTICULATION_COMMANDS.UPDATE, { params });
    } catch (error) {
      console.error(`[articulationsApi] Failed to update articulation ${params.id}:`, error);
      throw error;
    }
  },

  /**
   * Delete an articulation
   */
  delete: async (id: number): Promise<void> => {
    try {
      await invoke(ARTICULATION_COMMANDS.DELETE, { id });
    } catch (error) {
      console.error(`[articulationsApi] Failed to delete articulation ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get articulations assigned to a file
   */
  getForFile: async (fileId: number): Promise<Articulation[]> => {
    try {
      return await invoke<Articulation[]>(ARTICULATION_COMMANDS.GET_FOR_FILE, { file_id: fileId });
    } catch (error) {
      console.error(`[articulationsApi] Failed to get articulations for file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Assign an articulation to a file
   */
  assignToFile: async (fileId: number, articulationId: number): Promise<void> => {
    try {
      await invoke(ARTICULATION_COMMANDS.ASSIGN_TO_FILE, {
        file_id: fileId,
        articulation_id: articulationId,
      });
    } catch (error) {
      console.error(
        `[articulationsApi] Failed to assign articulation ${articulationId} to file ${fileId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Remove an articulation from a file
   */
  removeFromFile: async (fileId: number, articulationId: number): Promise<void> => {
    try {
      await invoke(ARTICULATION_COMMANDS.REMOVE_FROM_FILE, {
        file_id: fileId,
        articulation_id: articulationId,
      });
    } catch (error) {
      console.error(
        `[articulationsApi] Failed to remove articulation ${articulationId} from file ${fileId}:`,
        error
      );
      throw error;
    }
  },

  /**
   * Bulk assign articulations to multiple files
   */
  assignBulk: async (
    fileIds: number[],
    articulationIds: number[]
  ): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(ARTICULATION_COMMANDS.ASSIGN_BULK, {
        file_ids: fileIds,
        articulation_ids: articulationIds,
      });
    } catch (error) {
      console.error('[articulationsApi] Failed to bulk assign articulations:', error);
      throw error;
    }
  },

  /**
   * Get articulation counts (number of files per articulation)
   */
  getCounts: async (): Promise<ArticulationCounts> => {
    try {
      return await invoke<ArticulationCounts>(ARTICULATION_COMMANDS.GET_COUNTS);
    } catch (error) {
      console.error('[articulationsApi] Failed to get articulation counts:', error);
      throw error;
    }
  },
};

export default articulationsApi;
