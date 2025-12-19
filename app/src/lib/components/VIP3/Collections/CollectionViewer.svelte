<script lang="ts">
  import { onMount } from 'svelte';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { CollectionResponse, Vip3SearchResponse } from '$lib/types/vip3';

  export let collection: CollectionResponse;
  export let onClose: () => void = () => {};

  let files: Vip3SearchResponse | null = null;
  let loading = true;
  let error: string | null = null;
  let page = 1;
  const pageSize = 50;
  let draggedFileId: number | null = null;

  async function loadFiles() {
    try {
      loading = true;
      error = null;
      files = await Vip3BrowserApi.getCollectionFiles(collection.id, page, pageSize);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load collection files';
      console.error('Error loading collection files:', e);
    } finally {
      loading = false;
    }
  }

  async function handleRemoveFile(fileId: number) {
    if (!confirm('Remove this file from the collection?')) {
      return;
    }

    try {
      await Vip3BrowserApi.removeFileFromCollection(collection.id, fileId);
      await loadFiles(); // Reload
    } catch (e) {
      console.error('Error removing file:', e);
      alert(`Failed to remove file: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  async function handleClearAll() {
    if (!confirm(`Remove all ${files?.total_count || 0} files from this collection?`)) {
      return;
    }

    try {
      await Vip3BrowserApi.clearCollection(collection.id);
      await loadFiles(); // Reload
    } catch (e) {
      console.error('Error clearing collection:', e);
      alert(`Failed to clear collection: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  // Drag handlers for dragging files to DAW
  function handleDragStart(file: any, event: DragEvent) {
    if (!event.dataTransfer) { return; }

    draggedFileId = file.id;

    const dragData = {
      type: 'midi-file',
      id: file.id,
      filename: file.filename,
      bpm: file.bpm,
      key_signature: file.key_signature,
      duration_seconds: file.duration_seconds,
      source: 'collection'
    };

    event.dataTransfer.setData('application/json', JSON.stringify(dragData));
    event.dataTransfer.effectAllowed = 'copy';

    // Create custom drag image
    const dragImage = document.createElement('div');
    dragImage.className = 'drag-ghost';
    dragImage.style.cssText = `
      position: absolute;
      top: -1000px;
      left: -1000px;
      padding: 8px 12px;
      background: var(--color-primary, #007aff);
      color: white;
      border-radius: 6px;
      font-size: 12px;
      font-weight: 500;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      pointer-events: none;
      white-space: nowrap;
    `;
    dragImage.textContent = `📄 ${file.filename}`;
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 20, 20);

    // Cleanup drag image after a short delay
    setTimeout(() => {
      document.body.removeChild(dragImage);
    }, 0);
  }

  function handleDragEnd() {
    draggedFileId = null;
  }

  onMount(() => {
    loadFiles();
  });

  $: if (collection) {
    loadFiles();
  }
</script>

<div class="collection-viewer">
  <div class="header">
    <div class="title-section">
      {#if collection.icon}
        <span class="icon">{collection.icon}</span>
      {/if}
      <h3>{collection.name}</h3>
      {#if collection.is_smart}
        <span class="smart-badge" title="Smart Collection">⚡ Smart</span>
      {/if}
    </div>
    <div class="actions">
      {#if files && files.total_count > 0}
        <button class="btn-clear" on:click={handleClearAll}>
          Clear All
        </button>
      {/if}
      <button class="btn-close" on:click={onClose}>×</button>
    </div>
  </div>

  {#if collection.description}
    <div class="description">{collection.description}</div>
  {/if}

  <div class="content">
    {#if loading}
      <div class="loading">Loading files...</div>
    {:else if error}
      <div class="error">
        <p>{error}</p>
        <button on:click={loadFiles}>Retry</button>
      </div>
    {:else if !files || files.files.length === 0}
      <div class="empty">
        <p>No files in this collection</p>
        <p class="hint">Drag files here or use the + button to add files</p>
      </div>
    {:else}
      <div class="file-list">
        <div class="file-count-header">
          {files.total_count} files
          {#if files.total_pages > 1}
            (Page {page} of {files.total_pages})
          {/if}
        </div>
        {#each files.files as file (file.id)}
          <div
            class="file-item"
            class:dragging={draggedFileId === file.id}
            draggable="true"
            on:dragstart={(e) => handleDragStart(file, e)}
            on:dragend={handleDragEnd}
            role="listitem"
            tabindex="0"
          >
            <div class="file-info">
              <span class="file-name">{file.filename}</span>
              {#if file.bpm}
                <span class="meta">♩ {file.bpm} BPM</span>
              {/if}
              {#if file.key_signature}
                <span class="meta">🎹 {file.key_signature}</span>
              {/if}
            </div>
            <button
              class="btn-remove"
              on:click={() => handleRemoveFile(file.id)}
              title="Remove from collection"
            >
              ✕
            </button>
          </div>
        {/each}
      </div>

      {#if files && files.total_pages > 1}
        <div class="pagination">
          <button
            on:click={() => page = Math.max(1, page - 1)}
            disabled={page === 1}
          >
            ← Prev
          </button>
          <span>Page {page} of {files?.total_pages ?? 1}</span>
          <button
            on:click={() => page = Math.min(files?.total_pages ?? 1, page + 1)}
            disabled={page === (files?.total_pages ?? 1)}
          >
            Next →
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .collection-viewer {
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
  }

  .title-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon {
    font-size: 24px;
  }

  h3 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: var(--color-text, #fff);
  }

  .smart-badge {
    padding: 4px 8px;
    background: var(--color-primary, #007aff);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    color: white;
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .btn-clear {
    padding: 6px 12px;
    background: var(--color-error, #ff3b30);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .btn-clear:hover {
    background: #d32f2f;
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 32px;
    color: var(--color-text-secondary, #999);
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close:hover {
    color: var(--color-text, #fff);
  }

  .description {
    padding: 12px 16px;
    background: var(--color-bg-tertiary, #333);
    border-bottom: 1px solid var(--color-border, #3a3a3a);
    font-size: 14px;
    color: var(--color-text-secondary, #999);
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
    transition: background 0.2s, opacity 0.2s;
    cursor: grab;
  }

  .file-item:hover {
    background: var(--color-bg-hover, #3a3a3a);
  }

  .file-item.dragging {
    opacity: 0.4;
    cursor: grabbing;
  }

  .file-info {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
  }

  .file-name {
    font-size: 14px;
    color: var(--color-text, #fff);
  }

  .meta {
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .btn-remove {
    padding: 4px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 16px;
    color: var(--color-text-tertiary, #666);
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .btn-remove:hover {
    opacity: 1;
    color: var(--color-error, #ff3b30);
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
