import { writable, type Writable } from 'svelte/store';
import type { ImportProgress, ImportSummary } from '../types';

export interface PipelineState {
  operation: 'import' | 'analysis' | null;
  isRunning: boolean;
  isPaused: boolean;
  progress: number;
  currentFile: string;
  processed: number;
  totalFiles: number;
  errors: string[];
  lastResult: ImportSummary | null;
}

const initialState: PipelineState = {
  operation: null,
  isRunning: false,
  isPaused: false,
  progress: 0,
  currentFile: '',
  processed: 0,
  totalFiles: 0,
  errors: [],
  lastResult: null
};

const { subscribe, set, update }: Writable<PipelineState> = writable(initialState);

const pipelineActions = {
  startOperation: (operation: 'import' | 'analysis') => {
    update((state: PipelineState) => ({
      ...state,
      operation,
      isRunning: true,
      isPaused: false,
      progress: 0,
      currentFile: '',
      processed: 0,
      totalFiles: 0,
      errors: [],
      lastResult: null
    }));
  },

  pauseOperation: () => {
    update((state: PipelineState) => ({ ...state, isPaused: true }));
  },

  stopOperation: () => {
    set(initialState);
  },

  updateProgress: (progress: ImportProgress) => {
    update((state: PipelineState) => ({
      ...state,
      progress: (progress.current / progress.total) * 100,
      currentFile: progress.current_file,
      processed: progress.current,
      totalFiles: progress.total
    }));
  },

  setComplete: (result: ImportSummary) => {
    update((state: PipelineState) => ({
      ...state,
      isRunning: false,
      isPaused: false,
      lastResult: result
    }));
  },

  clearErrors: () => {
    update((state: PipelineState) => ({ ...state, errors: [] }));
  }
};

export const pipelineStore = { subscribe, ...pipelineActions };
export { pipelineActions };