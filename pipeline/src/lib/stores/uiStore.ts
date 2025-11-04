/**
 * UI Store - Window state and UI management
 *
 * Grown-up Script: Manages window positions, visibility, and UI state.
 * Handles window management system integration.
 */

import { writable, derived, get } from 'svelte/store';

// ============================================================================
// Types
// ============================================================================

export type WindowId = 'daw' | 'mixer' | 'database' | 'pipeline';

export interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
  minimized: boolean;
  zIndex: number;
}

export interface UIState {
  activeWindow: WindowId | null;
  windowPositions: Map<WindowId, WindowPosition>;
  windowVisibility: Map<WindowId, boolean>;
  highestZIndex: number;
  showMenuBar: boolean;
  showStatusBar: boolean;
  gridSnapping: boolean;
  gridSize: number;
  theme: 'dark' | 'light';
}

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_POSITIONS: Record<WindowId, WindowPosition> = {
  daw: {
    x: 50,
    y: 80,
    width: 1200,
    height: 600,
    minimized: false,
    zIndex: 1
  },
  mixer: {
    x: 100,
    y: 130,
    width: 1000,
    height: 500,
    minimized: false,
    zIndex: 2
  },
  database: {
    x: 150,
    y: 180,
    width: 900,
    height: 550,
    minimized: false,
    zIndex: 3
  },
  pipeline: {
    x: 200,
    y: 230,
    width: 800,
    height: 500,
    minimized: false,
    zIndex: 4
  }
};

const DEFAULT_STATE: UIState = {
  activeWindow: 'daw',
  windowPositions: new Map(Object.entries(DEFAULT_POSITIONS) as [WindowId, WindowPosition][]),
  windowVisibility: new Map([
    ['daw', true],
    ['mixer', false],
    ['database', false],
    ['pipeline', false]
  ]),
  highestZIndex: 4,
  showMenuBar: true,
  showStatusBar: true,
  gridSnapping: true,
  gridSize: 10,
  theme: 'dark'
};

// ============================================================================
// Store
// ============================================================================

const { subscribe, set, update } = writable<UIState>(DEFAULT_STATE);

// ============================================================================
// Local Storage Persistence
// ============================================================================

const STORAGE_KEY = 'midi-center-ui-state';

function loadStateFromStorage(): Partial<UIState> | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Convert plain objects back to Maps
      if (parsed.windowPositions) {
        parsed.windowPositions = new Map(Object.entries(parsed.windowPositions));
      }
      if (parsed.windowVisibility) {
        parsed.windowVisibility = new Map(Object.entries(parsed.windowVisibility));
      }
      return parsed;
    }
  } catch (error) {
    console.error('Failed to load UI state from storage:', error);
  }
  return null;
}

function saveStateToStorage(state: UIState): void {
  try {
    // Convert Maps to plain objects for JSON serialization
    const serializable = {
      ...state,
      windowPositions: Object.fromEntries(state.windowPositions),
      windowVisibility: Object.fromEntries(state.windowVisibility)
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(serializable));
  } catch (error) {
    console.error('Failed to save UI state to storage:', error);
  }
}

// Load initial state from storage
const storedState = loadStateFromStorage();
if (storedState) {
  update(s => ({ ...s, ...storedState }));
}

// Auto-save on changes
subscribe(saveStateToStorage);

// ============================================================================
// Actions
// ============================================================================

export const uiActions = {
  /**
   * Set active window
   */
  setActiveWindow(windowId: WindowId): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        const newZIndex = s.highestZIndex + 1;
        newPositions.set(windowId, { ...position, zIndex: newZIndex });

        return {
          ...s,
          activeWindow: windowId,
          windowPositions: newPositions,
          highestZIndex: newZIndex
        };
      }

      return { ...s, activeWindow: windowId };
    });
  },

  /**
   * Show a window
   */
  showWindow(windowId: WindowId): void {
    update(s => {
      const newVisibility = new Map(s.windowVisibility);
      newVisibility.set(windowId, true);

      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position && position.minimized) {
        newPositions.set(windowId, { ...position, minimized: false });
      }

      return {
        ...s,
        windowVisibility: newVisibility,
        windowPositions: newPositions,
        activeWindow: windowId
      };
    });

    this.setActiveWindow(windowId);
  },

  /**
   * Hide a window
   */
  hideWindow(windowId: WindowId): void {
    update(s => {
      const newVisibility = new Map(s.windowVisibility);
      newVisibility.set(windowId, false);

      return {
        ...s,
        windowVisibility: newVisibility,
        activeWindow: s.activeWindow === windowId ? null : s.activeWindow
      };
    });
  },

  /**
   * Toggle window visibility
   */
  toggleWindow(windowId: WindowId): void {
    const state = get(uiStore);
    const isVisible = state.windowVisibility.get(windowId) || false;

    if (isVisible) {
      this.hideWindow(windowId);
    } else {
      this.showWindow(windowId);
    }
  },

  /**
   * Minimize a window
   */
  minimizeWindow(windowId: WindowId): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        newPositions.set(windowId, { ...position, minimized: true });
      }

      return { ...s, windowPositions: newPositions };
    });
  },

  /**
   * Restore a minimized window
   */
  restoreWindow(windowId: WindowId): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        newPositions.set(windowId, { ...position, minimized: false });
      }

      return { ...s, windowPositions: newPositions };
    });

    this.setActiveWindow(windowId);
  },

  /**
   * Set window position
   */
  setWindowPosition(windowId: WindowId, x: number, y: number): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        let finalX = x;
        let finalY = y;

        // Apply grid snapping if enabled
        if (s.gridSnapping) {
          finalX = Math.round(x / s.gridSize) * s.gridSize;
          finalY = Math.round(y / s.gridSize) * s.gridSize;
        }

        newPositions.set(windowId, { ...position, x: finalX, y: finalY });
      }

      return { ...s, windowPositions: newPositions };
    });
  },

  /**
   * Set window size
   */
  setWindowSize(windowId: WindowId, width: number, height: number): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        newPositions.set(windowId, { ...position, width, height });
      }

      return { ...s, windowPositions: newPositions };
    });
  },

  /**
   * Set window bounds (position and size)
   */
  setWindowBounds(
    windowId: WindowId,
    x: number,
    y: number,
    width: number,
    height: number
  ): void {
    update(s => {
      const newPositions = new Map(s.windowPositions);
      const position = newPositions.get(windowId);

      if (position) {
        let finalX = x;
        let finalY = y;

        if (s.gridSnapping) {
          finalX = Math.round(x / s.gridSize) * s.gridSize;
          finalY = Math.round(y / s.gridSize) * s.gridSize;
        }

        newPositions.set(windowId, {
          ...position,
          x: finalX,
          y: finalY,
          width,
          height
        });
      }

      return { ...s, windowPositions: newPositions };
    });
  },

  /**
   * Toggle grid snapping
   */
  toggleGridSnapping(): void {
    update(s => ({ ...s, gridSnapping: !s.gridSnapping }));
  },

  /**
   * Set grid size
   */
  setGridSize(size: number): void {
    if (size < 1 || size > 50) {
      console.error('Grid size must be between 1 and 50');
      return;
    }
    update(s => ({ ...s, gridSize: size }));
  },

  /**
   * Toggle menu bar visibility
   */
  toggleMenuBar(): void {
    update(s => ({ ...s, showMenuBar: !s.showMenuBar }));
  },

  /**
   * Toggle status bar visibility
   */
  toggleStatusBar(): void {
    update(s => ({ ...s, showStatusBar: !s.showStatusBar }));
  },

  /**
   * Set theme
   */
  setTheme(theme: 'dark' | 'light'): void {
    update(s => ({ ...s, theme }));
  },

  /**
   * Reset window positions to defaults
   */
  resetWindowPositions(): void {
    update(s => ({
      ...s,
      windowPositions: new Map(Object.entries(DEFAULT_POSITIONS) as [WindowId, WindowPosition][])
    }));
  },

  /**
   * Reset all UI state to defaults
   */
  reset(): void {
    set(DEFAULT_STATE);
    localStorage.removeItem(STORAGE_KEY);
  }
};

// ============================================================================
// Derived Stores
// ============================================================================

export const visibleWindows = derived(
  uiStore,
  $ui => {
    const windows: WindowId[] = [];
    for (const [windowId, visible] of $ui.windowVisibility) {
      if (visible) {
        windows.push(windowId);
      }
    }
    return windows;
  }
);

export const activeWindowPosition = derived(
  uiStore,
  $ui => $ui.activeWindow ? $ui.windowPositions.get($ui.activeWindow) || null : null
);

// ============================================================================
// Export
// ============================================================================

export const uiStore = {
  subscribe,
  set,
  update
};
