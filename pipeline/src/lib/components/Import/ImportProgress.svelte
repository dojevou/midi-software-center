<!--
  ImportProgress.svelte - Display import progress
  Location: /pipeline/src/lib/components/Import/ImportProgress.svelte
  Archetype: CONTAINER (composition + layout, NO I/O)

  Container responsibilities:
  - UI composition (progress bar)
  - Display state from importStore
  - NO event listeners (Manager handles that)
-->

<script lang="ts">
  import { importStore } from '$lib/stores/import';

  export let onComplete: () => void;

  // Read state from store (reactive)
  $: progress = $importStore.progress;
  $: importing = $importStore.importing;

  // Calculate percentage
  $: percentage = progress
    ? (progress.current / progress.total) * 100
    : 0;

  // Watch for completion
  $: if (progress && progress.current === progress.total && !importing) {
    setTimeout(onComplete, 1000);
  }
</script>

{#if progress}
  <div class="card p-4">
    <div class="flex items-center justify-between mb-2">
      <span class="text-sm font-medium">
        Importing files... ({progress.current}/{progress.total})
      </span>
      <span class="text-sm text-slate-600">
        {Math.round(percentage)}%
      </span>
    </div>
    <div class="w-full bg-slate-200 rounded-full h-2">
      <div
        class="bg-primary-600 h-2 rounded-full transition-all"
        style="width: {percentage}%"
      />
    </div>
    <p class="mt-2 text-xs text-slate-600 truncate">{progress.fileName}</p>
  </div>
{/if}
