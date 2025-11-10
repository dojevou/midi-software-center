<script lang="ts">
  import { onMount } from 'svelte';
  import { projectStore, projectActions } from '$lib/stores/projectStore';
  import { api } from '$lib/api';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import type { Track } from '$lib/types';

  let tracks: Track[] = [];

  // Reactive subscription
  $: tracks = $projectStore.tracks;

  // Computed normalized values for each track
  $: normalizedVolumes = tracks.map(track => track.volume / 127);
  $: normalizedPans = tracks.map(track => (track.pan / 127) * 2 - 1);

  // Load initial tracks
  onMount(async () => {
    // Tracks are already loaded via projectStore
  });

  async function updateVolume(index: number, volume: number) {
    const trackId = tracks[index].id;
    const denormalizedVolume = Math.round(volume * 127);
    await api.window.setChannelVolume(trackId, volume);
    await projectActions.updateTrack(trackId, { volume: denormalizedVolume });
  }

  async function updatePan(index: number, pan: number) {
    const trackId = tracks[index].id;
    const denormalizedPan = Math.round((pan + 1) / 2 * 127);
    await api.window.setChannelPan(trackId, pan);
    await projectActions.updateTrack(trackId, { pan: denormalizedPan });
  }

  async function toggleMute(trackId: number) {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      const newMuted = !track.muted;
      await api.window.setChannelMute(trackId, newMuted);
      await projectActions.updateTrack(trackId, { muted: newMuted });
    }
  }

  async function toggleSolo(trackId: number) {
    const track = tracks.find(t => t.id === trackId);
    if (track) {
      const newSoloed = !track.solo;
      await api.window.setChannelSolo(trackId, newSoloed);
      await projectActions.updateTrack(trackId, { solo: newSoloed });
    }
  }

  function formatVolume(volume: number): string {
    return `${Math.round(volume * 100)}%`;
  }

  function formatPan(pan: number): string {
    if (pan < -0.5) return 'L';
    if (pan > 0.5) return 'R';
    return 'C';
  }
</script>

<WindowBase windowId="mixer" title="Mixer" width={800} height={500} resizable={true}>
  <div class="mixer-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
    <div class="channels flex space-x-4 overflow-x-auto pb-4">
      {#each tracks as track, index (track.id)}
        <div class="channel-strip dark:bg-window-subtle p-3 rounded border dark:border-window-border w-20 flex flex-col items-center space-y-2">
          <!-- Track Name -->
          <div class="track-name text-center text-xs dark:text-gray-300 truncate w-full">
            {track.name}
          </div>

          <!-- Volume Fader -->
          <div class="volume-fader flex flex-col items-center space-y-1">
            <label class="volume-label text-xs dark:text-gray-400">Vol</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={normalizedVolumes[index]}
              on:input={(e) => updateVolume(index, parseFloat(e.currentTarget.value))}
              class="volume-slider dark:bg-input w-4 h-32"
            />
            <span class="volume-display text-xs dark:text-gray-300">{formatVolume(tracks[index]?.volume / 127)}</span>
          </div>

          <!-- Pan Knob -->
          <div class="pan-control flex flex-col items-center space-y-1">
            <label class="pan-label text-xs dark:text-gray-400">Pan</label>
            <input
              type="range"
              min="-1"
              max="1"
              step="0.01"
              value={normalizedPans[index]}
              on:input={(e) => updatePan(index, parseFloat(e.currentTarget.value))}
              class="pan-slider dark:bg-input w-16 h-2"
            />
            <span class="pan-display text-xs dark:text-gray-300">{formatPan(normalizedPans[index])}</span>
          </div>

          <!-- Mute/Solo Buttons -->
          <div class="controls flex flex-col space-y-1">
            <button
              class="mute-btn {track.muted ? 'dark:bg-error text-white' : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs"
              on:click={() => toggleMute(track.id)}
            >
              M
            </button>
            <button
              class="solo-btn {track.solo ? 'dark:bg-primary text-white' : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs"
              on:click={() => toggleSolo(track.id)}
            >
              S
            </button>
          </div>
        </div>
      {/each}
    </div>

    <!-- Master Section (placeholder) -->
    <div class="master dark:bg-menu p-3 rounded mt-auto">
      <h3 class="dark:text-gray-200 mb-2">Master</h3>
      <div class="flex items-center space-x-4">
        <div class="volume-master">
          <label class="dark:text-gray-400">Master Vol</label>
          <input type="range" min="0" max="1" step="0.01" value="1" class="dark:bg-input w-32" />
          <span class="dark:text-gray-300">100%</span>
        </div>
      </div>
    </div>
  </div>

  <style>
    .mixer-window {
      height: 100%;
    }
    .volume-slider {
      writing-mode: bt-lr; /* Vertical slider */
      -webkit-appearance: slider-vertical;
      width: 8px;
      height: 128px;
    }
    .channel-strip {
      min-width: 80px;
    }
  </style>
</WindowBase>