<script lang="ts">
  import { fade } from 'svelte/transition';
  import { loadingStore, loadingTasks } from '$lib/stores/loadingStore';
  import LoadingSpinner from './LoadingSpinner.svelte';
  import ProgressBar from './ProgressBar.svelte';

  export let showOverlay = true;
  export let minDisplayTime = 500; // Prevent flash of loading state

  let visible = false;
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;

  $: if ($loadingTasks.length > 0) {
    visible = true;
    if (hideTimeout) {
      clearTimeout(hideTimeout);
      hideTimeout = null;
    }
  } else if (visible) {
    hideTimeout = setTimeout(() => {
      visible = false;
    }, minDisplayTime);
  }

  function getElapsedTime(startTime: number): string {
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    if (elapsed < 60) return `${elapsed}s`;
    return `${Math.floor(elapsed / 60)}m ${elapsed % 60}s`;
  }
</script>

{#if visible && showOverlay}
  <div
    class="loading-overlay"
    transition:fade={{ duration: 150 }}
    role="status"
    aria-live="polite"
  >
    <div class="loading-content">
      <LoadingSpinner size="large" />

      <div class="tasks-list">
        {#each $loadingTasks as task (task.id)}
          <div class="task-item">
            <div class="task-header">
              <span class="task-label">{task.label}</span>
              <span class="task-time">{getElapsedTime(task.startTime)}</span>
            </div>
            {#if task.progress !== undefined}
              <ProgressBar
                progress={task.progress}
                size="small"
                showPercentage={false}
              />
            {/if}
            {#if task.cancelable}
              <button
                class="cancel-btn"
                on:click={() => loadingStore.cancel(task.id)}
              >
                Cancel
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .loading-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 99999;
  }

  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    padding: 32px;
    background: var(--bg-secondary);
    border-radius: var(--border-radius-lg);
    min-width: 300px;
    max-width: 400px;
  }

  .tasks-list {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .task-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .task-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .task-label {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .task-time {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .cancel-btn {
    align-self: flex-start;
    padding: 4px 8px;
    background: none;
    border: 1px solid var(--border);
    border-radius: var(--border-radius-sm);
    color: var(--text-secondary);
    font-size: 11px;
    cursor: pointer;
  }

  .cancel-btn:hover {
    background: var(--bg-hover);
  }
</style>
