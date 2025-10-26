<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Track } from '$lib/trusty/tracks';

  export let track: Track;

  const dispatch = createEventDispatcher<{
    nameChange: string;
    muteToggle: void;
    soloToggle: void;
    recordArmToggle: void;
    volumeChange: number;
    panChange: number;
  }>();

  let isEditingName = false;
  let editedName = track.name;

  function handleDoubleClick() {
    isEditingName = true;
    editedName = track.name;
  }

  function handleNameBlur() {
    if (editedName.trim() && editedName !== track.name) {
      dispatch('nameChange', editedName.trim());
    }
    isEditingName = false;
  }

  function handleNameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleNameBlur();
    } else if (e.key === 'Escape') {
      isEditingName = false;
    }
  }

  function handleMuteToggle() {
    dispatch('muteToggle');
  }

  function handleSoloToggle() {
    dispatch('soloToggle');
  }

  function handleRecordArmToggle() {
    dispatch('recordArmToggle');
  }

  function handleVolumeChange(e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value, 10);
    dispatch('volumeChange', value);
  }

  function handlePanChange(e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value, 10);
    dispatch('panChange', value);
  }

  // Convert volume (0-127) to dB visualization
  function volumeTodB(volume: number): string {
    if (volume === 0) return '-∞ dB';
    const dB = (20 * Math.log10(volume / 127)).toFixed(1);
    return `${dB} dB`;
  }

  // Convert pan (-64 to 63) to visual representation
  function panToDisplay(pan: number): string {
    if (pan === 0) return 'C';
    if (pan < 0) return `L${Math.abs(pan)}`;
    return `R${pan}`;
  }
</script>

<div class="track-header">
  <!-- Track Name -->
  <div class="track-name-section">
    {#if isEditingName}
      <input
        type="text"
        class="name-input"
        bind:value={editedName}
        on:blur={handleNameBlur}
        on:keydown={handleNameKeydown}
        autofocus
      />
    {:else}
      <div
        class="track-name"
        on:dblclick={handleDoubleClick}
        title="Double-click to edit name"
      >
        {track.name}
      </div>
    {/if}
  </div>

  <!-- Control Buttons -->
  <div class="track-controls">
    <!-- Mute Button -->
    <button
      class="control-btn mute-btn"
      class:active={track.muted}
      on:click={handleMuteToggle}
      title={track.muted ? 'Unmute' : 'Mute'}
      aria-label={track.muted ? 'Unmute track' : 'Mute track'}
    >
      M
    </button>

    <!-- Solo Button -->
    <button
      class="control-btn solo-btn"
      class:active={track.solo}
      on:click={handleSoloToggle}
      title={track.solo ? 'Unsolo' : 'Solo'}
      aria-label={track.solo ? 'Unsolo track' : 'Solo track'}
    >
      S
    </button>

    <!-- Record Arm Button -->
    <button
      class="control-btn record-btn"
      class:active={track.recordArmed}
      on:click={handleRecordArmToggle}
      title={track.recordArmed ? 'Disarm recording' : 'Arm recording'}
      aria-label={track.recordArmed ? 'Disarm recording' : 'Arm recording'}
    >
      ●
    </button>
  </div>

  <!-- Volume Fader -->
  <div class="volume-section">
    <label for="volume-{track.id}" class="control-label">Vol</label>
    <input
      id="volume-{track.id}"
      type="range"
      class="volume-slider"
      min="0"
      max="127"
      value={track.volume}
      on:input={handleVolumeChange}
      title="Volume: {volumeTodB(track.volume)}"
      aria-label="Volume control"
    />
    <div class="volume-display">{volumeTodB(track.volume)}</div>
  </div>

  <!-- Pan Knob Display -->
  <div class="pan-section">
    <label for="pan-{track.id}" class="control-label">Pan</label>
    <input
      id="pan-{track.id}"
      type="range"
      class="pan-slider"
      min="-64"
      max="63"
      value={track.pan}
      on:input={handlePanChange}
      title="Pan: {panToDisplay(track.pan)}"
      aria-label="Pan control"
    />
    <div class="pan-display">{panToDisplay(track.pan)}</div>
  </div>
</div>

<style>
  .track-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--bg-secondary, #2a2a2a);
    border-right: 1px solid var(--border-color, #444);
    min-height: 60px;
    font-family: var(--font-mono, 'Monaco', monospace);
    font-size: 12px;
  }

  /* Track Name */
  .track-name-section {
    flex: 0 0 120px;
    min-width: 100px;
  }

  .track-name {
    padding: 4px 8px;
    background: var(--bg-tertiary, #1f1f1f);
    border: 1px solid var(--border-color, #444);
    border-radius: 3px;
    cursor: text;
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary, #e0e0e0);
    font-weight: 500;
  }

  .track-name:hover {
    background: var(--bg-hover, #252525);
    border-color: var(--accent, #00d4ff);
  }

  .name-input {
    width: 100%;
    padding: 4px 8px;
    background: var(--bg-tertiary, #1f1f1f);
    border: 2px solid var(--accent, #00d4ff);
    border-radius: 3px;
    color: var(--text-primary, #e0e0e0);
    font-family: inherit;
    font-size: inherit;
    font-weight: 500;
  }

  .name-input:focus {
    outline: none;
    border-color: var(--accent-bright, #00ffff);
  }

  /* Control Buttons */
  .track-controls {
    display: flex;
    gap: 4px;
    flex: 0 0 auto;
  }

  .control-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    background: var(--bg-tertiary, #1f1f1f);
    border: 1px solid var(--border-color, #444);
    border-radius: 3px;
    color: var(--text-secondary, #999);
    font-weight: bold;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .control-btn:hover {
    background: var(--bg-hover, #252525);
    border-color: var(--border-hover, #666);
  }

  .control-btn.active {
    background: var(--accent, #00d4ff);
    color: var(--bg-primary, #000);
    border-color: var(--accent-bright, #00ffff);
  }

  .mute-btn.active {
    background: #ff9800;
  }

  .solo-btn.active {
    background: #ffeb3b;
    color: #000;
  }

  .record-btn.active {
    background: #ff4444;
    color: #fff;
  }

  /* Volume Section */
  .volume-section {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
  }

  .control-label {
    font-size: 10px;
    font-weight: bold;
    color: var(--text-secondary, #999);
    text-transform: uppercase;
    min-width: 24px;
  }

  .volume-slider {
    width: 60px;
    height: 4px;
    cursor: pointer;
    -webkit-appearance: slider-horizontal;
    appearance: slider-horizontal;
  }

  .volume-display {
    min-width: 42px;
    text-align: right;
    color: var(--text-secondary, #999);
    font-size: 10px;
  }

  /* Pan Section */
  .pan-section {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
  }

  .pan-slider {
    width: 60px;
    height: 4px;
    cursor: pointer;
    -webkit-appearance: slider-horizontal;
    appearance: slider-horizontal;
  }

  .pan-display {
    min-width: 28px;
    text-align: center;
    color: var(--text-secondary, #999);
    font-size: 10px;
    font-weight: bold;
  }

  /* Dark theme input styling */
  input[type='range'] {
    background: linear-gradient(to right, #333, #555);
    border-radius: 2px;
  }

  input[type='range']::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent, #00d4ff);
    border: 1px solid var(--accent-bright, #00ffff);
    border-radius: 50%;
    cursor: pointer;
  }

  input[type='range']::-moz-range-thumb {
    width: 12px;
    height: 12px;
    background: var(--accent, #00d4ff);
    border: 1px solid var(--accent-bright, #00ffff);
    border-radius: 50%;
    cursor: pointer;
  }
</style>
