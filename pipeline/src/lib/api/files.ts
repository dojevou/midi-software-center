/**
 * File management API
 */

import { invokeCommand } from './client';
import type {
  FileMetadata,
  ImportSummary,
} from './types';

/**
 * Imports a single MIDI file
 */
export async function importSingleFile(filePath: string): Promise<number> {
  return invokeCommand<number>('import_single_file', {
    filePath,
  });
}

/**
 * Imports an entire directory of MIDI files
 */
export async function importDirectory(
  directoryPath: string,
  recursive: boolean = true
): Promise<ImportSummary> {
  return invokeCommand<ImportSummary>('import_directory', {
    directoryPath,
    recursive,
  });
}

/**
 * Gets file details by ID
 */
export async function getFile(fileId: number): Promise<FileMetadata> {
  return invokeCommand<FileMetadata>('get_file', {
    fileId,
  });
}

/**
 * Gets total file count
 */
export async function getFileCount(): Promise<number> {
  return invokeCommand<number>('get_file_count');
}

/**
 * Lists files with pagination
 */
export async function listFiles(
  limit: number = 50,
  offset: number = 0
): Promise<FileMetadata[]> {
  return invokeCommand<FileMetadata[]>('list_files', {
    limit,
    offset,
  });
}

/**
 * Updates file tags
 */
export async function updateFileTags(
  fileId: number,
  tags: string[]
): Promise<void> {
  return invokeCommand<void>('update_file_tags', {
    fileId,
    tags,
  });
}

/**
 * Deletes a file
 */
export async function deleteFile(fileId: number): Promise<void> {
  return invokeCommand<void>('delete_file', {
    fileId,
  });
}

/**
 * Gets files by category
 */
export async function getFilesByCategory(
  category: string,
  limit: number = 50
): Promise<FileMetadata[]> {
  return invokeCommand<FileMetadata[]>('get_files_by_category', {
    category,
    limit,
  });
}

/**
 * Gets recently imported files
 */
export async function getRecentFiles(limit: number = 10): Promise<FileMetadata[]> {
  return invokeCommand<FileMetadata[]>('get_recent_files', {
    limit,
  });
}
