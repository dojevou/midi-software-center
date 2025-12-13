/**
 * Preferences Store
 *
 * Manages application preferences including:
 * - App settings (key-value configuration)
 * - Window layouts (savable/loadable workspace configurations)
 * - Keyboard shortcuts (customizable key bindings)
 * - Recent projects (history with pinning support)
 */

import { derived, get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { Commands } from '$lib/api/commands';
import type {
  Setting,
  SettingType,
  WindowLayoutConfig,
  WindowLayoutEntry,
  KeyboardShortcut,
  RecentProject,
} from '$lib/types';

// ============================================================================
// TYPES
// ============================================================================

export interface PreferencesState {
  // App Settings
  settings: Setting[];
  settingsByCategory: Map<string, Setting[]>;

  // Window Layouts
  layouts: WindowLayoutConfig[];
  currentLayout: WindowLayoutConfig | null;

  // Keyboard Shortcuts
  shortcuts: KeyboardShortcut[];
  shortcutsByCategory: Map<string, KeyboardShortcut[]>;

  // Recent Projects
  recentProjects: RecentProject[];
  pinnedProjects: RecentProject[];

  // UI State
  activeSection: 'settings' | 'layouts' | 'shortcuts' | 'recent';
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}

// ============================================================================
// INITIAL STATE
// ============================================================================

const initialState: PreferencesState = {
  settings: [],
  settingsByCategory: new Map(),
  layouts: [],
  currentLayout: null,
  shortcuts: [],
  shortcutsByCategory: new Map(),
  recentProjects: [],
  pinnedProjects: [],
  activeSection: 'settings',
  isLoading: false,
  isSaving: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<PreferencesState>(initialState);

export const preferencesStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const settingsCategories = derived(preferencesStore, ($store) =>
  Array.from($store.settingsByCategory.keys())
);

export const settingsCount = derived(preferencesStore, ($store) => $store.settings.length);

export const shortcutCategories = derived(preferencesStore, ($store) =>
  Array.from($store.shortcutsByCategory.keys())
);

export const shortcutCount = derived(preferencesStore, ($store) => $store.shortcuts.length);

export const recentProjectCount = derived(
  preferencesStore,
  ($store) => $store.recentProjects.length
);

export const pinnedProjectCount = derived(
  preferencesStore,
  ($store) => $store.pinnedProjects.length
);

export const layoutNames = derived(preferencesStore, ($store) =>
  $store.layouts.map((l) => l.name)
);

export const layoutCount = derived(preferencesStore, ($store) => $store.layouts.length);

// ============================================================================
// ACTIONS
// ============================================================================

export const preferencesActions = {
  // ==========================================================================
  // APP SETTINGS
  // ==========================================================================

  async loadAllSettings(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const settings = await invoke<Setting[]>(Commands.SETTINGS_GET_ALL);

      // Group by category
      const byCategory = new Map<string, Setting[]>();
      for (const setting of settings) {
        const category = setting.category || 'general';
        if (!byCategory.has(category)) {
          byCategory.set(category, []);
        }
        byCategory.get(category)!.push(setting);
      }

      update((state) => ({
        ...state,
        settings,
        settingsByCategory: byCategory,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load settings:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadSettingsByCategory(category: string): Promise<Setting[]> {
    try {
      const settings = await invoke<Setting[]>(Commands.SETTINGS_GET_BY_CATEGORY, { category });
      update((state) => {
        const byCategory = new Map(state.settingsByCategory);
        byCategory.set(category, settings);
        return { ...state, settingsByCategory: byCategory };
      });
      return settings;
    } catch (error) {
      console.error('Failed to load settings by category:', error);
      return [];
    }
  },

  async getSetting(key: string): Promise<Setting | null> {
    try {
      return await invoke<Setting | null>(Commands.SETTINGS_GET, { key });
    } catch (error) {
      console.error('Failed to get setting:', error);
      return null;
    }
  },

  async setStringSetting(key: string, value: string, category?: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SETTINGS_SET_STRING, { key, value, category });
      await this.loadAllSettings();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set string setting:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setIntSetting(key: string, value: number, category?: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SETTINGS_SET_INT, { key, value, category });
      await this.loadAllSettings();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set int setting:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setFloatSetting(key: string, value: number, category?: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SETTINGS_SET_FLOAT, { key, value, category });
      await this.loadAllSettings();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set float setting:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setBoolSetting(key: string, value: boolean, category?: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SETTINGS_SET_BOOL, { key, value, category });
      await this.loadAllSettings();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set bool setting:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setJsonSetting(key: string, value: unknown, category?: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SETTINGS_SET_JSON, { key, value: JSON.stringify(value), category });
      await this.loadAllSettings();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set JSON setting:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async resetSetting(key: string): Promise<void> {
    try {
      await invoke(Commands.SETTINGS_RESET, { key });
      await this.loadAllSettings();
    } catch (error) {
      console.error('Failed to reset setting:', error);
    }
  },

  async resetCategory(category: string): Promise<void> {
    try {
      await invoke(Commands.SETTINGS_RESET_CATEGORY, { category });
      await this.loadAllSettings();
    } catch (error) {
      console.error('Failed to reset category:', error);
    }
  },

  async deleteSetting(key: string): Promise<void> {
    try {
      await invoke(Commands.SETTINGS_DELETE, { key });
      await this.loadAllSettings();
    } catch (error) {
      console.error('Failed to delete setting:', error);
    }
  },

  // ==========================================================================
  // WINDOW LAYOUTS
  // ==========================================================================

  async loadLayouts(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const layouts = await invoke<WindowLayoutConfig[]>(Commands.LAYOUTS_LIST);
      const current = await invoke<WindowLayoutConfig | null>(Commands.LAYOUTS_GET_CURRENT);
      update((state) => ({
        ...state,
        layouts,
        currentLayout: current,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load layouts:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async getLayout(name: string): Promise<WindowLayoutConfig | null> {
    try {
      return await invoke<WindowLayoutConfig | null>(Commands.LAYOUTS_GET, { name });
    } catch (error) {
      console.error('Failed to get layout:', error);
      return null;
    }
  },

  async saveLayout(name: string, windows: WindowLayoutEntry[]): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.LAYOUTS_SAVE, { name, windows });
      await this.loadLayouts();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to save layout:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async updateLayout(name: string, windows: WindowLayoutEntry[]): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.LAYOUTS_UPDATE, { name, windows });
      await this.loadLayouts();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update layout:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setDefaultLayout(name: string): Promise<void> {
    try {
      await invoke(Commands.LAYOUTS_SET_DEFAULT, { name });
      await this.loadLayouts();
    } catch (error) {
      console.error('Failed to set default layout:', error);
    }
  },

  async applyLayout(name: string): Promise<WindowLayoutConfig | null> {
    try {
      const layout = await invoke<WindowLayoutConfig | null>(Commands.LAYOUTS_APPLY, { name });
      if (layout) {
        update((state) => ({ ...state, currentLayout: layout }));
      }
      return layout;
    } catch (error) {
      console.error('Failed to apply layout:', error);
      return null;
    }
  },

  async deleteLayout(name: string): Promise<void> {
    try {
      await invoke(Commands.LAYOUTS_DELETE, { name });
      await this.loadLayouts();
    } catch (error) {
      console.error('Failed to delete layout:', error);
    }
  },

  // ==========================================================================
  // KEYBOARD SHORTCUTS
  // ==========================================================================

  async loadShortcuts(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const shortcuts = await invoke<KeyboardShortcut[]>(Commands.SHORTCUTS_LIST);

      // Group by category
      const byCategory = new Map<string, KeyboardShortcut[]>();
      for (const shortcut of shortcuts) {
        const category = shortcut.category || 'general';
        if (!byCategory.has(category)) {
          byCategory.set(category, []);
        }
        byCategory.get(category)!.push(shortcut);
      }

      update((state) => ({
        ...state,
        shortcuts,
        shortcutsByCategory: byCategory,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load shortcuts:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadShortcutsByCategory(category: string): Promise<KeyboardShortcut[]> {
    try {
      const shortcuts = await invoke<KeyboardShortcut[]>(Commands.SHORTCUTS_LIST_BY_CATEGORY, {
        category,
      });
      update((state) => {
        const byCategory = new Map(state.shortcutsByCategory);
        byCategory.set(category, shortcuts);
        return { ...state, shortcutsByCategory: byCategory };
      });
      return shortcuts;
    } catch (error) {
      console.error('Failed to load shortcuts by category:', error);
      return [];
    }
  },

  async getShortcutByAction(action: string): Promise<KeyboardShortcut | null> {
    try {
      return await invoke<KeyboardShortcut | null>(Commands.SHORTCUTS_GET_BY_ACTION, { action });
    } catch (error) {
      console.error('Failed to get shortcut by action:', error);
      return null;
    }
  },

  async getShortcutByCombo(keyCombo: string): Promise<KeyboardShortcut | null> {
    try {
      return await invoke<KeyboardShortcut | null>(Commands.SHORTCUTS_GET_BY_COMBO, {
        key_combo: keyCombo,
      });
    } catch (error) {
      console.error('Failed to get shortcut by combo:', error);
      return null;
    }
  },

  async setShortcut(action: string, keyCombo: string): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SHORTCUTS_SET, { action, key_combo: keyCombo });
      await this.loadShortcuts();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to set shortcut:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async addShortcut(
    action: string,
    keyCombo: string,
    category: string,
    description?: string
  ): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.SHORTCUTS_ADD, {
        action,
        key_combo: keyCombo,
        category,
        description,
      });
      await this.loadShortcuts();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to add shortcut:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async resetShortcut(action: string): Promise<void> {
    try {
      await invoke(Commands.SHORTCUTS_RESET, { action });
      await this.loadShortcuts();
    } catch (error) {
      console.error('Failed to reset shortcut:', error);
    }
  },

  async resetAllShortcuts(): Promise<void> {
    try {
      await invoke(Commands.SHORTCUTS_RESET_ALL);
      await this.loadShortcuts();
    } catch (error) {
      console.error('Failed to reset all shortcuts:', error);
    }
  },

  async deleteShortcut(action: string): Promise<void> {
    try {
      await invoke(Commands.SHORTCUTS_DELETE, { action });
      await this.loadShortcuts();
    } catch (error) {
      console.error('Failed to delete shortcut:', error);
    }
  },

  // ==========================================================================
  // RECENT PROJECTS
  // ==========================================================================

  async loadRecentProjects(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const projects = await invoke<RecentProject[]>(Commands.RECENT_LIST);
      const pinned = projects.filter((p) => p.is_pinned);
      update((state) => ({
        ...state,
        recentProjects: projects,
        pinnedProjects: pinned,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load recent projects:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async addRecentProject(path: string, name?: string): Promise<void> {
    try {
      await invoke(Commands.RECENT_ADD, { path, name });
      await this.loadRecentProjects();
    } catch (error) {
      console.error('Failed to add recent project:', error);
    }
  },

  async setPinned(path: string, pinned: boolean): Promise<void> {
    try {
      await invoke(Commands.RECENT_SET_PINNED, { path, pinned });
      await this.loadRecentProjects();
    } catch (error) {
      console.error('Failed to set pinned:', error);
    }
  },

  async removeRecentProject(path: string): Promise<void> {
    try {
      await invoke(Commands.RECENT_REMOVE, { path });
      await this.loadRecentProjects();
    } catch (error) {
      console.error('Failed to remove recent project:', error);
    }
  },

  async clearRecentProjects(): Promise<void> {
    try {
      await invoke(Commands.RECENT_CLEAR);
      await this.loadRecentProjects();
    } catch (error) {
      console.error('Failed to clear recent projects:', error);
    }
  },

  async clearAllRecentProjects(): Promise<void> {
    try {
      await invoke(Commands.RECENT_CLEAR_ALL);
      await this.loadRecentProjects();
    } catch (error) {
      console.error('Failed to clear all recent projects:', error);
    }
  },

  // ==========================================================================
  // UI STATE
  // ==========================================================================

  setActiveSection(section: PreferencesState['activeSection']): void {
    update((state) => ({ ...state, activeSection: section }));
  },

  clearError(): void {
    update((state) => ({ ...state, error: null }));
  },

  reset(): void {
    set(initialState);
  },

  // ==========================================================================
  // INITIALIZATION
  // ==========================================================================

  async initialize(): Promise<void> {
    await Promise.all([
      this.loadAllSettings(),
      this.loadLayouts(),
      this.loadShortcuts(),
      this.loadRecentProjects(),
    ]);
  },
};

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  // Delay initialization to avoid blocking
  setTimeout(() => {
    preferencesActions.initialize().catch(console.error);
  }, 100);
}
