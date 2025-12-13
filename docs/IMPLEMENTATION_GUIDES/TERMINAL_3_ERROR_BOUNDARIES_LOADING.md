# TERMINAL 3: Error Boundaries & Loading States

## Owner: Claude Instance #3
## Components: UI Error Handling Layer, Skeleton Loaders, Async Indicators

---

## PART A: ERROR BOUNDARY SYSTEM

### A1. Create Error Store (`app/src/lib/stores/errorStore.ts`)

```typescript
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export type ErrorSeverity = 'info' | 'warning' | 'error' | 'critical';
export type ErrorCategory = 'network' | 'database' | 'file' | 'midi' | 'ui' | 'unknown';

export interface AppError {
  id: string;
  message: string;
  details?: string;
  severity: ErrorSeverity;
  category: ErrorCategory;
  timestamp: number;
  stack?: string;
  component?: string;
  action?: ErrorAction;
  dismissed: boolean;
  retryable: boolean;
}

export interface ErrorAction {
  label: string;
  handler: () => void | Promise<void>;
}

export interface ErrorState {
  errors: AppError[];
  globalError: AppError | null;
  showErrorDialog: boolean;
  errorCount: number;
}

// ============================================================================
// ERROR UTILITIES
// ============================================================================

export function createErrorId(): string {
  return `err-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

export function categorizeError(error: unknown): ErrorCategory {
  if (error instanceof Error) {
    const msg = error.message.toLowerCase();
    if (msg.includes('network') || msg.includes('fetch') || msg.includes('timeout')) {
      return 'network';
    }
    if (msg.includes('database') || msg.includes('sql') || msg.includes('query')) {
      return 'database';
    }
    if (msg.includes('file') || msg.includes('path') || msg.includes('permission')) {
      return 'file';
    }
    if (msg.includes('midi') || msg.includes('device') || msg.includes('port')) {
      return 'midi';
    }
  }
  return 'unknown';
}

export function formatErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: unknown }).message);
  }
  return 'An unexpected error occurred';
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

const MAX_ERRORS = 100;

const initialState: ErrorState = {
  errors: [],
  globalError: null,
  showErrorDialog: false,
  errorCount: 0,
};

function createErrorStore() {
  const { subscribe, set, update } = writable<ErrorState>(initialState);

  return {
    subscribe,

    // Add a new error
    addError(
      error: unknown,
      options: {
        severity?: ErrorSeverity;
        category?: ErrorCategory;
        component?: string;
        action?: ErrorAction;
        retryable?: boolean;
      } = {}
    ): string {
      const id = createErrorId();
      const appError: AppError = {
        id,
        message: formatErrorMessage(error),
        details: error instanceof Error ? error.stack : undefined,
        severity: options.severity || 'error',
        category: options.category || categorizeError(error),
        timestamp: Date.now(),
        stack: error instanceof Error ? error.stack : undefined,
        component: options.component,
        action: options.action,
        dismissed: false,
        retryable: options.retryable ?? false,
      };

      update(state => {
        const errors = [appError, ...state.errors].slice(0, MAX_ERRORS);
        const showDialog = appError.severity === 'critical' || appError.severity === 'error';
        return {
          ...state,
          errors,
          globalError: showDialog ? appError : state.globalError,
          showErrorDialog: showDialog || state.showErrorDialog,
          errorCount: state.errorCount + 1,
        };
      });

      // Log to backend
      this.logToBackend(appError);

      return id;
    },

    // Dismiss an error
    dismiss(errorId: string) {
      update(state => ({
        ...state,
        errors: state.errors.map(e =>
          e.id === errorId ? { ...e, dismissed: true } : e
        ),
        globalError: state.globalError?.id === errorId ? null : state.globalError,
        showErrorDialog: state.globalError?.id === errorId ? false : state.showErrorDialog,
      }));
    },

    // Dismiss all errors
    dismissAll() {
      update(state => ({
        ...state,
        errors: state.errors.map(e => ({ ...e, dismissed: true })),
        globalError: null,
        showErrorDialog: false,
      }));
    },

    // Clear all errors
    clearAll() {
      set(initialState);
    },

    // Close error dialog
    closeDialog() {
      update(state => ({
        ...state,
        showErrorDialog: false,
        globalError: null,
      }));
    },

    // Retry action for an error
    async retry(errorId: string) {
      const state = getState();
      const error = state.errors.find(e => e.id === errorId);
      if (error?.action) {
        try {
          await error.action.handler();
          this.dismiss(errorId);
        } catch (e) {
          // Error persists, update message if different
          const newMessage = formatErrorMessage(e);
          if (newMessage !== error.message) {
            update(s => ({
              ...s,
              errors: s.errors.map(err =>
                err.id === errorId ? { ...err, message: newMessage } : err
              ),
            }));
          }
        }
      }
    },

    // Log error to Rust backend
    async logToBackend(error: AppError) {
      try {
        await invoke('log_frontend_error', {
          error: {
            message: error.message,
            severity: error.severity,
            category: error.category,
            component: error.component,
            stack: error.stack,
            timestamp: error.timestamp,
          },
        });
      } catch (e) {
        console.error('Failed to log error to backend:', e);
      }
    },
  };

  function getState(): ErrorState {
    let state: ErrorState = initialState;
    subscribe(s => { state = s; })();
    return state;
  }
}

export const errorStore = createErrorStore();

// Derived stores
export const activeErrors = derived(errorStore, $errors =>
  $errors.errors.filter(e => !e.dismissed)
);

export const criticalErrors = derived(errorStore, $errors =>
  $errors.errors.filter(e => e.severity === 'critical' && !e.dismissed)
);

export const hasErrors = derived(errorStore, $errors =>
  $errors.errors.some(e => !e.dismissed)
);

// Global error handler
if (typeof window !== 'undefined') {
  window.addEventListener('error', (event) => {
    errorStore.addError(event.error || event.message, {
      severity: 'error',
      component: 'window',
    });
  });

  window.addEventListener('unhandledrejection', (event) => {
    errorStore.addError(event.reason, {
      severity: 'error',
      component: 'promise',
    });
  });
}
```

### A2. Create Error Boundary Component (`app/src/lib/components/ErrorBoundary.svelte`)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { errorStore } from '$lib/stores/errorStore';
  import type { ErrorSeverity } from '$lib/stores/errorStore';

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
      <div class="error-icon" aria-hidden="true">⚠️</div>
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
        <div class="error-icon-large" aria-hidden="true">💥</div>
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
    font-size: 24px;
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
    font-size: 64px;
    margin-bottom: 24px;
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
```

### A3. Create Error Toast Component (`app/src/lib/components/ErrorToast.svelte`)

```svelte
<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { errorStore, activeErrors } from '$lib/stores/errorStore';
  import type { AppError } from '$lib/stores/errorStore';

  export let maxVisible = 5;
  export let position: 'top-right' | 'bottom-right' | 'top-left' | 'bottom-left' = 'bottom-right';

  $: visibleErrors = $activeErrors.slice(0, maxVisible);

  function getSeverityIcon(severity: string): string {
    switch (severity) {
      case 'info': return 'ℹ️';
      case 'warning': return '⚠️';
      case 'error': return '❌';
      case 'critical': return '🚨';
      default: return '❌';
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
      <div class="toast-icon" aria-hidden="true">
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
          ×
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
    font-size: 18px;
    flex-shrink: 0;
  }

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
```

### A4. Create Error Dialog Component (`app/src/lib/components/ErrorDialog.svelte`)

```svelte
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
    >
      <header>
        <span class="error-icon" aria-hidden="true">
          {error.severity === 'critical' ? '🚨' : '❌'}
        </span>
        <h2 id="error-dialog-title">
          {error.severity === 'critical' ? 'Critical Error' : 'Error'}
        </h2>
        <button class="close-btn" on:click={close} aria-label="Close dialog">
          ×
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
    font-size: 24px;
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
```

---

## PART B: LOADING STATES & SKELETON LOADERS

### B1. Create Loading Store (`app/src/lib/stores/loadingStore.ts`)

```typescript
import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export interface LoadingTask {
  id: string;
  label: string;
  progress?: number; // 0-100, undefined for indeterminate
  startTime: number;
  cancelable: boolean;
  cancel?: () => void;
}

export interface LoadingState {
  tasks: Map<string, LoadingTask>;
  globalLoading: boolean;
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

function createLoadingStore() {
  const { subscribe, set, update } = writable<LoadingState>({
    tasks: new Map(),
    globalLoading: false,
  });

  return {
    subscribe,

    // Start a loading task
    start(
      id: string,
      label: string,
      options: { cancelable?: boolean; cancel?: () => void } = {}
    ) {
      update(state => {
        const tasks = new Map(state.tasks);
        tasks.set(id, {
          id,
          label,
          progress: undefined,
          startTime: Date.now(),
          cancelable: options.cancelable ?? false,
          cancel: options.cancel,
        });
        return { ...state, tasks, globalLoading: true };
      });

      return {
        setProgress: (progress: number) => this.setProgress(id, progress),
        setLabel: (label: string) => this.setLabel(id, label),
        finish: () => this.finish(id),
      };
    },

    // Update progress for a task
    setProgress(id: string, progress: number) {
      update(state => {
        const tasks = new Map(state.tasks);
        const task = tasks.get(id);
        if (task) {
          tasks.set(id, { ...task, progress: Math.min(100, Math.max(0, progress)) });
        }
        return { ...state, tasks };
      });
    },

    // Update label for a task
    setLabel(id: string, label: string) {
      update(state => {
        const tasks = new Map(state.tasks);
        const task = tasks.get(id);
        if (task) {
          tasks.set(id, { ...task, label });
        }
        return { ...state, tasks };
      });
    },

    // Finish a loading task
    finish(id: string) {
      update(state => {
        const tasks = new Map(state.tasks);
        tasks.delete(id);
        return {
          ...state,
          tasks,
          globalLoading: tasks.size > 0,
        };
      });
    },

    // Cancel a task
    cancel(id: string) {
      const state = getState();
      const task = state.tasks.get(id);
      if (task?.cancelable && task.cancel) {
        task.cancel();
      }
      this.finish(id);
    },

    // Set global loading state
    setGlobalLoading(loading: boolean) {
      update(state => ({ ...state, globalLoading: loading }));
    },

    // Helper: wrap async function with loading state
    async withLoading<T>(
      id: string,
      label: string,
      fn: () => Promise<T>,
      options?: { cancelable?: boolean }
    ): Promise<T> {
      const controller = new AbortController();
      const task = this.start(id, label, {
        ...options,
        cancel: () => controller.abort(),
      });

      try {
        const result = await fn();
        return result;
      } finally {
        task.finish();
      }
    },
  };

  function getState(): LoadingState {
    let state: LoadingState = { tasks: new Map(), globalLoading: false };
    subscribe(s => { state = s; })();
    return state;
  }
}

export const loadingStore = createLoadingStore();

// Derived stores
export const isLoading = derived(loadingStore, $loading => $loading.globalLoading);
export const loadingTasks = derived(loadingStore, $loading => Array.from($loading.tasks.values()));
export const hasLoadingTasks = derived(loadingStore, $loading => $loading.tasks.size > 0);
```

### B2. Create Skeleton Component (`app/src/lib/components/Skeleton.svelte`)

```svelte
<script lang="ts">
  export let variant: 'text' | 'rectangular' | 'circular' | 'card' = 'text';
  export let width: string | undefined = undefined;
  export let height: string | undefined = undefined;
  export let lines = 1;
  export let animate = true;

  const defaultSizes = {
    text: { width: '100%', height: '1em' },
    rectangular: { width: '100%', height: '100px' },
    circular: { width: '40px', height: '40px' },
    card: { width: '100%', height: '200px' },
  };
</script>

{#if variant === 'text' && lines > 1}
  <div class="skeleton-lines">
    {#each Array(lines) as _, i}
      <div
        class="skeleton text"
        class:animate
        style="width: {i === lines - 1 ? '75%' : (width || defaultSizes.text.width)}; height: {height || defaultSizes.text.height}"
        role="presentation"
        aria-hidden="true"
      />
    {/each}
  </div>
{:else}
  <div
    class="skeleton {variant}"
    class:animate
    style="width: {width || defaultSizes[variant].width}; height: {height || defaultSizes[variant].height}"
    role="presentation"
    aria-hidden="true"
  />
{/if}

<style>
  .skeleton {
    background: var(--bg-tertiary);
    border-radius: var(--border-radius-sm);
    position: relative;
    overflow: hidden;
  }

  .skeleton.circular {
    border-radius: 50%;
  }

  .skeleton.card {
    border-radius: var(--border-radius);
  }

  .skeleton.animate::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(255, 255, 255, 0.08),
      transparent
    );
    animation: shimmer 1.5s infinite;
  }

  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }

  .skeleton-lines {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Respect reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .skeleton.animate::after {
      animation: none;
      background: rgba(255, 255, 255, 0.05);
    }
  }
</style>
```

### B3. Create Loading Spinner Component (`app/src/lib/components/LoadingSpinner.svelte`)

```svelte
<script lang="ts">
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let color: string | undefined = undefined;
  export let label = 'Loading...';
  export let showLabel = false;

  const sizes = {
    small: '16px',
    medium: '32px',
    large: '48px',
  };
</script>

<div
  class="spinner-container"
  role="status"
  aria-live="polite"
>
  <div
    class="spinner"
    style="--spinner-size: {sizes[size]}; --spinner-color: {color || 'var(--accent)'}"
  />
  {#if showLabel}
    <span class="spinner-label">{label}</span>
  {:else}
    <span class="sr-only">{label}</span>
  {/if}
</div>

<style>
  .spinner-container {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .spinner {
    width: var(--spinner-size);
    height: var(--spinner-size);
    border: 3px solid var(--bg-tertiary);
    border-top-color: var(--spinner-color);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .spinner-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* Respect reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation: none;
      border-style: dashed;
    }
  }
</style>
```

### B4. Create Progress Bar Component (`app/src/lib/components/ProgressBar.svelte`)

```svelte
<script lang="ts">
  export let progress: number | undefined = undefined; // 0-100, undefined for indeterminate
  export let label = '';
  export let showPercentage = true;
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let color: string | undefined = undefined;

  $: isIndeterminate = progress === undefined;
  $: clampedProgress = progress !== undefined ? Math.min(100, Math.max(0, progress)) : 0;

  const heights = {
    small: '4px',
    medium: '8px',
    large: '12px',
  };
</script>

<div
  class="progress-container"
  role="progressbar"
  aria-valuenow={isIndeterminate ? undefined : clampedProgress}
  aria-valuemin="0"
  aria-valuemax="100"
  aria-label={label || 'Loading progress'}
>
  {#if label || showPercentage}
    <div class="progress-header">
      {#if label}
        <span class="progress-label">{label}</span>
      {/if}
      {#if showPercentage && !isIndeterminate}
        <span class="progress-percentage">{Math.round(clampedProgress)}%</span>
      {/if}
    </div>
  {/if}

  <div
    class="progress-track"
    style="--progress-height: {heights[size]}"
  >
    <div
      class="progress-fill"
      class:indeterminate={isIndeterminate}
      style="--progress-width: {clampedProgress}%; --progress-color: {color || 'var(--accent)'}"
    />
  </div>
</div>

<style>
  .progress-container {
    width: 100%;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .progress-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .progress-percentage {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .progress-track {
    width: 100%;
    height: var(--progress-height);
    background: var(--bg-tertiary);
    border-radius: calc(var(--progress-height) / 2);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    width: var(--progress-width);
    background: var(--progress-color);
    border-radius: inherit;
    transition: width 0.2s ease;
  }

  .progress-fill.indeterminate {
    width: 30%;
    animation: indeterminate 1.5s infinite ease-in-out;
  }

  @keyframes indeterminate {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(400%); }
  }

  /* Respect reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .progress-fill {
      transition: none;
    }

    .progress-fill.indeterminate {
      animation: none;
      width: 100%;
      opacity: 0.5;
    }
  }
</style>
```

### B5. Create Global Loading Overlay (`app/src/lib/components/LoadingOverlay.svelte`)

```svelte
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
```

---

## BACKEND: Rust Error Logging Command

### Add to `daw/src-tauri/src/commands/system.rs`

```rust
use serde::Deserialize;
use tracing::{error, warn, info};

#[derive(Debug, Deserialize)]
pub struct FrontendError {
    pub message: String,
    pub severity: String,
    pub category: String,
    pub component: Option<String>,
    pub stack: Option<String>,
    pub timestamp: i64,
}

#[tauri::command]
pub async fn log_frontend_error(error: FrontendError) -> Result<(), String> {
    let component = error.component.as_deref().unwrap_or("unknown");

    match error.severity.as_str() {
        "critical" | "error" => {
            error!(
                category = %error.category,
                component = %component,
                timestamp = %error.timestamp,
                stack = ?error.stack,
                "Frontend error: {}",
                error.message
            );
        }
        "warning" => {
            warn!(
                category = %error.category,
                component = %component,
                "Frontend warning: {}",
                error.message
            );
        }
        _ => {
            info!(
                category = %error.category,
                component = %component,
                "Frontend info: {}",
                error.message
            );
        }
    }

    Ok(())
}
```

---

## TESTING CHECKLIST

### Error Boundaries
- [ ] Inline error boundary shows error message
- [ ] Fullscreen error boundary shows with retry option
- [ ] Error toasts appear and can be dismissed
- [ ] Error dialog shows for critical errors
- [ ] Retry action works correctly
- [ ] Errors are logged to backend
- [ ] Global error handler catches unhandled errors
- [ ] Multiple errors queue correctly

### Loading States
- [ ] Loading spinner animates
- [ ] Progress bar shows determinate progress
- [ ] Progress bar shows indeterminate state
- [ ] Skeleton loaders animate
- [ ] Loading overlay shows for async tasks
- [ ] Cancel button works for cancelable tasks
- [ ] Loading state clears after task completes
- [ ] Reduced motion is respected

---

## FILES TO CREATE/MODIFY

| File | Action |
|------|--------|
| `app/src/lib/stores/errorStore.ts` | CREATE |
| `app/src/lib/stores/loadingStore.ts` | CREATE |
| `app/src/lib/components/ErrorBoundary.svelte` | CREATE |
| `app/src/lib/components/ErrorToast.svelte` | CREATE |
| `app/src/lib/components/ErrorDialog.svelte` | CREATE |
| `app/src/lib/components/Skeleton.svelte` | CREATE |
| `app/src/lib/components/LoadingSpinner.svelte` | CREATE |
| `app/src/lib/components/ProgressBar.svelte` | CREATE |
| `app/src/lib/components/LoadingOverlay.svelte` | CREATE |
| `daw/src-tauri/src/commands/system.rs` | MODIFY - Add log_frontend_error |
| `app/src/App.svelte` | MODIFY - Add ErrorToast and LoadingOverlay |
