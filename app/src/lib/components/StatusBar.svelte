<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { playbackStore } from '$lib/stores/playbackStore';

  let cpuUsage = 0;
  let ramUsage = 0;
  let pollInterval: NodeJS.Timeout | null = null;

  // Reactive from playback store
  $: position = $playbackStore.position;
  $: tempo = $playbackStore.tempo;
  $: timeSignature = $playbackStore.timeSignature;
  $: isPlaying = $playbackStore.isPlaying;
  $: isPaused = $playbackStore.isPaused;

  // Poll system usage every 2s
  onMount(() => {
    pollInterval = setInterval(async () => {
      try {
        // Mock CPU and RAM for now - replace with actual Tauri invoke
        cpuUsage = Math.floor(Math.random() * 100);
        ramUsage = Math.floor(Math.random() * 1024 * 256); // Mock MB
      } catch (error) {
        console.error('Failed to poll system usage:', error);
        cpuUsage = 0;
        ramUsage = 0;
      }
    }, 2000);

    return () => {
      if (pollInterval) {
        clearInterval(pollInterval);
      }
    };
  });

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });

  function formatPosition() {
    return `${position.current_bar + 1}.${position.current_beat + 1}.${position.current_tick}`;
  }

  function formatTimeSig() {
    return `${timeSignature[0]}/${timeSignature[1]}`;
  }

  function formatRAM(mb: number) {
    return `${mb} MB`;
  }
</script>

<div
  class="status-bar dark:bg-menu dark:text-gray-300 flex items-center justify-between px-4 py-2 text-sm"
>
  <!-- Position, BPM, Time Sig -->
  <div class="flex items-center space-x-4">
    <span class="font-mono">Position: {formatPosition()}</span>
    <span>BPM: {tempo}</span>
    <span>Time Sig: {formatTimeSig()}</span>
  </div>

  <!-- Status Icon -->
  <div class="flex items-center space-x-2">
    {#if isPlaying || isPaused}
      <span class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></span>
      <span class="text-xs">{isPlaying ? 'Playing' : 'Paused'}</span>
    {:else}
      <span class="w-3 h-3 bg-gray-500 rounded-full"></span>
      <span class="text-xs">Stopped</span>
    {/if}
  </div>

  <!-- System Usage with Indicators -->
  <div class="flex items-center space-x-4">
    <span class="flex items-center space-x-1">
      <span class="w-2 h-2 bg-green-500 rounded-full"></span>
      <span>CPU: {cpuUsage}%</span>
    </span>
    <span class="flex items-center space-x-1">
      <span class="w-2 h-2 bg-green-500 rounded-full"></span>
      <span>RAM: {formatRAM(ramUsage)}</span>
    </span>
  </div>
</div>

<style>
  .status-bar {
    border-top: 1px solid var(--border);
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 1000;
    background-color: var(--menu-bg);
    color: var(--text-secondary);
  }
</style>
