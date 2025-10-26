<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let isPlaying = false;
  export let tempo = 120;
  export let currentPosition = 0;
  export let totalDuration = 0;

  let isLooping = false;
  let metronomeEnabled = false;

  async function handlePlay() {
    try {
      await invoke('start_sequencer');
      isPlaying = true;
      dispatch('play');
    } catch (error) {
      console.error('Failed to start playback:', error);
    }
  }

  async function handleStop() {
    try {
      await invoke('stop_sequencer');
      isPlaying = false;
      dispatch('stop');
    } catch (error) {
      console.error('Failed to stop playback:', error);
    }
  }

  async function handleTempoChange() {
    try {
      await invoke('set_tempo', { bpm: tempo });
      dispatch('tempoChange', { tempo });
    } catch (error) {
      console.error('Failed to set tempo:', error);
    }
  }

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="playback-controls">
  <div class="transport-section">
    <button
      class="transport-btn"
      class:active={isPlaying}
      on:click={isPlaying ? handleStop : handlePlay}
      title={isPlaying ? 'Stop' : 'Play'}
    >
      {isPlaying ? '■' : '▶'}
    </button>

    <button
      class="transport-btn"
      on:click={handleStop}
      title="Stop"
      disabled={!isPlaying}
    >
      ⏹
    </button>

    <button
      class="transport-btn"
      class:active={isLooping}
      on:click={() => isLooping = !isLooping}
      title="Loop"
    >
      🔁
    </button>

    <button
      class="transport-btn"
      class:active={metronomeEnabled}
      on:click={() => metronomeEnabled = !metronomeEnabled}
      title="Metronome"
    >
      🎯
    </button>
  </div>

  <div class="position-section">
    <div class="time-display">
      {formatTime(currentPosition)} / {formatTime(totalDuration)}
    </div>

    <div class="progress-bar">
      <div
        class="progress-fill"
        style="width: {totalDuration > 0 ? (currentPosition / totalDuration) * 100 : 0}%"
      ></div>
    </div>
  </div>

  <div class="tempo-section">
    <label for="tempo-input" class="tempo-label">BPM</label>
    <input
      id="tempo-input"
      type="number"
      class="tempo-input"
      bind:value={tempo}
      on:change={handleTempoChange}
      min="30"
      max="300"
      step="1"
    />
  </div>
</div>

<style>
  .playback-controls {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 16px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
  }

  .transport-section {
    display: flex;
    gap: 8px;
  }

  .transport-btn {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 20px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .transport-btn:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #4a9eff;
    transform: scale(1.05);
  }

  .transport-btn.active {
    background: #4a9eff;
    border-color: #4a9eff;
    color: white;
  }

  .transport-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .position-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .time-display {
    font-size: 14px;
    color: #b0b0b0;
    font-weight: 600;
    font-family: 'Courier New', monospace;
  }

  .progress-bar {
    height: 6px;
    background: #1e1e1e;
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4a9eff, #357abd);
    transition: width 0.1s linear;
  }

  .tempo-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tempo-label {
    font-size: 12px;
    color: #808080;
    font-weight: 600;
    text-transform: uppercase;
  }

  .tempo-input {
    width: 70px;
    padding: 8px 12px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 16px;
    font-weight: 600;
    text-align: center;
    font-family: 'Courier New', monospace;
  }

  .tempo-input:focus {
    outline: none;
    border-color: #4a9eff;
  }
</style>
