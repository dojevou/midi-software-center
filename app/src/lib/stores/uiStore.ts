import { writable, derived } from 'svelte/store';
import type { WindowId, WindowPosition } from '$lib/types';

// ============================================================================
// UI STATE
// ============================================================================

export interface UIState {
  windows: Record<WindowId, WindowPosition>;
  sidebarVisible: boolean;
  inspectorVisible: boolean;
  theme: 'dark' | 'light';
}

const initialState: UIState = {
  windows: {
    daw: {
      x: 50,
      y: 50,
      width: 800,
      height: 600,
      z_index: 1,
      visible: true,
    },
    mixer: {
      x: 900,
      y: 50,
      width: 400,
      height: 600,
      z_index: 2,
      visible: false,
    },
    database: {
      x: 50,
      y: 700,
      width: 600,
      height: 400,
      z_index: 3,
      visible: false,
    },
    pipeline: {
      x: 700,
      y: 700,
      width: 600,
      height: 400,
      z_index: 4,
      visible: false,
    },
  },
  sidebarVisible: true,
  inspectorVisible: true,
  theme: 'dark',
};

export const uiStore = writable<UIState>(initialState);

// Load from localStorage
if (typeof window !== 'undefined') {
  try {
    const saved = localStorage.getItem('ui-state');
    if (saved) {
      const parsed = JSON.parse(saved);
      uiStore.set({ ...initialState, ...parsed });
    }
  } catch (e) {
    console.error('Failed to load UI state:', e);
  }
}

// Save to localStorage on changes
uiStore.subscribe((state) => {
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem('ui-state', JSON.stringify(state));
    } catch (e) {
      console.error('Failed to save UI state:', e);
    }
  }
});

// ============================================================================
// DERIVED STORES
// ============================================================================

export const visibleWindows = derived(
  uiStore,
  ($ui) => Object.entries($ui.windows)
    .filter(([_, pos]) => pos.visible)
    .map(([id]) => id as WindowId)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const uiActions = {
  toggleWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: !state.windows[windowId].visible,
        },
      },
    }));
  },

  showWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: true,
        },
      },
    }));
  },

  hideWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: false,
        },
      },
    }));
  },

  setWindowPosition(windowId: WindowId, x: number, y: number) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          x,
          y,
        },
      },
    }));
  },

  setWindowSize(windowId: WindowId, width: number, height: number) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          width,
          height,
        },
      },
    }));
  },

  bringToFront(windowId: WindowId) {
    uiStore.update(state => {
      const maxZ = Math.max(...Object.values(state.windows).map(w => w.z_index));
      return {
        ...state,
        windows: {
          ...state.windows,
          [windowId]: {
            ...state.windows[windowId],
            z_index: maxZ + 1,
          },
        },
      };
    });
  },

  toggleSidebar() {
    uiStore.update(state => ({ ...state, sidebarVisible: !state.sidebarVisible }));
  },

  toggleInspector() {
    uiStore.update(state => ({ ...state, inspectorVisible: !state.inspectorVisible }));
  },

  setTheme(theme: 'dark' | 'light') {
    uiStore.update(state => ({ ...state, theme }));
  },
};