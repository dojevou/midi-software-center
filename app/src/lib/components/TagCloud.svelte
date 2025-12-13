<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Tag } from '$lib/types';

  // Extended Tag interface with optional file_count for backwards compatibility
  interface TagWithCount extends Tag {
    file_count?: number;
  }

  export let tags: TagWithCount[] = [];
  export let maxFontSize: number = 48;
  export let minFontSize: number = 12;
  export let selectedTags: number[] = [];

  const dispatch = createEventDispatcher<{
    tagClick: { tagId: number };
    tagSelect: { tagId: number; selected: boolean };
  }>();

  // Get count from either file_count or count property
  function getCount(tag: TagWithCount): number {
    return tag.file_count ?? tag.count ?? 0;
  }

  function calculateFontSize(tag: TagWithCount): number {
    if (tags.length === 0) {
      return minFontSize;
    }

    const maxCount = Math.max(...tags.map((t) => getCount(t)));
    const minCount = Math.min(...tags.map((t) => getCount(t)));

    if (maxCount === minCount) {
      return (maxFontSize + minFontSize) / 2;
    }

    const count = getCount(tag);
    const ratio = (count - minCount) / (maxCount - minCount);
    return minFontSize + ratio * (maxFontSize - minFontSize);
  }

  function getTagColor(index: number): string {
    const colors = [
      'text-blue-400',
      'text-green-400',
      'text-purple-400',
      'text-pink-400',
      'text-yellow-400',
      'text-orange-400',
      'text-red-400',
      'text-cyan-400',
    ];
    return colors[index % colors.length];
  }

  function handleTagClick(tag: TagWithCount) {
    dispatch('tagClick', { tagId: tag.id });
  }

  function handleTagSelect(tag: TagWithCount) {
    const isSelected = selectedTags.includes(tag.id);
    dispatch('tagSelect', { tagId: tag.id, selected: !isSelected });
  }

  function handleKeyDown(event: KeyboardEvent, tag: TagWithCount) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleTagClick(tag);
    }
  }
</script>

<div class="tag-cloud flex flex-wrap gap-3 justify-center items-center p-4">
  {#each tags as tag, index (tag.id)}
    <button
      class="tag-item transition-all hover:opacity-80 focus:outline-none focus:ring-2 focus:ring-primary rounded-lg px-2 py-1 {getTagColor(
        index
      )}"
      class:ring-2={selectedTags.includes(tag.id)}
      class:ring-primary={selectedTags.includes(tag.id)}
      class:dark:bg-window-subtle={selectedTags.includes(tag.id)}
      style="font-size: {calculateFontSize(tag)}px"
      on:click={() => handleTagClick(tag)}
      on:keydown={(e) => handleKeyDown(e, tag)}
      title="{tag.name} ({getCount(tag)} files)"
    >
      {tag.name}
    </button>
  {:else}
    <div class="text-center py-8 dark:text-gray-500">
      <div class="text-4xl mb-2">🏷️</div>
      <div>No tags to display</div>
    </div>
  {/each}
</div>

<style>
  .tag-item {
    cursor: pointer;
    line-height: 1.2;
  }

  .tag-item:hover {
    transform: scale(1.05);
  }
</style>
