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
    {#each Array(lines) as _, i (i)}
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
