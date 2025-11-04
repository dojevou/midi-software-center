/**
 * Window Store - Grown-up Script
 *
 * Manages window state and provides Tauri IPC integration for window management.
 * Handles showing/hiding windows, saving/loading layouts, and arranging windows.
 */

import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types matching Rust WindowInfo structure
export interface Position {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized: boolean;
}

export interface Docking {
  docked_to: string | null;
  side: 'left' | 'right' | 'top' | 'bottom';
  size_ratio: number;
}

export interface WindowInfo {
  label: string;
  title: string;
  window_type: 'main' | 'dockable' | 'floating' | 'modal' | 'palette';
  position: Position;
  docking: Docking;
  visible: boolean;
  resizable: boolean;
  closeable: boolean;
  created_at: number;
}

interface WindowStoreState {
  windows: Map<string, WindowInfo>;
  currentLayout: string;
  loading: boolean;
  error: string | null;
}

function createWindowStore() {
  const { subscribe, update, set } = writable<WindowStoreState>({
    windows: new Map(),
    currentLayout: 'default',
    loading: false,
    error: null,
  });

  return {
    subscribe,

    // Show a window by label
    showWindow: async (label: string): Promise<void> => {
      try {
        await invoke<void>('show_window', { label });
        update(state => {
          const window = state.windows.get(label);
          if (window) {
            window.visible = true;
            state.windows.set(label, window);
          }
          return { ...state, error: null };
        });
      } catch (error) {
        console.error('Failed to show window:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Hide a window by label
    hideWindow: async (label: string): Promise<void> => {
      try {
        await invoke<void>('hide_window', { label });
        update(state => {
          const window = state.windows.get(label);
          if (window) {
            window.visible = false;
            state.windows.set(label, window);
          }
          return { ...state, error: null };
        });
      } catch (error) {
        console.error('Failed to hide window:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Toggle window visibility
    toggleWindow: async (label: string): Promise<void> => {
      try {
        await invoke<void>('toggle_window', { label });
        update(state => {
          const window = state.windows.get(label);
          if (window) {
            window.visible = !window.visible;
            state.windows.set(label, window);
          }
          return { ...state, error: null };
        });
      } catch (error) {
        console.error('Failed to toggle window:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Save current layout with a name
    saveLayout: async (name: string): Promise<void> => {
      try {
        update(state => ({ ...state, loading: true }));
        await invoke<void>('save_layout', { name });
        update(state => ({
          ...state,
          currentLayout: name,
          loading: false,
          error: null
        }));
      } catch (error) {
        console.error('Failed to save layout:', error);
        update(state => ({ ...state, loading: false, error: String(error) }));
        throw error;
      }
    },

    // Load a saved layout by name
    loadLayout: async (name: string): Promise<void> => {
      try {
        update(state => ({ ...state, loading: true }));
        await invoke<void>('load_layout', { name });

        // Refresh window list after loading
        const windows = await invoke<WindowInfo[]>('get_all_windows');
        const windowsMap = new Map(windows.map(w => [w.label, w]));

        update(state => ({
          ...state,
          windows: windowsMap,
          currentLayout: name,
          loading: false,
          error: null,
        }));
      } catch (error) {
        console.error('Failed to load layout:', error);
        update(state => ({ ...state, loading: false, error: String(error) }));
        throw error;
      }
    },

    // Get list of available saved layouts
    getLayoutList: async (): Promise<string[]> => {
      try {
        const layouts = await invoke<string[]>('get_layout_list');
        return layouts;
      } catch (error) {
        console.error('Failed to get layout list:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Delete a saved layout
    deleteLayout: async (name: string): Promise<void> => {
      try {
        await invoke<void>('delete_layout', { name });
        update(state => ({ ...state, error: null }));
      } catch (error) {
        console.error('Failed to delete layout:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Arrange windows using different layout patterns
    arrangeWindows: async (type: 'tile_h' | 'tile_v' | 'cascade'): Promise<void> => {
      try {
        update(state => ({ ...state, loading: true }));
        await invoke<void>('arrange_windows', { arrangement: type });

        // Refresh window positions after arranging
        const windows = await invoke<WindowInfo[]>('get_all_windows');
        const windowsMap = new Map(windows.map(w => [w.label, w]));

        update(state => ({
          ...state,
          windows: windowsMap,
          loading: false,
          error: null,
        }));
      } catch (error) {
        console.error('Failed to arrange windows:', error);
        update(state => ({ ...state, loading: false, error: String(error) }));
        throw error;
      }
    },

    // Get all windows
    getAllWindows: async (): Promise<WindowInfo[]> => {
      try {
        update(state => ({ ...state, loading: true }));
        const windows = await invoke<WindowInfo[]>('get_all_windows');
        const windowsMap = new Map(windows.map(w => [w.label, w]));

        update(state => ({
          ...state,
          windows: windowsMap,
          loading: false,
          error: null,
        }));

        return windows;
      } catch (error) {
        console.error('Failed to get all windows:', error);
        update(state => ({ ...state, loading: false, error: String(error) }));
        throw error;
      }
    },

    // Get only visible windows
    getVisibleWindows: async (): Promise<WindowInfo[]> => {
      try {
        const windows = await invoke<WindowInfo[]>('get_visible_windows');
        return windows;
      } catch (error) {
        console.error('Failed to get visible windows:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Get total window count
    getWindowCount: async (): Promise<number> => {
      try {
        const count = await invoke<number>('get_window_count');
        return count;
      } catch (error) {
        console.error('Failed to get window count:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Get currently focused window
    getFocusedWindow: async (): Promise<WindowInfo | null> => {
      try {
        const window = await invoke<WindowInfo | null>('get_focused_window');
        return window;
      } catch (error) {
        console.error('Failed to get focused window:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Set focused window by label
    setFocusedWindow: async (label: string): Promise<void> => {
      try {
        await invoke<void>('set_focused_window', { label });
        update(state => ({ ...state, error: null }));
      } catch (error) {
        console.error('Failed to set focused window:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Get current layout name
    getCurrentLayout: async (): Promise<string> => {
      try {
        const layout = await invoke<string>('get_current_layout');
        update(state => ({ ...state, currentLayout: layout, error: null }));
        return layout;
      } catch (error) {
        console.error('Failed to get current layout:', error);
        update(state => ({ ...state, error: String(error) }));
        throw error;
      }
    },

    // Clear error state
    clearError: () => {
      update(state => ({ ...state, error: null }));
    },
  };
}

export const windowStore = createWindowStore();
