import { writable } from 'svelte/store';
import { checkDatabaseConnection } from '$lib/api/system';

type View = 'search' | 'import' | 'sequencer';

interface AppState {
  currentView: View;
  connectionStatus: boolean;
  checking: boolean;
  lastChecked: Date | null;
}

function createAppStateStore() {
  const { subscribe, update } = writable<AppState>({
    currentView: 'search',
    connectionStatus: false,
    checking: false,
    lastChecked: null
  });

  return {
    subscribe,

    // Pure state mutation
    setView: (view: View) => update(state => ({ ...state, currentView: view })),

    // MANAGER handles BOTH state AND I/O
    checkConnection: async () => {
      update(state => ({ ...state, checking: true }));

      try {
        const isConnected = await checkDatabaseConnection();
        update(state => ({
          ...state,
          connectionStatus: isConnected,
          checking: false,
          lastChecked: new Date()
        }));
      } catch (err) {
        update(state => ({
          ...state,
          connectionStatus: false,
          checking: false,
          lastChecked: new Date()
        }));
      }
    },

    // Start periodic connection checks
    startMonitoring: (intervalMs: number = 30000) => {
      const interval = setInterval(async () => {
        try {
          const isConnected = await checkDatabaseConnection();
          update(state => ({
            ...state,
            connectionStatus: isConnected,
            lastChecked: new Date()
          }));
        } catch {
          update(state => ({
            ...state,
            connectionStatus: false,
            lastChecked: new Date()
          }));
        }
      }, intervalMs);

      return () => clearInterval(interval);
    }
  };
}

export const appStateStore = createAppStateStore();
