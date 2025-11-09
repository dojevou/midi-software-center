<script lang="ts">
  /**
   * Tag Suggestions Component
   *
   * ARCHETYPE: TASK-O-MATIC (UI Component)
   * Purpose: Display and manage tag suggestions with confidence scores
   *
   * Features:
   * - Tag cloud display with category colors
   * - Confidence bars for each tag
   * - Priority-based sorting (genre first, world last)
   * - Multi-select tag acceptance/rejection
   * - Search/filter by tag name or category
   * - Auto-complete suggestions
   */

  import { onMount } from 'svelte';
  import { slide, fade } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import {
    tagSuggestionsState,
    filteredSuggestions,
    tagsByCategory,
    selectedSuggestions,
    hasSelection,
    suggestionStats,
    fetchTagSuggestions,
    generateTagSuggestions,
    acceptSuggestion,
    rejectSuggestion,
    batchProcessSuggestions,
    toggleSuggestionSelection,
    selectAllSuggestions,
    clearSelection,
    selectByCategory,
    selectByConfidence,
    setSearchQuery,
    toggleCategoryFilter,
    resetFilters,
    initializeTagSuggestionsStore
  } from '../stores/tagSuggestions';
  import {
    getCategoryColor,
    formatConfidence,
    getConfidenceLevel,
    getConfidenceLevelColor,
    getDetectionMethodIcon,
    calculateConfidenceOpacity
  } from '../utils/tagUtils';
  import type { TagSuggestion } from '../types/tagSuggestions';

  // =============================================================================
  // PROPS
  // =============================================================================

  export let fileId: number | null = null;
  export let showBatchActions: boolean = true;
  export let allowGenerate: boolean = true;
  export let maxHeight: string = '600px';
  export let compact: boolean = false;

  // =============================================================================
  // LOCAL STATE
  // =============================================================================

  let searchQuery = '';
  let selectedCategories: Set<string> = new Set();
  let minConfidenceFilter = 0.60;
  let sortBy: 'priority' | 'confidence' | 'name' = 'priority';
  let viewMode: 'list' | 'cloud' = 'list';
  let showOnlyPending = true;

  // =============================================================================
  // REACTIVE STATEMENTS
  // =============================================================================

  $: state = $tagSuggestionsState;
  $: suggestions = $filteredSuggestions;
  $: categories = $tagsByCategory;
  $: selected = $selectedSuggestions;
  $: stats = $suggestionStats;
  $: isLoading = state.loading;
  $: error = state.error;
  $: hasAnySelection = $hasSelection;

  // Group suggestions by tag for cloud view
  $: suggestionsByTag = suggestions.reduce((acc, suggestion) => {
    const tagName = suggestion.suggestedTag.name;
    if (!acc.has(tagName)) {
      acc.set(tagName, []);
    }
    acc.get(tagName)!.push(suggestion);
    return acc;
  }, new Map<string, TagSuggestion[]>());

  // =============================================================================
  // FUNCTIONS
  // =============================================================================

  async function handleGenerate() {
    if (!fileId) return;

    try {
      await generateTagSuggestions(fileId);
    } catch (err) {
      console.error('Failed to generate tag suggestions:', err);
    }
  }

  async function handleAccept(suggestionId: number) {
    try {
      await acceptSuggestion(suggestionId);
    } catch (err) {
      console.error('Failed to accept suggestion:', err);
    }
  }

  async function handleReject(suggestionId: number) {
    try {
      await rejectSuggestion(suggestionId);
    } catch (err) {
      console.error('Failed to reject suggestion:', err);
    }
  }

  async function handleBatchAccept() {
    if (selected.length === 0) return;

    try {
      await batchProcessSuggestions(
        selected.map((s) => s.id),
        'accept'
      );
    } catch (err) {
      console.error('Failed to batch accept suggestions:', err);
    }
  }

  async function handleBatchReject() {
    if (selected.length === 0) return;

    try {
      await batchProcessSuggestions(
        selected.map((s) => s.id),
        'reject'
      );
    } catch (err) {
      console.error('Failed to batch reject suggestions:', err);
    }
  }

  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
    setSearchQuery(searchQuery);
  }

  function handleCategoryToggle(category: string) {
    toggleCategoryFilter(category);

    // Update local state for visual feedback
    if (selectedCategories.has(category)) {
      selectedCategories.delete(category);
    } else {
      selectedCategories.add(category);
    }
    selectedCategories = new Set(selectedCategories);
  }

  function handleSelectAll() {
    selectAllSuggestions();
  }

  function handleClearSelection() {
    clearSelection();
  }

  function handleSelectHighConfidence() {
    selectByConfidence(0.85);
  }

  function handleToggleSelection(suggestionId: number) {
    toggleSuggestionSelection(suggestionId);
  }

  function getConfidenceBarWidth(confidence: number): number {
    return confidence * 100;
  }

  // =============================================================================
  // LIFECYCLE
  // =============================================================================

  onMount(async () => {
    await initializeTagSuggestionsStore();

    if (fileId) {
      await fetchTagSuggestions(fileId);
    }
  });
</script>

<div class="tag-suggestions" class:compact style="--max-height: {maxHeight}">
  <!-- Header -->
  <div class="header">
    <div class="title-section">
      <h3 class="title">Tag Suggestions</h3>
      {#if stats.total > 0}
        <span class="badge" transition:fade>{stats.total}</span>
      {/if}
    </div>

    <div class="header-actions">
      {#if allowGenerate && fileId}
        <button
          class="btn btn-secondary btn-sm"
          on:click={handleGenerate}
          disabled={isLoading}
          aria-label="Generate tag suggestions"
        >
          <span class="icon">🤖</span>
          Generate
        </button>
      {/if}

      <div class="view-toggle">
        <button
          class="toggle-btn"
          class:active={viewMode === 'list'}
          on:click={() => (viewMode = 'list')}
          aria-label="List view"
          title="List view"
        >
          <span>☰</span>
        </button>
        <button
          class="toggle-btn"
          class:active={viewMode === 'cloud'}
          on:click={() => (viewMode = 'cloud')}
          aria-label="Cloud view"
          title="Cloud view"
        >
          <span>☁️</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Search & Filters -->
  <div class="filters-section">
    <div class="search-box">
      <span class="search-icon">🔍</span>
      <input
        type="text"
        class="search-input"
        placeholder="Search tags..."
        value={searchQuery}
        on:input={handleSearchInput}
        aria-label="Search tags"
      />
      {#if searchQuery}
        <button
          class="clear-search"
          on:click={() => {
            searchQuery = '';
            setSearchQuery('');
          }}
          aria-label="Clear search"
        >
          ✕
        </button>
      {/if}
    </div>

    <!-- Confidence Filter -->
    <div class="confidence-filter">
      <label for="confidence-slider" class="filter-label">
        Min Confidence: {formatConfidence(minConfidenceFilter)}
      </label>
      <input
        id="confidence-slider"
        type="range"
        class="slider"
        min="0.5"
        max="0.95"
        step="0.05"
        bind:value={minConfidenceFilter}
        aria-label="Minimum confidence filter"
      />
    </div>

    <!-- Category Filters -->
    {#if state.categories.length > 0}
      <div class="category-filters">
        {#each state.categories as category}
          <button
            class="category-chip"
            class:active={selectedCategories.has(category.name)}
            style="--category-color: {category.color}"
            on:click={() => handleCategoryToggle(category.name)}
            aria-label="Filter by {category.name}"
          >
            <span class="chip-dot"></span>
            {category.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Stats Bar -->
  {#if stats.total > 0}
    <div class="stats-bar" transition:slide={{ duration: 300, easing: quintOut }}>
      <div class="stat">
        <span class="stat-icon" style="color: #4caf50">●</span>
        <span class="stat-label">High:</span>
        <span class="stat-value">{stats.highConfidence}</span>
      </div>
      <div class="stat">
        <span class="stat-icon" style="color: #ff9800">●</span>
        <span class="stat-label">Medium:</span>
        <span class="stat-value">{stats.mediumConfidence}</span>
      </div>
      <div class="stat">
        <span class="stat-icon" style="color: #f44336">●</span>
        <span class="stat-label">Low:</span>
        <span class="stat-value">{stats.lowConfidence}</span>
      </div>
    </div>
  {/if}

  <!-- Batch Actions -->
  {#if showBatchActions && suggestions.length > 0}
    <div class="batch-actions" transition:slide={{ duration: 300, easing: quintOut }}>
      <div class="selection-info">
        {#if hasAnySelection}
          <span class="selected-count">{selected.length} selected</span>
          <button class="link-btn" on:click={handleClearSelection}>Clear</button>
        {:else}
          <button class="link-btn" on:click={handleSelectAll}>Select All</button>
          <button class="link-btn" on:click={handleSelectHighConfidence}>
            Select High Confidence
          </button>
        {/if}
      </div>

      {#if hasAnySelection}
        <div class="action-buttons">
          <button class="btn btn-success btn-sm" on:click={handleBatchAccept}>
            <span class="icon">✓</span>
            Accept ({selected.length})
          </button>
          <button class="btn btn-danger btn-sm" on:click={handleBatchReject}>
            <span class="icon">✕</span>
            Reject ({selected.length})
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Error Message -->
  {#if error}
    <div class="error-message" transition:slide={{ duration: 300, easing: quintOut }}>
      <span class="error-icon">⚠️</span>
      <span>{error}</span>
    </div>
  {/if}

  <!-- Loading State -->
  {#if isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading suggestions...</p>
    </div>
  {:else if suggestions.length === 0}
    <!-- Empty State -->
    <div class="empty-state">
      <div class="empty-icon">🏷️</div>
      <p class="empty-title">No tag suggestions</p>
      <p class="empty-description">
        {#if fileId && allowGenerate}
          Click "Generate" to analyze this file and suggest tags.
        {:else}
          Try adjusting your filters or generate new suggestions.
        {/if}
      </p>
    </div>
  {:else if viewMode === 'list'}
    <!-- List View -->
    <div class="suggestions-list" style="max-height: {maxHeight}">
      {#each Array.from(categories.entries()) as [categoryName, categoryTags]}
        <div class="category-section" transition:slide={{ duration: 300, easing: quintOut }}>
          <div
            class="category-header"
            style="border-left-color: {getCategoryColor(categoryName)}"
          >
            <h4 class="category-name">{categoryName}</h4>
            <span class="category-count">{categoryTags.length}</span>
          </div>

          <div class="suggestions-grid">
            {#each suggestions.filter((s) => s.suggestedTag.category === categoryName) as suggestion (suggestion.id)}
              <div
                class="suggestion-card"
                class:selected={state.selectedSuggestionIds.includes(suggestion.id)}
                class:accepted={suggestion.isAccepted === true}
                class:rejected={suggestion.isAccepted === false}
                transition:fade={{ duration: 200 }}
              >
                <!-- Selection Checkbox -->
                <label class="checkbox-wrapper">
                  <input
                    type="checkbox"
                    class="checkbox"
                    checked={state.selectedSuggestionIds.includes(suggestion.id)}
                    on:change={() => handleToggleSelection(suggestion.id)}
                    aria-label="Select {suggestion.suggestedTag.name}"
                  />
                </label>

                <!-- Tag Info -->
                <div class="tag-info">
                  <div class="tag-header">
                    <span
                      class="tag-name"
                      style="color: {getCategoryColor(suggestion.suggestedTag.category)}"
                    >
                      {suggestion.suggestedTag.name}
                    </span>
                    <span class="detection-method" title={suggestion.suggestedTag.detectionMethod}>
                      {getDetectionMethodIcon(suggestion.suggestedTag.detectionMethod)}
                    </span>
                  </div>

                  <!-- Confidence Bar -->
                  <div class="confidence-section">
                    <div class="confidence-label">
                      <span>Confidence:</span>
                      <span class="confidence-value" style="color: {getConfidenceLevelColor(suggestion.confidence)}">
                        {formatConfidence(suggestion.confidence)}
                      </span>
                    </div>
                    <div class="confidence-bar">
                      <div
                        class="confidence-fill"
                        style="width: {getConfidenceBarWidth(suggestion.confidence)}%; background-color: {getConfidenceLevelColor(suggestion.confidence)}; opacity: {calculateConfidenceOpacity(suggestion.confidence)}"
                      ></div>
                    </div>
                  </div>

                  <!-- Source -->
                  <div class="tag-meta">
                    <span class="meta-label">Source:</span>
                    <span class="meta-value">{suggestion.source}</span>
                  </div>
                </div>

                <!-- Actions -->
                {#if suggestion.isAccepted === undefined || suggestion.isAccepted === null}
                  <div class="suggestion-actions">
                    <button
                      class="action-btn accept-btn"
                      on:click={() => handleAccept(suggestion.id)}
                      aria-label="Accept tag"
                      title="Accept this tag"
                    >
                      ✓
                    </button>
                    <button
                      class="action-btn reject-btn"
                      on:click={() => handleReject(suggestion.id)}
                      aria-label="Reject tag"
                      title="Reject this tag"
                    >
                      ✕
                    </button>
                  </div>
                {/if}

                <!-- Status Badge -->
                {#if suggestion.isAccepted === true}
                  <div class="status-badge accepted">Accepted</div>
                {:else if suggestion.isAccepted === false}
                  <div class="status-badge rejected">Rejected</div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <!-- Cloud View -->
    <div class="tag-cloud" style="max-height: {maxHeight}">
      {#each Array.from(suggestionsByTag.entries()) as [tagName, tagSuggestions]}
        {@const avgConfidence =
          tagSuggestions.reduce((sum, s) => sum + s.confidence, 0) / tagSuggestions.length}
        {@const category = tagSuggestions[0].suggestedTag.category}
        {@const fontSize = 12 + avgConfidence * 20}

        <button
          class="cloud-tag"
          style="
            font-size: {fontSize}px;
            color: {getCategoryColor(category)};
            opacity: {calculateConfidenceOpacity(avgConfidence)};
          "
          title="{tagName} - {formatConfidence(avgConfidence)} ({tagSuggestions.length} files)"
        >
          {tagName}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tag-suggestions {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    background: var(--color-surface, #2d2d2d);
    border-radius: 8px;
    border: 1px solid var(--color-border, #3d3d3d);
  }

  .tag-suggestions.compact {
    padding: 12px;
    gap: 12px;
  }

  /* Header */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
  }

  .title-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 8px;
    background: var(--color-primary, #3b82f6);
    color: white;
    font-size: 12px;
    font-weight: 600;
    border-radius: 12px;
  }

  .header-actions {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .view-toggle {
    display: flex;
    gap: 4px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 6px;
    padding: 2px;
  }

  .toggle-btn {
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--color-text-secondary, #a3a3a3);
    cursor: pointer;
    transition: all 0.2s;
  }

  .toggle-btn:hover {
    color: var(--color-text, #e5e5e5);
  }

  .toggle-btn.active {
    background: var(--color-primary, #3b82f6);
    color: white;
  }

  /* Filters Section */
  .filters-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: var(--color-bg, #1e1e1e);
    border: 1px solid var(--color-border, #3d3d3d);
    border-radius: 6px;
    transition: border-color 0.2s;
  }

  .search-box:focus-within {
    border-color: var(--color-primary, #3b82f6);
  }

  .search-icon {
    font-size: 16px;
    opacity: 0.6;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--color-text, #e5e5e5);
    font-size: 14px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--color-text-secondary, #a3a3a3);
  }

  .clear-search {
    padding: 4px 8px;
    background: transparent;
    border: none;
    color: var(--color-text-secondary, #a3a3a3);
    cursor: pointer;
    font-size: 14px;
    transition: color 0.2s;
  }

  .clear-search:hover {
    color: var(--color-text, #e5e5e5);
  }

  /* Confidence Filter */
  .confidence-filter {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .filter-label {
    font-size: 12px;
    color: var(--color-text-secondary, #a3a3a3);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .slider {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--color-bg, #1e1e1e);
    outline: none;
    -webkit-appearance: none;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-primary, #3b82f6);
    cursor: pointer;
    transition: transform 0.2s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-primary, #3b82f6);
    cursor: pointer;
    border: none;
    transition: transform 0.2s;
  }

  .slider::-moz-range-thumb:hover {
    transform: scale(1.2);
  }

  /* Category Filters */
  .category-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .category-chip {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--color-bg, #1e1e1e);
    border: 1px solid var(--color-border, #3d3d3d);
    border-radius: 16px;
    color: var(--color-text-secondary, #a3a3a3);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .category-chip:hover {
    border-color: var(--category-color);
    color: var(--color-text, #e5e5e5);
  }

  .category-chip.active {
    background: var(--category-color);
    border-color: var(--category-color);
    color: white;
  }

  .chip-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--category-color);
  }

  /* Stats Bar */
  .stats-bar {
    display: flex;
    gap: 24px;
    padding: 12px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 6px;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }

  .stat-icon {
    font-size: 12px;
  }

  .stat-label {
    color: var(--color-text-secondary, #a3a3a3);
  }

  .stat-value {
    color: var(--color-text, #e5e5e5);
    font-weight: 600;
  }

  /* Batch Actions */
  .batch-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 12px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 6px;
  }

  .selection-info {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }

  .selected-count {
    color: var(--color-text, #e5e5e5);
    font-weight: 600;
  }

  .link-btn {
    padding: 0;
    background: transparent;
    border: none;
    color: var(--color-primary, #3b82f6);
    font-size: 13px;
    cursor: pointer;
    text-decoration: underline;
    transition: opacity 0.2s;
  }

  .link-btn:hover {
    opacity: 0.8;
  }

  .action-buttons {
    display: flex;
    gap: 8px;
  }

  /* Buttons */
  .btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-sm {
    padding: 6px 12px;
    font-size: 13px;
  }

  .btn-secondary {
    background: var(--color-bg, #1e1e1e);
    color: var(--color-text, #e5e5e5);
    border: 1px solid var(--color-border, #3d3d3d);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-surface, #2d2d2d);
    border-color: var(--color-primary, #3b82f6);
  }

  .btn-success {
    background: #4caf50;
    color: white;
  }

  .btn-success:hover:not(:disabled) {
    background: #45a049;
  }

  .btn-danger {
    background: #f44336;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #da190b;
  }

  .icon {
    font-size: 16px;
    line-height: 1;
  }

  /* Error Message */
  .error-message {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid #f44336;
    border-radius: 6px;
    color: #f44336;
    font-size: 14px;
  }

  .error-icon {
    font-size: 20px;
  }

  /* Loading State */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    gap: 16px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--color-border, #3d3d3d);
    border-top-color: var(--color-primary, #3b82f6);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    gap: 12px;
  }

  .empty-icon {
    font-size: 48px;
    opacity: 0.5;
  }

  .empty-title {
    margin: 0;
    font-size: 16px;
    color: var(--color-text, #e5e5e5);
    font-weight: 600;
  }

  .empty-description {
    margin: 0;
    font-size: 14px;
    color: var(--color-text-secondary, #a3a3a3);
    text-align: center;
    max-width: 400px;
  }

  /* Suggestions List */
  .suggestions-list {
    display: flex;
    flex-direction: column;
    gap: 20px;
    overflow-y: auto;
  }

  /* Category Section */
  .category-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .category-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-left: 12px;
    border-left: 3px solid;
  }

  .category-name {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    text-transform: capitalize;
  }

  .category-count {
    font-size: 12px;
    color: var(--color-text-secondary, #a3a3a3);
  }

  /* Suggestions Grid */
  .suggestions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 12px;
  }

  /* Suggestion Card */
  .suggestion-card {
    position: relative;
    display: flex;
    gap: 12px;
    padding: 14px;
    background: var(--color-bg, #1e1e1e);
    border: 1px solid var(--color-border, #3d3d3d);
    border-radius: 8px;
    transition: all 0.2s;
  }

  .suggestion-card:hover {
    border-color: var(--color-primary, #3b82f6);
  }

  .suggestion-card.selected {
    background: rgba(59, 130, 246, 0.1);
    border-color: var(--color-primary, #3b82f6);
  }

  .suggestion-card.accepted {
    opacity: 0.6;
    border-color: #4caf50;
  }

  .suggestion-card.rejected {
    opacity: 0.4;
    border-color: #f44336;
  }

  /* Checkbox */
  .checkbox-wrapper {
    display: flex;
    align-items: flex-start;
    padding-top: 2px;
  }

  .checkbox {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  /* Tag Info */
  .tag-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .tag-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tag-name {
    font-size: 15px;
    font-weight: 600;
    line-height: 1.2;
  }

  .detection-method {
    font-size: 16px;
    opacity: 0.7;
  }

  /* Confidence Section */
  .confidence-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .confidence-label {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--color-text-secondary, #a3a3a3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .confidence-value {
    font-weight: 600;
  }

  .confidence-bar {
    height: 6px;
    background: var(--color-surface, #2d2d2d);
    border-radius: 3px;
    overflow: hidden;
  }

  .confidence-fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  /* Tag Meta */
  .tag-meta {
    display: flex;
    gap: 6px;
    font-size: 12px;
  }

  .meta-label {
    color: var(--color-text-secondary, #a3a3a3);
  }

  .meta-value {
    color: var(--color-text, #e5e5e5);
    font-weight: 500;
  }

  /* Suggestion Actions */
  .suggestion-actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .action-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: 6px;
    font-size: 16px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .accept-btn {
    background: rgba(76, 175, 80, 0.1);
    color: #4caf50;
  }

  .accept-btn:hover {
    background: #4caf50;
    color: white;
  }

  .reject-btn {
    background: rgba(244, 67, 54, 0.1);
    color: #f44336;
  }

  .reject-btn:hover {
    background: #f44336;
    color: white;
  }

  /* Status Badge */
  .status-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-badge.accepted {
    background: rgba(76, 175, 80, 0.2);
    color: #4caf50;
  }

  .status-badge.rejected {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
  }

  /* Tag Cloud */
  .tag-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    padding: 24px;
    align-items: center;
    justify-content: center;
    overflow-y: auto;
  }

  .cloud-tag {
    padding: 8px 16px;
    background: transparent;
    border: none;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border-radius: 4px;
  }

  .cloud-tag:hover {
    transform: scale(1.1);
    background: rgba(255, 255, 255, 0.05);
  }

  /* Scrollbar */
  .suggestions-list::-webkit-scrollbar,
  .tag-cloud::-webkit-scrollbar {
    width: 6px;
  }

  .suggestions-list::-webkit-scrollbar-track,
  .tag-cloud::-webkit-scrollbar-track {
    background: transparent;
  }

  .suggestions-list::-webkit-scrollbar-thumb,
  .tag-cloud::-webkit-scrollbar-thumb {
    background: var(--color-border, #3d3d3d);
    border-radius: 3px;
  }

  .suggestions-list::-webkit-scrollbar-thumb:hover,
  .tag-cloud::-webkit-scrollbar-thumb:hover {
    background: #525252;
  }
</style>
