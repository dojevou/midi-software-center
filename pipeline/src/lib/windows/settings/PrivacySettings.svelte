<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher<{
    save: PrivacySettingsData;
    cancel: void;
  }>();

  export interface PrivacySettingsData {
    analyticsEnabled: boolean;
    crashReportingEnabled: boolean;
    usageTrackingEnabled: boolean;
    dataRetention: '7days' | '30days' | '90days';
  }

  export let settings: PrivacySettingsData = {
    analyticsEnabled: true,
    crashReportingEnabled: true,
    usageTrackingEnabled: false,
    dataRetention: '30days'
  };

  let localSettings: PrivacySettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let showModal: boolean = false;
  let modalAction: 'export' | 'delete' | null = null;

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleAnalyticsToggle() {
    localSettings.analyticsEnabled = !localSettings.analyticsEnabled;
    localSettings = localSettings;
  }

  function handleCrashReportingToggle() {
    localSettings.crashReportingEnabled = !localSettings.crashReportingEnabled;
    localSettings = localSettings;
  }

  function handleUsageTrackingToggle() {
    localSettings.usageTrackingEnabled = !localSettings.usageTrackingEnabled;
    localSettings = localSettings;
  }

  function handleDataRetentionChange(retention: '7days' | '30days' | '90days') {
    localSettings.dataRetention = retention;
    localSettings = localSettings;
  }

  function handleExportData() {
    modalAction = 'export';
    showModal = true;
  }

  function handleDeleteData() {
    modalAction = 'delete';
    showModal = true;
  }

  function handleModalConfirm() {
    if (modalAction === 'export') {
      // In real implementation, would export data via Tauri
      console.log('Exporting privacy data');
    } else if (modalAction === 'delete') {
      // In real implementation, would delete tracking data via Tauri
      console.log('Deleting tracking data');
    }
    showModal = false;
    modalAction = null;
  }

  function handleModalCancel() {
    showModal = false;
    modalAction = null;
  }

  function handlePrivacyPolicy() {
    // In real implementation, would open privacy policy URL
    window.open('https://example.com/privacy', '_blank');
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
    const defaults: PrivacySettingsData = {
      analyticsEnabled: true,
      crashReportingEnabled: true,
      usageTrackingEnabled: false,
      dataRetention: '30days'
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

<div class="privacy-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Data Collection</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Analytics</label>
            <p class="setting-description">Help improve the application by sending anonymous usage statistics</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.analyticsEnabled}
            on:click={handleAnalyticsToggle}
            aria-label="Toggle analytics"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Crash Reporting</label>
            <p class="setting-description">Automatically send crash reports to help identify and fix issues</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.crashReportingEnabled}
            on:click={handleCrashReportingToggle}
            aria-label="Toggle crash reporting"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Usage Tracking</label>
            <p class="setting-description">Track feature usage to understand how you use the application</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.usageTrackingEnabled}
            on:click={handleUsageTrackingToggle}
            aria-label="Toggle usage tracking"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Data Retention</h3>

      <div class="setting-group">
        <label class="setting-label">Keep Data For</label>
        <div class="retention-selector">
          <button
            class="retention-option"
            class:active={localSettings.dataRetention === '7days'}
            on:click={() => handleDataRetentionChange('7days')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            <span class="retention-label">7 Days</span>
            <span class="retention-description">Minimum retention</span>
          </button>

          <button
            class="retention-option"
            class:active={localSettings.dataRetention === '30days'}
            on:click={() => handleDataRetentionChange('30days')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            <span class="retention-label">30 Days</span>
            <span class="retention-description">Balanced retention</span>
          </button>

          <button
            class="retention-option"
            class:active={localSettings.dataRetention === '90days'}
            on:click={() => handleDataRetentionChange('90days')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
              <line x1="16" y1="2" x2="16" y2="6"/>
              <line x1="8" y1="2" x2="8" y2="6"/>
              <line x1="3" y1="10" x2="21" y2="10"/>
            </svg>
            <span class="retention-label">90 Days</span>
            <span class="retention-description">Extended retention</span>
          </button>
        </div>
        <p class="setting-description">How long to keep analytics and usage data before automatic deletion</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Your Data</h3>

      <div class="data-actions">
        <div class="data-action-card">
          <div class="data-action-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
          </div>
          <div class="data-action-content">
            <h4 class="data-action-title">Export Your Data</h4>
            <p class="data-action-description">Download a copy of all your usage data and analytics</p>
            <button class="btn btn-secondary" on:click={handleExportData}>
              Export Data
            </button>
          </div>
        </div>

        <div class="data-action-card">
          <div class="data-action-icon danger">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </div>
          <div class="data-action-content">
            <h4 class="data-action-title">Delete Tracking Data</h4>
            <p class="data-action-description">Permanently delete all stored analytics and usage data</p>
            <button class="btn btn-danger" on:click={handleDeleteData}>
              Delete All Data
            </button>
          </div>
        </div>

        <div class="data-action-card">
          <div class="data-action-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="16" x2="12" y2="12"/>
              <line x1="12" y1="8" x2="12.01" y2="8"/>
            </svg>
          </div>
          <div class="data-action-content">
            <h4 class="data-action-title">Privacy Policy</h4>
            <p class="data-action-description">Read our full privacy policy and data handling practices</p>
            <button class="btn btn-secondary" on:click={handlePrivacyPolicy}>
              View Policy
            </button>
          </div>
        </div>
      </div>
    </section>

    <div class="info-box">
      <svg class="info-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
      <div>
        <strong>Your Privacy Matters</strong>
        <p>We take your privacy seriously. All collected data is anonymous, encrypted, and used solely to improve the application. You can disable any tracking feature at any time.</p>
      </div>
    </div>
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
  {#if modalAction === 'export'}
    <SettingsModal
      title="Export Data"
      message="Your data will be exported as a ZIP file containing JSON files with all analytics and usage information."
      confirmText="Export"
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'delete'}
    <SettingsModal
      title="Delete Tracking Data"
      message="Are you sure you want to permanently delete all tracking data? This action cannot be undone."
      confirmText="Delete Data"
      dangerous={true}
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {/if}
{/if}

<style>
  .privacy-settings {
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

  .retention-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .retention-option {
    background: var(--bg-secondary, #1e1e1e);
    border: 2px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem 1rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .retention-option:hover {
    border-color: var(--border-hover, #444);
  }

  .retention-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .retention-option svg {
    color: var(--text-secondary, #a0a0a0);
  }

  .retention-option.active svg {
    color: var(--accent-color, #4a9eff);
  }

  .retention-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .retention-description {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
    text-align: center;
  }

  .data-actions {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .data-action-card {
    display: flex;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
  }

  .data-action-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .data-action-icon.danger {
    color: var(--error-color, #ff4444);
  }

  .data-action-content {
    flex: 1;
  }

  .data-action-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin: 0 0 0.5rem 0;
  }

  .data-action-description {
    font-size: 0.875rem;
    color: var(--text-secondary, #a0a0a0);
    margin: 0 0 1rem 0;
    line-height: 1.4;
  }

  .info-box {
    display: flex;
    gap: 1rem;
    background: rgba(74, 158, 255, 0.1);
    border: 1px solid rgba(74, 158, 255, 0.3);
    border-radius: 8px;
    padding: 1rem;
    margin-top: 2rem;
  }

  .info-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .info-box strong {
    display: block;
    color: var(--accent-color, #4a9eff);
    margin-bottom: 0.25rem;
    font-size: 0.95rem;
  }

  .info-box p {
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

  .btn-danger {
    background: var(--error-color, #ff4444);
    color: #fff;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ee3333;
  }
</style>
