<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' = 'secondary';
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let disabled = false;
  export let loading = false;
  export let ariaLabel: string | undefined = undefined;
  export let ariaDescribedby: string | undefined = undefined;
  export let ariaExpanded: boolean | undefined = undefined;
  export let ariaHaspopup: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog' | undefined = undefined;
  export let ariaPressed: boolean | undefined = undefined;
  export let type: 'button' | 'submit' | 'reset' = 'button';

  const dispatch = createEventDispatcher<{ click: MouseEvent }>();

  function handleClick(event: MouseEvent) {
    if (!disabled && !loading) {
      dispatch('click', event);
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      const mouseEvent = new MouseEvent('click', { bubbles: true });
      handleClick(mouseEvent);
    }
  }
</script>

<button
  {type}
  class="accessible-button {variant} {size}"
  class:loading
  {disabled}
  aria-label={ariaLabel}
  aria-describedby={ariaDescribedby}
  aria-expanded={ariaExpanded}
  aria-haspopup={ariaHaspopup}
  aria-pressed={ariaPressed}
  aria-busy={loading}
  aria-disabled={disabled}
  on:click={handleClick}
  on:keydown={handleKeyDown}
>
  {#if loading}
    <span class="spinner" aria-hidden="true"></span>
    <span class="sr-only">Loading...</span>
  {/if}
  <span class="content" class:hidden={loading}>
    <slot />
  </span>
</button>

<style>
  .accessible-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: none;
    border-radius: var(--border-radius);
    font-family: inherit;
    font-weight: 500;
    cursor: pointer;
    transition: background-color var(--transition-duration) ease,
                transform var(--transition-duration) ease,
                box-shadow var(--transition-duration) ease;
    position: relative;
  }

  .accessible-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .accessible-button:active:not(:disabled) {
    transform: scale(0.98);
  }

  .accessible-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Sizes */
  .small {
    padding: 4px 8px;
    font-size: var(--font-size-sm);
  }

  .medium {
    padding: 8px 16px;
    font-size: var(--font-size-base);
  }

  .large {
    padding: 12px 24px;
    font-size: var(--font-size-lg);
  }

  /* Variants */
  .primary {
    background: var(--accent);
    color: white;
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--border-focus);
  }

  .ghost {
    background: transparent;
    color: var(--text-secondary);
  }

  .ghost:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .danger {
    background: var(--error);
    color: white;
  }

  .danger:hover:not(:disabled) {
    background: #c82333;
  }

  /* Loading state */
  .loading {
    pointer-events: none;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid transparent;
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .content.hidden {
    visibility: hidden;
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
</style>
