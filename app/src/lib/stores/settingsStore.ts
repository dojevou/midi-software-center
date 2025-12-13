import { derived, get, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export interface AudioSettings {
  sampleRate: number;
  bufferSize: number;
  inputDevice: string;
  outputDevice: string;
  inputChannels: number;
  outputChannels: number;
  driverType: 'asio' | 'wasapi' | 'alsa' | 'coreaudio';
}

export interface MIDISettings {
  defaultInputDevice: string;
  defaultOutputDevice: string;
  midiThrough: boolean;
  recordVelocityCurve: 'linear' | 'soft' | 'hard';
  sendClock: boolean;
  receiveClock: boolean;
  syncMode: 'internal' | 'external' | 'auto';
}

export interface DisplaySettings {
  theme: 'dark' | 'light' | 'system';
  accentColor: string;
  fontSize: 'small' | 'medium' | 'large';
  showWaveforms: boolean;
  showPeakMeters: boolean;
  highContrastMode: boolean;
  reducedMotion: boolean;
}

export interface GeneralSettings {
  language: string;
  autoSave: boolean;
  autoSaveInterval: number;
  undoLevels: number;
  defaultProject: string;
  recentFilesLimit: number;
  confirmOnClose: boolean;
  showTooltips: boolean;
}

export interface LibrarySettings {
  libraryPath: string;
  watchForChanges: boolean;
  autoAnalyze: boolean;
  analysisQuality: 'fast' | 'balanced' | 'thorough';
  duplicateHandling: 'skip' | 'replace' | 'rename';
  fileOrganization: 'flat' | 'byType' | 'byDate' | 'byArtist';
}

export interface PerformanceSettings {
  enableHardwareAcceleration: boolean;
  maxCpuUsage: number;
  memoryLimit: number;
  cacheSize: number;
  preloadSamples: boolean;
  streamFromDisk: boolean;
}

export interface Settings {
  audio: AudioSettings;
  midi: MIDISettings;
  display: DisplaySettings;
  general: GeneralSettings;
  library: LibrarySettings;
  performance: PerformanceSettings;
}

export interface SettingsState {
  settings: Settings;
  pendingChanges: Partial<Settings>;
  activeTab: string;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}

// ============================================================================
// DEFAULTS
// ============================================================================

const defaultSettings: Settings = {
  audio: {
    sampleRate: 44100,
    bufferSize: 256,
    inputDevice: '',
    outputDevice: '',
    inputChannels: 2,
    outputChannels: 2,
    driverType: 'wasapi',
  },
  midi: {
    defaultInputDevice: '',
    defaultOutputDevice: '',
    midiThrough: true,
    recordVelocityCurve: 'linear',
    sendClock: false,
    receiveClock: true,
    syncMode: 'internal',
  },
  display: {
    theme: 'dark',
    accentColor: '#3b82f6',
    fontSize: 'medium',
    showWaveforms: true,
    showPeakMeters: true,
    highContrastMode: false,
    reducedMotion: false,
  },
  general: {
    language: 'en',
    autoSave: true,
    autoSaveInterval: 60,
    undoLevels: 100,
    defaultProject: '',
    recentFilesLimit: 10,
    confirmOnClose: true,
    showTooltips: true,
  },
  library: {
    libraryPath: '',
    watchForChanges: true,
    autoAnalyze: true,
    analysisQuality: 'balanced',
    duplicateHandling: 'skip',
    fileOrganization: 'flat',
  },
  performance: {
    enableHardwareAcceleration: true,
    maxCpuUsage: 80,
    memoryLimit: 4096,
    cacheSize: 1024,
    preloadSamples: true,
    streamFromDisk: false,
  },
};

// ============================================================================
// STORE
// ============================================================================

const initialState: SettingsState = {
  settings: defaultSettings,
  pendingChanges: {},
  activeTab: 'general',
  isLoading: false,
  isSaving: false,
  error: null,
};

const { subscribe, set, update } = writable<SettingsState>(initialState);

export const settingsStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const hasPendingChanges = derived(
  settingsStore,
  ($store) => Object.keys($store.pendingChanges).length > 0
);

export const currentTheme = derived(settingsStore, ($store) => $store.settings.display.theme);

export const mergedSettings = derived(settingsStore, ($store) => {
  // Deep merge settings with pending changes
  const merged: Settings = {
    audio: { ...$store.settings.audio },
    midi: { ...$store.settings.midi },
    display: { ...$store.settings.display },
    general: { ...$store.settings.general },
    library: { ...$store.settings.library },
    performance: { ...$store.settings.performance },
  };

  // Apply pending changes
  if ($store.pendingChanges.audio) {
    merged.audio = { ...merged.audio, ...$store.pendingChanges.audio };
  }
  if ($store.pendingChanges.midi) {
    merged.midi = { ...merged.midi, ...$store.pendingChanges.midi };
  }
  if ($store.pendingChanges.display) {
    merged.display = { ...merged.display, ...$store.pendingChanges.display };
  }
  if ($store.pendingChanges.general) {
    merged.general = { ...merged.general, ...$store.pendingChanges.general };
  }
  if ($store.pendingChanges.library) {
    merged.library = { ...merged.library, ...$store.pendingChanges.library };
  }
  if ($store.pendingChanges.performance) {
    merged.performance = { ...merged.performance, ...$store.pendingChanges.performance };
  }

  return merged;
});

// ============================================================================
// ACTIONS
// ============================================================================

export const settingsActions = {
  /**
   * Load settings from backend/storage
   */
  async loadSettings(): Promise<Settings> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));

      // Try to load from backend
      try {
        const settings = await safeInvoke<Settings>(Commands.GET_SETTINGS);
        if (settings) {
          update((state) => ({
            ...state,
            settings,
            isLoading: false,
          }));
          return settings;
        }
      } catch {
        // Fall back to localStorage
        const stored = localStorage.getItem('midi-center-settings');
        if (stored) {
          const settings = JSON.parse(stored);
          update((state) => ({
            ...state,
            settings: { ...defaultSettings, ...settings },
            isLoading: false,
          }));
          return settings;
        }
      }

      update((state) => ({ ...state, isLoading: false }));
      return defaultSettings;
    } catch (error) {
      console.error('Failed to load settings:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return defaultSettings;
    }
  },

  /**
   * Save settings to backend/storage
   */
  async saveSettings(): Promise<void> {
    const state = get(settingsStore);

    try {
      update((s) => ({ ...s, isSaving: true, error: null }));

      // Merge pending changes into settings
      const mergedSettings: Settings = {
        audio: { ...state.settings.audio, ...(state.pendingChanges.audio || {}) },
        midi: { ...state.settings.midi, ...(state.pendingChanges.midi || {}) },
        display: { ...state.settings.display, ...(state.pendingChanges.display || {}) },
        general: { ...state.settings.general, ...(state.pendingChanges.general || {}) },
        library: { ...state.settings.library, ...(state.pendingChanges.library || {}) },
        performance: { ...state.settings.performance, ...(state.pendingChanges.performance || {}) },
      };

      // Try to save to backend
      try {
        await safeInvoke(Commands.SAVE_SETTINGS, { settings: mergedSettings });
      } catch {
        // Fall back to localStorage
        localStorage.setItem('midi-center-settings', JSON.stringify(mergedSettings));
      }

      update((s) => ({
        ...s,
        settings: mergedSettings,
        pendingChanges: {},
        isSaving: false,
      }));
    } catch (error) {
      console.error('Failed to save settings:', error);
      update((s) => ({
        ...s,
        isSaving: false,
        error: String(error),
      }));
    }
  },

  /**
   * Reset settings to defaults
   */
  async resetToDefaults(category?: keyof Settings): Promise<void> {
    if (category) {
      update((state) => ({
        ...state,
        pendingChanges: {
          ...state.pendingChanges,
          [category]: defaultSettings[category],
        },
      }));
    } else {
      update((state) => ({
        ...state,
        pendingChanges: { ...defaultSettings },
      }));
    }
  },

  /**
   * Discard pending changes
   */
  discardChanges(): void {
    update((state) => ({ ...state, pendingChanges: {} }));
  },

  // ============================================================================
  // SETTING UPDATES
  // ============================================================================

  updateAudioSetting<K extends keyof AudioSettings>(key: K, value: AudioSettings[K]): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        audio: {
          ...state.settings.audio,
          ...(state.pendingChanges.audio || {}),
          [key]: value,
        },
      },
    }));
  },

  updateMIDISetting<K extends keyof MIDISettings>(key: K, value: MIDISettings[K]): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        midi: {
          ...state.settings.midi,
          ...(state.pendingChanges.midi || {}),
          [key]: value,
        },
      },
    }));
  },

  updateDisplaySetting<K extends keyof DisplaySettings>(key: K, value: DisplaySettings[K]): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        display: {
          ...state.settings.display,
          ...(state.pendingChanges.display || {}),
          [key]: value,
        },
      },
    }));
  },

  updateGeneralSetting<K extends keyof GeneralSettings>(key: K, value: GeneralSettings[K]): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        general: {
          ...state.settings.general,
          ...(state.pendingChanges.general || {}),
          [key]: value,
        },
      },
    }));
  },

  updateLibrarySetting<K extends keyof LibrarySettings>(key: K, value: LibrarySettings[K]): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        library: {
          ...state.settings.library,
          ...(state.pendingChanges.library || {}),
          [key]: value,
        },
      },
    }));
  },

  updatePerformanceSetting<K extends keyof PerformanceSettings>(
    key: K,
    value: PerformanceSettings[K]
  ): void {
    update((state) => ({
      ...state,
      pendingChanges: {
        ...state.pendingChanges,
        performance: {
          ...state.settings.performance,
          ...(state.pendingChanges.performance || {}),
          [key]: value,
        },
      },
    }));
  },

  // ============================================================================
  // TAB NAVIGATION
  // ============================================================================

  setActiveTab(tab: string): void {
    update((state) => ({ ...state, activeTab: tab }));
  },

  reset(): void {
    set(initialState);
  },
};

// ============================================================================
// INITIALIZATION
// ============================================================================

// Load settings on module import
if (typeof window !== 'undefined') {
  settingsActions.loadSettings();
}
