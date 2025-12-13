<script lang="ts">
  import { onMount } from 'svelte';
  import TagCloud from '$lib/components/TagCloud.svelte';
  import { api } from '$lib/api';
  import type { Tag, TagStats } from '$lib/types';

  // State using Svelte 4 let bindings
  let allTags: Tag[] = [];
  let selectedTags: number[] = [];
  let tagCategories: string[] = [];
  let newTag = { name: '', category: '', description: '' };
  let editTag: Tag | null = null;
  let filterCategory = '';
  let searchQuery = '';
  let tagStats: TagStats | null = null;
  let bulkOperation = {
    type: 'merge',
    sourceTags: [] as number[],
    targetTag: null as number | null,
  };
  let isLoading = false;

  onMount(async () => {
    await loadAllData();
  });

  async function loadAllData() {
    isLoading = true;
    try {
      allTags = await api.tags.getAllTags();
      tagCategories = await api.tags.getTagCategories();
      tagStats = await api.tags.getTagStats();
    } catch (error) {
      console.error('Failed to load tags:', error);
    } finally {
      isLoading = false;
    }
  }

  async function createTag() {
    if (!newTag.name.trim()) {
      return;
    }

    try {
      const created = await api.tags.createTag(newTag);
      allTags = [...allTags, created];
      const prevCategory = newTag.category;
      newTag = { name: '', category: '', description: '' };

      // Reload categories if new
      if (prevCategory && !tagCategories.includes(prevCategory)) {
        tagCategories = await api.tags.getTagCategories();
      }
    } catch (error) {
      console.error('Failed to create tag:', error);
    }
  }

  async function updateTag(tagId: number, updates: Partial<Tag>) {
    try {
      await api.tags.updateTag(tagId, updates);

      // Update local state
      const index = allTags.findIndex((t) => t.id === tagId);
      if (index !== -1) {
        allTags[index] = { ...allTags[index], ...updates };
        allTags = [...allTags];
      }
    } catch (error) {
      console.error('Failed to update tag:', error);
    }
  }

  async function deleteTag(tagId: number) {
    if (!confirm('Delete this tag? This will remove it from all files.')) {
      return;
    }

    try {
      await api.tags.deleteTag(tagId);
      allTags = allTags.filter((t) => t.id !== tagId);
      selectedTags = selectedTags.filter((id) => id !== tagId);
    } catch (error) {
      console.error('Failed to delete tag:', error);
    }
  }

  async function mergeTags() {
    if (bulkOperation.sourceTags.length < 2 || !bulkOperation.targetTag) {
      alert('Please select at least two source tags and a target tag');
      return;
    }

    try {
      await api.tags.mergeTags(bulkOperation.sourceTags, bulkOperation.targetTag);

      // Remove merged tags from list
      allTags = allTags.filter((t) => !bulkOperation.sourceTags.includes(t.id));
      bulkOperation = { type: 'merge', sourceTags: [], targetTag: null };

      await loadAllData(); // Reload stats
    } catch (error) {
      console.error('Failed to merge tags:', error);
    }
  }

  async function exportTags() {
    try {
      const csv = await api.tags.exportTagsCsv();
      const blob = new Blob([csv], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'tags_export.csv';
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export tags:', error);
    }
  }

  async function importTags(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const csv = e.target?.result as string;
        await api.tags.importTagsCsv(csv);
        await loadAllData();
        alert('Tags imported successfully');
      } catch (error) {
        console.error('Failed to import tags:', error);
        alert('Failed to import tags: ' + error);
      }
    };
    reader.readAsText(file);
  }

  function toggleTagSelection(tagId: number) {
    if (selectedTags.includes(tagId)) {
      selectedTags = selectedTags.filter((id) => id !== tagId);
    } else {
      selectedTags = [...selectedTags, tagId];
    }
  }

  function handleTagClick(tag: Tag) {
    editTag = { ...tag };
  }

  function handleEditInput(field: 'name' | 'category' | 'description', value: string) {
    if (editTag) {
      editTag = { ...editTag, [field]: value };
    } else {
      newTag = { ...newTag, [field]: value };
    }
  }

  function handleFormSubmit() {
    if (editTag) {
      updateTag(editTag.id, editTag);
    } else {
      createTag();
    }
  }

  function handleSaveAndClose() {
    if (editTag) {
      updateTag(editTag.id, editTag);
      editTag = null;
    }
  }

  function handleCancelEdit() {
    editTag = null;
  }

  function handleMergeSelected() {
    bulkOperation = {
      type: 'merge',
      sourceTags: [...selectedTags],
      targetTag: null,
    };
  }

  function handleDeleteSelected() {
    if (confirm(`Delete ${selectedTags.length} tags?`)) {
      selectedTags.forEach((tagId) => deleteTag(tagId));
      selectedTags = [];
    }
  }

  function handleClearSelection() {
    selectedTags = [];
  }

  function handleCancelMerge() {
    bulkOperation = { type: 'merge', sourceTags: [], targetTag: null };
  }

  function handleTagCloudClick(event: CustomEvent<{ tagId: number }>) {
    const tag = allTags.find((t) => t.id === event.detail.tagId);
    if (tag) {
      editTag = { ...tag };
    }
  }

  function handleFileInputClick() {
    const input = document.querySelector('input[type="file"]') as HTMLInputElement;
    if (input) {
      input.click();
    }
  }

  // Reactive statement for filtered tags (Svelte 4 pattern)
  $: filteredTags = allTags.filter((tag) => {
    const matchesSearch =
      searchQuery === '' ||
      tag.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (tag.category && tag.category.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesCategory = filterCategory === '' || tag.category === filterCategory;

    return matchesSearch && matchesCategory;
  });

  // Reactive for display values
  $: displayName = editTag ? editTag.name : newTag.name;
  $: displayCategory = editTag ? editTag.category : newTag.category;
  $: displayDescription = editTag ? (editTag as any).description || '' : newTag.description;
</script>

<div class="tag-editor-window dark:bg-window dark:text-app-text h-full flex">
  <!-- Left Panel: Tag List -->
  <div class="left-panel w-1/3 border-r dark:border-window-border flex flex-col">
    <!-- Header -->
    <div class="p-4 border-b dark:border-window-border">
      <h2 class="text-lg dark:text-gray-200 mb-3">Tag Management</h2>

      <!-- Search -->
      <div class="search mb-3">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search tags..."
          class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
        />
      </div>

      <!-- Category Filter -->
      <div class="category-filter mb-3">
        <select
          bind:value={filterCategory}
          class="w-full px-2 py-1 dark:bg-input dark:border-window-border rounded"
        >
          <option value="">All Categories</option>
          {#each tagCategories as category (category)}
            <option value={category}>{category}</option>
          {/each}
        </select>
      </div>

      <!-- Stats -->
      {#if tagStats}
        <div class="stats text-sm dark:text-gray-400">
          <div>Total Tags: {tagStats.total_tags}</div>
          <div>Total File Tags: {tagStats.total_file_tags}</div>
        </div>
      {/if}
    </div>

    <!-- Tag List -->
    <div class="tag-list flex-1 overflow-auto p-4">
      {#if isLoading}
        <div class="loading dark:text-gray-500 text-center py-8">Loading tags...</div>
      {:else if filteredTags.length === 0}
        <div class="no-tags dark:text-gray-500 text-center py-8">
          {searchQuery ? 'No tags match your search' : 'No tags yet'}
        </div>
      {:else}
        <div class="space-y-2">
          {#each filteredTags as tag (tag.id)}
            <div
              class="tag-item p-3 rounded border dark:border-window-border cursor-pointer transition-all"
              class:dark:bg-secondary={selectedTags.includes(tag.id)}
              class:dark:hover:bg-window-subtle={!selectedTags.includes(tag.id)}
              on:click={() => handleTagClick(tag)}
              on:keydown={(e) => e.key === 'Enter' && handleTagClick(tag)}
              role="button"
              tabindex="0"
            >
              <div class="flex justify-between items-start">
                <div class="flex-1">
                  <div class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={selectedTags.includes(tag.id)}
                      on:click|stopPropagation={() => toggleTagSelection(tag.id)}
                      class="rounded"
                    />
                    <h3 class="text-sm dark:text-gray-200 font-medium">{tag.name}</h3>
                    {#if tag.category}
                      <span
                        class="text-xs px-2 py-0.5 dark:bg-menu rounded-full dark:text-gray-400"
                      >
                        {tag.category}
                      </span>
                    {/if}
                  </div>

                  <div class="text-xs dark:text-gray-500 mt-2">
                    Used in {tag.count}
                    {tag.count === 1 ? 'file' : 'files'}
                  </div>
                </div>

                <div class="flex gap-1">
                  <button
                    on:click|stopPropagation={() => handleTagClick(tag)}
                    class="p-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                    title="Edit"
                  >
                    ✏️
                  </button>
                  <button
                    on:click|stopPropagation={() => deleteTag(tag.id)}
                    class="p-1 text-xs dark:bg-error rounded hover:opacity-80"
                    title="Delete"
                  >
                    🗑
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Bulk Actions -->
    <div class="bulk-actions p-4 border-t dark:border-window-border">
      <div class="text-sm dark:text-gray-300 mb-2">
        {selectedTags.length} tag{selectedTags.length === 1 ? '' : 's'} selected
      </div>

      <div class="flex flex-wrap gap-2">
        <button
          on:click={handleMergeSelected}
          class="px-3 py-1 text-sm dark:bg-primary rounded hover:opacity-80"
          disabled={selectedTags.length < 2}
        >
          Merge Selected
        </button>

        <button
          on:click={handleDeleteSelected}
          class="px-3 py-1 text-sm dark:bg-error rounded hover:opacity-80"
          disabled={selectedTags.length === 0}
        >
          Delete Selected
        </button>

        <button
          on:click={handleClearSelection}
          class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
        >
          Clear Selection
        </button>
      </div>
    </div>
  </div>

  <!-- Right Panel: Editor & Details -->
  <div class="right-panel flex-1 flex flex-col">
    <!-- Tag Editor -->
    <div class="editor p-6 border-b dark:border-window-border">
      <h3 class="text-lg dark:text-gray-200 mb-4">
        {editTag ? 'Edit Tag' : 'Create New Tag'}
      </h3>

      <form on:submit|preventDefault={handleFormSubmit}>
        <div class="space-y-4">
          <div>
            <label for="tag-name" class="block text-sm dark:text-gray-400 mb-1">Tag Name</label>
            <input
              id="tag-name"
              type="text"
              value={displayName}
              on:input={(e) => handleEditInput('name', e.currentTarget.value)}
              class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              placeholder="e.g., Jazz, Upbeat, Synth Bass"
              required
            />
          </div>

          <div>
            <label for="tag-category" class="block text-sm dark:text-gray-400 mb-1">Category</label>
            <div class="flex gap-2">
              <select
                id="tag-category"
                value={displayCategory}
                on:change={(e) => handleEditInput('category', e.currentTarget.value)}
                class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
              >
                <option value="">No Category</option>
                {#each tagCategories as category (category)}
                  <option value={category}>{category}</option>
                {/each}
              </select>
              {#if !editTag}
                <input
                  type="text"
                  value={newTag.category}
                  on:input={(e) => (newTag.category = e.currentTarget.value)}
                  class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  placeholder="Or enter new category..."
                />
              {/if}
            </div>
          </div>

          <div>
            <label for="tag-description" class="block text-sm dark:text-gray-400 mb-1"
              >Description</label
            >
            <textarea
              id="tag-description"
              value={displayDescription}
              on:input={(e) => handleEditInput('description', e.currentTarget.value)}
              class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded h-24"
              placeholder="Optional description..."
            ></textarea>
          </div>

          <div class="flex gap-3">
            {#if editTag}
              <button
                type="submit"
                class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
              >
                Update Tag
              </button>
              <button
                type="button"
                on:click={handleSaveAndClose}
                class="px-4 py-2 dark:bg-secondary dark:text-gray-300 rounded hover:opacity-80"
              >
                Save & Close
              </button>
              <button
                type="button"
                on:click={handleCancelEdit}
                class="px-4 py-2 dark:bg-secondary dark:text-gray-300 rounded hover:opacity-80"
              >
                Cancel
              </button>
            {:else}
              <button
                type="submit"
                class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
              >
                Create Tag
              </button>
            {/if}
          </div>
        </div>
      </form>
    </div>

    <!-- Tag Cloud Visualization -->
    <div class="visualization p-6 flex-1">
      <h3 class="text-lg dark:text-gray-200 mb-4">Tag Cloud</h3>
      <div class="h-64">
        <TagCloud
          tags={allTags.slice(0, 50)}
          maxFontSize={48}
          minFontSize={12}
          on:tagClick={handleTagCloudClick}
        />
      </div>
    </div>

    <!-- Import/Export -->
    <div class="import-export p-6 border-t dark:border-window-border">
      <div class="flex gap-4">
        <div class="flex-1">
          <h4 class="text-sm dark:text-gray-300 mb-2">Import Tags</h4>
          <div class="flex gap-2">
            <input
              type="file"
              accept=".csv,.json"
              on:change={importTags}
              class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
            />
            <button
              on:click={handleFileInputClick}
              class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Browse...
            </button>
          </div>
          <p class="text-xs dark:text-gray-500 mt-1">CSV format: name,category,description</p>
        </div>

        <div class="flex-1">
          <h4 class="text-sm dark:text-gray-300 mb-2">Export Tags</h4>
          <button
            on:click={exportTags}
            class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
          >
            Export as CSV
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Merge Tags Modal -->
  {#if bulkOperation.type === 'merge' && bulkOperation.sourceTags.length > 0}
    <div
      class="modal-overlay fixed inset-0 dark:bg-black dark:bg-opacity-50 flex items-center justify-center z-50"
    >
      <div class="modal dark:bg-window p-6 rounded-lg w-96">
        <h3 class="text-lg dark:text-gray-200 mb-4">Merge Tags</h3>

        <div class="mb-4">
          <p class="text-sm dark:text-gray-400 mb-2">
            Merging {bulkOperation.sourceTags.length} tags into:
          </p>

          <select
            bind:value={bulkOperation.targetTag}
            class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded mb-2"
          >
            <option value={null}>Select target tag...</option>
            {#each allTags.filter((t) => !bulkOperation.sourceTags.includes(t.id)) as tag (tag.id)}
              <option value={tag.id}>{tag.name} ({tag.count} files)</option>
            {/each}
          </select>

          <p class="text-xs dark:text-gray-500">
            All files with source tags will be updated to use the target tag.
          </p>
        </div>

        <div class="flex gap-3">
          <button
            on:click={mergeTags}
            disabled={!bulkOperation.targetTag}
            class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80 disabled:opacity-50"
          >
            Merge Tags
          </button>
          <button
            on:click={handleCancelMerge}
            class="px-4 py-2 dark:bg-secondary dark:text-gray-300 rounded hover:opacity-80"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .tag-cloud-word {
    cursor: pointer;
    transition: transform 0.2s;
  }

  .tag-cloud-word:hover {
    transform: scale(1.1);
    text-decoration: underline;
  }

  .modal-overlay {
    backdrop-filter: blur(4px);
  }

  .tag-item:focus {
    outline: 2px solid var(--primary-color);
    outline-offset: 2px;
  }
</style>
