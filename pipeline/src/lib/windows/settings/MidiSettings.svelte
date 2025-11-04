<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: MidiSettingsData;
    cancel: void;
  }>();

  export interface MidiSettingsData {
    defaultInputDevice: string;
    defaultOutputDevice: string;
    syncMode: 'internal' | 'external' | 'host';
    tempoSyncEnabled: boolean;
    flushNotesOnStop: boolean;
  }

  export let settings: MidiSettingsData = {
    defaultInputDevice: 'default',
    defaultOutputDevice: 'default',
    syncMode: 'internal',
    tempoSyncEnabled: true,
    flushNotesOnStop: true
  };

  let localSettings: MidiSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let testing: boolean = false;

  const midiInputDevices = [
    { value: 'default', label: 'Default MIDI Input' },
    { value: 'midi-keyboard', label: 'MIDI Keyboard' },
    { value: 'usb-controller', label: 'USB MIDI Controller' },
    { value: 'virtual-port', label: 'Virtual MIDI Port' }
  ];

  const midiOutputDevices = [
    { value: 'default', label: 'Default MIDI Output' },
    { value: 'synth', label: 'Built-in Synthesizer' },
    { value: 'usb-device', label: 'USB MIDI Device' },
    { value: 'virtual-port', label: 'Virtual MIDI Port' }
  ];

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleInputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.defaultInputDevice = target.value;
    localSettings = localSettings;
  }

  function handleOutputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.defaultOutputDevice = target.value;
    localSettings = localSettings;
  }

  function handleSyncModeChange(mode: 'internal' | 'external' | 'host') {
    localSettings.syncMode = mode;
    localSettings = localSettings;
  }

  function handleTempoSyncToggle() {
    localSettings.tempoSyncEnabled = !localSettings.tempoSyncEnabled;
    localSettings = localSettings;
  }

  function handleFlushNotesToggle() {
    localSettings.flushNotesOnStop = !localSettings.flushNotesOnStop;
    localSettings = localSettings;
  }

  async function handleTestOutput() {
    testing = true;
    // In real implementation, would send MIDI test note
    await new Promise(resolve => setTimeout(resolve, 1000));
    testing = false;
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
    const defaults: MidiSettingsData = {
      defaultInputDevice: 'default',
      defaultOutputDevice: 'default',
      syncMode: 'internal',
      tempoSyncEnabled: true,
      flushNotesOnStop: true
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

<div class="midi-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">MIDI Input</h3>

      <div class="setting-group">
        <label class="setting-label" for="input-device">Default MIDI Input Device</label>
        <select
          id="input-device"
          class="setting-select"
          value={localSettings.defaultInputDevice}
          on:change={handleInputDeviceChange}
        >
          {#each midiInputDevices as device}
            <option value={device.value}>{device.label}</option>
          {/each}
        </select>
        <p class="setting-description">Select the default device for MIDI input</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">MIDI Output</h3>

      <div class="setting-group">
        <label class="setting-label" for="output-device">Default MIDI Output Device</label>
        <div class="device-selector">
          <select
            id="output-device"
            class="setting-select"
            value={localSettings.defaultOutputDevice}
            on:change={handleOutputDeviceChange}
          >
            {#each midiOutputDevices as device}
              <option value={device.value}>{device.label}</option>
            {/each}
          </select>
          <button class="btn btn-secondary test-btn" on:click={handleTestOutput} disabled={testing}>
            {testing ? 'Testing...' : 'Test'}
          </button>
        </div>
        <p class="setting-description">Select the default device for MIDI output</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">MIDI Sync</h3>

      <div class="setting-group">
        <label class="setting-label">Sync Mode</label>
        <div class="sync-mode-selector">
          <button
            class="sync-option"
            class:active={localSettings.syncMode === 'internal'}
            on:click={() => handleSyncModeChange('internal')}
          >
            <div class="sync-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
              </svg>
            </div>
            <div class="sync-info">
              <span class="sync-label">Internal Clock</span>
              <span class="sync-description">Use internal tempo</span>
            </div>
          </button>

          <button
            class="sync-option"
            class:active={localSettings.syncMode === 'external'}
            on:click={() => handleSyncModeChange('external')}
          >
            <div class="sync-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
              </svg>
            </div>
            <div class="sync-info">
              <span class="sync-label">External Clock</span>
              <span class="sync-description">Sync to MIDI clock</span>
            </div>
          </button>

          <button
            class="sync-option"
            class:active={localSettings.syncMode === 'host'}
            on:click={() => handleSyncModeChange('host')}
          >
            <div class="sync-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
                <line x1="8" y1="21" x2="16" y2="21"/>
                <line x1="12" y1="17" x2="12" y2="21"/>
              </svg>
            </div>
            <div class="sync-info">
              <span class="sync-label">Host Sync</span>
              <span class="sync-description">Sync to DAW host</span>
            </div>
          </button>
        </div>
      </div>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Tempo Sync Enabled</label>
            <p class="setting-description">Enable tempo synchronization with external devices</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.tempoSyncEnabled}
            on:click={handleTempoSyncToggle}
            aria-label="Toggle tempo sync"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Playback</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Flush Notes on Stop (Panic)</label>
            <p class="setting-description">Send all notes off when stopping playback to prevent stuck notes</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.flushNotesOnStop}
            on:click={handleFlushNotesToggle}
            aria-label="Toggle flush notes"
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
  .midi-settings {
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

  .sync-mode-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .sync-option {
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

  .sync-option:hover {
    border-color: var(--border-hover, #444);
  }

  .sync-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .sync-icon {
    color: var(--text-secondary, #a0a0a0);
  }

  .sync-option.active .sync-icon {
    color: var(--accent-color, #4a9eff);
  }

  .sync-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    text-align: center;
  }

  .sync-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .sync-description {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
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
