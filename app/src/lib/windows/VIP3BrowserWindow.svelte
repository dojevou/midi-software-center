<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import VIP3Browser from '$lib/components/VIP3Browser.svelte';
  import VIP3ContextMenu from '$lib/components/VIP3ContextMenu.svelte';
  import VIP3SavedSearches from '$lib/components/VIP3SavedSearches.svelte';
  import { databaseStore, databaseActions } from '$lib/stores/databaseStore';
  import { vip3Store, vip3Actions, type VIP3Filters, type SavedSearch } from '$lib/stores/vip3Store';
  import { sequencerActions } from '$lib/stores/sequencerStore';
  import { uiActions } from '$lib/stores/uiStore';
  import { api } from '$lib/api';
  import type { FileDetails, WindowId } from '$lib/types';

  export let windowId: WindowId = 'vip3-browser';

  // Context menu state
  let contextMenuFile: FileDetails | null = null;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let showContextMenu = false;

  // Saved searches state
  let savedSearchesCollapsed = false;

  // Subscribe to vip3Store for saved searches
  $: savedSearches = $vip3Store.savedSearches;
  $: recentFilters = $vip3Store.recentSearches;
  $: currentFilters = {
    timbreIds: [],
    styleIds: [],
    articulationIds: [],
    bpmRangeId: null,
    keyIds: [],
    searchQuery: searchQuery,
    trackLayerFilter: trackLayerFilter,
    sortBy: 'filename' as const,
    sortOrder: 'asc' as const,
  };

  // Filter state - kept in component for reactivity
  let folders: string[] = [];
  let instruments: string[] = [];
  let timbres: string[] = [];
  let styles: string[] = [];
  let articulations: string[] = [];
  let bpmRanges: string[] = [];
  let keys: string[] = [];
  let channels: string[] = [];
  let trackLayerFilter: 'all' | 'single' | 'multi' = 'all';
  let searchQuery: string = '';

  // Options populated from backend
  let folderOptions: { value: string; label: string; count?: number }[] = [];
  let instrumentOptions: { value: string; label: string; count?: number }[] = [];
  let timbreOptions: { value: string; label: string; count?: number }[] = [];
  let styleOptions: { value: string; label: string; count?: number }[] = [];
  let articulationOptions: { value: string; label: string; count?: number }[] = [];
  let bpmRangeOptions: { value: string; label: string; count?: number }[] = [];
  let keyOptions: { value: string; label: string; count?: number }[] = [];
  let channelOptions: { value: string; label: string; count?: number }[] = [];

  // Search suggestions and recent searches
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const searchSuggestions: string[] = [];
  let recentSearches: string[] = [];

  // Selection state
  let selectedFileIds: number[] = [];

  // Subscribe to database store
  $: files = $databaseStore.searchResults;
  $: isLoading = $databaseStore.isLoading;
  $: currentPage = $databaseStore.currentPage;
  $: totalPages = $databaseStore.totalPages;
  $: totalFiles = $databaseStore.totalFiles ?? files.length;

  // Initialize default filter options
  onMount(async () => {
    // Initialize BPM range options (static)
    bpmRangeOptions = [
      { value: '60-80', label: '60-80 (Slow)', count: 0 },
      { value: '80-100', label: '80-100', count: 0 },
      { value: '100-120', label: '100-120', count: 0 },
      { value: '120-140', label: '120-140', count: 0 },
      { value: '140-160', label: '140-160', count: 0 },
      { value: '160-180', label: '160-180', count: 0 },
      { value: '180-200', label: '180-200', count: 0 },
      { value: '200+', label: '200+ (Fast)', count: 0 },
    ];

    // Initialize key options (static - 12 major + 12 minor)
    keyOptions = [
      { value: 'C major', label: 'C Major', count: 0 },
      { value: 'C minor', label: 'C Minor', count: 0 },
      { value: 'C# major', label: 'C# Major', count: 0 },
      { value: 'C# minor', label: 'C# Minor', count: 0 },
      { value: 'D major', label: 'D Major', count: 0 },
      { value: 'D minor', label: 'D Minor', count: 0 },
      { value: 'D# major', label: 'D# Major', count: 0 },
      { value: 'D# minor', label: 'D# Minor', count: 0 },
      { value: 'E major', label: 'E Major', count: 0 },
      { value: 'E minor', label: 'E Minor', count: 0 },
      { value: 'F major', label: 'F Major', count: 0 },
      { value: 'F minor', label: 'F Minor', count: 0 },
      { value: 'F# major', label: 'F# Major', count: 0 },
      { value: 'F# minor', label: 'F# Minor', count: 0 },
      { value: 'G major', label: 'G Major', count: 0 },
      { value: 'G minor', label: 'G Minor', count: 0 },
      { value: 'G# major', label: 'G# Major', count: 0 },
      { value: 'G# minor', label: 'G# Minor', count: 0 },
      { value: 'A major', label: 'A Major', count: 0 },
      { value: 'A minor', label: 'A Minor', count: 0 },
      { value: 'A# major', label: 'A# Major', count: 0 },
      { value: 'A# minor', label: 'A# Minor', count: 0 },
      { value: 'B major', label: 'B Major', count: 0 },
      { value: 'B minor', label: 'B Minor', count: 0 },
    ];

    // Initialize channel options (1-16)
    channelOptions = Array.from({ length: 16 }, (_, i) => ({
      value: String(i + 1),
      label: `Channel ${i + 1}`,
      count: 0,
    }));

    // Load dynamic options from backend
    await loadFilterOptions();

    // Load recent searches from localStorage
    const stored = localStorage.getItem('vip3-recent-searches');
    if (stored) {
      try {
        recentSearches = JSON.parse(stored);
      } catch {
        recentSearches = [];
      }
    }

    // Initial search
    await executeSearch();
  });

  // Helper to transform tag response to filter option format
  interface TagResponse {
    id: number;
    name: string;
    category: string | null;
    usage_count: number;
  }

  function tagToOption(tag: TagResponse): { value: string; label: string; count: number } {
    // Capitalize first letter for label
    const label = tag.name.charAt(0).toUpperCase() + tag.name.slice(1);
    return {
      value: tag.name,
      label: label,
      count: tag.usage_count,
    };
  }

  async function loadFilterOptions() {
    try {
      // Fetch all filter options from database in parallel
      const [instrumentTags, styleTags, moodTags, keyTags] = await Promise.all([
        invoke<TagResponse[]>('get_all_instruments'),
        invoke<TagResponse[]>('get_vip3_styles'),
        invoke<TagResponse[]>('get_vip3_moods'),
        invoke<TagResponse[]>('get_vip3_keys'),
      ]);

      // Transform to filter option format
      instrumentOptions = instrumentTags.map(tagToOption);
      styleOptions = styleTags.map(tagToOption);
      timbreOptions = moodTags.map(tagToOption);
      keyOptions = keyTags.map(tagToOption);

      // Folders use the same instrument categories (drums, bass, keys, etc.)
      // Group by category for folder-level filtering
      const categoryMap = new Map<string, number>();
      for (const tag of instrumentTags) {
        if (tag.category) {
          const current = categoryMap.get(tag.category) || 0;
          categoryMap.set(tag.category, current + tag.usage_count);
        }
      }
      folderOptions = Array.from(categoryMap.entries())
        .map(([category, count]) => ({
          value: category,
          label: category.charAt(0).toUpperCase() + category.slice(1),
          count,
        }))
        .sort((a, b) => b.count - a.count);

      // BPM ranges - static for now (could be from DB)
      bpmRangeOptions = [
        { value: '60-80', label: '60-80 BPM', count: 0 },
        { value: '80-100', label: '80-100 BPM', count: 0 },
        { value: '100-120', label: '100-120 BPM', count: 0 },
        { value: '120-140', label: '120-140 BPM', count: 0 },
        { value: '140-160', label: '140-160 BPM', count: 0 },
        { value: '160+', label: '160+ BPM', count: 0 },
      ];

      // Channel options - static (MIDI channels 1-16)
      channelOptions = Array.from({ length: 16 }, (_, i) => ({
        value: String(i + 1),
        label: `Channel ${i + 1}`,
        count: 0,
      }));

      console.log('Filter options loaded from database:', {
        instruments: instrumentOptions.length,
        styles: styleOptions.length,
        moods: timbreOptions.length,
        keys: keyOptions.length,
        folders: folderOptions.length,
      });
    } catch (error) {
      console.error('Failed to load filter options:', error);
      // Fallback to empty arrays on error
      instrumentOptions = [];
      styleOptions = [];
      timbreOptions = [];
      keyOptions = [];
      folderOptions = [];
    }
  }

  async function executeSearch() {
    const filters = {
      query: searchQuery,
      folders,
      instruments,
      timbres,
      styles,
      articulations,
      bpmRanges,
      keys,
      channels,
      trackLayerFilter,
    };

    await databaseActions.search(filters);
  }

  function handleFilterChange(event: CustomEvent<{ type: string; values: string[] | string }>) {
    const { type, values } = event.detail;

    switch (type) {
      case 'folder': folders = values as string[]; break;
      case 'instrument': instruments = values as string[]; break;
      case 'timbre': timbres = values as string[]; break;
      case 'style': styles = values as string[]; break;
      case 'articulation': articulations = values as string[]; break;
      case 'bpm': bpmRanges = values as string[]; break;
      case 'key': keys = values as string[]; break;
      case 'channel': channels = values as string[]; break;
      case 'trackLayer': trackLayerFilter = values as 'all' | 'single' | 'multi'; break;
    }

    void executeSearch();
  }

  function handleSearch(event: CustomEvent<{ query: string }>) {
    searchQuery = event.detail.query;

    // Add to recent searches
    if (searchQuery && !recentSearches.includes(searchQuery)) {
      recentSearches = [searchQuery, ...recentSearches.slice(0, 9)];
      localStorage.setItem('vip3-recent-searches', JSON.stringify(recentSearches));
    }

    void executeSearch();
  }

  function handleClearAllFilters() {
    folders = [];
    instruments = [];
    timbres = [];
    styles = [];
    articulations = [];
    bpmRanges = [];
    keys = [];
    channels = [];
    trackLayerFilter = 'all';
    searchQuery = '';
    void executeSearch();
  }

  function handleSelectFile(event: CustomEvent<{ file: FileDetails; ctrlKey: boolean; shiftKey: boolean }>) {
    const { file, ctrlKey, shiftKey } = event.detail;

    if (ctrlKey) {
      // Toggle selection
      if (selectedFileIds.includes(file.id)) {
        selectedFileIds = selectedFileIds.filter(id => id !== file.id);
      } else {
        selectedFileIds = [...selectedFileIds, file.id];
      }
    } else if (shiftKey && selectedFileIds.length > 0) {
      // Range selection
      const lastSelectedId = selectedFileIds[selectedFileIds.length - 1];
      const lastIndex = files.findIndex(f => f.id === lastSelectedId);
      const currentIndex = files.findIndex(f => f.id === file.id);

      if (lastIndex !== -1 && currentIndex !== -1) {
        const start = Math.min(lastIndex, currentIndex);
        const end = Math.max(lastIndex, currentIndex);
        const rangeIds = files.slice(start, end + 1).map(f => f.id);
        selectedFileIds = [...new Set([...selectedFileIds, ...rangeIds])];
      }
    } else {
      // Single selection
      selectedFileIds = [file.id];
    }
  }

  function handleDoubleClickFile(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    // Open file preview or add to sequencer
    console.log('Double-clicked file:', file.filename);
  }

  function handleContextMenu(event: CustomEvent<{ file: FileDetails; x: number; y: number }>) {
    const { file, x, y } = event.detail;
    contextMenuFile = file;
    contextMenuX = x;
    contextMenuY = y;
    showContextMenu = true;
  }

  function handleCloseContextMenu() {
    showContextMenu = false;
    contextMenuFile = null;
  }

  function handlePageChange(event: CustomEvent<{ page: number }>) {
    databaseActions.goToPage(event.detail.page);
  }

  // Context menu action handlers
  async function handlePreview(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    console.log('Preview:', file.filename);
    try {
      // Use transport API to preview the MIDI file
      await api.transport.playFile(file.id);
    } catch (error) {
      console.error('Failed to preview file:', error);
    }
  }

  async function handleAddToSequencer(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    console.log('Add to sequencer:', file.filename);
    try {
      // Add a new track and clip for this file
      const trackId = sequencerActions.addTrack(file.filename);
      if (trackId) {
        // Add clip at start of timeline (tick 0) with default length
        sequencerActions.addClip(trackId, 0, 1920, file.id, file.filename); // 1920 ticks = 1 bar at 480 PPQ
      }
    } catch (error) {
      console.error('Failed to add to sequencer:', error);
    }
  }

  async function handleAddToTrack(event: CustomEvent<{ file: FileDetails; trackIndex: number }>) {
    const { file, trackIndex } = event.detail;
    console.log('Add to track', trackIndex, ':', file.filename);
    try {
      // Get tracks from sequencer store
      const state = $databaseStore;
      // For now, just add to sequencer with a new track - track selection can be added later
      const trackId = sequencerActions.addTrack(`Track ${trackIndex + 1}`);
      if (trackId) {
        sequencerActions.addClip(trackId, 0, 1920, file.id, file.filename);
      }
    } catch (error) {
      console.error('Failed to add to track:', error);
    }
  }

  async function handleToggleFavorite(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    console.log('Toggle favorite:', file.filename);
    try {
      if (file.is_favorite) {
        await api.analysis.removeFavorite(file.id);
      } else {
        await api.analysis.addFavorite(file.id);
      }
      // Refresh the file list to show updated favorite status
      await databaseActions.refresh();
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }

  async function handleEditTags(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    console.log('Edit tags:', file.filename);
    // Store the selected file ID for the tag editor to use
    databaseActions.selectFile(file.id);
    // Show the tag editor window
    uiActions.showWindow('database'); // Show database window which has tag editing
  }

  async function handleShowInFolder(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    try {
      // Use Tauri's shell command to open file location
      await invoke('show_in_folder', { path: file.filepath });
    } catch (error) {
      console.error('Failed to show in folder:', error);
      // Fallback: copy path to clipboard
      await navigator.clipboard.writeText(file.filepath);
      console.log('Path copied to clipboard:', file.filepath);
    }
  }

  async function handleCopyPath(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    try {
      await navigator.clipboard.writeText(file.filepath);
      console.log('Path copied to clipboard:', file.filepath);
    } catch (error) {
      console.error('Failed to copy path:', error);
    }
  }

  async function handleDelete(event: CustomEvent<{ file: FileDetails }>) {
    const { file } = event.detail;
    if (confirm(`Are you sure you want to delete "${file.filename}"? This action cannot be undone.`)) {
      console.log('Delete:', file.filename);
      try {
        await api.files.deleteFile(file.id);
        // Refresh the file list after deletion
        await databaseActions.refresh();
      } catch (error) {
        console.error('Failed to delete file:', error);
        alert(`Failed to delete file: ${error}`);
      }
    }
  }

  // Saved searches handlers
  async function handleSaveSearch(event: CustomEvent<{ name: string }>) {
    const { name } = event.detail;
    await vip3Actions.saveCurrentSearch(name);
  }

  async function handleLoadSavedSearch(event: CustomEvent<{ id: number }>) {
    const { id } = event.detail;
    await vip3Actions.loadSavedSearch(id);
  }

  async function handleDeleteSavedSearch(event: CustomEvent<{ id: number }>) {
    const { id } = event.detail;
    await vip3Actions.deleteSavedSearch(id);
  }

  function handleLoadRecentSearch(event: CustomEvent<{ filters: VIP3Filters }>) {
    const { filters } = event.detail;
    // Apply the filters from the recent search
    searchQuery = filters.searchQuery;
    void executeSearch();
  }

  function handleToggleSavedSearches(event: CustomEvent<{ collapsed: boolean }>) {
    savedSearchesCollapsed = event.detail.collapsed;
  }
</script>

<WindowBase
  {windowId}
  title="VIP3 Browser"
  width={1200}
  height={700}
  minWidth={800}
  minHeight={500}
>
  <div class="browser-layout">
    <!-- Saved Searches Sidebar -->
    <div class="sidebar" class:collapsed={savedSearchesCollapsed}>
      <VIP3SavedSearches
        {savedSearches}
        recentSearches={recentFilters}
        {currentFilters}
        collapsed={savedSearchesCollapsed}
        on:save={handleSaveSearch}
        on:load={handleLoadSavedSearch}
        on:delete={handleDeleteSavedSearch}
        on:loadRecent={handleLoadRecentSearch}
        on:toggle={handleToggleSavedSearches}
      />
    </div>

    <!-- Main Browser -->
    <div class="browser-main">
      <VIP3Browser
        {folders}
        {instruments}
        {timbres}
        {styles}
        {articulations}
        {bpmRanges}
        {keys}
        {channels}
        {trackLayerFilter}
        {folderOptions}
        {instrumentOptions}
        {timbreOptions}
        {styleOptions}
        {articulationOptions}
        {bpmRangeOptions}
        {keyOptions}
        {channelOptions}
        {searchQuery}
        {searchSuggestions}
        {recentSearches}
        {files}
        {selectedFileIds}
        {isLoading}
        {currentPage}
        {totalPages}
        {totalFiles}
        on:filterChange={handleFilterChange}
        on:search={handleSearch}
        on:selectFile={handleSelectFile}
        on:doubleClickFile={handleDoubleClickFile}
        on:contextMenu={handleContextMenu}
        on:pageChange={handlePageChange}
        on:clearAllFilters={handleClearAllFilters}
      />
    </div>
  </div>
</WindowBase>

<!-- Context Menu (positioned absolutely outside window) -->
<VIP3ContextMenu
  file={contextMenuFile}
  x={contextMenuX}
  y={contextMenuY}
  visible={showContextMenu}
  on:preview={handlePreview}
  on:addToSequencer={handleAddToSequencer}
  on:addToTrack={handleAddToTrack}
  on:toggleFavorite={handleToggleFavorite}
  on:editTags={handleEditTags}
  on:showInFolder={handleShowInFolder}
  on:copyPath={handleCopyPath}
  on:delete={handleDelete}
  on:close={handleCloseContextMenu}
/>

<style>
  .browser-layout {
    display: flex;
    height: 100%;
    gap: 0;
  }

  .sidebar {
    width: 220px;
    flex-shrink: 0;
    background: var(--gray-850, #1f1f23);
    border-right: 1px solid var(--gray-700, #3f3f46);
    overflow-y: auto;
    transition: width 0.2s ease;
  }

  .sidebar.collapsed {
    width: 40px;
  }

  .browser-main {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  /* Scrollbar styling for sidebar */
  .sidebar::-webkit-scrollbar {
    width: 6px;
  }

  .sidebar::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .sidebar::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 3px;
  }

  .sidebar::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }
</style>
