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
  aria-valuemin={0}
  aria-valuemax={100}
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
