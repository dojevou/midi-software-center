<script lang="ts">
  import { Music } from 'lucide-svelte';
  import { formatFileSize, formatBpm } from '$lib/utils/formatters';
  import type { FileMetadata } from '$lib/api/types';

  export let file: FileMetadata;
  export let selected = false;
  export let onClick: () => void;
</script>

<button
  on:click={onClick}
  class="card p-4 text-left transition-all hover:shadow-md cursor-pointer
         {selected ? 'ring-2 ring-primary-500 bg-primary-50' : ''}"
>
  <div class="flex items-start gap-3">
    <div class="p-2 rounded-lg bg-slate-100 flex-shrink-0">
      <Music class="h-6 w-6 text-slate-600" />
    </div>
    <div class="flex-1 min-w-0">
      <h3 class="font-medium text-sm truncate">{file.filename}</h3>
      <div class="flex items-center gap-2 mt-1 text-xs text-slate-600">
        <span class="px-2 py-0.5 rounded bg-slate-100">
          {file.category === 'UNKNOWN' && file.parentFolder ? file.parentFolder : file.category}
        </span>
        {#if file.bpm}
          <span>{formatBpm(file.bpm)}</span>
        {/if}
        {#if file.key}
          <span>{file.key}</span>
        {/if}
      </div>
      <div class="mt-2 text-xs text-slate-500">
        {formatFileSize(file.fileSize)}
      </div>
    </div>
  </div>
</button>
