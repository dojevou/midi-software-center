/**
 * Event listeners for backend events
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ImportProgress } from '$lib/types';

/**
 * Listens for import progress events
 */
export async function listenToImportProgress(
  callback: (progress: ImportProgress) => void
): Promise<UnlistenFn> {
  return listen<ImportProgress>('import-progress', (event) => {
    callback(event.payload);
  });
}

/**
 * Listens for file added events
 */
export async function listenToFileAdded(
  callback: (fileId: number) => void
): Promise<UnlistenFn> {
  return listen<number>('file-added', (event) => {
    callback(event.payload);
  });
}

/**
 * Listens for file updated events
 */
export async function listenToFileUpdated(
  callback: (fileId: number) => void
): Promise<UnlistenFn> {
  return listen<number>('file-updated', (event) => {
    callback(event.payload);
  });
}

/**
 * Listens for file deleted events
 */
export async function listenToFileDeleted(
  callback: (fileId: number) => void
): Promise<UnlistenFn> {
  return listen<number>('file-deleted', (event) => {
    callback(event.payload);
  });
}
