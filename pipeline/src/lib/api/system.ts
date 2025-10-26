/**
 * System-level API functions
 */

import { invokeCommand } from './client';
import { open } from '@tauri-apps/plugin-dialog';

/**
 * Opens file picker dialog
 */
export async function pickFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [
      {
        name: 'MIDI Files',
        extensions: ['mid', 'midi'],
      },
    ],
  });

  if (Array.isArray(result)) {
    return result[0] || null;
  }

  return result;
}

/**
 * Opens directory picker dialog
 */
export async function pickDirectory(): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
  });

  if (Array.isArray(result)) {
    return result[0] || null;
  }

  return result;
}

/**
 * Opens multiple file picker dialog
 */
export async function pickMultipleFiles(): Promise<string[]> {
  const result = await open({
    multiple: true,
    filters: [
      {
        name: 'MIDI Files',
        extensions: ['mid', 'midi'],
      },
    ],
  });

  if (Array.isArray(result)) {
    return result;
  }

  return result ? [result] : [];
}

/**
 * Gets system info
 */
export async function getSystemInfo(): Promise<{
  version: string;
  platform: string;
}> {
  return invokeCommand('get_system_info');
}

/**
 * Checks if database is connected
 */
export async function checkDatabaseConnection(): Promise<boolean> {
  try {
    await invokeCommand('test_db_connection');
    return true;
  } catch {
    return false;
  }
}
