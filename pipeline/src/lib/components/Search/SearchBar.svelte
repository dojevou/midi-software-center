<script lang="ts">
  /**
   * SearchBar Component
   * Archetype: CONTAINER (UI composition, delegates I/O to searchStore)
   *
   * Responsibilities:
   * - Handle user input and interactions
   * - Manage local UI state (debouncing, input focus)
   * - Call searchStore methods (MANAGER) for I/O
   * - Display loading and error states from store
   */

  import { Search, X } from 'lucide-svelte';
  import { searchStore } from '$lib/stores/search';

  // Local UI state (not persisted)
  let query = '';
  let debounceTimeout: number | null = null;
  let inputElement: HTMLInputElement;

  // Subscribe to store state for reactive display
  $: loading = $searchStore.loading;
  $: error = $searchStore.error;
  $: hasResults = $searchStore.results.length > 0;

  /**
   * Debounced search - waits 300ms after user stops typing
   */
  function handleInput() {
    if (debounceTimeout !== null) {
      clearTimeout(debounceTimeout);
    }

    debounceTimeout = window.setTimeout(() => {
      if (query.trim()) {
        handleSearch();
      } else {
        searchStore.clear();
      }
    }, 300);
  }

  /**
   * Trigger search via store (MANAGER handles I/O)
   */
  async function handleSearch() {
    if (!query.trim()) {
      searchStore.clear();
      return;
    }

    // Delegate I/O to store (MANAGER)
    await searchStore.search(query.trim());
  }

  /**
   * Handle keyboard shortcuts
   * - Enter: Immediate search (bypass debounce)
   * - Escape: Clear search
   */
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      if (debounceTimeout !== null) {
        clearTimeout(debounceTimeout);
        debounceTimeout = null;
      }
      handleSearch();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClear();
    }
  }

  /**
   * Clear search query and results
   */
  function handleClear() {
    query = '';
    if (debounceTimeout !== null) {
      clearTimeout(debounceTimeout);
      debounceTimeout = null;
    }
    searchStore.clear();
    inputElement?.focus();
  }

  /**
   * Dismiss error message
   */
  function dismissError() {
    // Just clear the error in the store
    searchStore.search(query.trim());
  }
</script>

<div class="search-bar">
  <div class="search-container">
    <!-- Search Input -->
    <div class="search-input-wrapper">
      <Search
        class="search-icon"
        size={20}
        aria-hidden="true"
      />

      <input
        bind:this={inputElement}
        type="text"
        class="search-input"
        placeholder="Search MIDI files by name, BPM, key..."
        bind:value={query}
        on:input={handleInput}
        on:keydown={handleKeyPress}
        disabled={loading}
        aria-label="Search MIDI files"
      />

      <!-- Clear Button -->
      {#if query}
        <button
          class="clear-button"
          on:click={handleClear}
          title="Clear search (Esc)"
          aria-label="Clear search"
          type="button"
        >
          <X size={16} />
        </button>
      {/if}
    </div>

    <!-- Search Button -->
    <button
      class="search-button"
      on:click={handleSearch}
      disabled={loading || !query.trim()}
      aria-label="Search"
      type="button"
    >
      {#if loading}
        <span class="spinner" aria-hidden="true"></span>
        <span>Searching...</span>
      {:else}
        <span>Search</span>
      {/if}
    </button>
  </div>

  <!-- Error Display -->
  {#if error}
    <div class="error-message" role="alert">
      <span class="error-text">{error}</span>
      <button
        class="error-dismiss"
        on:click={dismissError}
        aria-label="Dismiss error"
        type="button"
      >
        <X size={16} />
      </button>
    </div>
  {/if}

  <!-- Results Count -->
  {#if hasResults && !loading && !error}
    <div class="results-info">
      Found {$searchStore.total.toLocaleString()} file{$searchStore.total !== 1 ? 's' : ''}
    </div>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    width: 100%;
    background: var(--color-surface, #1e1e1e);
    border-bottom: 1px solid var(--color-border, #3d3d3d);
  }

  .search-container {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    height: 60px;
  }

  .search-input-wrapper {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 14px;
    color: var(--color-text-tertiary, #808080);
    pointer-events: none;
    z-index: 1;
  }

  .search-input {
    width: 100%;
    height: 42px;
    padding: 0 40px 0 44px;
    background: var(--color-bg-secondary, #252525);
    border: 2px solid var(--color-border, #3d3d3d);
    border-radius: 8px;
    color: var(--color-text-primary, #e0e0e0);
    font-size: 14px;
    font-family: inherit;
    transition: all 0.2s ease;
  }

  .search-input::placeholder {
    color: var(--color-text-tertiary, #808080);
  }

  .search-input:hover:not(:disabled) {
    border-color: var(--color-border-hover, #4d4d4d);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-accent, #4a9eff);
    background: var(--color-bg-primary, #1a1a1a);
  }

  .search-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .clear-button {
    position: absolute;
    right: 10px;
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-tertiary, #3d3d3d);
    border: none;
    border-radius: 50%;
    color: var(--color-text-secondary, #b0b0b0);
    cursor: pointer;
    transition: all 0.15s ease;
    z-index: 2;
  }

  .clear-button:hover {
    background: var(--color-bg-hover, #4d4d4d);
    color: var(--color-text-primary, #e0e0e0);
    transform: scale(1.1);
  }

  .clear-button:active {
    transform: scale(0.95);
  }

  .search-button {
    height: 42px;
    min-width: 120px;
    padding: 0 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background: var(--color-accent, #4a9eff);
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .search-button:hover:not(:disabled) {
    background: var(--color-accent-hover, #357abd);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(74, 158, 255, 0.3);
  }

  .search-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .search-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-message {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin: 0 16px 12px 16px;
    padding: 12px 16px;
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid var(--color-error, #f44336);
    border-radius: 6px;
    animation: slideDown 0.2s ease;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .error-text {
    flex: 1;
    color: var(--color-error, #ff6b6b);
    font-size: 13px;
    font-weight: 500;
  }

  .error-dismiss {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--color-error, #f44336);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .error-dismiss:hover {
    background: rgba(244, 67, 54, 0.2);
    transform: scale(1.1);
  }

  .results-info {
    padding: 0 16px 12px 16px;
    color: var(--color-text-secondary, #b0b0b0);
    font-size: 12px;
    font-weight: 500;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  /* Accessibility */
  @media (prefers-reduced-motion: reduce) {
    .spinner,
    .error-message,
    .results-info {
      animation: none;
    }

    .search-button,
    .clear-button,
    .search-input {
      transition: none;
    }
  }

  /* Responsive */
  @media (max-width: 640px) {
    .search-container {
      flex-direction: column;
      height: auto;
      gap: 8px;
    }

    .search-button {
      width: 100%;
      min-width: auto;
    }
  }
</style>
