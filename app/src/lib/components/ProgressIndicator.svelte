<script lang="ts">
  export let current: number = 0;
  export let total: number = 100;
  export let message: string = '';
  export let showPercentage: boolean = true;
  export let showCount: boolean = false;
  export let indeterminate: boolean = false;
  export let color: string = '#3b82f6';

  $: percentage = total > 0 ? Math.round((current / total) * 100) : 0;
  $: progressWidth = indeterminate ? 100 : percentage;
</script>

<div class="progress-indicator">
  <div class="progress-bar-container h-2 dark:bg-gray-700 rounded-full overflow-hidden">
    <div
      class="progress-bar h-full transition-all duration-300 ease-out"
      class:animate-pulse={indeterminate}
      style="width: {progressWidth}%; background-color: {color}"
    ></div>
  </div>

  <div class="progress-info flex justify-between items-center mt-1 text-xs dark:text-gray-400">
    {#if message}
      <span class="message truncate flex-1 mr-2">{message}</span>
    {:else}
      <span></span>
    {/if}

    <div class="stats flex gap-2">
      {#if showCount}
        <span>{current.toLocaleString()} / {total.toLocaleString()}</span>
      {/if}
      {#if showPercentage && !indeterminate}
        <span>{percentage}%</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .progress-bar-container {
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .progress-bar {
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  @keyframes shimmer {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }
</style>
