<script lang="ts">
  import {
    midiClockStore,
    transport,
    transportState,
    bpm,
    position,
    isPlaying
  } from '$lib/stores/midiClockStore';
  import { onMount, onDestroy } from 'svelte';

  export let compact = false;
  export let showPosition = true;
  export let showBpm = true;

  let bpmInput = 120;
  let isEditingBpm = false;
  let tapTimes: number[] | null = null;

  $: bpmInput = $bpm;

  function formatPosition(bar: number, beat: number, tick: number): string {
    return `${bar.toString().padStart(3, '0')}:${beat}:${tick.toString().padStart(2, '0')}`;
  }

  function formatTime(millis: number): string {
    const seconds = Math.floor(millis / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    const remainingMillis = millis % 1000;

    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}.${Math.floor(remainingMillis / 100)}`;
  }

  async function handleBpmChange() {
    isEditingBpm = false;
    if (bpmInput !== $bpm) {
      await midiClockStore.setBpm(bpmInput);
    }
  }

  function handleBpmKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      handleBpmChange();
    } else if (event.key === 'Escape') {
      isEditingBpm = false;
      bpmInput = $bpm;
    }
  }

  function tapTempo() {
    const now = Date.now();
    if (!tapTimes) {
      tapTimes = [now];
      return;
    }

    // Reset if more than 2 seconds since last tap
    if (now - tapTimes[tapTimes.length - 1] > 2000) {
      tapTimes = [now];
      return;
    }

    tapTimes.push(now);
    if (tapTimes.length > 8) {
      tapTimes.shift();
    }

    if (tapTimes.length >= 2) {
      const intervals: number[] = [];
      for (let i = 1; i < tapTimes.length; i++) {
        intervals.push(tapTimes[i] - tapTimes[i - 1]);
      }
      const avgInterval = intervals.reduce((a, b) => a + b, 0) / intervals.length;
      const calculatedBpm = Math.round(60000 / avgInterval);

      if (calculatedBpm >= 20 && calculatedBpm <= 300) {
        midiClockStore.setBpm(calculatedBpm);
      }
    }
  }

  onMount(() => {
    midiClockStore.startUpdates();
  });

  onDestroy(() => {
    midiClockStore.stopUpdates();
  });
</script>

<div class="transport-controls" class:compact role="toolbar" aria-label="Transport controls">
  <!-- Transport buttons -->
  <div class="transport-buttons">
    <button
      class="transport-btn"
      on:click={() => midiClockStore.stop()}
      aria-label="Stop"
      title="Stop (Home)"
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
        <rect x="6" y="6" width="12" height="12" />
      </svg>
    </button>

    {#if $isPlaying}
      <button
        class="transport-btn"
        on:click={() => midiClockStore.pause()}
        aria-label="Pause"
        title="Pause (Space)"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <rect x="6" y="5" width="4" height="14" />
          <rect x="14" y="5" width="4" height="14" />
        </svg>
      </button>
    {:else}
      <button
        class="transport-btn play"
        on:click={() => $transportState === 'Paused' ? midiClockStore.continue() : midiClockStore.play()}
        aria-label="Play"
        title="Play (Space)"
      >
        <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
          <polygon points="8,5 19,12 8,19" />
        </svg>
      </button>
    {/if}

    <button
      class="transport-btn record"
      class:active={$transportState === 'Recording'}
      aria-label="Record"
      aria-pressed={$transportState === 'Recording'}
      title="Record (R)"
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
        <circle cx="12" cy="12" r="7" />
      </svg>
    </button>
  </div>

  <!-- Position display -->
  {#if showPosition && !compact}
    <div class="position-display" aria-label="Position">
      <span class="position-bbt">{formatPosition($position.bar, $position.beat, $position.tick)}</span>
      <span class="position-time">{formatTime($position.millis)}</span>
    </div>
  {/if}

  <!-- BPM control -->
  {#if showBpm}
    <div class="bpm-control">
      {#if isEditingBpm}
        <input
          type="number"
          class="bpm-input"
          bind:value={bpmInput}
          on:blur={handleBpmChange}
          on:keydown={handleBpmKeydown}
          min="20"
          max="300"
          step="0.1"
          aria-label="BPM"
        />
      {:else}
        <button
          class="bpm-display"
          on:click={() => isEditingBpm = true}
          aria-label="BPM: {$bpm.toFixed(1)}. Click to edit"
        >
          {$bpm.toFixed(1)}
        </button>
      {/if}
      <span class="bpm-label">BPM</span>

      {#if !compact}
        <button
          class="tap-tempo"
          on:click={tapTempo}
          title="Tap Tempo"
          aria-label="Tap tempo"
        >
          TAP
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .transport-controls {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 12px;
    background: var(--panel-bg, #1a1a1a);
    border-radius: 6px;
  }

  .transport-controls.compact {
    gap: 8px;
    padding: 4px 8px;
  }

  .transport-buttons {
    display: flex;
    gap: 4px;
  }

  .transport-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: var(--text-color, #fff);
    cursor: pointer;
    transition: background-color 0.15s, transform 0.1s;
  }

  .compact .transport-btn {
    width: 28px;
    height: 28px;
  }

  .transport-btn:hover {
    background: var(--button-hover-bg, #444);
  }

  .transport-btn:active {
    transform: scale(0.95);
  }

  .transport-btn.play {
    color: var(--color-success, #22c55e);
  }

  .transport-btn.record {
    color: var(--text-muted, #666);
  }

  .transport-btn.record.active,
  .transport-btn.record:hover {
    color: var(--color-error, #ef4444);
  }

  .position-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 100px;
    font-family: monospace;
  }

  .position-bbt {
    font-size: 18px;
    font-weight: bold;
    letter-spacing: 1px;
  }

  .position-time {
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  .bpm-control {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .bpm-display {
    min-width: 60px;
    padding: 4px 8px;
    background: var(--input-bg, #222);
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    color: inherit;
    font-family: monospace;
    font-size: 16px;
    font-weight: bold;
    text-align: center;
    cursor: pointer;
    transition: border-color 0.15s;
  }

  .bpm-display:hover {
    border-color: var(--accent-color, #3b82f6);
  }

  .bpm-input {
    width: 60px;
    padding: 4px 8px;
    background: var(--input-bg, #222);
    border: 1px solid var(--accent-color, #3b82f6);
    border-radius: 4px;
    color: inherit;
    font-family: monospace;
    font-size: 16px;
    font-weight: bold;
    text-align: center;
  }

  .bpm-input:focus {
    outline: none;
  }

  .bpm-label {
    font-size: 11px;
    color: var(--text-muted, #888);
    text-transform: uppercase;
  }

  .tap-tempo {
    padding: 4px 10px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 11px;
    font-weight: bold;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .tap-tempo:hover {
    background: var(--button-hover-bg, #444);
  }

  .tap-tempo:active {
    background: var(--accent-color, #3b82f6);
  }
</style>
