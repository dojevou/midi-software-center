import { writable, type Writable } from 'svelte/store';
import type { ArchiveError, ArchiveProgress } from '../types';

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
  extractedPaths: [],
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
      extractedPaths: [],
    }));
  },

  updateProgress: (progress: ArchiveProgress) => {
    update((state: ArchiveState) => ({
      ...state,
      progress: progress.total > 0 ? (progress.current / progress.total) * 100 : 0,
      extracted: progress.current,
      totalFiles: progress.total,
      currentArchive: progress.current_archive || state.currentArchive,
    }));
  },

  addError: (error: ArchiveError) => {
    const path = error.file_path || error.archivePath || 'unknown';
    const message = error.error_message || error.error || 'unknown error';
    update((state: ArchiveState) => ({
      ...state,
      errors: [...state.errors, `${path}: ${message}`],
    }));
  },

  setComplete: () => {
    update((state: ArchiveState) => ({
      ...state,
      isExtracting: false,
    }));
  },

  clearState: () => {
    set(initialState);
  },
};

export const archiveStore = { subscribe, ...archiveActions };
export { archiveActions };
