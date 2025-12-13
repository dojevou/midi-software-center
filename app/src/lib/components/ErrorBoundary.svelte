<script lang="ts">
  import { onMount } from 'svelte';
  import { errorStore } from '$lib/stores/errorStore';

  export let fallback: 'inline' | 'fullscreen' | 'toast' = 'inline';
  export let componentName = 'Unknown';
  export let onError: ((error: Error) => void) | undefined = undefined;
  export let resetKey: unknown = undefined; // Change this to reset the boundary

  let hasError = false;
  let error: Error | null = null;
  let errorInfo: string | null = null;

  // Reset when resetKey changes
  $: if (resetKey !== undefined) {
    reset();
  }

  function handleError(e: Error, info?: string) {
    hasError = true;
    error = e;
    errorInfo = info || null;

    errorStore.addError(e, {
      component: componentName,
      severity: 'error',
      retryable: true,
      action: {
        label: 'Retry',
        handler: reset,
      },
    });

    onError?.(e);
  }

  function reset() {
    hasError = false;
    error = null;
    errorInfo = null;
  }

  // Expose to parent
  export function captureError(e: Error, info?: string) {
    handleError(e, info);
  }

  onMount(() => {
    const originalOnError = window.onerror;
    window.onerror = (msg, url, line, col, err) => {
      if (err) {
        handleError(err, `${url}:${line}:${col}`);
      }
      return originalOnError?.call(window, msg, url, line, col, err) ?? false;
    };

    return () => {
      window.onerror = originalOnError;
    };
  });
</script>

{#if hasError}
  {#if fallback === 'inline'}
    <div class="error-boundary-inline" role="alert">
      <div class="error-icon" aria-hidden="true">!</div>
      <div class="error-content">
        <h4>Something went wrong</h4>
        <p class="error-message">{error?.message || 'An unexpected error occurred'}</p>
        {#if errorInfo}
          <details>
            <summary>Technical details</summary>
            <pre>{errorInfo}</pre>
          </details>
        {/if}
      </div>
      <button class="retry-btn" on:click={reset}>
        Try Again
      </button>
    </div>
  {:else if fallback === 'fullscreen'}
    <div class="error-boundary-fullscreen" role="alert">
      <div class="error-container">
        <div class="error-icon-large" aria-hidden="true">!</div>
        <h2>Oops! Something went wrong</h2>
        <p class="error-message">{error?.message || 'An unexpected error occurred'}</p>
        <div class="error-actions">
          <button class="primary-btn" on:click={reset}>
            Try Again
          </button>
          <button class="secondary-btn" on:click={() => window.location.reload()}>
            Reload App
          </button>
        </div>
        {#if error?.stack}
          <details class="stack-trace">
            <summary>Stack trace</summary>
            <pre>{error.stack}</pre>
          </details>
        {/if}
      </div>
    </div>
  {:else}
    <!-- Toast fallback handled by ErrorToast component -->
    <slot />
  {/if}
{:else}
  <slot />
{/if}

<style>
  .error-boundary-inline {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 16px;
    background: rgba(220, 53, 69, 0.1);
    border: 1px solid var(--error);
    border-radius: var(--border-radius);
    color: var(--text-primary);
  }

  .error-icon {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--error);
    color: white;
    border-radius: 50%;
    font-weight: bold;
    font-size: 14px;
    flex-shrink: 0;
  }

  .error-content {
    flex: 1;
  }

  .error-content h4 {
    margin: 0 0 4px 0;
    font-size: var(--font-size-base);
    color: var(--error);
  }

  .error-message {
    margin: 0;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  details {
    margin-top: 8px;
  }

  details summary {
    cursor: pointer;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  details pre {
    margin: 8px 0 0 0;
    padding: 8px;
    background: var(--bg-tertiary);
    border-radius: var(--border-radius-sm);
    font-size: 11px;
    overflow-x: auto;
    max-height: 150px;
    overflow-y: auto;
  }

  .retry-btn {
    padding: 6px 12px;
    background: var(--error);
    color: white;
    border: none;
    border-radius: var(--border-radius);
    cursor: pointer;
    flex-shrink: 0;
  }

  .retry-btn:hover {
    background: #c82333;
  }

  /* Fullscreen error */
  .error-boundary-fullscreen {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-primary);
    z-index: 100000;
  }

  .error-container {
    text-align: center;
    max-width: 500px;
    padding: 40px;
  }

  .error-icon-large {
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--error);
    color: white;
    border-radius: 50%;
    font-weight: bold;
    font-size: 32px;
    margin: 0 auto 24px;
  }

  .error-container h2 {
    margin: 0 0 12px 0;
    color: var(--text-primary);
  }

  .error-container .error-message {
    margin-bottom: 24px;
    font-size: var(--font-size-base);
  }

  .error-actions {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  .primary-btn, .secondary-btn {
    padding: 10px 20px;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-size: var(--font-size-base);
  }

  .primary-btn {
    background: var(--accent);
    color: white;
    border: none;
  }

  .secondary-btn {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border);
  }

  .stack-trace {
    margin-top: 32px;
    text-align: left;
  }

  .stack-trace pre {
    max-height: 200px;
    overflow: auto;
  }
</style>
