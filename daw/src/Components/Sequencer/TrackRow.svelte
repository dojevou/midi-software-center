<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let track: {
    id: number;
    name: string;
    file_id: number;
    channel: number;
    muted: boolean;
    solo: boolean;
    volume: number;
    pan: number;
    color: string;
  };

  export let isPlaying = false;

  function handleMuteToggle() {
    dispatch('update', {
      trackId: track.id,
      properties: { muted: !track.muted }
    });
  }

  function handleSoloToggle() {
    dispatch('update', {
      trackId: track.id,
      properties: { solo: !track.solo }
    });
  }

  function handleVolumeChange() {
    dispatch('update', {
      trackId: track.id,
      properties: { volume: track.volume }
    });
  }

  function handlePanChange() {
    dispatch('update', {
      trackId: track.id,
      properties: { pan: track.pan }
    });
  }

  function handleRemove() {
    dispatch('remove', { trackId: track.id });
  }
</script>

<div class="track-row" style="border-left-color: {track.color}">
  <div class="track-header">
    <div class="track-info">
      <span class="track-name">{track.name}</span>
      <span class="track-channel">Ch {track.channel + 1}</span>
    </div>

    <div class="track-controls">
      <button
        class="control-btn"
        class:active={track.muted}
        on:click={handleMuteToggle}
        title="Mute"
      >
        M
      </button>

      <button
        class="control-btn"
        class:active={track.solo}
        on:click={handleSoloToggle}
        title="Solo"
      >
        S
      </button>

      <button
        class="control-btn remove"
        on:click={handleRemove}
        title="Remove track"
      >
        ✕
      </button>
    </div>
  </div>

  <div class="track-body">
    <div class="control-group">
      <label class="control-label">Volume</label>
      <input
        type="range"
        class="slider"
        bind:value={track.volume}
        on:change={handleVolumeChange}
        min="0"
        max="127"
      />
      <span class="control-value">{track.volume}</span>
    </div>

    <div class="control-group">
      <label class="control-label">Pan</label>
      <input
        type="range"
        class="slider"
        bind:value={track.pan}
        on:change={handlePanChange}
        min="0"
        max="127"
      />
      <span class="control-value">{track.pan - 64}</span>
    </div>
  </div>
</div>

<style>
  .track-row {
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-left-width: 4px;
    border-radius: 6px;
    margin-bottom: 8px;
    padding: 12px;
    transition: all 0.2s;
  }

  .track-row:hover {
    background: #353535;
    border-color: #4d4d4d;
  }

  .track-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .track-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .track-name {
    font-size: 14px;
    font-weight: 600;
    color: #e0e0e0;
  }

  .track-channel {
    font-size: 11px;
    color: #808080;
    font-family: 'Courier New', monospace;
  }

  .track-controls {
    display: flex;
    gap: 4px;
  }

  .control-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    color: #b0b0b0;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.2s;
  }

  .control-btn:hover {
    background: #3d3d3d;
  }

  .control-btn.active {
    background: #ff6b6b;
    border-color: #ff6b6b;
    color: white;
  }

  .control-btn.remove:hover {
    background: #ff6b6b;
    border-color: #ff6b6b;
    color: white;
  }

  .track-body {
    display: flex;
    gap: 16px;
  }

  .control-group {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .control-label {
    font-size: 11px;
    color: #808080;
    font-weight: 600;
    text-transform: uppercase;
    min-width: 50px;
  }

  .slider {
    flex: 1;
    -webkit-appearance: none;
    height: 4px;
    border-radius: 2px;
    background: #3d3d3d;
    outline: none;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
  }

  .slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
    border: none;
  }

  .control-value {
    font-size: 12px;
    color: #e0e0e0;
    font-weight: 600;
    font-family: 'Courier New', monospace;
    min-width: 30px;
    text-align: right;
  }
</style>
