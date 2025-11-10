<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { playbackStore, playbackActions, formattedPosition } from '$lib/stores/playbackStore';
  import { projectStore, projectActions } from '$lib/stores/projectStore';
  import { api } from '$lib/api';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import type { Track } from '$lib/types';
  import { formatBPM } from '$lib/utils/formatters';

  let tracks: Track[] = [];
  let selectedTrackId: number | null = null;
  let tempoInput: number;
  let timeSignature: [number, number] = [4, 4];

  // Reactive subscriptions
  $: tracks = $projectStore.tracks;
  $: selectedTrackId = $projectStore.selectedTrackId;
  $: tempoInput = $playbackStore.tempo;
  $: timeSignature = $playbackStore.timeSignature;

  // Load initial data
  onMount(async () => {
    await projectActions.loadTracks();
    const state = $playbackStore;
    tempoInput = state.tempo;
    timeSignature = state.timeSignature;
  });

  async function handlePlay() {
    await playbackActions.play();
  }

  async function handlePause() {
    await playbackActions.pause();
  }

  async function handleStop() {
    await playbackActions.stop();
  }

  async function handleTempoChange() {
    await playbackActions.setTempo(tempoInput);
  }

  async function addTrack() {
    // For demo, add empty track; in full impl, select from database
    const newId = await api.window.addWindowTrack('New Track');
    // Sync with project store if needed
    projectActions.markUnsaved();
  }

  function selectTrack(id: number) {
    projectActions.selectTrack(id);
  }

  async function toggleMute(trackId: number) {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      await projectActions.updateTrack(trackId, { muted: !track.muted });
    }
  }

  async function toggleSolo(trackId: number) {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      await projectActions.updateTrack(trackId, { solo: !track.solo });
    }
  }

  async function removeTrack(trackId: number) {
    await projectActions.removeTrack(trackId);
  }
</script>

<WindowBase windowId="daw" title="DAW" width={1000} height={600} resizable={true}>
  <div class="daw-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
    <!-- Transport Bar -->
    <div class="transport-bar dark:bg-menu p-3 rounded mb-4 flex items-center justify-between">
      <div class="controls flex space-x-2">
        <button
          class="play-btn dark:bg-primary dark:text-white px-4 py-2 rounded hover:dark:bg-primary-dark"
          on:click={handlePlay}
          disabled={$playbackStore.isPlaying || $playbackStore.isPaused}
        >
          Play
        </button>
        <button
          class="pause-btn dark:bg-secondary dark:text-white px-4 py-2 rounded hover:dark:bg-secondary-dark"
          on:click={handlePause}
          disabled={!$playbackStore.isPlaying}
        >
          Pause
        </button>
        <button
          class="stop-btn dark:bg-error dark:text-white px-4 py-2 rounded hover:dark:bg-error-dark"
          on:click={handleStop}
          disabled={!$playbackStore.isPlaying && !$playbackStore.isPaused}
        >
          Stop
        </button>
      </div>

      <div class="position dark:text-gray-300">
        Position: {$formattedPosition}
      </div>

      <div class="tempo-control flex items-center space-x-2">
        <label class="dark:text-gray-300">Tempo:</label>
        <input
          type="number"
          bind:value={tempoInput}
          min={20}
          max={300}
          class="dark:bg-input dark:text-app-text px-2 py-1 rounded w-20"
          on:blur={handleTempoChange}
        />
        <span>{formatBPM(tempoInput)}</span>
        <button on:click={() => tempoInput = Math.max(20, tempoInput - 1)} class="dark:text-gray-300">-</button>
        <button on:click={() => tempoInput = Math.min(300, tempoInput + 1)} class="dark:text-gray-300">+</button>
      </div>

      <div class="time-sig dark:text-gray-300">
        Time: {timeSignature[0]}/{timeSignature[1]}
      </div>
    </div>

    <!-- Track List -->
    <div class="track-list flex-1 overflow-auto dark:bg-window-subtle rounded border dark:border-window-border">
      <div class="track-header dark:bg-menu p-2 flex font-medium">
        <div class="w-8"></div>
        <div class="flex-1">Track</div>
        <div class="w-16 text-center">Mute</div>
        <div class="w-16 text-center">Solo</div>
        <div class="w-12 text-center">Actions</div>
      </div>
      {#each tracks as track (track.id)}
        <div
          class="track-row dark:bg-window hover:dark:bg-hover p-2 flex items-center border-b dark:border-window-border"
          class:selected={$selectedTrackId === track.id}
        >
          <div class="w-8">
            <input type="checkbox" bind:group={selectedTrackId} value={track.id} on:change={() => selectTrack(track.id)} />
          </div>
          <div class="flex-1">{track.name}</div>
          <div class="w-16 text-center">
            <button
              class="mute-btn {track.muted ? 'dark:bg-error' : 'dark:bg-success'} text-white px-2 py-1 rounded text-xs"
              on:click={() => toggleMute(track.id)}
            >
              M
            </button>
          </div>
          <div class="w-16 text-center">
            <button
              class="solo-btn {track.solo ? 'dark:bg-primary' : 'dark:bg-secondary'} text-white px-2 py-1 rounded text-xs"
              on:click={() => toggleSolo(track.id)}
            >
              S
            </button>
          </div>
          <div class="w-12 text-center">
            <button
              class="delete-btn dark:text-error hover:dark:text-error-dark px-1 py-1 rounded"
              on:click={() => removeTrack(track.id)}
              title="Delete Track"
            >
              ×
            </button>
          </div>
        </div>
      {/each}
      <div class="add-track dark:bg-window p-2">
        <button
          class="add-btn dark:bg-primary dark:text-white px-4 py-2 rounded hover:dark:bg-primary-dark"
          on:click={addTrack}
        >
          + Add Track
        </button>
      </div>
    </div>
  </div>

  <style>
    .daw-window {
      height: 100%;
    }
    .selected {
      background-color: var(--primary-color) !important;
      color: white !important;
    }
    .play-btn:disabled, .pause-btn:disabled, .stop-btn:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  </style>
</WindowBase>