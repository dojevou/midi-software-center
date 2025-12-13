import { writable } from 'svelte/store';
import type { ImportProgress, ImportSummary } from '../types';

export interface PipelineState {
  operation: 'idle' | 'importing' | 'analyzing' | 'archiving';
  progress: ImportProgress;
  errors: string[];
}

// Store
const { subscribe, set, update } = writable<PipelineState>({
  operation: 'idle',
  progress: { current: 0, total: 0, rate: 0, eta: 0 },
  errors: [],
});

export const pipelineStore = { subscribe };

// Actions (separate export)
export const pipelineActions = {
  startOperation: async (operation: 'import' | 'analyze' | 'archive') => {
    update((state) => ({
      ...state,
      operation:
        operation === 'import' ? 'importing' : operation === 'analyze' ? 'analyzing' : 'archiving',
    }));
  },
  pauseOperation: () => {
    update((state) => ({ ...state, operation: 'idle' }));
  },
  stopOperation: () => {
    update((state) => ({ ...state, operation: 'idle' }));
  },
  updateProgress: (progress: ImportProgress) => {
    update((state) => ({ ...state, progress }));
  },
  setComplete: (result: ImportSummary, progress?: ImportProgress) => {
    update((state) => ({ ...state, operation: 'idle', errors: result.errors || [] }));
  },
  clearErrors: () => {
    update((state) => ({ ...state, errors: [] }));
  },
};
