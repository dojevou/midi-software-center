import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeColors {
  // Background colors
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;
  bgHover: string;
  bgActive: string;

  // Text colors
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  textInverse: string;

  // Accent colors
  accent: string;
  accentHover: string;
  accentMuted: string;

  // Status colors
  success: string;
  warning: string;
  error: string;
  info: string;

  // Border colors
  border: string;
  borderFocus: string;

  // Component-specific
  windowHeader: string;
  menuBar: string;
  statusBar: string;
  scrollbar: string;
  scrollbarHover: string;

  // DAW-specific
  pianoRollWhiteKey: string;
  pianoRollBlackKey: string;
  pianoRollGrid: string;
  pianoRollNote: string;
  pianoRollNoteSelected: string;
  sequencerTrackEven: string;
  sequencerTrackOdd: string;
  waveformFg: string;
  waveformBg: string;
  vuMeterGreen: string;
  vuMeterYellow: string;
  vuMeterRed: string;
}

export interface ThemeConfig {
  mode: ThemeMode;
  colors: {
    light: ThemeColors;
    dark: ThemeColors;
  };
  customColors?: Partial<ThemeColors>;
  fontSize: 'small' | 'medium' | 'large';
  fontFamily: string;
  borderRadius: 'none' | 'small' | 'medium' | 'large';
  reducedMotion: boolean;
  highContrast: boolean;
}

export interface ThemeState {
  config: ThemeConfig;
  resolvedMode: 'light' | 'dark'; // Actual mode after resolving 'system'
  cssVariables: Record<string, string>;
}

// ============================================================================
// DEFAULT THEMES
// ============================================================================

const DARK_THEME: ThemeColors = {
  bgPrimary: '#121212',
  bgSecondary: '#1e1e1e',
  bgTertiary: '#2a2a2a',
  bgHover: '#333333',
  bgActive: '#3a3a3a',

  textPrimary: '#ffffff',
  textSecondary: '#b3b3b3',
  textMuted: '#666666',
  textInverse: '#121212',

  accent: '#007bff',
  accentHover: '#0056b3',
  accentMuted: 'rgba(0, 123, 255, 0.2)',

  success: '#28a745',
  warning: '#ffc107',
  error: '#dc3545',
  info: '#17a2b8',

  border: '#333333',
  borderFocus: '#007bff',

  windowHeader: '#252525',
  menuBar: '#1a1a1a',
  statusBar: '#1a1a1a',
  scrollbar: '#444444',
  scrollbarHover: '#555555',

  pianoRollWhiteKey: '#3a3a3a',
  pianoRollBlackKey: '#1a1a1a',
  pianoRollGrid: '#2a2a2a',
  pianoRollNote: '#007bff',
  pianoRollNoteSelected: '#00bfff',
  sequencerTrackEven: '#1e1e1e',
  sequencerTrackOdd: '#252525',
  waveformFg: '#00ff88',
  waveformBg: '#1a1a1a',
  vuMeterGreen: '#00ff00',
  vuMeterYellow: '#ffff00',
  vuMeterRed: '#ff0000',
};

const LIGHT_THEME: ThemeColors = {
  bgPrimary: '#ffffff',
  bgSecondary: '#f5f5f5',
  bgTertiary: '#e8e8e8',
  bgHover: '#e0e0e0',
  bgActive: '#d0d0d0',

  textPrimary: '#1a1a1a',
  textSecondary: '#555555',
  textMuted: '#888888',
  textInverse: '#ffffff',

  accent: '#0066cc',
  accentHover: '#004499',
  accentMuted: 'rgba(0, 102, 204, 0.15)',

  success: '#198754',
  warning: '#fd7e14',
  error: '#dc3545',
  info: '#0dcaf0',

  border: '#d0d0d0',
  borderFocus: '#0066cc',

  windowHeader: '#e8e8e8',
  menuBar: '#f0f0f0',
  statusBar: '#f0f0f0',
  scrollbar: '#c0c0c0',
  scrollbarHover: '#a0a0a0',

  pianoRollWhiteKey: '#f8f8f8',
  pianoRollBlackKey: '#333333',
  pianoRollGrid: '#e0e0e0',
  pianoRollNote: '#0066cc',
  pianoRollNoteSelected: '#0099ff',
  sequencerTrackEven: '#ffffff',
  sequencerTrackOdd: '#f5f5f5',
  waveformFg: '#0066cc',
  waveformBg: '#f8f8f8',
  vuMeterGreen: '#00cc00',
  vuMeterYellow: '#cccc00',
  vuMeterRed: '#cc0000',
};

const DEFAULT_CONFIG: ThemeConfig = {
  mode: 'dark',
  colors: {
    light: LIGHT_THEME,
    dark: DARK_THEME,
  },
  fontSize: 'medium',
  fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, sans-serif',
  borderRadius: 'small',
  reducedMotion: false,
  highContrast: false,
};

// ============================================================================
// CSS VARIABLE GENERATION
// ============================================================================

function colorsToCssVariables(colors: ThemeColors, prefix = ''): Record<string, string> {
  const vars: Record<string, string> = {};

  // Convert camelCase to kebab-case
  const toKebab = (str: string) => str.replace(/([A-Z])/g, '-$1').toLowerCase();

  for (const [key, value] of Object.entries(colors)) {
    vars[`--${prefix}${toKebab(key)}`] = value;
  }

  return vars;
}

function generateCssVariables(state: ThemeState): Record<string, string> {
  const { config, resolvedMode } = state;
  const colors = config.colors[resolvedMode];

  // Merge with custom colors if any
  const mergedColors = config.customColors
    ? { ...colors, ...config.customColors }
    : colors;

  const vars = colorsToCssVariables(mergedColors);

  // Add font size
  const fontSizes = {
    small: { base: '12px', sm: '11px', lg: '14px', xl: '16px' },
    medium: { base: '14px', sm: '12px', lg: '16px', xl: '18px' },
    large: { base: '16px', sm: '14px', lg: '18px', xl: '20px' },
  };
  const fs = fontSizes[config.fontSize];
  vars['--font-size-base'] = fs.base;
  vars['--font-size-sm'] = fs.sm;
  vars['--font-size-lg'] = fs.lg;
  vars['--font-size-xl'] = fs.xl;

  // Add font family
  vars['--font-family'] = config.fontFamily;

  // Add border radius
  const radii = {
    none: '0px',
    small: '4px',
    medium: '8px',
    large: '12px',
  };
  vars['--border-radius'] = radii[config.borderRadius];
  vars['--border-radius-sm'] = config.borderRadius === 'none' ? '0px' : '2px';
  vars['--border-radius-lg'] = config.borderRadius === 'none' ? '0px' : radii[config.borderRadius] === '4px' ? '8px' : '16px';

  // Add animation duration (for reduced motion)
  vars['--animation-duration'] = config.reducedMotion ? '0ms' : '200ms';
  vars['--transition-duration'] = config.reducedMotion ? '0ms' : '150ms';

  return vars;
}

function applyCssVariables(vars: Record<string, string>) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

function getSystemPreference(): 'light' | 'dark' {
  if (typeof window === 'undefined') {return 'dark';}
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function getReducedMotionPreference(): boolean {
  if (typeof window === 'undefined') {return false;}
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function saveConfig(config: ThemeConfig) {
  try {
    localStorage.setItem('theme-config', JSON.stringify(config));
  } catch (e) {
    console.error('Failed to save theme config:', e);
  }
}

function enhanceContrastDark(colors: ThemeColors): ThemeColors {
  return {
    ...colors,
    textPrimary: '#ffffff',
    textSecondary: '#e0e0e0',
    border: '#555555',
    accent: '#4da6ff',
  };
}

function enhanceContrastLight(colors: ThemeColors): ThemeColors {
  return {
    ...colors,
    textPrimary: '#000000',
    textSecondary: '#333333',
    border: '#666666',
    accent: '#0044aa',
  };
}

function createThemeStore() {
  // Load saved config
  let savedConfig: Partial<ThemeConfig> = {};
  if (typeof window !== 'undefined') {
    try {
      const saved = localStorage.getItem('theme-config');
      if (saved) {
        savedConfig = JSON.parse(saved);
      }
    } catch (e) {
      console.error('Failed to load theme config:', e);
    }
  }

  const config: ThemeConfig = { ...DEFAULT_CONFIG, ...savedConfig };

  // Check system preferences
  if (config.mode === 'system' || savedConfig.reducedMotion === undefined) {
    const systemReducedMotion = getReducedMotionPreference();
    if (savedConfig.reducedMotion === undefined) {
      config.reducedMotion = systemReducedMotion;
    }
  }

  const resolvedMode = config.mode === 'system' ? getSystemPreference() : config.mode;

  const initialState: ThemeState = {
    config,
    resolvedMode,
    cssVariables: {},
  };

  initialState.cssVariables = generateCssVariables(initialState);

  const { subscribe, set, update } = writable<ThemeState>(initialState);

  // Apply initial CSS variables
  if (typeof window !== 'undefined') {
    applyCssVariables(initialState.cssVariables);
    document.documentElement.setAttribute('data-theme', resolvedMode);
  }

  // Listen for system theme changes
  if (typeof window !== 'undefined') {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', (e) => {
      update(state => {
        if (state.config.mode !== 'system') {return state;}
        const newResolved: 'dark' | 'light' = e.matches ? 'dark' : 'light';
        const newState: ThemeState = { ...state, resolvedMode: newResolved };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        document.documentElement.setAttribute('data-theme', newResolved);
        return newState;
      });
    });

    // Listen for reduced motion preference changes
    const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    motionQuery.addEventListener('change', (e) => {
      update(state => {
        const newConfig = { ...state.config, reducedMotion: e.matches };
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        return newState;
      });
    });
  }

  return {
    subscribe,

    setMode(mode: ThemeMode) {
      update(state => {
        const newConfig = { ...state.config, mode };
        const resolvedMode = mode === 'system' ? getSystemPreference() : mode;
        const newState: ThemeState = {
          config: newConfig,
          resolvedMode,
          cssVariables: {},
        };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        document.documentElement.setAttribute('data-theme', resolvedMode);
        saveConfig(newConfig);
        return newState;
      });
    },

    setFontSize(fontSize: ThemeConfig['fontSize']) {
      update(state => {
        const newConfig = { ...state.config, fontSize };
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        saveConfig(newConfig);
        return newState;
      });
    },

    setBorderRadius(borderRadius: ThemeConfig['borderRadius']) {
      update(state => {
        const newConfig = { ...state.config, borderRadius };
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        saveConfig(newConfig);
        return newState;
      });
    },

    setReducedMotion(reducedMotion: boolean) {
      update(state => {
        const newConfig = { ...state.config, reducedMotion };
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        saveConfig(newConfig);
        return newState;
      });
    },

    setHighContrast(highContrast: boolean) {
      update(state => {
        const newConfig = { ...state.config, highContrast };
        // Modify colors for high contrast if enabled
        if (highContrast) {
          newConfig.colors = {
            light: enhanceContrastLight(state.config.colors.light),
            dark: enhanceContrastDark(state.config.colors.dark),
          };
        } else {
          newConfig.colors = { light: LIGHT_THEME, dark: DARK_THEME };
        }
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        saveConfig(newConfig);
        return newState;
      });
    },

    setCustomColor(key: keyof ThemeColors, value: string) {
      update(state => {
        const customColors = { ...state.config.customColors, [key]: value };
        const newConfig = { ...state.config, customColors };
        const newState = { ...state, config: newConfig };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        saveConfig(newConfig);
        return newState;
      });
    },

    resetToDefaults() {
      const resolvedMode = DEFAULT_CONFIG.mode === 'system' ? getSystemPreference() : DEFAULT_CONFIG.mode;
      const newState: ThemeState = {
        config: { ...DEFAULT_CONFIG },
        resolvedMode,
        cssVariables: {},
      };
      newState.cssVariables = generateCssVariables(newState);
      applyCssVariables(newState.cssVariables);
      document.documentElement.setAttribute('data-theme', resolvedMode);
      localStorage.removeItem('theme-config');
      set(newState);
    },

    // Toggle between light and dark (ignoring system)
    toggle() {
      update(state => {
        const newMode = state.resolvedMode === 'dark' ? 'light' : 'dark';
        const newConfig = { ...state.config, mode: newMode as ThemeMode };
        const newState: ThemeState = {
          config: newConfig,
          resolvedMode: newMode,
          cssVariables: {},
        };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        document.documentElement.setAttribute('data-theme', newMode);
        saveConfig(newConfig);
        return newState;
      });
    },
  };
}

export const themeStore = createThemeStore();

// Derived stores
export const isDarkMode = derived(themeStore, $theme => $theme.resolvedMode === 'dark');
export const themeMode = derived(themeStore, $theme => $theme.config.mode);
export const reducedMotion = derived(themeStore, $theme => $theme.config.reducedMotion);
