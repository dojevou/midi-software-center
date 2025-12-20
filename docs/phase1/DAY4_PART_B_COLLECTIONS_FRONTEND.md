# Day 4, Part 4B: Collections Frontend

**Duration:** 2 hours
**Prerequisites:** Day 4 Part 4A completed
**Files to create:** 2

---

## Overview

Backend is complete. Now build the collections UI with:
1. collectionsApi.ts
2. VIP3Collections component with drag-and-drop
3. Add to collection from VIP3 results
4. Reorder files within collections

---

## Step 1: Collections API (20 min)

Create `app/src/lib/api/collectionsApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface Collection {
  id: number;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface CollectionFileWithMetadata {
  id: number;
  collection_id: number;
  file_id: number;
  order_index: number;
  added_at: string;
  file_name: string;
  file_path: string;
  bpm?: number;
  key_signature?: string;
  duration_seconds?: number;
}

export class CollectionsApi {
  static async getAll(): Promise<Collection[]> {
    return await invoke('get_collections');
  }

  static async getById(id: number): Promise<Collection> {
    return await invoke('get_collection', { id });
  }

  static async create(name: string, description?: string): Promise<Collection> {
    return await invoke('create_collection', { name, description });
  }

  static async update(
    id: number,
    name?: string,
    description?: string
  ): Promise<Collection> {
    return await invoke('update_collection', { id, name, description });
  }

  static async delete(id: number): Promise<void> {
    await invoke('delete_collection', { id });
  }

  static async getFiles(collectionId: number): Promise<CollectionFileWithMetadata[]> {
    return await invoke('get_collection_files', { collectionId });
  }

  static async addFile(
    collectionId: number,
    fileId: number,
    orderIndex?: number
  ): Promise<void> {
    await invoke('add_file_to_collection', {
      collectionId,
      fileId,
      orderIndex
    });
  }

  static async removeFile(collectionId: number, fileId: number): Promise<void> {
    await invoke('remove_file_from_collection', { collectionId, fileId });
  }

  static async reorderFiles(collectionId: number, fileIds: number[]): Promise<void> {
    await invoke('reorder_collection_files', { collectionId, fileIds });
  }
}
```

---

## Step 2: VIP3Collections Component (1.5 hours)

Create `app/src/lib/components/VIP3/VIP3Collections.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    CollectionsApi,
    type Collection,
    type CollectionFileWithMetadata
  } from '$lib/api/collectionsApi';

  let collections: Collection[] = [];
  let selectedCollection: Collection | null = null;
  let collectionFiles: CollectionFileWithMetadata[] = [];
  let loading = false;
  let showCreateDialog = false;
  let newName = '';
  let newDescription = '';

  // Drag-and-drop state
  let draggedFileId: number | null = null;
  let draggedIndex: number | null = null;

  onMount(async () => {
    await loadCollections();
  });

  async function loadCollections() {
    loading = true;
    try {
      collections = await CollectionsApi.getAll();
    } catch (error) {
      console.error('Failed to load collections:', error);
    } finally {
      loading = false;
    }
  }

  async function selectCollection(collection: Collection) {
    selectedCollection = collection;
    try {
      collectionFiles = await CollectionsApi.getFiles(collection.id);
    } catch (error) {
      console.error('Failed to load collection files:', error);
    }
  }

  async function createCollection() {
    if (!newName.trim()) return;

    try {
      const created = await CollectionsApi.create(newName, newDescription || undefined);
      collections = [...collections, created];
      newName = '';
      newDescription = '';
      showCreateDialog = false;

      // Select the new collection
      await selectCollection(created);
    } catch (error) {
      alert(`Failed to create collection: ${error}`);
    }
  }

  async function deleteCollection(id: number, name: string) {
    if (!confirm(`Delete collection "${name}"?`)) return;

    try {
      await CollectionsApi.delete(id);
      collections = collections.filter((c) => c.id !== id);

      if (selectedCollection?.id === id) {
        selectedCollection = null;
        collectionFiles = [];
      }
    } catch (error) {
      alert(`Failed to delete collection: ${error}`);
    }
  }

  async function removeFile(fileId: number) {
    if (!selectedCollection) return;

    try {
      await CollectionsApi.removeFile(selectedCollection.id, fileId);
      collectionFiles = collectionFiles.filter((f) => f.file_id !== fileId);
    } catch (error) {
      alert(`Failed to remove file: ${error}`);
    }
  }

  // Drag and drop handlers
  function handleDragStart(event: DragEvent, fileId: number, index: number) {
    draggedFileId = fileId;
    draggedIndex = index;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
  }

  async function handleDrop(event: DragEvent, targetIndex: number) {
    event.preventDefault();

    if (draggedIndex === null || draggedFileId === null || !selectedCollection) {
      return;
    }

    if (draggedIndex === targetIndex) {
      return;
    }

    // Reorder locally first for immediate feedback
    const newOrder = [...collectionFiles];
    const [draggedFile] = newOrder.splice(draggedIndex, 1);
    newOrder.splice(targetIndex, 0, draggedFile);

    // Update order_index
    newOrder.forEach((file, idx) => {
      file.order_index = idx;
    });

    collectionFiles = newOrder;

    // Send to backend
    try {
      const fileIds = newOrder.map((f) => f.file_id);
      await CollectionsApi.reorderFiles(selectedCollection.id, fileIds);
    } catch (error) {
      console.error('Failed to reorder files:', error);
      // Reload on error
      await selectCollection(selectedCollection);
    }

    draggedFileId = null;
    draggedIndex = null;
  }

  function formatDuration(seconds: number | undefined): string {
    if (!seconds) return '--:--';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="collections">
  <div class="sidebar">
    <div class="header">
      <h3>Collections</h3>
      <button class="create-btn" on:click={() => (showCreateDialog = true)}>
        + New
      </button>
    </div>

    {#if loading}
      <p class="loading">Loading...</p>
    {:else if collections.length === 0}
      <p class="empty">No collections yet</p>
    {:else}
      <div class="collection-list">
        {#each collections as collection (collection.id)}
          <div
            class="collection-item"
            class:active={selectedCollection?.id === collection.id}
          >
            <button
              class="collection-name"
              on:click={() => selectCollection(collection)}
            >
              <span class="name">{collection.name}</span>
              {#if collection.description}
                <span class="desc">{collection.description}</span>
              {/if}
            </button>
            <button
              class="delete-btn"
              on:click|stopPropagation={() => deleteCollection(collection.id, collection.name)}
            >
              ×
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="content">
    {#if selectedCollection}
      <div class="content-header">
        <h2>{selectedCollection.name}</h2>
        <span class="file-count">{collectionFiles.length} files</span>
      </div>

      {#if collectionFiles.length === 0}
        <p class="empty-collection">
          No files in this collection yet.<br />
          Drag files here from VIP3 browser.
        </p>
      {:else}
        <div class="file-list">
          {#each collectionFiles as file, index (file.file_id)}
            <div
              class="file-item"
              draggable="true"
              on:dragstart={(e) => handleDragStart(e, file.file_id, index)}
              on:dragover={handleDragOver}
              on:drop={(e) => handleDrop(e, index)}
            >
              <span class="drag-handle">☰</span>
              <div class="file-info">
                <span class="file-name">{file.file_name}</span>
                <span class="file-meta">
                  {#if file.bpm}
                    {file.bpm.toFixed(0)} BPM
                  {/if}
                  {#if file.key_signature}
                    · {file.key_signature}
                  {/if}
                  {#if file.duration_seconds}
                    · {formatDuration(file.duration_seconds)}
                  {/if}
                </span>
              </div>
              <button class="remove-btn" on:click={() => removeFile(file.file_id)}>
                Remove
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {:else}
      <div class="empty-state">
        <p>Select a collection to view files</p>
      </div>
    {/if}
  </div>
</div>

<!-- Create Dialog -->
{#if showCreateDialog}
  <div class="dialog-overlay" on:click={() => (showCreateDialog = false)}>
    <div class="dialog" on:click|stopPropagation>
      <h3>New Collection</h3>
      <label>
        Name *
        <input type="text" bind:value={newName} placeholder="e.g., Favorite Loops" />
      </label>
      <label>
        Description
        <textarea bind:value={newDescription} placeholder="Optional..." rows="3" />
      </label>
      <div class="dialog-actions">
        <button on:click={() => (showCreateDialog = false)}>Cancel</button>
        <button class="primary" on:click={createCollection}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .collections {
    display: flex;
    height: 100%;
    background: #1a1a1a;
  }

  .sidebar {
    width: 280px;
    border-right: 1px solid #333;
    display: flex;
    flex-direction: column;
  }

  .header {
    padding: 16px;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .header h3 {
    margin: 0;
    color: #fff;
    font-size: 16px;
  }

  .create-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .create-btn:hover {
    background: #2563eb;
  }

  .collection-list {
    overflow-y: auto;
    padding: 8px;
  }

  .collection-item {
    display: flex;
    margin-bottom: 4px;
    background: #252525;
    border-radius: 4px;
    overflow: hidden;
  }

  .collection-item.active {
    background: #3b82f6;
  }

  .collection-name {
    flex: 1;
    padding: 10px 12px;
    background: none;
    border: none;
    color: #ccc;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .collection-item.active .collection-name {
    color: #fff;
  }

  .collection-name:hover {
    opacity: 0.8;
  }

  .name {
    font-weight: 500;
  }

  .desc {
    font-size: 12px;
    opacity: 0.7;
  }

  .delete-btn {
    padding: 0 12px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 20px;
  }

  .delete-btn:hover {
    background: #991b1b;
    color: #fff;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .content-header {
    padding: 16px 24px;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .content-header h2 {
    margin: 0;
    color: #fff;
    font-size: 18px;
  }

  .file-count {
    color: #999;
    font-size: 14px;
  }

  .file-list {
    padding: 16px;
    overflow-y: auto;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 12px;
    margin-bottom: 8px;
    background: #252525;
    border-radius: 4px;
    cursor: grab;
  }

  .file-item:active {
    cursor: grabbing;
  }

  .drag-handle {
    margin-right: 12px;
    color: #666;
    cursor: grab;
  }

  .file-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-name {
    color: #fff;
    font-weight: 500;
  }

  .file-meta {
    color: #999;
    font-size: 12px;
  }

  .remove-btn {
    padding: 6px 12px;
    background: #444;
    color: #ccc;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .remove-btn:hover {
    background: #991b1b;
    color: #fff;
  }

  .empty-state,
  .empty-collection {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: #666;
    text-align: center;
  }

  .loading,
  .empty {
    padding: 24px;
    text-align: center;
    color: #666;
  }

  /* Dialog styles (reuse from saved searches) */
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #2a2a2a;
    padding: 24px;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
    color: #fff;
  }

  .dialog label {
    display: block;
    margin-bottom: 16px;
    color: #ccc;
  }

  .dialog input,
  .dialog textarea {
    width: 100%;
    margin-top: 6px;
    padding: 8px 12px;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #fff;
    font-family: inherit;
  }

  .dialog-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .dialog-actions button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .dialog-actions button.primary {
    background: #3b82f6;
    color: #fff;
  }
</style>
```

---

## Verification (10 min)

Test the full workflow:

```bash
make dev
```

1. **Create Collection:**
   - Click "+ New"
   - Enter name "Test Collection"
   - Verify appears in sidebar

2. **Add Files:** (Manual for now, drag-drop in Phase 2)
   ```javascript
   await window.__TAURI__.invoke('add_file_to_collection', {
     collectionId: 1,
     fileId: 5
   });
   ```

3. **Reorder Files:**
   - Drag a file up/down
   - Verify order persists (reload page)

4. **Remove File:**
   - Click "Remove" button
   - Verify file removed

---

## What's Next?

✅ **Day 4 Complete! Collections system implemented:**
- ✅ Part 4A: Backend models, repository, ordering
- ✅ Part 4B: Frontend UI with drag-and-drop

**Next:** [Day 5, Part 5A: Favorites System](./DAY5_PART_A_FAVORITES.md)
