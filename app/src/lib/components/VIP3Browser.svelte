<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { logger } from '$lib/utils/logger';
  import VIP3Column from './VIP3Column.svelte';
  import VIP3SearchBar from './VIP3SearchBar.svelte';
  import VIP3FileList from './VIP3FileList.svelte';
  import type { FileDetails } from '$lib/types';

  const log = logger.child({ component: 'VIP3Browser' });

  // Filter state
  export let folders: string[] = [];
  export let instruments: string[] = [];
  export let timbres: string[] = [];
  export let styles: string[] = [];
  export let articulations: string[] = [];
  export let bpmRanges: string[] = [];
  export let keys: string[] = [];
  export let channels: string[] = [];

  // Options for each column (loaded from backend)
  export let folderOptions: { value: string; label: string; count?: number }[] = [];
  export let instrumentOptions: { value: string; label: string; count?: number }[] = [];
  export let timbreOptions: { value: string; label: string; count?: number }[] = [];
  export let styleOptions: { value: string; label: string; count?: number }[] = [];
  export let articulationOptions: { value: string; label: string; count?: number }[] = [];
  export let bpmRangeOptions: { value: string; label: string; count?: number }[] = [];
  export let keyOptions: { value: string; label: string; count?: number }[] = [];
  export let channelOptions: { value: string; label: string; count?: number }[] = [];

  // Search
  export let searchQuery: string = '';
  export let searchSuggestions: string[] = [];
  export let recentSearches: string[] = [];

  // Results
  export let files: FileDetails[] = [];
  export let selectedFileIds: number[] = [];
  export let isLoading: boolean = false;
  export let currentPage: number = 1;
  export let totalPages: number = 1;
  export let totalFiles: number = 0;

  // Column collapse state
  const columnCollapsed = {
    folder: false,
    instrument: false,
    timbre: false,
    style: false,
    articulation: false,
    bpm: false,
    key: false,
    channel: false
  };

  const dispatch = createEventDispatcher<{
    filterChange: {
      type: 'folder' | 'instrument' | 'timbre' | 'style' | 'articulation' | 'bpm' | 'key' | 'channel';
      values: string[];
    };
    search: { query: string };
    selectFile: { file: FileDetails; ctrlKey: boolean; shiftKey: boolean };
    doubleClickFile: { file: FileDetails };
    contextMenu: { file: FileDetails; x: number; y: number };
    pageChange: { page: number };
    clearAllFilters: void;
  }>();

  // Active filter tags
  $: activeFilters = [
    ...folders.map(v => ({ type: 'folder' as const, value: v })),
    ...instruments.map(v => ({ type: 'instrument' as const, value: v })),
    ...timbres.map(v => ({ type: 'timbre' as const, value: v })),
    ...styles.map(v => ({ type: 'style' as const, value: v })),
    ...articulations.map(v => ({ type: 'articulation' as const, value: v })),
    ...bpmRanges.map(v => ({ type: 'bpm' as const, value: v })),
    ...keys.map(v => ({ type: 'key' as const, value: v })),
    ...channels.map(v => ({ type: 'channel' as const, value: v }))
  ];

  $: hasActiveFilters = activeFilters.length > 0 || searchQuery.length > 0;

  function handleColumnSelect(
    type: 'folder' | 'instrument' | 'timbre' | 'style' | 'articulation' | 'bpm' | 'key' | 'channel',
    event: CustomEvent<{ value: string; selected: boolean }>
  ) {
    const { value, selected } = event.detail;
    let currentValues: string[];

    switch (type) {
      case 'folder': currentValues = folders; break;
      case 'instrument': currentValues = instruments; break;
      case 'timbre': currentValues = timbres; break;
      case 'style': currentValues = styles; break;
      case 'articulation': currentValues = articulations; break;
      case 'bpm': currentValues = bpmRanges; break;
      case 'key': currentValues = keys; break;
      case 'channel': currentValues = channels; break;
    }

    const newValues = selected
      ? [...currentValues, value]
      : currentValues.filter(v => v !== value);

    log.info('Filter changed', { type, value, selected, totalSelected: newValues.length });
    dispatch('filterChange', { type, values: newValues });
  }

  function handleColumnClear(
    type: 'folder' | 'instrument' | 'timbre' | 'style' | 'articulation' | 'bpm' | 'key' | 'channel'
  ) {
    log.info('Filter column cleared', { type });
    dispatch('filterChange', { type, values: [] });
  }

  function handleColumnToggle(
    column: keyof typeof columnCollapsed,
    event: CustomEvent<{ collapsed: boolean }>
  ) {
    columnCollapsed[column] = event.detail.collapsed;
  }

  function removeFilter(type: string, value: string) {
    let currentValues: string[];
    const filterType = type as 'folder' | 'instrument' | 'timbre' | 'style' | 'articulation' | 'bpm' | 'key' | 'channel';

    switch (filterType) {
      case 'folder': currentValues = folders; break;
      case 'instrument': currentValues = instruments; break;
      case 'timbre': currentValues = timbres; break;
      case 'style': currentValues = styles; break;
      case 'articulation': currentValues = articulations; break;
      case 'bpm': currentValues = bpmRanges; break;
      case 'key': currentValues = keys; break;
      case 'channel': currentValues = channels; break;
    }

    log.debug('Filter removed', { type: filterType, value });
    dispatch('filterChange', {
      type: filterType,
      values: currentValues.filter(v => v !== value)
    });
  }

  function handleClearAllFilters() {
    log.info('All filters cleared', { previousFilterCount: activeFilters.length });
    dispatch('clearAllFilters');
  }

  function handleSearch(event: CustomEvent<{ query: string }>) {
    log.info('Search executed', { query: event.detail.query });
    dispatch('search', event.detail);
  }

  function handleSearchClear() {
    log.debug('Search cleared');
    searchQuery = '';
    dispatch('search', { query: '' });
  }

  function getFilterLabel(type: string): string {
    switch (type) {
      case 'folder': return '📁';
      case 'instrument': return '🎹';
      case 'timbre': return '🎨';
      case 'style': return '🎭';
      case 'articulation': return '🎯';
      case 'bpm': return '⏱️';
      case 'key': return '🔑';
      case 'channel': return '📻';
      default: return '';
    }
  }
</script>

<div class="vip3-browser">
  <!-- Search Bar -->
  <div class="browser-header">
    <VIP3SearchBar
      value={searchQuery}
      suggestions={searchSuggestions}
      recentSearches={recentSearches}
      {isLoading}
      on:search={handleSearch}
      on:clear={handleSearchClear}
      on:input={(e) => searchQuery = e.detail.value}
    />
  </div>

  <!-- Active Filters Bar -->
  {#if hasActiveFilters}
    <div class="active-filters">
      <span class="filters-label">Active Filters:</span>
      <div class="filter-tags">
        {#if searchQuery}
          <button
            type="button"
            class="filter-tag search-tag"
            on:click={handleSearchClear}
          >
            <span class="tag-icon">🔍</span>
            <span class="tag-text">"{searchQuery}"</span>
            <span class="tag-remove">✕</span>
          </button>
        {/if}
        {#each activeFilters as filter (filter.type + filter.value)}
          <button
            type="button"
            class="filter-tag"
            on:click={() => removeFilter(filter.type, filter.value)}
          >
            <span class="tag-icon">{getFilterLabel(filter.type)}</span>
            <span class="tag-text">{filter.value}</span>
            <span class="tag-remove">✕</span>
          </button>
        {/each}
      </div>
      <button
        type="button"
        class="clear-all-button"
        on:click={handleClearAllFilters}
      >
        Clear All
      </button>
    </div>
  {/if}

  <!-- Main Browser Area -->
  <div class="browser-content">
    <!-- Filter Columns -->
    <div class="filter-columns">
      <VIP3Column
        title="FOLDER"
        icon="📁"
        options={folderOptions}
        selected={folders}
        multiSelect={true}
        collapsed={columnCollapsed.folder}
        on:select={(e) => handleColumnSelect('folder', e)}
        on:clear={() => handleColumnClear('folder')}
        on:toggle={(e) => handleColumnToggle('folder', e)}
      />

      <VIP3Column
        title="INSTRUMENT"
        icon="🎹"
        options={instrumentOptions}
        selected={instruments}
        multiSelect={true}
        collapsed={columnCollapsed.instrument}
        on:select={(e) => handleColumnSelect('instrument', e)}
        on:clear={() => handleColumnClear('instrument')}
        on:toggle={(e) => handleColumnToggle('instrument', e)}
      />

      <VIP3Column
        title="TIMBRE"
        icon="🎨"
        options={timbreOptions}
        selected={timbres}
        multiSelect={true}
        collapsed={columnCollapsed.timbre}
        on:select={(e) => handleColumnSelect('timbre', e)}
        on:clear={() => handleColumnClear('timbre')}
        on:toggle={(e) => handleColumnToggle('timbre', e)}
      />

      <VIP3Column
        title="STYLE"
        icon="🎭"
        options={styleOptions}
        selected={styles}
        multiSelect={true}
        collapsed={columnCollapsed.style}
        on:select={(e) => handleColumnSelect('style', e)}
        on:clear={() => handleColumnClear('style')}
        on:toggle={(e) => handleColumnToggle('style', e)}
      />

      <VIP3Column
        title="ARTICULATION"
        icon="🎯"
        options={articulationOptions}
        selected={articulations}
        multiSelect={true}
        collapsed={columnCollapsed.articulation}
        on:select={(e) => handleColumnSelect('articulation', e)}
        on:clear={() => handleColumnClear('articulation')}
        on:toggle={(e) => handleColumnToggle('articulation', e)}
      />

      <VIP3Column
        title="BPM"
        icon="⏱️"
        options={bpmRangeOptions}
        selected={bpmRanges}
        multiSelect={false}
        collapsed={columnCollapsed.bpm}
        on:select={(e) => handleColumnSelect('bpm', e)}
        on:clear={() => handleColumnClear('bpm')}
        on:toggle={(e) => handleColumnToggle('bpm', e)}
      />

      <VIP3Column
        title="KEY"
        icon="🔑"
        options={keyOptions}
        selected={keys}
        multiSelect={true}
        collapsed={columnCollapsed.key}
        on:select={(e) => handleColumnSelect('key', e)}
        on:clear={() => handleColumnClear('key')}
        on:toggle={(e) => handleColumnToggle('key', e)}
      />

      <VIP3Column
        title="CHANNEL"
        icon="📻"
        options={channelOptions}
        selected={channels}
        multiSelect={true}
        collapsed={columnCollapsed.channel}
        on:select={(e) => handleColumnSelect('channel', e)}
        on:clear={() => handleColumnClear('channel')}
        on:toggle={(e) => handleColumnToggle('channel', e)}
      />
    </div>

    <!-- Results List -->
    <div class="results-area">
      <VIP3FileList
        {files}
        selectedIds={selectedFileIds}
        {isLoading}
        {currentPage}
        {totalPages}
        {totalFiles}
        on:select={(e) => dispatch('selectFile', e.detail)}
        on:doubleClick={(e) => dispatch('doubleClickFile', e.detail)}
        on:contextMenu={(e) => dispatch('contextMenu', e.detail)}
        on:pageChange={(e) => dispatch('pageChange', e.detail)}
      />
    </div>
  </div>
</div>

<style>
  .vip3-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--gray-900, #18181b);
    color: var(--gray-100, #f4f4f5);
  }

  .browser-header {
    padding: 12px;
    background: var(--gray-850, #1f1f23);
    border-bottom: 1px solid var(--gray-700, #3f3f46);
  }

  .active-filters {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--gray-800, #27272a);
    border-bottom: 1px solid var(--gray-700, #3f3f46);
    flex-wrap: wrap;
  }

  .filters-label {
    font-size: 11px;
    color: var(--gray-500, #71717a);
    font-weight: 500;
    flex-shrink: 0;
  }

  .filter-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    flex: 1;
  }

  .filter-tag {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--primary-600, #2563eb);
    border: none;
    border-radius: 4px;
    color: white;
    font-size: 11px;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .filter-tag:hover {
    background: var(--primary-500, #3b82f6);
  }

  .filter-tag.search-tag {
    background: var(--gray-600, #52525b);
  }

  .filter-tag.search-tag:hover {
    background: var(--gray-500, #71717a);
  }

  .tag-icon {
    font-size: 10px;
  }

  .tag-text {
    max-width: 120px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag-remove {
    font-size: 10px;
    opacity: 0.7;
  }

  .filter-tag:hover .tag-remove {
    opacity: 1;
  }

  .clear-all-button {
    background: transparent;
    border: 1px solid var(--gray-600, #52525b);
    color: var(--gray-400, #a1a1aa);
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s ease;
    flex-shrink: 0;
  }

  .clear-all-button:hover {
    background: var(--gray-700, #3f3f46);
    border-color: var(--gray-500, #71717a);
    color: var(--gray-200, #e4e4e7);
  }

  .browser-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .filter-columns {
    display: flex;
    flex-shrink: 0;
    border-right: 1px solid var(--gray-700, #3f3f46);
    overflow-x: auto;
  }

  .results-area {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  /* Scrollbar for filter columns horizontal scroll */
  .filter-columns::-webkit-scrollbar {
    height: 6px;
  }

  .filter-columns::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .filter-columns::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 3px;
  }

  .filter-columns::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }
</style>
