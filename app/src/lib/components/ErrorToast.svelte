<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { errorStore, activeErrors } from '$lib/stores/errorStore';
  import type { AppError } from '$lib/stores/errorStore';

  export let maxVisible = 5;
  export let position: 'top-right' | 'bottom-right' | 'top-left' | 'bottom-left' = 'bottom-right';

  $: visibleErrors = $activeErrors.slice(0, maxVisible);

  function getSeverityIcon(severity: string): string {
    switch (severity) {
      case 'info': return 'i';
      case 'warning': return '!';
      case 'error': return 'x';
      case 'critical': return '!!';
      default: return 'x';
    }
  }

  function formatTime(timestamp: number): string {
    return new Date(timestamp).toLocaleTimeString();
  }
</script>

<div class="toast-container {position}" role="log" aria-live="polite">
  {#each visibleErrors as error (error.id)}
    <div
      class="toast {error.severity}"
      role="alert"
      in:fly={{ x: position.includes('right') ? 100 : -100, duration: 200 }}
      out:fade={{ duration: 150 }}
    >
      <div class="toast-icon {error.severity}" aria-hidden="true">
        {getSeverityIcon(error.severity)}
      </div>
      <div class="toast-content">
        <div class="toast-header">
          <span class="toast-title">{error.category}</span>
          <span class="toast-time">{formatTime(error.timestamp)}</span>
        </div>
        <p class="toast-message">{error.message}</p>
        {#if error.component}
          <span class="toast-component">in {error.component}</span>
        {/if}
      </div>
      <div class="toast-actions">
        {#if error.retryable && error.action}
          <button
            class="toast-action retry"
            on:click={() => errorStore.retry(error.id)}
          >
            {error.action.label}
          </button>
        {/if}
        <button
          class="toast-action dismiss"
          on:click={() => errorStore.dismiss(error.id)}
          aria-label="Dismiss"
        >
          x
        </button>
      </div>
    </div>
  {/each}

  {#if $activeErrors.length > maxVisible}
    <div class="toast-overflow">
      +{$activeErrors.length - maxVisible} more errors
      <button on:click={() => errorStore.dismissAll()}>Dismiss all</button>
    </div>
  {/if}
</div>

<style>
  .toast-container {
    position: fixed;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 400px;
    pointer-events: none;
  }

  .toast-container > * {
    pointer-events: auto;
  }

  .top-right { top: 60px; right: 16px; }
  .bottom-right { bottom: 60px; right: 16px; }
  .top-left { top: 60px; left: 16px; }
  .bottom-left { bottom: 60px; left: 16px; }

  .toast {
    display: flex;
    gap: 12px;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-radius: var(--border-radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    border-left: 4px solid;
  }

  .toast.info { border-left-color: var(--info); }
  .toast.warning { border-left-color: var(--warning); }
  .toast.error { border-left-color: var(--error); }
  .toast.critical {
    border-left-color: var(--error);
    background: rgba(220, 53, 69, 0.15);
  }

  .toast-icon {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 11px;
    font-weight: bold;
    flex-shrink: 0;
    color: white;
  }

  .toast-icon.info { background: var(--info); }
  .toast-icon.warning { background: var(--warning); }
  .toast-icon.error { background: var(--error); }
  .toast-icon.critical { background: var(--error); }

  .toast-content {
    flex: 1;
    min-width: 0;
  }

  .toast-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .toast-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: capitalize;
  }

  .toast-time {
    font-size: 10px;
    color: var(--text-muted);
  }

  .toast-message {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    word-break: break-word;
  }

  .toast-component {
    font-size: 10px;
    color: var(--text-muted);
    margin-top: 4px;
    display: block;
  }

  .toast-actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  .toast-action {
    padding: 4px 8px;
    border: none;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    font-size: 12px;
  }

  .toast-action.retry {
    background: var(--accent);
    color: white;
  }

  .toast-action.dismiss {
    background: transparent;
    color: var(--text-muted);
    font-size: 18px;
    line-height: 1;
    padding: 0 4px;
  }

  .toast-action.dismiss:hover {
    color: var(--text-primary);
  }

  .toast-overflow {
    padding: 8px 16px;
    background: var(--bg-tertiary);
    border-radius: var(--border-radius);
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .toast-overflow button {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }
</style>
