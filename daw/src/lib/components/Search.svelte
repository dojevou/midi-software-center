<script lang="ts">
  import { api } from '../api';
  import { searchQuery, searchFilters, searchResults, searchTotal, searchLoading } from '../stores';

  let bpmMin = 40;
  let bpmMax = 200;
  let selectedKey = '';
  let selectedTimeSignature = '';
  let selectedCategories: string[] = [];
  let hasDrums: boolean | null = null;

  let debounceTimer: number;

  // Perform search
  async function performSearch() {
    $searchLoading = true;
    try {
      const filters = {
        search_text: $searchQuery || undefined,
        min_bpm: bpmMin || undefined,
        max_bpm: bpmMax || undefined,
        key_signature: selectedKey || undefined,
        category: selectedCategories.length > 0 ? selectedCategories.join(',') : undefined,
        limit: 20,
        offset: 0,
      };

      const results = await api.search.files(filters);
      $searchResults = results.files;
      $searchTotal = results.total;
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      $searchLoading = false;
    }
  }

  // Debounced search on query change
  function onQueryChange() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      performSearch();
    }, 300);
  }

  // Toggle category
  function toggleCategory(category: string) {
    if (selectedCategories.includes(category)) {
      selectedCategories = selectedCategories.filter((c) => c !== category);
    } else {
      selectedCategories = [...selectedCategories, category];
    }
    performSearch();
  }

  // Key signatures for filter
  const keySignatures = [
    'C',
    'C#',
    'D',
    'D#',
    'E',
    'F',
    'F#',
    'G',
    'G#',
    'A',
    'A#',
    'B',
    'Cm',
    'C#m',
    'Dm',
    'D#m',
    'Em',
    'Fm',
    'F#m',
    'Gm',
    'G#m',
    'Am',
    'A#m',
    'Bm',
  ];

  // Categories
  const categories = ['loops', 'one-shots', 'full-songs'];

  // Time signatures
  const timeSignatures = ['4/4', '3/4', '6/8', '5/4', '7/8', '2/4', '12/8'];

  // Drag and drop handler for search results
  function handleDragStart(event: DragEvent, file: any) {
    if (!event.dataTransfer) return;

    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData(
      'application/json',
      JSON.stringify({
        type: 'midi-file',
        fileId: file.id,
        filename: file.file_name,
        bpm: file.bpm,
        key: file.key,
        category: file.category,
        duration: file.duration_seconds,
      })
    );

    // Create custom drag image
    const dragImage = document.createElement('div');
    dragImage.style.position = 'absolute';
    dragImage.style.top = '-1000px';
    dragImage.style.padding = '0.75rem 1rem';
    dragImage.style.background = 'rgba(255, 62, 0, 0.9)';
    dragImage.style.color = '#fff';
    dragImage.style.borderRadius = '6px';
    dragImage.style.fontSize = '0.875rem';
    dragImage.style.fontWeight = '600';
    dragImage.style.boxShadow = '0 4px 12px rgba(0, 0, 0, 0.5)';
    dragImage.textContent = `🎵 ${file.file_name}`;
    document.body.appendChild(dragImage);
    event.dataTransfer.setDragImage(dragImage, 0, 0);
    setTimeout(() => dragImage.remove(), 0);
  }
</script>

<div class="search-container">
  <div class="search-bar">
    <input
      type="text"
      bind:value={$searchQuery}
      on:input={onQueryChange}
      placeholder="Search MIDI files..."
      class="search-input"
    />
    <button on:click={performSearch} class="search-btn"> 🔍 Search </button>
  </div>

  <div class="filters">
    <!-- BPM Range Slider -->
    <div class="filter-group bpm-slider-group">
      <label>BPM Range: {bpmMin} - {bpmMax}</label>
      <div class="slider-container">
        <input
          type="range"
          bind:value={bpmMin}
          min="40"
          max="200"
          on:change={performSearch}
          class="slider"
        />
        <input
          type="range"
          bind:value={bpmMax}
          min="40"
          max="200"
          on:change={performSearch}
          class="slider"
        />
      </div>
      <div class="slider-labels">
        <span>40</span>
        <span>200</span>
      </div>
    </div>

    <!-- Key Signature -->
    <div class="filter-group">
      <label>Key Signature</label>
      <select bind:value={selectedKey} on:change={performSearch}>
        <option value="">Any Key</option>
        {#each keySignatures as key}
          <option value={key}>{key}</option>
        {/each}
      </select>
    </div>

    <!-- Time Signature -->
    <div class="filter-group">
      <label>Time Signature</label>
      <select bind:value={selectedTimeSignature} on:change={performSearch}>
        <option value="">Any</option>
        {#each timeSignatures as ts}
          <option value={ts}>{ts}</option>
        {/each}
      </select>
    </div>

    <!-- Category Checkboxes -->
    <div class="filter-group category-group">
      <label>Categories</label>
      <div class="checkbox-list">
        {#each categories as category}
          <label class="checkbox-label">
            <input
              type="checkbox"
              checked={selectedCategories.includes(category)}
              on:change={() => toggleCategory(category)}
            />
            <span>{category}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Drums Filter -->
    <div class="filter-group">
      <label>Drums</label>
      <select bind:value={hasDrums} on:change={performSearch}>
        <option value={null}>Any</option>
        <option value={true}>Has Drums</option>
        <option value={false}>No Drums</option>
      </select>
    </div>

    <!-- Clear Button -->
    <div class="filter-group">
      <button
        on:click={() => {
          $searchQuery = '';
          bpmMin = 40;
          bpmMax = 200;
          selectedKey = '';
          selectedTimeSignature = '';
          selectedCategories = [];
          hasDrums = null;
          performSearch();
        }}
        class="clear-btn"
      >
        Clear All Filters
      </button>
    </div>
  </div>

  <div class="results-info">
    {#if $searchLoading}
      <div class="loading-spinner">
        <div class="spinner"></div>
        <span>Searching...</span>
      </div>
    {:else}
      <p>Showing {$searchResults.length} of {$searchTotal} files</p>
    {/if}
  </div>
</div>

<style>
  .search-container {
    padding: 1.5rem;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
  }

  .search-bar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }

  .search-input {
    flex: 1;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    font-size: 1rem;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: #ff3e00;
    background: rgba(255, 255, 255, 0.08);
  }

  .search-btn {
    padding: 0.75rem 1.5rem;
    background: #ff3e00;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 1rem;
  }

  .search-btn:hover {
    background: #e63900;
    transform: translateY(-2px);
  }

  .filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .filter-group label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 600;
  }

  .filter-group select {
    padding: 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .filter-group select:focus {
    outline: none;
    border-color: #ff3e00;
  }

  /* BPM Slider Styles */
  .bpm-slider-group {
    grid-column: span 2;
  }

  .slider-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .slider {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    outline: none;
    -webkit-appearance: none;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ff3e00;
    cursor: pointer;
    transition: all 0.2s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    background: #ff5722;
  }

  .slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ff3e00;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .slider::-moz-range-thumb:hover {
    transform: scale(1.2);
    background: #ff5722;
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  /* Category Checkboxes */
  .category-group {
    grid-column: span 2;
  }

  .checkbox-list {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.8);
    transition: color 0.2s;
  }

  .checkbox-label:hover {
    color: #fff;
  }

  .checkbox-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: #ff3e00;
  }

  /* Clear Button */
  .clear-btn {
    width: 100%;
    padding: 0.5rem 1rem;
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid rgba(244, 67, 54, 0.3);
    border-radius: 4px;
    color: #f44336;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.875rem;
    font-weight: 600;
    margin-top: auto;
  }

  .clear-btn:hover {
    background: rgba(244, 67, 54, 0.2);
    border-color: #f44336;
  }

  /* Results Info */
  .results-info {
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.875rem;
  }

  .results-info p {
    margin: 0;
    font-weight: 600;
  }

  .loading-spinner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #ff3e00;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Responsive */
  @media (max-width: 768px) {
    .bpm-slider-group,
    .category-group {
      grid-column: span 1;
    }

    .filters {
      grid-template-columns: 1fr;
    }
  }
</style>
