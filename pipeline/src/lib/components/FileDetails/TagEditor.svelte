<!--
  TagEditor.svelte - Edit file tags
  Location: /pipeline/src/lib/components/FileDetails/TagEditor.svelte
  Archetype: CONTAINER (composition + layout, NO I/O)

  Container responsibilities:
  - UI composition (tag display + input)
  - User interaction handling
  - Delegates I/O to filesStore (Manager)
-->

<script lang="ts">
  import { X } from 'lucide-svelte';
  import { filesStore } from '$lib/stores/files';
  import { uiStore } from '$lib/stores/ui';
  import type { FileMetadata } from '$lib/api/types';

  export let file: FileMetadata;

  let tags = file.userTags || [];
  let newTag = '';

  // Read state from store (reactive)
  $: saving = $filesStore.updatingTags;

  async function addTag() {
    if (!newTag.trim()) return;
    tags = [...tags, newTag.trim()];
    newTag = '';
    await saveTags();
  }

  async function removeTag(index: number) {
    tags = tags.filter((_, i) => i !== index);
    await saveTags();
  }

  async function saveTags() {
    // Delegate to Manager store (handles I/O)
    const success = await filesStore.updateTags(file.id, tags);

    if (success) {
      uiStore.addNotification({
        type: 'success',
        message: 'Tags updated',
        duration: 2000,
      });
    } else if ($filesStore.error) {
      uiStore.addNotification({
        type: 'error',
        message: $filesStore.error,
        duration: 5000,
      });
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      addTag();
    }
  }
</script>

<div class="space-y-3">
  <h3 class="font-semibold text-sm">Tags</h3>

  <!-- Existing tags -->
  <div class="flex flex-wrap gap-2">
    {#each tags as tag, i}
      <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-primary-100 text-primary-700 text-xs">
        {tag}
        <button
          on:click={() => removeTag(i)}
          class="hover:text-primary-900"
          disabled={saving}
          aria-label="Remove tag"
        >
          <X class="h-3 w-3" />
        </button>
      </span>
    {/each}
  </div>

  <!-- Add tag input -->
  <div class="flex gap-2">
    <input
      type="text"
      bind:value={newTag}
      on:keydown={handleKeydown}
      placeholder="Add tag..."
      class="input flex-1 text-sm"
      disabled={saving}
    />
    <button
      on:click={addTag}
      disabled={!newTag.trim() || saving}
      class="btn-primary px-3 py-1 text-sm"
    >
      Add
    </button>
  </div>
</div>
