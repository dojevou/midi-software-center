/**
 * Search and filter API
 */

import { invokeCommand } from './client';
import type {
  SearchResults,
  SearchFilters,
  FileMetadata,
} from './types';

/**
 * Searches files with filters and pagination
 */
export async function searchFiles(
  query: string,
  filters: SearchFilters = {},
  page: number = 1,
  pageSize: number = 50
): Promise<SearchResults> {
  return invokeCommand<SearchResults>('search_files', {
    query,
    filters,
    page,
    pageSize,
  });
}

/**
 * Gets all available tags
 */
export async function getAllTags(): Promise<string[]> {
  return invokeCommand<string[]>('get_all_tags');
}

/**
 * Gets files by tag
 */
export async function getFilesByTag(tag: string): Promise<FileMetadata[]> {
  return invokeCommand<FileMetadata[]>('get_files_by_tag', {
    tag,
  });
}

/**
 * Gets BPM range from database
 */
export async function getBpmRange(): Promise<{ min: number; max: number }> {
  return invokeCommand<{ min: number; max: number }>('get_bpm_range');
}

/**
 * Gets all unique keys from database
 */
export async function getAllKeys(): Promise<string[]> {
  return invokeCommand<string[]>('get_all_keys');
}
