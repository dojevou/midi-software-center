import { derived, get, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export interface FavoriteItem {
  id: number;
  fileId: number;
  filename: string;
  filepath: string;
  addedAt: string;
  notes?: string;
  rating?: number;
  bpm?: number;
  key?: string;
  duration?: number;
  tags?: string[];
}

export interface FavoriteFolder {
  id: number;
  name: string;
  description?: string;
  color?: string;
  itemCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface FavoritesState {
  items: FavoriteItem[];
  folders: FavoriteFolder[];
  selectedFolder: number | null;
  selectedItems: Set<number>;
  searchQuery: string;
  sortBy: 'name' | 'date' | 'rating' | 'bpm';
  sortDesc: boolean;
  isLoading: boolean;
  error: string | null;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: FavoritesState = {
  items: [],
  folders: [],
  selectedFolder: null,
  selectedItems: new Set(),
  searchQuery: '',
  sortBy: 'date',
  sortDesc: true,
  isLoading: false,
  error: null,
};

const { subscribe, set, update } = writable<FavoritesState>(initialState);

export const favoritesStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const filteredItems = derived(favoritesStore, ($store) => {
  let items = $store.items;

  // Filter by search query
  if ($store.searchQuery) {
    const query = $store.searchQuery.toLowerCase();
    items = items.filter(
      (item) =>
        item.filename.toLowerCase().includes(query) ||
        item.tags?.some((tag) => tag.toLowerCase().includes(query))
    );
  }

  // Sort items
  items = [...items].sort((a, b) => {
    let comparison = 0;
    switch ($store.sortBy) {
      case 'name':
        comparison = a.filename.localeCompare(b.filename);
        break;
      case 'date':
        comparison = new Date(a.addedAt).getTime() - new Date(b.addedAt).getTime();
        break;
      case 'rating':
        comparison = (a.rating || 0) - (b.rating || 0);
        break;
      case 'bpm':
        comparison = (a.bpm || 0) - (b.bpm || 0);
        break;
    }
    return $store.sortDesc ? -comparison : comparison;
  });

  return items;
});

export const itemCount = derived(favoritesStore, ($store) => $store.items.length);
export const hasSelection = derived(favoritesStore, ($store) => $store.selectedItems.size > 0);
export const selectionCount = derived(favoritesStore, ($store) => $store.selectedItems.size);

// ============================================================================
// ACTIONS
// ============================================================================

export const favoritesActions = {
  /**
   * Load all favorites
   */
  async loadFavorites(): Promise<FavoriteItem[]> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const items = (await safeInvoke<FavoriteItem[]>(Commands.GET_FAVORITES)) || [];
      update((state) => ({
        ...state,
        items,
        isLoading: false,
      }));
      return items;
    } catch (error) {
      console.error('Failed to load favorites:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Load favorites for a specific folder
   */
  async loadFolderFavorites(folderId: number): Promise<FavoriteItem[]> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null, selectedFolder: folderId }));
      const items =
        (await safeInvoke<FavoriteItem[]>(Commands.GET_FOLDER_FAVORITES, { folderId })) || [];
      update((state) => ({
        ...state,
        items,
        isLoading: false,
      }));
      return items;
    } catch (error) {
      console.error('Failed to load folder favorites:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Load all folders
   */
  async loadFolders(): Promise<FavoriteFolder[]> {
    try {
      const folders = (await safeInvoke<FavoriteFolder[]>(Commands.GET_FAVORITE_FOLDERS)) || [];
      update((state) => ({ ...state, folders }));
      return folders;
    } catch (error) {
      console.error('Failed to load folders:', error);
      return [];
    }
  },

  /**
   * Add a file to favorites
   */
  async addToFavorites(fileId: number, folderId?: number): Promise<FavoriteItem | null> {
    try {
      const item = await safeInvoke<FavoriteItem>(Commands.ADD_TO_FAVORITES, {
        fileId,
        folderId: folderId || null,
      });
      if (item) {
        update((state) => ({
          ...state,
          items: [...state.items, item],
        }));
      }
      return item;
    } catch (error) {
      console.error('Failed to add to favorites:', error);
      return null;
    }
  },

  /**
   * Remove from favorites
   */
  async removeFromFavorites(favoriteId: number): Promise<void> {
    try {
      await safeInvoke(Commands.REMOVE_FROM_FAVORITES, { favoriteId });
      update((state) => ({
        ...state,
        items: state.items.filter((item) => item.id !== favoriteId),
        selectedItems: (() => {
          const newSelection = new Set(state.selectedItems);
          newSelection.delete(favoriteId);
          return newSelection;
        })(),
      }));
    } catch (error) {
      console.error('Failed to remove from favorites:', error);
    }
  },

  /**
   * Update favorite item (notes, rating)
   */
  async updateFavorite(
    favoriteId: number,
    updates: Partial<Pick<FavoriteItem, 'notes' | 'rating'>>
  ): Promise<void> {
    try {
      await safeInvoke(Commands.UPDATE_FAVORITE, { favoriteId, ...updates });
      update((state) => ({
        ...state,
        items: state.items.map((item) => (item.id === favoriteId ? { ...item, ...updates } : item)),
      }));
    } catch (error) {
      console.error('Failed to update favorite:', error);
    }
  },

  /**
   * Create a new folder
   */
  async createFolder(
    name: string,
    description?: string,
    color?: string
  ): Promise<FavoriteFolder | null> {
    try {
      const folder = await safeInvoke<FavoriteFolder>(Commands.CREATE_FAVORITE_FOLDER, {
        name,
        description,
        color,
      });
      if (folder) {
        update((state) => ({
          ...state,
          folders: [...state.folders, folder],
        }));
      }
      return folder;
    } catch (error) {
      console.error('Failed to create folder:', error);
      return null;
    }
  },

  /**
   * Delete a folder
   */
  async deleteFolder(folderId: number): Promise<void> {
    try {
      await safeInvoke(Commands.DELETE_FAVORITE_FOLDER, { folderId });
      update((state) => ({
        ...state,
        folders: state.folders.filter((folder) => folder.id !== folderId),
        selectedFolder: state.selectedFolder === folderId ? null : state.selectedFolder,
      }));
    } catch (error) {
      console.error('Failed to delete folder:', error);
    }
  },

  /**
   * Move items to a folder
   */
  async moveToFolder(favoriteIds: number[], folderId: number | null): Promise<void> {
    try {
      await safeInvoke(Commands.MOVE_FAVORITES_TO_FOLDER, { favoriteIds, folderId });
      // Reload to update counts
      await this.loadFolders();
    } catch (error) {
      console.error('Failed to move items to folder:', error);
    }
  },

  // ============================================================================
  // LOCAL STATE MANAGEMENT
  // ============================================================================

  selectItem(itemId: number, addToSelection: boolean = false): void {
    update((state) => {
      const newSelection = addToSelection
        ? new Set([...state.selectedItems, itemId])
        : new Set([itemId]);
      return { ...state, selectedItems: newSelection };
    });
  },

  toggleItemSelection(itemId: number): void {
    update((state) => {
      const newSelection = new Set(state.selectedItems);
      if (newSelection.has(itemId)) {
        newSelection.delete(itemId);
      } else {
        newSelection.add(itemId);
      }
      return { ...state, selectedItems: newSelection };
    });
  },

  selectAll(): void {
    update((state) => ({
      ...state,
      selectedItems: new Set(state.items.map((item) => item.id)),
    }));
  },

  clearSelection(): void {
    update((state) => ({ ...state, selectedItems: new Set() }));
  },

  setSearchQuery(query: string): void {
    update((state) => ({ ...state, searchQuery: query }));
  },

  setSortBy(sortBy: FavoritesState['sortBy']): void {
    update((state) => ({ ...state, sortBy }));
  },

  toggleSortOrder(): void {
    update((state) => ({ ...state, sortDesc: !state.sortDesc }));
  },

  selectFolder(folderId: number | null): void {
    update((state) => ({ ...state, selectedFolder: folderId }));
  },

  reset(): void {
    set(initialState);
  },
};
