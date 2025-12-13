import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface LogEntry {
  timestamp: string;
  level: string;
  category: string;
  message: string;
  target: string;
  fields: Record<string, unknown>;
  span?: {
    name: string;
    id: string;
    parent_id?: string;
  };
}

export interface PerformanceMetrics {
  counters: Record<string, number>;
  timings: Record<
    string,
    {
      count: number;
      total_ms: number;
      min_ms: number;
      max_ms: number;
      avg_ms: number;
    }
  >;
  gauges: Record<string, number>;
}

export interface LogState {
  entries: LogEntry[];
  isStreaming: boolean;
  filter: {
    level: string | null;
    category: string | null;
    search: string;
  };
  metrics: PerformanceMetrics | null;
}

const initialState: LogState = {
  entries: [],
  isStreaming: false,
  filter: {
    level: null,
    category: null,
    search: '',
  },
  metrics: null,
};

function createLogStore() {
  const { subscribe, set, update } = writable<LogState>(initialState);

  return {
    subscribe,

    async fetchLogs(limit = 100, level?: string, category?: string) {
      try {
        const entries = await invoke<LogEntry[]>('get_recent_logs', {
          limit,
          level,
          category,
        });
        update((state) => ({ ...state, entries }));
      } catch (e) {
        console.error('Failed to fetch logs:', e);
      }
    },

    async fetchMetrics() {
      try {
        const metrics = await invoke<PerformanceMetrics>('get_performance_metrics');
        update((state) => ({ ...state, metrics }));
      } catch (e) {
        console.error('Failed to fetch metrics:', e);
      }
    },

    async clearLogs() {
      try {
        await invoke('clear_logs');
        update((state) => ({ ...state, entries: [] }));
      } catch (e) {
        console.error('Failed to clear logs:', e);
      }
    },

    async exportLogs(outputPath: string) {
      try {
        await invoke('export_logs', { outputPath });
      } catch (e) {
        console.error('Failed to export logs:', e);
        throw e;
      }
    },

    setFilter(filter: Partial<LogState['filter']>) {
      update((state) => ({
        ...state,
        filter: { ...state.filter, ...filter },
      }));
    },

    addEntry(entry: LogEntry) {
      update((state) => ({
        ...state,
        entries: [...state.entries.slice(-999), entry],
      }));
    },

    async startStreaming() {
      try {
        await invoke('subscribe_to_logs');
        const unlisten = await listen<LogEntry>('log-entry', (event) => {
          this.addEntry(event.payload);
        });
        update((state) => ({ ...state, isStreaming: true }));
        return unlisten;
      } catch (e) {
        console.error('Failed to start log streaming:', e);
      }
    },
  };
}

export const logStore = createLogStore();

export const filteredLogs = derived(logStore, ($log) => {
  let entries = $log.entries;

  if ($log.filter.level) {
    entries = entries.filter((e) => e.level.toLowerCase() === $log.filter.level?.toLowerCase());
  }

  if ($log.filter.category) {
    entries = entries.filter((e) => e.category === $log.filter.category);
  }

  if ($log.filter.search) {
    const search = $log.filter.search.toLowerCase();
    entries = entries.filter(
      (e) => e.message.toLowerCase().includes(search) || e.target.toLowerCase().includes(search)
    );
  }

  return entries;
});
