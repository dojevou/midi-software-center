<script lang="ts">
  import { onMount } from 'svelte';
  import { databaseActions, databaseStore, totalPages } from '$lib/stores/databaseStore';
  import { api } from '$lib/api';
  import type { DatabaseStats, FileDetails, SearchFilters } from '$lib/types';

  console.log('🔧 DatabaseWindow.svelte: VIP3-Style Component loading');

  // ============================================================================
  // STATE
  // ============================================================================

  let searchQuery = '';
  let selectedFile: FileDetails | null = null;
  let isLoading = false;
  let stats: DatabaseStats | null = null;

  // VIP3-style filter categories with multi-select
  let filters = {
    folders: [] as string[],
    instruments: [] as string[],
    timbres: [] as string[],
    styles: [] as string[],
    articulations: [] as string[],
    bpmRange: '' as string,
    key: '' as string,
  };

  // Available options for each category (reserved for dynamic population)
  // These will be populated from database stats in the future
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const availableFolders: { name: string; count: number }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let availableInstruments: { name: string; count: number }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let availableTimbres: { name: string; count: number }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let availableStyles: { name: string; count: number }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let availableArticulations: { name: string; count: number }[] = [];
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let availableKeys: string[] = [];

  // VIP3-style BPM ranges
  const bpmRanges = [
    { label: 'All', value: '' },
    { label: '60-80', value: '60-80', min: 60, max: 80 },
    { label: '80-100', value: '80-100', min: 80, max: 100 },
    { label: '100-120', value: '100-120', min: 100, max: 120 },
    { label: '120-140', value: '120-140', min: 120, max: 140 },
    { label: '140-160', value: '140-160', min: 140, max: 160 },
    { label: '160-180', value: '160-180', min: 160, max: 180 },
    { label: '180+', value: '180+', min: 180, max: 999 },
  ];

  // Collapsed state for each filter column
  const columnCollapsed = {
    folders: false,
    instruments: false,
    timbres: false,
    styles: false,
    articulations: false,
    bpm: false,
    key: false,
  };

  // Reactive stores
  $: searchResults = $databaseStore.searchResults;
  $: currentPage = $databaseStore.currentPage;
  $: isLoading = $databaseStore.isLoading;

  // ============================================================================
  // PREDEFINED FILTER OPTIONS (VIP3-style)
  // ============================================================================

  // Instruments (same as VIP3)
  const defaultInstruments = [
    { name: 'Bass', icon: '🎸' },
    { name: 'Drums', icon: '🥁' },
    { name: 'Keys', icon: '🎹' },
    { name: 'Synth', icon: '🎛️' },
    { name: 'Strings', icon: '🎻' },
    { name: 'Guitar', icon: '🎸' },
    { name: 'Brass', icon: '🎺' },
    { name: 'Vocal', icon: '🎤' },
    { name: 'Pad', icon: '☁️' },
    { name: 'Pluck', icon: '✨' },
    { name: 'FX', icon: '💫' },
    { name: 'Percussion', icon: '🪘' },
  ];

  // Timbres (sound character)
  const defaultTimbres = [
    'Aggressive', 'Airy', 'Bright', 'Clean', 'Dark', 'Dirty',
    'Distorted', 'Fat', 'Gritty', 'Hard', 'Metallic', 'Muted',
    'Punchy', 'Soft', 'Thin', 'Warm', 'Wide', 'Analog', 'Digital',
  ];

  // Styles/Genres
  const defaultStyles = [
    'Ambient', 'Cinematic', 'Dance', 'EDM', 'Funk', 'Hip-Hop',
    'House', 'Jazz', 'Latin', 'Lo-Fi', 'Metal', 'Pop', 'R&B',
    'Reggae', 'Rock', 'Soul', 'Techno', 'Trap', 'Orchestral',
  ];

  // Articulations
  const defaultArticulations = [
    'Arpeggio', 'Chord', 'Fill', 'Loop', 'Melody', 'One-Shot',
    'Phrase', 'Riff', 'Stab', 'Sustain', 'Pattern', 'Break',
  ];

  // Musical keys
  const musicalKeys = [
    'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B',
    'Cm', 'C#m', 'Dm', 'D#m', 'Em', 'Fm', 'F#m', 'Gm', 'G#m', 'Am', 'A#m', 'Bm',
  ];

  // ============================================================================
  // LIFECYCLE
  // ============================================================================

  onMount(async () => {
    console.log('🔧 DatabaseWindow.svelte: VIP3-style onMount started');

    // Initialize available options with defaults
    availableInstruments = defaultInstruments.map(i => ({ name: i.name, count: 0 }));
    availableTimbres = defaultTimbres.map(t => ({ name: t, count: 0 }));
    availableStyles = defaultStyles.map(s => ({ name: s, count: 0 }));
    availableArticulations = defaultArticulations.map(a => ({ name: a, count: 0 }));
    availableKeys = musicalKeys;

    // Load database statistics
    try {
      stats = await api.database.getStats();
      console.log('🔧 DatabaseWindow.svelte: Database stats loaded:', stats);
    } catch (error) {
      console.warn('⚠️ DatabaseWindow.svelte: Failed to load database stats:', error);
    }

    // Initial search
    await performSearch();
  });

  // ============================================================================
  // SEARCH & FILTERS
  // ============================================================================

  async function performSearch() {
    const bpmRange = bpmRanges.find(r => r.value === filters.bpmRange);

    const searchFilters: SearchFilters = {
      search_text: searchQuery || undefined,
      min_bpm: bpmRange?.min,
      max_bpm: bpmRange?.max,
      key_signature: filters.key || undefined,
      instruments: filters.instruments.length > 0 ? filters.instruments : undefined,
      limit: 50,
      offset: 0,
    };

    await databaseActions.search(searchFilters);
  }

  function handleSearchKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      void performSearch();
    }
  }

  function toggleFilter(category: keyof typeof filters, value: string) {
    if (category === 'bpmRange' || category === 'key') {
      // Single select for BPM and Key
      if (filters[category] === value) {
        filters[category] = '';
      } else {
        filters[category] = value;
      }
    } else {
      // Multi-select for other categories
      const arr = filters[category];
      const index = arr.indexOf(value);
      if (index > -1) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (filters as any)[category] = arr.filter(v => v !== value);
      } else {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (filters as any)[category] = [...arr, value];
      }
    }
    void performSearch();
  }

  function clearFilters() {
    filters = {
      folders: [],
      instruments: [],
      timbres: [],
      styles: [],
      articulations: [],
      bpmRange: '',
      key: '',
    };
    searchQuery = '';
    void performSearch();
  }

  function clearCategory(category: keyof typeof filters) {
    if (category === 'bpmRange' || category === 'key') {
      filters[category] = '';
    } else {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (filters as any)[category] = [];
    }
    void performSearch();
  }

  // ============================================================================
  // FILE ACTIONS
  // ============================================================================

  function selectFile(file: FileDetails) {
    selectedFile = file;
  }

  async function handleDoubleClick(file: FileDetails) {
    selectedFile = file;
    try {
      await api.sequencer.addTrack(file.id, 0);
      console.log('Loaded file into DAW:', file.filename);
    } catch (error) {
      console.error('Failed to load file into DAW:', error);
    }
  }

  // Local rating storage key
  const RATINGS_STORAGE_KEY = 'midi-file-ratings';

  function getRatingsFromStorage(): Record<number, number> {
    try {
      const stored = localStorage.getItem(RATINGS_STORAGE_KEY);
      return stored ? JSON.parse(stored) : {};
    } catch {
      return {};
    }
  }

  function saveRatingToStorage(fileId: number, rating: number) {
    const ratings = getRatingsFromStorage();
    ratings[fileId] = rating;
    localStorage.setItem(RATINGS_STORAGE_KEY, JSON.stringify(ratings));
  }

  function getFileRating(fileId: number): number {
    const ratings = getRatingsFromStorage();
    return ratings[fileId] || 0;
  }

  function setRating(file: FileDetails, rating: number) {
    // Store in localStorage (backend rating not yet implemented)
    saveRatingToStorage(file.id, rating);
    // Update local state
    file.rating = rating;
    searchResults = [...searchResults];
    console.log('Set rating:', file.filename, rating);
  }

  async function toggleFavorite(file: FileDetails) {
    try {
      if (file.is_favorite) {
        await api.analysis.removeFavorite(file.id);
      } else {
        await api.analysis.addFavorite(file.id);
      }
      file.is_favorite = !file.is_favorite;
      searchResults = [...searchResults];
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }

  // Drag and drop
  function handleDragStart(event: DragEvent, file: FileDetails) {
    if (!event.dataTransfer) { return; }
    event.dataTransfer.setData('application/json', JSON.stringify({
      type: 'midi-file',
      id: file.id,
      filename: file.filename,
      filepath: file.filepath,
      bpm: file.bpm,
      key_signature: file.key_signature,
    }));
    event.dataTransfer.effectAllowed = 'copy';
  }

  // ============================================================================
  // PAGINATION
  // ============================================================================

  async function nextPage() {
    await databaseActions.nextPage();
  }

  async function previousPage() {
    await databaseActions.previousPage();
  }

  // ============================================================================
  // HELPERS
  // ============================================================================

  function formatBPM(bpm: number | undefined): string {
    return bpm ? bpm.toFixed(1) : '--';
  }

  function formatDuration(seconds: number | undefined): string {
    if (!seconds) { return '--:--'; }
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) { return '0 B'; }
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  }

  function getActiveFilterCount(): number {
    let count = 0;
    if (filters.folders.length) { count += filters.folders.length; }
    if (filters.instruments.length) { count += filters.instruments.length; }
    if (filters.timbres.length) { count += filters.timbres.length; }
    if (filters.styles.length) { count += filters.styles.length; }
    if (filters.articulations.length) { count += filters.articulations.length; }
    if (filters.bpmRange) { count++; }
    if (filters.key) { count++; }
    return count;
  }

  function toggleColumn(col: keyof typeof columnCollapsed) {
    columnCollapsed[col] = !columnCollapsed[col];
  }
</script>

<div class="vip3-browser">
  <!-- ====================================================================== -->
  <!-- TOP BAR: Search + Stats -->
  <!-- ====================================================================== -->
  <div class="top-bar">
    <div class="search-section">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search MIDI files..."
        on:keydown={handleSearchKeydown}
        class="search-input"
        disabled={isLoading}
      />
      <button class="search-btn" on:click={performSearch} disabled={isLoading}>
        Search
      </button>
    </div>

    <div class="stats-section">
      {#if stats}
        <span class="stat">📁 {stats.total_files.toLocaleString()} files</span>
        <span class="stat">💾 {formatBytes(stats.total_size)}</span>
        <span class="stat">🎵 Avg {stats.avg_bpm.toFixed(0)} BPM</span>
      {/if}
    </div>

    {#if getActiveFilterCount() > 0}
      <button class="clear-all-btn" on:click={clearFilters}>
        Clear All ({getActiveFilterCount()})
      </button>
    {/if}
  </div>

  <!-- ====================================================================== -->
  <!-- MAIN CONTENT: Filter Columns + Results + Preview -->
  <!-- ====================================================================== -->
  <div class="main-content">
    <!-- VIP3-Style Filter Columns -->
    <div class="filter-columns">
      <!-- INSTRUMENT Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.instruments}>
        <button class="column-header" on:click={() => toggleColumn('instruments')}>
          <span class="column-title">🎹 Instrument</span>
          {#if filters.instruments.length > 0}
            <span class="filter-badge">{filters.instruments.length}</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.instruments ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.instruments}
          <div class="column-content">
            <button
              class="filter-option"
              class:active={filters.instruments.length === 0}
              on:click={() => clearCategory('instruments')}
            >
              <span class="option-check">{filters.instruments.length === 0 ? '●' : '○'}</span>
              <span class="option-label">All</span>
            </button>
            <div class="divider"></div>
            {#each defaultInstruments as inst (inst.name)}
              <button
                class="filter-option"
                class:active={filters.instruments.includes(inst.name.toLowerCase())}
                on:click={() => toggleFilter('instruments', inst.name.toLowerCase())}
              >
                <span class="option-check">
                  {filters.instruments.includes(inst.name.toLowerCase()) ? '☑' : '☐'}
                </span>
                <span class="option-icon">{inst.icon}</span>
                <span class="option-label">{inst.name}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- TIMBRE Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.timbres}>
        <button class="column-header" on:click={() => toggleColumn('timbres')}>
          <span class="column-title">🎨 Timbre</span>
          {#if filters.timbres.length > 0}
            <span class="filter-badge">{filters.timbres.length}</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.timbres ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.timbres}
          <div class="column-content">
            <button
              class="filter-option"
              class:active={filters.timbres.length === 0}
              on:click={() => clearCategory('timbres')}
            >
              <span class="option-check">{filters.timbres.length === 0 ? '●' : '○'}</span>
              <span class="option-label">All</span>
            </button>
            <div class="divider"></div>
            {#each defaultTimbres as timbre (timbre)}
              <button
                class="filter-option"
                class:active={filters.timbres.includes(timbre.toLowerCase())}
                on:click={() => toggleFilter('timbres', timbre.toLowerCase())}
              >
                <span class="option-check">
                  {filters.timbres.includes(timbre.toLowerCase()) ? '☑' : '☐'}
                </span>
                <span class="option-label">{timbre}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- STYLE Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.styles}>
        <button class="column-header" on:click={() => toggleColumn('styles')}>
          <span class="column-title">🎭 Style</span>
          {#if filters.styles.length > 0}
            <span class="filter-badge">{filters.styles.length}</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.styles ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.styles}
          <div class="column-content">
            <button
              class="filter-option"
              class:active={filters.styles.length === 0}
              on:click={() => clearCategory('styles')}
            >
              <span class="option-check">{filters.styles.length === 0 ? '●' : '○'}</span>
              <span class="option-label">All</span>
            </button>
            <div class="divider"></div>
            {#each defaultStyles as style (style)}
              <button
                class="filter-option"
                class:active={filters.styles.includes(style.toLowerCase())}
                on:click={() => toggleFilter('styles', style.toLowerCase())}
              >
                <span class="option-check">
                  {filters.styles.includes(style.toLowerCase()) ? '☑' : '☐'}
                </span>
                <span class="option-label">{style}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- ARTICULATION Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.articulations}>
        <button class="column-header" on:click={() => toggleColumn('articulations')}>
          <span class="column-title">🎯 Articulation</span>
          {#if filters.articulations.length > 0}
            <span class="filter-badge">{filters.articulations.length}</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.articulations ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.articulations}
          <div class="column-content">
            <button
              class="filter-option"
              class:active={filters.articulations.length === 0}
              on:click={() => clearCategory('articulations')}
            >
              <span class="option-check">{filters.articulations.length === 0 ? '●' : '○'}</span>
              <span class="option-label">All</span>
            </button>
            <div class="divider"></div>
            {#each defaultArticulations as art (art)}
              <button
                class="filter-option"
                class:active={filters.articulations.includes(art.toLowerCase())}
                on:click={() => toggleFilter('articulations', art.toLowerCase())}
              >
                <span class="option-check">
                  {filters.articulations.includes(art.toLowerCase()) ? '☑' : '☐'}
                </span>
                <span class="option-label">{art}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- BPM Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.bpm}>
        <button class="column-header" on:click={() => toggleColumn('bpm')}>
          <span class="column-title">⏱️ BPM</span>
          {#if filters.bpmRange}
            <span class="filter-badge">1</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.bpm ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.bpm}
          <div class="column-content">
            {#each bpmRanges as range (range.value)}
              <button
                class="filter-option"
                class:active={filters.bpmRange === range.value}
                on:click={() => toggleFilter('bpmRange', range.value)}
              >
                <span class="option-check">
                  {filters.bpmRange === range.value ? '●' : '○'}
                </span>
                <span class="option-label">{range.label}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- KEY Column -->
      <div class="filter-column" class:collapsed={columnCollapsed.key}>
        <button class="column-header" on:click={() => toggleColumn('key')}>
          <span class="column-title">🎼 Key</span>
          {#if filters.key}
            <span class="filter-badge">1</span>
          {/if}
          <span class="collapse-icon">{columnCollapsed.key ? '▶' : '▼'}</span>
        </button>
        {#if !columnCollapsed.key}
          <div class="column-content">
            <button
              class="filter-option"
              class:active={!filters.key}
              on:click={() => clearCategory('key')}
            >
              <span class="option-check">{!filters.key ? '●' : '○'}</span>
              <span class="option-label">All</span>
            </button>
            <div class="divider"></div>
            <div class="key-grid">
              {#each musicalKeys as key (key)}
                <button
                  class="key-option"
                  class:active={filters.key === key}
                  on:click={() => toggleFilter('key', key)}
                >
                  {key}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Results List -->
    <div class="results-panel">
      <div class="results-header">
        <span class="results-count">
          {searchResults.length} results
          {#if getActiveFilterCount() > 0}
            <span class="filter-summary">
              (filtered by {getActiveFilterCount()} {getActiveFilterCount() === 1 ? 'criterion' : 'criteria'})
            </span>
          {/if}
        </span>
        <div class="results-pagination">
          Page {currentPage + 1} of {$totalPages || 1}
        </div>
      </div>

      <div class="results-list">
        {#if isLoading}
          <div class="loading-state">
            <span class="spinner">⟳</span>
            Searching...
          </div>
        {:else if searchResults.length === 0}
          <div class="empty-state">
            <span class="empty-icon">🔍</span>
            <span class="empty-text">No files found</span>
            <span class="empty-hint">Try adjusting your filters</span>
          </div>
        {:else}
          {#each searchResults as file (file.id)}
            <button
              type="button"
              class="result-item"
              class:selected={selectedFile?.id === file.id}
              on:click={() => selectFile(file)}
              on:dblclick={() => handleDoubleClick(file)}
              draggable="true"
              on:dragstart={(e) => handleDragStart(e, file)}
            >
              <!-- Rating Stars -->
              <div class="rating-stars">
                {#each [1, 2, 3, 4, 5] as star (star)}
                  <button
                    class="star-btn"
                    class:filled={(file.rating ?? getFileRating(file.id)) >= star}
                    on:click|stopPropagation={() => setRating(file, star)}
                  >
                    {(file.rating ?? getFileRating(file.id)) >= star ? '★' : '☆'}
                  </button>
                {/each}
              </div>

              <!-- File Info -->
              <div class="file-info">
                <span class="file-name">{file.filename}</span>
                <span class="file-tags">
                  {#if file.bpm}
                    <span class="tag bpm">{formatBPM(file.bpm)} BPM</span>
                  {/if}
                  {#if file.key_signature}
                    <span class="tag key">{file.key_signature}</span>
                  {/if}
                  {#if file.duration_seconds}
                    <span class="tag duration">{formatDuration(file.duration_seconds)}</span>
                  {/if}
                </span>
              </div>

              <!-- Favorite -->
              <button
                class="favorite-btn"
                class:active={file.is_favorite}
                on:click|stopPropagation={() => toggleFavorite(file)}
              >
                {file.is_favorite ? '❤️' : '🤍'}
              </button>

              <!-- Context Menu -->
              <button class="context-btn" on:click|stopPropagation>⋮</button>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Pagination Controls -->
      {#if $totalPages > 1}
        <div class="pagination-controls">
          <button
            class="page-btn"
            on:click={previousPage}
            disabled={currentPage === 0 || isLoading}
          >
            ◀ Prev
          </button>
          <span class="page-info">
            Page {currentPage + 1} of {$totalPages}
          </span>
          <button
            class="page-btn"
            on:click={nextPage}
            disabled={currentPage >= $totalPages - 1 || isLoading}
          >
            Next ▶
          </button>
        </div>
      {/if}
    </div>

    <!-- Preview Panel -->
    <div class="preview-panel">
      {#if selectedFile}
        <div class="preview-header">
          <span class="preview-title">🎵 Preview</span>
        </div>
        <div class="preview-content">
          <div class="preview-filename">{selectedFile.filename}</div>

          <div class="preview-waveform">
            <!-- Placeholder for waveform visualization -->
            <div class="waveform-placeholder">
              <span>🎵</span>
            </div>
          </div>

          <div class="preview-metadata">
            <div class="meta-row">
              <span class="meta-label">BPM:</span>
              <span class="meta-value">{formatBPM(selectedFile.bpm)}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Key:</span>
              <span class="meta-value">{selectedFile.key_signature || '--'}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Duration:</span>
              <span class="meta-value">{formatDuration(selectedFile.duration_seconds)}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Tracks:</span>
              <span class="meta-value">{selectedFile.track_count || '--'}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">Notes:</span>
              <span class="meta-value">{selectedFile.total_notes?.toLocaleString() || '--'}</span>
            </div>
          </div>

          <div class="preview-rating">
            <span class="rating-label">Rating:</span>
            <div class="rating-stars large">
              {#each [1, 2, 3, 4, 5] as star (star)}
                <button
                  class="star-btn large"
                  class:filled={(selectedFile.rating ?? getFileRating(selectedFile.id)) >= star}
                  on:click={() => selectedFile && setRating(selectedFile, star)}
                >
                  {(selectedFile.rating ?? getFileRating(selectedFile.id)) >= star ? '★' : '☆'}
                </button>
              {/each}
            </div>
          </div>

          <div class="preview-actions">
            <button class="action-btn primary" on:click={() => selectedFile && handleDoubleClick(selectedFile)}>
              ▶ Load to DAW
            </button>
            <button class="action-btn" on:click={() => selectedFile && toggleFavorite(selectedFile)}>
              {selectedFile.is_favorite ? '💔 Remove Favorite' : '❤️ Add Favorite'}
            </button>
          </div>
        </div>
      {:else}
        <div class="preview-empty">
          <span class="empty-icon">📋</span>
          <span class="empty-text">Select a file to preview</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* ========================================================================== */
  /* VIP3-STYLE DATABASE BROWSER */
  /* ========================================================================== */

  .vip3-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    font-family: system-ui, -apple-system, sans-serif;
  }

  /* -------------------------------------------------------------------------- */
  /* TOP BAR */
  /* -------------------------------------------------------------------------- */

  .top-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: var(--bg-secondary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .search-section {
    display: flex;
    align-items: center;
    flex: 1;
    max-width: 500px;
    background: var(--bg-input, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    padding: 0 12px;
  }

  .search-icon {
    font-size: 14px;
    opacity: 0.6;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary, #e0e0e0);
    padding: 8px 12px;
    font-size: 14px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--text-muted, #666);
  }

  .search-btn {
    background: var(--accent-color, #4a9eff);
    color: white;
    border: none;
    padding: 6px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }

  .search-btn:hover {
    background: var(--accent-hover, #3a8eef);
  }

  .stats-section {
    display: flex;
    gap: 16px;
  }

  .stat {
    font-size: 12px;
    color: var(--text-secondary, #999);
  }

  .clear-all-btn {
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-secondary, #999);
    border: 1px solid var(--border-color, #444);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .clear-all-btn:hover {
    background: var(--danger-color, #ff4444);
    color: white;
    border-color: var(--danger-color, #ff4444);
  }

  /* -------------------------------------------------------------------------- */
  /* MAIN CONTENT */
  /* -------------------------------------------------------------------------- */

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* -------------------------------------------------------------------------- */
  /* FILTER COLUMNS (VIP3 Style) */
  /* -------------------------------------------------------------------------- */

  .filter-columns {
    display: flex;
    width: 660px;
    min-width: 660px;
    background: var(--bg-secondary, #252525);
    border-right: 1px solid var(--border-color, #333);
    overflow-x: auto;
  }

  .filter-column {
    display: flex;
    flex-direction: column;
    width: 110px;
    min-width: 110px;
    border-right: 1px solid var(--border-color, #333);
  }

  .filter-column:last-child {
    border-right: none;
  }

  .filter-column.collapsed {
    width: 40px;
    min-width: 40px;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 8px;
    background: var(--bg-tertiary, #2a2a2a);
    border: none;
    border-bottom: 1px solid var(--border-color, #333);
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .column-header:hover {
    background: var(--bg-hover, #333);
  }

  .column-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .collapsed .column-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
  }

  .filter-badge {
    background: var(--accent-color, #4a9eff);
    color: white;
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
  }

  .collapse-icon {
    font-size: 10px;
    color: var(--text-muted, #666);
  }

  .column-content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .filter-option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: var(--text-secondary, #999);
    font-size: 11px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .filter-option:hover {
    background: var(--bg-hover, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .filter-option.active {
    background: var(--accent-color, #4a9eff);
    color: white;
  }

  .option-check {
    font-size: 12px;
    width: 14px;
  }

  .option-icon {
    font-size: 12px;
  }

  .option-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .divider {
    height: 1px;
    background: var(--border-color, #333);
    margin: 4px 8px;
  }

  .key-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2px;
    padding: 4px;
  }

  .key-option {
    padding: 4px;
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 3px;
    color: var(--text-secondary, #999);
    font-size: 10px;
    cursor: pointer;
    text-align: center;
  }

  .key-option:hover {
    background: var(--bg-hover, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .key-option.active {
    background: var(--accent-color, #4a9eff);
    color: white;
    border-color: var(--accent-color, #4a9eff);
  }

  /* -------------------------------------------------------------------------- */
  /* RESULTS PANEL */
  /* -------------------------------------------------------------------------- */

  .results-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 300px;
    background: var(--bg-primary, #1a1a1a);
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background: var(--bg-secondary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .results-count {
    font-size: 12px;
    color: var(--text-secondary, #999);
  }

  .filter-summary {
    color: var(--text-muted, #666);
  }

  .results-pagination {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  .results-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-muted, #666);
  }

  .spinner {
    font-size: 24px;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
  }

  .empty-text {
    font-size: 14px;
    margin-bottom: 4px;
  }

  .empty-hint {
    font-size: 12px;
    opacity: 0.6;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-secondary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 6px;
    cursor: pointer;
    width: 100%;
    text-align: left;
    transition: all 0.15s ease;
  }

  .result-item:hover {
    background: var(--bg-hover, #2a2a2a);
    border-color: var(--accent-color, #4a9eff);
  }

  .result-item.selected {
    background: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
  }

  .result-item.selected .file-name,
  .result-item.selected .file-tags .tag {
    color: white;
  }

  .rating-stars {
    display: flex;
    gap: 1px;
  }

  .star-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #444);
    font-size: 12px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .star-btn.filled {
    color: var(--warning-color, #ffc107);
  }

  .star-btn:hover {
    color: var(--warning-color, #ffc107);
  }

  .star-btn.large {
    font-size: 18px;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 2px;
  }

  .file-tags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .tag {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-tertiary, #333);
    border-radius: 3px;
    color: var(--text-secondary, #999);
  }

  .tag.bpm {
    background: rgba(74, 158, 255, 0.2);
    color: var(--accent-color, #4a9eff);
  }

  .tag.key {
    background: rgba(76, 175, 80, 0.2);
    color: #4caf50;
  }

  .tag.duration {
    background: rgba(156, 39, 176, 0.2);
    color: #9c27b0;
  }

  .favorite-btn {
    background: transparent;
    border: none;
    font-size: 14px;
    cursor: pointer;
    padding: 4px;
    opacity: 0.6;
    transition: opacity 0.15s;
  }

  .favorite-btn:hover,
  .favorite-btn.active {
    opacity: 1;
  }

  .context-btn {
    background: transparent;
    border: none;
    color: var(--text-muted, #666);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
  }

  .context-btn:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .pagination-controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 12px;
    background: var(--bg-secondary, #252525);
    border-top: 1px solid var(--border-color, #333);
  }

  .page-btn {
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    color: var(--text-secondary, #999);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .page-btn:hover:not(:disabled) {
    background: var(--bg-hover, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .page-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  /* -------------------------------------------------------------------------- */
  /* PREVIEW PANEL */
  /* -------------------------------------------------------------------------- */

  .preview-panel {
    width: 280px;
    min-width: 280px;
    background: var(--bg-secondary, #252525);
    border-left: 1px solid var(--border-color, #333);
    display: flex;
    flex-direction: column;
  }

  .preview-header {
    padding: 12px 16px;
    background: var(--bg-tertiary, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .preview-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .preview-content {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
  }

  .preview-filename {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 16px;
    word-break: break-word;
  }

  .preview-waveform {
    background: var(--bg-primary, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    height: 80px;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .waveform-placeholder {
    font-size: 32px;
    opacity: 0.3;
  }

  .preview-metadata {
    margin-bottom: 16px;
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .meta-label {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  .meta-value {
    font-size: 12px;
    color: var(--text-primary, #e0e0e0);
    font-weight: 500;
  }

  .preview-rating {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .rating-label {
    font-size: 12px;
    color: var(--text-muted, #666);
  }

  .rating-stars.large {
    gap: 4px;
  }

  .preview-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .action-btn {
    padding: 10px 16px;
    border: 1px solid var(--border-color, #444);
    border-radius: 6px;
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-secondary, #999);
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: var(--bg-hover, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .action-btn.primary {
    background: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
    color: white;
  }

  .action-btn.primary:hover {
    background: var(--accent-hover, #3a8eef);
  }

  .preview-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted, #666);
  }

  .preview-empty .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .preview-empty .empty-text {
    font-size: 13px;
  }

  /* -------------------------------------------------------------------------- */
  /* RESPONSIVE */
  /* -------------------------------------------------------------------------- */

  @media (max-width: 1400px) {
    .filter-columns {
      width: 440px;
      min-width: 440px;
    }

    .filter-column {
      width: 73px;
      min-width: 73px;
    }

    .preview-panel {
      width: 240px;
      min-width: 240px;
    }
  }

  @media (max-width: 1200px) {
    .preview-panel {
      display: none;
    }
  }
</style>
