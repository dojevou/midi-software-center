<script lang="ts">
  /**
   * LoopBrowserWindow - MIDI Loop Browser
   *
   * Task-O-Matic: Search, filter, preview, and drag MIDI loops to DAW timeline.
   */

  import { onMount } from 'svelte';
  import { debounce } from '$lib/utils/debounce';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { playbackStore } from '$lib/stores/playbackStore';

  // ============================================================================
  // Types
  // ============================================================================

  interface MIDILoop {
    id: number;
    name: string;
    filePath: string;
    bpm: number;
    key: string;
    duration: number; // seconds
    sampleRate: number;
    category: string;
    tags: string[];
    favorite: boolean;
    createdAt: Date;
  }

  interface SearchFilters {
    query: string;
    bpmMin: number;
    bpmMax: number;
    durationMin: number; // seconds
    durationMax: number;
    category: string;
    tags: string[];
  }

  // ============================================================================
  // Constants
  // ============================================================================

  const ITEMS_PER_PAGE = 50;
  const CATEGORIES = [
    'All',
    'Drums',
    'Bass',
    'Melody',
    'Chords',
    'Arpeggio',
    'Percussion',
    'FX',
    'Ambient'
  ];

  const ALL_TAGS = [
    'Techno',
    'House',
    'Trance',
    'Dubstep',
    'Hip Hop',
    'Jazz',
    'Classical',
    'Rock',
    'Electronic',
    'Acoustic'
  ];

  // ============================================================================
  // State
  // ============================================================================

  $: position = $playbackStore.position;
  $: isPlaying = $playbackStore.isPlaying;

  let allLoops: MIDILoop[] = [];
  let filteredLoops: MIDILoop[] = [];
  let currentPage = 1;
  let totalPages = 1;

  let filters: SearchFilters = {
    query: '',
    bpmMin: 60,
    bpmMax: 200,
    durationMin: 0,
    durationMax: 60,
    category: 'All',
    tags: []
  };

  let selectedLoop: MIDILoop | null = null;
  let previewPlaying = false;
  let previewLoop = false;
  let zoomLevel = 1.0;

  // Drag and drop
  let isDragging = false;
  let draggedLoop: MIDILoop | null = null;

  // ============================================================================
  // Search and Filter
  // ============================================================================

  function filterLoops(): void {
    let results = [...allLoops];

    // Text search
    if (filters.query.trim()) {
      const query = filters.query.toLowerCase();
      results = results.filter(
        loop =>
          loop.name.toLowerCase().includes(query) ||
          loop.tags.some(tag => tag.toLowerCase().includes(query))
      );
    }

    // BPM range
    results = results.filter(
      loop => loop.bpm >= filters.bpmMin && loop.bpm <= filters.bpmMax
    );

    // Duration range
    results = results.filter(
      loop => loop.duration >= filters.durationMin && loop.duration <= filters.durationMax
    );

    // Category
    if (filters.category !== 'All') {
      results = results.filter(loop => loop.category === filters.category);
    }

    // Tags
    if (filters.tags.length > 0) {
      results = results.filter(loop =>
        filters.tags.some(tag => loop.tags.includes(tag))
      );
    }

    filteredLoops = results;
    totalPages = Math.ceil(filteredLoops.length / ITEMS_PER_PAGE);
    currentPage = 1;
  }

  // Debounced search
  const debouncedFilter = debounce(filterLoops, 300);

  function handleSearchInput(): void {
    debouncedFilter();
  }

  function clearFilters(): void {
    filters = {
      query: '',
      bpmMin: 60,
      bpmMax: 200,
      durationMin: 0,
      durationMax: 60,
      category: 'All',
      tags: []
    };
    filterLoops();
  }

  // ============================================================================
  // Pagination
  // ============================================================================

  function getPaginatedLoops(): MIDILoop[] {
    const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
    const endIndex = startIndex + ITEMS_PER_PAGE;
    return filteredLoops.slice(startIndex, endIndex);
  }

  function goToPage(page: number): void {
    if (page < 1 || page > totalPages) return;
    currentPage = page;
  }

  function nextPage(): void {
    goToPage(currentPage + 1);
  }

  function prevPage(): void {
    goToPage(currentPage - 1);
  }

  // ============================================================================
  // Preview Player
  // ============================================================================

  async function previewLoop_play(loop: MIDILoop): Promise<void> {
    selectedLoop = loop;
    previewPlaying = true;
    // TODO: Integrate with Tauri backend to actually play the loop
    console.log('Playing loop:', loop.name);
  }

  function previewStop(): void {
    previewPlaying = false;
    // TODO: Stop playback
  }

  function togglePreviewLoop(): void {
    previewLoop = !previewLoop;
  }

  // ============================================================================
  // Favorites
  // ============================================================================

  function toggleFavorite(loop: MIDILoop): void {
    allLoops = allLoops.map(l =>
      l.id === loop.id ? { ...l, favorite: !l.favorite } : l
    );
    filterLoops();
  }

  // ============================================================================
  // Drag and Drop
  // ============================================================================

  function handleDragStart(e: DragEvent, loop: MIDILoop): void {
    isDragging = true;
    draggedLoop = loop;

    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'copy';
      e.dataTransfer.setData('application/json', JSON.stringify(loop));
      e.dataTransfer.setData('text/plain', loop.name);
    }
  }

  function handleDragEnd(): void {
    isDragging = false;
    draggedLoop = null;
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatDate(date: Date): string {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    }).format(date);
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    // Load demo loops
    allLoops = Array.from({ length: 150 }, (_, i) => ({
      id: i + 1,
      name: `Loop ${i + 1}`,
      filePath: `/loops/loop_${i + 1}.mid`,
      bpm: 60 + Math.floor(Math.random() * 140),
      key: ['C', 'D', 'E', 'F', 'G', 'A', 'B'][Math.floor(Math.random() * 7)],
      duration: 4 + Math.random() * 12,
      sampleRate: 44100,
      category: CATEGORIES[Math.floor(Math.random() * (CATEGORIES.length - 1)) + 1],
      tags: [ALL_TAGS[Math.floor(Math.random() * ALL_TAGS.length)]],
      favorite: Math.random() > 0.8,
      createdAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000)
    }));

    filterLoops();
  });

  $: paginatedLoops = getPaginatedLoops();
  $: resultCount = filteredLoops.length;
</script>

<WindowBase windowId="loop-browser" title="Loop Browser">
  <div class="loop-browser-window">
    <!-- Search Bar -->
    <div class="search-bar">
      <input
        type="text"
        placeholder="Search loops by name or tags..."
        bind:value={filters.query}
        on:input={handleSearchInput}
        class="search-input"
      />
      <button class="clear-btn" on:click={clearFilters} title="Clear all filters">
        Clear Filters
      </button>
    </div>

    <!-- Filters Section -->
    <div class="filters-section">
      <div class="filter-group">
        <label class="filter-label">
          BPM Range:
          <div class="range-inputs">
            <input
              type="number"
              min="20"
              max="300"
              bind:value={filters.bpmMin}
              on:change={filterLoops}
              class="range-input"
            />
            <span class="range-separator">-</span>
            <input
              type="number"
              min="20"
              max="300"
              bind:value={filters.bpmMax}
              on:change={filterLoops}
              class="range-input"
            />
          </div>
        </label>
      </div>

      <div class="filter-group">
        <label class="filter-label">
          Duration (s):
          <div class="range-inputs">
            <input
              type="number"
              min="0"
              max="120"
              bind:value={filters.durationMin}
              on:change={filterLoops}
              class="range-input"
            />
            <span class="range-separator">-</span>
            <input
              type="number"
              min="0"
              max="120"
              bind:value={filters.durationMax}
              on:change={filterLoops}
              class="range-input"
            />
          </div>
        </label>
      </div>

      <div class="filter-group">
        <label class="filter-label">
          Category:
          <select
            bind:value={filters.category}
            on:change={filterLoops}
            class="category-select"
          >
            {#each CATEGORIES as category}
              <option value={category}>{category}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="filter-group tags-filter">
        <span class="filter-label">Tags:</span>
        <div class="tags-container">
          {#each ALL_TAGS as tag}
            <button
              class="tag-chip"
              class:active={filters.tags.includes(tag)}
              on:click={() => {
                if (filters.tags.includes(tag)) {
                  filters.tags = filters.tags.filter(t => t !== tag);
                } else {
                  filters.tags = [...filters.tags, tag];
                }
                filterLoops();
              }}
            >
              {tag}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Results Header -->
    <div class="results-header">
      <span class="results-count">
        Showing {(currentPage - 1) * ITEMS_PER_PAGE + 1}-{Math.min(currentPage * ITEMS_PER_PAGE, resultCount)} of {resultCount} results
      </span>

      <div class="view-controls">
        <label class="zoom-label">
          Zoom:
          <input
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            bind:value={zoomLevel}
            class="zoom-slider"
          />
          <span class="zoom-value">{zoomLevel.toFixed(1)}x</span>
        </label>
      </div>
    </div>

    <!-- Results List -->
    <div class="results-container">
      {#each paginatedLoops as loop (loop.id)}
        <div
          class="loop-item"
          class:selected={selectedLoop?.id === loop.id}
          draggable="true"
          on:dragstart={(e) => handleDragStart(e, loop)}
          on:dragend={handleDragEnd}
          style="transform: scale({zoomLevel}); transform-origin: left top"
        >
          <!-- Favorite Star -->
          <button
            class="favorite-btn"
            class:active={loop.favorite}
            on:click={() => toggleFavorite(loop)}
            title={loop.favorite ? 'Remove from favorites' : 'Add to favorites'}
          >
            ★
          </button>

          <!-- Loop Info -->
          <div class="loop-info">
            <div class="loop-name">{loop.name}</div>
            <div class="loop-metadata">
              <span class="meta-item">{loop.bpm} BPM</span>
              <span class="meta-separator">•</span>
              <span class="meta-item">{loop.key} Key</span>
              <span class="meta-separator">•</span>
              <span class="meta-item">{formatDuration(loop.duration)}</span>
              <span class="meta-separator">•</span>
              <span class="meta-item">{loop.category}</span>
            </div>
            <div class="loop-tags">
              {#each loop.tags as tag}
                <span class="tag-badge">{tag}</span>
              {/each}
            </div>
          </div>

          <!-- Actions -->
          <div class="loop-actions">
            <button
              class="action-btn"
              on:click={() => previewLoop_play(loop)}
              disabled={previewPlaying && selectedLoop?.id === loop.id}
              title="Preview loop"
            >
              {previewPlaying && selectedLoop?.id === loop.id ? '⏸' : '▶'}
            </button>
            <span class="drag-hint">Drag to DAW</span>
          </div>
        </div>
      {/each}

      {#if paginatedLoops.length === 0}
        <div class="no-results">
          <p>No loops found</p>
          <p class="hint">Try adjusting your search filters</p>
        </div>
      {/if}
    </div>

    <!-- Preview Player -->
    {#if selectedLoop}
      <div class="preview-player">
        <div class="preview-info">
          <span class="preview-name">{selectedLoop.name}</span>
          <span class="preview-meta">{selectedLoop.bpm} BPM • {selectedLoop.key} • {formatDuration(selectedLoop.duration)}</span>
        </div>

        <div class="preview-controls">
          <button
            class="preview-btn"
            on:click={() => selectedLoop && previewLoop_play(selectedLoop)}
            disabled={!selectedLoop}
          >
            ▶ Play
          </button>
          <button class="preview-btn" on:click={previewStop} disabled={!previewPlaying}>
            ⏹ Stop
          </button>
          <button
            class="preview-btn"
            class:active={previewLoop}
            on:click={togglePreviewLoop}
          >
            🔁 Loop
          </button>
        </div>

        <!-- Waveform Placeholder -->
        <div class="waveform">
          <div class="waveform-placeholder">Waveform visualization</div>
        </div>
      </div>
    {/if}

    <!-- Pagination -->
    <div class="pagination">
      <button
        class="page-btn"
        on:click={prevPage}
        disabled={currentPage === 1}
      >
        ← Prev
      </button>

      <div class="page-numbers">
        {#each Array(Math.min(totalPages, 5)) as _, i}
          {@const pageNum = Math.max(1, currentPage - 2) + i}
          {#if pageNum <= totalPages}
            <button
              class="page-num"
              class:active={pageNum === currentPage}
              on:click={() => goToPage(pageNum)}
            >
              {pageNum}
            </button>
          {/if}
        {/each}

        {#if totalPages > 5 && currentPage < totalPages - 2}
          <span class="page-ellipsis">...</span>
          <button class="page-num" on:click={() => goToPage(totalPages)}>
            {totalPages}
          </button>
        {/if}
      </div>

      <button
        class="page-btn"
        on:click={nextPage}
        disabled={currentPage === totalPages}
      >
        Next →
      </button>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">Page {currentPage} of {totalPages}</span>
      <span class="status-item">{resultCount} loops</span>
      <span class="status-item">Category: {filters.category}</span>
      {#if filters.tags.length > 0}
        <span class="status-item">Tags: {filters.tags.join(', ')}</span>
      {/if}
    </div>
  </div>
</WindowBase>

<style>
  .loop-browser-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Search Bar */
  .search-bar {
    display: flex;
    gap: 8px;
    padding: 12px;
    background: var(--search-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .search-input {
    flex: 1;
    padding: 8px 12px;
    background: var(--input-bg, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    font-size: 14px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .clear-btn {
    padding: 8px 16px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .clear-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  /* Filters Section */
  .filters-section {
    display: flex;
    gap: 16px;
    padding: 12px;
    background: var(--filters-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    flex-wrap: wrap;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-group.tags-filter {
    flex: 1 0 100%;
    flex-direction: column;
    align-items: flex-start;
  }

  .filter-label {
    font-size: 12px;
    color: var(--text-secondary, #888);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .range-input {
    width: 60px;
    padding: 6px 8px;
    background: var(--input-bg, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
  }

  .range-separator {
    color: var(--text-secondary, #888);
  }

  .category-select {
    padding: 6px 12px;
    background: var(--select-bg, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .tags-container {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .tag-chip {
    padding: 4px 12px;
    background: var(--chip-bg, #2a2a2a);
    color: var(--chip-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 16px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s ease;
  }

  .tag-chip:hover {
    background: var(--chip-hover, #3a3a3a);
  }

  .tag-chip.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  /* Results Header */
  .results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .results-count {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
  }

  .view-controls {
    display: flex;
    gap: 12px;
  }

  .zoom-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .zoom-slider {
    width: 80px;
    height: 4px;
    -webkit-appearance: none;
    background: var(--slider-bg, #3a3a3a);
    border-radius: 2px;
  }

  .zoom-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
  }

  .zoom-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .zoom-value {
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    min-width: 35px;
  }

  /* Results Container */
  .results-container {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .loop-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    margin-bottom: 8px;
    background: var(--item-bg, #252525);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    cursor: move;
    transition: all 0.15s ease;
  }

  .loop-item:hover {
    background: var(--item-hover, #2a2a2a);
    border-color: var(--accent-color, #0078d4);
  }

  .loop-item.selected {
    background: var(--item-selected, #3a3a3a);
    border-color: var(--accent-color, #0078d4);
  }

  .favorite-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    color: var(--star-inactive, #666);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 20px;
    transition: all 0.15s ease;
  }

  .favorite-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--star-hover, #f7dc6f);
  }

  .favorite-btn.active {
    color: var(--star-active, #f7dc6f);
  }

  .loop-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .loop-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .loop-metadata {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .meta-item {
    font-variant-numeric: tabular-nums;
  }

  .meta-separator {
    opacity: 0.5;
  }

  .loop-tags {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tag-badge {
    padding: 2px 8px;
    background: var(--badge-bg, #3a3a3a);
    color: var(--badge-text, #e0e0e0);
    border-radius: 12px;
    font-size: 10px;
  }

  .loop-actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .action-btn {
    width: 40px;
    height: 40px;
    background: var(--action-bg, #0078d4);
    color: white;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    font-size: 18px;
    transition: all 0.15s ease;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--action-hover, #0084e8);
    transform: scale(1.1);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .drag-hint {
    font-size: 10px;
    color: var(--text-tertiary, #666);
  }

  .no-results {
    padding: 48px 24px;
    text-align: center;
    color: var(--text-secondary, #888);
  }

  .no-results p {
    margin: 0 0 8px 0;
  }

  .hint {
    font-size: 13px;
    color: var(--text-tertiary, #666);
  }

  /* Preview Player */
  .preview-player {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    background: var(--preview-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .preview-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .preview-meta {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .preview-controls {
    display: flex;
    gap: 8px;
  }

  .preview-btn {
    padding: 8px 16px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .preview-btn:hover:not(:disabled) {
    background: var(--button-hover, #4a4a4a);
  }

  .preview-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .waveform {
    height: 60px;
    background: var(--waveform-bg, #1a1a1a);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .waveform-placeholder {
    font-size: 12px;
    color: var(--text-tertiary, #666);
  }

  /* Pagination */
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px;
    background: var(--pagination-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .page-btn,
  .page-num {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .page-btn:hover:not(:disabled),
  .page-num:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-num.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .page-numbers {
    display: flex;
    gap: 4px;
  }

  .page-ellipsis {
    padding: 6px;
    color: var(--text-secondary, #888);
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 12px;
    background: var(--status-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .status-item {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Scrollbar */
  .results-container::-webkit-scrollbar {
    width: 12px;
  }

  .results-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .results-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
