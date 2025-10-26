<script lang="ts">
  import {
    selectedFile,
    favorites,
    isCurrentFileFavorited,
    toggleFavorite,
    addToRecent,
    searchResults,
    searchLoading,
  } from '../stores';
  import type { FileDetails } from '../api';

  export let files: FileDetails[] = [];

  // Use search results if no files provided
  $: displayFiles = files.length > 0 ? files : $searchResults;

  function selectFile(file: FileDetails) {
    selectedFile.set(file);
    addToRecent(file);
  }

  function handleFavoriteClick(event: MouseEvent, fileId: number) {
    event.stopPropagation();
    toggleFavorite(fileId);
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="file-browser">
  {#if $searchLoading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading files...</p>
    </div>
  {:else if displayFiles.length === 0}
    <div class="empty-state">
      <p>No files found</p>
      <span>Try adjusting your search filters</span>
    </div>
  {:else}
    <div class="file-list">
      {#each displayFiles as file (file.id)}
        <div
          class="file-item"
          class:selected={$selectedFile?.id === file.id}
          on:click={() => selectFile(file)}
          on:keypress={(e) => e.key === 'Enter' && selectFile(file)}
          role="button"
          tabindex="0"
        >
          <div class="file-icon">
            {#if file.has_drums}
              🥁
            {:else if file.has_notes}
              🎹
            {:else}
              🎵
            {/if}
          </div>

          <div class="file-info">
            <div class="file-name">{file.file_name}</div>
            <div class="file-meta">
              {#if file.bpm}
                <span class="meta-item">{file.bpm} BPM</span>
              {/if}
              {#if file.key}
                <span class="meta-item">{file.key}</span>
              {/if}
              {#if file.time_signature}
                <span class="meta-item">{file.time_signature}</span>
              {/if}
              <span class="meta-item">{formatDuration(file.duration_seconds ?? 0)}</span>
            </div>
            {#if file.category}
              <div class="file-category">{file.category}</div>
            {/if}
          </div>

          <button
            class="favorite-btn"
            class:favorited={$favorites.includes(file.id)}
            on:click={(e) => handleFavoriteClick(e, file.id)}
            title={$favorites.includes(file.id) ? 'Remove from favorites' : 'Add to favorites'}
          >
            {$favorites.includes(file.id) ? '★' : '☆'}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-browser {
    height: 100%;
    overflow-y: auto;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(255, 255, 255, 0.1);
    border-top-color: #ff3e00;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    text-align: center;
  }

  .empty-state p {
    font-size: 1.125rem;
    color: rgba(255, 255, 255, 0.7);
    margin: 0 0 0.5rem 0;
  }

  .empty-state span {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.4);
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .file-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 62, 0, 0.5);
    transform: translateX(4px);
  }

  .file-item.selected {
    background: rgba(255, 62, 0, 0.15);
    border-color: #ff3e00;
  }

  .file-icon {
    font-size: 2rem;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-weight: 600;
    color: #fff;
    margin-bottom: 0.25rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-meta {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .meta-item {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.6);
    padding: 0.125rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 4px;
  }

  .file-category {
    margin-top: 0.25rem;
    font-size: 0.75rem;
    color: #ff3e00;
    font-weight: 600;
  }

  .favorite-btn {
    width: 36px;
    height: 36px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.4);
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .favorite-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffd700;
    transform: scale(1.1);
  }

  .favorite-btn.favorited {
    color: #ffd700;
    background: rgba(255, 215, 0, 0.1);
    border-color: rgba(255, 215, 0, 0.3);
  }
</style>
