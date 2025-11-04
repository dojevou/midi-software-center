<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: TrackSettingsData;
    cancel: void;
  }>();

  export interface TrackSettingsData {
    defaultColor: string;
    defaultVolume: number;
    defaultPan: number;
    autoArmOnSelection: boolean;
    defaultTrackType: 'audio' | 'midi' | 'instrument';
  }

  export let settings: TrackSettingsData = {
    defaultColor: '#4a9eff',
    defaultVolume: 75,
    defaultPan: 0,
    autoArmOnSelection: false,
    defaultTrackType: 'midi'
  };

  let localSettings: TrackSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;

  const presetColors = [
    '#4a9eff', '#ff4a4a', '#4aff4a', '#ffaa4a',
    '#aa4aff', '#4affaa', '#ffaa4a', '#ff4aaa',
    '#aaff4a', '#4aaaff', '#ffaaaa', '#aaffaa',
    '#aaaaff', '#ffff4a', '#ff4aff', '#4affff'
  ];

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleColorSelect(color: string) {
    localSettings.defaultColor = color;
    localSettings = localSettings;
  }

  function handleVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.defaultVolume = parseInt(target.value);
    localSettings = localSettings;
  }

  function handlePanChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.defaultPan = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleAutoArmToggle() {
    localSettings.autoArmOnSelection = !localSettings.autoArmOnSelection;
    localSettings = localSettings;
  }

  function handleTrackTypeChange(type: 'audio' | 'midi' | 'instrument') {
    localSettings.defaultTrackType = type;
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
    const defaults: TrackSettingsData = {
      defaultColor: '#4a9eff',
      defaultVolume: 75,
      defaultPan: 0,
      autoArmOnSelection: false,
      defaultTrackType: 'midi'
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

<div class="track-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Track Defaults</h3>

      <div class="setting-group">
        <label class="setting-label">Default Track Color</label>
        <div class="color-picker">
          {#each presetColors as color}
            <button
              class="color-swatch"
              class:active={localSettings.defaultColor === color}
              style="background-color: {color}"
              on:click={() => handleColorSelect(color)}
              aria-label="Select color {color}"
            >
              {#if localSettings.defaultColor === color}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="3">
                  <path d="M20 6L9 17l-5-5"/>
                </svg>
              {/if}
            </button>
          {/each}
        </div>
        <p class="setting-description">Color for newly created tracks</p>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="volume-slider">
          Default Track Volume: {localSettings.defaultVolume}%
        </label>
        <input
          id="volume-slider"
          type="range"
          class="setting-slider"
          min="0"
          max="100"
          step="1"
          value={localSettings.defaultVolume}
          on:input={handleVolumeChange}
        />
        <div class="slider-labels">
          <span>0%</span>
          <span>50%</span>
          <span>100%</span>
        </div>
      </div>

      <div class="setting-group">
        <label class="setting-label" for="pan-slider">
          Default Track Pan: {localSettings.defaultPan > 0 ? `${localSettings.defaultPan}R` : localSettings.defaultPan < 0 ? `${Math.abs(localSettings.defaultPan)}L` : 'Center'}
        </label>
        <input
          id="pan-slider"
          type="range"
          class="setting-slider"
          min="-100"
          max="100"
          step="1"
          value={localSettings.defaultPan}
          on:input={handlePanChange}
        />
        <div class="slider-labels">
          <span>100L</span>
          <span>Center</span>
          <span>100R</span>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Track Type</h3>

      <div class="setting-group">
        <label class="setting-label">Default Track Type</label>
        <div class="track-type-selector">
          <button
            class="track-type-option"
            class:active={localSettings.defaultTrackType === 'audio'}
            on:click={() => handleTrackTypeChange('audio')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 18V5l12-2v13"/>
              <circle cx="6" cy="18" r="3"/>
              <circle cx="18" cy="16" r="3"/>
            </svg>
            <span class="track-type-label">Audio</span>
          </button>

          <button
            class="track-type-option"
            class:active={localSettings.defaultTrackType === 'midi'}
            on:click={() => handleTrackTypeChange('midi')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 5H3v14h6M15 5h6v14h-6M9 12h6"/>
            </svg>
            <span class="track-type-label">MIDI</span>
          </button>

          <button
            class="track-type-option"
            class:active={localSettings.defaultTrackType === 'instrument'}
            on:click={() => handleTrackTypeChange('instrument')}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 2v20M13 2v20M3 8h18M3 16h18"/>
            </svg>
            <span class="track-type-label">Instrument</span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Recording</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Auto-Arm on Selection</label>
            <p class="setting-description">Automatically arm track for recording when selected</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.autoArmOnSelection}
            on:click={handleAutoArmToggle}
            aria-label="Toggle auto-arm"
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
  .track-settings {
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

  .color-picker {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .color-swatch {
    width: 100%;
    aspect-ratio: 1;
    border: 2px solid var(--border-color, #333);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .color-swatch:hover {
    transform: scale(1.1);
    border-color: var(--border-hover, #444);
  }

  .color-swatch.active {
    border-color: #fff;
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.2);
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

  .track-type-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .track-type-option {
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

  .track-type-option:hover {
    border-color: var(--border-hover, #444);
  }

  .track-type-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .track-type-option svg {
    color: var(--text-secondary, #a0a0a0);
  }

  .track-type-option.active svg {
    color: var(--accent-color, #4a9eff);
  }

  .track-type-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
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
