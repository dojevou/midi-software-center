<script lang="ts">
  import { searchStore } from '$lib/stores/search';
  import type { SearchFilters, FileCategory, MusicalKey } from '$lib/types';
  import { isValidBpm } from '$lib/utils/validators';
  import TagCloud from '$lib/components/Tags/TagCloud.svelte';
  import { selectedTags, tagMatchMode, clearSelectedTags, deselectTag, toggleTagMatchMode } from '$lib/stores/tags';

  let localFilters: SearchFilters = { ...$searchStore.filters };
  let showTagsSection = false;

  const categories: FileCategory[] = [
    'KICK', 'SNARE', 'HIHAT', 'PERCUSSION', 'BASS',
    'LEAD', 'PAD', 'CHORD', 'ARP', 'FX', 'VOCAL', 'LOOP', 'UNKNOWN'
  ];

  const keys: MusicalKey[] = [
    'C', 'Cm', 'C#', 'C#m', 'D', 'Dm', 'D#', 'D#m',
    'E', 'Em', 'F', 'Fm', 'F#', 'F#m', 'G', 'Gm',
    'G#', 'G#m', 'A', 'Am', 'A#', 'A#m', 'B', 'Bm', 'UNKNOWN'
  ];

  function applyFilters() {
    if (localFilters.minBpm && !isValidBpm(localFilters.minBpm)) {
      alert('Minimum BPM must be between 20 and 300');
      return;
    }
    if (localFilters.maxBpm && !isValidBpm(localFilters.maxBpm)) {
      alert('Maximum BPM must be between 20 and 300');
      return;
    }
    if (localFilters.minBpm && localFilters.maxBpm && localFilters.minBpm > localFilters.maxBpm) {
      alert('Minimum BPM cannot be greater than maximum BPM');
      return;
    }

    // Include selected tags in filters
    const filtersWithTags = {
      ...localFilters,
      tags: $selectedTags.length > 0 ? $selectedTags : undefined,
      tagMatchAll: $tagMatchMode === 'all',
    };

    searchStore.search($searchStore.query, filtersWithTags);
  }

  function clearFilters() {
    localFilters = {};
    clearSelectedTags();
    searchStore.search($searchStore.query, {});
  }

  $: hasActiveFilters = Object.keys(localFilters).length > 0 || $selectedTags.length > 0;
</script>

<div class="card p-4 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h3 class="font-semibold text-slate-900 dark:text-slate-100 text-lg">Filters</h3>
    {#if hasActiveFilters}
      <button
        on:click={clearFilters}
        class="text-sm text-primary-600 hover:text-primary-700 transition-colors"
      >
        Clear All
      </button>
    {/if}
  </div>

  <!-- Category Filter -->
  <div>
    <label for="category-select" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
      Category
    </label>
    <select
      id="category-select"
      bind:value={localFilters.category}
      class="input w-full"
    >
      <option value="">All Categories</option>
      {#each categories as category}
        <option value={category}>{category}</option>
      {/each}
    </select>
  </div>

  <!-- BPM Range -->
  <div>
    <label class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
      BPM Range
    </label>
    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="min-bpm-input" class="sr-only">Minimum BPM</label>
        <input
          id="min-bpm-input"
          type="number"
          bind:value={localFilters.minBpm}
          placeholder="Min"
          min="20"
          max="300"
          class="input w-full"
        />
      </div>
      <div>
        <label for="max-bpm-input" class="sr-only">Maximum BPM</label>
        <input
          id="max-bpm-input"
          type="number"
          bind:value={localFilters.maxBpm}
          placeholder="Max"
          min="20"
          max="300"
          class="input w-full"
        />
      </div>
    </div>
    <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">
      Range: 20 - 300 BPM
    </p>
  </div>

  <!-- Key Filter -->
  <div>
    <label for="key-select" class="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
      Musical Key
    </label>
    <select
      id="key-select"
      bind:value={localFilters.key}
      class="input w-full"
    >
      <option value="">Any Key</option>
      {#each keys as key}
        <option value={key}>{key}</option>
      {/each}
    </select>
  </div>

  <!-- Tags Filter -->
  <div>
    <button
      on:click={() => showTagsSection = !showTagsSection}
      class="w-full flex items-center justify-between text-sm font-medium text-slate-700 dark:text-slate-300 mb-2"
    >
      <span>Tags</span>
      <span class="text-xs">{showTagsSection ? '▼' : '▶'}</span>
    </button>

    {#if showTagsSection}
      <div class="space-y-3">
        <!-- Selected Tags -->
        {#if $selectedTags.length > 0}
          <div class="flex flex-wrap gap-2 mb-3">
            {#each $selectedTags as tag}
              <span class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200">
                {tag}
                <button
                  on:click={() => deselectTag(tag)}
                  class="hover:bg-primary-200 dark:hover:bg-primary-800 rounded-full p-0.5"
                  aria-label="Remove tag"
                >×</button>
              </span>
            {/each}
          </div>

          <!-- AND/OR Toggle -->
          <div class="flex items-center gap-2 mb-3">
            <span class="text-xs text-slate-600 dark:text-slate-400">Match:</span>
            <button
              on:click={toggleTagMatchMode}
              class="px-2 py-1 rounded text-xs font-medium transition-colors {$tagMatchMode === 'all' ? 'bg-primary-600 text-white' : 'bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300'}"
            >
              ALL
            </button>
            <button
              on:click={toggleTagMatchMode}
              class="px-2 py-1 rounded text-xs font-medium transition-colors {$tagMatchMode === 'any' ? 'bg-primary-600 text-white' : 'bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300'}"
            >
              ANY
            </button>
          </div>
        {/if}

        <!-- Tag Cloud -->
        <div class="max-h-64 overflow-y-auto">
          <TagCloud maxTags={50} />
        </div>
      </div>
    {/if}
  </div>

  <!-- Active Filters Summary -->
  {#if hasActiveFilters}
    <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
      <h4 class="text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">Active Filters</h4>
      <div class="flex flex-wrap gap-1">
        {#if localFilters.category}
          <span class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200">
            Category: {localFilters.category}
            <button
              on:click={() => localFilters = { ...localFilters, category: undefined }}
              class="ml-1 hover:bg-primary-200 dark:hover:bg-primary-800 rounded-full p-0.5"
              aria-label="Remove category filter"
            >×</button>
          </span>
        {/if}
        {#if localFilters.minBpm || localFilters.maxBpm}
          <span class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200">
            BPM: {localFilters.minBpm || 'Any'} - {localFilters.maxBpm || 'Any'}
            <button
              on:click={() => localFilters = { ...localFilters, minBpm: undefined, maxBpm: undefined }}
              class="ml-1 hover:bg-primary-200 dark:hover:bg-primary-800 rounded-full p-0.5"
              aria-label="Remove BPM filter"
            >×</button>
          </span>
        {/if}
        {#if localFilters.key}
          <span class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200">
            Key: {localFilters.key}
            <button
              on:click={() => localFilters = { ...localFilters, key: undefined }}
              class="ml-1 hover:bg-primary-200 dark:hover:bg-primary-800 rounded-full p-0.5"
              aria-label="Remove key filter"
            >×</button>
          </span>
        {/if}
        {#if $selectedTags.length > 0}
          <span class="inline-flex items-center px-2 py-1 rounded-full text-xs bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200">
            Tags: {$selectedTags.length} ({$tagMatchMode === 'all' ? 'ALL' : 'ANY'})
            <button
              on:click={clearSelectedTags}
              class="ml-1 hover:bg-primary-200 dark:hover:bg-primary-800 rounded-full p-0.5"
              aria-label="Clear tags"
            >×</button>
          </span>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Action Buttons -->
  <div class="flex gap-2 pt-2">
    <button
      on:click={applyFilters}
      class="btn-primary flex-1 px-4 py-2 text-sm font-medium"
      disabled={!hasActiveFilters}
    >
      Apply Filters
    </button>
    <button
      on:click={clearFilters}
      class="btn-secondary px-4 py-2 text-sm font-medium"
      disabled={!hasActiveFilters}
    >
      Clear
    </button>
  </div>
</div>
