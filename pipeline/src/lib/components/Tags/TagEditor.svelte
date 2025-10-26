<script lang="ts">
  import { onMount } from 'svelte';
  import { X, Plus } from 'lucide-svelte';
  import {
    getFileTags,
    addTagsToFile,
    removeTagFromFile,
    searchTags,
    getTagCategoryColor,
  } from '$lib/stores/tags';
  import type { Tag } from '$lib/types';

  export let fileId: number;
  export let onChange: (() => void) | undefined = undefined;

  let tags: Tag[] = [];
  let loading = false;
  let saving = false;
  let error: string | null = null;

  // New tag input
  let newTagInput = '';
  let showAutocomplete = false;
  let autocompleteResults: Tag[] = [];

  // Separate auto and manual tags
  $: autoTags = tags.filter((t) => t.category); // Tags with categories are auto-generated
  $: manualTags = tags.filter((t) => !t.category); // Tags without categories are user-added

  onMount(async () => {
    await loadTags();
  });

  async function loadTags() {
    loading = true;
    error = null;

    try {
      tags = await getFileTags(fileId);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load tags';
      console.error('Error loading tags:', err);
    } finally {
      loading = false;
    }
  }

  async function handleAddTag() {
    const tagName = newTagInput.trim();
    if (!tagName) return;

    // Check if tag already exists
    if (tags.some((t) => t.name.toLowerCase() === tagName.toLowerCase())) {
      alert('Tag already exists for this file');
      return;
    }

    saving = true;
    error = null;

    try {
      await addTagsToFile(fileId, [tagName]);
      await loadTags();
      newTagInput = '';
      showAutocomplete = false;

      if (onChange) {
        onChange();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to add tag';
      console.error('Error adding tag:', err);
    } finally {
      saving = false;
    }
  }

  async function handleRemoveTag(tag: Tag) {
    if (!confirm(`Remove tag "${tag.name}"?`)) {
      return;
    }

    saving = true;
    error = null;

    try {
      await removeTagFromFile(fileId, tag.id);
      await loadTags();

      if (onChange) {
        onChange();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to remove tag';
      console.error('Error removing tag:', err);
    } finally {
      saving = false;
    }
  }

  async function handleTagInput() {
    const query = newTagInput.trim();

    if (query.length < 2) {
      showAutocomplete = false;
      autocompleteResults = [];
      return;
    }

    showAutocomplete = true;
    autocompleteResults = await searchTags(query, 5);
  }

  function selectAutocomplete(tag: Tag) {
    newTagInput = tag.name;
    showAutocomplete = false;
    handleAddTag();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAddTag();
    } else if (e.key === 'Escape') {
      showAutocomplete = false;
    }
  }
</script>

<div class="tag-editor space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h3 class="font-semibold text-sm text-slate-900 dark:text-slate-100">Tags</h3>
    {#if tags.length > 0}
      <span class="text-xs text-slate-500 dark:text-slate-400">{tags.length} total</span>
    {/if}
  </div>

  <!-- Loading State -->
  {#if loading}
    <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
      <div class="spinner"></div>
      <span class="text-sm">Loading tags...</span>
    </div>
  {:else}
    <!-- Auto-Generated Tags -->
    {#if autoTags.length > 0}
      <div class="space-y-2">
        <h4 class="text-xs font-medium text-slate-600 dark:text-slate-400 flex items-center gap-1">
          <span>Auto-Generated</span>
          <span class="text-xs opacity-70">(from filename & content)</span>
        </h4>
        <div class="flex flex-wrap gap-2">
          {#each autoTags as tag (tag.id)}
            {@const colorClass = getTagCategoryColor(tag.category)}
            <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs {colorClass}">
              <span class="font-medium">{tag.category}:</span>
              <span>{tag.name}</span>
              {#if !saving}
                <button
                  on:click={() => handleRemoveTag(tag)}
                  class="hover:bg-black/10 dark:hover:bg-white/10 rounded-full p-0.5 transition-colors"
                  aria-label="Remove tag"
                  title="Remove tag"
                >
                  <X class="h-3 w-3" />
                </button>
              {/if}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Manual Tags -->
    {#if manualTags.length > 0}
      <div class="space-y-2">
        <h4 class="text-xs font-medium text-slate-600 dark:text-slate-400">Manual Tags</h4>
        <div class="flex flex-wrap gap-2">
          {#each manualTags as tag (tag.id)}
            <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs bg-slate-100 text-slate-800 dark:bg-slate-700 dark:text-slate-200">
              {tag.name}
              {#if !saving}
                <button
                  on:click={() => handleRemoveTag(tag)}
                  class="hover:bg-black/10 dark:hover:bg-white/10 rounded-full p-0.5 transition-colors"
                  aria-label="Remove tag"
                  title="Remove tag"
                >
                  <X class="h-3 w-3" />
                </button>
              {/if}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Empty State -->
    {#if tags.length === 0}
      <div class="text-center py-6 text-slate-500 dark:text-slate-400">
        <p class="text-sm">No tags yet</p>
        <p class="text-xs mt-1">Add tags below to organize this file</p>
      </div>
    {/if}
  {/if}

  <!-- Add Tag Input -->
  <div class="relative">
    <label for="new-tag-input" class="sr-only">Add new tag</label>
    <div class="flex gap-2">
      <input
        id="new-tag-input"
        type="text"
        bind:value={newTagInput}
        on:input={handleTagInput}
        on:keydown={handleKeydown}
        on:focus={() => {
          if (newTagInput.length >= 2) {
            showAutocomplete = true;
          }
        }}
        on:blur={() => {
          // Delay to allow clicking autocomplete items
          setTimeout(() => {
            showAutocomplete = false;
          }, 200);
        }}
        placeholder="Add tag..."
        class="input flex-1 text-sm"
        disabled={saving || loading}
      />
      <button
        on:click={handleAddTag}
        disabled={!newTagInput.trim() || saving || loading}
        class="btn-primary px-3 py-2 text-sm flex items-center gap-1"
        aria-label="Add tag"
      >
        <Plus class="h-4 w-4" />
        <span>Add</span>
      </button>
    </div>

    <!-- Autocomplete Dropdown -->
    {#if showAutocomplete && autocompleteResults.length > 0}
      <div class="absolute z-10 w-full mt-1 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg shadow-lg max-h-40 overflow-y-auto">
        {#each autocompleteResults as result}
          {@const colorClass = getTagCategoryColor(result.category)}
          <button
            on:click={() => selectAutocomplete(result)}
            class="w-full px-3 py-2 text-left text-sm hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors flex items-center justify-between"
          >
            <span class="inline-flex items-center gap-2">
              <span class="px-2 py-0.5 rounded-full text-xs {colorClass}">
                {result.category ? `${result.category}:` : ''}{result.name}
              </span>
            </span>
            <span class="text-xs text-slate-500 dark:text-slate-400">{result.usageCount} files</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Error Message -->
  {#if error}
    <div class="p-2 bg-red-50 dark:bg-red-900/20 rounded text-xs text-red-600 dark:text-red-400">
      {error}
    </div>
  {/if}

  <!-- Saving Indicator -->
  {#if saving}
    <div class="flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400">
      <div class="spinner small"></div>
      <span>Saving...</span>
    </div>
  {/if}
</div>

<style>
  .spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid rgba(0, 0, 0, 0.1);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .spinner.small {
    width: 0.75rem;
    height: 0.75rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
