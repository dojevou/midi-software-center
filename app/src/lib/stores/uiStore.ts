import { derived, writable, get } from 'svelte/store';
import type { WindowId, WindowPosition } from '$lib/types';

// ============================================================================
// TYPES
// ============================================================================

export interface WindowLayout {
  id: string;
  name: string;
  windows: Record<WindowId, WindowPosition>;
  tabbedGroups: TabbedGroup[];
  createdAt: number;
  isPreset: boolean;
}

export interface TabbedGroup {
  id: string;
  windowIds: WindowId[];
  activeTab: WindowId;
  position: { x: number; y: number; width: number; height: number };
}

// ============================================================================
// UI STATE
// ============================================================================

export interface UIState {
  windows: Record<WindowId, WindowPosition>;
  minimizedWindows: WindowId[];
  tabbedGroups: TabbedGroup[];
  savedLayouts: WindowLayout[];
  activeLayoutId: string | null;
  sidebarVisible: boolean;
  inspectorVisible: boolean;
  theme: 'dark' | 'light';
}

// ============================================================================
// PRESET LAYOUTS
// ============================================================================

const PRESET_LAYOUTS: WindowLayout[] = [
  {
    id: 'arrangement',
    name: 'Arrangement (MPC 3.0)',
    windows: {
      arrangement: { x: 280, y: 32, width: 1000, height: 700, z_index: 1, visible: true },
      database: { x: 0, y: 32, width: 280, height: 700, z_index: 2, visible: true },
      mixer: { x: 280, y: 732, width: 1000, height: 200, z_index: 3, visible: true },
    } as Record<WindowId, WindowPosition>,
    tabbedGroups: [],
    createdAt: 0,
    isPreset: true,
  },
  {
    id: 'browse-edit',
    name: 'Browse & Edit',
    windows: {
      arrangement: { x: 0, y: 32, width: 800, height: 500, z_index: 1, visible: true },
      database: { x: 0, y: 532, width: 800, height: 400, z_index: 2, visible: true },
      mixer: { x: 800, y: 32, width: 480, height: 900, z_index: 3, visible: true },
    } as Record<WindowId, WindowPosition>,
    tabbedGroups: [],
    createdAt: 0,
    isPreset: true,
  },
  {
    id: 'performance',
    name: 'Performance',
    windows: {
      arrangement: { x: 0, y: 32, width: 960, height: 600, z_index: 1, visible: true },
      mixer: { x: 960, y: 32, width: 320, height: 600, z_index: 2, visible: true },
      'midi-monitor': { x: 0, y: 632, width: 640, height: 300, z_index: 3, visible: true },
    } as Record<WindowId, WindowPosition>,
    tabbedGroups: [],
    createdAt: 0,
    isPreset: true,
  },
  {
    id: 'full-edit',
    name: 'Full Edit',
    windows: {
      arrangement: { x: 0, y: 32, width: 640, height: 450, z_index: 1, visible: true },
      mixer: { x: 640, y: 32, width: 640, height: 450, z_index: 2, visible: true },
      database: { x: 0, y: 482, width: 640, height: 450, z_index: 3, visible: true },
      pipeline: { x: 640, y: 482, width: 640, height: 450, z_index: 4, visible: true },
    } as Record<WindowId, WindowPosition>,
    tabbedGroups: [],
    createdAt: 0,
    isPreset: true,
  },
];

const initialState: UIState = {
  windows: {
    // VIP3 Browser - Pro Tools style left side (2/5ths of 1920px = 768px)
    'vip3-browser': {
      x: 0,
      y: 32,
      width: 768,
      height: 1024,  // Full height minus menu/status bars
      z_index: 1,
      visible: true,
    },
    // Arrangement window - Pro Tools style top right (3/5ths width)
    arrangement: {
      x: 768,
      y: 32,
      width: 1152,
      height: 512,
      z_index: 2,
      visible: true,
    },
    // MIDI Mixer - Pro Tools style bottom right (3/5ths width)
    mixer: {
      x: 768,
      y: 544,
      width: 1152,
      height: 512,
      z_index: 3,
      visible: true,
    },
    // Database window - hidden by default (replaced by VIP3)
    database: {
      x: 0,
      y: 32,
      width: 280,
      height: 900,
      z_index: 4,
      visible: false,
    },
    // Legacy DAW window - hidden by default
    daw: {
      x: 50,
      y: 50,
      width: 800,
      height: 600,
      z_index: 5,
      visible: false,
    },
    pipeline: {
      x: 700,
      y: 700,
      width: 600,
      height: 400,
      z_index: 6,
      visible: false,
    },
    // Piano Roll - detachable editor
    'piano-roll': {
      x: 300,
      y: 200,
      width: 900,
      height: 500,
      z_index: 7,
      visible: false,
    },
    'midi-io-setup': {
      x: 150,
      y: 150,
      width: 900,
      height: 600,
      z_index: 8,
      visible: false,
    },
    'midi-monitor': {
      x: 200,
      y: 200,
      width: 800,
      height: 500,
      z_index: 9,
      visible: false,
    },
    'preferences': {
      x: 100,
      y: 100,
      width: 900,
      height: 650,
      z_index: 10,
      visible: false,
    },
    'gear-manager': {
      x: 120,
      y: 120,
      width: 950,
      height: 700,
      z_index: 11,
      visible: false,
    },
    'presets-manager': {
      x: 140,
      y: 140,
      width: 950,
      height: 700,
      z_index: 12,
      visible: false,
    },
    'export': {
      x: 180,
      y: 80,
      width: 700,
      height: 550,
      z_index: 16,
      visible: false,
    },
    'tag-editor': {
      x: 200,
      y: 100,
      width: 800,
      height: 600,
      z_index: 17,
      visible: false,
    },
    'favorites': {
      x: 220,
      y: 120,
      width: 900,
      height: 650,
      z_index: 18,
      visible: false,
    },
    'score': {
      x: 180,
      y: 80,
      width: 900,
      height: 650,
      z_index: 12,
      visible: false,
    },
    'script-editor': {
      x: 200,
      y: 100,
      width: 800,
      height: 600,
      z_index: 13,
      visible: false,
    },
    'midi-learn': {
      x: 220,
      y: 120,
      width: 600,
      height: 500,
      z_index: 14,
      visible: false,
    },
    'link-sync': {
      x: 240,
      y: 140,
      width: 500,
      height: 400,
      z_index: 15,
      visible: false,
    },
  },
  minimizedWindows: [],
  tabbedGroups: [],
  savedLayouts: [...PRESET_LAYOUTS],
  activeLayoutId: null,
  sidebarVisible: true,
  inspectorVisible: true,
  theme: 'dark',
};

export const uiStore = writable<UIState>(initialState);

// Load from localStorage but always force windows to be visible
if (typeof window !== 'undefined') {
  try {
    const saved = localStorage.getItem('ui-state');
    if (saved) {
      const parsed = JSON.parse(saved);
      // Merge saved state but always force all windows to visible: true
      const mergedState = { ...initialState, ...parsed };
      // Force all windows to be visible on startup
      Object.keys(mergedState.windows).forEach((key) => {
        mergedState.windows[key as WindowId].visible = true;
      });
      uiStore.set(mergedState);
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

export const visibleWindows = derived(uiStore, ($ui) =>
  Object.entries($ui.windows)
    .filter(([_, pos]) => pos.visible)
    .map(([id]) => id as WindowId)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const uiActions = {
  // ========== WINDOW VISIBILITY ==========
  toggleWindow(windowId: WindowId) {
    uiStore.update((state) => ({
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
    uiStore.update((state) => ({
      ...state,
      minimizedWindows: state.minimizedWindows.filter(id => id !== windowId),
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: true,
        },
      },
    }));
  },

  openWindow(windowId: WindowId) {
    // Alias for showWindow + bringToFront
    this.showWindow(windowId);
    this.bringToFront(windowId);
  },

  hideWindow(windowId: WindowId) {
    uiStore.update((state) => ({
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

  // ========== WINDOW POSITIONING ==========
  setWindowPosition(windowId: WindowId, x: number, y: number) {
    uiStore.update((state) => ({
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
    uiStore.update((state) => ({
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
    uiStore.update((state) => {
      const maxZ = Math.max(...Object.values(state.windows).map((w) => w.z_index));
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

  // ========== MINIMIZE / RESTORE ==========
  minimizeWindow(windowId: WindowId) {
    uiStore.update((state) => ({
      ...state,
      minimizedWindows: state.minimizedWindows.includes(windowId)
        ? state.minimizedWindows
        : [...state.minimizedWindows, windowId],
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: false,
        },
      },
    }));
  },

  restoreWindow(windowId: WindowId) {
    uiStore.update((state) => {
      const maxZ = Math.max(...Object.values(state.windows).map((w) => w.z_index));
      return {
        ...state,
        minimizedWindows: state.minimizedWindows.filter(id => id !== windowId),
        windows: {
          ...state.windows,
          [windowId]: {
            ...state.windows[windowId],
            visible: true,
            z_index: maxZ + 1,
          },
        },
      };
    });
  },

  closeMinimizedWindow(windowId: WindowId) {
    uiStore.update((state) => ({
      ...state,
      minimizedWindows: state.minimizedWindows.filter(id => id !== windowId),
    }));
  },

  reorderMinimizedWindows(draggedId: WindowId, targetId: WindowId) {
    uiStore.update((state) => {
      const windows = [...state.minimizedWindows];
      const draggedIdx = windows.indexOf(draggedId);
      const targetIdx = windows.indexOf(targetId);
      if (draggedIdx === -1 || targetIdx === -1) {return state;}
      windows.splice(draggedIdx, 1);
      windows.splice(targetIdx, 0, draggedId);
      return { ...state, minimizedWindows: windows };
    });
  },

  minimizeAll() {
    uiStore.update((state) => {
      const visibleWindows = Object.entries(state.windows)
        .filter(([_, pos]) => pos.visible)
        .map(([id]) => id as WindowId);
      const newMinimized = [...new Set([...state.minimizedWindows, ...visibleWindows])];
      const newWindows = { ...state.windows };
      for (const id of visibleWindows) {
        newWindows[id] = { ...newWindows[id], visible: false };
      }
      return { ...state, minimizedWindows: newMinimized, windows: newWindows };
    });
  },

  restoreAll() {
    uiStore.update((state) => {
      const newWindows = { ...state.windows };
      for (const id of state.minimizedWindows) {
        if (newWindows[id]) {
          newWindows[id] = { ...newWindows[id], visible: true };
        }
      }
      return { ...state, minimizedWindows: [], windows: newWindows };
    });
  },

  // ========== TABBED WINDOWS ==========
  createTabbedGroup(windowIds: WindowId[]) {
    if (windowIds.length < 2) {return;}
    const state = get(uiStore);
    const firstWindow = state.windows[windowIds[0]];
    if (!firstWindow) {return;}

    const groupId = `tab-group-${Date.now()}`;
    const group: TabbedGroup = {
      id: groupId,
      windowIds,
      activeTab: windowIds[0],
      position: {
        x: firstWindow.x,
        y: firstWindow.y,
        width: firstWindow.width,
        height: firstWindow.height,
      },
    };

    uiStore.update((state) => ({
      ...state,
      tabbedGroups: [...state.tabbedGroups, group],
    }));
  },

  setActiveTab(groupId: string, windowId: WindowId) {
    uiStore.update((state) => ({
      ...state,
      tabbedGroups: state.tabbedGroups.map(g =>
        g.id === groupId ? { ...g, activeTab: windowId } : g
      ),
    }));
  },

  removeFromTabbedGroup(groupId: string, windowId: WindowId) {
    uiStore.update((state) => {
      const groups = state.tabbedGroups.map(g => {
        if (g.id !== groupId) {return g;}
        const newWindowIds = g.windowIds.filter(id => id !== windowId);
        if (newWindowIds.length < 2) {return null;} // Dissolve group
        return {
          ...g,
          windowIds: newWindowIds,
          activeTab: g.activeTab === windowId ? newWindowIds[0] : g.activeTab,
        };
      }).filter(Boolean) as TabbedGroup[];
      return { ...state, tabbedGroups: groups };
    });
  },

  // ========== LAYOUTS ==========
  saveLayout(name: string) {
    const state = get(uiStore);
    const layout: WindowLayout = {
      id: `layout-${Date.now()}`,
      name,
      windows: JSON.parse(JSON.stringify(state.windows)),
      tabbedGroups: JSON.parse(JSON.stringify(state.tabbedGroups)),
      createdAt: Date.now(),
      isPreset: false,
    };
    uiStore.update((state) => ({
      ...state,
      savedLayouts: [...state.savedLayouts, layout],
      activeLayoutId: layout.id,
    }));
    return layout.id;
  },

  loadLayout(layoutId: string) {
    uiStore.update((state) => {
      const layout = state.savedLayouts.find(l => l.id === layoutId);
      if (!layout) {return state;}

      // Merge layout windows with existing (preserving windows not in layout)
      const newWindows = { ...state.windows };
      for (const [id, pos] of Object.entries(layout.windows)) {
        newWindows[id as WindowId] = { ...pos };
      }
      // Hide windows not in layout
      for (const id of Object.keys(newWindows)) {
        if (!layout.windows[id as WindowId]) {
          newWindows[id as WindowId].visible = false;
        }
      }

      return {
        ...state,
        windows: newWindows,
        tabbedGroups: layout.tabbedGroups,
        activeLayoutId: layoutId,
        minimizedWindows: [],
      };
    });
  },

  deleteLayout(layoutId: string) {
    uiStore.update((state) => ({
      ...state,
      savedLayouts: state.savedLayouts.filter(l => l.id !== layoutId || l.isPreset),
      activeLayoutId: state.activeLayoutId === layoutId ? null : state.activeLayoutId,
    }));
  },

  resetToDefault() {
    uiStore.update((state) => ({
      ...initialState,
      savedLayouts: state.savedLayouts, // Preserve user layouts
      theme: state.theme,
    }));
  },

  // ========== TILING ==========
  tileWindows() {
    uiStore.update((state) => {
      const visibleIds = Object.entries(state.windows)
        .filter(([_, pos]) => pos.visible)
        .map(([id]) => id as WindowId);

      if (visibleIds.length === 0) {return state;}

      const menuBarHeight = 32;
      const statusBarHeight = 24;
      const viewportWidth = typeof window !== 'undefined' ? window.innerWidth : 1280;
      const viewportHeight = typeof window !== 'undefined' ? window.innerHeight : 800;
      const availableHeight = viewportHeight - menuBarHeight - statusBarHeight;

      // Calculate grid
      const count = visibleIds.length;
      const cols = Math.ceil(Math.sqrt(count));
      const rows = Math.ceil(count / cols);
      const tileWidth = Math.floor(viewportWidth / cols);
      const tileHeight = Math.floor(availableHeight / rows);

      const newWindows = { ...state.windows };
      visibleIds.forEach((id, i) => {
        const col = i % cols;
        const row = Math.floor(i / cols);
        newWindows[id] = {
          ...newWindows[id],
          x: col * tileWidth,
          y: menuBarHeight + row * tileHeight,
          width: tileWidth,
          height: tileHeight,
          z_index: i + 1,
        };
      });

      return { ...state, windows: newWindows };
    });
  },

  // ========== SIDEBAR / INSPECTOR ==========
  toggleSidebar() {
    uiStore.update((state) => ({ ...state, sidebarVisible: !state.sidebarVisible }));
  },

  toggleInspector() {
    uiStore.update((state) => ({ ...state, inspectorVisible: !state.inspectorVisible }));
  },

  setTheme(theme: 'dark' | 'light') {
    uiStore.update((state) => ({ ...state, theme }));
  },
};

// ============================================================================
// KEYBOARD SHORTCUTS
// ============================================================================

// Window ID to shortcut mapping (Cmd/Ctrl + number)
export const WINDOW_SHORTCUTS: Record<string, WindowId> = {
  '1': 'arrangement',
  '2': 'database',
  '3': 'mixer',
  '4': 'piano-roll',
  '5': 'pipeline',
  '6': 'midi-monitor',
  '7': 'midi-io-setup',
};

export function handleKeyboardShortcut(event: KeyboardEvent): boolean {
  const isMeta = event.metaKey || event.ctrlKey;
  if (!isMeta) {return false;}

  const key = event.key.toLowerCase();

  // Cmd+1-7: Toggle windows
  if (WINDOW_SHORTCUTS[key]) {
    event.preventDefault();
    uiActions.toggleWindow(WINDOW_SHORTCUTS[key]);
    return true;
  }

  // Cmd+M: Minimize all
  if (key === 'm' && !event.shiftKey) {
    event.preventDefault();
    uiActions.minimizeAll();
    return true;
  }

  // Cmd+Shift+M: Restore all
  if (key === 'm' && event.shiftKey) {
    event.preventDefault();
    uiActions.restoreAll();
    return true;
  }

  // Cmd+T: Tile windows
  if (key === 't' && !event.shiftKey) {
    event.preventDefault();
    uiActions.tileWindows();
    return true;
  }

  return false;
}
