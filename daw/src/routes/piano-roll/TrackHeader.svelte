<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let trackId: string;
  export let trackName: string;
  export let volume: number; // 0-127
  export let pan: number; // -1 to 1
  export let muted: boolean = false;
  export let solo: boolean = false;

  const dispatch = createEventDispatcher<{
    updateVolume: number;
    updatePan: number;
    toggleMute: void;
    toggleSolo: void;
    deleteTrack: void;
  }>();

  function handleVolumeChange(e: Event) {
    const value = parseInt((e.target as HTMLInputElement).value);
    dispatch('updateVolume', value);
  }

  function handlePanChange(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    dispatch('updatePan', value);
  }

  function handleMuteToggle() {
    dispatch('toggleMute');
  }

  function handleSoloToggle() {
    dispatch('toggleSolo');
  }

  function handleDeleteTrack() {
    if (confirm(`Delete track "${trackName}"?`)) {
      dispatch('deleteTrack');
    }
  }

  // Convert volume (0-127) to dB display (-∞ to 0)
  function getVolumeDisplay(): string {
    if (volume === 0) return '-∞';
    // Simple approximation: log scale
    const dB = 20 * Math.log10(volume / 127);
    return dB.toFixed(1);
  }

  // Convert pan (-1 to 1) to display string (L/R)
  function getPanDisplay(): string {
    if (Math.abs(pan) < 0.05) return 'C';
    if (pan < 0) return `${Math.abs(pan * 100).toFixed(0)}L`;
    return `${(pan * 100).toFixed(0)}R`;
  }
</script>

<div class="track-header" data-track-id={trackId}>
  <!-- Track Name -->
  <div class="track-info">
    <div class="track-name">{trackName}</div>
    <div class="channel-label">Ch {parseInt(trackId) % 16}</div>
  </div>

  <!-- Control Buttons -->
  <div class="controls">
    <!-- Mute Button -->
    <button
      class="control-btn mute-btn"
      class:active={muted}
      on:click={handleMuteToggle}
      title="Mute track"
    >
      M
    </button>

    <!-- Solo Button -->
    <button
      class="control-btn solo-btn"
      class:active={solo}
      on:click={handleSoloToggle}
      title="Solo track"
    >
      S
    </button>

    <!-- Delete Button -->
    <button
      class="control-btn delete-btn"
      on:click={handleDeleteTrack}
      title="Delete track"
    >
      ✕
    </button>
  </div>

  <!-- Volume Control -->
  <div class="fader-section">
    <label for="volume-{trackId}">Volume</label>
    <input
      id="volume-{trackId}"
      type="range"
      min="0"
      max="127"
      value={volume}
      on:input={handleVolumeChange}
      class="fader volume-fader"
      title="Volume: {getVolumeDisplay()} dB"
    />
    <div class="fader-value">{getVolumeDisplay()} dB</div>
  </div>

  <!-- Pan Control -->
  <div class="fader-section">
    <label for="pan-{trackId}">Pan</label>
    <input
      id="pan-{trackId}"
      type="range"
      min="-1"
      max="1"
      step="0.01"
      value={pan}
      on:input={handlePanChange}
      class="fader pan-fader"
      title="Pan: {getPanDisplay()}"
    />
    <div class="fader-value">{getPanDisplay()}</div>
  </div>
</div>

<style>
  .track-header {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
    border: 1px solid #3a3a52;
    border-radius: 6px;
    min-width: 180px;
  }

  .track-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .track-name {
    font-weight: 600;
    color: #e0e0ff;
    font-size: 0.95rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .channel-label {
    font-size: 0.75rem;
    color: #888899;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    justify-content: space-between;
  }

  .control-btn {
    flex: 1;
    padding: 0.5rem;
    background: #2d2d44;
    border: 1px solid #3a3a52;
    color: #888899;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.85rem;
    transition: all 0.2s ease;
  }

  .control-btn:hover {
    background: #3a3a52;
    color: #b0b0cc;
  }

  .control-btn.active {
    background: #4a9eff;
    border-color: #2d7fff;
    color: #ffffff;
  }

  .delete-btn {
    background: #3a2a2a;
    border-color: #5a3a3a;
  }

  .delete-btn:hover {
    background: #4a3a3a;
    color: #ff7777;
  }

  .fader-section {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .fader-section label {
    font-size: 0.75rem;
    color: #888899;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .fader {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: #2d2d44;
    outline: none;
    -webkit-appearance: none;
    appearance: none;
    cursor: pointer;
  }

  .fader::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
    border: 1px solid #2d7fff;
    transition: background 0.2s ease;
  }

  .fader::-webkit-slider-thumb:hover {
    background: #6bb3ff;
  }

  .fader::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
    border: 1px solid #2d7fff;
    transition: background 0.2s ease;
  }

  .fader::-moz-range-thumb:hover {
    background: #6bb3ff;
  }

  .volume-fader::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #2d2d44, #4a9eff);
    height: 6px;
    border-radius: 3px;
  }

  .fader-value {
    font-size: 0.75rem;
    color: #888899;
    text-align: center;
  }

  @media (prefers-color-scheme: light) {
    .track-header {
      background: linear-gradient(135deg, #f5f5f5 0%, #efefef 100%);
      border-color: #d0d0d0;
    }

    .track-name {
      color: #222222;
    }

    .channel-label {
      color: #666666;
    }

    .control-btn {
      background: #e8e8e8;
      border-color: #d0d0d0;
      color: #666666;
    }

    .control-btn:hover {
      background: #d8d8d8;
      color: #333333;
    }

    .fader-section label {
      color: #666666;
    }

    .fader {
      background: #e8e8e8;
    }

    .fader-value {
      color: #666666;
    }
  }
</style>
