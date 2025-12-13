# TERMINAL 2: Theming System & Accessibility (a11y)

## Owner: Claude Instance #2
## Components: Dark/Light Mode Toggle, Screen Reader Support, ARIA Labels

---

## PART A: THEMING SYSTEM

### A1. Create Theme Store (`app/src/lib/stores/themeStore.ts`)

```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

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
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function getReducedMotionPreference(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
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
        if (state.config.mode !== 'system') return state;
        const newResolved = e.matches ? 'dark' : 'light';
        const newState = { ...state, resolvedMode: newResolved };
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

export const themeStore = createThemeStore();

// Derived stores
export const isDarkMode = derived(themeStore, $theme => $theme.resolvedMode === 'dark');
export const themeMode = derived(themeStore, $theme => $theme.config.mode);
export const reducedMotion = derived(themeStore, $theme => $theme.config.reducedMotion);
```

### A2. Create Theme Toggle Component (`app/src/lib/components/ThemeToggle.svelte`)

```svelte
<script lang="ts">
  import { themeStore, isDarkMode, themeMode } from '$lib/stores/themeStore';
  import type { ThemeMode } from '$lib/stores/themeStore';

  export let showLabel = false;
  export let size: 'small' | 'medium' | 'large' = 'medium';

  const sizes = {
    small: { button: '24px', icon: '14px' },
    medium: { button: '32px', icon: '18px' },
    large: { button: '40px', icon: '22px' },
  };

  function cycleMode() {
    const modes: ThemeMode[] = ['light', 'dark', 'system'];
    const currentIndex = modes.indexOf($themeMode);
    const nextMode = modes[(currentIndex + 1) % modes.length];
    themeStore.setMode(nextMode);
  }

  function getIcon(mode: ThemeMode, resolved: boolean): string {
    if (mode === 'system') return '💻';
    return resolved ? '🌙' : '☀️';
  }

  function getLabel(mode: ThemeMode): string {
    switch (mode) {
      case 'light': return 'Light mode';
      case 'dark': return 'Dark mode';
      case 'system': return 'System preference';
    }
  }
</script>

<button
  class="theme-toggle"
  class:small={size === 'small'}
  class:large={size === 'large'}
  on:click={cycleMode}
  aria-label={getLabel($themeMode)}
  title={getLabel($themeMode)}
  style="--btn-size: {sizes[size].button}; --icon-size: {sizes[size].icon}"
>
  <span class="icon" aria-hidden="true">
    {getIcon($themeMode, $isDarkMode)}
  </span>
  {#if showLabel}
    <span class="label">{getLabel($themeMode)}</span>
  {/if}
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: 4px 8px;
    min-width: var(--btn-size);
    min-height: var(--btn-size);
    cursor: pointer;
    transition: background-color var(--transition-duration) ease,
                border-color var(--transition-duration) ease;
  }

  .theme-toggle:hover {
    background: var(--bg-hover);
    border-color: var(--border-focus);
  }

  .theme-toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .icon {
    font-size: var(--icon-size);
    line-height: 1;
  }

  .label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .small {
    padding: 2px 4px;
  }

  .large {
    padding: 8px 12px;
  }
</style>
```

### A3. Create Theme Settings Panel (`app/src/lib/components/ThemeSettings.svelte`)

```svelte
<script lang="ts">
  import { themeStore, themeMode, isDarkMode, reducedMotion } from '$lib/stores/themeStore';
  import type { ThemeMode, ThemeConfig } from '$lib/stores/themeStore';

  let currentConfig: ThemeConfig;
  themeStore.subscribe(state => {
    currentConfig = state.config;
  });

  const fontSizeOptions = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];

  const borderRadiusOptions = [
    { value: 'none', label: 'None' },
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];
</script>

<div class="theme-settings" role="region" aria-label="Theme settings">
  <h3>Appearance</h3>

  <!-- Theme Mode -->
  <div class="setting-group">
    <label class="setting-label" id="theme-mode-label">Theme</label>
    <div class="theme-mode-buttons" role="radiogroup" aria-labelledby="theme-mode-label">
      <button
        class="mode-btn"
        class:active={$themeMode === 'light'}
        on:click={() => themeStore.setMode('light')}
        role="radio"
        aria-checked={$themeMode === 'light'}
      >
        <span aria-hidden="true">☀️</span>
        Light
      </button>
      <button
        class="mode-btn"
        class:active={$themeMode === 'dark'}
        on:click={() => themeStore.setMode('dark')}
        role="radio"
        aria-checked={$themeMode === 'dark'}
      >
        <span aria-hidden="true">🌙</span>
        Dark
      </button>
      <button
        class="mode-btn"
        class:active={$themeMode === 'system'}
        on:click={() => themeStore.setMode('system')}
        role="radio"
        aria-checked={$themeMode === 'system'}
      >
        <span aria-hidden="true">💻</span>
        System
      </button>
    </div>
  </div>

  <!-- Font Size -->
  <div class="setting-group">
    <label class="setting-label" for="font-size-select">Font Size</label>
    <select
      id="font-size-select"
      value={currentConfig.fontSize}
      on:change={(e) => themeStore.setFontSize(e.currentTarget.value as ThemeConfig['fontSize'])}
    >
      {#each fontSizeOptions as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>

  <!-- Border Radius -->
  <div class="setting-group">
    <label class="setting-label" for="border-radius-select">Corner Rounding</label>
    <select
      id="border-radius-select"
      value={currentConfig.borderRadius}
      on:change={(e) => themeStore.setBorderRadius(e.currentTarget.value as ThemeConfig['borderRadius'])}
    >
      {#each borderRadiusOptions as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
  </div>

  <h3>Accessibility</h3>

  <!-- Reduced Motion -->
  <div class="setting-group checkbox-group">
    <label class="checkbox-label">
      <input
        type="checkbox"
        checked={currentConfig.reducedMotion}
        on:change={(e) => themeStore.setReducedMotion(e.currentTarget.checked)}
      />
      <span>Reduce motion</span>
    </label>
    <p class="setting-description">Minimizes animations and transitions</p>
  </div>

  <!-- High Contrast -->
  <div class="setting-group checkbox-group">
    <label class="checkbox-label">
      <input
        type="checkbox"
        checked={currentConfig.highContrast}
        on:change={(e) => themeStore.setHighContrast(e.currentTarget.checked)}
      />
      <span>High contrast</span>
    </label>
    <p class="setting-description">Increases color contrast for better visibility</p>
  </div>

  <!-- Reset -->
  <div class="setting-group">
    <button class="reset-btn" on:click={() => themeStore.resetToDefaults()}>
      Reset to Defaults
    </button>
  </div>
</div>

<style>
  .theme-settings {
    padding: 16px;
  }

  h3 {
    font-size: var(--font-size-lg);
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  h3:not(:first-child) {
    margin-top: 24px;
  }

  .setting-group {
    margin-bottom: 16px;
  }

  .setting-label {
    display: block;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin-bottom: 8px;
  }

  .theme-mode-buttons {
    display: flex;
    gap: 8px;
  }

  .mode-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 12px;
    background: var(--bg-secondary);
    border: 2px solid var(--border);
    border-radius: var(--border-radius);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-duration) ease;
  }

  .mode-btn:hover {
    background: var(--bg-hover);
  }

  .mode-btn.active {
    border-color: var(--accent);
    background: var(--accent-muted);
    color: var(--text-primary);
  }

  .mode-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  select {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    color: var(--text-primary);
    font-size: var(--font-size-base);
  }

  select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    color: var(--text-primary);
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
  }

  .setting-description {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin: 0;
    padding-left: 26px;
  }

  .reset-btn {
    width: 100%;
    padding: 10px;
    background: transparent;
    border: 1px solid var(--error);
    border-radius: var(--border-radius);
    color: var(--error);
    cursor: pointer;
    transition: all var(--transition-duration) ease;
  }

  .reset-btn:hover {
    background: rgba(220, 53, 69, 0.1);
  }
</style>
```

### A4. Base CSS Variables (`app/src/app.css`)

Add to the beginning of `app.css`:

```css
/* ============================================================================
   CSS CUSTOM PROPERTIES (set by themeStore)
   ============================================================================ */

:root {
  /* Defaults (will be overridden by themeStore) */
  --bg-primary: #121212;
  --bg-secondary: #1e1e1e;
  --bg-tertiary: #2a2a2a;
  --bg-hover: #333333;
  --bg-active: #3a3a3a;

  --text-primary: #ffffff;
  --text-secondary: #b3b3b3;
  --text-muted: #666666;
  --text-inverse: #121212;

  --accent: #007bff;
  --accent-hover: #0056b3;
  --accent-muted: rgba(0, 123, 255, 0.2);

  --success: #28a745;
  --warning: #ffc107;
  --error: #dc3545;
  --info: #17a2b8;

  --border: #333333;
  --border-focus: #007bff;

  --font-size-base: 14px;
  --font-size-sm: 12px;
  --font-size-lg: 16px;
  --font-size-xl: 18px;
  --font-family: Inter, -apple-system, BlinkMacSystemFont, sans-serif;

  --border-radius: 4px;
  --border-radius-sm: 2px;
  --border-radius-lg: 8px;

  --animation-duration: 200ms;
  --transition-duration: 150ms;
}

/* Apply font family globally */
body {
  font-family: var(--font-family);
  font-size: var(--font-size-base);
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

/* Reduced motion media query fallback */
@media (prefers-reduced-motion: reduce) {
  :root {
    --animation-duration: 0ms;
    --transition-duration: 0ms;
  }
}
```

---

## PART B: ACCESSIBILITY (a11y)

### B1. Create Accessibility Store (`app/src/lib/stores/a11yStore.ts`)

```typescript
import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export interface A11yState {
  announcements: string[];
  focusTrapActive: boolean;
  focusTrapElement: HTMLElement | null;
  screenReaderMode: boolean;
  keyboardNavigationActive: boolean;
  lastFocusedElement: HTMLElement | null;
}

export interface A11yAnnouncement {
  message: string;
  priority: 'polite' | 'assertive';
  timeout?: number;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: A11yState = {
  announcements: [],
  focusTrapActive: false,
  focusTrapElement: null,
  screenReaderMode: false,
  keyboardNavigationActive: false,
  lastFocusedElement: null,
};

function createA11yStore() {
  const { subscribe, set, update } = writable<A11yState>(initialState);

  let announcementQueue: A11yAnnouncement[] = [];
  let liveRegion: HTMLElement | null = null;

  // Create live region on init
  if (typeof document !== 'undefined') {
    liveRegion = document.createElement('div');
    liveRegion.setAttribute('role', 'status');
    liveRegion.setAttribute('aria-live', 'polite');
    liveRegion.setAttribute('aria-atomic', 'true');
    liveRegion.className = 'sr-only';
    liveRegion.id = 'a11y-announcer';
    document.body.appendChild(liveRegion);
  }

  return {
    subscribe,

    // Screen reader announcements
    announce(message: string, priority: 'polite' | 'assertive' = 'polite') {
      if (!liveRegion) return;

      liveRegion.setAttribute('aria-live', priority);
      // Clear then set to trigger announcement
      liveRegion.textContent = '';
      requestAnimationFrame(() => {
        if (liveRegion) {
          liveRegion.textContent = message;
        }
      });

      update(state => ({
        ...state,
        announcements: [...state.announcements.slice(-9), message],
      }));
    },

    // Focus management
    trapFocus(element: HTMLElement) {
      update(state => ({
        ...state,
        focusTrapActive: true,
        focusTrapElement: element,
        lastFocusedElement: document.activeElement as HTMLElement,
      }));

      // Focus first focusable element
      const focusable = getFocusableElements(element);
      if (focusable.length > 0) {
        focusable[0].focus();
      }

      // Add trap handler
      element.addEventListener('keydown', handleFocusTrap);
    },

    releaseFocus() {
      update(state => {
        if (state.focusTrapElement) {
          state.focusTrapElement.removeEventListener('keydown', handleFocusTrap);
        }
        if (state.lastFocusedElement) {
          state.lastFocusedElement.focus();
        }
        return {
          ...state,
          focusTrapActive: false,
          focusTrapElement: null,
        };
      });
    },

    // Skip link support
    skipToContent(targetId: string) {
      const target = document.getElementById(targetId);
      if (target) {
        target.setAttribute('tabindex', '-1');
        target.focus();
        target.scrollIntoView({ behavior: 'smooth' });
      }
    },

    // Keyboard navigation detection
    enableKeyboardMode() {
      update(state => ({ ...state, keyboardNavigationActive: true }));
      document.body.classList.add('keyboard-navigation');
    },

    disableKeyboardMode() {
      update(state => ({ ...state, keyboardNavigationActive: false }));
      document.body.classList.remove('keyboard-navigation');
    },

    // Screen reader mode toggle
    setScreenReaderMode(enabled: boolean) {
      update(state => ({ ...state, screenReaderMode: enabled }));
      if (enabled) {
        document.body.classList.add('screen-reader-mode');
      } else {
        document.body.classList.remove('screen-reader-mode');
      }
    },
  };
}

// Focus trap handler
function handleFocusTrap(event: KeyboardEvent) {
  if (event.key !== 'Tab') return;

  const element = event.currentTarget as HTMLElement;
  const focusable = getFocusableElements(element);
  if (focusable.length === 0) return;

  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

// Get focusable elements within container
function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  return Array.from(container.querySelectorAll(selector)) as HTMLElement[];
}

export const a11yStore = createA11yStore();

// Derived stores
export const isKeyboardNav = derived(a11yStore, $a11y => $a11y.keyboardNavigationActive);
export const isFocusTrapped = derived(a11yStore, $a11y => $a11y.focusTrapActive);
```

### B2. Create Screen Reader Only Component (`app/src/lib/components/ScreenReaderOnly.svelte`)

```svelte
<script lang="ts">
  export let tag: 'span' | 'div' | 'p' | 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' = 'span';
</script>

<svelte:element this={tag} class="sr-only">
  <slot />
</svelte:element>

<style>
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
```

### B3. Create Skip Links Component (`app/src/lib/components/SkipLinks.svelte`)

```svelte
<script lang="ts">
  import { a11yStore } from '$lib/stores/a11yStore';

  interface SkipLink {
    id: string;
    label: string;
  }

  export let links: SkipLink[] = [
    { id: 'main-content', label: 'Skip to main content' },
    { id: 'main-navigation', label: 'Skip to navigation' },
  ];
</script>

<nav class="skip-links" aria-label="Skip navigation">
  {#each links as link}
    <a
      href="#{link.id}"
      class="skip-link"
      on:click|preventDefault={() => a11yStore.skipToContent(link.id)}
    >
      {link.label}
    </a>
  {/each}
</nav>

<style>
  .skip-links {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 100001;
  }

  .skip-link {
    position: absolute;
    top: -100%;
    left: 0;
    padding: 12px 24px;
    background: var(--accent);
    color: var(--text-inverse);
    text-decoration: none;
    font-weight: 600;
    border-radius: 0 0 var(--border-radius) 0;
    transition: top 0.2s ease;
  }

  .skip-link:focus {
    top: 0;
    outline: 3px solid var(--warning);
    outline-offset: 2px;
  }
</style>
```

### B4. Create Accessible Button Component (`app/src/lib/components/AccessibleButton.svelte`)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'secondary';
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let disabled = false;
  export let loading = false;
  export let ariaLabel: string | undefined = undefined;
  export let ariaDescribedby: string | undefined = undefined;
  export let ariaExpanded: boolean | undefined = undefined;
  export let ariaHaspopup: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog' | undefined = undefined;
  export let ariaPressed: boolean | undefined = undefined;
  export let type: 'button' | 'submit' | 'reset' = 'button';

  const dispatch = createEventDispatcher<{ click: MouseEvent }>();

  function handleClick(event: MouseEvent) {
    if (!disabled && !loading) {
      dispatch('click', event);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const mouseEvent = new MouseEvent('click', { bubbles: true });
      handleClick(mouseEvent);
    }
  }
</script>

<button
  {type}
  class="accessible-button {variant} {size}"
  class:loading
  {disabled}
  aria-label={ariaLabel}
  aria-describedby={ariaDescribedby}
  aria-expanded={ariaExpanded}
  aria-haspopup={ariaHaspopup}
  aria-pressed={ariaPressed}
  aria-busy={loading}
  aria-disabled={disabled}
  on:click={handleClick}
  on:keydown={handleKeyDown}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
    <span class="sr-only">Loading...</span>
  {/if}
  <span class="content" class:hidden={loading}>
    <slot />
  </span>
</button>

<style>
  .accessible-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: none;
    border-radius: var(--border-radius);
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--transition-duration) ease,
                transform var(--transition-duration) ease,
                box-shadow var(--transition-duration) ease;
    position: relative;
  }

  .accessible-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .accessible-button:active:not(:disabled) {
    transform: scale(0.98);
  }

  .accessible-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Sizes */
  .small {
    padding: 4px 8px;
    font-size: var(--font-size-sm);
  }

  .medium {
    padding: 8px 16px;
    font-size: var(--font-size-base);
  }

  .large {
    padding: 12px 24px;
    font-size: var(--font-size-lg);
  }

  /* Variants */
  .primary {
    background: var(--accent);
    color: white;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-focus);
  }

  .ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .ghost:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .danger {
    background: var(--error);
    color: white;
  }

  .danger:hover:not(:disabled) {
    background: #c82333;
  }

  /* Loading state */
  .loading {
    pointer-events: none;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid transparent;
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .content.hidden {
    visibility: hidden;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
```

### B5. Create Focus Visible Styles (`app/src/app.css`)

Add to `app.css`:

```css
/* ============================================================================
   ACCESSIBILITY STYLES
   ============================================================================ */

/* Screen reader only class */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

/* Focus visible for keyboard navigation */
:focus {
  outline: none;
}

:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* Keyboard navigation mode - show focus rings on everything */
body.keyboard-navigation *:focus {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* Hide focus rings when using mouse */
body:not(.keyboard-navigation) *:focus:not(:focus-visible) {
  outline: none;
}

/* High contrast mode enhancements */
@media (prefers-contrast: high) {
  :root {
    --border: #888888;
    --text-secondary: #cccccc;
  }

  button, input, select, textarea {
    border-width: 2px;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* Screen reader mode - extra visual indicators */
body.screen-reader-mode [aria-label]::after {
  content: ' [' attr(aria-label) ']';
  font-size: 10px;
  color: var(--accent);
}

/* Focus trap visual indicator */
[data-focus-trap="true"] {
  box-shadow: 0 0 0 3px var(--accent-muted);
}
```

### B6. Initialize Keyboard Detection in App.svelte

Add to `App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { a11yStore } from '$lib/stores/a11yStore';
  import SkipLinks from '$lib/components/SkipLinks.svelte';

  onMount(() => {
    // Detect keyboard vs mouse navigation
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Tab') {
        a11yStore.enableKeyboardMode();
      }
    };

    const handleMouseDown = () => {
      a11yStore.disableKeyboardMode();
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('mousedown', handleMouseDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('mousedown', handleMouseDown);
    };
  });
</script>

<SkipLinks />
<!-- Rest of your app -->
<main id="main-content" tabindex="-1">
  <!-- Main content -->
</main>
```

---

## TESTING CHECKLIST

### Theming
- [ ] Dark mode applies correctly
- [ ] Light mode applies correctly
- [ ] System preference detection works
- [ ] Theme persists across sessions
- [ ] Font size changes apply globally
- [ ] Border radius changes apply globally
- [ ] Reduced motion disables animations
- [ ] High contrast mode increases visibility
- [ ] Reset to defaults works

### Accessibility
- [ ] Skip links are visible on focus
- [ ] Skip links navigate correctly
- [ ] Screen reader announcements work
- [ ] Focus trap works in modals
- [ ] Keyboard navigation mode detected
- [ ] Focus visible on keyboard nav
- [ ] ARIA labels on interactive elements
- [ ] Color contrast meets WCAG AA
- [ ] Reduced motion respected

---

## FILES TO CREATE/MODIFY

| File | Action |
|------|--------|
| `app/src/lib/stores/themeStore.ts` | CREATE |
| `app/src/lib/stores/a11yStore.ts` | CREATE |
| `app/src/lib/components/ThemeToggle.svelte` | CREATE |
| `app/src/lib/components/ThemeSettings.svelte` | CREATE |
| `app/src/lib/components/ScreenReaderOnly.svelte` | CREATE |
| `app/src/lib/components/SkipLinks.svelte` | CREATE |
| `app/src/lib/components/AccessibleButton.svelte` | CREATE |
| `app/src/app.css` | MODIFY - Add theme variables & a11y styles |
| `app/src/App.svelte` | MODIFY - Add keyboard detection & skip links |
| `app/src/lib/stores/index.ts` | MODIFY - Export new stores |
