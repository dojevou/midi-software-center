<script lang="ts">
  import WindowBase from '../components/WindowBase.svelte';
  import ScoreView from '../components/ScoreView.svelte';
  import { notationState, notationActions } from '../stores/notationStore';
  import { databaseStore } from '../stores/databaseStore';
  import { derived } from 'svelte/store';
  import type { WindowId } from '../types';

  export let windowId: WindowId = 'score';

  let midiData: Uint8Array | null = null;
  let title = 'Score View';

  // Derived store for selected file
  const selectedFile = derived(databaseStore, ($store) => {
    if ($store.selectedFileId === null) return null;
    return $store.searchResults.find(f => f.id === $store.selectedFileId) || null;
  });

  // Load MIDI data when file is selected
  $: if ($selectedFile?.filepath) {
    loadMidiFile($selectedFile.filepath);
  }

  async function loadMidiFile(path: string) {
    try {
      const response = await fetch(`file://${path}`);
      const buffer = await response.arrayBuffer();
      midiData = new Uint8Array(buffer);
      title = $selectedFile?.filename || 'Score View';
    } catch (e) {
      console.error('Failed to load MIDI file:', e);
    }
  }

  function handleExport() {
    if (midiData) {
      notationActions.exportMusicXML(Array.from(midiData), title)
        .then(xml => {
          const blob = new Blob([xml], { type: 'application/xml' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `${title.replace(/[^a-z0-9]/gi, '_')}.musicxml`;
          a.click();
          URL.revokeObjectURL(url);
        });
    }
  }
</script>

<WindowBase {windowId} title="Score View" minWidth={600} minHeight={400}>
  <div class="score-window">
    <div class="toolbar">
      <button on:click={handleExport} disabled={!midiData}>
        Export MusicXML
      </button>
      <select bind:value={$notationState.quantizeLevel}>
        <option value="whole">Whole</option>
        <option value="half">Half</option>
        <option value="quarter">Quarter</option>
        <option value="eighth">8th</option>
        <option value="sixteenth">16th</option>
        <option value="32nd">32nd</option>
      </select>
    </div>
    <div class="content">
      <ScoreView {midiData} {title} width={800} height={500} showControls={true} />
    </div>
  </div>
</WindowBase>

<style>
  .score-window {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .toolbar {
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    gap: 8px;
  }
  .content {
    flex: 1;
    overflow: auto;
  }
</style>
