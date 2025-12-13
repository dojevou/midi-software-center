import { derived, get, writable } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

/**
 * Represents a single undoable action
 */
export interface UndoableAction<T = unknown> {
  /** Unique identifier for the action */
  id: string;
  /** Human-readable description of the action */
  description: string;
  /** Category for grouping related actions */
  category: ActionCategory;
  /** Timestamp when the action was performed */
  timestamp: number;
  /** Data needed to undo the action */
  undoData: T;
  /** Data needed to redo the action */
  redoData: T;
  /** Optional group ID for batched operations */
  groupId?: string;
  /** Whether this action can be merged with previous similar actions */
  mergeable?: boolean;
  /** Custom merge key for identifying mergeable actions */
  mergeKey?: string;
}

/**
 * Categories of undoable actions
 */
export type ActionCategory =
  | 'note' // Note editing (add, delete, move, resize)
  | 'velocity' // Velocity changes
  | 'automation' // Automation lane edits
  | 'track' // Track operations (add, delete, mute, solo)
  | 'mixer' // Mixer changes (volume, pan, sends)
  | 'effect' // Effect parameter changes
  | 'transport' // Transport settings (tempo, time signature)
  | 'selection' // Selection changes
  | 'clipboard' // Cut/copy/paste operations
  | 'file' // File operations (import, export)
  | 'project' // Project-level changes
  | 'midi' // MIDI device/mapping changes
  | 'settings' // Application settings
  | 'custom'; // Custom/plugin actions

/**
 * Handler functions for executing undo/redo operations
 */
export interface UndoHandler<T = unknown> {
  /** Execute the undo operation */
  undo: (data: T) => Promise<void> | void;
  /** Execute the redo operation */
  redo: (data: T) => Promise<void> | void;
  /** Optional validation before undo */
  canUndo?: (data: T) => boolean;
  /** Optional validation before redo */
  canRedo?: (data: T) => boolean;
}

/**
 * Registry of undo handlers by category
 */
export type UndoHandlerRegistry = Partial<Record<ActionCategory, UndoHandler>>;

/**
 * Configuration for the undo system
 */
export interface UndoConfig {
  /** Maximum number of actions to keep in history */
  maxHistorySize: number;
  /** Time window in ms for merging similar actions */
  mergeWindowMs: number;
  /** Whether to persist history across sessions */
  persistHistory: boolean;
  /** Storage key for persistence */
  storageKey: string;
  /** Categories to exclude from history */
  excludeCategories: ActionCategory[];
  /** Whether to log undo/redo operations */
  debug: boolean;
}

/**
 * State of the undo system
 */
export interface UndoState {
  /** Stack of actions that can be undone */
  undoStack: UndoableAction[];
  /** Stack of actions that can be redone */
  redoStack: UndoableAction[];
  /** Currently active group ID for batch operations */
  activeGroupId: string | null;
  /** Whether an undo/redo operation is in progress */
  isProcessing: boolean;
  /** Last error that occurred */
  lastError: string | null;
  /** Configuration */
  config: UndoConfig;
}

/**
 * Snapshot of application state for full state undo
 */
export interface StateSnapshot<T = unknown> {
  id: string;
  timestamp: number;
  description: string;
  state: T;
}

// ============================================================================
// DEFAULT CONFIGURATION
// ============================================================================

const defaultConfig: UndoConfig = {
  maxHistorySize: 100,
  mergeWindowMs: 500,
  persistHistory: false,
  storageKey: 'midi-center-undo-history',
  excludeCategories: [],
  debug: false,
};

// ============================================================================
// STORE
// ============================================================================

const initialState: UndoState = {
  undoStack: [],
  redoStack: [],
  activeGroupId: null,
  isProcessing: false,
  lastError: null,
  config: defaultConfig,
};

const { subscribe, set, update } = writable<UndoState>(initialState);

export const undoStore = { subscribe };

// ============================================================================
// HANDLER REGISTRY
// ============================================================================

const handlerRegistry: UndoHandlerRegistry = {};

// ============================================================================
// DERIVED STORES
// ============================================================================

/** Whether undo is available */
export const canUndo = derived(
  undoStore,
  ($store) => $store.undoStack.length > 0 && !$store.isProcessing
);

/** Whether redo is available */
export const canRedo = derived(
  undoStore,
  ($store) => $store.redoStack.length > 0 && !$store.isProcessing
);

/** Number of actions that can be undone */
export const undoCount = derived(undoStore, ($store) => $store.undoStack.length);

/** Number of actions that can be redone */
export const redoCount = derived(undoStore, ($store) => $store.redoStack.length);

/** Description of the next undo action */
export const nextUndoDescription = derived(undoStore, ($store) => {
  const action = $store.undoStack[$store.undoStack.length - 1];
  return action?.description ?? null;
});

/** Description of the next redo action */
export const nextRedoDescription = derived(undoStore, ($store) => {
  const action = $store.redoStack[$store.redoStack.length - 1];
  return action?.description ?? null;
});

/** Whether a batch operation is in progress */
export const isBatching = derived(undoStore, ($store) => $store.activeGroupId !== null);

/** Whether undo/redo is currently processing */
export const isProcessing = derived(undoStore, ($store) => $store.isProcessing);

/** Recent undo history (last 10 actions) */
export const recentHistory = derived(undoStore, ($store) => $store.undoStack.slice(-10).reverse());

/** Actions grouped by category */
export const actionsByCategory = derived(undoStore, ($store) => {
  const grouped: Partial<Record<ActionCategory, number>> = {};
  for (const action of $store.undoStack) {
    grouped[action.category] = (grouped[action.category] || 0) + 1;
  }
  return grouped;
});

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Generate a unique action ID
 */
function generateActionId(): string {
  return `action_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Generate a unique group ID
 */
function generateGroupId(): string {
  return `group_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Check if two actions can be merged
 */
function canMergeActions(
  existing: UndoableAction,
  incoming: UndoableAction,
  mergeWindowMs: number
): boolean {
  if (!existing.mergeable || !incoming.mergeable) {
    return false;
  }
  if (existing.category !== incoming.category) {
    return false;
  }
  if (existing.mergeKey !== incoming.mergeKey) {
    return false;
  }
  if (incoming.timestamp - existing.timestamp > mergeWindowMs) {
    return false;
  }
  return true;
}

/**
 * Log debug messages if enabled
 */
function debugLog(config: UndoConfig, message: string, data?: unknown): void {
  if (config.debug) {
    console.log(`[UndoStore] ${message}`, data ?? '');
  }
}

// ============================================================================
// ACTIONS
// ============================================================================

export const undoActions = {
  // ==========================================================================
  // CONFIGURATION
  // ==========================================================================

  /**
   * Update configuration
   */
  configure(config: Partial<UndoConfig>): void {
    update((state) => ({
      ...state,
      config: { ...state.config, ...config },
    }));
  },

  /**
   * Register a handler for a category
   */
  registerHandler<T>(category: ActionCategory, handler: UndoHandler<T>): void {
    handlerRegistry[category] = handler as UndoHandler;
  },

  /**
   * Unregister a handler
   */
  unregisterHandler(category: ActionCategory): void {
    delete handlerRegistry[category];
  },

  // ==========================================================================
  // RECORDING ACTIONS
  // ==========================================================================

  /**
   * Record an undoable action
   */
  record<T>(
    description: string,
    category: ActionCategory,
    undoData: T,
    redoData: T,
    options?: {
      mergeable?: boolean;
      mergeKey?: string;
    }
  ): string {
    const state = get(undoStore);

    // Check if category is excluded
    if (state.config.excludeCategories.includes(category)) {
      debugLog(state.config, `Skipping excluded category: ${category}`);
      return '';
    }

    const actionId = generateActionId();
    const action: UndoableAction<T> = {
      id: actionId,
      description,
      category,
      timestamp: Date.now(),
      undoData,
      redoData,
      groupId: state.activeGroupId ?? undefined,
      mergeable: options?.mergeable,
      mergeKey: options?.mergeKey,
    };

    update((s) => {
      let newUndoStack = [...s.undoStack];

      // Check for merge with last action
      const lastAction = newUndoStack[newUndoStack.length - 1];
      if (lastAction && canMergeActions(lastAction, action, s.config.mergeWindowMs)) {
        // Merge: update the last action's redo data
        newUndoStack[newUndoStack.length - 1] = {
          ...lastAction,
          redoData: action.redoData,
          timestamp: action.timestamp,
        };
        debugLog(s.config, 'Merged action with previous', { description });
      } else {
        // Add new action
        newUndoStack.push(action as UndoableAction);

        // Trim history if needed
        if (newUndoStack.length > s.config.maxHistorySize) {
          newUndoStack = newUndoStack.slice(-s.config.maxHistorySize);
        }
      }

      debugLog(s.config, 'Recorded action', { description, category, actionId });

      return {
        ...s,
        undoStack: newUndoStack,
        // Clear redo stack when new action is recorded
        redoStack: [],
        lastError: null,
      };
    });

    return actionId;
  },

  /**
   * Record a simple action with symmetric undo/redo data
   */
  recordSimple<T>(
    description: string,
    category: ActionCategory,
    beforeState: T,
    afterState: T,
    options?: { mergeable?: boolean; mergeKey?: string }
  ): string {
    return this.record(description, category, beforeState, afterState, options);
  },

  // ==========================================================================
  // UNDO/REDO OPERATIONS
  // ==========================================================================

  /**
   * Undo the last action
   */
  async undo(): Promise<boolean> {
    const state = get(undoStore);

    if (state.undoStack.length === 0 || state.isProcessing) {
      return false;
    }

    const action = state.undoStack[state.undoStack.length - 1];
    const handler = handlerRegistry[action.category];

    // Check if handler can undo
    if (handler?.canUndo && !handler.canUndo(action.undoData)) {
      update((s) => ({ ...s, lastError: `Cannot undo: ${action.description}` }));
      return false;
    }

    update((s) => ({ ...s, isProcessing: true, lastError: null }));

    try {
      // Execute undo handler if registered
      if (handler?.undo) {
        await handler.undo(action.undoData);
      }

      update((s) => {
        const newUndoStack = s.undoStack.slice(0, -1);
        const newRedoStack = [...s.redoStack, action];

        debugLog(s.config, 'Undid action', { description: action.description });

        return {
          ...s,
          undoStack: newUndoStack,
          redoStack: newRedoStack,
          isProcessing: false,
        };
      });

      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      update((s) => ({
        ...s,
        isProcessing: false,
        lastError: `Undo failed: ${errorMessage}`,
      }));
      console.error('[UndoStore] Undo failed:', error);
      return false;
    }
  },

  /**
   * Redo the last undone action
   */
  async redo(): Promise<boolean> {
    const state = get(undoStore);

    if (state.redoStack.length === 0 || state.isProcessing) {
      return false;
    }

    const action = state.redoStack[state.redoStack.length - 1];
    const handler = handlerRegistry[action.category];

    // Check if handler can redo
    if (handler?.canRedo && !handler.canRedo(action.redoData)) {
      update((s) => ({ ...s, lastError: `Cannot redo: ${action.description}` }));
      return false;
    }

    update((s) => ({ ...s, isProcessing: true, lastError: null }));

    try {
      // Execute redo handler if registered
      if (handler?.redo) {
        await handler.redo(action.redoData);
      }

      update((s) => {
        const newRedoStack = s.redoStack.slice(0, -1);
        const newUndoStack = [...s.undoStack, action];

        debugLog(s.config, 'Redid action', { description: action.description });

        return {
          ...s,
          undoStack: newUndoStack,
          redoStack: newRedoStack,
          isProcessing: false,
        };
      });

      return true;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      update((s) => ({
        ...s,
        isProcessing: false,
        lastError: `Redo failed: ${errorMessage}`,
      }));
      console.error('[UndoStore] Redo failed:', error);
      return false;
    }
  },

  /**
   * Undo multiple actions at once
   */
  async undoMultiple(count: number): Promise<number> {
    let undoneCount = 0;
    for (let i = 0; i < count; i++) {
      const success = await this.undo();
      if (!success) {
        break;
      }
      undoneCount++;
    }
    return undoneCount;
  },

  /**
   * Redo multiple actions at once
   */
  async redoMultiple(count: number): Promise<number> {
    let redoneCount = 0;
    for (let i = 0; i < count; i++) {
      const success = await this.redo();
      if (!success) {
        break;
      }
      redoneCount++;
    }
    return redoneCount;
  },

  /**
   * Undo to a specific action ID
   */
  async undoTo(actionId: string): Promise<boolean> {
    const state = get(undoStore);
    const index = state.undoStack.findIndex((a) => a.id === actionId);

    if (index === -1) {
      return false;
    }

    const count = state.undoStack.length - index;
    const undoneCount = await this.undoMultiple(count);
    return undoneCount === count;
  },

  // ==========================================================================
  // BATCH OPERATIONS
  // ==========================================================================

  /**
   * Start a batch operation (groups multiple actions as one undo unit)
   */
  beginBatch(description?: string): string {
    const groupId = generateGroupId();
    update((s) => {
      debugLog(s.config, 'Started batch', { groupId, description });
      return { ...s, activeGroupId: groupId };
    });
    return groupId;
  },

  /**
   * End the current batch operation
   */
  endBatch(): void {
    update((s) => {
      debugLog(s.config, 'Ended batch', { groupId: s.activeGroupId });
      return { ...s, activeGroupId: null };
    });
  },

  /**
   * Execute a function as a batch operation
   */
  async batch<T>(description: string, fn: () => Promise<T> | T): Promise<T> {
    const groupId = this.beginBatch(description);
    try {
      const result = await fn();
      return result;
    } finally {
      this.endBatch();
    }
  },

  /**
   * Undo all actions in a group
   */
  async undoGroup(groupId: string): Promise<boolean> {
    const state = get(undoStore);
    const groupActions = state.undoStack.filter((a) => a.groupId === groupId);

    if (groupActions.length === 0) {
      return false;
    }

    // Undo from most recent to oldest
    for (let i = groupActions.length - 1; i >= 0; i--) {
      await this.undoTo(groupActions[i].id);
    }

    return true;
  },

  // ==========================================================================
  // HISTORY MANAGEMENT
  // ==========================================================================

  /**
   * Clear all undo history
   */
  clearHistory(): void {
    update((s) => {
      debugLog(s.config, 'Cleared history');
      return {
        ...s,
        undoStack: [],
        redoStack: [],
        lastError: null,
      };
    });
  },

  /**
   * Clear only the redo stack
   */
  clearRedoStack(): void {
    update((s) => ({ ...s, redoStack: [] }));
  },

  /**
   * Remove actions older than a specified age
   */
  pruneHistory(maxAgeMs: number): number {
    const cutoff = Date.now() - maxAgeMs;
    let removedCount = 0;

    update((s) => {
      const newUndoStack = s.undoStack.filter((a) => {
        if (a.timestamp < cutoff) {
          removedCount++;
          return false;
        }
        return true;
      });

      debugLog(s.config, `Pruned ${removedCount} actions`);

      return {
        ...s,
        undoStack: newUndoStack,
      };
    });

    return removedCount;
  },

  /**
   * Remove actions of a specific category
   */
  removeCategory(category: ActionCategory): number {
    let removedCount = 0;

    update((s) => {
      const newUndoStack = s.undoStack.filter((a) => {
        if (a.category === category) {
          removedCount++;
          return false;
        }
        return true;
      });

      const newRedoStack = s.redoStack.filter((a) => a.category !== category);

      return {
        ...s,
        undoStack: newUndoStack,
        redoStack: newRedoStack,
      };
    });

    return removedCount;
  },

  /**
   * Get the full undo history
   */
  getHistory(): UndoableAction[] {
    return get(undoStore).undoStack;
  },

  /**
   * Get the full redo history
   */
  getRedoHistory(): UndoableAction[] {
    return get(undoStore).redoStack;
  },

  // ==========================================================================
  // PERSISTENCE
  // ==========================================================================

  /**
   * Save history to storage
   */
  saveHistory(): void {
    const state = get(undoStore);
    if (!state.config.persistHistory) {
      return;
    }

    try {
      const data = {
        undoStack: state.undoStack,
        redoStack: state.redoStack,
        timestamp: Date.now(),
      };
      localStorage.setItem(state.config.storageKey, JSON.stringify(data));
      debugLog(state.config, 'Saved history to storage');
    } catch (error) {
      console.error('[UndoStore] Failed to save history:', error);
    }
  },

  /**
   * Load history from storage
   */
  loadHistory(): boolean {
    const state = get(undoStore);
    if (!state.config.persistHistory) {
      return false;
    }

    try {
      const stored = localStorage.getItem(state.config.storageKey);
      if (!stored) {
        return false;
      }

      const data = JSON.parse(stored);
      update((s) => ({
        ...s,
        undoStack: data.undoStack || [],
        redoStack: data.redoStack || [],
      }));

      debugLog(state.config, 'Loaded history from storage');
      return true;
    } catch (error) {
      console.error('[UndoStore] Failed to load history:', error);
      return false;
    }
  },

  /**
   * Clear persisted history
   */
  clearStoredHistory(): void {
    const state = get(undoStore);
    localStorage.removeItem(state.config.storageKey);
    debugLog(state.config, 'Cleared stored history');
  },

  // ==========================================================================
  // STATE MANAGEMENT
  // ==========================================================================

  /**
   * Clear any error state
   */
  clearError(): void {
    update((s) => ({ ...s, lastError: null }));
  },

  /**
   * Reset the entire undo system
   */
  reset(): void {
    set(initialState);
  },
};

// ============================================================================
// SNAPSHOT-BASED UNDO (Alternative approach for full state capture)
// ============================================================================

const snapshotStore = writable<StateSnapshot[]>([]);
const snapshotIndex = writable<number>(-1);

export const snapshotUndoStore = { subscribe: snapshotStore.subscribe };

export const snapshotActions = {
  /**
   * Maximum number of snapshots to keep
   */
  maxSnapshots: 50,

  /**
   * Take a snapshot of the current state
   */
  takeSnapshot<T>(state: T, description: string): string {
    const id = `snapshot_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
    const snapshot: StateSnapshot<T> = {
      id,
      timestamp: Date.now(),
      description,
      state: structuredClone(state),
    };

    snapshotStore.update((snapshots) => {
      const currentIndex = get(snapshotIndex);
      // Remove any snapshots after current position (invalidate redo)
      const newSnapshots = snapshots.slice(0, currentIndex + 1);
      newSnapshots.push(snapshot);

      // Trim if too many
      if (newSnapshots.length > this.maxSnapshots) {
        newSnapshots.shift();
      }

      return newSnapshots;
    });

    snapshotIndex.update((i) => {
      const snapshots = get(snapshotStore);
      return snapshots.length - 1;
    });

    return id;
  },

  /**
   * Restore a previous snapshot
   */
  restoreSnapshot<T>(index: number): T | null {
    const snapshots = get(snapshotStore);
    if (index < 0 || index >= snapshots.length) {
      return null;
    }

    snapshotIndex.set(index);
    return structuredClone(snapshots[index].state) as T;
  },

  /**
   * Go to previous snapshot
   */
  previous<T>(): T | null {
    const currentIndex = get(snapshotIndex);
    if (currentIndex <= 0) {
      return null;
    }
    return this.restoreSnapshot(currentIndex - 1);
  },

  /**
   * Go to next snapshot
   */
  next<T>(): T | null {
    const currentIndex = get(snapshotIndex);
    const snapshots = get(snapshotStore);
    if (currentIndex >= snapshots.length - 1) {
      return null;
    }
    return this.restoreSnapshot(currentIndex + 1);
  },

  /**
   * Check if previous snapshot exists
   */
  hasPrevious(): boolean {
    return get(snapshotIndex) > 0;
  },

  /**
   * Check if next snapshot exists
   */
  hasNext(): boolean {
    const currentIndex = get(snapshotIndex);
    const snapshots = get(snapshotStore);
    return currentIndex < snapshots.length - 1;
  },

  /**
   * Get all snapshots
   */
  getSnapshots(): StateSnapshot[] {
    return get(snapshotStore);
  },

  /**
   * Clear all snapshots
   */
  clearSnapshots(): void {
    snapshotStore.set([]);
    snapshotIndex.set(-1);
  },
};

// ============================================================================
// KEYBOARD SHORTCUT HELPERS
// ============================================================================

/**
 * Check if an event is Ctrl/Cmd+Z (undo)
 */
export function isUndoShortcut(event: KeyboardEvent): boolean {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const modifier = isMac ? event.metaKey : event.ctrlKey;
  return modifier && !event.shiftKey && event.key.toLowerCase() === 'z';
}

/**
 * Check if an event is Ctrl/Cmd+Shift+Z or Ctrl/Cmd+Y (redo)
 */
export function isRedoShortcut(event: KeyboardEvent): boolean {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const modifier = isMac ? event.metaKey : event.ctrlKey;

  // Ctrl/Cmd+Shift+Z
  if (modifier && event.shiftKey && event.key.toLowerCase() === 'z') {
    return true;
  }

  // Ctrl/Cmd+Y (Windows/Linux convention)
  if (!isMac && modifier && !event.shiftKey && event.key.toLowerCase() === 'y') {
    return true;
  }

  return false;
}

/**
 * Setup global keyboard handler for undo/redo
 */
export function setupUndoKeyboardHandler(): () => void {
  const handler = async (event: KeyboardEvent) => {
    // Don't handle if focus is in an input
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
      return;
    }

    if (isUndoShortcut(event)) {
      event.preventDefault();
      await undoActions.undo();
    } else if (isRedoShortcut(event)) {
      event.preventDefault();
      await undoActions.redo();
    }
  };

  window.addEventListener('keydown', handler);

  return () => {
    window.removeEventListener('keydown', handler);
  };
}

// ============================================================================
// PRE-BUILT HANDLERS FOR COMMON OPERATIONS
// ============================================================================

/**
 * Create a simple state swap handler
 */
export function createSwapHandler<T>(
  applyState: (state: T) => void | Promise<void>
): UndoHandler<T> {
  return {
    undo: applyState,
    redo: applyState,
  };
}

/**
 * Create a handler for array operations (add/remove items)
 */
export function createArrayHandler<T>(
  getArray: () => T[],
  setArray: (items: T[]) => void
): UndoHandler<{ items: T[]; indices: number[]; operation: 'add' | 'remove' }> {
  return {
    undo: (data) => {
      const current = getArray();
      if (data.operation === 'add') {
        // Undo add = remove items
        const newArray = current.filter((_, i) => !data.indices.includes(i));
        setArray(newArray);
      } else {
        // Undo remove = add items back
        const newArray = [...current];
        data.items.forEach((item, i) => {
          newArray.splice(data.indices[i], 0, item);
        });
        setArray(newArray);
      }
    },
    redo: (data) => {
      const current = getArray();
      if (data.operation === 'add') {
        // Redo add = add items
        const newArray = [...current];
        data.items.forEach((item, i) => {
          newArray.splice(data.indices[i], 0, item);
        });
        setArray(newArray);
      } else {
        // Redo remove = remove items
        const newArray = current.filter((_, i) => !data.indices.includes(i));
        setArray(newArray);
      }
    },
  };
}

/**
 * Create a handler for property changes
 */
export function createPropertyHandler<T extends object>(
  getObject: () => T,
  setObject: (obj: T) => void
): UndoHandler<{ property: keyof T; oldValue: T[keyof T]; newValue: T[keyof T] }> {
  return {
    undo: (data) => {
      const obj = getObject();
      setObject({ ...obj, [data.property]: data.oldValue });
    },
    redo: (data) => {
      const obj = getObject();
      setObject({ ...obj, [data.property]: data.newValue });
    },
  };
}
