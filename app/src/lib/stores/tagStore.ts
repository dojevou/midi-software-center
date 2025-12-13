import { derived, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

console.log('🏷️ tagStore.ts: Store module loading');

// ============================================================================
// TYPES
// ============================================================================

export interface Tag {
  id: number;
  name: string;
  category: string;
  count: number;
}

export interface TagStats {
  total_tags: number;
  total_file_tags: number;
  top_tags: Tag[];
  categories: string[];
}

export interface TagState {
  tags: Tag[];
  popularTags: Tag[];
  categories: string[];
  selectedTags: string[];
  searchQuery: string;
  isLoading: boolean;
  error: string | null;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: TagState = {
  tags: [],
  popularTags: [],
  categories: [],
  selectedTags: [],
  searchQuery: '',
  isLoading: false,
  error: null,
};

const { subscribe, set, update } = writable<TagState>(initialState);

console.log('🏷️ tagStore.ts: Store created with initial state');

export const tagStore = { subscribe };

// Derived stores
export const selectedTagCount = derived(tagStore, ($store) => $store.selectedTags.length);
export const hasSelectedTags = derived(tagStore, ($store) => $store.selectedTags.length > 0);
export const filteredTags = derived(tagStore, ($store) => {
  if (!$store.searchQuery) {
    return $store.tags;
  }
  const query = $store.searchQuery.toLowerCase();
  return $store.tags.filter((tag) => tag.name.toLowerCase().includes(query));
});

// ============================================================================
// ACTIONS
// ============================================================================

export const tagActions = {
  /**
   * Get all tags from database
   */
  getAllTags: async (): Promise<Tag[]> => {
    console.log('🏷️ tagActions.getAllTags called');
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const tags = (await safeInvoke<Tag[]>(Commands.GET_ALL_TAGS)) || [];
      update((state) => ({
        ...state,
        tags,
        isLoading: false,
      }));
      return tags;
    } catch (error) {
      console.error('❌ tagActions.getAllTags failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Get popular tags (most used)
   */
  getPopularTags: async (limit: number = 50): Promise<Tag[]> => {
    console.log('🏷️ tagActions.getPopularTags called with limit:', limit);
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const popularTags = (await safeInvoke<Tag[]>(Commands.GET_POPULAR_TAGS, { limit })) || [];
      update((state) => ({
        ...state,
        popularTags,
        isLoading: false,
      }));
      return popularTags;
    } catch (error) {
      console.error('❌ tagActions.getPopularTags failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Search tags by query
   */
  searchTags: async (query: string): Promise<Tag[]> => {
    console.log('🏷️ tagActions.searchTags called with query:', query);
    try {
      update((state) => ({ ...state, searchQuery: query, isLoading: true, error: null }));
      const tags = (await safeInvoke<Tag[]>(Commands.SEARCH_TAGS, { query })) || [];
      update((state) => ({
        ...state,
        tags,
        isLoading: false,
      }));
      return tags;
    } catch (error) {
      console.error('❌ tagActions.searchTags failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Get all tag categories
   */
  getTagCategories: async (): Promise<string[]> => {
    console.log('🏷️ tagActions.getTagCategories called');
    try {
      const categories = (await safeInvoke<string[]>(Commands.GET_TAG_CATEGORIES)) || [];
      update((state) => ({
        ...state,
        categories,
      }));
      return categories;
    } catch (error) {
      console.error('❌ tagActions.getTagCategories failed:', error);
      return [];
    }
  },

  /**
   * Get tags by category
   */
  getTagsByCategory: async (category: string): Promise<Tag[]> => {
    console.log('🏷️ tagActions.getTagsByCategory called with category:', category);
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const tags = (await safeInvoke<Tag[]>(Commands.GET_TAGS_BY_CATEGORY, { category })) || [];
      update((state) => ({
        ...state,
        tags,
        isLoading: false,
      }));
      return tags;
    } catch (error) {
      console.error('❌ tagActions.getTagsByCategory failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Get tags for a specific file
   */
  getFileTags: async (fileId: number): Promise<string[]> => {
    console.log('🏷️ tagActions.getFileTags called for file:', fileId);
    try {
      const tags = (await safeInvoke<string[]>(Commands.GET_FILE_TAGS, { file_id: fileId })) || [];
      return tags;
    } catch (error) {
      console.error('❌ tagActions.getFileTags failed:', error);
      return [];
    }
  },

  /**
   * Add tags to a file
   */
  addTagsToFile: async (fileId: number, tags: string[]): Promise<void> => {
    console.log('🏷️ tagActions.addTagsToFile called for file:', fileId, 'tags:', tags);
    try {
      await safeInvoke<void>(Commands.ADD_TAGS_TO_FILE, { file_id: fileId, tags });
      console.log('✅ Tags added successfully');
    } catch (error) {
      console.error('❌ tagActions.addTagsToFile failed:', error);
      throw error;
    }
  },

  /**
   * Remove a tag from a file
   */
  removeTagFromFile: async (fileId: number, tag: string): Promise<void> => {
    console.log('🏷️ tagActions.removeTagFromFile called for file:', fileId, 'tag:', tag);
    try {
      await safeInvoke<void>(Commands.REMOVE_TAG_FROM_FILE, { file_id: fileId, tag });
      console.log('✅ Tag removed successfully');
    } catch (error) {
      console.error('❌ tagActions.removeTagFromFile failed:', error);
      throw error;
    }
  },

  /**
   * Update all tags for a file (replace)
   */
  updateFileTags: async (fileId: number, tags: string[]): Promise<void> => {
    console.log('🏷️ tagActions.updateFileTags called for file:', fileId, 'tags:', tags);
    try {
      await safeInvoke<void>(Commands.UPDATE_FILE_TAGS, { file_id: fileId, tags });
      console.log('✅ File tags updated successfully');
    } catch (error) {
      console.error('❌ tagActions.updateFileTags failed:', error);
      throw error;
    }
  },

  /**
   * Get files that have specific tags
   */
  getFilesByTags: async (tags: string[]): Promise<number[]> => {
    console.log('🏷️ tagActions.getFilesByTags called with tags:', tags);
    try {
      const fileIds = (await safeInvoke<number[]>(Commands.GET_FILES_BY_TAGS, { tags })) || [];
      return fileIds;
    } catch (error) {
      console.error('❌ tagActions.getFilesByTags failed:', error);
      return [];
    }
  },

  /**
   * Get tag statistics
   */
  getTagStats: async (): Promise<TagStats | null> => {
    console.log('🏷️ tagActions.getTagStats called');
    try {
      const stats = await safeInvoke<TagStats>(Commands.GET_TAG_STATS);
      return stats;
    } catch (error) {
      console.error('❌ tagActions.getTagStats failed:', error);
      return null;
    }
  },

  // ============================================================================
  // LOCAL STATE MANAGEMENT
  // ============================================================================

  /**
   * Select a tag (add to selection)
   */
  selectTag: (tag: string): void => {
    update((state) => ({
      ...state,
      selectedTags: state.selectedTags.includes(tag)
        ? state.selectedTags
        : [...state.selectedTags, tag],
    }));
  },

  /**
   * Deselect a tag (remove from selection)
   */
  deselectTag: (tag: string): void => {
    update((state) => ({
      ...state,
      selectedTags: state.selectedTags.filter((t) => t !== tag),
    }));
  },

  /**
   * Toggle tag selection
   */
  toggleTag: (tag: string): void => {
    update((state) => ({
      ...state,
      selectedTags: state.selectedTags.includes(tag)
        ? state.selectedTags.filter((t) => t !== tag)
        : [...state.selectedTags, tag],
    }));
  },

  /**
   * Clear all selected tags
   */
  clearSelection: (): void => {
    update((state) => ({
      ...state,
      selectedTags: [],
    }));
  },

  /**
   * Set search query
   */
  setSearchQuery: (query: string): void => {
    update((state) => ({
      ...state,
      searchQuery: query,
    }));
  },

  /**
   * Clear any errors
   */
  clearError: (): void => {
    update((state) => ({
      ...state,
      error: null,
    }));
  },

  /**
   * Reset store to initial state
   */
  reset: (): void => {
    set(initialState);
  },
};

console.log('🏷️ tagStore.ts: Store and actions exported');
