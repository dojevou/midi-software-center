<script lang="ts">
  import WindowBase from '../components/WindowBase.svelte';
  import { linkState, linkActions } from '../stores/linkStore';
  import { onMount, onDestroy } from 'svelte';
  import type { WindowId } from '../types';

  export let windowId: WindowId = 'link-sync';

  let refreshInterval: ReturnType<typeof setInterval>;

  onMount(() => {
    linkActions.refresh();
    refreshInterval = setInterval(() => {
      if ($linkState.enabled) {
        linkActions.refresh();
      }
    }, 100);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });
</script>

<WindowBase {windowId} title="Ableton Link" minWidth={350} minHeight={300}>
  <div class="link-window">
    <div class="status-card" class:enabled={$linkState.enabled}>
      <div class="toggle">
        <label>
          <input
            type="checkbox"
            checked={$linkState.enabled}
            on:change={(e) => linkActions.enable(e.currentTarget.checked)}
          />
          Link Enabled
        </label>
      </div>
      <div class="peers">
        <span class="count">{$linkState.numPeers}</span>
        <span class="label">Peers Connected</span>
      </div>
    </div>

    <div class="tempo-section">
      <label>
        Tempo
        <input
          type="number"
          min="20"
          max="300"
          step="0.1"
          value={$linkState.tempo}
          on:change={(e) => linkActions.setTempo(parseFloat(e.currentTarget.value))}
        />
        BPM
      </label>
    </div>

    <div class="quantum-section">
      <label>
        Quantum
        <input
          type="number"
          min="1"
          max="16"
          step="1"
          value={$linkState.quantum}
          on:change={(e) => linkActions.setQuantum(parseFloat(e.currentTarget.value))}
        />
        beats
      </label>
    </div>

    <div class="transport">
      <div class="beat-display">
        <span class="beat">{$linkState.beat.toFixed(2)}</span>
        <span class="phase">Phase: {$linkState.phase.toFixed(2)}</span>
      </div>
      <div class="controls">
        <button
          class:active={$linkState.isPlaying}
          on:click={() => $linkState.isPlaying ? linkActions.stop() : linkActions.start()}
        >
          {$linkState.isPlaying ? 'Stop' : 'Start'}
        </button>
      </div>
    </div>
  </div>
</WindowBase>

<style>
  .link-window {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .status-card {
    padding: 16px;
    border-radius: 8px;
    background: var(--bg-secondary);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .status-card.enabled {
    border: 2px solid #00cc00;
  }
  .peers {
    text-align: right;
  }
  .peers .count {
    font-size: 2em;
    font-weight: bold;
  }
  .peers .label {
    display: block;
    font-size: 0.8em;
    color: var(--text-secondary);
  }
  .tempo-section, .quantum-section {
    display: flex;
    gap: 8px;
  }
  input[type="number"] {
    width: 80px;
    padding: 4px 8px;
  }
  .transport {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--bg-secondary);
    border-radius: 8px;
  }
  .beat-display .beat {
    font-size: 2em;
    font-family: monospace;
  }
  .beat-display .phase {
    display: block;
    font-size: 0.9em;
    color: var(--text-secondary);
  }
  button.active {
    background: #ff4444;
  }
</style>
