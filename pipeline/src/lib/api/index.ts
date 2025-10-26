/**
 * API barrel export
 * Import all API functions from here
 */

// Client
export {
  ApiError,
  isApiError,
  getErrorMessage,
  invokeCommand,
} from './client';

// Files
export {
  importSingleFile,
  importDirectory,
  getFile,
  getFileCount,
  listFiles,
  updateFileTags,
  deleteFile,
  getFilesByCategory,
  getRecentFiles,
} from './files';

// Search
export {
  searchFiles,
  getAllTags,
  getFilesByTag,
  getBpmRange,
  getAllKeys,
} from './search';

// Events
export {
  listenToImportProgress,
  listenToFileAdded,
  listenToFileUpdated,
  listenToFileDeleted,
} from './events';

// System
export {
  pickFile,
  pickDirectory,
  pickMultipleFiles,
  getSystemInfo,
  checkDatabaseConnection,
} from './system';

// Connection
export {
  testDatabaseConnection,
  pingDatabase,
} from './connection';

// Stats
export {
  getCategoryStats,
  getManufacturerStats,
  getKeySignatureStats,
  getRecentlyAddedCount,
  getDuplicateCount,
  getDatabaseSize,
  checkDatabaseHealth,
} from './stats';

// Processing
export {
  startProcessing,
  getProcessingProgress,
  pauseProcessing,
  resumeProcessing,
  cancelProcessing,
} from './processing';

// Re-export types
export type { ProcessingPhase, ProcessingStats, ProcessingProgress, ProcessingConfig } from './processing';

// Types
export type * from './types';
