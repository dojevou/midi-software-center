/**
 * Statistics API
 * ARCHETYPE: MANAGER (I/O operations)
 * PURPOSE: Fetch database statistics and metrics
 */

import { invokeCommand } from './client';

/**
 * Get total file count
 */
export async function getFileCount(): Promise<number> {
  return invokeCommand<number>('get_file_count');
}

/**
 * Get file count breakdown by category
 */
export async function getCategoryStats(): Promise<Record<string, number>> {
  return invokeCommand<Record<string, number>>('get_category_stats');
}

/**
 * Get file count breakdown by manufacturer
 */
export async function getManufacturerStats(): Promise<Record<string, number>> {
  return invokeCommand<Record<string, number>>('get_manufacturer_stats');
}

/**
 * Get file count breakdown by key signature
 */
export async function getKeySignatureStats(): Promise<Record<string, number>> {
  return invokeCommand<Record<string, number>>('get_key_signature_stats');
}

/**
 * Get count of recently added files (last 7 days)
 */
export async function getRecentlyAddedCount(): Promise<number> {
  return invokeCommand<number>('get_recently_added_count');
}

/**
 * Get count of duplicate files
 */
export async function getDuplicateCount(): Promise<number> {
  return invokeCommand<number>('get_duplicate_count');
}

/**
 * Get database size as formatted string
 */
export async function getDatabaseSize(): Promise<string> {
  return invokeCommand<string>('get_database_size');
}

/**
 * Check database health status
 */
export async function checkDatabaseHealth(): Promise<'good' | 'warning' | 'error'> {
  return invokeCommand<'good' | 'warning' | 'error'>('check_database_health');
}
