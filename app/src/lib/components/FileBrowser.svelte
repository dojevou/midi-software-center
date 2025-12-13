<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  interface FileItem {
    id: number;
    name: string;
    path: string;
    type: 'file' | 'folder';
    size?: number;
    modified?: string;
    duration?: number;
    bpm?: number;
    key?: string;
    tags?: string[];
  }

  export let files: FileItem[] = [];
  export let selectedFiles: number[] = [];
  export let viewMode: 'list' | 'grid' = 'list';
  export let showDetails: boolean = true;
  export let sortBy: string = 'name';
  export let sortDesc: boolean = false;
  export let filter: string = '';

  const dispatch = createEventDispatcher<{
    fileSelect: { fileId: number; ctrlKey: boolean; shiftKey: boolean };
    fileOpen: { fileId: number };
    fileContextMenu: { fileId: number; x: number; y: number };
    folderOpen: { folderId: number };
  }>();

  let lastSelectedIndex: number = -1;

  $: filteredFiles = files.filter((file) => file.name.toLowerCase().includes(filter.toLowerCase()));

  $: sortedFiles = [...filteredFiles].sort((a, b) => {
    // Folders first
    if (a.type !== b.type) {
      return a.type === 'folder' ? -1 : 1;
    }

    let comparison = 0;
    switch (sortBy) {
      case 'name':
        comparison = a.name.localeCompare(b.name);
        break;
      case 'size':
        comparison = (a.size || 0) - (b.size || 0);
        break;
      case 'modified':
        comparison = (a.modified || '').localeCompare(b.modified || '');
        break;
      case 'duration':
        comparison = (a.duration || 0) - (b.duration || 0);
        break;
      case 'bpm':
        comparison = (a.bpm || 0) - (b.bpm || 0);
        break;
      default:
        comparison = 0;
    }

    return sortDesc ? -comparison : comparison;
  });

  function handleFileClick(event: MouseEvent, file: FileItem, index: number) {
    if (event.shiftKey && lastSelectedIndex >= 0) {
      // Range selection
      const start = Math.min(lastSelectedIndex, index);
      const end = Math.max(lastSelectedIndex, index);
      const rangeIds = sortedFiles.slice(start, end + 1).map((f) => f.id);

      if (event.ctrlKey) {
        // Add range to selection
        selectedFiles = [...new Set([...selectedFiles, ...rangeIds])];
      } else {
        // Replace with range
        selectedFiles = rangeIds;
      }
    } else {
      dispatch('fileSelect', {
        fileId: file.id,
        ctrlKey: event.ctrlKey,
        shiftKey: event.shiftKey,
      });
      lastSelectedIndex = index;
    }
  }

  function handleFileDoubleClick(file: FileItem) {
    if (file.type === 'folder') {
      dispatch('folderOpen', { folderId: file.id });
    } else {
      dispatch('fileOpen', { fileId: file.id });
    }
  }

  function handleContextMenu(event: MouseEvent, file: FileItem) {
    event.preventDefault();
    dispatch('fileContextMenu', {
      fileId: file.id,
      x: event.clientX,
      y: event.clientY,
    });
  }

  // Drag state for visual feedback
  let draggedFileId: number | null = null;

  function handleDragStart(file: FileItem, event: DragEvent) {
    if (file.type === 'folder' || !event.dataTransfer) return;

    draggedFileId = file.id;

    // Set drag data in the format expected by Sequencer.svelte
    const dragData = {
      type: 'midi-file',
      id: file.id,
      filename: file.name,
      bpm: file.bpm,
      key_signature: file.key,
      duration_ticks: file.duration ? file.duration * 480 : undefined // Convert to ticks
    };

    event.dataTransfer.setData('application/json', JSON.stringify(dragData));
    event.dataTransfer.effectAllowed = 'copy';

    // Create custom drag image using safe DOM methods
    const dragImage = document.createElement('div');
    dragImage.style.cssText = `
      position: absolute;
      top: -1000px;
      left: -1000px;
      background: #2563eb;
      color: white;
      padding: 8px 12px;
      border-radius: 6px;
      font-size: 12px;
      display: flex;
      align-items: center;
      gap: 8px;
      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
      max-width: 250px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    `;

    const iconSpan = document.createElement('span');
    iconSpan.textContent = '🎵';

    const textSpan = document.createElement('span');
    textSpan.textContent = file.name;

    dragImage.appendChild(iconSpan);
    dragImage.appendChild(textSpan);
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 20, 20);

    setTimeout(() => {
      document.body.removeChild(dragImage);
    }, 0);
  }

  function handleDragEnd() {
    draggedFileId = null;
  }

  function formatSize(bytes?: number): string {
    if (!bytes) {
      return '-';
    }
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatDuration(seconds?: number): string {
    if (!seconds) {
      return '-';
    }
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function isSelected(fileId: number): boolean {
    return selectedFiles.includes(fileId);
  }

  function getFileIcon(file: FileItem): string {
    if (file.type === 'folder') {
      return '📁';
    }
    if (file.name.endsWith('.mid') || file.name.endsWith('.midi')) {
      return '🎵';
    }
    if (file.name.endsWith('.wav') || file.name.endsWith('.mp3')) {
      return '🔊';
    }
    return '📄';
  }
</script>

<div class="file-browser h-full flex flex-col dark:bg-window">
  <!-- Toolbar -->
  <div class="toolbar p-2 border-b dark:border-window-border flex items-center gap-2">
    <input
      type="text"
      bind:value={filter}
      placeholder="Filter files..."
      class="flex-1 px-3 py-1 text-sm dark:bg-input dark:border-window-border rounded"
    />

    <div class="view-toggle flex gap-1">
      <button
        class="p-1 rounded"
        class:dark:bg-primary={viewMode === 'list'}
        class:dark:bg-secondary={viewMode !== 'list'}
        on:click={() => (viewMode = 'list')}
        title="List view"
      >
        ☰
      </button>
      <button
        class="p-1 rounded"
        class:dark:bg-primary={viewMode === 'grid'}
        class:dark:bg-secondary={viewMode !== 'grid'}
        on:click={() => (viewMode = 'grid')}
        title="Grid view"
      >
        ⊞
      </button>
    </div>
  </div>

  <!-- File List -->
  <div class="file-list flex-1 overflow-auto">
    {#if viewMode === 'list'}
      <table class="w-full text-sm">
        <thead class="sticky top-0 dark:bg-window-subtle">
          <tr class="border-b dark:border-window-border">
            <th
              class="p-2 text-left cursor-pointer hover:dark:bg-menu"
              on:click={() => {
                sortBy = 'name';
                sortDesc = sortBy === 'name' && !sortDesc;
              }}
            >
              Name {sortBy === 'name' ? (sortDesc ? '▼' : '▲') : ''}
            </th>
            {#if showDetails}
              <th
                class="p-2 text-left cursor-pointer hover:dark:bg-menu w-24"
                on:click={() => {
                  sortBy = 'size';
                  sortDesc = sortBy === 'size' && !sortDesc;
                }}
              >
                Size {sortBy === 'size' ? (sortDesc ? '▼' : '▲') : ''}
              </th>
              <th
                class="p-2 text-left cursor-pointer hover:dark:bg-menu w-20"
                on:click={() => {
                  sortBy = 'duration';
                  sortDesc = sortBy === 'duration' && !sortDesc;
                }}
              >
                Duration {sortBy === 'duration' ? (sortDesc ? '▼' : '▲') : ''}
              </th>
              <th
                class="p-2 text-left cursor-pointer hover:dark:bg-menu w-16"
                on:click={() => {
                  sortBy = 'bpm';
                  sortDesc = sortBy === 'bpm' && !sortDesc;
                }}
              >
                BPM {sortBy === 'bpm' ? (sortDesc ? '▼' : '▲') : ''}
              </th>
              <th class="p-2 text-left w-12">Key</th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each sortedFiles as file, index (file.id)}
            <tr
              class="border-b dark:border-window-border cursor-pointer transition-colors"
              class:dark:bg-primary={isSelected(file.id)}
              class:dark:bg-opacity-30={isSelected(file.id)}
              class:hover:dark:bg-menu={!isSelected(file.id)}
              class:dragging={draggedFileId === file.id}
              draggable={file.type === 'file'}
              on:click={(e) => handleFileClick(e, file, index)}
              on:dblclick={() => handleFileDoubleClick(file)}
              on:contextmenu={(e) => handleContextMenu(e, file)}
              on:dragstart={(e) => handleDragStart(file, e)}
              on:dragend={handleDragEnd}
            >
              <td class="p-2">
                <div class="flex items-center gap-2">
                  <span>{getFileIcon(file)}</span>
                  <span class="dark:text-gray-200">{file.name}</span>
                </div>
              </td>
              {#if showDetails}
                <td class="p-2 dark:text-gray-400">{formatSize(file.size)}</td>
                <td class="p-2 dark:text-gray-400">{formatDuration(file.duration)}</td>
                <td class="p-2 dark:text-gray-400">{file.bpm || '-'}</td>
                <td class="p-2 dark:text-gray-400">{file.key || '-'}</td>
              {/if}
            </tr>
          {:else}
            <tr>
              <td colspan={showDetails ? 5 : 1} class="p-8 text-center dark:text-gray-500">
                No files found
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <!-- Grid View -->
      <div class="grid grid-cols-4 gap-4 p-4">
        {#each sortedFiles as file, index (file.id)}
          <div
            class="file-card p-4 rounded-lg border cursor-pointer transition-colors"
            class:dark:bg-primary={isSelected(file.id)}
            class:dark:bg-opacity-30={isSelected(file.id)}
            class:dark:border-primary={isSelected(file.id)}
            class:dark:bg-window-subtle={!isSelected(file.id)}
            class:dark:border-window-border={!isSelected(file.id)}
            class:dragging={draggedFileId === file.id}
            draggable={file.type === 'file'}
            on:click={(e) => handleFileClick(e, file, index)}
            on:dblclick={() => handleFileDoubleClick(file)}
            on:contextmenu={(e) => handleContextMenu(e, file)}
            on:dragstart={(e) => handleDragStart(file, e)}
            on:dragend={handleDragEnd}
            role="button"
            tabindex="0"
          >
            <div class="text-4xl text-center mb-2">{getFileIcon(file)}</div>
            <div class="text-sm dark:text-gray-200 text-center truncate" title={file.name}>
              {file.name}
            </div>
            {#if showDetails && file.type === 'file'}
              <div class="text-xs dark:text-gray-400 text-center mt-1">
                {file.bpm ? `${file.bpm} BPM` : ''}
                {file.key || ''}
              </div>
            {/if}
          </div>
        {:else}
          <div class="col-span-4 text-center py-12 dark:text-gray-500">No files found</div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Status Bar -->
  <div
    class="status-bar p-2 border-t dark:border-window-border text-xs dark:text-gray-400 flex justify-between"
  >
    <span>{sortedFiles.length} items</span>
    <span>{selectedFiles.length} selected</span>
  </div>
</div>

<style>
  .file-card:hover {
    transform: translateY(-2px);
  }

  tr.dragging,
  .file-card.dragging {
    opacity: 0.5;
  }

  tr[draggable="true"],
  .file-card[draggable="true"] {
    cursor: grab;
  }

  tr[draggable="true"]:active,
  .file-card[draggable="true"]:active {
    cursor: grabbing;
  }
</style>
