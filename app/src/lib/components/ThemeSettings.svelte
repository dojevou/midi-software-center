<script lang="ts">
  import { themeStore, themeMode } from '$lib/stores/themeStore';
  import type { ThemeConfig } from '$lib/stores/themeStore';

  let currentConfig: ThemeConfig;
  themeStore.subscribe(state => {
    currentConfig = state.config;
  });

  const fontSizeOptions: { value: ThemeConfig['fontSize']; label: string }[] = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];

  const borderRadiusOptions: { value: ThemeConfig['borderRadius']; label: string }[] = [
    { value: 'none', label: 'None' },
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];

  function handleFontSizeChange(e: Event) {
    const target = e.currentTarget as HTMLSelectElement;
    themeStore.setFontSize(target.value as ThemeConfig['fontSize']);
  }

  function handleBorderRadiusChange(e: Event) {
    const target = e.currentTarget as HTMLSelectElement;
    themeStore.setBorderRadius(target.value as ThemeConfig['borderRadius']);
  }
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
      on:change={handleFontSizeChange}
    >
      {#each fontSizeOptions as opt (opt.value)}
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
      on:change={handleBorderRadiusChange}
    >
      {#each borderRadiusOptions as opt (opt.value)}
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
