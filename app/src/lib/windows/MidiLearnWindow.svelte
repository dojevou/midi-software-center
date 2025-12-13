<script lang="ts">
  import WindowBase from '../components/WindowBase.svelte';
  import MidiLearnOverlay from '../components/MidiLearnOverlay.svelte';
  import { learnState, learnActions } from '../stores/learnStore';
  import { onMount } from 'svelte';
  import type { WindowId } from '../types';

  export let windowId: WindowId = 'midi-learn';

  onMount(() => {
    learnActions.refreshMappings();
  });

  async function exportMappings() {
    const json = await learnActions.exportMappings();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'midi-mappings.json';
    a.click();
    URL.revokeObjectURL(url);
  }

  async function importMappings() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const text = await file.text();
        const count = await learnActions.importMappings(text);
        alert(`Imported ${count} mappings`);
      }
    };
    input.click();
  }
</script>

<WindowBase {windowId} title="MIDI Learn" minWidth={500} minHeight={400}>
  <div class="midi-learn-window">
    <div class="toolbar">
      <button on:click={exportMappings}>Export Mappings</button>
      <button on:click={importMappings}>Import Mappings</button>
      <button on:click={() => learnActions.refreshMappings()}>Refresh</button>
    </div>

    {#if $learnState.isLearning}
      <div class="learning-indicator">
        <div class="pulse"></div>
        <span>Learning: {$learnState.targetPath}</span>
        <button on:click={() => learnActions.cancelLearning()}>Cancel</button>
      </div>
    {/if}

    <div class="mappings-list">
      <h3>Active Mappings ({$learnState.mappings.length})</h3>
      <table>
        <thead>
          <tr>
            <th>Channel</th>
            <th>CC#</th>
            <th>Target</th>
            <th>Range</th>
            <th>Mode</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each $learnState.mappings as mapping}
            <tr>
              <td>{mapping.channel + 1}</td>
              <td>{mapping.ccNumber}</td>
              <td>{mapping.targetPath}</td>
              <td>{mapping.minValue} - {mapping.maxValue}</td>
              <td>{mapping.scalingMode}</td>
              <td>
                <button on:click={() => learnActions.removeMapping(mapping.id)}>×</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</WindowBase>

<style>
  .midi-learn-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
  }
  .toolbar {
    padding: 8px;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border-color);
  }
  .learning-indicator {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--accent-color);
    color: white;
    border-radius: 4px;
    margin: 0 8px;
  }
  .pulse {
    width: 12px;
    height: 12px;
    background: #ff4444;
    border-radius: 50%;
    animation: pulse 1s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  .mappings-list {
    flex: 1;
    padding: 0 8px;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th, td {
    padding: 8px;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
  }
</style>
