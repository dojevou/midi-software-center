<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher<{
    save: AdvancedSettingsData;
    cancel: void;
  }>();

  export interface AdvancedSettingsData {
    debugLoggingEnabled: boolean;
    logFilePath: string;
    virtualMemoryPoolSize: number;
    networkTimeout: number;
    pluginSearchPaths: string[];
    experimentalFeatures: {
      newRenderer: boolean;
      betaAnalyzer: boolean;
      advancedMidi: boolean;
      gpuAcceleration: boolean;
    };
  }

  export let settings: AdvancedSettingsData = {
    debugLoggingEnabled: false,
    logFilePath: '/var/log/midi-pipeline.log',
    virtualMemoryPoolSize: 500,
    networkTimeout: 30,
    pluginSearchPaths: [],
    experimentalFeatures: {
      newRenderer: false,
      betaAnalyzer: false,
      advancedMidi: false,
      gpuAcceleration: false
    }
  };

  let localSettings: AdvancedSettingsData = JSON.parse(JSON.stringify(settings));
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let showModal: boolean = false;
  let modalAction: 'add-path' | 'remove-path' | 'view-log' | 'reset-all' | null = null;
  let selectedPathIndex: number = -1;

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleDebugLoggingToggle() {
    localSettings.debugLoggingEnabled = !localSettings.debugLoggingEnabled;
    localSettings = localSettings;
  }

  function handleVirtualMemoryChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.virtualMemoryPoolSize = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleNetworkTimeoutChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.networkTimeout = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleViewLog() {
    modalAction = 'view-log';
    showModal = true;
  }

  function handleOpenLog() {
    // In real implementation, would open log file location via Tauri
    console.log('Opening log file:', localSettings.logFilePath);
  }

  function handleAddPluginPath() {
    modalAction = 'add-path';
    showModal = true;
  }

  function handleRemovePluginPath(index: number) {
    selectedPathIndex = index;
    modalAction = 'remove-path';
    showModal = true;
  }

  function handleExperimentalFeatureToggle(feature: keyof AdvancedSettingsData['experimentalFeatures']) {
    localSettings.experimentalFeatures[feature] = !localSettings.experimentalFeatures[feature];
    localSettings = localSettings;
  }

  function handleResetToDefaults() {
    modalAction = 'reset-all';
    showModal = true;
  }

  function handleModalConfirm(event: CustomEvent<string | null>) {
    if (modalAction === 'add-path' && event.detail) {
      localSettings.pluginSearchPaths = [...localSettings.pluginSearchPaths, event.detail];
      localSettings = localSettings;
    } else if (modalAction === 'remove-path') {
      localSettings.pluginSearchPaths = localSettings.pluginSearchPaths.filter((_, i) => i !== selectedPathIndex);
      localSettings = localSettings;
    } else if (modalAction === 'reset-all') {
      handleReset();
    }
    showModal = false;
    modalAction = null;
    selectedPathIndex = -1;
  }

  function handleModalCancel() {
    showModal = false;
    modalAction = null;
    selectedPathIndex = -1;
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
    const defaults: AdvancedSettingsData = {
      debugLoggingEnabled: false,
      logFilePath: '/var/log/midi-pipeline.log',
      virtualMemoryPoolSize: 500,
      networkTimeout: 30,
      pluginSearchPaths: [],
      experimentalFeatures: {
        newRenderer: false,
        betaAnalyzer: false,
        advancedMidi: false,
        gpuAcceleration: false
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

<div class="advanced-settings">
  <div class="settings-content">
    <div class="warning-banner">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <span>Advanced settings are intended for experienced users. Incorrect values may cause instability.</span>
    </div>

    <section class="settings-section">
      <h3 class="section-title">Debug & Logging</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Debug Logging</label>
            <p class="setting-description">Enable detailed logging for troubleshooting (increases disk usage)</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.debugLoggingEnabled}
            on:click={handleDebugLoggingToggle}
            aria-label="Toggle debug logging"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <label class="setting-label">Log File Location</label>
        <div class="log-file-display">
          <svg class="file-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          <span class="log-path">{localSettings.logFilePath}</span>
          <button class="btn btn-small btn-secondary" on:click={handleViewLog}>
            View
          </button>
          <button class="btn btn-small btn-secondary" on:click={handleOpenLog}>
            Open Folder
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Memory & Network</h3>

      <div class="setting-group">
        <label class="setting-label" for="virtual-memory">
          Virtual Memory Pool Size: {localSettings.virtualMemoryPoolSize} MB
        </label>
        <input
          id="virtual-memory"
          type="range"
          class="setting-slider"
          min="100"
          max="1000"
          step="50"
          value={localSettings.virtualMemoryPoolSize}
          on:input={handleVirtualMemoryChange}
        />
        <div class="slider-labels">
          <span>100 MB</span>
          <span>550 MB</span>
          <span>1000 MB</span>
        </div>
        <p class="setting-description">Amount of memory reserved for virtual file operations</p>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="network-timeout">
          Network Timeout: {localSettings.networkTimeout} seconds
        </label>
        <input
          id="network-timeout"
          type="range"
          class="setting-slider"
          min="5"
          max="60"
          step="5"
          value={localSettings.networkTimeout}
          on:input={handleNetworkTimeoutChange}
        />
        <div class="slider-labels">
          <span>5s</span>
          <span>30s</span>
          <span>60s</span>
        </div>
        <p class="setting-description">Timeout for network operations (sync, downloads, etc.)</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Plugin Search Paths</h3>

      <div class="setting-group">
        <label class="setting-label">Additional Plugin Directories</label>
        <div class="paths-list">
          {#if localSettings.pluginSearchPaths.length === 0}
            <div class="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
              </svg>
              <p>No additional plugin paths configured</p>
            </div>
          {:else}
            {#each localSettings.pluginSearchPaths as path, index}
              <div class="path-item">
                <svg class="path-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                <span class="path-text">{path}</span>
                <button
                  class="btn btn-small btn-danger"
                  on:click={() => handleRemovePluginPath(index)}
                >
                  Remove
                </button>
              </div>
            {/each}
          {/if}
        </div>
        <button class="btn btn-secondary" on:click={handleAddPluginPath}>
          Add Path
        </button>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Experimental Features</h3>

      <div class="experimental-warning">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/>
          <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <span>These features are experimental and may be unstable</span>
      </div>

      <div class="experimental-features">
        <div class="experimental-item">
          <div class="experimental-info">
            <div class="experimental-header">
              <span class="experimental-label">New Waveform Renderer</span>
              <span class="experimental-badge">Beta</span>
            </div>
            <p class="experimental-description">Improved waveform visualization with better performance</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.experimentalFeatures.newRenderer}
            on:click={() => handleExperimentalFeatureToggle('newRenderer')}
            aria-label="Toggle new renderer"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>

        <div class="experimental-item">
          <div class="experimental-info">
            <div class="experimental-header">
              <span class="experimental-label">Beta Analyzer Engine</span>
              <span class="experimental-badge">Alpha</span>
            </div>
            <p class="experimental-description">Next-generation BPM and key detection algorithms</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.experimentalFeatures.betaAnalyzer}
            on:click={() => handleExperimentalFeatureToggle('betaAnalyzer')}
            aria-label="Toggle beta analyzer"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>

        <div class="experimental-item">
          <div class="experimental-info">
            <div class="experimental-header">
              <span class="experimental-label">Advanced MIDI Processing</span>
              <span class="experimental-badge">Beta</span>
            </div>
            <p class="experimental-description">Extended MIDI message support and real-time effects</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.experimentalFeatures.advancedMidi}
            on:click={() => handleExperimentalFeatureToggle('advancedMidi')}
            aria-label="Toggle advanced MIDI"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>

        <div class="experimental-item">
          <div class="experimental-info">
            <div class="experimental-header">
              <span class="experimental-label">GPU Acceleration</span>
              <span class="experimental-badge">Experimental</span>
            </div>
            <p class="experimental-description">Use GPU for waveform rendering and analysis (requires restart)</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.experimentalFeatures.gpuAcceleration}
            on:click={() => handleExperimentalFeatureToggle('gpuAcceleration')}
            aria-label="Toggle GPU acceleration"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Reset</h3>

      <div class="setting-group">
        <button class="btn btn-danger" on:click={handleResetToDefaults}>
          Reset All Settings to Defaults
        </button>
        <p class="setting-description">This will reset ALL application settings, not just advanced settings</p>
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

{#if showModal}
  {#if modalAction === 'add-path'}
    <SettingsModal
      title="Add Plugin Path"
      message="Enter the path to a folder containing plugins:"
      confirmText="Add"
      showInput={true}
      inputPlaceholder="/path/to/plugins"
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'remove-path'}
    <SettingsModal
      title="Remove Plugin Path"
      message="Are you sure you want to remove this plugin search path?"
      confirmText="Remove"
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'view-log'}
    <SettingsModal
      title="View Log File"
      message="Log file will open in your default text editor."
      confirmText="Open"
      showCancel={true}
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'reset-all'}
    <SettingsModal
      title="Reset All Settings"
      message="Are you sure you want to reset ALL application settings to their default values? This action cannot be undone."
      confirmText="Reset Everything"
      dangerous={true}
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {/if}
{/if}

<style>
  .advanced-settings {
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

  .warning-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: rgba(255, 170, 68, 0.1);
    border: 1px solid rgba(255, 170, 68, 0.3);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 2rem;
    color: var(--warning-color, #ffaa44);
    font-size: 0.9rem;
  }

  .warning-banner svg {
    flex-shrink: 0;
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

  .log-file-display {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .file-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .log-path {
    flex: 1;
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .paths-list {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    min-height: 100px;
    max-height: 200px;
    overflow-y: auto;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 1rem;
    text-align: center;
  }

  .empty-state svg {
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 0.75rem;
  }

  .empty-state p {
    color: var(--text-secondary, #a0a0a0);
    font-size: 0.9rem;
    margin: 0;
  }

  .path-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .path-item:last-child {
    margin-bottom: 0;
  }

  .path-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .path-text {
    flex: 1;
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .experimental-warning {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background: rgba(255, 170, 68, 0.1);
    border: 1px solid rgba(255, 170, 68, 0.3);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    margin-bottom: 1.5rem;
    color: var(--warning-color, #ffaa44);
    font-size: 0.85rem;
  }

  .experimental-warning svg {
    flex-shrink: 0;
  }

  .experimental-features {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .experimental-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 1.25rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
  }

  .experimental-info {
    flex: 1;
  }

  .experimental-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .experimental-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .experimental-badge {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 170, 68, 0.2);
    border: 1px solid rgba(255, 170, 68, 0.4);
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--warning-color, #ffaa44);
    text-transform: uppercase;
  }

  .experimental-description {
    font-size: 0.85rem;
    color: var(--text-secondary, #a0a0a0);
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

  .btn-small {
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
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

  .btn-danger {
    background: var(--error-color, #ff4444);
    color: #fff;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ee3333;
  }
</style>
