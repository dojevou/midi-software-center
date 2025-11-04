<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: GeneralSettingsData;
    cancel: void;
  }>();

  export interface GeneralSettingsData {
    theme: 'dark' | 'light';
    language: string;
    autoSave: boolean;
    autoSaveInterval: number;
    checkUpdates: boolean;
    showSplashScreen: boolean;
    openLastProject: boolean;
  }

  export let settings: GeneralSettingsData = {
    theme: 'dark',
    language: 'en',
    autoSave: true,
    autoSaveInterval: 5,
    checkUpdates: true,
    showSplashScreen: true,
    openLastProject: true
  };

  let localSettings: GeneralSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;

  const languages = [
    { value: 'en', label: 'English' },
    { value: 'es', label: 'Spanish' },
    { value: 'fr', label: 'French' },
    { value: 'de', label: 'German' },
    { value: 'ja', label: 'Japanese' }
  ];

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleThemeChange(theme: 'dark' | 'light') {
    localSettings.theme = theme;
    localSettings = localSettings;
  }

  function handleLanguageChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.language = target.value;
    localSettings = localSettings;
  }

  function handleAutoSaveToggle() {
    localSettings.autoSave = !localSettings.autoSave;
    localSettings = localSettings;
  }

  function handleIntervalChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.autoSaveInterval = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleCheckUpdatesToggle() {
    localSettings.checkUpdates = !localSettings.checkUpdates;
    localSettings = localSettings;
  }

  function handleSplashToggle() {
    localSettings.showSplashScreen = !localSettings.showSplashScreen;
    localSettings = localSettings;
  }

  function handleLastProjectToggle() {
    localSettings.openLastProject = !localSettings.openLastProject;
    localSettings = localSettings;
  }

  function handleApply() {
    dispatch('save', localSettings);
    settings = { ...localSettings };
    hasChanges = false;
    showSaveIndicator();
  }

  function handleCancel() {
    localSettings = { ...settings };
    hasChanges = false;
    dispatch('cancel');
  }

  function handleReset() {
    const defaults: GeneralSettingsData = {
      theme: 'dark',
      language: 'en',
      autoSave: true,
      autoSaveInterval: 5,
      checkUpdates: true,
      showSplashScreen: true,
      openLastProject: true
    };
    localSettings = { ...defaults };
    localSettings = localSettings;
  }

  function showSaveIndicator() {
    saveIndicator = true;
    setTimeout(() => {
      saveIndicator = false;
    }, 2000);
  }
</script>

<div class="general-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Appearance</h3>

      <div class="setting-group">
        <label class="setting-label">Theme</label>
        <div class="theme-selector">
          <button
            class="theme-option"
            class:active={localSettings.theme === 'dark'}
            on:click={() => handleThemeChange('dark')}
          >
            <div class="theme-preview dark-preview">
              <div class="preview-header"></div>
              <div class="preview-content">
                <div class="preview-sidebar"></div>
                <div class="preview-main"></div>
              </div>
            </div>
            <span class="theme-label">Dark</span>
          </button>

          <button
            class="theme-option"
            class:active={localSettings.theme === 'light'}
            on:click={() => handleThemeChange('light')}
          >
            <div class="theme-preview light-preview">
              <div class="preview-header"></div>
              <div class="preview-content">
                <div class="preview-sidebar"></div>
                <div class="preview-main"></div>
              </div>
            </div>
            <span class="theme-label">Light</span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="language-select">Language</label>
        <select
          id="language-select"
          class="setting-select"
          value={localSettings.language}
          on:change={handleLanguageChange}
        >
          {#each languages as lang}
            <option value={lang.value}>{lang.label}</option>
          {/each}
        </select>
        <p class="setting-description">Application restart required to apply language changes</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Auto-Save</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Enable Auto-Save</label>
            <p class="setting-description">Automatically save your work at regular intervals</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.autoSave}
            on:click={handleAutoSaveToggle}
            aria-label="Toggle auto-save"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      {#if localSettings.autoSave}
        <div class="setting-group">
          <label class="setting-label" for="interval-slider">
            Auto-Save Interval: {localSettings.autoSaveInterval} {localSettings.autoSaveInterval === 1 ? 'minute' : 'minutes'}
          </label>
          <input
            id="interval-slider"
            type="range"
            class="setting-slider"
            min="1"
            max="60"
            step="1"
            value={localSettings.autoSaveInterval}
            on:input={handleIntervalChange}
          />
          <div class="slider-labels">
            <span>1 min</span>
            <span>30 min</span>
            <span>60 min</span>
          </div>
        </div>
      {/if}
    </section>

    <section class="settings-section">
      <h3 class="section-title">Updates</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Check for Updates</label>
            <p class="setting-description">Automatically check for software updates on startup</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.checkUpdates}
            on:click={handleCheckUpdatesToggle}
            aria-label="Toggle update checks"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Startup Behavior</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Show Splash Screen</label>
            <p class="setting-description">Display splash screen while application loads</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.showSplashScreen}
            on:click={handleSplashToggle}
            aria-label="Toggle splash screen"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Open Last Project</label>
            <p class="setting-description">Automatically open the last project on startup</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.openLastProject}
            on:click={handleLastProjectToggle}
            aria-label="Toggle open last project"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>
  </div>

  <div class="settings-footer">
    <div class="footer-left">
      {#if saveIndicator}
        <div class="save-indicator">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M20 6L9 17l-5-5"/>
          </svg>
          <span>Settings saved</span>
        </div>
      {/if}
    </div>
    <div class="footer-right">
      <button class="btn btn-secondary" on:click={handleReset}>
        Reset to Defaults
      </button>
      <button class="btn btn-secondary" on:click={handleCancel}>
        Cancel
      </button>
      <button
        class="btn btn-primary"
        disabled={!hasChanges}
        on:click={handleApply}
      >
        Apply
      </button>
    </div>
  </div>
</div>

<style>
  .general-settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #252525);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  .settings-section {
    margin-bottom: 2.5rem;
  }

  .section-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin: 0 0 1.5rem 0;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .setting-group {
    margin-bottom: 1.5rem;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .setting-label {
    display: block;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 0.5rem;
  }

  .setting-description {
    font-size: 0.875rem;
    color: var(--text-secondary, #a0a0a0);
    margin: 0.25rem 0 0 0;
    line-height: 1.4;
  }

  .theme-selector {
    display: flex;
    gap: 1rem;
  }

  .theme-option {
    flex: 1;
    background: var(--bg-secondary, #1e1e1e);
    border: 2px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .theme-option:hover {
    border-color: var(--border-hover, #444);
  }

  .theme-option.active {
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .theme-preview {
    width: 100%;
    height: 120px;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.75rem;
    border: 1px solid rgba(0, 0, 0, 0.2);
  }

  .dark-preview {
    background: #1a1a1a;
  }

  .light-preview {
    background: #f5f5f5;
  }

  .preview-header {
    height: 20%;
  }

  .dark-preview .preview-header {
    background: #2a2a2a;
    border-bottom: 1px solid #333;
  }

  .light-preview .preview-header {
    background: #fff;
    border-bottom: 1px solid #ddd;
  }

  .preview-content {
    display: flex;
    height: 80%;
  }

  .preview-sidebar {
    width: 30%;
  }

  .dark-preview .preview-sidebar {
    background: #252525;
    border-right: 1px solid #333;
  }

  .light-preview .preview-sidebar {
    background: #eee;
    border-right: 1px solid #ddd;
  }

  .preview-main {
    flex: 1;
  }

  .dark-preview .preview-main {
    background: #1a1a1a;
  }

  .light-preview .preview-main {
    background: #f5f5f5;
  }

  .theme-label {
    display: block;
    text-align: center;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .setting-select {
    width: 100%;
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .setting-select:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .toggle-btn {
    position: relative;
    width: 48px;
    height: 24px;
    background: var(--bg-tertiary, #333);
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }

  .toggle-btn.active {
    background: var(--accent-color, #4a9eff);
  }

  .toggle-slider {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: #fff;
    border-radius: 10px;
    transition: transform 0.2s;
  }

  .toggle-btn.active .toggle-slider {
    transform: translateX(24px);
  }

  .setting-slider {
    width: 100%;
    height: 6px;
    background: var(--bg-tertiary, #333);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
  }

  .setting-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent-color, #4a9eff);
    border-radius: 50%;
    cursor: pointer;
    transition: all 0.2s;
  }

  .setting-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .setting-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--accent-color, #4a9eff);
    border: none;
    border-radius: 50%;
    cursor: pointer;
    transition: all 0.2s;
  }

  .setting-slider::-moz-range-thumb:hover {
    transform: scale(1.2);
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .settings-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    background: var(--bg-secondary, #1e1e1e);
    border-top: 1px solid var(--border-color, #333);
  }

  .footer-left {
    min-width: 150px;
  }

  .footer-right {
    display: flex;
    gap: 0.75rem;
  }

  .save-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--success-color, #44ff44);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--bg-tertiary, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover, #3a3a3a);
  }

  .btn-primary {
    background: var(--accent-color, #4a9eff);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3a8eef;
  }
</style>
