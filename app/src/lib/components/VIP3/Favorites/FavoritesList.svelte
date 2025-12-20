<script lang="ts">
  import { onMount } from 'svelte';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { Vip3SearchResponse } from '$lib/types/vip3';
  import FavoritesButton from './FavoritesButton.svelte';

  export let onFileSelect: (fileId: number) => void = () => {};

  let favorites: Vip3SearchResponse | null = null;
  let loading = true;
  let error: string | null = null;
  let page = 1;
  const pageSize = 50;
  let totalCount = 0;

  async function loadFavorites() {
    try {
      loading = true;
      error = null;
      favorites = await Vip3BrowserApi.getFavorites(page, pageSize);
      totalCount = favorites.total_count;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load favorites';
      console.error('Error loading favorites:', e);
    } finally {
      loading = false;
    }
  }

  function handleFavoriteToggle(fileId: number, newState: boolean) {
    if (!newState) {
      loadFavorites();
    }
  }

  function handlePageChange(newPage: number) {
    page = newPage;
    loadFavorites();
  }

  onMount(() => {
    loadFavorites();
  });

  $: if (page) {
    loadFavorites();
  }
</script>

<div class="favorites-list">
  <div class="header">
    <h3>Favorites</h3>
    {#if totalCount > 0}
      <span class="count-badge">{totalCount}</span>
    {/if}
  </div>

  <div class="content">
    {#if loading}
      <div class="loading">Loading favorites...</div>
    {:else if error}
      <div class="error">
        <p>{error}</p>
        <button on:click={loadFavorites}>Retry</button>
      </div>
    {:else if !favorites || favorites.files.length === 0}
      <div class="empty">
        <div class="empty-icon">❤️</div>
        <p>No favorites yet</p>
        <p class="hint">Click the heart icon on any file to add it to your favorites</p>
      </div>
    {:else}
      <div class="file-list">
        <div class="file-count-header">
          {favorites.total_count} favorites
          {#if favorites.total_pages > 1}
            (Page {page} of {favorites.total_pages})
          {/if}
        </div>
        {#each favorites.files as file (file.id)}
          <div class="file-item" on:click={() => onFileSelect(file.id)}>
            <div class="file-info">
              <span class="file-name">{file.filename}</span>
              <div class="file-meta">
                {#if file.bpm}
                  <span class="meta">♩ {file.bpm} BPM</span>
                {/if}
                {#if file.key_signature}
                  <span class="meta">🎹 {file.key_signature}</span>
                {/if}
                {#if file.duration_ms}
                  <span class="meta">⏱️ {Math.floor(file.duration_ms / 60000)}:{String(Math.floor((file.duration_ms % 60000) / 1000)).padStart(2, '0')}</span>
                {/if}
              </div>
            </div>
            <FavoritesButton
              fileId={file.id}
              isFavorite={true}
              onToggle={(newState) => handleFavoriteToggle(file.id, newState)}
              size="small"
            />
          </div>
        {/each}
      </div>

      {#if favorites && favorites.total_pages > 1}
        <div class="pagination">
          <button
            on:click={() => handlePageChange(Math.max(1, page - 1))}
            disabled={page === 1}
          >
            ← Prev
          </button>
          <span>Page {page} of {favorites?.total_pages ?? 1}</span>
          <button
            on:click={() => handlePageChange(Math.min(favorites?.total_pages ?? 1, page + 1))}
            disabled={page === (favorites?.total_pages ?? 1)}
          >
            Next →
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .favorites-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 8px;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid var(--color-border, #3a3a3a);
    gap: 12px;
  }

  h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text, #ffffff);
  }

  .count-badge {
    padding: 4px 10px;
    background: var(--color-error, #ff3b30);
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    color: white;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .loading,
  .error,
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
    color: var(--color-text-secondary, #999);
  }

  .error {
    color: var(--color-error, #ff3b30);
  }

  .error button {
    margin-top: 12px;
    padding: 8px 16px;
    background: var(--color-primary, #007aff);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
  }

  .empty .hint {
    margin-top: 8px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .file-count-header {
    margin-bottom: 12px;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-secondary, #999);
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--color-bg-tertiary, #333);
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .file-item:hover {
    background: var(--color-bg-hover, #3a3a3a);
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-size: 14px;
    color: var(--color-text, #fff);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .meta {
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border, #3a3a3a);
  }

  .pagination button {
    padding: 8px 16px;
    background: var(--color-bg-tertiary, #333);
    color: var(--color-text, #fff);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .pagination button:hover:not(:disabled) {
    background: var(--color-bg-hover, #3a3a3a);
  }

  .pagination button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .pagination span {
    font-size: 14px;
    color: var(--color-text-secondary, #999);
  }
</style>
