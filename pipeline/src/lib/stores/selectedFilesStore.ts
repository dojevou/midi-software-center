import { writable } from 'svelte/store';
import { getFile, deleteFile } from '$lib/api/files';
import type { FileMetadata } from '$lib/types';

interface SelectedFilesState {
  selectedFiles: FileMetadata[];
  loading: boolean;
  error: string | null;
}

function createSelectedFilesStore() {
  const { subscribe, set, update } = writable<SelectedFilesState>({
    selectedFiles: [],
    loading: false,
    error: null
  });

  return {
    subscribe,

    // MANAGER handles BOTH state AND I/O
    addFile: (file: FileMetadata) => update(state => ({
      ...state,
      selectedFiles: [...state.selectedFiles, file],
      error: null
    })),

    removeFile: (fileId: number) => update(state => ({
      ...state,
      selectedFiles: state.selectedFiles.filter(f => f.id !== fileId)
    })),

    toggleFile: (file: FileMetadata) => update(state => {
      const exists = state.selectedFiles.some(f => f.id === file.id);
      if (exists) {
        return {
          ...state,
          selectedFiles: state.selectedFiles.filter(f => f.id !== file.id)
        };
      } else {
        return {
          ...state,
          selectedFiles: [...state.selectedFiles, file]
        };
      }
    }),

    // I/O operation: Load file details and add to selection
    loadAndAdd: async (fileId: number) => {
      update(state => ({ ...state, loading: true, error: null }));

      try {
        const file = await getFile(fileId);
        update(state => ({
          ...state,
          selectedFiles: [...state.selectedFiles, file],
          loading: false
        }));
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to load file';
        update(state => ({
          ...state,
          loading: false,
          error: errorMessage
        }));
      }
    },

    // I/O operation: Delete selected files
    deleteSelected: async () => {
      const filesToDelete = [...(await new Promise<FileMetadata[]>(resolve => {
        const unsubscribe = subscribe(state => {
          resolve(state.selectedFiles);
          unsubscribe();
        });
      }))];

      update(state => ({ ...state, loading: true, error: null }));

      try {
        for (const file of filesToDelete) {
          await deleteFile(file.id);
        }
        update(state => ({
          ...state,
          selectedFiles: [],
          loading: false
        }));
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to delete files';
        update(state => ({
          ...state,
          loading: false,
          error: errorMessage
        }));
      }
    },

    clearAll: () => set({ selectedFiles: [], loading: false, error: null })
  };
}

export const selectedFilesStore = createSelectedFilesStore();
