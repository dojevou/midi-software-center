/**
 * Connection/App State API
 * ARCHETYPE: MANAGER (I/O operations)
 * PURPOSE: Handle database connection and app initialization
 */

import { invokeCommand } from './client';

/**
 * Test database connection
 */
export async function testDatabaseConnection(): Promise<boolean> {
  try {
    await invokeCommand('test_db_connection');
    return true;
  } catch {
    return false;
  }
}

/**
 * Get file count (used for connection verification)
 */
export async function getFileCount(): Promise<number> {
  return invokeCommand<number>('get_file_count');
}

/**
 * Ping database to verify connection
 */
export async function pingDatabase(): Promise<void> {
  return invokeCommand<void>('test_db_connection');
}
