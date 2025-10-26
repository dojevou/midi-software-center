<script lang="ts">
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let message: string | undefined = undefined;

  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };
</script>

<div class="flex flex-col items-center justify-center gap-2">
  <div class="spinner {sizeClasses[size]}" role="status" aria-label="Loading">
    <span class="sr-only">Loading...</span>
  </div>
  {#if message}
    <p class="text-sm text-slate-600">{message}</p>
  {/if}
</div>
