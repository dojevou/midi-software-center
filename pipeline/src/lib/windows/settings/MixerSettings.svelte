<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: MixerSettingsData;
    cancel: void;
  }>();

  export interface MixerSettingsData {
    peakMeteringMode: 'peak' | 'rms' | 'both';
    faderType: 'linear' | 'exponential';
    masterLevel: number;
    clipThreshold: number;
    meterRefreshRate: number;
  }

  export let settings: MixerSettingsData = {
    peakMeteringMode: 'both',
    faderType: 'linear',
    masterLevel: 0,
    clipThreshold: -1,
    meterRefreshRate: 50
  };

  let localSettings: MixerSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;

  const refreshRates = [
    { value: 16, label: '16ms (60 FPS)' },
    { value: 50, label: '50ms (20 FPS)' },
    { value: 100, label: '100ms (10 FPS)' }
  ];

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);
  $: masterLevelDb = localSettings.masterLevel === -100 ? '-∞' : `${localSettings.masterLevel.toFixed(1)} dB`;

  function handleMeteringModeChange(mode: 'peak' | 'rms' | 'both') {
    localSettings.peakMeteringMode = mode;
    localSettings = localSettings;
  }

  function handleFaderTypeChange(type: 'linear' | 'exponential') {
    localSettings.faderType = type;
    localSettings = localSettings;
  }

  function handleMasterLevelChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.masterLevel = parseFloat(target.value);
    localSettings = localSettings;
  }

  function handleClipThresholdChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.clipThreshold = parseFloat(target.value);
    localSettings = localSettings;
  }

  function handleRefreshRateChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.meterRefreshRate = parseInt(target.value);
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
    const defaults: MixerSettingsData = {
      peakMeteringMode: 'both',
      faderType: 'linear',
      masterLevel: 0,
      clipThreshold: -1,
      meterRefreshRate: 50
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

<div class="mixer-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Metering</h3>

      <div class="setting-group">
        <label class="setting-label">Peak Metering Mode</label>
        <div class="metering-selector">
          <button
            class="metering-option"
            class:active={localSettings.peakMeteringMode === 'peak'}
            on:click={() => handleMeteringModeChange('peak')}
          >
            <div class="meter-preview">
              <div class="meter-bar peak-only"></div>
            </div>
            <span class="metering-label">Peak</span>
            <span class="metering-description">Show peak levels only</span>
          </button>

          <button
            class="metering-option"
            class:active={localSettings.peakMeteringMode === 'rms'}
            on:click={() => handleMeteringModeChange('rms')}
          >
            <div class="meter-preview">
              <div class="meter-bar rms-only"></div>
            </div>
            <span class="metering-label">RMS</span>
            <span class="metering-description">Show RMS average</span>
          </button>

          <button
            class="metering-option"
            class:active={localSettings.peakMeteringMode === 'both'}
            on:click={() => handleMeteringModeChange('both')}
          >
            <div class="meter-preview">
              <div class="meter-bar peak-rms"></div>
            </div>
            <span class="metering-label">Both</span>
            <span class="metering-description">Show peak and RMS</span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="refresh-rate">Meter Refresh Rate</label>
        <select
          id="refresh-rate"
          class="setting-select"
          value={localSettings.meterRefreshRate}
          on:change={handleRefreshRateChange}
        >
          {#each refreshRates as rate}
            <option value={rate.value}>{rate.label}</option>
          {/each}
        </select>
        <p class="setting-description">Higher refresh rates provide smoother meters but use more CPU</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Faders</h3>

      <div class="setting-group">
        <label class="setting-label">Fader Type</label>
        <div class="fader-type-selector">
          <button
            class="fader-option"
            class:active={localSettings.faderType === 'linear'}
            on:click={() => handleFaderTypeChange('linear')}
          >
            <div class="fader-curve">
              <svg width="100" height="60" viewBox="0 0 100 60">
                <line x1="0" y1="60" x2="100" y2="0" stroke="currentColor" stroke-width="2"/>
              </svg>
            </div>
            <span class="fader-label">Linear dB</span>
            <span class="fader-description">Linear decibel scale</span>
          </button>

          <button
            class="fader-option"
            class:active={localSettings.faderType === 'exponential'}
            on:click={() => handleFaderTypeChange('exponential')}
          >
            <div class="fader-curve">
              <svg width="100" height="60" viewBox="0 0 100 60">
                <path d="M 0 60 Q 50 40, 100 0" stroke="currentColor" stroke-width="2" fill="none"/>
              </svg>
            </div>
            <span class="fader-label">Exponential</span>
            <span class="fader-description">Exponential curve</span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Master</h3>

      <div class="setting-group">
        <label class="setting-label" for="master-level">
          Master Level: {masterLevelDb}
        </label>
        <input
          id="master-level"
          type="range"
          class="setting-slider"
          min="-100"
          max="0"
          step="0.1"
          value={localSettings.masterLevel}
          on:input={handleMasterLevelChange}
        />
        <div class="slider-labels">
          <span>-∞ dB</span>
          <span>-50 dB</span>
          <span>0 dB</span>
        </div>
        <p class="setting-description">Default master output level</p>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="clip-threshold">
          Clip Threshold: {localSettings.clipThreshold.toFixed(1)} dB
        </label>
        <input
          id="clip-threshold"
          type="range"
          class="setting-slider"
          min="-10"
          max="-1"
          step="0.5"
          value={localSettings.clipThreshold}
          on:input={handleClipThresholdChange}
        />
        <div class="slider-labels">
          <span>-10 dB</span>
          <span>-5 dB</span>
          <span>-1 dB</span>
        </div>
        <p class="setting-description">Level at which meters display clipping warning</p>
      </div>
    </section>

    <div class="info-box">
      <svg class="info-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
      <div>
        <strong>Metering Best Practices</strong>
        <p>For accurate monitoring, use "Both" mode to see peak and RMS levels. Keep master level at 0 dB and adjust individual track levels instead.</p>
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

<style>
  .mixer-settings {
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

  .metering-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .metering-option {
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

  .metering-option:hover {
    border-color: var(--border-hover, #444);
  }

  .metering-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .meter-preview {
    width: 40px;
    height: 60px;
    background: var(--bg-tertiary, #333);
    border-radius: 4px;
    overflow: hidden;
    position: relative;
  }

  .meter-bar {
    position: absolute;
    bottom: 0;
    width: 100%;
    height: 70%;
    transition: all 0.3s;
  }

  .meter-bar.peak-only {
    background: linear-gradient(to top, #44ff44, #ffaa44, #ff4444);
  }

  .meter-bar.rms-only {
    background: linear-gradient(to top, #4488ff, #44aaff);
  }

  .meter-bar.peak-rms {
    background: linear-gradient(to top, #44ff44 0%, #44ff44 50%, #4488ff 50%, #4488ff 100%);
  }

  .metering-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .metering-description {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
    text-align: center;
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

  .fader-type-selector {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }

  .fader-option {
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

  .fader-option:hover {
    border-color: var(--border-hover, #444);
  }

  .fader-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .fader-curve {
    color: var(--text-secondary, #a0a0a0);
  }

  .fader-option.active .fader-curve {
    color: var(--accent-color, #4a9eff);
  }

  .fader-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .fader-description {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
    text-align: center;
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
</style>
