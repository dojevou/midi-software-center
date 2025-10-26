<script lang="ts">
  import { onMount } from 'svelte';
  import {
    popularTags,
    loading,
    error,
    fetchPopularTags,
    selectTag,
    deselectTag,
    selectedTags,
    getTagCategoryColor,
    getTagFontSize,
  } from '$lib/stores/tags';
  import type { Tag } from '$lib/types';

  export let maxTags: number = 50;
  export let interactive: boolean = true; // Allow clicking to select tags

  let maxCount = 0;

  // Calculate max count for font sizing
  $: {
    if ($popularTags.length > 0) {
      maxCount = Math.max(...$popularTags.map((t) => t.usageCount));
    }
  }

  onMount(() => {
    fetchPopularTags(maxTags);
  });

  function handleTagClick(tag: Tag) {
    if (!interactive) return;

    const tagName = tag.category ? `${tag.category}:${tag.name}` : tag.name;

    if ($selectedTags.includes(tagName)) {
      deselectTag(tagName);
    } else {
      selectTag(tagName);
    }
  }

  function isTagSelected(tag: Tag): boolean {
    const tagName = tag.category ? `${tag.category}:${tag.name}` : tag.name;
    return $selectedTags.includes(tagName);
  }
</script>

<div class="tag-cloud">
  {#if $loading}
    <div class="flex items-center justify-center py-8">
      <div class="spinner"></div>
      <span class="ml-2 text-slate-600 dark:text-slate-400">Loading tags...</span>
    </div>
  {:else if $error}
    <div class="p-4 bg-red-50 dark:bg-red-900/20 rounded-lg">
      <p class="text-sm text-red-600 dark:text-red-400">{$error}</p>
      <button
        on:click={() => fetchPopularTags(maxTags)}
        class="mt-2 text-xs text-red-700 dark:text-red-300 hover:underline"
      >
        Retry
      </button>
    </div>
  {:else if $popularTags.length === 0}
    <div class="text-center py-8 text-slate-500 dark:text-slate-400">
      <p class="text-sm">No tags found</p>
      <p class="text-xs mt-1">Import some files to see tags</p>
    </div>
  {:else}
    <div class="flex flex-wrap gap-2 items-center">
      {#each $popularTags as tag (tag.id)}
        {@const fontSize = getTagFontSize(tag.usageCount, maxCount, 12, 24)}
        {@const colorClass = getTagCategoryColor(tag.category)}
        {@const isSelected = isTagSelected(tag)}
        {@const tagDisplay = tag.category ? `${tag.category}:${tag.name}` : tag.name}

        <button
          class="tag-button {colorClass} {isSelected ? 'ring-2 ring-primary-500' : ''}"
          style="font-size: {fontSize}px;"
          on:click={() => handleTagClick(tag)}
          disabled={!interactive}
          title="{tagDisplay} ({tag.usageCount} files)"
          class:cursor-pointer={interactive}
          class:cursor-default={!interactive}
        >
          <span class="tag-name">{tag.name}</span>
          <span class="tag-count">{tag.usageCount}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tag-cloud {
    width: 100%;
  }

  .tag-button {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.75rem;
    border-radius: 9999px;
    transition: all 0.2s ease-in-out;
    font-weight: 500;
    line-height: 1.2;
    border: 1px solid transparent;
  }

  .tag-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    filter: brightness(0.95);
  }

  .tag-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .tag-name {
    font-weight: inherit;
  }

  .tag-count {
    font-size: 0.75em;
    opacity: 0.7;
    font-weight: 600;
  }

  .spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid rgba(0, 0, 0, 0.1);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
