<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { playbackStore, formattedPosition } from '$lib/stores/playbackStore';
  import { isPlayingOrPaused } from '$lib/stores/playbackStore';
  import type { PlaybackState } from '$lib/stores/playbackStore';

  let cpuUsage = 0;
  let ramUsage = 0;
  let pollInterval: NodeJS.Timeout | null = null;

  // Subscribe to playback store
  $: playback = $playbackStore;
  $: position = $formattedPosition;
  $: isActive = $isPlayingOrPaused;

  // Poll CPU/RAM every 1s (placeholder - assumes api.system.cpuUsage() and api.system.ramUsage() exist)
  onMount(() => {
    pollInterval = setInterval(async () => {
      try {
        // Placeholder API calls - replace with actual Tauri API when implemented
        // const cpu = await api.system.cpuUsage();
        // const ram = await api.system.ramUsage();
        cpuUsage = Math.floor(Math.random() * 100); // Mock for now
        ramUsage = Math.floor(Math.random() * 100); // Mock for now
      } catch (error) {
        console.error('Failed to poll system usage:', error);
        cpuUsage = 0;
        ramUsage = 0;
      }
    }, 1000);

    return () => {
      if (pollInterval) clearInterval(pollInterval);
    };
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
  });

  function formatPosition(playback: PlaybackState): string {
    const { current_bar, current_beat } = playback.position;
    return `${current_bar + 1}.${current_beat + 1}`;
  }
</script>

<div class="status-bar dark:bg-window dark:border-window-border dark:text-app-text flex items-center justify-between px-4 py-2 text-sm">
  <!-- Playback Position -->
  <div class="flex items-center space-x-4">
    <span class="font-mono">{formatPosition(playback)}</span>
    <span class="text-xs opacity-70">BPM: {playback.tempo}</span>
  </div>

  <!-- Status Icon -->
  <div class="flex items-center space-x-2">
    {#if isActive}
      <span class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></span>
      <span class="text-xs">Playing</span>
    {:else}
      <span class="w-3 h-3 bg-gray-500 rounded-full"></span>
      <span class="text-xs">Stopped</span>
    {/if}
  </div>

  <!-- System Usage -->
  <div class="flex items-center space-x-4">
    <span class="text-xs opacity-70">CPU: {cpuUsage}%</span>
    <span class="text-xs opacity-70">RAM: {ramUsage}%</span>
  </div>
</div>

<style>
  .status-bar {
    border-top: 1px solid var(--window-border);
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 1000;
  }
</style>