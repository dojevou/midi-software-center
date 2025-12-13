import { writable, type Writable } from 'svelte/store';
import type { AnalysisProgress, AnalysisSummary } from '../types';

export interface AnalysisState {
  isRunning: boolean;
  progress: number;
  currentFile: string;
  processed: number;
  totalFiles: number;
  results: Array<{
    fileId: number;
    bpm: number;
    key: string;
    timeSignature: string;
    instruments: string[];
    duration: number;
  }>;
  errors: string[];
  lastComplete: AnalysisSummary | null;
}

const initialState: AnalysisState = {
  isRunning: false,
  progress: 0,
  currentFile: '',
  processed: 0,
  totalFiles: 0,
  results: [],
  errors: [],
  lastComplete: null,
};

const { subscribe, set, update }: Writable<AnalysisState> = writable(initialState);

const analysisActions = {
  startAnalysis: () => {
    update((state: AnalysisState) => ({
      ...state,
      isRunning: true,
      progress: 0,
      currentFile: '',
      processed: 0,
      totalFiles: 0,
      results: [],
      errors: [],
      lastComplete: null,
    }));
  },

  updateProgress: (progress: AnalysisProgress) => {
    update((state: AnalysisState) => ({
      ...state,
      progress: (progress.current / progress.total) * 100,
      currentFile: progress.current_file,
      processed: progress.current,
      totalFiles: progress.total,
    }));
  },

  setComplete: (result: AnalysisSummary) => {
    update((state: AnalysisState) => ({
      ...state,
      isRunning: false,
      lastComplete: result,
    }));
  },

  clearResults: () => {
    update((state: AnalysisState) => ({
      ...state,
      results: [],
      lastComplete: null,
    }));
  },

  addError: (error: string) => {
    update((state: AnalysisState) => ({
      ...state,
      errors: [...state.errors, error],
    }));
  },
};

export const analysisStore = { subscribe, ...analysisActions };
export { analysisActions };
