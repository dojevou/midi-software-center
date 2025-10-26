<script lang="ts">
  import { formatFileSize, formatBpm, formatDate } from '$lib/utils/formatters';
  import type { FileMetadata } from '$lib/api/types';

  export let file: FileMetadata;
</script>

<div class="space-y-3">
  <h3 class="font-semibold text-sm">Metadata</h3>

  <dl class="space-y-2 text-sm">
    <div class="flex justify-between">
      <dt class="text-slate-600">Filename:</dt>
      <dd class="font-medium truncate ml-2">{file.filename}</dd>
    </div>

    <div class="flex justify-between">
      <dt class="text-slate-600">Category:</dt>
      <dd class="font-medium">{file.category}</dd>
    </div>

    {#if file.subcategory}
      <div class="flex justify-between">
        <dt class="text-slate-600">Subcategory:</dt>
        <dd class="font-medium">{file.subcategory}</dd>
      </div>
    {/if}

    {#if file.bpm}
      <div class="flex justify-between">
        <dt class="text-slate-600">BPM:</dt>
        <dd class="font-medium">{formatBpm(file.bpm)}</dd>
      </div>
    {/if}

    {#if file.key}
      <div class="flex justify-between">
        <dt class="text-slate-600">Key:</dt>
        <dd class="font-medium">{file.key}</dd>
      </div>
    {/if}

    <div class="flex justify-between">
      <dt class="text-slate-600">Size:</dt>
      <dd class="font-medium">{formatFileSize(file.fileSize)}</dd>
    </div>

    <div class="flex justify-between">
      <dt class="text-slate-600">Added:</dt>
      <dd class="font-medium">{formatDate(file.createdAt)}</dd>
    </div>
  </dl>
</div>
