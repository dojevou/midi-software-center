<script lang="ts">
  import { onMount } from 'svelte';
  import FileBrowser from '$lib/components/FileBrowser.svelte';
  import { api } from '$lib/api';
  import type { FileDetails } from '$lib/types';

  interface FavoriteFile extends FileDetails {
    favorited_at?: string;
    thumbnail_url?: string;
  }

  // State using Svelte 4 let bindings
  let favorites: FavoriteFile[] = [];
  let categories: Record<string, FavoriteFile[]> = {};
  let viewMode: 'grid' | 'list' | 'detailed' = 'grid';
  let sortBy = 'added_date';
  let sortDesc = true;
  let searchQuery = '';
  let selectedCategory = '';
  let isLoading = false;

  onMount(async () => {
    await loadFavorites();
  });

  async function loadFavorites() {
    isLoading = true;
    try {
      favorites = await api.favorites.getFavorites();
      organizeCategories();
    } catch (error) {
      console.error('Failed to load favorites:', error);
    } finally {
      isLoading = false;
    }
  }

  function organizeCategories() {
    const cats: Record<string, FavoriteFile[]> = {};
    favorites.forEach((file) => {
      file.tags?.forEach((tag) => {
        if (!cats[tag]) {
          cats[tag] = [];
        }
        if (!cats[tag].find((f) => f.id === file.id)) {
          cats[tag].push(file);
        }
      });
    });
    categories = cats;
  }

  async function toggleFavorite(fileId: number) {
    try {
      const isFavorite = favorites.some((f) => f.id === fileId);
      if (isFavorite) {
        await api.favorites.removeFavorite(fileId);
      } else {
        await api.favorites.addFavorite(fileId);
      }
      await loadFavorites();
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }

  async function removeFromFavorites(fileId: number) {
    if (!confirm('Remove from favorites?')) {
      return;
    }

    try {
      await api.favorites.removeFavorite(fileId);
      favorites = favorites.filter((f) => f.id !== fileId);
      organizeCategories();
    } catch (error) {
      console.error('Failed to remove favorite:', error);
    }
  }

  async function playFile(fileId: number) {
    try {
      await api.project.loadMultipleTracks([fileId]);
      await api.transport.play();
    } catch (error) {
      console.error('Failed to play file:', error);
    }
  }

  async function analyzeFile(fileId: number) {
    try {
      await api.compatibility.findCompatibleFiles(fileId);
      // Show compatibility results
    } catch (error) {
      console.error('Failed to analyze file:', error);
    }
  }

  function exportFavorites() {
    const data = {
      favorites: favorites.map((f) => ({
        id: f.id,
        filename: f.filename,
        added_date: f.favorited_at,
        tags: f.tags,
      })),
      exported_at: new Date().toISOString(),
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `favorites_${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleViewModeChange(mode: 'grid' | 'list' | 'detailed') {
    viewMode = mode;
  }

  function handleCategoryClick(category: string) {
    selectedCategory = selectedCategory === category ? '' : category;
  }

  function handleSortToggle() {
    sortDesc = !sortDesc;
  }

  function handlePlayEvent(
    event: CustomEvent<{ fileId: number; ctrlKey: boolean; shiftKey: boolean }>
  ) {
    playFile(event.detail.fileId);
  }

  function handleAnalyzeEvent(event: CustomEvent<{ fileId: number }>) {
    analyzeFile(event.detail.fileId);
  }

  function handleToggleFavoriteEvent(event: CustomEvent<number>) {
    toggleFavorite(event.detail);
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatSize(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'today';
    }
    if (diffDays === 1) {
      return 'yesterday';
    }
    if (diffDays < 7) {
      return `${diffDays} days ago`;
    }
    if (diffDays < 30) {
      return `${Math.floor(diffDays / 7)} weeks ago`;
    }
    return date.toLocaleDateString();
  }

  // Reactive statement for filtered favorites (Svelte 4 pattern)
  $: filteredFavorites = favorites
    .filter((file) => {
      const matchesSearch =
        searchQuery === '' ||
        file.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
        file.tags?.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchesCategory = selectedCategory === '' || file.tags?.includes(selectedCategory);

      return matchesSearch && matchesCategory;
    })
    .sort((a, b) => {
      let valA: string | number;
      let valB: string | number;
      switch (sortBy) {
        case 'filename':
          valA = a.filename.toLowerCase();
          valB = b.filename.toLowerCase();
          break;
        case 'bpm':
          valA = a.bpm || 0;
          valB = b.bpm || 0;
          break;
        case 'duration':
          valA = a.duration_seconds || 0;
          valB = b.duration_seconds || 0;
          break;
        case 'added_date':
          valA = new Date(a.favorited_at || a.created_at).getTime();
          valB = new Date(b.favorited_at || b.created_at).getTime();
          break;
        default:
          valA = (a as any)[sortBy] || 0;
          valB = (b as any)[sortBy] || 0;
      }

      return sortDesc ? (valA > valB ? -1 : 1) : valA < valB ? -1 : 1;
    });

  // Transform to FileItem format for FileBrowser component
  $: fileBrowserItems = filteredFavorites.map((f) => ({
    id: f.id,
    name: f.filename,
    path: f.filepath,
    type: 'file' as const,
    size: f.file_size_bytes,
    modified: f.created_at,
    duration: f.duration_seconds,
    bpm: f.bpm,
    key: f.key_signature,
    tags: f.tags,
  }));

  $: categoryKeys = Object.keys(categories);
</script>

<div class="favorites-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header -->
  <div class="header p-4 border-b dark:border-window-border">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl dark:text-gray-200">
        <span class="text-yellow-400">★</span> Favorites
        <span class="text-lg dark:text-gray-400 ml-2">({favorites.length} files)</span>
      </h2>

      <div class="flex gap-2">
        <button
          on:click={exportFavorites}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 12h2v-2h2v2h2V8h2l-4-4-4 4h2v4zM5 14h10v2H5v-2z" />
          </svg>
          Export
        </button>

        <div class="view-toggle flex border dark:border-window-border rounded overflow-hidden">
          <button
            on:click={() => handleViewModeChange('grid')}
            class="px-3 py-1"
            class:dark:bg-secondary={viewMode === 'grid'}
            title="Grid View"
          >
            ⏹
          </button>
          <button
            on:click={() => handleViewModeChange('list')}
            class="px-3 py-1"
            class:dark:bg-secondary={viewMode === 'list'}
            title="List View"
          >
            ≡
          </button>
          <button
            on:click={() => handleViewModeChange('detailed')}
            class="px-3 py-1"
            class:dark:bg-secondary={viewMode === 'detailed'}
            title="Detailed View"
          >
            ⧉
          </button>
        </div>
      </div>
    </div>

    <!-- Search & Filter -->
    <div class="search-filters flex gap-4">
      <div class="search flex-1">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search favorites..."
          class="w-full px-4 py-2 dark:bg-input dark:border-window-border rounded"
        />
      </div>

      <div class="category-filter w-48">
        <select
          bind:value={selectedCategory}
          class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
        >
          <option value="">All Categories</option>
          {#each categoryKeys as category (category)}
            <option value={category}>{category} ({categories[category].length})</option>
          {/each}
        </select>
      </div>

      <div class="sort-controls flex items-center gap-2">
        <span class="text-sm dark:text-gray-400">Sort:</span>
        <select
          bind:value={sortBy}
          class="px-2 py-1 dark:bg-input dark:border-window-border rounded"
        >
          <option value="added_date">Date Added</option>
          <option value="filename">Filename</option>
          <option value="bpm">BPM</option>
          <option value="duration">Duration</option>
          <option value="file_size_bytes">Size</option>
        </select>
        <button on:click={handleSortToggle} class="px-2 py-1 dark:bg-secondary rounded">
          {sortDesc ? '↓' : '↑'}
        </button>
      </div>
    </div>
  </div>

  <!-- Category Navigation -->
  <div class="categories p-4 border-b dark:border-window-border">
    <div class="flex flex-wrap gap-2">
      {#each categoryKeys.slice(0, 10) as category (category)}
        <button
          on:click={() => handleCategoryClick(category)}
          class="px-3 py-1 rounded-full text-sm transition-all"
          class:dark:bg-primary={selectedCategory === category}
          class:dark:bg-secondary={selectedCategory !== category}
          class:dark:text-white={selectedCategory === category}
          class:dark:text-gray-300={selectedCategory !== category}
        >
          {category} ({categories[category].length})
        </button>
      {/each}
      {#if categoryKeys.length > 10}
        <button class="px-3 py-1 dark:bg-secondary rounded-full text-sm dark:text-gray-300">
          +{categoryKeys.length - 10} more
        </button>
      {/if}
    </div>
  </div>

  <!-- Favorites Content -->
  <div class="content flex-1 overflow-auto p-4">
    {#if isLoading}
      <div class="loading dark:text-gray-500 text-center py-12">
        <div
          class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 dark:border-gray-400 mb-2"
        ></div>
        <div>Loading favorites...</div>
      </div>
    {:else if favorites.length === 0}
      <div class="empty-state text-center py-12">
        <div class="text-6xl mb-4 dark:text-gray-600">★</div>
        <h3 class="text-xl dark:text-gray-300 mb-2">No favorites yet</h3>
        <p class="dark:text-gray-500 mb-6">
          Star files you like to add them here for quick access.
        </p>
        <button class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80">
          Browse Files
        </button>
      </div>
    {:else if viewMode === 'grid'}
      <!-- Grid View -->
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
        {#each filteredFavorites as file (file.id)}
          <div
            class="favorite-card dark:bg-window-subtle rounded-lg border dark:border-window-border overflow-hidden hover:shadow-lg transition-shadow"
          >
            <div class="aspect-video dark:bg-menu flex items-center justify-center relative group">
              {#if file.thumbnail_url}
                <img
                  src={file.thumbnail_url}
                  alt={file.filename}
                  class="w-full h-full object-cover"
                />
              {:else}
                <div class="text-4xl dark:text-gray-600">♫</div>
              {/if}

              <div
                class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 transition-all flex items-center justify-center opacity-0 group-hover:opacity-100"
              >
                <div class="flex gap-2">
                  <button
                    on:click={() => playFile(file.id)}
                    class="p-2 dark:bg-primary rounded-full hover:scale-110 transition-transform"
                    title="Play"
                  >
                    ▶
                  </button>
                  <button
                    on:click={() => analyzeFile(file.id)}
                    class="p-2 dark:bg-secondary rounded-full hover:scale-110 transition-transform"
                    title="Analyze"
                  >
                    🔍
                  </button>
                  <button
                    on:click={() => removeFromFavorites(file.id)}
                    class="p-2 dark:bg-error rounded-full hover:scale-110 transition-transform"
                    title="Remove"
                  >
                    ★
                  </button>
                </div>
              </div>
            </div>

            <div class="p-3">
              <h4 class="font-medium dark:text-gray-200 truncate mb-1" title={file.filename}>
                {file.filename}
              </h4>

              <div class="text-xs dark:text-gray-400 space-y-1">
                {#if file.bpm}
                  <div class="flex items-center gap-1">
                    <span class="text-yellow-400">♪</span>
                    {Math.round(file.bpm)} BPM
                  </div>
                {/if}

                {#if file.key_signature}
                  <div class="flex items-center gap-1">
                    <span class="text-blue-400">#</span>
                    {file.key_signature}
                  </div>
                {/if}

                {#if file.duration_seconds}
                  <div class="flex items-center gap-1">
                    <span class="text-green-400">⏱</span>
                    {formatDuration(file.duration_seconds)}
                  </div>
                {/if}
              </div>

              <div class="tags mt-2 flex flex-wrap gap-1">
                {#each (file.tags || []).slice(0, 3) as tag (tag)}
                  <span class="text-xs px-2 py-0.5 dark:bg-menu rounded-full dark:text-gray-400">
                    {tag}
                  </span>
                {/each}
                {#if file.tags && file.tags.length > 3}
                  <span class="text-xs px-2 py-0.5 dark:text-gray-500">
                    +{file.tags.length - 3}
                  </span>
                {/if}
              </div>

              <div class="mt-3 text-xs dark:text-gray-500">
                Added {formatDate(file.favorited_at || file.created_at)}
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else if viewMode === 'list'}
      <!-- List View -->
      <div class="favorites-list space-y-2">
        {#each filteredFavorites as file (file.id)}
          <div
            class="favorite-item p-3 dark:bg-window-subtle rounded border dark:border-window-border hover:dark:bg-menu transition-colors"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3 flex-1 min-w-0">
                <div
                  class="flex-shrink-0 w-10 h-10 dark:bg-menu rounded flex items-center justify-center"
                >
                  <span class="text-lg">♫</span>
                </div>

                <div class="flex-1 min-w-0">
                  <h4 class="font-medium dark:text-gray-200 truncate">{file.filename}</h4>
                  <div class="text-xs dark:text-gray-400 flex gap-3">
                    {#if file.bpm}
                      <span>{Math.round(file.bpm)} BPM</span>
                    {/if}
                    {#if file.key_signature}
                      <span>{file.key_signature}</span>
                    {/if}
                    {#if file.duration_seconds}
                      <span>{formatDuration(file.duration_seconds)}</span>
                    {/if}
                    <span>{formatSize(file.file_size_bytes)}</span>
                  </div>
                </div>
              </div>

              <div class="tags flex gap-1 flex-wrap justify-end max-w-xs">
                {#each (file.tags || []).slice(0, 5) as tag (tag)}
                  <span class="text-xs px-2 py-0.5 dark:bg-menu rounded dark:text-gray-400">
                    {tag}
                  </span>
                {/each}
              </div>

              <div class="actions flex gap-2 ml-4">
                <button
                  on:click={() => playFile(file.id)}
                  class="p-2 dark:bg-secondary rounded hover:opacity-80"
                  title="Play"
                >
                  ▶
                </button>
                <button
                  on:click={() => analyzeFile(file.id)}
                  class="p-2 dark:bg-secondary rounded hover:opacity-80"
                  title="Analyze"
                >
                  🔍
                </button>
                <button
                  on:click={() => removeFromFavorites(file.id)}
                  class="p-2 dark:bg-error rounded hover:opacity-80"
                  title="Remove from favorites"
                >
                  ★
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <!-- Detailed View -->
      <FileBrowser
        files={fileBrowserItems}
        showDetails={true}
        on:fileSelect={handlePlayEvent}
        on:fileOpen={handleAnalyzeEvent}
      />
    {/if}
  </div>

  <!-- Stats Footer -->
  <div class="footer p-3 border-t dark:border-window-border text-sm dark:text-gray-400">
    <div class="flex justify-between">
      <div>
        Showing {filteredFavorites.length} of {favorites.length} favorites
      </div>
      <div class="space-x-4">
        <span>By Category:</span>
        {#each categoryKeys.slice(0, 5) as category (category)}
          <span class="dark:text-gray-300">
            {category}: {categories[category].length}
          </span>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .favorite-card {
    transition:
      transform 0.2s,
      box-shadow 0.2s;
  }

  .favorite-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
  }

  .empty-state {
    animation: fadeIn 0.5s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
