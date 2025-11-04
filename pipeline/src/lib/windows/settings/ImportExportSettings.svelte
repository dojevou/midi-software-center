<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: ImportExportSettingsData;
    cancel: void;
  }>();

  export interface ImportExportSettingsData {
    autoTagOnImport: boolean;
    analyzeBpmOnImport: boolean;
    analyzeKeyOnImport: boolean;
    maxNestedDepth: number;
    skipPatterns: string[];
    deleteAfterImport: boolean;
  }

  export let settings: ImportExportSettingsData = {
    autoTagOnImport: true,
    analyzeBpmOnImport: true,
    analyzeKeyOnImport: true,
    maxNestedDepth: 3,
    skipPatterns: [],
    deleteAfterImport: false
  };

  let localSettings: ImportExportSettingsData = JSON.parse(JSON.stringify(settings));
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let skipPatternsText: string = localSettings.skipPatterns.join('\n');

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleAutoTagToggle() {
    localSettings.autoTagOnImport = !localSettings.autoTagOnImport;
    localSettings = localSettings;
  }

  function handleAnalyzeBpmToggle() {
    localSettings.analyzeBpmOnImport = !localSettings.analyzeBpmOnImport;
    localSettings = localSettings;
  }

  function handleAnalyzeKeyToggle() {
    localSettings.analyzeKeyOnImport = !localSettings.analyzeKeyOnImport;
    localSettings = localSettings;
  }

  function handleNestedDepthChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.maxNestedDepth = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleSkipPatternsChange(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    skipPatternsText = target.value;
    localSettings.skipPatterns = skipPatternsText.split('\n').filter(p => p.trim());
    localSettings = localSettings;
  }

  function handleDeleteAfterImportToggle() {
    localSettings.deleteAfterImport = !localSettings.deleteAfterImport;
    localSettings = localSettings;
  }

  function handleApply() {
    dispatch('save', localSettings);
    settings = JSON.parse(JSON.stringify(localSettings));
    hasChanges = false;
    showSaveIndicator();
  }

  function handleCancel() {
    localSettings = JSON.parse(JSON.stringify(settings));
    skipPatternsText = localSettings.skipPatterns.join('\n');
    hasChanges = false;
    dispatch('cancel');
  }

  function handleReset() {
    const defaults: ImportExportSettingsData = {
      autoTagOnImport: true,
      analyzeBpmOnImport: true,
      analyzeKeyOnImport: true,
      maxNestedDepth: 3,
      skipPatterns: [],
      deleteAfterImport: false
    };
    localSettings = JSON.parse(JSON.stringify(defaults));
    skipPatternsText = '';
    localSettings = localSettings;
  }

  function showSaveIndicator() {
    saveIndicator = true;
    setTimeout(() => {
      saveIndicator = false;
    }, 2000);
  }
</script>

<div class="import-export-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Import Behavior</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Auto-Tag on Import</label>
            <p class="setting-description">Automatically analyze and tag MIDI files during import</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.autoTagOnImport}
            on:click={handleAutoTagToggle}
            aria-label="Toggle auto-tag"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Analyze BPM on Import</label>
            <p class="setting-description">Detect tempo automatically when importing files</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.analyzeBpmOnImport}
            on:click={handleAnalyzeBpmToggle}
            aria-label="Toggle BPM analysis"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Analyze Key on Import</label>
            <p class="setting-description">Detect musical key automatically when importing files</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.analyzeKeyOnImport}
            on:click={handleAnalyzeKeyToggle}
            aria-label="Toggle key analysis"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Archive Extraction</h3>

      <div class="setting-group">
        <label class="setting-label" for="nested-depth">
          Max Nested Archive Depth: {localSettings.maxNestedDepth}
        </label>
        <input
          id="nested-depth"
          type="range"
          class="setting-slider"
          min="0"
          max="5"
          step="1"
          value={localSettings.maxNestedDepth}
          on:input={handleNestedDepthChange}
        />
        <div class="slider-labels">
          <span>None (0)</span>
          <span>3 levels</span>
          <span>5 levels</span>
        </div>
        <p class="setting-description">Maximum depth for extracting nested archives (ZIP within ZIP)</p>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="skip-patterns">Skip Patterns (Regex)</label>
        <textarea
          id="skip-patterns"
          class="setting-textarea"
          rows="6"
          placeholder=".*\.tmp&#10;.*test.*&#10;.*backup.*"
          value={skipPatternsText}
          on:input={handleSkipPatternsChange}
        ></textarea>
        <p class="setting-description">One regex pattern per line. Files matching these patterns will be skipped during import.</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Post-Import Actions</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Delete Files After Import</label>
            <p class="setting-description" style="color: var(--error-color, #ff4444);">
              WARNING: Original files will be permanently deleted after successful import
            </p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.deleteAfterImport}
            on:click={handleDeleteAfterImportToggle}
            aria-label="Toggle delete after import"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    {#if localSettings.deleteAfterImport}
      <div class="warning-box">
        <svg class="warning-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/>
          <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <div>
          <strong>Danger: File Deletion Enabled</strong>
          <p>Original MIDI files will be permanently deleted after import. Make sure you have backups before enabling this option.</p>
        </div>
      </div>
    {/if}
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
  .import-export-settings {
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

  .setting-textarea {
    width: 100%;
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.9rem;
    font-family: monospace;
    resize: vertical;
    transition: all 0.2s;
  }

  .setting-textarea:focus {
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

  .warning-box {
    display: flex;
    gap: 1rem;
    background: rgba(255, 68, 68, 0.1);
    border: 1px solid rgba(255, 68, 68, 0.3);
    border-radius: 8px;
    padding: 1rem;
    margin-top: 2rem;
  }

  .warning-icon {
    color: var(--error-color, #ff4444);
    flex-shrink: 0;
  }

  .warning-box strong {
    display: block;
    color: var(--error-color, #ff4444);
    margin-bottom: 0.25rem;
    font-size: 0.95rem;
  }

  .warning-box p {
    color: var(--text-secondary, #a0a0a0);
    font-size: 0.875rem;
    margin: 0;
    line-height: 1.4;
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
