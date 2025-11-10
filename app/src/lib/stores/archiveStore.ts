import { writable, type Writable } from 'svelte/store';
import type { ArchiveProgress, ArchiveError } from '../types';

export interface ArchiveState {
  isExtracting: boolean;
  progress: number;
  currentArchive: string;
  extracted: number;
  totalFiles: number;
  errors: string[];
  extractedPaths: string[];
}

const initialState: ArchiveState = {
  isExtracting: false,
  progress: 0,
  currentArchive: '',
  extracted: 0,
  totalFiles: 0,
  errors: [],
  extractedPaths: []
};

const { subscribe, set, update }: Writable<ArchiveState> = writable(initialState);

const archiveActions = {
  startExtraction: (archivePath: string) => {
    update((state: ArchiveState) => ({
      ...state,
      isExtracting: true,
      progress: 0,
      currentArchive: archivePath,
      extracted: 0,
      totalFiles: 0,
      errors: [],
      extractedPaths: []
    }));
  },

  updateProgress: (progress: ArchiveProgress) => {
    update((state: ArchiveState) => ({
      ...state,
      progress: (progress.current / progress.total) * 100,
      extracted: progress.current,
      totalFiles: progress.total,
      currentArchive: progress.current_archive
    }));
  },

  addError: (error: ArchiveError) => {
    update((state: ArchiveState) => ({
      ...state,
      errors: [...state.errors, `${error.archivePath}: ${error.error}`]
    }));
  },

  setComplete: () => {
    update((state: ArchiveState) => ({
      ...state,
      isExtracting: false
    }));
  },

  clearState: () => {
    set(initialState);
  }
};

export const archiveStore = { subscribe, ...archiveActions };
export { archiveActions };