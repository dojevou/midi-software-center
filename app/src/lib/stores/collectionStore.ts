/**
 * Collection Store
 *
 * Manages user-created collections state
 * Follows the pattern from databaseStore.ts
 */

import { derived, writable, get } from 'svelte/store';
import { collectionsApi } from '../api/collections';
import type {
  Collection,
  CollectionWithFiles,
  VIP3BrowserFilters,
  BulkOperationResult,
} from '../types';

console.log('🔧 collectionStore.ts: Store module loading');

/**
 * Collection store state
 */
export interface CollectionState {
  // All collections
  collections: Collection[];

  // Currently selected collection (with files)
  selectedCollection: CollectionWithFiles | null;

  // Loading states
  isLoading: boolean;
  isLoadingFiles: boolean;

  // Error state
  error: string | null;

  // UI state
  editingCollection: Collection | null;
  showCreateDialog: boolean;
  showSmartCreateDialog: boolean;
}

// Initial state
const initialState: CollectionState = {
  collections: [],
  selectedCollection: null,
  isLoading: false,
  isLoadingFiles: false,
  error: null,
  editingCollection: null,
  showCreateDialog: false,
  showSmartCreateDialog: false,
};

// Create writable store
const { subscribe, set, update } = writable<CollectionState>(initialState);

console.log('🔧 collectionStore.ts: Store created with initial state');

// Read-only store export
export const collectionStore = { subscribe };

// Derived stores
export const collections = derived(collectionStore, ($store) => $store.collections);
export const selectedCollection = derived(collectionStore, ($store) => $store.selectedCollection);
export const collectionIsLoading = derived(collectionStore, ($store) => $store.isLoading);
export const collectionError = derived(collectionStore, ($store) => $store.error);

// Smart collections derived
export const smartCollections = derived(collectionStore, ($store) =>
  $store.collections.filter((c) => c.is_smart)
);
export const regularCollections = derived(collectionStore, ($store) =>
  $store.collections.filter((c) => !c.is_smart)
);

// Actions
export const collectionActions = {
  /**
   * Load all collections
   */
  loadAll: async (): Promise<void> => {
    console.log('🔧 collectionActions.loadAll: Loading all collections');

    update((state) => ({
      ...state,
      isLoading: true,
      error: null,
    }));

    try {
      const collections = await collectionsApi.getAll();

      update((state) => ({
        ...state,
        collections,
        isLoading: false,
      }));

      console.log('🔧 collectionActions.loadAll: Loaded', collections.length, 'collections');
    } catch (error) {
      console.error('❌ collectionActions.loadAll: Failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  /**
   * Select a collection and load its files
   */
  select: async (collectionId: number): Promise<void> => {
    console.log('🔧 collectionActions.select:', collectionId);

    update((state) => ({
      ...state,
      isLoadingFiles: true,
      error: null,
    }));

    try {
      const collection = await collectionsApi.getById(collectionId);

      update((state) => ({
        ...state,
        selectedCollection: collection,
        isLoadingFiles: false,
      }));
    } catch (error) {
      console.error('❌ collectionActions.select: Failed:', error);
      update((state) => ({
        ...state,
        isLoadingFiles: false,
        error: String(error),
      }));
    }
  },

  /**
   * Clear selection
   */
  clearSelection: (): void => {
    update((state) => ({
      ...state,
      selectedCollection: null,
    }));
  },

  /**
   * Create a new collection
   */
  create: async (name: string, description?: string, color?: string): Promise<Collection | null> => {
    console.log('🔧 collectionActions.create:', name);

    update((state) => ({
      ...state,
      isLoading: true,
      error: null,
    }));

    try {
      const collection = await collectionsApi.create({ name, description, color });

      update((state) => ({
        ...state,
        collections: [...state.collections, collection],
        isLoading: false,
        showCreateDialog: false,
      }));

      console.log('🔧 collectionActions.create: Created collection', collection.id);
      return collection;
    } catch (error) {
      console.error('❌ collectionActions.create: Failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return null;
    }
  },

  /**
   * Create a smart collection
   */
  createSmart: async (
    name: string,
    filters: VIP3BrowserFilters,
    description?: string,
    color?: string
  ): Promise<Collection | null> => {
    console.log('🔧 collectionActions.createSmart:', name);

    update((state) => ({
      ...state,
      isLoading: true,
      error: null,
    }));

    try {
      const collection = await collectionsApi.createSmart({
        name,
        description,
        color,
        filters,
      });

      update((state) => ({
        ...state,
        collections: [...state.collections, collection],
        isLoading: false,
        showSmartCreateDialog: false,
      }));

      console.log('🔧 collectionActions.createSmart: Created smart collection', collection.id);
      return collection;
    } catch (error) {
      console.error('❌ collectionActions.createSmart: Failed:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return null;
    }
  },

  /**
   * Update a collection
   */
  update: async (
    id: number,
    updates: { name?: string; description?: string; color?: string }
  ): Promise<void> => {
    console.log('🔧 collectionActions.update:', id, updates);

    try {
      const updated = await collectionsApi.update({ id, ...updates });

      update((state) => ({
        ...state,
        collections: state.collections.map((c) => (c.id === id ? updated : c)),
        selectedCollection:
          state.selectedCollection?.id === id
            ? { ...state.selectedCollection, ...updated }
            : state.selectedCollection,
        editingCollection: null,
      }));
    } catch (error) {
      console.error('❌ collectionActions.update: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
    }
  },

  /**
   * Delete a collection
   */
  delete: async (id: number): Promise<void> => {
    console.log('🔧 collectionActions.delete:', id);

    try {
      await collectionsApi.delete(id);

      update((state) => ({
        ...state,
        collections: state.collections.filter((c) => c.id !== id),
        selectedCollection: state.selectedCollection?.id === id ? null : state.selectedCollection,
      }));
    } catch (error) {
      console.error('❌ collectionActions.delete: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
    }
  },

  /**
   * Add files to the selected collection
   */
  addFiles: async (fileIds: number[]): Promise<BulkOperationResult | null> => {
    const currentState = get(collectionStore);
    if (!currentState.selectedCollection) {
      console.error('❌ collectionActions.addFiles: No collection selected');
      return null;
    }

    console.log('🔧 collectionActions.addFiles:', fileIds.length, 'files to collection', currentState.selectedCollection.id);

    try {
      const result = await collectionsApi.addFilesBulk(currentState.selectedCollection.id, fileIds);

      // Refresh the selected collection
      await collectionActions.select(currentState.selectedCollection.id);

      // Update file count in collections list
      update((state) => ({
        ...state,
        collections: state.collections.map((c) =>
          c.id === currentState.selectedCollection!.id
            ? { ...c, file_count: c.file_count + result.success_count }
            : c
        ),
      }));

      return result;
    } catch (error) {
      console.error('❌ collectionActions.addFiles: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
      return null;
    }
  },

  /**
   * Remove files from the selected collection
   */
  removeFiles: async (fileIds: number[]): Promise<BulkOperationResult | null> => {
    const currentState = get(collectionStore);
    if (!currentState.selectedCollection) {
      console.error('❌ collectionActions.removeFiles: No collection selected');
      return null;
    }

    console.log('🔧 collectionActions.removeFiles:', fileIds.length, 'files from collection', currentState.selectedCollection.id);

    try {
      const result = await collectionsApi.removeFilesBulk(
        currentState.selectedCollection.id,
        fileIds
      );

      // Refresh the selected collection
      await collectionActions.select(currentState.selectedCollection.id);

      // Update file count in collections list
      update((state) => ({
        ...state,
        collections: state.collections.map((c) =>
          c.id === currentState.selectedCollection!.id
            ? { ...c, file_count: Math.max(0, c.file_count - result.success_count) }
            : c
        ),
      }));

      return result;
    } catch (error) {
      console.error('❌ collectionActions.removeFiles: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
      return null;
    }
  },

  /**
   * Refresh a smart collection
   */
  refreshSmart: async (id: number): Promise<void> => {
    console.log('🔧 collectionActions.refreshSmart:', id);

    try {
      const updated = await collectionsApi.refreshSmart(id);

      update((state) => ({
        ...state,
        collections: state.collections.map((c) => (c.id === id ? updated : c)),
      }));

      // If this is the selected collection, refresh it
      const currentState = get(collectionStore);
      if (currentState.selectedCollection?.id === id) {
        await collectionActions.select(id);
      }
    } catch (error) {
      console.error('❌ collectionActions.refreshSmart: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
    }
  },

  /**
   * Duplicate a collection
   */
  duplicate: async (id: number, newName?: string): Promise<Collection | null> => {
    console.log('🔧 collectionActions.duplicate:', id);

    try {
      const collection = await collectionsApi.duplicate(id, newName);

      update((state) => ({
        ...state,
        collections: [...state.collections, collection],
      }));

      return collection;
    } catch (error) {
      console.error('❌ collectionActions.duplicate: Failed:', error);
      update((state) => ({
        ...state,
        error: String(error),
      }));
      return null;
    }
  },

  /**
   * Show create dialog
   */
  showCreate: (): void => {
    update((state) => ({
      ...state,
      showCreateDialog: true,
    }));
  },

  /**
   * Hide create dialog
   */
  hideCreate: (): void => {
    update((state) => ({
      ...state,
      showCreateDialog: false,
    }));
  },

  /**
   * Show smart create dialog
   */
  showSmartCreate: (): void => {
    update((state) => ({
      ...state,
      showSmartCreateDialog: true,
    }));
  },

  /**
   * Hide smart create dialog
   */
  hideSmartCreate: (): void => {
    update((state) => ({
      ...state,
      showSmartCreateDialog: false,
    }));
  },

  /**
   * Start editing a collection
   */
  startEditing: (collection: Collection): void => {
    update((state) => ({
      ...state,
      editingCollection: collection,
    }));
  },

  /**
   * Stop editing
   */
  stopEditing: (): void => {
    update((state) => ({
      ...state,
      editingCollection: null,
    }));
  },

  /**
   * Clear error
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

console.log('🔧 collectionStore.ts: Store module loaded');
