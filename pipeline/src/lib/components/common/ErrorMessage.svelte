<script lang="ts">
  import { AlertCircle } from 'lucide-svelte';

  export let error: string;
  export let onRetry: (() => void) | undefined = undefined;
</script>

<div class="rounded-lg border border-red-200 bg-red-50 p-4">
  <div class="flex items-start gap-3">
    <AlertCircle class="h-5 w-5 text-red-600 flex-shrink-0 mt-0.5" />
    <div class="flex-1">
      <h3 class="text-sm font-medium text-red-800">Error</h3>
      <p class="mt-1 text-sm text-red-700">{error}</p>
      {#if onRetry}
        <button
          on:click={onRetry}
          class="mt-3 text-sm font-medium text-red-600 hover:text-red-700"
        >
          Try again
        </button>
      {/if}
    </div>
  </div>
</div>
