<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import type { FileDetails } from '$lib/types';

  export let files: FileDetails[] = [];
  export let selectedIds: number[] = [];
  export let isLoading: boolean = false;
  export let itemHeight: number = 64; // Increased for rating row
  export let overscan: number = 5;
  export let currentPage: number = 1;
  export let totalPages: number = 1;
  export let totalFiles: number = 0;
  export let sortBy: string = 'filename';
  export let sortOrder: 'asc' | 'desc' = 'asc';
  export let pageSize: number = 50;

  // File ratings stored locally (in production would sync with backend)
  let fileRatings: Record<number, number> = {};

  const sortOptions = [
    { value: 'filename', label: 'Name' },
    { value: 'bpm', label: 'BPM' },
    { value: 'key_signature', label: 'Key' },
    { value: 'duration', label: 'Duration' },
    { value: 'created_at', label: 'Date Added' },
    { value: 'rating', label: 'Rating' },
  ];

  const pageSizeOptions = [10, 25, 50, 100, 500];

  const dispatch = createEventDispatcher<{
    select: { file: FileDetails; ctrlKey: boolean; shiftKey: boolean };
    doubleClick: { file: FileDetails };
    contextMenu: { file: FileDetails; x: number; y: number };
    pageChange: { page: number };
    sortChange: { sortBy: string; sortOrder: 'asc' | 'desc' };
    pageSizeChange: { pageSize: number };
    ratingChange: { fileId: number; rating: number };
  }>();

  // Load ratings from localStorage on mount
  onMount(() => {
    const stored = localStorage.getItem('vip3-file-ratings');
    if (stored) {
      try {
        fileRatings = JSON.parse(stored);
      } catch {
        fileRatings = {};
      }
    }
  });

  function setRating(fileId: number, rating: number, event: MouseEvent) {
    event.stopPropagation();
    // Toggle off if clicking same rating
    const currentRating = fileRatings[fileId] || 0;
    const newRating = currentRating === rating ? 0 : rating;
    fileRatings[fileId] = newRating;
    fileRatings = { ...fileRatings }; // Trigger reactivity
    localStorage.setItem('vip3-file-ratings', JSON.stringify(fileRatings));
    dispatch('ratingChange', { fileId, rating: newRating });
  }

  function handleSortChange() {
    dispatch('sortChange', { sortBy, sortOrder });
  }

  function toggleSortOrder() {
    sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
    handleSortChange();
  }

  function handlePageSizeChange() {
    dispatch('pageSizeChange', { pageSize });
  }

  let containerElement: HTMLDivElement;
  let scrollTop = 0;
  let containerHeight = 0;

  // Virtual scrolling calculations
  $: totalHeight = files.length * itemHeight;
  $: startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  $: endIndex = Math.min(
    files.length,
    Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan
  );
  $: visibleFiles = files.slice(startIndex, endIndex);
  $: offsetY = startIndex * itemHeight;

  function handleScroll() {
    if (containerElement) {
      scrollTop = containerElement.scrollTop;
    }
  }

  function handleResize() {
    if (containerElement) {
      containerHeight = containerElement.clientHeight;
    }
  }

  function handleSelect(file: FileDetails, event: MouseEvent) {
    dispatch('select', {
      file,
      ctrlKey: event.ctrlKey || event.metaKey,
      shiftKey: event.shiftKey
    });
  }

  function handleDoubleClick(file: FileDetails) {
    dispatch('doubleClick', { file });
  }

  function handleContextMenu(file: FileDetails, event: MouseEvent) {
    event.preventDefault();
    dispatch('contextMenu', { file, x: event.clientX, y: event.clientY });
  }

  // Drag state for visual feedback
  let draggedFileId: number | null = null;

  function handleDragStart(file: FileDetails, event: DragEvent) {
    if (!event.dataTransfer) return;

    draggedFileId = file.id;

    // Set drag data in the format expected by Sequencer.svelte
    const dragData = {
      type: 'midi-file',
      id: file.id,
      filename: file.filename,
      bpm: file.bpm,
      key_signature: file.key_signature,
      duration_seconds: file.duration_seconds
    };

    event.dataTransfer.setData('application/json', JSON.stringify(dragData));
    event.dataTransfer.effectAllowed = 'copy';

    // Create custom drag image using safe DOM methods
    const dragImage = document.createElement('div');
    dragImage.className = 'drag-ghost';

    const iconSpan = document.createElement('span');
    iconSpan.className = 'drag-icon';
    iconSpan.textContent = '🎵';

    const textSpan = document.createElement('span');
    textSpan.className = 'drag-text';
    textSpan.textContent = file.filename;

    dragImage.appendChild(iconSpan);
    dragImage.appendChild(textSpan);
    dragImage.style.cssText = `
      position: absolute;
      top: -1000px;
      left: -1000px;
      background: var(--primary-600, #2563eb);
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
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 20, 20);

    // Clean up drag image after a short delay
    setTimeout(() => {
      document.body.removeChild(dragImage);
    }, 0);
  }

  function handleDragEnd() {
    draggedFileId = null;
  }

  function handleKeydown(event: KeyboardEvent, file: FileDetails, index: number) {
    switch (event.key) {
      case 'Enter':
      case ' ':
        event.preventDefault();
        dispatch('select', { file, ctrlKey: false, shiftKey: false });
        break;
      case 'ArrowDown':
        event.preventDefault();
        focusItem(startIndex + index + 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        focusItem(startIndex + index - 1);
        break;
    }
  }

  function focusItem(globalIndex: number) {
    if (globalIndex >= 0 && globalIndex < files.length) {
      const itemElement = containerElement?.querySelector(
        `[data-index="${globalIndex}"]`
      ) as HTMLElement;
      if (itemElement) {
        itemElement.focus();
      } else {
        // Scroll to make item visible
        const targetScroll = globalIndex * itemHeight - containerHeight / 2;
        containerElement?.scrollTo({ top: targetScroll, behavior: 'smooth' });
      }
    }
  }

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages && page !== currentPage) {
      dispatch('pageChange', { page });
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) { return `${bytes} B`; }
    if (bytes < 1024 * 1024) { return `${(bytes / 1024).toFixed(1)} KB`; }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDuration(durationSeconds?: number): string {
    if (!durationSeconds) { return '--:--'; }
    const totalSeconds = Math.round(durationSeconds);
    const mins = Math.floor(totalSeconds / 60);
    const secs = totalSeconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  onMount(() => {
    handleResize();
    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
  });
</script>

<div class="vip3-file-list">
  <div class="list-header">
    <span class="header-cell rating">Rating</span>
    <span class="header-cell name">
      Name
      <div class="sort-controls">
        <select bind:value={sortBy} on:change={handleSortChange} class="sort-select">
          {#each sortOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
        <button
          type="button"
          class="sort-order-btn"
          on:click={toggleSortOrder}
          title={sortOrder === 'asc' ? 'Ascending' : 'Descending'}
        >
          {sortOrder === 'asc' ? '↑' : '↓'}
        </button>
      </div>
    </span>
    <span class="header-cell bpm">BPM</span>
    <span class="header-cell key">Key</span>
    <span class="header-cell duration">Duration</span>
    <span class="header-cell size">Size</span>
    <span class="header-cell actions"></span>
  </div>

  <div
    bind:this={containerElement}
    class="list-container"
    on:scroll={handleScroll}
    role="listbox"
    aria-label="File list"
    tabindex="-1"
  >
    {#if isLoading}
      <div class="loading-overlay">
        <span class="loading-spinner">⟳</span>
        <span class="loading-text">Loading files...</span>
      </div>
    {:else if files.length === 0}
      <div class="empty-state">
        <span class="empty-icon">📂</span>
        <span class="empty-text">No files match your filters</span>
      </div>
    {:else}
      <div class="virtual-list" style="height: {totalHeight}px;">
        <div class="virtual-list-inner" style="transform: translateY({offsetY}px);">
          {#each visibleFiles as file, i (file.id)}
            {@const globalIndex = startIndex + i}
            {@const isSelected = selectedIds.includes(file.id)}
            <div
              class="file-item"
              class:selected={isSelected}
              class:dragging={draggedFileId === file.id}
              data-index={globalIndex}
              role="option"
              aria-selected={isSelected}
              tabindex={isSelected ? 0 : -1}
              draggable="true"
              on:click={(e) => handleSelect(file, e)}
              on:dblclick={() => handleDoubleClick(file)}
              on:contextmenu={(e) => handleContextMenu(file, e)}
              on:keydown={(e) => handleKeydown(e, file, i)}
              on:dragstart={(e) => handleDragStart(file, e)}
              on:dragend={handleDragEnd}
              style="height: {itemHeight}px;"
            >
              <!-- Star Rating -->
              <span class="file-cell rating">
                <div class="star-rating" role="group" aria-label="Rating for {file.filename}">
                  {#each [1, 2, 3, 4, 5] as star}
                    <button
                      type="button"
                      class="star-btn"
                      class:filled={(fileRatings[file.id] || 0) >= star}
                      on:click={(e) => setRating(file.id, star, e)}
                      title="Rate {star} star{star > 1 ? 's' : ''}"
                      aria-label="{star} star{star > 1 ? 's' : ''}"
                    >
                      {(fileRatings[file.id] || 0) >= star ? '★' : '☆'}
                    </button>
                  {/each}
                </div>
              </span>
              <span class="file-cell name" title={file.filename}>
                <span class="file-icon">🎵</span>
                <span class="file-name">{file.filename}</span>
              </span>
              <span class="file-cell bpm">
                {file.bpm ? Math.round(file.bpm) : '--'}
              </span>
              <span class="file-cell key">
                {file.key_signature || '--'}
              </span>
              <span class="file-cell duration">
                {formatDuration(file.duration_seconds)}
              </span>
              <span class="file-cell size">
                {formatFileSize(file.file_size_bytes)}
              </span>
              <!-- Context Menu Button -->
              <span class="file-cell actions">
                <button
                  type="button"
                  class="context-menu-btn"
                  on:click={(e) => { e.stopPropagation(); handleContextMenu(file, e); }}
                  title="More actions"
                  aria-label="More actions for {file.filename}"
                >
                  ⋮
                </button>
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <div class="list-footer">
    <div class="file-count">
      {totalFiles.toLocaleString()} file{totalFiles === 1 ? '' : 's'}
      {#if selectedIds.length > 0}
        <span class="selected-count">({selectedIds.length} selected)</span>
      {/if}
    </div>

    <div class="pagination-controls">
      {#if totalPages > 1}
        <div class="pagination">
          <button
            type="button"
            class="page-button"
            disabled={currentPage <= 1}
            on:click={() => goToPage(1)}
            aria-label="First page"
          >
            ⟪
          </button>
          <button
            type="button"
            class="page-button"
            disabled={currentPage <= 1}
            on:click={() => goToPage(currentPage - 1)}
            aria-label="Previous page"
          >
            ◀
          </button>
          <span class="page-info">
            Page {currentPage} of {totalPages}
          </span>
          <button
            type="button"
            class="page-button"
            disabled={currentPage >= totalPages}
            on:click={() => goToPage(currentPage + 1)}
            aria-label="Next page"
          >
            ▶
          </button>
          <button
            type="button"
            class="page-button"
            disabled={currentPage >= totalPages}
            on:click={() => goToPage(totalPages)}
            aria-label="Last page"
          >
            ⟫
          </button>
        </div>
      {/if}

      <div class="page-size-selector">
        <label for="page-size">Show:</label>
        <select
          id="page-size"
          bind:value={pageSize}
          on:change={handlePageSizeChange}
          class="page-size-select"
        >
          {#each pageSizeOptions as size}
            <option value={size}>{size}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>
</div>

<style>
  .vip3-file-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--gray-900, #18181b);
    border-radius: 6px;
    overflow: hidden;
  }

  .list-header {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background: var(--gray-800, #27272a);
    border-bottom: 1px solid var(--gray-700, #3f3f46);
    font-size: 11px;
    font-weight: 600;
    color: var(--gray-400, #a1a1aa);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .header-cell {
    flex-shrink: 0;
  }

  .header-cell.name {
    flex: 1;
    min-width: 0;
  }

  .header-cell.bpm,
  .header-cell.key {
    width: 60px;
    text-align: center;
  }

  .header-cell.duration {
    width: 70px;
    text-align: center;
  }

  .header-cell.size {
    width: 70px;
    text-align: right;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
  }

  .virtual-list {
    position: relative;
    width: 100%;
  }

  .virtual-list-inner {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }

  .file-item {
    display: flex;
    align-items: center;
    padding: 0 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
    border-bottom: 1px solid var(--gray-800, #27272a);
  }

  .file-item:hover {
    background: var(--gray-800, #27272a);
  }

  .file-item:focus {
    outline: none;
    background: var(--gray-800, #27272a);
  }

  .file-item:focus-visible {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }

  .file-item.selected {
    background: var(--primary-600, #2563eb);
    color: white;
  }

  .file-item.selected:hover {
    background: var(--primary-500, #3b82f6);
  }

  .file-item.dragging {
    opacity: 0.5;
    background: var(--gray-700, #3f3f46);
    border: 1px dashed var(--primary-400, #60a5fa);
  }

  .file-item[draggable="true"] {
    cursor: grab;
  }

  .file-item[draggable="true"]:active {
    cursor: grabbing;
  }

  .file-cell {
    flex-shrink: 0;
    font-size: 12px;
  }

  .file-cell.name {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .file-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .file-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--gray-200, #e4e4e7);
  }

  .file-item.selected .file-name {
    color: white;
  }

  .file-cell.bpm,
  .file-cell.key {
    width: 60px;
    text-align: center;
    color: var(--gray-400, #a1a1aa);
  }

  .file-cell.duration {
    width: 70px;
    text-align: center;
    color: var(--gray-400, #a1a1aa);
  }

  .file-cell.size {
    width: 70px;
    text-align: right;
    color: var(--gray-500, #71717a);
  }

  .file-item.selected .file-cell {
    color: rgba(255, 255, 255, 0.8);
  }

  .loading-overlay,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--gray-500, #71717a);
    gap: 12px;
  }

  .loading-spinner {
    font-size: 24px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-text,
  .empty-text {
    font-size: 13px;
  }

  .empty-icon {
    font-size: 32px;
    opacity: 0.5;
  }

  .list-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--gray-800, #27272a);
    border-top: 1px solid var(--gray-700, #3f3f46);
    font-size: 11px;
    color: var(--gray-400, #a1a1aa);
  }

  .file-count {
    display: flex;
    gap: 8px;
  }

  .selected-count {
    color: var(--primary-400, #60a5fa);
  }

  .pagination {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .page-button {
    background: var(--gray-700, #3f3f46);
    border: none;
    color: var(--gray-300, #d4d4d8);
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 10px;
    transition: background-color 0.15s ease;
  }

  .page-button:hover:not(:disabled) {
    background: var(--gray-600, #52525b);
  }

  .page-button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .page-info {
    padding: 0 8px;
    white-space: nowrap;
  }

  /* Custom scrollbar */
  .list-container::-webkit-scrollbar {
    width: 8px;
  }

  .list-container::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .list-container::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 4px;
  }

  .list-container::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }

  /* Header name cell with sort controls */
  .header-cell.name {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .sort-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .sort-select {
    background: var(--gray-700, #3f3f46);
    border: 1px solid var(--gray-600, #52525b);
    color: var(--gray-200, #e4e4e7);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    cursor: pointer;
    text-transform: none;
    font-weight: 400;
  }

  .sort-select:hover {
    background: var(--gray-600, #52525b);
  }

  .sort-select:focus {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }

  .sort-order-btn {
    background: var(--gray-700, #3f3f46);
    border: 1px solid var(--gray-600, #52525b);
    color: var(--gray-200, #e4e4e7);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    cursor: pointer;
    line-height: 1;
  }

  .sort-order-btn:hover {
    background: var(--gray-600, #52525b);
  }

  /* Rating column */
  .header-cell.rating {
    width: 80px;
    text-align: center;
  }

  .file-cell.rating {
    width: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .star-rating {
    display: flex;
    gap: 1px;
  }

  .star-btn {
    background: transparent;
    border: none;
    padding: 0 1px;
    cursor: pointer;
    font-size: 12px;
    color: var(--gray-600, #52525b);
    transition: color 0.1s ease, transform 0.1s ease;
    line-height: 1;
  }

  .star-btn:hover {
    color: var(--yellow-400, #facc15);
    transform: scale(1.1);
  }

  .star-btn.filled {
    color: var(--yellow-500, #eab308);
  }

  .file-item.selected .star-btn {
    color: rgba(255, 255, 255, 0.4);
  }

  .file-item.selected .star-btn.filled {
    color: var(--yellow-400, #facc15);
  }

  /* Actions column */
  .header-cell.actions {
    width: 32px;
  }

  .file-cell.actions {
    width: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .context-menu-btn {
    background: transparent;
    border: none;
    color: var(--gray-500, #71717a);
    padding: 4px 8px;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    border-radius: 4px;
    transition: background-color 0.1s ease, color 0.1s ease;
  }

  .context-menu-btn:hover {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-200, #e4e4e7);
  }

  .file-item.selected .context-menu-btn {
    color: rgba(255, 255, 255, 0.6);
  }

  .file-item.selected .context-menu-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  /* Pagination controls wrapper */
  .pagination-controls {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  /* Page size selector */
  .page-size-selector {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .page-size-selector label {
    color: var(--gray-500, #71717a);
    font-size: 11px;
  }

  .page-size-select {
    background: var(--gray-700, #3f3f46);
    border: 1px solid var(--gray-600, #52525b);
    color: var(--gray-200, #e4e4e7);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .page-size-select:hover {
    background: var(--gray-600, #52525b);
  }

  .page-size-select:focus {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }
</style>
