import { writable, derived } from 'svelte/store';
import { listFiles, getFileCount, updateFileTags } from '$lib/api';
import type { FileMetadata } from '$lib/api/types';

interface FilesState {
  items: FileMetadata[];
  total: number;
  loading: boolean;
  error: string | null;
  selectedId: number | null;
  viewMode: 'grid' | 'list';
  sortBy: 'name' | 'date' | 'bpm' | 'category';
  sortOrder: 'asc' | 'desc';
  updatingTags: boolean;
}

function createFilesStore() {
  const { subscribe, update } = writable<FilesState>({
    items: [],
    total: 0,
    loading: false,
    error: null,
    selectedId: null,
    viewMode: 'grid',
    sortBy: 'date',
    sortOrder: 'desc',
    updatingTags: false,
  });

  return {
    subscribe,
    loadFiles: async (limit = 50, offset = 0) => {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const [items, total] = await Promise.all([
          listFiles(limit, offset),
          getFileCount(),
        ]);
        update(s => ({ ...s, items, total, loading: false }));
      } catch (error) {
        update(s => ({ ...s, error: String(error), loading: false }));
      }
    },
    selectFile: (id: number | null) => update(s => ({ ...s, selectedId: id })),
    setViewMode: (mode: 'grid' | 'list') => update(s => ({ ...s, viewMode: mode })),
    setSorting: (by: string, order: 'asc' | 'desc') =>
      update(s => ({ ...s, sortBy: by as any, sortOrder: order })),

    // Tag management (I/O operation)
    updateTags: async (fileId: number, tags: string[]): Promise<boolean> => {
      update(s => ({ ...s, updatingTags: true, error: null }));
      try {
        await updateFileTags(fileId, tags);

        // Update the file in the local state
        update(s => ({
          ...s,
          items: s.items.map(item =>
            item.id === fileId ? { ...item, userTags: tags } : item
          ),
          updatingTags: false,
        }));

        return true;
      } catch (error) {
        update(s => ({
          ...s,
          error: error instanceof Error ? error.message : 'Failed to update tags',
          updatingTags: false,
        }));
        return false;
      }
    },
  };
}

export const filesStore = createFilesStore();
export const selectedFile = derived(
  filesStore,
  $files => $files.items.find(f => f.id === $files.selectedId)
);
