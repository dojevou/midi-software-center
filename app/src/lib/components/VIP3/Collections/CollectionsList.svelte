<script lang="ts">
  import { onMount } from 'svelte';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { CollectionResponse } from '$lib/types/vip3';
  import CreateCollectionDialog from './CreateCollectionDialog.svelte';

  export let onSelectCollection: (collection: CollectionResponse) => void = () => {};

  let collections: CollectionResponse[] = [];
  let loading = true;
  let error: string | null = null;
  let showCreateDialog = false;
  let selectedId: number | null = null;

  async function loadCollections() {
    try {
      loading = true;
      error = null;
      collections = await Vip3BrowserApi.getCollections();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load collections';
      console.error('Error loading collections:', e);
    } finally {
      loading = false;
    }
  }

  async function handleDeleteCollection(collectionId: number) {
    if (!confirm('Are you sure you want to delete this collection? Files will not be deleted.')) {
      return;
    }

    try {
      await Vip3BrowserApi.deleteCollection(collectionId);
      collections = collections.filter(c => c.id !== collectionId);
      if (selectedId === collectionId) {
        selectedId = null;
      }
    } catch (e) {
      console.error('Error deleting collection:', e);
      alert(`Failed to delete collection: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }

  function handleSelectCollection(collection: CollectionResponse) {
    selectedId = collection.id;
    onSelectCollection(collection);
  }

  function handleCreateSuccess() {
    showCreateDialog = false;
    loadCollections();
  }

  onMount(() => {
    loadCollections();
  });
</script>

<div class="collections-list">
  <div class="header">
    <h3>Collections</h3>
    <button class="btn-new" on:click={() => showCreateDialog = true}>
      + New Collection
    </button>
  </div>

  {#if loading}
    <div class="loading">Loading collections...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={loadCollections}>Retry</button>
    </div>
  {:else if collections.length === 0}
    <div class="empty">
      <p>No collections yet</p>
      <p class="hint">Create collections to organize your MIDI files</p>
    </div>
  {:else}
    <div class="collections">
      {#each collections as collection (collection.id)}
        <div
          class="collection-item"
          class:selected={selectedId === collection.id}
          class:smart={collection.is_smart}
          style:border-left-color={collection.color || '#007aff'}
          on:click={() => handleSelectCollection(collection)}
        >
          <div class="content">
            <div class="header-row">
              <div class="title-section">
                {#if collection.icon}
                  <span class="icon">{collection.icon}</span>
                {/if}
                <h4>{collection.name}</h4>
                {#if collection.is_smart}
                  <span class="smart-badge" title="Smart Collection">⚡</span>
                {/if}
              </div>
              <button
                class="btn-delete"
                on:click|stopPropagation={() => handleDeleteCollection(collection.id)}
                title="Delete collection"
              >
                🗑️
              </button>
            </div>

            {#if collection.description}
              <p class="description">{collection.description}</p>
            {/if}

            <div class="file-count">
              📁 {collection.file_count || 0} files
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreateDialog}
  <CreateCollectionDialog
    onClose={() => showCreateDialog = false}
    onCreate={handleCreateSuccess}
  />
{/if}

<style>
  .collections-list {
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

  h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text, #ffffff);
  }

  .btn-new {
    padding: 8px 16px;
    background: var(--color-primary, #007aff);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-new:hover {
    background: var(--color-primary-hover, #0056b3);
  }

  .loading,
  .error,
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
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

  .empty .hint {
    margin-top: 8px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .collections {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .collection-item {
    background: var(--color-bg-tertiary, #333);
    border-radius: 8px;
    border-left: 4px solid;
    margin-bottom: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .collection-item:hover {
    transform: translateX(2px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .collection-item.selected {
    background: var(--color-bg-hover, #3a3a3a);
    box-shadow: 0 0 0 2px var(--color-primary, #007aff);
  }

  .collection-item.smart {
    border-left-style: dashed;
  }

  .content {
    padding: 12px;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 8px;
  }

  .title-section {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .icon {
    font-size: 20px;
  }

  h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text, #fff);
  }

  .smart-badge {
    font-size: 14px;
    opacity: 0.8;
  }

  .btn-delete {
    padding: 4px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 16px;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .btn-delete:hover {
    opacity: 1;
  }

  .description {
    margin: 0 0 8px 0;
    font-size: 13px;
    color: var(--color-text-secondary, #999);
    line-height: 1.4;
  }

  .file-count {
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }
</style>
