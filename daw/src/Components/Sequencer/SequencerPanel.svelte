<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  import PlaybackControls from './PlaybackControls.svelte';
  import Timeline from './Timeline.svelte';
  import TrackRow from './TrackRow.svelte';

  interface Track {
    id: number;
    name: string;
    file_id: number;
    channel: number;
    muted: boolean;
    solo: boolean;
    volume: number;
    pan: number;
    color: string;
  }

  let tracks: Track[] = [];
  let isPlaying = false;
  let tempo = 120;
  let currentPosition = 0;
  let totalDuration = 0;
  let positionInterval: number;

  async function loadTracks() {
    try {
      tracks = await invoke<Track[]>('get_tracks');
      // Calculate total duration from tracks
      if (tracks.length > 0) {
        totalDuration = 120; // TODO: Calculate from track data
      }
    } catch (error) {
      console.error('Failed to load tracks:', error);
    }
  }

  async function updatePosition() {
    if (!isPlaying) return;

    try {
      const position = await invoke<{ current_tick: number }>('get_playback_position');
      // Convert ticks to seconds (simplified)
      currentPosition = position.current_tick / 480 * (60 / tempo);
    } catch (error) {
      console.error('Failed to get position:', error);
    }
  }

  function handlePlay() {
    isPlaying = true;
    positionInterval = setInterval(updatePosition, 100) as unknown as number;
  }

  function handleStop() {
    isPlaying = false;
    currentPosition = 0;
    if (positionInterval) {
      clearInterval(positionInterval);
    }
  }

  async function handleTrackUpdate(event: CustomEvent<{ trackId: number, properties: any }>) {
    const { trackId, properties } = event.detail;

    try {
      await invoke('update_track', { trackId, properties });
      await loadTracks();
    } catch (error) {
      console.error('Failed to update track:', error);
    }
  }

  async function handleTrackRemove(event: CustomEvent<{ trackId: number }>) {
    const { trackId } = event.detail;

    try {
      await invoke('remove_track', { trackId });
      await loadTracks();
    } catch (error) {
      console.error('Failed to remove track:', error);
    }
  }

  onMount(() => {
    loadTracks();
  });

  onDestroy(() => {
    if (positionInterval) {
      clearInterval(positionInterval);
    }
  });
</script>

<div class="sequencer-panel">
  <div class="sequencer-header">
    <h3>Sequencer</h3>
    <PlaybackControls
      {isPlaying}
      {tempo}
      {currentPosition}
      {totalDuration}
      on:play={handlePlay}
      on:stop={handleStop}
      on:tempoChange={(e) => tempo = e.detail.tempo}
    />
  </div>

  <div class="sequencer-content">
    <Timeline {currentPosition} {totalDuration} {tempo} />

    <div class="tracks-container">
      {#if tracks.length === 0}
        <div class="empty-sequencer">
          <p>No tracks in sequencer</p>
          <p class="hint">Drag files from search results to add tracks</p>
        </div>
      {:else}
        {#each tracks as track (track.id)}
          <TrackRow
            {track}
            {isPlaying}
            on:update={handleTrackUpdate}
            on:remove={handleTrackRemove}
          />
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .sequencer-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
  }

  .sequencer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #3d3d3d;
  }

  .sequencer-header h3 {
    margin: 0;
    font-size: 18px;
    color: #e0e0e0;
  }

  .sequencer-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tracks-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .empty-sequencer {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #808080;
  }

  .empty-sequencer p {
    margin: 4px 0;
    font-size: 14px;
  }

  .empty-sequencer .hint {
    font-size: 12px;
    color: #606060;
  }

  /* Custom scrollbar */
  .tracks-container::-webkit-scrollbar {
    width: 8px;
  }

  .tracks-container::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .tracks-container::-webkit-scrollbar-thumb {
    background: #3d3d3d;
    border-radius: 4px;
  }
</style>
