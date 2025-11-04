<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: RecordingSettingsData;
    cancel: void;
  }>();

  export interface RecordingSettingsData {
    recordingFormat: 'wav' | 'mp3' | 'flac';
    inputMonitoring: boolean;
    latencyCompensation: number;
    autoPunchInOut: boolean;
    punchInLeadIn: number;
    punchOutLeadOut: number;
    recordToFolder: string;
  }

  export let settings: RecordingSettingsData = {
    recordingFormat: 'wav',
    inputMonitoring: false,
    latencyCompensation: 0,
    autoPunchInOut: false,
    punchInLeadIn: 1,
    punchOutLeadOut: 1,
    recordToFolder: ''
  };

  let localSettings: RecordingSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleFormatChange(format: 'wav' | 'mp3' | 'flac') {
    localSettings.recordingFormat = format;
    localSettings = localSettings;
  }

  function handleInputMonitoringToggle() {
    localSettings.inputMonitoring = !localSettings.inputMonitoring;
    localSettings = localSettings;
  }

  function handleLatencyCompensationChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.latencyCompensation = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleAutoPunchToggle() {
    localSettings.autoPunchInOut = !localSettings.autoPunchInOut;
    localSettings = localSettings;
  }

  function handlePunchInLeadInChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.punchInLeadIn = parseInt(target.value);
    localSettings = localSettings;
  }

  function handlePunchOutLeadOutChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.punchOutLeadOut = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleSelectFolder() {
    // In real implementation, would open folder picker via Tauri
    const mockPath = '/path/to/recordings';
    localSettings.recordToFolder = mockPath;
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
    const defaults: RecordingSettingsData = {
      recordingFormat: 'wav',
      inputMonitoring: false,
      latencyCompensation: 0,
      autoPunchInOut: false,
      punchInLeadIn: 1,
      punchOutLeadOut: 1,
      recordToFolder: ''
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

<div class="recording-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Recording Format</h3>

      <div class="setting-group">
        <label class="setting-label">Audio Format</label>
        <div class="format-selector">
          <button
            class="format-option"
            class:active={localSettings.recordingFormat === 'wav'}
            on:click={() => handleFormatChange('wav')}
          >
            <span class="format-name">WAV</span>
            <span class="format-description">Uncompressed, lossless</span>
            <span class="format-quality">Best quality</span>
          </button>

          <button
            class="format-option"
            class:active={localSettings.recordingFormat === 'mp3'}
            on:click={() => handleFormatChange('mp3')}
          >
            <span class="format-name">MP3</span>
            <span class="format-description">Compressed, lossy</span>
            <span class="format-quality">Smaller files</span>
          </button>

          <button
            class="format-option"
            class:active={localSettings.recordingFormat === 'flac'}
            on:click={() => handleFormatChange('flac')}
          >
            <span class="format-name">FLAC</span>
            <span class="format-description">Compressed, lossless</span>
            <span class="format-quality">Balance</span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Input Monitoring</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Enable Input Monitoring</label>
            <p class="setting-description">Hear input audio through outputs during recording</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.inputMonitoring}
            on:click={handleInputMonitoringToggle}
            aria-label="Toggle input monitoring"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="latency-comp">
          Latency Compensation: {localSettings.latencyCompensation} ms
        </label>
        <input
          id="latency-comp"
          type="range"
          class="setting-slider"
          min="0"
          max="100"
          step="1"
          value={localSettings.latencyCompensation}
          on:input={handleLatencyCompensationChange}
        />
        <div class="slider-labels">
          <span>0 ms</span>
          <span>50 ms</span>
          <span>100 ms</span>
        </div>
        <p class="setting-description">Compensate for audio interface latency</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Punch In/Out</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Auto-Punch In/Out</label>
            <p class="setting-description">Automatically start and stop recording at specified points</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.autoPunchInOut}
            on:click={handleAutoPunchToggle}
            aria-label="Toggle auto-punch"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      {#if localSettings.autoPunchInOut}
        <div class="setting-group">
          <label class="setting-label" for="punch-in">
            Punch In Lead-In: {localSettings.punchInLeadIn} {localSettings.punchInLeadIn === 1 ? 'bar' : 'bars'}
          </label>
          <input
            id="punch-in"
            type="range"
            class="setting-slider"
            min="0"
            max="4"
            step="1"
            value={localSettings.punchInLeadIn}
            on:input={handlePunchInLeadInChange}
          />
          <div class="slider-labels">
            <span>0 bars</span>
            <span>2 bars</span>
            <span>4 bars</span>
          </div>
          <p class="setting-description">Play this many bars before punch-in point</p>
        </div>

        <div class="setting-group">
          <label class="setting-label" for="punch-out">
            Punch Out Lead-Out: {localSettings.punchOutLeadOut} {localSettings.punchOutLeadOut === 1 ? 'bar' : 'bars'}
          </label>
          <input
            id="punch-out"
            type="range"
            class="setting-slider"
            min="0"
            max="4"
            step="1"
            value={localSettings.punchOutLeadOut}
            on:input={handlePunchOutLeadOutChange}
          />
          <div class="slider-labels">
            <span>0 bars</span>
            <span>2 bars</span>
            <span>4 bars</span>
          </div>
          <p class="setting-description">Continue playing this many bars after punch-out point</p>
        </div>
      {/if}
    </section>

    <section class="settings-section">
      <h3 class="section-title">Recording Location</h3>

      <div class="setting-group">
        <label class="setting-label">Record to Folder</label>
        {#if localSettings.recordToFolder}
          <div class="folder-display">
            <svg class="folder-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span class="folder-path">{localSettings.recordToFolder}</span>
          </div>
        {:else}
          <p class="no-folder-message">Default recordings folder will be used</p>
        {/if}
        <button class="btn btn-secondary" on:click={handleSelectFolder}>
          Select Folder
        </button>
        <p class="setting-description">Recordings will be saved to this location</p>
      </div>
    </section>

    <div class="info-box">
      <svg class="info-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
      <div>
        <strong>Recording Tip</strong>
        <p>For best results, enable input monitoring with low latency compensation and use WAV format for professional recordings.</p>
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
  .recording-settings {
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

  .format-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .format-option {
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
    text-align: center;
  }

  .format-option:hover {
    border-color: var(--border-hover, #444);
  }

  .format-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .format-name {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .format-description {
    font-size: 0.85rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .format-quality {
    font-size: 0.8rem;
    color: var(--accent-color, #4a9eff);
    font-weight: 500;
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

  .folder-display {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 0.75rem;
  }

  .folder-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .folder-path {
    flex: 1;
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-folder-message {
    color: var(--text-secondary, #a0a0a0);
    font-size: 0.9rem;
    font-style: italic;
    margin: 0 0 0.75rem 0;
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
