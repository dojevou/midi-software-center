<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: SyncSettingsData;
    cancel: void;
  }>();

  export interface SyncSettingsData {
    cloudSyncEnabled: boolean;
    autoSyncInterval: 'manual' | '5min' | '15min' | '1hour';
    syncProjects: boolean;
    syncLoops: boolean;
    syncPresets: boolean;
  }

  export let settings: SyncSettingsData = {
    cloudSyncEnabled: false,
    autoSyncInterval: 'manual',
    syncProjects: true,
    syncLoops: true,
    syncPresets: true
  };

  let localSettings: SyncSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let syncStatus: 'synced' | 'syncing' | 'error' = 'synced';
  let lastSyncTime: string = '2 hours ago';
  let isSyncing: boolean = false;

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleCloudSyncToggle() {
    localSettings.cloudSyncEnabled = !localSettings.cloudSyncEnabled;
    localSettings = localSettings;
  }

  function handleAutoSyncIntervalChange(interval: 'manual' | '5min' | '15min' | '1hour') {
    localSettings.autoSyncInterval = interval;
    localSettings = localSettings;
  }

  function handleSyncProjectsToggle() {
    localSettings.syncProjects = !localSettings.syncProjects;
    localSettings = localSettings;
  }

  function handleSyncLoopsToggle() {
    localSettings.syncLoops = !localSettings.syncLoops;
    localSettings = localSettings;
  }

  function handleSyncPresetsToggle() {
    localSettings.syncPresets = !localSettings.syncPresets;
    localSettings = localSettings;
  }

  async function handleManualSync() {
    isSyncing = true;
    syncStatus = 'syncing';
    // In real implementation, would trigger sync via Tauri
    await new Promise(resolve => setTimeout(resolve, 2000));
    syncStatus = 'synced';
    lastSyncTime = 'Just now';
    isSyncing = false;
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
    const defaults: SyncSettingsData = {
      cloudSyncEnabled: false,
      autoSyncInterval: 'manual',
      syncProjects: true,
      syncLoops: true,
      syncPresets: true
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

<div class="sync-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Cloud Sync</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Enable Cloud Sync</label>
            <p class="setting-description">Automatically sync your data to the cloud</p>
          </div>
          <div class="sync-status-toggle">
            <div class="status-indicator" class:synced={syncStatus === 'synced'} class:syncing={syncStatus === 'syncing'} class:error={syncStatus === 'error'}></div>
            <button
              class="toggle-btn"
              class:active={localSettings.cloudSyncEnabled}
              on:click={handleCloudSyncToggle}
              aria-label="Toggle cloud sync"
            >
              <span class="toggle-slider"></span>
            </button>
          </div>
        </div>
      </div>

      {#if localSettings.cloudSyncEnabled}
        <div class="sync-info-box">
          <div class="sync-info-row">
            <span class="sync-info-label">Status:</span>
            <span class="sync-info-value" class:synced={syncStatus === 'synced'} class:syncing={syncStatus === 'syncing'} class:error={syncStatus === 'error'}>
              {#if syncStatus === 'synced'}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 6L9 17l-5-5"/>
                </svg>
                Synced
              {:else if syncStatus === 'syncing'}
                <svg class="spinning" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                </svg>
                Syncing...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="15" y1="9" x2="9" y2="15"/>
                  <line x1="9" y1="9" x2="15" y2="15"/>
                </svg>
                Error
              {/if}
            </span>
          </div>
          <div class="sync-info-row">
            <span class="sync-info-label">Last Sync:</span>
            <span class="sync-info-value">{lastSyncTime}</span>
          </div>
        </div>
      {/if}
    </section>

    {#if localSettings.cloudSyncEnabled}
      <section class="settings-section">
        <h3 class="section-title">Auto-Sync Interval</h3>

        <div class="setting-group">
          <label class="setting-label">Sync Frequency</label>
          <div class="interval-selector">
            <button
              class="interval-option"
              class:active={localSettings.autoSyncInterval === 'manual'}
              on:click={() => handleAutoSyncIntervalChange('manual')}
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 11l3 3L22 4"/>
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
              </svg>
              <span class="interval-label">Manual</span>
            </button>

            <button
              class="interval-option"
              class:active={localSettings.autoSyncInterval === '5min'}
              on:click={() => handleAutoSyncIntervalChange('5min')}
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
              <span class="interval-label">5 min</span>
            </button>

            <button
              class="interval-option"
              class:active={localSettings.autoSyncInterval === '15min'}
              on:click={() => handleAutoSyncIntervalChange('15min')}
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
              <span class="interval-label">15 min</span>
            </button>

            <button
              class="interval-option"
              class:active={localSettings.autoSyncInterval === '1hour'}
              on:click={() => handleAutoSyncIntervalChange('1hour')}
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
              <span class="interval-label">1 hour</span>
            </button>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">Selective Sync</h3>

        <div class="setting-group">
          <label class="setting-label">Choose what to sync:</label>

          <div class="sync-options">
            <div class="sync-option-row">
              <div class="sync-option-info">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                <div>
                  <span class="sync-option-label">Projects</span>
                  <span class="sync-option-description">Project files and settings</span>
                </div>
              </div>
              <input
                type="checkbox"
                checked={localSettings.syncProjects}
                on:change={handleSyncProjectsToggle}
              />
            </div>

            <div class="sync-option-row">
              <div class="sync-option-info">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <polyline points="12 6 12 12 16 14"/>
                </svg>
                <div>
                  <span class="sync-option-label">Loops</span>
                  <span class="sync-option-description">Audio loops and samples</span>
                </div>
              </div>
              <input
                type="checkbox"
                checked={localSettings.syncLoops}
                on:change={handleSyncLoopsToggle}
              />
            </div>

            <div class="sync-option-row">
              <div class="sync-option-info">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                <div>
                  <span class="sync-option-label">Presets</span>
                  <span class="sync-option-description">Instrument and effect presets</span>
                </div>
              </div>
              <input
                type="checkbox"
                checked={localSettings.syncPresets}
                on:change={handleSyncPresetsToggle}
              />
            </div>
          </div>
        </div>
      </section>

      <section class="settings-section">
        <h3 class="section-title">Manual Sync</h3>

        <div class="setting-group">
          <button
            class="btn btn-primary"
            on:click={handleManualSync}
            disabled={isSyncing}
          >
            {isSyncing ? 'Syncing...' : 'Sync Now'}
          </button>
          <p class="setting-description">Manually trigger a sync to cloud</p>
        </div>
      </section>
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
  .sync-settings {
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

  .sync-status-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text-secondary, #a0a0a0);
  }

  .status-indicator.synced {
    background: var(--success-color, #44ff44);
    box-shadow: 0 0 8px rgba(68, 255, 68, 0.5);
  }

  .status-indicator.syncing {
    background: var(--warning-color, #ffaa44);
    animation: pulse 1.5s infinite;
  }

  .status-indicator.error {
    background: var(--error-color, #ff4444);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
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

  .sync-info-box {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    margin-top: 1rem;
  }

  .sync-info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
  }

  .sync-info-row:not(:last-child) {
    border-bottom: 1px solid var(--border-color, #333);
  }

  .sync-info-label {
    font-size: 0.9rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .sync-info-value {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .sync-info-value.synced {
    color: var(--success-color, #44ff44);
  }

  .sync-info-value.syncing {
    color: var(--warning-color, #ffaa44);
  }

  .sync-info-value.error {
    color: var(--error-color, #ff4444);
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .interval-selector {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }

  .interval-option {
    background: var(--bg-secondary, #1e1e1e);
    border: 2px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem 1rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }

  .interval-option:hover {
    border-color: var(--border-hover, #444);
  }

  .interval-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .interval-option svg {
    color: var(--text-secondary, #a0a0a0);
  }

  .interval-option.active svg {
    color: var(--accent-color, #4a9eff);
  }

  .interval-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .sync-options {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .sync-option-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    transition: background 0.2s;
  }

  .sync-option-row:hover {
    background: var(--bg-hover, #2a2a2a);
  }

  .sync-option-info {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .sync-option-info svg {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .sync-option-label {
    display: block;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 0.25rem;
  }

  .sync-option-description {
    display: block;
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .sync-option-row input[type="checkbox"] {
    width: 20px;
    height: 20px;
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
