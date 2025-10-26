<script lang="ts">
  import { onMount } from 'svelte';
  import { Grid, List } from 'lucide-svelte';
  import { filesStore } from '$lib/stores/files';
  import FileCard from './FileCard.svelte';
  import LoadingSpinner from '$lib/components/common/LoadingSpinner.svelte';
  import ErrorMessage from '$lib/components/common/ErrorMessage.svelte';
  import EmptyState from '$lib/components/common/EmptyState.svelte';
  import { Music } from 'lucide-svelte';

  onMount(() => {
    filesStore.loadFiles();
  });

  function handleFileClick(id: number) {
    filesStore.selectFile(id);
  }
</script>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center justify-between p-4 border-b">
    <h2 class="text-xl font-semibold">
      Files ({$filesStore.total})
    </h2>
    <div class="flex items-center gap-2">
      <button
        on:click={() => filesStore.setViewMode('grid')}
        class="btn-ghost p-2 {$filesStore.viewMode === 'grid' ? 'bg-slate-100' : ''}"
      >
        <Grid class="h-5 w-5" />
      </button>
      <button
        on:click={() => filesStore.setViewMode('list')}
        class="btn-ghost p-2 {$filesStore.viewMode === 'list' ? 'bg-slate-100' : ''}"
      >
        <List class="h-5 w-5" />
      </button>
    </div>
  </div>

  <!-- Content -->
  <div class="flex-1 overflow-auto p-4">
    {#if $filesStore.loading}
      <LoadingSpinner size="lg" message="Loading files..." />
    {:else if $filesStore.error}
      <ErrorMessage
        error={$filesStore.error}
        onRetry={() => filesStore.loadFiles()}
      />
    {:else if $filesStore.items.length === 0}
      <EmptyState
        icon={Music}
        title="No files found"
        description="Import your first MIDI file to get started"
      />
    {:else}
      {#if $filesStore.viewMode === 'grid'}
        <!-- Grid View -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each $filesStore.items as file}
            <FileCard
              {file}
              selected={file.id === $filesStore.selectedId}
              onClick={() => handleFileClick(file.id)}
            />
          {/each}
        </div>
      {:else}
        <!-- List View -->
        <div class="flex flex-col gap-2">
          {#each $filesStore.items as file}
            <div
              on:click={() => handleFileClick(file.id)}
              on:keydown={(e) => e.key === 'Enter' && handleFileClick(file.id)}
              role="button"
              tabindex="0"
              class="flex items-center gap-4 p-4 border rounded-lg cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors
                {file.id === $filesStore.selectedId ? 'border-blue-500 bg-blue-50 dark:bg-blue-950' : 'border-slate-200 dark:border-slate-700'}"
            >
              <div class="flex-1 min-w-0">
                <h3 class="font-medium truncate">{file.filename}</h3>
                <div class="flex gap-4 text-sm text-slate-600 dark:text-slate-400 mt-1">
                  <span>
                    {file.category === 'UNKNOWN' && file.parentFolder ? file.parentFolder : (file.category || 'Root')}
                  </span>
                  {#if file.bpm}
                    <span>{file.bpm} BPM</span>
                  {/if}
                  {#if file.key}
                    <span>{file.key}</span>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>
