<script lang="ts">
  import { errorStore } from '$lib/stores/errorStore';
  import { a11yStore } from '$lib/stores/a11yStore';
  import { onMount, onDestroy } from 'svelte';

  let dialogRef: HTMLDivElement;

  $: error = $errorStore.globalError;
  $: show = $errorStore.showErrorDialog && error;

  $: if (show && dialogRef) {
    a11yStore.trapFocus(dialogRef);
  }

  function close() {
    errorStore.closeDialog();
    a11yStore.releaseFocus();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      close();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
    if (show) {
      a11yStore.releaseFocus();
    }
  });
</script>

{#if show && error}
  <div
    class="error-dialog-overlay"
    on:click={close}
    on:keydown={handleKeyDown}
    role="presentation"
  >
    <div
      bind:this={dialogRef}
      class="error-dialog"
      class:critical={error.severity === 'critical'}
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="error-dialog-title"
      aria-describedby="error-dialog-desc"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <header>
        <span class="error-icon" aria-hidden="true">
          {error.severity === 'critical' ? '!!' : '!'}
        </span>
        <h2 id="error-dialog-title">
          {error.severity === 'critical' ? 'Critical Error' : 'Error'}
        </h2>
        <button class="close-btn" on:click={close} aria-label="Close dialog">
          x
        </button>
      </header>

      <div class="error-body">
        <p id="error-dialog-desc" class="error-message">{error.message}</p>

        {#if error.component}
          <p class="error-location">
            <span>Component:</span> {error.component}
          </p>
        {/if}

        {#if error.details || error.stack}
          <details class="error-details">
            <summary>Technical Details</summary>
            <pre>{error.details || error.stack}</pre>
          </details>
        {/if}
      </div>

      <footer>
        {#if error.retryable && error.action}
          <button class="primary-btn" on:click={() => errorStore.retry(error.id)}>
            {error.action.label}
          </button>
        {/if}
        <button class="secondary-btn" on:click={close}>
          {error.severity === 'critical' ? 'Acknowledge' : 'Dismiss'}
        </button>
        {#if error.severity === 'critical'}
          <button class="danger-btn" on:click={() => window.location.reload()}>
            Reload Application
          </button>
        {/if}
      </footer>
    </div>
  </div>
{/if}

<style>
  .error-dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100001;
  }

  .error-dialog {
    background: var(--bg-secondary);
    border-radius: var(--border-radius-lg);
    width: 500px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    border: 1px solid var(--error);
  }

  .error-dialog.critical {
    border-width: 2px;
    animation: shake 0.5s ease-in-out;
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    20%, 60% { transform: translateX(-5px); }
    40%, 80% { transform: translateX(5px); }
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .error-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--error);
    color: white;
    border-radius: 50%;
    font-weight: bold;
    font-size: 14px;
  }

  header h2 {
    flex: 1;
    margin: 0;
    font-size: var(--font-size-lg);
    color: var(--error);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .error-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
  }

  .error-message {
    margin: 0 0 16px 0;
    font-size: var(--font-size-base);
    color: var(--text-primary);
    line-height: 1.5;
  }

  .error-location {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin: 0 0 16px 0;
  }

  .error-location span {
    color: var(--text-muted);
  }

  .error-details summary {
    cursor: pointer;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .error-details pre {
    margin: 0;
    padding: 12px;
    background: var(--bg-tertiary);
    border-radius: var(--border-radius);
    font-size: 11px;
    overflow-x: auto;
    max-height: 200px;
    color: var(--text-secondary);
  }

  footer {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 20px;
    border-top: 1px solid var(--border);
  }

  footer button {
    padding: 8px 16px;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-size: var(--font-size-sm);
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

  .danger-btn {
    background: var(--error);
    color: white;
    border: none;
  }
</style>
