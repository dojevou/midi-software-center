<script lang="ts">
  import type { ComponentType } from 'svelte';

  export let icon: ComponentType;
  export let title: string;
  export let description: string;
  export let actionLabel: string | undefined = undefined;
  export let onAction: (() => void) | undefined = undefined;
</script>

<div class="flex flex-col items-center justify-center py-12 text-center">
  <div class="rounded-full bg-slate-100 p-3 mb-4">
    <svelte:component this={icon} class="h-8 w-8 text-slate-400" />
  </div>
  <h3 class="text-lg font-semibold text-slate-900 mb-2">{title}</h3>
  <p class="text-sm text-slate-600 mb-6 max-w-sm">{description}</p>
  {#if actionLabel && onAction}
    <button on:click={onAction} class="btn-primary px-4 py-2">
      {actionLabel}
    </button>
  {/if}
</div>
