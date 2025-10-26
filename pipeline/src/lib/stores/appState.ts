// ARCHETYPE: MANAGER (State Management Only - NO I/O)
// LOCATION: pipeline/src/lib/stores/appState.ts
// PURPOSE: Manage application state, current view, connection status, and settings
// PATTERN: Pure state management - delegates I/O to $lib/api/

import { writable } from 'svelte/store';

export type ViewType = 'search' | 'import' | 'sequencer';

export interface AppSettings {
  darkMode: boolean;
  autoConnect: boolean;
  defaultMidiDevice: string | null;
  theme: string;
}

export interface AppState {
  currentView: ViewType;
  isConnected: boolean;
  connectionError: string | null;
  settings: AppSettings;
  isInitializing: boolean;
  lastError: string | null;
}

const initialState: AppState = {
  currentView: 'search',
  isConnected: false,
  connectionError: null,
  settings: {
    darkMode: true,
    autoConnect: false,
    defaultMidiDevice: null,
    theme: 'default',
  },
  isInitializing: false,
  lastError: null,
};

function createAppStateStore() {
  const { subscribe, set, update } = writable<AppState>(initialState);

  return {
    subscribe,

    /**
     * Set the current view
     */
    setView: (view: ViewType) => {
      update(state => ({ ...state, currentView: view }));
    },

    /**
     * Set connection status
     */
    setConnected: (isConnected: boolean) => {
      update(state => ({
        ...state,
        isConnected,
        connectionError: isConnected ? null : state.connectionError,
      }));
    },

    /**
     * Set connection error
     */
    setConnectionError: (error: string | null) => {
      update(state => ({
        ...state,
        isConnected: error ? false : state.isConnected,
        connectionError: error,
        lastError: error,
      }));
    },

    /**
     * Set initializing state
     */
    setInitializing: (isInitializing: boolean) => {
      update(state => ({ ...state, isInitializing }));
    },

    /**
     * Update app settings
     */
    updateSettings: (newSettings: Partial<AppSettings>) => {
      update(state => ({
        ...state,
        settings: { ...state.settings, ...newSettings },
      }));

      // Persist settings to localStorage
      try {
        const currentState = { ...initialState };
        update(state => {
          Object.assign(currentState, state);
          return state;
        });

        if (typeof window !== 'undefined') {
          localStorage.setItem('appSettings', JSON.stringify(currentState.settings));
        }
      } catch (error) {
        console.error('Failed to save settings:', error);
      }
    },

    /**
     * Load settings from localStorage
     */
    loadSettings: () => {
      try {
        if (typeof window !== 'undefined') {
          const saved = localStorage.getItem('appSettings');
          if (saved) {
            const settings = JSON.parse(saved) as AppSettings;
            update(state => ({ ...state, settings }));
          }
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    },

    /**
     * Set last error
     */
    setError: (error: string | null) => {
      update(state => ({ ...state, lastError: error }));
    },

    /**
     * Clear any error messages
     */
    clearError: () => {
      update(state => ({ ...state, lastError: null, connectionError: null }));
    },

    /**
     * Reset to initial state
     */
    reset: () => {
      set(initialState);
    },
  };
}

export const appStateStore = createAppStateStore();
