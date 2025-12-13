import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export interface LoadingTask {
  id: string;
  label: string;
  progress?: number; // 0-100, undefined for indeterminate
  startTime: number;
  cancelable: boolean;
  cancel?: () => void;
}

export interface LoadingState {
  tasks: Map<string, LoadingTask>;
  globalLoading: boolean;
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

function createLoadingStore() {
  const { subscribe, set, update } = writable<LoadingState>({
    tasks: new Map(),
    globalLoading: false,
  });

  function getState(): LoadingState {
    let state: LoadingState = { tasks: new Map(), globalLoading: false };
    subscribe(s => { state = s; })();
    return state;
  }

  const store = {
    subscribe,

    // Start a loading task
    start(
      id: string,
      label: string,
      options: { cancelable?: boolean; cancel?: () => void } = {}
    ) {
      update(state => {
        const tasks = new Map(state.tasks);
        tasks.set(id, {
          id,
          label,
          progress: undefined,
          startTime: Date.now(),
          cancelable: options.cancelable ?? false,
          cancel: options.cancel,
        });
        return { ...state, tasks, globalLoading: true };
      });

      return {
        setProgress: (progress: number) => store.setProgress(id, progress),
        setLabel: (label: string) => store.setLabel(id, label),
        finish: () => store.finish(id),
      };
    },

    // Update progress for a task
    setProgress(id: string, progress: number) {
      update(state => {
        const tasks = new Map(state.tasks);
        const task = tasks.get(id);
        if (task) {
          tasks.set(id, { ...task, progress: Math.min(100, Math.max(0, progress)) });
        }
        return { ...state, tasks };
      });
    },

    // Update label for a task
    setLabel(id: string, label: string) {
      update(state => {
        const tasks = new Map(state.tasks);
        const task = tasks.get(id);
        if (task) {
          tasks.set(id, { ...task, label });
        }
        return { ...state, tasks };
      });
    },

    // Finish a loading task
    finish(id: string) {
      update(state => {
        const tasks = new Map(state.tasks);
        tasks.delete(id);
        return {
          ...state,
          tasks,
          globalLoading: tasks.size > 0,
        };
      });
    },

    // Cancel a task
    cancel(id: string) {
      const state = getState();
      const task = state.tasks.get(id);
      if (task?.cancelable && task.cancel) {
        task.cancel();
      }
      store.finish(id);
    },

    // Set global loading state
    setGlobalLoading(loading: boolean) {
      update(state => ({ ...state, globalLoading: loading }));
    },

    // Helper: wrap async function with loading state
    async withLoading<T>(
      id: string,
      label: string,
      fn: () => Promise<T>,
      options?: { cancelable?: boolean }
    ): Promise<T> {
      const controller = new AbortController();
      const task = store.start(id, label, {
        ...options,
        cancel: () => controller.abort(),
      });

      try {
        const result = await fn();
        return result;
      } finally {
        task.finish();
      }
    },
  };

  return store;
}

export const loadingStore = createLoadingStore();

// Derived stores
export const isLoading = derived(loadingStore, $loading => $loading.globalLoading);
export const loadingTasks = derived(loadingStore, $loading => Array.from($loading.tasks.values()));
export const hasLoadingTasks = derived(loadingStore, $loading => $loading.tasks.size > 0);
