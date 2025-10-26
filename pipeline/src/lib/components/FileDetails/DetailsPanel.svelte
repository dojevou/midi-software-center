<script lang="ts">
  import { X } from 'lucide-svelte';
  import { selectedFile } from '$lib/stores/files';
  import { uiStore } from '$lib/stores/ui';
  import MetadataView from './MetadataView.svelte';
  import TagEditor from './TagEditor.svelte';
</script>

{#if $uiStore.detailsPanelOpen && $selectedFile}
  <div class="w-96 border-l bg-white flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b">
      <h2 class="text-lg font-semibold">File Details</h2>
      <button
        on:click={() => uiStore.toggleDetailsPanel()}
        class="btn-ghost p-1"
      >
        <X class="h-5 w-5" />
      </button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto p-4 space-y-6">
      <MetadataView file={$selectedFile} />
      <TagEditor file={$selectedFile} />
    </div>
  </div>
{/if}
