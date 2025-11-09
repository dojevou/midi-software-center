/**
 * Tag Suggestions Store
 *
 * ARCHETYPE: GROWN-UP SCRIPT (State management + I/O)
 * Purpose: Manage tag suggestions state, Tauri IPC, and side effects
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  TagSuggestion,
  TagCategory,
  TagSuggestionFilters,
  TagSuggestionAction,
  TagSuggestionState,
  EnhancedTag
} from '../types/tagSuggestions';
import {
  filterTagsBySearch,
  filterTagsByCategories,
  filterTagsByConfidence,
  sortTags,
  groupTagsByCategory
} from '../utils/tagUtils';

// =============================================================================
// STATE
// =============================================================================

const initialState: TagSuggestionState = {
  suggestions: [],
  categories: [],
  filters: {
    minConfidence: 0.60,
    onlyPending: true
  },
  selectedSuggestionIds: [],
  loading: false,
  error: null
};

export const tagSuggestionsState = writable<TagSuggestionState>(initialState);

// =============================================================================
// DERIVED STORES
// =============================================================================

/**
 * Filtered suggestions based on current filters
 */
export const filteredSuggestions = derived(tagSuggestionsState, ($state) => {
  let filtered = $state.suggestions;

  // Filter by pending status
  if ($state.filters.onlyPending) {
    filtered = filtered.filter((s) => s.isAccepted === undefined || s.isAccepted === null);
  }

  // Filter by confidence range
  filtered = filtered.filter((s) => {
    if ($state.filters.minConfidence !== undefined && s.confidence < $state.filters.minConfidence) {
      return false;
    }
    if ($state.filters.maxConfidence !== undefined && s.confidence > $state.filters.maxConfidence) {
      return false;
    }
    return true;
  });

  // Filter by categories
  if ($state.filters.categories && $state.filters.categories.length > 0) {
    filtered = filtered.filter(
      (s) => s.suggestedTag.category && $state.filters.categories!.includes(s.suggestedTag.category)
    );
  }

  // Filter by sources
  if ($state.filters.sources && $state.filters.sources.length > 0) {
    filtered = filtered.filter((s) => $state.filters.sources!.includes(s.source));
  }

  // Filter by search query
  if ($state.filters.search && $state.filters.search.length >= 2) {
    const query = $state.filters.search.toLowerCase();
    filtered = filtered.filter((s) =>
      s.suggestedTag.name.toLowerCase().includes(query)
    );
  }

  return filtered;
});

/**
 * Unique tags from suggestions
 */
export const uniqueTags = derived(tagSuggestionsState, ($state) => {
  const tagMap = new Map<number, EnhancedTag>();

  for (const suggestion of $state.suggestions) {
    if (!tagMap.has(suggestion.suggestedTagId)) {
      tagMap.set(suggestion.suggestedTagId, suggestion.suggestedTag);
    }
  }

  return Array.from(tagMap.values());
});

/**
 * Tags grouped by category
 */
export const tagsByCategory = derived(uniqueTags, ($tags) => {
  return groupTagsByCategory($tags);
});

/**
 * Selected suggestions (for batch operations)
 */
export const selectedSuggestions = derived(tagSuggestionsState, ($state) => {
  return $state.suggestions.filter((s) => $state.selectedSuggestionIds.includes(s.id));
});

/**
 * Whether any suggestions are selected
 */
export const hasSelection = derived(
  tagSuggestionsState,
  ($state) => $state.selectedSuggestionIds.length > 0
);

/**
 * Suggestion statistics
 */
export const suggestionStats = derived(filteredSuggestions, ($suggestions) => {
  const total = $suggestions.length;
  const highConfidence = $suggestions.filter((s) => s.confidence >= 0.85).length;
  const mediumConfidence = $suggestions.filter((s) => s.confidence >= 0.70 && s.confidence < 0.85).length;
  const lowConfidence = $suggestions.filter((s) => s.confidence < 0.70).length;

  const bySources = new Map<string, number>();
  for (const suggestion of $suggestions) {
    bySources.set(suggestion.source, (bySources.get(suggestion.source) || 0) + 1);
  }

  return {
    total,
    highConfidence,
    mediumConfidence,
    lowConfidence,
    bySources
  };
});

// =============================================================================
// ACTIONS - FETCHING DATA
// =============================================================================

/**
 * Fetch tag categories from backend
 */
export async function fetchTagCategories(): Promise<void> {
  try {
    const categories = await invoke<TagCategory[]>('get_tag_categories');
    tagSuggestionsState.update((state) => ({
      ...state,
      categories
    }));
  } catch (err) {
    console.error('Error fetching tag categories:', err);
    tagSuggestionsState.update((state) => ({
      ...state,
      error: err instanceof Error ? err.message : 'Failed to fetch tag categories'
    }));
  }
}

/**
 * Fetch tag suggestions for a specific file
 */
export async function fetchTagSuggestions(fileId: number): Promise<void> {
  tagSuggestionsState.update((state) => ({
    ...state,
    loading: true,
    error: null
  }));

  try {
    const suggestions = await invoke<TagSuggestion[]>('get_tag_suggestions', { fileId });

    tagSuggestionsState.update((state) => ({
      ...state,
      suggestions,
      loading: false
    }));
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to fetch tag suggestions';
    tagSuggestionsState.update((state) => ({
      ...state,
      loading: false,
      error: errorMsg
    }));
    throw new Error(errorMsg);
  }
}

/**
 * Generate tag suggestions for a file using auto-tagging
 */
export async function generateTagSuggestions(fileId: number): Promise<void> {
  tagSuggestionsState.update((state) => ({
    ...state,
    loading: true,
    error: null
  }));

  try {
    await invoke('generate_tag_suggestions', { fileId });

    // Refresh suggestions after generation
    await fetchTagSuggestions(fileId);
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to generate tag suggestions';
    tagSuggestionsState.update((state) => ({
      ...state,
      loading: false,
      error: errorMsg
    }));
    throw new Error(errorMsg);
  }
}

/**
 * Fetch pending tag suggestions (across all files)
 */
export async function fetchPendingSuggestions(limit: number = 100): Promise<void> {
  tagSuggestionsState.update((state) => ({
    ...state,
    loading: true,
    error: null
  }));

  try {
    const suggestions = await invoke<TagSuggestion[]>('get_pending_tag_suggestions', { limit });

    tagSuggestionsState.update((state) => ({
      ...state,
      suggestions,
      loading: false
    }));
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to fetch pending suggestions';
    tagSuggestionsState.update((state) => ({
      ...state,
      loading: false,
      error: errorMsg
    }));
  }
}

// =============================================================================
// ACTIONS - MODIFYING SUGGESTIONS
// =============================================================================

/**
 * Accept a tag suggestion
 */
export async function acceptSuggestion(suggestionId: number): Promise<void> {
  try {
    await invoke('accept_tag_suggestion', { suggestionId });

    // Update local state
    tagSuggestionsState.update((state) => ({
      ...state,
      suggestions: state.suggestions.map((s) =>
        s.id === suggestionId
          ? { ...s, isAccepted: true, acceptedAt: new Date().toISOString() }
          : s
      )
    }));
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to accept suggestion';
    throw new Error(errorMsg);
  }
}

/**
 * Reject a tag suggestion
 */
export async function rejectSuggestion(suggestionId: number): Promise<void> {
  try {
    await invoke('reject_tag_suggestion', { suggestionId });

    // Update local state
    tagSuggestionsState.update((state) => ({
      ...state,
      suggestions: state.suggestions.map((s) =>
        s.id === suggestionId ? { ...s, isAccepted: false, acceptedAt: new Date().toISOString() } : s
      )
    }));
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to reject suggestion';
    throw new Error(errorMsg);
  }
}

/**
 * Batch accept/reject suggestions
 */
export async function batchProcessSuggestions(
  suggestionIds: number[],
  action: TagSuggestionAction
): Promise<void> {
  if (suggestionIds.length === 0) return;

  try {
    await invoke('batch_process_tag_suggestions', { suggestionIds, action });

    // Update local state
    tagSuggestionsState.update((state) => ({
      ...state,
      suggestions: state.suggestions.map((s) => {
        if (suggestionIds.includes(s.id)) {
          if (action === 'accept') {
            return { ...s, isAccepted: true, acceptedAt: new Date().toISOString() };
          } else if (action === 'reject') {
            return { ...s, isAccepted: false, acceptedAt: new Date().toISOString() };
          }
        }
        return s;
      }),
      selectedSuggestionIds: [] // Clear selection after batch operation
    }));
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to process suggestions';
    throw new Error(errorMsg);
  }
}

// =============================================================================
// ACTIONS - SELECTION
// =============================================================================

/**
 * Toggle suggestion selection
 */
export function toggleSuggestionSelection(suggestionId: number): void {
  tagSuggestionsState.update((state) => {
    const selected = state.selectedSuggestionIds.includes(suggestionId);
    return {
      ...state,
      selectedSuggestionIds: selected
        ? state.selectedSuggestionIds.filter((id) => id !== suggestionId)
        : [...state.selectedSuggestionIds, suggestionId]
    };
  });
}

/**
 * Select all filtered suggestions
 */
export function selectAllSuggestions(): void {
  const filtered = get(filteredSuggestions);
  tagSuggestionsState.update((state) => ({
    ...state,
    selectedSuggestionIds: filtered.map((s) => s.id)
  }));
}

/**
 * Clear selection
 */
export function clearSelection(): void {
  tagSuggestionsState.update((state) => ({
    ...state,
    selectedSuggestionIds: []
  }));
}

/**
 * Select suggestions by category
 */
export function selectByCategory(category: string): void {
  const filtered = get(filteredSuggestions);
  const categoryIds = filtered
    .filter((s) => s.suggestedTag.category === category)
    .map((s) => s.id);

  tagSuggestionsState.update((state) => ({
    ...state,
    selectedSuggestionIds: Array.from(new Set([...state.selectedSuggestionIds, ...categoryIds]))
  }));
}

/**
 * Select suggestions by confidence level
 */
export function selectByConfidence(minConfidence: number): void {
  const filtered = get(filteredSuggestions);
  const highConfidenceIds = filtered
    .filter((s) => s.confidence >= minConfidence)
    .map((s) => s.id);

  tagSuggestionsState.update((state) => ({
    ...state,
    selectedSuggestionIds: Array.from(new Set([...state.selectedSuggestionIds, ...highConfidenceIds]))
  }));
}

// =============================================================================
// ACTIONS - FILTERING
// =============================================================================

/**
 * Update filters
 */
export function updateFilters(filters: Partial<TagSuggestionFilters>): void {
  tagSuggestionsState.update((state) => ({
    ...state,
    filters: { ...state.filters, ...filters }
  }));
}

/**
 * Reset filters to defaults
 */
export function resetFilters(): void {
  tagSuggestionsState.update((state) => ({
    ...state,
    filters: {
      minConfidence: 0.60,
      onlyPending: true
    }
  }));
}

/**
 * Set search query
 */
export function setSearchQuery(query: string): void {
  updateFilters({ search: query });
}

/**
 * Toggle category filter
 */
export function toggleCategoryFilter(category: string): void {
  tagSuggestionsState.update((state) => {
    const categories = state.filters.categories || [];
    const updated = categories.includes(category)
      ? categories.filter((c) => c !== category)
      : [...categories, category];

    return {
      ...state,
      filters: {
        ...state.filters,
        categories: updated.length > 0 ? updated : undefined
      }
    };
  });
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the tag suggestions store
 */
export async function initializeTagSuggestionsStore(): Promise<void> {
  await fetchTagCategories();
}

/**
 * Reset the store to initial state
 */
export function resetTagSuggestionsStore(): void {
  tagSuggestionsState.set(initialState);
}
