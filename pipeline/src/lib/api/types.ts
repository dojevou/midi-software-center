/**
 * API-specific types that match Rust backend structures
 */

import type {
  FileMetadata,
  FileCategory,
  MusicalKey,
  ImportSummary,
  SearchFilters,
  SearchResults,
} from '$lib/types';

// Re-export for convenience
export type {
  FileMetadata,
  FileCategory,
  MusicalKey,
  ImportSummary,
  SearchFilters,
  SearchResults,
};

/**
 * Tauri command result types
 */
export interface TauriResult<T> {
  data?: T;
  error?: string;
}

/**
 * File import request
 */
export interface ImportFileRequest {
  filePath: string;
}

/**
 * Directory import request
 */
export interface ImportDirectoryRequest {
  directoryPath: string;
  recursive: boolean;
}

/**
 * Search request
 */
export interface SearchRequest {
  query: string;
  filters: SearchFilters;
  page: number;
  pageSize: number;
}

/**
 * Update tags request
 */
export interface UpdateTagsRequest {
  fileId: number;
  tags: string[];
}

/**
 * Get file details request
 */
export interface GetFileRequest {
  fileId: number;
}
