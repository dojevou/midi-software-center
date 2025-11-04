<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher<{
    save: PerformanceSettingsData;
    cancel: void;
  }>();

  export interface PerformanceSettingsData {
    cacheSize: number;
    virtualScrollingThreshold: number;
    threadCount: number;
    memoryLimitAlert: number;
  }

  export let settings: PerformanceSettingsData = {
    cacheSize: 500,
    virtualScrollingThreshold: 100,
    threadCount: 4,
    memoryLimitAlert: 2000
  };

  let localSettings: PerformanceSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let showClearCacheModal: boolean = false;

  // Mock performance stats - would come from Tauri in real implementation
  let perfStats = {
    memoryUsed: 256,
    cacheHits: 1247,
    cpuUsage: 23
  };

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);
  $: availableCores = navigator.hardwareConcurrency || 4;

  function handleCacheSizeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.cacheSize = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleVirtualScrollingChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.virtualScrollingThreshold = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleThreadCountChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.threadCount = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleMemoryLimitChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.memoryLimitAlert = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleClearCache() {
    showClearCacheModal = true;
  }

  function handleClearCacheConfirm() {
    // In real implementation, would clear cache via Tauri
    showClearCacheModal = false;
    perfStats.cacheHits = 0;
  }

  function handleClearCacheCancel() {
    showClearCacheModal = false;
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
    const defaults: PerformanceSettingsData = {
      cacheSize: 500,
      virtualScrollingThreshold: 100,
      threadCount: Math.min(4, availableCores),
      memoryLimitAlert: 2000
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

<div class="performance-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Cache</h3>

      <div class="setting-group">
        <label class="setting-label" for="cache-size">
          Cache Size: {localSettings.cacheSize} MB
        </label>
        <input
          id="cache-size"
          type="range"
          class="setting-slider"
          min="100"
          max="2000"
          step="50"
          value={localSettings.cacheSize}
          on:input={handleCacheSizeChange}
        />
        <div class="slider-labels">
          <span>100 MB</span>
          <span>1050 MB</span>
          <span>2000 MB</span>
        </div>
        <p class="setting-description">Maximum cache size for MIDI file data</p>
      </div>

      <div class="cache-info">
        <div class="cache-stat">
          <span class="stat-label">Current Usage:</span>
          <span class="stat-value">{perfStats.memoryUsed} MB</span>
        </div>
        <div class="cache-stat">
          <span class="stat-label">Cache Hits:</span>
          <span class="stat-value">{perfStats.cacheHits.toLocaleString()}</span>
        </div>
      </div>

      <div class="setting-group">
        <button class="btn btn-secondary" on:click={handleClearCache}>
          Clear Cache
        </button>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Rendering</h3>

      <div class="setting-group">
        <label class="setting-label" for="virtual-scrolling">
          Virtual Scrolling Threshold: {localSettings.virtualScrollingThreshold} items
        </label>
        <input
          id="virtual-scrolling"
          type="range"
          class="setting-slider"
          min="50"
          max="500"
          step="50"
          value={localSettings.virtualScrollingThreshold}
          on:input={handleVirtualScrollingChange}
        />
        <div class="slider-labels">
          <span>50</span>
          <span>275</span>
          <span>500</span>
        </div>
        <p class="setting-description">Enable virtual scrolling for lists with more than this many items</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Processing</h3>

      <div class="setting-group">
        <label class="setting-label" for="thread-count">
          Thread Count for Batch Operations: {localSettings.threadCount} / {availableCores} cores
        </label>
        <input
          id="thread-count"
          type="range"
          class="setting-slider"
          min="1"
          max={availableCores}
          step="1"
          value={localSettings.threadCount}
          on:input={handleThreadCountChange}
        />
        <div class="slider-labels">
          <span>1 core</span>
          <span>{Math.floor(availableCores / 2)} cores</span>
          <span>{availableCores} cores</span>
        </div>
        <p class="setting-description">Number of CPU cores to use for parallel processing</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Memory</h3>

      <div class="setting-group">
        <label class="setting-label" for="memory-limit">
          Memory Limit Alert: {localSettings.memoryLimitAlert} MB
        </label>
        <input
          id="memory-limit"
          type="range"
          class="setting-slider"
          min="500"
          max="8000"
          step="500"
          value={localSettings.memoryLimitAlert}
          on:input={handleMemoryLimitChange}
        />
        <div class="slider-labels">
          <span>500 MB</span>
          <span>4 GB</span>
          <span>8 GB</span>
        </div>
        <p class="setting-description">Show warning when memory usage exceeds this threshold</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Performance Statistics</h3>

      <div class="perf-stats-grid">
        <div class="perf-stat-card">
          <div class="perf-stat-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
            </svg>
          </div>
          <div class="perf-stat-info">
            <span class="perf-stat-label">Memory Used</span>
            <span class="perf-stat-value">{perfStats.memoryUsed} MB</span>
          </div>
        </div>

        <div class="perf-stat-card">
          <div class="perf-stat-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 20v-6M6 20V10M18 20V4"/>
            </svg>
          </div>
          <div class="perf-stat-info">
            <span class="perf-stat-label">Cache Hits</span>
            <span class="perf-stat-value">{perfStats.cacheHits.toLocaleString()}</span>
          </div>
        </div>

        <div class="perf-stat-card">
          <div class="perf-stat-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
          </div>
          <div class="perf-stat-info">
            <span class="perf-stat-label">CPU Usage</span>
            <span class="perf-stat-value">{perfStats.cpuUsage}%</span>
          </div>
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

{#if showClearCacheModal}
  <SettingsModal
    title="Clear Cache"
    message="Are you sure you want to clear the cache? This will free up memory but may temporarily slow down file access."
    confirmText="Clear Cache"
    on:confirm={handleClearCacheConfirm}
    on:cancel={handleClearCacheCancel}
  />
{/if}

<style>
  .performance-settings {
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

  .cache-info {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-around;
  }

  .cache-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .stat-label {
    font-size: 0.85rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .stat-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .perf-stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .perf-stat-card {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .perf-stat-icon {
    color: var(--accent-color, #4a9eff);
  }

  .perf-stat-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    text-align: center;
  }

  .perf-stat-label {
    font-size: 0.85rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .perf-stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
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
