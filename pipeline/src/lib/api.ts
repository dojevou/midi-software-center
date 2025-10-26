// Frontend API Wrapper - MANAGER Component
// Handles I/O operations (Tauri IPC) with error handling and retry logic

import { invoke } from '@tauri-apps/api/core';

// Import functions from api/ directory for re-export
import {
  listFiles as listFilesFromApi,
  updateFileTags as updateFileTagsFromApi,
  importSingleFile as importSingleFileFromApi,
  importDirectory as importDirectoryFromApi,
  getFile as getFileFromApi,
  deleteFile as deleteFileFromApi,
  getFilesByCategory as getFilesByCategoryFromApi,
  getRecentFiles as getRecentFilesFromApi,
} from './api/files';

import {
  getAllTags as getAllTagsFromApi,
  getFilesByTag as getFilesByTagFromApi,
  getBpmRange as getBpmRangeFromApi,
  getAllKeys as getAllKeysFromApi,
} from './api/search';

import {
  pickFile as pickFileFromApi,
  pickDirectory as pickDirectoryFromApi,
  pickMultipleFiles as pickMultipleFilesFromApi,
  getSystemInfo as getSystemInfoFromApi,
  checkDatabaseConnection as checkDatabaseConnectionFromApi,
} from './api/system';

import {
  testDatabaseConnection as testDatabaseConnectionFromApiV2,
  pingDatabase as pingDatabaseFromApi,
} from './api/connection';

import {
  getCategoryStats as getCategoryStatsFromApi,
  getManufacturerStats as getManufacturerStatsFromApi,
  getKeySignatureStats as getKeySignatureStatsFromApi,
  getRecentlyAddedCount as getRecentlyAddedCountFromApi,
  getDuplicateCount as getDuplicateCountFromApi,
  getDatabaseSize as getDatabaseSizeFromApi,
  checkDatabaseHealth as checkDatabaseHealthFromApi,
  getFileCount as getFileCountFromStatsApi,
} from './api/stats';

import {
  startProcessing as startProcessingFromApi,
  getProcessingProgress as getProcessingProgressFromApi,
  pauseProcessing as pauseProcessingFromApi,
  resumeProcessing as resumeProcessingFromApi,
  cancelProcessing as cancelProcessingFromApi,
} from './api/processing';

import {
  listenToImportProgress as listenToImportProgressFromApi,
  listenToFileAdded as listenToFileAddedFromApi,
  listenToFileUpdated as listenToFileUpdatedFromApi,
  listenToFileDeleted as listenToFileDeletedFromApi,
} from './api/events';

import {
  ApiError,
  getErrorMessage as getErrorMessageFromApi,
  isApiError,
} from './api/client';

// ============================================================================
// Type Definitions
// ============================================================================

export interface MidiFile {
  id: number;
  filename: string;
  filepath: string;
  bpm: number | null;
  key_signature: string | null;
  duration_seconds: number | null;
  created_at: string;
}

export interface SearchFilters {
  query: string;
  bpmMin?: number;
  bpmMax?: number;
}

export interface ProgressData {
  stage: string;
  current_file: string;
  files_processed: number;
  files_total: number;
  files_per_second: number;
  errors_count: number;
}

export interface ImportSettings {
  source_path: string;
  recursive: boolean;
  skip_duplicates: boolean;
}

// ============================================================================
// Custom Error Class
// ============================================================================

export class APIError extends Error {
  public readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = 'APIError';
    this.code = code;

    // Maintains proper stack trace for where error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, APIError);
    }
  }

  /**
   * Determines if this error is retryable
   *
   * Retryable errors:
   * - CONNECTION_FAILED: Database connection issues
   * - TIMEOUT: Request timed out
   * - UNKNOWN: Unknown errors (might be transient)
   *
   * Non-retryable errors:
   * - NOT_FOUND: Resource doesn't exist
   * - INVALID_INPUT: Bad request data
   * - PERMISSION_DENIED: Authorization issues
   */
  public isRetryable(): boolean {
    const retryableCodes = [
      'CONNECTION_FAILED',
      'TIMEOUT',
      'UNKNOWN',
      'NETWORK_ERROR',
      'DATABASE_BUSY'
    ];

    return retryableCodes.includes(this.code);
  }
}

// ============================================================================
// Retry Helper
// ============================================================================

/**
 * Execute a function with retry logic using exponential backoff
 *
 * @param fn - The async function to execute
 * @param maxAttempts - Maximum number of attempts (default: 3)
 * @returns The result of the function
 * @throws APIError if all attempts fail
 */
async function withRetry<T>(
  fn: () => Promise<T>,
  maxAttempts: number = 3
): Promise<T> {
  let lastError: APIError | null = null;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      // Convert to APIError if not already
      const apiError = error instanceof APIError
        ? error
        : new APIError(
            error instanceof Error ? error.message : String(error),
            'UNKNOWN'
          );

      lastError = apiError;

      // Don't retry if error is not retryable
      if (!apiError.isRetryable()) {
        throw apiError;
      }

      // Don't wait after last attempt
      if (attempt === maxAttempts) {
        break;
      }

      // Exponential backoff: 1s, 2s, 4s
      const delayMs = Math.pow(2, attempt - 1) * 1000;
      console.warn(
        `API call failed (attempt ${attempt}/${maxAttempts}), ` +
        `retrying in ${delayMs}ms:`,
        apiError.message
      );

      await new Promise(resolve => setTimeout(resolve, delayMs));
    }
  }

  // All attempts failed
  throw lastError || new APIError('All retry attempts failed', 'RETRY_EXHAUSTED');
}

// ============================================================================
// Error Handler Helper
// ============================================================================

/**
 * Convert Tauri invoke errors to APIError instances
 *
 * @param error - The caught error
 * @returns APIError instance
 */
function handleInvokeError(error: unknown): APIError {
  if (error instanceof APIError) {
    return error;
  }

  const errorMessage = error instanceof Error ? error.message : String(error);

  // Parse error codes from backend error messages
  if (errorMessage.includes('Connection') || errorMessage.includes('connect')) {
    return new APIError(errorMessage, 'CONNECTION_FAILED');
  }

  if (errorMessage.includes('not found') || errorMessage.includes('Not found')) {
    return new APIError(errorMessage, 'NOT_FOUND');
  }

  if (errorMessage.includes('timeout') || errorMessage.includes('Timeout')) {
    return new APIError(errorMessage, 'TIMEOUT');
  }

  if (errorMessage.includes('Invalid') || errorMessage.includes('invalid')) {
    return new APIError(errorMessage, 'INVALID_INPUT');
  }

  if (errorMessage.includes('Permission') || errorMessage.includes('permission')) {
    return new APIError(errorMessage, 'PERMISSION_DENIED');
  }

  if (errorMessage.includes('Database') || errorMessage.includes('database')) {
    return new APIError(errorMessage, 'DATABASE_ERROR');
  }

  // Unknown error - potentially retryable
  return new APIError(errorMessage, 'UNKNOWN');
}

// ============================================================================
// API Functions
// ============================================================================

/**
 * Test database connection
 *
 * @returns true if connected, false otherwise
 * @throws APIError on connection failure
 */
export async function testDatabaseConnection(): Promise<boolean> {
  try {
    const result = await invoke<boolean>('test_db_connection');
    return result;
  } catch (error) {
    const apiError = handleInvokeError(error);
    console.error('Database connection test failed:', apiError);
    throw apiError;
  }
}

/**
 * Get total count of MIDI files in database
 *
 * @returns Total number of files
 * @throws APIError on failure
 */
export async function getFileCount(): Promise<number> {
  try {
    const count = await invoke<number>('get_file_count');
    return count;
  } catch (error) {
    const apiError = handleInvokeError(error);
    console.error('Failed to get file count:', apiError);
    throw apiError;
  }
}

/**
 * Search for MIDI files with filters
 * Includes automatic retry with exponential backoff
 *
 * @param filters - Search criteria
 * @returns Array of matching MIDI files
 * @throws APIError on failure after all retries
 */
export async function searchFiles(filters: SearchFilters): Promise<MidiFile[]> {
  return await withRetry(async () => {
    try {
      const results = await invoke<MidiFile[]>('search_files', {
        query: filters.query,
        bpmMin: filters.bpmMin ?? null,
        bpmMax: filters.bpmMax ?? null,
      });
      return results;
    } catch (error) {
      throw handleInvokeError(error);
    }
  }, 3);
}

/**
 * Get detailed information for a specific file
 *
 * @param fileId - The ID of the file to retrieve
 * @returns MidiFile object with full details
 * @throws APIError if file not found or on database error
 */
export async function getFileDetails(fileId: number): Promise<MidiFile> {
  try {
    const file = await invoke<MidiFile>('get_file_details', { fileId });
    return file;
  } catch (error) {
    const apiError = handleInvokeError(error);
    console.error(`Failed to get file details for ID ${fileId}:`, apiError);
    throw apiError;
  }
}

/**
 * Start file import process
 *
 * @param settings - Import configuration
 * @returns void - Use event listeners to track progress
 * @throws APIError on failure to start import
 */
export async function startImport(settings: ImportSettings): Promise<void> {
  try {
    await invoke('start_import', {
      sourcePath: settings.source_path,
      recursive: settings.recursive,
      skipDuplicates: settings.skip_duplicates,
    });
  } catch (error) {
    const apiError = handleInvokeError(error);
    console.error('Failed to start import:', apiError);
    throw apiError;
  }
}

/**
 * Stop ongoing import process
 *
 * @returns void
 * @throws APIError on failure to stop import
 */
export async function stopImport(): Promise<void> {
  try {
    await invoke('stop_import');
  } catch (error) {
    const apiError = handleInvokeError(error);
    console.error('Failed to stop import:', apiError);
    throw apiError;
  }
}

// ============================================================================
// Re-export functions from api/ directory
// ============================================================================

// Re-export all functions from the api/ directory for convenience
export { listFilesFromApi as listFiles };
export { updateFileTagsFromApi as updateFileTags };
export { importSingleFileFromApi as importSingleFile };
export { importDirectoryFromApi as importDirectory };
export { getFileFromApi as getFile };
export { deleteFileFromApi as deleteFile };
export { getFilesByCategoryFromApi as getFilesByCategory };
export { getRecentFilesFromApi as getRecentFiles };
export { getAllTagsFromApi as getAllTags };
export { getFilesByTagFromApi as getFilesByTag };
export { getBpmRangeFromApi as getBpmRange };
export { getAllKeysFromApi as getAllKeys };
export { pickFileFromApi as pickFile };
export { pickDirectoryFromApi as pickDirectory };
export { pickMultipleFilesFromApi as pickMultipleFiles };
export { getSystemInfoFromApi as getSystemInfo };
export { checkDatabaseConnectionFromApi as checkDatabaseConnection };
export { testDatabaseConnectionFromApiV2 as testDatabaseConnectionV2 };
export { pingDatabaseFromApi as pingDatabase };
export { getCategoryStatsFromApi as getCategoryStats };
export { getManufacturerStatsFromApi as getManufacturerStats };
export { getKeySignatureStatsFromApi as getKeySignatureStats };
export { getRecentlyAddedCountFromApi as getRecentlyAddedCount };
export { getDuplicateCountFromApi as getDuplicateCount };
export { getDatabaseSizeFromApi as getDatabaseSize };
export { checkDatabaseHealthFromApi as checkDatabaseHealth };
export { getFileCountFromStatsApi as getFileCountV2 };
export { startProcessingFromApi as startProcessing };
export { getProcessingProgressFromApi as getProcessingProgress };
export { pauseProcessingFromApi as pauseProcessing };
export { resumeProcessingFromApi as resumeProcessing };
export { cancelProcessingFromApi as cancelProcessing };
export { listenToImportProgressFromApi as listenToImportProgress };
export { listenToFileAddedFromApi as listenToFileAdded };
export { listenToFileUpdatedFromApi as listenToFileUpdated };
export { listenToFileDeletedFromApi as listenToFileDeleted };

// Re-export error utilities
export { ApiError, isApiError };
export { getErrorMessageFromApi as getErrorMessage };

// Re-export types
export type { FileMetadata, ImportSummary } from './api/types';
export type { ProcessingPhase, ProcessingStats, ProcessingProgress, ProcessingConfig } from './api/processing';

// ============================================================================
// API Object Export (Alternative usage pattern)
// ============================================================================

/**
 * API object with all functions
 * Can be used as: import { api } from '$lib/api'; api.searchFiles(...)
 */
export const api = {
  testDatabaseConnection,
  getFileCount,
  searchFiles,
  getFileDetails,
  startImport,
  stopImport,
  // From api/ directory
  listFiles: listFilesFromApi,
  updateFileTags: updateFileTagsFromApi,
  importSingleFile: importSingleFileFromApi,
  importDirectory: importDirectoryFromApi,
  getFile: getFileFromApi,
  deleteFile: deleteFileFromApi,
  getFilesByCategory: getFilesByCategoryFromApi,
  getRecentFiles: getRecentFilesFromApi,
  getAllTags: getAllTagsFromApi,
  getFilesByTag: getFilesByTagFromApi,
  getBpmRange: getBpmRangeFromApi,
  getAllKeys: getAllKeysFromApi,
  pickFile: pickFileFromApi,
  pickDirectory: pickDirectoryFromApi,
  pickMultipleFiles: pickMultipleFilesFromApi,
  getSystemInfo: getSystemInfoFromApi,
  checkDatabaseConnection: checkDatabaseConnectionFromApi,
  pingDatabase: pingDatabaseFromApi,
  getCategoryStats: getCategoryStatsFromApi,
  getManufacturerStats: getManufacturerStatsFromApi,
  getKeySignatureStats: getKeySignatureStatsFromApi,
  getRecentlyAddedCount: getRecentlyAddedCountFromApi,
  getDuplicateCount: getDuplicateCountFromApi,
  getDatabaseSize: getDatabaseSizeFromApi,
  checkDatabaseHealth: checkDatabaseHealthFromApi,
  startProcessing: startProcessingFromApi,
  getProcessingProgress: getProcessingProgressFromApi,
  pauseProcessing: pauseProcessingFromApi,
  resumeProcessing: resumeProcessingFromApi,
  cancelProcessing: cancelProcessingFromApi,
};

// ============================================================================
// Default Export
// ============================================================================

export default api;
