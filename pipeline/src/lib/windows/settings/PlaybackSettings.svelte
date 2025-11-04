<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    save: PlaybackSettingsData;
    cancel: void;
  }>();

  export interface PlaybackSettingsData {
    metronomeEnabled: boolean;
    clickSound: 'beep' | 'cowbell' | 'wood' | 'electric';
    metronomeVolume: number;
    clickOnBeat1: boolean;
    clickOnOtherBeats: boolean;
    backingTrackPath: string;
    backingTrackVolume: number;
  }

  export let settings: PlaybackSettingsData = {
    metronomeEnabled: true,
    clickSound: 'beep',
    metronomeVolume: 75,
    clickOnBeat1: true,
    clickOnOtherBeats: true,
    backingTrackPath: '',
    backingTrackVolume: 50
  };

  let localSettings: PlaybackSettingsData = { ...settings };
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let previewingSound: boolean = false;

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleMetronomeToggle() {
    localSettings.metronomeEnabled = !localSettings.metronomeEnabled;
    localSettings = localSettings;
  }

  function handleClickSoundChange(sound: 'beep' | 'cowbell' | 'wood' | 'electric') {
    localSettings.clickSound = sound;
    localSettings = localSettings;
    handlePreviewSound();
  }

  async function handlePreviewSound() {
    previewingSound = true;
    // In real implementation, would play the selected click sound
    await new Promise(resolve => setTimeout(resolve, 500));
    previewingSound = false;
  }

  function handleMetronomeVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.metronomeVolume = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleClickOnBeat1Toggle() {
    localSettings.clickOnBeat1 = !localSettings.clickOnBeat1;
    localSettings = localSettings;
  }

  function handleClickOnOtherBeatsToggle() {
    localSettings.clickOnOtherBeats = !localSettings.clickOnOtherBeats;
    localSettings = localSettings;
  }

  function handleBackingTrackVolume Change(event: Event) {
    const target = event.target as HTMLInputElement;
    localSettings.backingTrackVolume = parseInt(target.value);
    localSettings = localSettings;
  }

  function handleSelectBackingTrack() {
    // In real implementation, would open file picker via Tauri
    const mockPath = '/path/to/backing/track.mid';
    localSettings.backingTrackPath = mockPath;
    localSettings = localSettings;
  }

  function handleClearBackingTrack() {
    localSettings.backingTrackPath = '';
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
    const defaults: PlaybackSettingsData = {
      metronomeEnabled: true,
      clickSound: 'beep',
      metronomeVolume: 75,
      clickOnBeat1: true,
      clickOnOtherBeats: true,
      backingTrackPath: '',
      backingTrackVolume: 50
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

<div class="playback-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Metronome</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Enable Metronome</label>
            <p class="setting-description">Play metronome click during playback and recording</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.metronomeEnabled}
            on:click={handleMetronomeToggle}
            aria-label="Toggle metronome"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>

      {#if localSettings.metronomeEnabled}
        <div class="setting-group">
          <label class="setting-label">Click Sound</label>
          <div class="click-sound-selector">
            <button
              class="sound-option"
              class:active={localSettings.clickSound === 'beep'}
              on:click={() => handleClickSoundChange('beep')}
              disabled={previewingSound}
            >
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 18V5l12-2v13"/>
                <circle cx="6" cy="18" r="3"/>
                <circle cx="18" cy="16" r="3"/>
              </svg>
              <span>Beep</span>
            </button>

            <button
              class="sound-option"
              class:active={localSettings.clickSound === 'cowbell'}
              on:click={() => handleClickSoundChange('cowbell')}
              disabled={previewingSound}
            >
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2l8 4v4c0 6-8 10-8 10s-8-4-8-10V6z"/>
              </svg>
              <span>Cowbell</span>
            </button>

            <button
              class="sound-option"
              class:active={localSettings.clickSound === 'wood'}
              on:click={() => handleClickSoundChange('wood')}
              disabled={previewingSound}
            >
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
              </svg>
              <span>Wood Block</span>
            </button>

            <button
              class="sound-option"
              class:active={localSettings.clickSound === 'electric'}
              on:click={() => handleClickSoundChange('electric')}
              disabled={previewingSound}
            >
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
              </svg>
              <span>Electric</span>
            </button>
          </div>
        </div>

        <div class="setting-group">
          <label class="setting-label" for="metronome-volume">
            Metronome Volume: {localSettings.metronomeVolume}%
          </label>
          <input
            id="metronome-volume"
            type="range"
            class="setting-slider"
            min="0"
            max="100"
            step="1"
            value={localSettings.metronomeVolume}
            on:input={handleMetronomeVolumeChange}
          />
          <div class="slider-labels">
            <span>0%</span>
            <span>50%</span>
            <span>100%</span>
          </div>
        </div>

        <div class="setting-group">
          <div class="setting-row">
            <div>
              <label class="setting-label">Click on Beat 1</label>
              <p class="setting-description">Accent the first beat of each measure</p>
            </div>
            <button
              class="toggle-btn"
              class:active={localSettings.clickOnBeat1}
              on:click={handleClickOnBeat1Toggle}
              aria-label="Toggle click on beat 1"
            >
              <span class="toggle-slider"></span>
            </button>
          </div>
        </div>

        <div class="setting-group">
          <div class="setting-row">
            <div>
              <label class="setting-label">Click on Other Beats</label>
              <p class="setting-description">Play click on beats 2, 3, 4, etc.</p>
            </div>
            <button
              class="toggle-btn"
              class:active={localSettings.clickOnOtherBeats}
              on:click={handleClickOnOtherBeatsToggle}
              aria-label="Toggle click on other beats"
            >
              <span class="toggle-slider"></span>
            </button>
          </div>
        </div>
      {/if}
    </section>

    <section class="settings-section">
      <h3 class="section-title">Backing Track</h3>

      <div class="setting-group">
        <label class="setting-label">Backing Track File</label>
        {#if localSettings.backingTrackPath}
          <div class="file-display">
            <svg class="file-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            <span class="file-path">{localSettings.backingTrackPath}</span>
            <button class="btn btn-small btn-secondary" on:click={handleClearBackingTrack}>
              Clear
            </button>
          </div>
        {:else}
          <p class="no-file-message">No backing track selected</p>
        {/if}
        <button class="btn btn-secondary" on:click={handleSelectBackingTrack}>
          Select File
        </button>
      </div>

      {#if localSettings.backingTrackPath}
        <div class="setting-group">
          <label class="setting-label" for="backing-track-volume">
            Backing Track Volume: {localSettings.backingTrackVolume}%
          </label>
          <input
            id="backing-track-volume"
            type="range"
            class="setting-slider"
            min="0"
            max="100"
            step="1"
            value={localSettings.backingTrackVolume}
            on:input={handleBackingTrackVolumeChange}
          />
          <div class="slider-labels">
            <span>0%</span>
            <span>50%</span>
            <span>100%</span>
          </div>
        </div>
      {/if}
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
  .playback-settings {
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

  .click-sound-selector {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }

  .sound-option {
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

  .sound-option:hover:not(:disabled) {
    border-color: var(--border-hover, #444);
  }

  .sound-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .sound-option:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .sound-option svg {
    color: var(--text-secondary, #a0a0a0);
  }

  .sound-option.active svg {
    color: var(--accent-color, #4a9eff);
  }

  .sound-option span {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
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

  .file-display {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 0.75rem;
  }

  .file-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .file-path {
    flex: 1;
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .no-file-message {
    color: var(--text-secondary, #a0a0a0);
    font-size: 0.9rem;
    font-style: italic;
    margin: 0 0 0.75rem 0;
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
</style>
