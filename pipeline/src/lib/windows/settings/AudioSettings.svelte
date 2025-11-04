<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: AudioSettingsData;
    cancel: void;
  }>();

  export interface AudioSettingsData {
    bufferSize: number;
    sampleRate: number;
    inputDevice: string;
    outputDevice: string;
    monitorLatency: boolean;
  }

  export let settings: AudioSettingsData = {
    bufferSize: 256,
    sampleRate: 48000,
    inputDevice: 'default',
    outputDevice: 'default',
    monitorLatency: true
  };

  let localSettings: AudioSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let testingInput: boolean = false;
  let testingOutput: boolean = false;

  const bufferSizes = [
    { value: 32, label: '32 samples', latency: '0.7 ms', description: 'Ultra-low latency (high CPU)' },
    { value: 64, label: '64 samples', latency: '1.3 ms', description: 'Very low latency (high CPU)' },
    { value: 128, label: '128 samples', latency: '2.7 ms', description: 'Low latency' },
    { value: 256, label: '256 samples', latency: '5.3 ms', description: 'Balanced (recommended)' },
    { value: 512, label: '512 samples', latency: '10.7 ms', description: 'Medium latency (lower CPU)' },
    { value: 1024, label: '1024 samples', latency: '21.3 ms', description: 'High latency (low CPU)' },
    { value: 2048, label: '2048 samples', latency: '42.7 ms', description: 'Very high latency' },
    { value: 4096, label: '4096 samples', latency: '85.3 ms', description: 'Extreme latency' }
  ];

  const sampleRates = [
    { value: 44100, label: '44.1 kHz (CD Quality)' },
    { value: 48000, label: '48 kHz (Standard)' },
    { value: 88200, label: '88.2 kHz (High Quality)' },
    { value: 96000, label: '96 kHz (Professional)' },
    { value: 192000, label: '192 kHz (Ultra High Quality)' }
  ];

  // Mock audio devices - would come from Tauri in real implementation
  const inputDevices = [
    { value: 'default', label: 'Default Input Device' },
    { value: 'microphone', label: 'Built-in Microphone' },
    { value: 'line-in', label: 'Line In' },
    { value: 'usb-audio', label: 'USB Audio Interface' }
  ];

  const outputDevices = [
    { value: 'default', label: 'Default Output Device' },
    { value: 'speakers', label: 'Built-in Speakers' },
    { value: 'headphones', label: 'Headphones' },
    { value: 'usb-audio', label: 'USB Audio Interface' }
  ];

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);
  $: selectedBufferSize = bufferSizes.find(b => b.value === localSettings.bufferSize);
  $: estimatedLatency = calculateLatency(localSettings.bufferSize, localSettings.sampleRate);

  function calculateLatency(bufferSize: number, sampleRate: number): number {
    return (bufferSize / sampleRate) * 1000; // Convert to milliseconds
  }

  function handleBufferSizeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.bufferSize = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleSampleRateChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.sampleRate = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleInputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.inputDevice = target.value;
    localSettings = localSettings;
  }

  function handleOutputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.outputDevice = target.value;
    localSettings = localSettings;
  }

  function handleMonitorLatencyToggle() {
    localSettings.monitorLatency = !localSettings.monitorLatency;
    localSettings = localSettings;
  }

  async function handleTestInput() {
    testingInput = true;
    // In real implementation, would use Tauri to test input device
    await new Promise(resolve => setTimeout(resolve, 1000));
    testingInput = false;
  }

  async function handleTestOutput() {
    testingOutput = true;
    // In real implementation, would use Tauri to play test tone
    await new Promise(resolve => setTimeout(resolve, 1000));
    testingOutput = false;
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
    const defaults: AudioSettingsData = {
      bufferSize: 256,
      sampleRate: 48000,
      inputDevice: 'default',
      outputDevice: 'default',
      monitorLatency: true
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

<div class="audio-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Audio Engine</h3>

      <div class="setting-group">
        <label class="setting-label" for="buffer-size">Buffer Size</label>
        <select
          id="buffer-size"
          class="setting-select"
          value={localSettings.bufferSize}
          on:change={handleBufferSizeChange}
        >
          {#each bufferSizes as buffer}
            <option value={buffer.value}>
              {buffer.label} - {buffer.latency} ({buffer.description})
            </option>
          {/each}
        </select>
        {#if selectedBufferSize}
          <p class="setting-description">{selectedBufferSize.description}</p>
        {/if}
      </div>

      <div class="setting-group">
        <label class="setting-label" for="sample-rate">Sample Rate</label>
        <select
          id="sample-rate"
          class="setting-select"
          value={localSettings.sampleRate}
          on:change={handleSampleRateChange}
        >
          {#each sampleRates as rate}
            <option value={rate.value}>{rate.label}</option>
          {/each}
        </select>
        <p class="setting-description">Higher sample rates provide better quality but require more CPU</p>
      </div>

      <div class="latency-display">
        <div class="latency-info">
          <span class="latency-label">Estimated Latency:</span>
          <span class="latency-value">{estimatedLatency.toFixed(2)} ms</span>
        </div>
        <div class="latency-bar">
          <div
            class="latency-fill"
            class:low={estimatedLatency < 10}
            class:medium={estimatedLatency >= 10 && estimatedLatency < 30}
            class:high={estimatedLatency >= 30}
            style="width: {Math.min((estimatedLatency / 100) * 100, 100)}%"
          ></div>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Input Device</h3>

      <div class="setting-group">
        <label class="setting-label" for="input-device">Audio Input</label>
        <div class="device-selector">
          <select
            id="input-device"
            class="setting-select"
            value={localSettings.inputDevice}
            on:change={handleInputDeviceChange}
          >
            {#each inputDevices as device}
              <option value={device.value}>{device.label}</option>
            {/each}
          </select>
          <button
            class="btn btn-secondary test-btn"
            on:click={handleTestInput}
            disabled={testingInput}
          >
            {testingInput ? 'Testing...' : 'Test'}
          </button>
        </div>
        <p class="setting-description">Select the audio input device for recording</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Output Device</h3>

      <div class="setting-group">
        <label class="setting-label" for="output-device">Audio Output</label>
        <div class="device-selector">
          <select
            id="output-device"
            class="setting-select"
            value={localSettings.outputDevice}
            on:change={handleOutputDeviceChange}
          >
            {#each outputDevices as device}
              <option value={device.value}>{device.label}</option>
            {/each}
          </select>
          <button
            class="btn btn-secondary test-btn"
            on:click={handleTestOutput}
            disabled={testingOutput}
          >
            {testingOutput ? 'Playing...' : 'Test'}
          </button>
        </div>
        <p class="setting-description">Select the audio output device for playback</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Monitoring</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Latency Monitoring</label>
            <p class="setting-description">Display real-time latency information during playback</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.monitorLatency}
            on:click={handleMonitorLatencyToggle}
            aria-label="Toggle latency monitoring"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <div class="warning-box">
      <svg class="warning-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/>
        <line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <div>
        <strong>Audio Driver Restart Required</strong>
        <p>Changing buffer size or sample rate requires restarting the audio engine. All audio will stop momentarily.</p>
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
  .audio-settings {
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

  .device-selector {
    display: flex;
    gap: 0.75rem;
  }

  .device-selector .setting-select {
    flex: 1;
  }

  .test-btn {
    min-width: 100px;
    flex-shrink: 0;
  }

  .latency-display {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.25rem;
    margin-top: 1rem;
  }

  .latency-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .latency-label {
    font-size: 0.9rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .latency-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .latency-bar {
    height: 8px;
    background: var(--bg-tertiary, #333);
    border-radius: 4px;
    overflow: hidden;
  }

  .latency-fill {
    height: 100%;
    transition: all 0.3s;
    border-radius: 4px;
  }

  .latency-fill.low {
    background: var(--success-color, #44ff44);
  }

  .latency-fill.medium {
    background: var(--warning-color, #ffaa44);
  }

  .latency-fill.high {
    background: var(--error-color, #ff4444);
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
    background: rgba(255, 170, 68, 0.1);
    border: 1px solid rgba(255, 170, 68, 0.3);
    border-radius: 8px;
    padding: 1rem;
    margin-top: 2rem;
  }

  .warning-icon {
    color: var(--warning-color, #ffaa44);
    flex-shrink: 0;
  }

  .warning-box strong {
    display: block;
    color: var(--warning-color, #ffaa44);
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
