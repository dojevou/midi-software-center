/**
 * Styles API
 *
 * CRUD operations for style/genre categories
 * Backend: pipeline/src-tauri/src/commands/styles.rs
 */

import { invoke } from '@tauri-apps/api/core';
import type { Style, BulkOperationResult } from '../types';

// Command constants (to be added to commands.ts when backend is ready)
const STYLE_COMMANDS = {
  GET_ALL: 'get_all_styles',
  GET_BY_ID: 'get_style_by_id',
  GET_CHILDREN: 'get_style_children',
  CREATE: 'create_style',
  UPDATE: 'update_style',
  DELETE: 'delete_style',
  GET_FOR_FILE: 'get_styles_for_file',
  ASSIGN_TO_FILE: 'assign_style_to_file',
  REMOVE_FROM_FILE: 'remove_style_from_file',
  ASSIGN_BULK: 'assign_styles_bulk',
  GET_COUNTS: 'get_style_counts',
  GET_HIERARCHY: 'get_style_hierarchy',
} as const;

/**
 * Style creation parameters
 */
export interface CreateStyleParams {
  name: string;
  description?: string;
  parent_id?: number;
  icon?: string;
  sort_order?: number;
}

/**
 * Style update parameters
 */
export interface UpdateStyleParams {
  id: number;
  name?: string;
  description?: string;
  parent_id?: number;
  icon?: string;
  sort_order?: number;
}

/**
 * Style with children (hierarchical)
 */
export interface StyleWithChildren extends Style {
  children: StyleWithChildren[];
}

/**
 * Style counts by ID
 */
export type StyleCounts = Record<number, number>;

/**
 * Styles API module
 */
export const stylesApi = {
  /**
   * Get all styles (flat list)
   */
  getAll: async (): Promise<Style[]> => {
    try {
      return await invoke<Style[]>(STYLE_COMMANDS.GET_ALL);
    } catch (error) {
      console.error('[stylesApi] Failed to get all styles:', error);
      throw error;
    }
  },

  /**
   * Get style by ID
   */
  getById: async (id: number): Promise<Style | null> => {
    try {
      return await invoke<Style | null>(STYLE_COMMANDS.GET_BY_ID, { id });
    } catch (error) {
      console.error(`[stylesApi] Failed to get style ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get children of a style (for hierarchical display)
   */
  getChildren: async (parentId: number): Promise<Style[]> => {
    try {
      return await invoke<Style[]>(STYLE_COMMANDS.GET_CHILDREN, { parent_id: parentId });
    } catch (error) {
      console.error(`[stylesApi] Failed to get children of style ${parentId}:`, error);
      throw error;
    }
  },

  /**
   * Get full style hierarchy
   */
  getHierarchy: async (): Promise<StyleWithChildren[]> => {
    try {
      return await invoke<StyleWithChildren[]>(STYLE_COMMANDS.GET_HIERARCHY);
    } catch (error) {
      console.error('[stylesApi] Failed to get style hierarchy:', error);
      throw error;
    }
  },

  /**
   * Create a new style
   */
  create: async (params: CreateStyleParams): Promise<Style> => {
    try {
      return await invoke<Style>(STYLE_COMMANDS.CREATE, { params });
    } catch (error) {
      console.error('[stylesApi] Failed to create style:', error);
      throw error;
    }
  },

  /**
   * Update an existing style
   */
  update: async (params: UpdateStyleParams): Promise<Style> => {
    try {
      return await invoke<Style>(STYLE_COMMANDS.UPDATE, { params });
    } catch (error) {
      console.error(`[stylesApi] Failed to update style ${params.id}:`, error);
      throw error;
    }
  },

  /**
   * Delete a style
   */
  delete: async (id: number): Promise<void> => {
    try {
      await invoke(STYLE_COMMANDS.DELETE, { id });
    } catch (error) {
      console.error(`[stylesApi] Failed to delete style ${id}:`, error);
      throw error;
    }
  },

  /**
   * Get styles assigned to a file
   */
  getForFile: async (fileId: number): Promise<Style[]> => {
    try {
      return await invoke<Style[]>(STYLE_COMMANDS.GET_FOR_FILE, { file_id: fileId });
    } catch (error) {
      console.error(`[stylesApi] Failed to get styles for file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Assign a style to a file
   */
  assignToFile: async (fileId: number, styleId: number): Promise<void> => {
    try {
      await invoke(STYLE_COMMANDS.ASSIGN_TO_FILE, { file_id: fileId, style_id: styleId });
    } catch (error) {
      console.error(`[stylesApi] Failed to assign style ${styleId} to file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Remove a style from a file
   */
  removeFromFile: async (fileId: number, styleId: number): Promise<void> => {
    try {
      await invoke(STYLE_COMMANDS.REMOVE_FROM_FILE, { file_id: fileId, style_id: styleId });
    } catch (error) {
      console.error(`[stylesApi] Failed to remove style ${styleId} from file ${fileId}:`, error);
      throw error;
    }
  },

  /**
   * Bulk assign styles to multiple files
   */
  assignBulk: async (
    fileIds: number[],
    styleIds: number[]
  ): Promise<BulkOperationResult> => {
    try {
      return await invoke<BulkOperationResult>(STYLE_COMMANDS.ASSIGN_BULK, {
        file_ids: fileIds,
        style_ids: styleIds,
      });
    } catch (error) {
      console.error('[stylesApi] Failed to bulk assign styles:', error);
      throw error;
    }
  },

  /**
   * Get style counts (number of files per style)
   */
  getCounts: async (): Promise<StyleCounts> => {
    try {
      return await invoke<StyleCounts>(STYLE_COMMANDS.GET_COUNTS);
    } catch (error) {
      console.error('[stylesApi] Failed to get style counts:', error);
      throw error;
    }
  },
};

export default stylesApi;
