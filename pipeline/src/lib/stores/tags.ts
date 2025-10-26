/**
 * Tags Store - State management for tag operations
 *
 * This store manages:
 * - Popular tags (for tag cloud)
 * - Tag search/autocomplete
 * - Selected tags for filtering
 * - Tag-based file filtering
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Tag } from '$lib/types';

// =============================================================================
// STATE
// =============================================================================

/**
 * Popular tags with usage counts
 */
export const popularTags = writable<Tag[]>([]);

/**
 * Tags selected for filtering
 */
export const selectedTags = writable<string[]>([]);

/**
 * Tag match mode: 'all' (AND) or 'any' (OR)
 */
export const tagMatchMode = writable<'all' | 'any'>('any');

/**
 * Loading state
 */
export const loading = writable<boolean>(false);

/**
 * Error state
 */
export const error = writable<string | null>(null);

/**
 * Search results for autocomplete
 */
export const searchResults = writable<Tag[]>([]);

// =============================================================================
// DERIVED STORES
// =============================================================================

/**
 * Tag categories (genre, instrument, brand, etc.)
 */
export const tagCategories = derived(popularTags, ($tags) => {
  const categories = new Set<string>();
  $tags.forEach((tag) => {
    if (tag.category) {
      categories.add(tag.category);
    }
  });
  return Array.from(categories).sort();
});

/**
 * Tags grouped by category
 */
export const tagsByCategory = derived(popularTags, ($tags) => {
  const grouped = new Map<string, Tag[]>();

  $tags.forEach((tag) => {
    const category = tag.category || 'other';
    if (!grouped.has(category)) {
      grouped.set(category, []);
    }
    grouped.get(category)!.push(tag);
  });

  // Sort tags within each category by usage count
  grouped.forEach((tags) => {
    tags.sort((a, b) => b.usageCount - a.usageCount);
  });

  return grouped;
});

/**
 * Whether any tags are selected
 */
export const hasSelectedTags = derived(selectedTags, ($tags) => $tags.length > 0);

// =============================================================================
// ACTIONS
// =============================================================================

/**
 * Fetch popular tags from backend
 * @param limit - Maximum number of tags to fetch (default: 50)
 */
export async function fetchPopularTags(limit: number = 50): Promise<void> {
  loading.set(true);
  error.set(null);

  try {
    const tags = await invoke<Tag[]>('get_popular_tags', { limit });
    popularTags.set(tags);
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to fetch popular tags';
    error.set(errorMsg);
    console.error('Error fetching popular tags:', err);
  } finally {
    loading.set(false);
  }
}

/**
 * Search tags by name prefix (for autocomplete)
 * @param query - Search query
 * @param limit - Maximum number of results (default: 10)
 */
export async function searchTags(query: string, limit: number = 10): Promise<Tag[]> {
  if (!query || query.length < 2) {
    searchResults.set([]);
    return [];
  }

  try {
    const results = await invoke<Tag[]>('search_tags', { query, limit });
    searchResults.set(results);
    return results;
  } catch (err) {
    console.error('Error searching tags:', err);
    searchResults.set([]);
    return [];
  }
}

/**
 * Get all unique tag categories
 */
export async function fetchTagCategories(): Promise<string[]> {
  try {
    return await invoke<string[]>('get_tag_categories');
  } catch (err) {
    console.error('Error fetching tag categories:', err);
    return [];
  }
}

/**
 * Get tags by category
 * @param category - Category name (e.g., 'genre', 'instrument')
 */
export async function fetchTagsByCategory(category: string): Promise<Tag[]> {
  try {
    return await invoke<Tag[]>('get_tags_by_category', { category });
  } catch (err) {
    console.error(`Error fetching tags for category ${category}:`, err);
    return [];
  }
}

/**
 * Get file IDs by tags
 * @param tags - Array of tag names to filter by
 * @param matchAll - If true, file must have ALL tags (AND). If false, at least one tag (OR)
 */
export async function getFilesByTags(tags: string[], matchAll: boolean = false): Promise<number[]> {
  if (tags.length === 0) {
    return [];
  }

  try {
    return await invoke<number[]>('get_files_by_tags', { tagNames: tags, matchAll });
  } catch (err) {
    console.error('Error getting files by tags:', err);
    return [];
  }
}

/**
 * Get tags for a specific file
 * @param fileId - File ID
 */
export async function getFileTags(fileId: number): Promise<Tag[]> {
  try {
    return await invoke<Tag[]>('get_file_tags', { fileId });
  } catch (err) {
    console.error(`Error fetching tags for file ${fileId}:`, err);
    return [];
  }
}

/**
 * Replace all tags for a file
 * @param fileId - File ID
 * @param tagNames - Array of tag names to set
 */
export async function replaceFileTags(fileId: number, tagNames: string[]): Promise<void> {
  try {
    await invoke('replace_file_tags', { fileId, tagNames });
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to update tags';
    throw new Error(errorMsg);
  }
}

/**
 * Add tags to a file (without removing existing tags)
 * @param fileId - File ID
 * @param tagNames - Array of tag names to add
 */
export async function addTagsToFile(fileId: number, tagNames: string[]): Promise<void> {
  try {
    await invoke('add_tags_to_file', { fileId, tagNames });
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to add tags';
    throw new Error(errorMsg);
  }
}

/**
 * Remove a specific tag from a file
 * @param fileId - File ID
 * @param tagId - Tag ID to remove
 */
export async function removeTagFromFile(fileId: number, tagId: number): Promise<void> {
  try {
    await invoke('remove_tag_from_file', { fileId, tagId });
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to remove tag';
    throw new Error(errorMsg);
  }
}

/**
 * Get usage statistics for a tag
 * @param tagId - Tag ID
 */
export async function getTagStats(tagId: number): Promise<number> {
  try {
    return await invoke<number>('get_tag_stats', { tagId });
  } catch (err) {
    console.error(`Error fetching stats for tag ${tagId}:`, err);
    return 0;
  }
}

// =============================================================================
// SELECTION MANAGEMENT
// =============================================================================

/**
 * Add a tag to the selected tags
 */
export function selectTag(tagName: string): void {
  selectedTags.update((tags) => {
    if (!tags.includes(tagName)) {
      return [...tags, tagName];
    }
    return tags;
  });
}

/**
 * Remove a tag from the selected tags
 */
export function deselectTag(tagName: string): void {
  selectedTags.update((tags) => tags.filter((t) => t !== tagName));
}

/**
 * Toggle a tag selection
 */
export function toggleTag(tagName: string): void {
  const current = get(selectedTags);
  if (current.includes(tagName)) {
    deselectTag(tagName);
  } else {
    selectTag(tagName);
  }
}

/**
 * Clear all selected tags
 */
export function clearSelectedTags(): void {
  selectedTags.set([]);
}

/**
 * Set tag match mode (AND/OR)
 */
export function setTagMatchMode(mode: 'all' | 'any'): void {
  tagMatchMode.set(mode);
}

/**
 * Toggle tag match mode between 'all' and 'any'
 */
export function toggleTagMatchMode(): void {
  tagMatchMode.update((mode) => (mode === 'all' ? 'any' : 'all'));
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/**
 * Get color class for a tag category
 */
export function getTagCategoryColor(category?: string): string {
  switch (category) {
    case 'genre':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
    case 'instrument':
      return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
    case 'brand':
      return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
    case 'bpm':
      return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
    case 'key':
      return 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200';
    default:
      return 'bg-slate-100 text-slate-800 dark:bg-slate-700 dark:text-slate-200';
  }
}

/**
 * Calculate font size based on tag usage count (for tag cloud)
 */
export function getTagFontSize(count: number, maxCount: number, minSize: number = 12, maxSize: number = 24): number {
  if (maxCount === 0) return minSize;

  // Logarithmic scale for better visual distribution
  const ratio = Math.log(count + 1) / Math.log(maxCount + 1);
  return Math.round(minSize + (maxSize - minSize) * ratio);
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the tags store
 * Call this when the app starts
 */
export function initializeTagsStore(): void {
  fetchPopularTags();
}
