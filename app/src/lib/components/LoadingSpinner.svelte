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
