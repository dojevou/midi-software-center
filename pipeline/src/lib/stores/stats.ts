// ARCHETYPE: MANAGER (State Management Only - NO I/O)
// LOCATION: pipeline/src/lib/stores/stats.ts
// PURPOSE: Manage file counts and database statistics
// PATTERN: Pure state management - delegates I/O to $lib/api/

import { writable } from 'svelte/store';

export interface FileStats {
  totalFiles: number;
  byCategory: Record<string, number>;
  byManufacturer: Record<string, number>;
  byKeySignature: Record<string, number>;
  recentlyAdded: number;
  duplicates: number;
}

export interface DatabaseStats {
  databaseSize: string;
  lastUpdated: Date | null;
  indexHealth: 'good' | 'warning' | 'error';
}

export interface StatsState {
  fileStats: FileStats;
  databaseStats: DatabaseStats;
  isLoading: boolean;
  lastRefresh: Date | null;
  error: string | null;
}

const initialState: StatsState = {
  fileStats: {
    totalFiles: 0,
    byCategory: {},
    byManufacturer: {},
    byKeySignature: {},
    recentlyAdded: 0,
    duplicates: 0,
  },
  databaseStats: {
    databaseSize: '0 MB',
    lastUpdated: null,
    indexHealth: 'good',
  },
  isLoading: false,
  lastRefresh: null,
  error: null,
};

function createStatsStore() {
  const { subscribe, set, update } = writable<StatsState>(initialState);

  return {
    subscribe,

    /**
     * Set loading state
     */
    setLoading: (isLoading: boolean) => {
      update(state => ({ ...state, isLoading }));
    },

    /**
     * Set error state
     */
    setError: (error: string | null) => {
      update(state => ({ ...state, error, isLoading: false }));
    },

    /**
     * Update file stats
     */
    setFileStats: (fileStats: Partial<FileStats>) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, ...fileStats },
        lastRefresh: new Date(),
      }));
    },

    /**
     * Update total file count
     */
    setTotalFiles: (totalFiles: number) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, totalFiles },
        lastRefresh: new Date(),
      }));
    },

    /**
     * Update category stats
     */
    setCategoryStats: (byCategory: Record<string, number>) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, byCategory },
      }));
    },

    /**
     * Update manufacturer stats
     */
    setManufacturerStats: (byManufacturer: Record<string, number>) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, byManufacturer },
      }));
    },

    /**
     * Update key signature stats
     */
    setKeySignatureStats: (byKeySignature: Record<string, number>) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, byKeySignature },
      }));
    },

    /**
     * Update recently added count
     */
    setRecentlyAdded: (recentlyAdded: number) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, recentlyAdded },
      }));
    },

    /**
     * Update duplicates count
     */
    setDuplicates: (duplicates: number) => {
      update(state => ({
        ...state,
        fileStats: { ...state.fileStats, duplicates },
      }));
    },

    /**
     * Update database stats
     */
    setDatabaseStats: (databaseStats: Partial<DatabaseStats>) => {
      update(state => ({
        ...state,
        databaseStats: { ...state.databaseStats, ...databaseStats },
      }));
    },

    /**
     * Update database size
     */
    setDatabaseSize: (databaseSize: string) => {
      update(state => ({
        ...state,
        databaseStats: {
          ...state.databaseStats,
          databaseSize,
          lastUpdated: new Date(),
        },
      }));
    },

    /**
     * Update database health (FIX: proper type literal)
     */
    setDatabaseHealth: (indexHealth: 'good' | 'warning' | 'error') => {
      update(state => ({
        ...state,
        databaseStats: {
          ...state.databaseStats,
          indexHealth,
        },
      }));
    },

    /**
     * Set complete stats state (used after fetching all stats)
     */
    setCompleteStats: (stats: {
      totalFiles: number;
      byCategory: Record<string, number>;
      byManufacturer: Record<string, number>;
      byKeySignature: Record<string, number>;
      recentlyAdded: number;
      duplicates: number;
      databaseSize: string;
    }) => {
      update(state => ({
        ...state,
        fileStats: {
          totalFiles: stats.totalFiles,
          byCategory: stats.byCategory,
          byManufacturer: stats.byManufacturer,
          byKeySignature: stats.byKeySignature,
          recentlyAdded: stats.recentlyAdded,
          duplicates: stats.duplicates,
        },
        databaseStats: {
          ...state.databaseStats,
          databaseSize: stats.databaseSize,
          lastUpdated: new Date(),
        },
        isLoading: false,
        lastRefresh: new Date(),
        error: null,
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
  };
}

export const statsStore = createStatsStore();
