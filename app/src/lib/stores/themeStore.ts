import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

/**
 * MIDI Pattern Theme Names
 * Each theme name is an acronym following the MIDI pattern naming convention:
 * - DARK = Deep Ambient Rich Kontrast
 * - WARM = Wooden Amber Rustic Mellow
 * - NEON = Night Electric Orange Noir
 * - MINT = Modern Icy Neutral Teal
 * - ROSE = Rosy Ochre Soft Elegant
 * - BASS = Bold Abyss Slate Shadow
 */
export type ThemeName = 'DARK' | 'WARM' | 'NEON' | 'MINT' | 'ROSE' | 'BASS';

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
  themeName: ThemeName;
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

// DARK = Deep Ambient Rich Kontrast - High contrast dark theme
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

// WARM = Wooden Amber Rustic Mellow - Browns, oranges, cozy studio feel
const WARM_THEME: ThemeColors = {
  bgPrimary: '#1a1512',
  bgSecondary: '#2a221a',
  bgTertiary: '#3a3025',
  bgHover: '#4a3d30',
  bgActive: '#5a4a38',

  textPrimary: '#f5e6d3',
  textSecondary: '#c9b8a5',
  textMuted: '#8a7a68',
  textInverse: '#1a1512',

  accent: '#e8a54b',
  accentHover: '#d4923a',
  accentMuted: 'rgba(232, 165, 75, 0.25)',

  success: '#7cb342',
  warning: '#ffa726',
  error: '#e57373',
  info: '#81d4fa',

  border: '#4a3d30',
  borderFocus: '#e8a54b',

  windowHeader: '#2a221a',
  menuBar: '#1f1a15',
  statusBar: '#1f1a15',
  scrollbar: '#5a4a38',
  scrollbarHover: '#6a5a45',

  pianoRollWhiteKey: '#4a3d30',
  pianoRollBlackKey: '#1f1a15',
  pianoRollGrid: '#3a3025',
  pianoRollNote: '#e8a54b',
  pianoRollNoteSelected: '#f5c67a',
  sequencerTrackEven: '#2a221a',
  sequencerTrackOdd: '#332a20',
  waveformFg: '#e8a54b',
  waveformBg: '#1f1a15',
  vuMeterGreen: '#8bc34a',
  vuMeterYellow: '#ffc107',
  vuMeterRed: '#ef5350',
};

// NEON = Night Electric Orange Noir - Cyberpunk, bright accents on black
const NEON_THEME: ThemeColors = {
  bgPrimary: '#0a0a0f',
  bgSecondary: '#12121a',
  bgTertiary: '#1a1a25',
  bgHover: '#252530',
  bgActive: '#30303d',

  textPrimary: '#ffffff',
  textSecondary: '#b0b0c0',
  textMuted: '#606070',
  textInverse: '#0a0a0f',

  accent: '#ff6b2b',
  accentHover: '#ff8a50',
  accentMuted: 'rgba(255, 107, 43, 0.25)',

  success: '#00ff9f',
  warning: '#ffea00',
  error: '#ff1744',
  info: '#00e5ff',

  border: '#2a2a35',
  borderFocus: '#ff6b2b',

  windowHeader: '#12121a',
  menuBar: '#0d0d12',
  statusBar: '#0d0d12',
  scrollbar: '#30303d',
  scrollbarHover: '#404050',

  pianoRollWhiteKey: '#1a1a25',
  pianoRollBlackKey: '#0d0d12',
  pianoRollGrid: '#252530',
  pianoRollNote: '#ff6b2b',
  pianoRollNoteSelected: '#00e5ff',
  sequencerTrackEven: '#12121a',
  sequencerTrackOdd: '#18182a',
  waveformFg: '#00ff9f',
  waveformBg: '#0d0d12',
  vuMeterGreen: '#00ff9f',
  vuMeterYellow: '#ffea00',
  vuMeterRed: '#ff1744',
};

// MINT = Modern Icy Neutral Teal - Cool greens and teals, minimal
const MINT_THEME: ThemeColors = {
  bgPrimary: '#0f1614',
  bgSecondary: '#162220',
  bgTertiary: '#1e2e2a',
  bgHover: '#283a35',
  bgActive: '#324540',

  textPrimary: '#e8f5f2',
  textSecondary: '#a8c8c0',
  textMuted: '#5a8078',
  textInverse: '#0f1614',

  accent: '#26c6a0',
  accentHover: '#2dd4aa',
  accentMuted: 'rgba(38, 198, 160, 0.2)',

  success: '#4caf50',
  warning: '#ffb74d',
  error: '#ef5350',
  info: '#4dd0e1',

  border: '#283a35',
  borderFocus: '#26c6a0',

  windowHeader: '#162220',
  menuBar: '#121a18',
  statusBar: '#121a18',
  scrollbar: '#324540',
  scrollbarHover: '#3c5550',

  pianoRollWhiteKey: '#1e2e2a',
  pianoRollBlackKey: '#121a18',
  pianoRollGrid: '#243532',
  pianoRollNote: '#26c6a0',
  pianoRollNoteSelected: '#64ffda',
  sequencerTrackEven: '#162220',
  sequencerTrackOdd: '#1a2a28',
  waveformFg: '#26c6a0',
  waveformBg: '#121a18',
  vuMeterGreen: '#4caf50',
  vuMeterYellow: '#ffc107',
  vuMeterRed: '#ff5722',
};

// ROSE = Rosy Ochre Soft Elegant - Soft pinks and roses, refined
const ROSE_THEME: ThemeColors = {
  bgPrimary: '#18121a',
  bgSecondary: '#241c28',
  bgTertiary: '#302636',
  bgHover: '#3c3244',
  bgActive: '#483e52',

  textPrimary: '#f8e8f0',
  textSecondary: '#c8b0c0',
  textMuted: '#7a6878',
  textInverse: '#18121a',

  accent: '#e878a0',
  accentHover: '#f090b0',
  accentMuted: 'rgba(232, 120, 160, 0.2)',

  success: '#81c784',
  warning: '#ffcc80',
  error: '#e57373',
  info: '#90caf9',

  border: '#3c3244',
  borderFocus: '#e878a0',

  windowHeader: '#241c28',
  menuBar: '#1c1620',
  statusBar: '#1c1620',
  scrollbar: '#483e52',
  scrollbarHover: '#584e62',

  pianoRollWhiteKey: '#302636',
  pianoRollBlackKey: '#1c1620',
  pianoRollGrid: '#2c222e',
  pianoRollNote: '#e878a0',
  pianoRollNoteSelected: '#f8a0c0',
  sequencerTrackEven: '#241c28',
  sequencerTrackOdd: '#2a2230',
  waveformFg: '#e878a0',
  waveformBg: '#1c1620',
  vuMeterGreen: '#81c784',
  vuMeterYellow: '#ffd54f',
  vuMeterRed: '#e57373',
};

// BASS = Bold Abyss Slate Shadow - Deep blacks, minimal pro look
const BASS_THEME: ThemeColors = {
  bgPrimary: '#08080a',
  bgSecondary: '#101014',
  bgTertiary: '#18181e',
  bgHover: '#222228',
  bgActive: '#2c2c34',

  textPrimary: '#e8e8ec',
  textSecondary: '#9898a0',
  textMuted: '#505058',
  textInverse: '#08080a',

  accent: '#6366f1',
  accentHover: '#818cf8',
  accentMuted: 'rgba(99, 102, 241, 0.2)',

  success: '#10b981',
  warning: '#f59e0b',
  error: '#ef4444',
  info: '#3b82f6',

  border: '#222228',
  borderFocus: '#6366f1',

  windowHeader: '#101014',
  menuBar: '#0a0a0e',
  statusBar: '#0a0a0e',
  scrollbar: '#2c2c34',
  scrollbarHover: '#38383f',

  pianoRollWhiteKey: '#18181e',
  pianoRollBlackKey: '#0a0a0e',
  pianoRollGrid: '#1a1a20',
  pianoRollNote: '#6366f1',
  pianoRollNoteSelected: '#a5b4fc',
  sequencerTrackEven: '#101014',
  sequencerTrackOdd: '#141418',
  waveformFg: '#6366f1',
  waveformBg: '#0a0a0e',
  vuMeterGreen: '#10b981',
  vuMeterYellow: '#f59e0b',
  vuMeterRed: '#ef4444',
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

// Theme lookup map
const THEMES: Record<ThemeName, ThemeColors> = {
  DARK: DARK_THEME,
  WARM: WARM_THEME,
  NEON: NEON_THEME,
  MINT: MINT_THEME,
  ROSE: ROSE_THEME,
  BASS: BASS_THEME,
};

// Theme descriptions for UI
export const THEME_INFO: Record<ThemeName, { name: string; description: string }> = {
  DARK: { name: 'DARK', description: 'Deep Ambient Rich Kontrast' },
  WARM: { name: 'WARM', description: 'Wooden Amber Rustic Mellow' },
  NEON: { name: 'NEON', description: 'Night Electric Orange Noir' },
  MINT: { name: 'MINT', description: 'Modern Icy Neutral Teal' },
  ROSE: { name: 'ROSE', description: 'Rosy Ochre Soft Elegant' },
  BASS: { name: 'BASS', description: 'Bold Abyss Slate Shadow' },
};

const DEFAULT_CONFIG: ThemeConfig = {
  mode: 'dark',
  themeName: 'DARK',
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
  const { config } = state;
  // Use named theme from THEMES map, fallback to DARK if not found
  const themeColors = THEMES[config.themeName] || THEMES.DARK;

  // Merge with custom colors if any
  const mergedColors = config.customColors
    ? { ...themeColors, ...config.customColors }
    : themeColors;

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
    document.documentElement.setAttribute('data-theme-name', config.themeName);
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

    setTheme(themeName: ThemeName) {
      update(state => {
        const newConfig = { ...state.config, themeName };
        const newState: ThemeState = {
          ...state,
          config: newConfig,
        };
        newState.cssVariables = generateCssVariables(newState);
        applyCssVariables(newState.cssVariables);
        document.documentElement.setAttribute('data-theme-name', themeName);
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
export const themeName = derived(themeStore, $theme => $theme.config.themeName);
export const reducedMotion = derived(themeStore, $theme => $theme.config.reducedMotion);

// Export available theme names for UI
export const THEME_NAMES: ThemeName[] = ['DARK', 'WARM', 'NEON', 'MINT', 'ROSE', 'BASS'];
