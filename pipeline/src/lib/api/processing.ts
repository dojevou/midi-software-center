/**
 * Processing/Import API
 * ARCHETYPE: MANAGER (I/O operations)
 * PURPOSE: Manage file processing and import operations
 */

import { invokeCommand } from './client';

export type ProcessingPhase =
  | 'idle'
  | 'scanning'
  | 'decompressing'
  | 'analyzing'
  | 'splitting'
  | 'renaming'
  | 'indexing'
  | 'completed'
  | 'paused'
  | 'cancelled'
  | 'error';

export interface ProcessingStats {
  filesScanned: number;
  filesDecompressed: number;
  filesAnalyzed: number;
  filesSplit: number;
  filesRenamed: number;
  filesIndexed: number;
  duplicatesSkipped: number;
  errorsEncountered: number;
}

export interface ProcessingProgress {
  phase: ProcessingPhase;
  currentFile: string | null;
  currentFileIndex: number;
  totalFiles: number;
  percentage: number;
  speed: number; // files per second
  estimatedTimeRemaining: number; // seconds
  stats: ProcessingStats;
}

export interface ProcessingConfig {
  source_path: string;
  split_multi_track?: boolean;
  rename_files?: boolean;
  detect_duplicates?: boolean;
  max_depth?: number;
}

/**
 * Start a new processing job
 */
export async function startProcessing(config: ProcessingConfig): Promise<string> {
  return invokeCommand<string>('start_processing', { config });
}

/**
 * Get current processing progress
 */
export async function getProcessingProgress(): Promise<ProcessingProgress> {
  return invokeCommand<ProcessingProgress>('get_processing_progress');
}

/**
 * Pause the current processing job
 */
export async function pauseProcessing(): Promise<void> {
  return invokeCommand<void>('pause_processing');
}

/**
 * Resume a paused processing job
 */
export async function resumeProcessing(): Promise<void> {
  return invokeCommand<void>('resume_processing');
}

/**
 * Cancel the current processing job
 */
export async function cancelProcessing(): Promise<void> {
  return invokeCommand<void>('cancel_processing');
}
