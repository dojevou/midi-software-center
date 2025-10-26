// ARCHETYPE: MANAGER (State Management Only - NO I/O)
// LOCATION: pipeline/src/lib/stores/processing.ts
// PURPOSE: Manage import/processing status and progress
// PATTERN: Pure state management - delegates I/O to $lib/api/

import { writable } from 'svelte/store';

export type ProcessingPhase =
  | 'idle'
  | 'scanning'
  | 'decompressing'
  | 'analyzing'
  | 'splitting'
  | 'renaming'
  | 'indexing'
  | 'completed'
  | 'paused'
  | 'cancelled'
  | 'error';

export interface ProcessingStats {
  filesScanned: number;
  filesDecompressed: number;
  filesAnalyzed: number;
  filesSplit: number;
  filesRenamed: number;
  filesIndexed: number;
  duplicatesSkipped: number;
  errorsEncountered: number;
}

export interface ProcessingProgress {
  phase: ProcessingPhase;
  currentFile: string | null;
  currentFileIndex: number;
  totalFiles: number;
  percentage: number;
  speed: number; // files per second
  estimatedTimeRemaining: number; // seconds
  stats: ProcessingStats;
}

export interface ProcessingState {
  isProcessing: boolean;
  isPaused: boolean;
  progress: ProcessingProgress;
  jobId: string | null;
  error: string | null;
  startTime: Date | null;
  endTime: Date | null;
}

const initialProgress: ProcessingProgress = {
  phase: 'idle',
  currentFile: null,
  currentFileIndex: 0,
  totalFiles: 0,
  percentage: 0,
  speed: 0,
  estimatedTimeRemaining: 0,
  stats: {
    filesScanned: 0,
    filesDecompressed: 0,
    filesAnalyzed: 0,
    filesSplit: 0,
    filesRenamed: 0,
    filesIndexed: 0,
    duplicatesSkipped: 0,
    errorsEncountered: 0,
  },
};

const initialState: ProcessingState = {
  isProcessing: false,
  isPaused: false,
  progress: initialProgress,
  jobId: null,
  error: null,
  startTime: null,
  endTime: null,
};

function createProcessingStore() {
  const { subscribe, set, update } = writable<ProcessingState>(initialState);

  return {
    subscribe,

    /**
     * Set processing state
     */
    setProcessing: (isProcessing: boolean) => {
      update(state => ({ ...state, isProcessing }));
    },

    /**
     * Set paused state
     */
    setPaused: (isPaused: boolean) => {
      update(state => ({ ...state, isPaused }));
    },

    /**
     * Set job ID
     */
    setJobId: (jobId: string | null) => {
      update(state => ({ ...state, jobId }));
    },

    /**
     * Set error
     */
    setError: (error: string | null) => {
      update(state => ({ ...state, error }));
    },

    /**
     * Set start time
     */
    setStartTime: (startTime: Date | null) => {
      update(state => ({ ...state, startTime }));
    },

    /**
     * Set end time
     */
    setEndTime: (endTime: Date | null) => {
      update(state => ({ ...state, endTime }));
    },

    /**
     * Update progress
     */
    setProgress: (progress: ProcessingProgress) => {
      update(state => ({ ...state, progress }));
    },

    /**
     * Update progress phase only
     */
    setPhase: (phase: ProcessingPhase) => {
      update(state => ({
        ...state,
        progress: { ...state.progress, phase },
      }));
    },

    /**
     * Start processing - set initial state
     */
    start: (jobId: string) => {
      update(state => ({
        ...state,
        isProcessing: true,
        isPaused: false,
        jobId,
        error: null,
        startTime: new Date(),
        endTime: null,
        progress: initialProgress,
      }));
    },

    /**
     * Pause processing
     */
    pause: () => {
      update(state => ({
        ...state,
        isPaused: true,
        progress: {
          ...state.progress,
          phase: 'paused',
        },
      }));
    },

    /**
     * Resume processing
     */
    resume: () => {
      update(state => ({
        ...state,
        isPaused: false,
      }));
    },

    /**
     * Complete processing
     */
    complete: () => {
      update(state => ({
        ...state,
        isProcessing: false,
        isPaused: false,
        endTime: new Date(),
        progress: {
          ...state.progress,
          phase: 'completed',
          percentage: 100,
        },
      }));
    },

    /**
     * Cancel processing
     */
    cancel: () => {
      update(state => ({
        ...state,
        isProcessing: false,
        isPaused: false,
        endTime: new Date(),
        progress: {
          ...state.progress,
          phase: 'cancelled',
        },
      }));
    },

    /**
     * Fail processing
     */
    fail: (error: string) => {
      update(state => ({
        ...state,
        isProcessing: false,
        isPaused: false,
        error,
        endTime: new Date(),
        progress: {
          ...state.progress,
          phase: 'error',
        },
      }));
    },

    /**
     * Clear any error messages
     */
    clearError: () => {
      update(state => ({ ...state, error: null }));
    },

    /**
     * Reset to initial state
     */
    reset: () => {
      set(initialState);
    },

    /**
     * Get processing duration in seconds
     */
    getDuration: (): number => {
      let duration = 0;
      update(state => {
        if (state.startTime) {
          const endTime = state.endTime || new Date();
          duration = Math.floor((endTime.getTime() - state.startTime.getTime()) / 1000);
        }
        return state;
      });
      return duration;
    },

    /**
     * Format processing duration as string
     */
    getFormattedDuration: (): string => {
      let formatted = '0s';
      update(state => {
        if (state.startTime) {
          const endTime = state.endTime || new Date();
          const seconds = Math.floor((endTime.getTime() - state.startTime.getTime()) / 1000);

          const hours = Math.floor(seconds / 3600);
          const minutes = Math.floor((seconds % 3600) / 60);
          const secs = seconds % 60;

          if (hours > 0) {
            formatted = `${hours}h ${minutes}m ${secs}s`;
          } else if (minutes > 0) {
            formatted = `${minutes}m ${secs}s`;
          } else {
            formatted = `${secs}s`;
          }
        }
        return state;
      });
      return formatted;
    },
  };
}

export const processingStore = createProcessingStore();
