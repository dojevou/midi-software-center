import { writable, derived, type Writable, type Readable } from 'svelte/store';

/**
 * Database connection status
 */
export const dbConnected: Writable<boolean> = writable(false);

/**
 * Total number of files in database
 */
export const totalFiles: Writable<number> = writable(0);

/**
 * Currently selected file IDs
 */
export const selectedFiles: Writable<Set<number>> = writable(new Set());

/**
 * View mode for results
 */
export type ViewMode = 'grid' | 'list';
export const viewMode: Writable<ViewMode> = writable('grid');

/**
 * Loading states
 */
export const isLoading: Writable<boolean> = writable(false);
export const loadingMessage: Writable<string> = writable('');

/**
 * Error state
 */
export const errorMessage: Writable<string | null> = writable(null);

/**
 * Helper: Check if any files are selected
 */
export const hasSelection: Readable<boolean> = derived(
  selectedFiles,
  $selectedFiles => $selectedFiles.size > 0
);

/**
 * Helper: Get selection count
 */
export const selectionCount: Readable<number> = derived(
  selectedFiles,
  $selectedFiles => $selectedFiles.size
);
