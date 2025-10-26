// importStore.ts - Manager for file import I/O operations
// Archetype: Grown-up Script (manages state AND does I/O)

import { writable, type Writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { importSingleFile, importDirectory } from '$lib/api/files';
import type { ImportProgress } from '$lib/types';

// Types
export interface ImportState {
  importing: boolean;
  progress: ImportProgress | null;
  error: string | null;
}

const initialState: ImportState = {
  importing: false,
  progress: null,
  error: null,
};

// Create the store
function createImportStore() {
  const { subscribe, set, update }: Writable<ImportState> = writable<ImportState>(initialState);
  let unlistenProgress: UnlistenFn | null = null;

  return {
    subscribe,

    /**
     * Pick a single MIDI file using file dialog (I/O operation)
     */
    pickFile: async (): Promise<string | null> => {
      try {
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
      } catch (error) {
        console.error('Failed to pick file:', error);
        update(state => ({
          ...state,
          error: error instanceof Error ? error.message : 'Failed to pick file'
        }));
        return null;
      }
    },

    /**
     * Pick a directory using directory dialog (I/O operation)
     */
    pickDirectory: async (): Promise<string | null> => {
      try {
        const result = await open({
          directory: true,
          multiple: false,
        });

        if (Array.isArray(result)) {
          return result[0] || null;
        }

        return result;
      } catch (error) {
        console.error('Failed to pick directory:', error);
        update(state => ({
          ...state,
          error: error instanceof Error ? error.message : 'Failed to pick directory'
        }));
        return null;
      }
    },

    /**
     * Import a single MIDI file (I/O operation)
     */
    importFile: async (filePath: string): Promise<number | null> => {
      update(state => ({ ...state, importing: true, error: null }));

      try {
        // Start listening for progress events
        await startProgressListener(update);

        const fileId = await importSingleFile(filePath);

        update(state => ({ ...state, importing: false }));

        // Clean up listener
        if (unlistenProgress) {
          unlistenProgress();
          unlistenProgress = null;
        }

        return fileId;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to import file';
        console.error('Import failed:', error);

        update(state => ({
          ...state,
          importing: false,
          error: errorMessage
        }));

        // Clean up listener
        if (unlistenProgress) {
          unlistenProgress();
          unlistenProgress = null;
        }

        return null;
      }
    },

    /**
     * Import a directory of MIDI files (I/O operation)
     */
    importDirectory: async (directoryPath: string, recursive: boolean = true): Promise<boolean> => {
      update(state => ({ ...state, importing: true, error: null }));

      try {
        // Start listening for progress events
        await startProgressListener(update);

        await importDirectory(directoryPath, recursive);

        update(state => ({ ...state, importing: false }));

        // Clean up listener
        if (unlistenProgress) {
          unlistenProgress();
          unlistenProgress = null;
        }

        return true;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'Failed to import directory';
        console.error('Import directory failed:', error);

        update(state => ({
          ...state,
          importing: false,
          error: errorMessage
        }));

        // Clean up listener
        if (unlistenProgress) {
          unlistenProgress();
          unlistenProgress = null;
        }

        return false;
      }
    },

    /**
     * Pick and import a single file (convenience method)
     */
    pickAndImport: async (): Promise<number | null> => {
      const filePath = await importStore.pickFile();
      if (!filePath) return null;

      return await importStore.importFile(filePath);
    },

    /**
     * Pick and import a directory (convenience method)
     */
    pickAndImportDirectory: async (recursive: boolean = true): Promise<boolean> => {
      const dirPath = await importStore.pickDirectory();
      if (!dirPath) return false;

      return await importStore.importDirectory(dirPath, recursive);
    },

    /**
     * Clear error message
     */
    clearError: () => {
      update(state => ({ ...state, error: null }));
    },

    /**
     * Reset store to initial state
     */
    reset: () => {
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
      set(initialState);
    },
  };

  /**
   * Internal function to start listening for import progress events
   */
  async function startProgressListener(
    update: (updater: (state: ImportState) => ImportState) => void
  ): Promise<void> {
    // Clean up existing listener
    if (unlistenProgress) {
      unlistenProgress();
    }

    // Start new listener
    unlistenProgress = await listen<ImportProgress>('import-progress', (event) => {
      update(state => ({
        ...state,
        progress: event.payload
      }));
    });
  }
}

// Export singleton instance
export const importStore = createImportStore();
