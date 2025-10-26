<script lang="ts">
  import {
    isPlaying,
    tempo,
    playbackPosition,
    togglePlayback,
    stopPlayback,
    setTempo,
    canPlay,
  } from '../stores';

  let tempoInput = $tempo;

  function handleTempoChange() {
    if (tempoInput >= 20 && tempoInput <= 300) {
      setTempo(tempoInput);
    }
  }

  function formatPosition(pos: typeof $playbackPosition) {
    return `${pos.current_bar}:${pos.current_beat}:${pos.current_tick}`;
  }
</script>

<div class="transport">
  <div class="transport-controls">
    <button
      class="btn-play"
      on:click={togglePlayback}
      disabled={!$canPlay}
      title={$canPlay ? ($isPlaying ? 'Pause' : 'Play') : 'Connect MIDI and add tracks first'}
    >
      {$isPlaying ? '⏸️' : '▶️'}
    </button>

    <button class="btn-stop" on:click={stopPlayback} disabled={!$isPlaying}> ⏹️ </button>
  </div>

  <div class="transport-info">
    <div class="tempo-control">
      <label for="tempo">Tempo:</label>
      <input
        id="tempo"
        type="number"
        bind:value={tempoInput}
        on:change={handleTempoChange}
        min="20"
        max="300"
        step="1"
      />
      <span class="unit">BPM</span>
    </div>

    <div class="position-display">
      <span class="label">Position:</span>
      <span class="value">{formatPosition($playbackPosition)}</span>
    </div>
  </div>
</div>

<style>
  .transport {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
  }

  .transport-controls {
    display: flex;
    gap: 0.5rem;
  }

  .btn-play,
  .btn-stop {
    width: 48px;
    height: 48px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
    font-size: 1.5rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-play:not(:disabled):hover,
  .btn-stop:not(:disabled):hover {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
    transform: translateY(-2px);
  }

  .btn-play:disabled,
  .btn-stop:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .transport-info {
    display: flex;
    gap: 2rem;
    align-items: center;
  }

  .tempo-control {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tempo-control label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .tempo-control input {
    width: 60px;
    padding: 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 1rem;
    text-align: center;
  }

  .tempo-control input:focus {
    outline: none;
    border-color: #ff3e00;
  }

  .unit {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .position-display {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: 'Monaco', 'Courier New', monospace;
  }

  .position-display .label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .position-display .value {
    font-size: 1.125rem;
    color: #ff3e00;
    font-weight: 600;
  }

  .position-display .percentage {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.5);
  }
</style>
