<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: DisplaySettingsData;
    cancel: void;
  }>();

  export interface DisplaySettingsData {
    windowScaling: number;
    fontSize: 'small' | 'medium' | 'large';
    gridSnap: string;
    timelineZoom: number;
    toolbarButtons: {
      import: boolean;
      export: boolean;
      analyze: boolean;
      tags: boolean;
      search: boolean;
      settings: boolean;
    };
  }

  export let settings: DisplaySettingsData = {
    windowScaling: 1,
    fontSize: 'medium',
    gridSnap: '1/16',
    timelineZoom: 100,
    toolbarButtons: {
      import: true,
      export: true,
      analyze: true,
      tags: true,
      search: true,
      settings: true
    }
  };

  let localSettings: DisplaySettingsData = JSON.parse(JSON.stringify(settings));
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;

  const gridSnapOptions = [
    { value: 'off', label: 'Off (No Grid Snap)' },
    { value: '1/16', label: '1/16 Note' },
    { value: '1/8', label: '1/8 Note' },
    { value: '1/4', label: '1/4 Note' },
    { value: '1/2', label: '1/2 Note' },
    { value: '1', label: '1 Beat' },
    { value: 'bar', label: '1 Bar' }
  ];

  const fontSizePreview = {
    small: '12px',
    medium: '14px',
    large: '16px'
  };

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);
  $: previewScale = `scale(${localSettings.windowScaling})`;

  function handleScalingChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.windowScaling = parseFloat(target.value);
    localSettings = localSettings;
  }

  function handleFontSizeChange(size: 'small' | 'medium' | 'large') {
    localSettings.fontSize = size;
    localSettings = localSettings;
  }

  function handleGridSnapChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.gridSnap = target.value;
    localSettings = localSettings;
  }

  function handleTimelineZoomChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.timelineZoom = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleToolbarToggle(button: keyof DisplaySettingsData['toolbarButtons']) {
    localSettings.toolbarButtons[button] = !localSettings.toolbarButtons[button];
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
    hasChanges = false;
    dispatch('cancel');
  }

  function handleReset() {
    const defaults: DisplaySettingsData = {
      windowScaling: 1,
      fontSize: 'medium',
      gridSnap: '1/16',
      timelineZoom: 100,
      toolbarButtons: {
        import: true,
        export: true,
        analyze: true,
        tags: true,
        search: true,
        settings: true
      }
    };
    localSettings = JSON.parse(JSON.stringify(defaults));
    localSettings = localSettings;
  }

  function showSaveIndicator() {
    saveIndicator = true;
    setTimeout(() => {
      saveIndicator = false;
    }, 2000);
  }
</script>

<div class="display-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Window Scaling</h3>

      <div class="setting-group">
        <label class="setting-label" for="scaling-slider">
          UI Scale: {(localSettings.windowScaling * 100).toFixed(0)}%
        </label>
        <input
          id="scaling-slider"
          type="range"
          class="setting-slider"
          min="1"
          max="4"
          step="0.25"
          value={localSettings.windowScaling}
          on:input={handleScalingChange}
        />
        <div class="slider-labels">
          <span>100%</span>
          <span>250%</span>
          <span>400%</span>
        </div>
        <p class="setting-description">Adjust the overall size of UI elements</p>
      </div>

      <div class="scale-preview">
        <p class="preview-label">Preview:</p>
        <div class="preview-container" style="transform: {previewScale}; transform-origin: top left;">
          <div class="preview-window">
            <div class="preview-header">
              <div class="preview-title">Window Title</div>
              <div class="preview-controls">
                <span class="preview-control"></span>
                <span class="preview-control"></span>
                <span class="preview-control"></span>
              </div>
            </div>
            <div class="preview-content">
              <div class="preview-item">Item 1</div>
              <div class="preview-item">Item 2</div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Font Size</h3>

      <div class="setting-group">
        <label class="setting-label">Text Size</label>
        <div class="font-size-selector">
          <button
            class="font-size-option"
            class:active={localSettings.fontSize === 'small'}
            on:click={() => handleFontSizeChange('small')}
          >
            <span class="font-size-preview" style="font-size: {fontSizePreview.small}">
              Aa
            </span>
            <span class="font-size-label">Small</span>
          </button>

          <button
            class="font-size-option"
            class:active={localSettings.fontSize === 'medium'}
            on:click={() => handleFontSizeChange('medium')}
          >
            <span class="font-size-preview" style="font-size: {fontSizePreview.medium}">
              Aa
            </span>
            <span class="font-size-label">Medium</span>
          </button>

          <button
            class="font-size-option"
            class:active={localSettings.fontSize === 'large'}
            on:click={() => handleFontSizeChange('large')}
          >
            <span class="font-size-preview" style="font-size: {fontSizePreview.large}">
              Aa
            </span>
            <span class="font-size-label">Large</span>
          </button>
        </div>
      </div>

      <div class="font-preview-box">
        <p style="font-size: {fontSizePreview[localSettings.fontSize]}">
          This is how text will appear in the application with the selected font size. The quick brown fox jumps over the lazy dog.
        </p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Grid & Timeline</h3>

      <div class="setting-group">
        <label class="setting-label" for="grid-snap">Grid Snap</label>
        <select
          id="grid-snap"
          class="setting-select"
          value={localSettings.gridSnap}
          on:change={handleGridSnapChange}
        >
          {#each gridSnapOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
        <p class="setting-description">Default grid snapping for timeline editing</p>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="timeline-zoom">
          Default Timeline Zoom: {localSettings.timelineZoom}%
        </label>
        <input
          id="timeline-zoom"
          type="range"
          class="setting-slider"
          min="20"
          max="500"
          step="10"
          value={localSettings.timelineZoom}
          on:input={handleTimelineZoomChange}
        />
        <div class="slider-labels">
          <span>20%</span>
          <span>260%</span>
          <span>500%</span>
        </div>
        <p class="setting-description">Initial zoom level when opening timeline view</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Toolbar Customization</h3>

      <div class="setting-group">
        <label class="setting-label">Visible Toolbar Buttons</label>
        <p class="setting-description">Choose which buttons to show in the main toolbar</p>

        <div class="toolbar-checkboxes">
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.import}
              on:change={() => handleToolbarToggle('import')}
            />
            <span>Import Files</span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.export}
              on:change={() => handleToolbarToggle('export')}
            />
            <span>Export</span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.analyze}
              on:change={() => handleToolbarToggle('analyze')}
            />
            <span>Analyze</span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.tags}
              on:change={() => handleToolbarToggle('tags')}
            />
            <span>Manage Tags</span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.search}
              on:change={() => handleToolbarToggle('search')}
            />
            <span>Search</span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={localSettings.toolbarButtons.settings}
              on:change={() => handleToolbarToggle('settings')}
            />
            <span>Settings</span>
          </label>
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
  .display-settings {
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

  .scale-preview {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem;
    margin-top: 1rem;
  }

  .preview-label {
    font-size: 0.9rem;
    color: var(--text-secondary, #a0a0a0);
    margin: 0 0 1rem 0;
  }

  .preview-container {
    transition: transform 0.3s;
  }

  .preview-window {
    width: 200px;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: var(--bg-secondary, #1e1e1e);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .preview-title {
    font-size: 0.75rem;
    color: var(--text-primary, #e0e0e0);
  }

  .preview-controls {
    display: flex;
    gap: 0.25rem;
  }

  .preview-control {
    width: 8px;
    height: 8px;
    background: var(--text-secondary, #a0a0a0);
    border-radius: 50%;
  }

  .preview-content {
    padding: 0.5rem;
  }

  .preview-item {
    padding: 0.25rem;
    font-size: 0.7rem;
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.25rem;
  }

  .font-size-selector {
    display: flex;
    gap: 1rem;
  }

  .font-size-option {
    flex: 1;
    background: var(--bg-secondary, #1e1e1e);
    border: 2px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }

  .font-size-option:hover {
    border-color: var(--border-hover, #444);
  }

  .font-size-option.active {
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .font-size-preview {
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .font-size-label {
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
  }

  .font-preview-box {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.25rem;
    margin-top: 1rem;
  }

  .font-preview-box p {
    color: var(--text-primary, #e0e0e0);
    line-height: 1.6;
    margin: 0;
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

  .toolbar-checkboxes {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.95rem;
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background 0.2s;
  }

  .checkbox-label:hover {
    background: var(--bg-hover, #2a2a2a);
  }

  .checkbox-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
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
