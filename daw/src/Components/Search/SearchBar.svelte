<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { debounce } from 'lodash-es';

  export let query: string = '';

  const dispatch = createEventDispatcher();

  interface Suggestion {
    text: string;
    type: 'category' | 'instrument' | 'key' | 'bpm';
    count: number;
  }

  let suggestions: Suggestion[] = [];
  let showSuggestions = false;
  let selectedIndex = -1;
  let inputElement: HTMLInputElement;

  const loadSuggestions = debounce(async (searchText: string) => {
    if (searchText.length < 2) {
      suggestions = [];
      showSuggestions = false;
      return;
    }

    try {
      suggestions = await invoke<Suggestion[]>('get_search_suggestions', {
        query: searchText
      });
      showSuggestions = suggestions.length > 0;
    } catch (error) {
      console.error('Failed to load suggestions:', error);
      suggestions = [];
      showSuggestions = false;
    }
  }, 300);

  function handleInput() {
    selectedIndex = -1;
    loadSuggestions(query);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!showSuggestions) {
      if (event.key === 'Enter') {
        handleSearch();
      }
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, -1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (selectedIndex >= 0) {
        selectSuggestion(suggestions[selectedIndex]);
      } else {
        handleSearch();
      }
    } else if (event.key === 'Escape') {
      showSuggestions = false;
      selectedIndex = -1;
    }
  }

  function selectSuggestion(suggestion: Suggestion) {
    query = suggestion.text;
    showSuggestions = false;
    selectedIndex = -1;
    handleSearch();
  }

  function handleSearch() {
    dispatch('search', { query });
  }

  function handleClear() {
    query = '';
    suggestions = [];
    showSuggestions = false;
    inputElement.focus();
  }

  function getSuggestionIcon(type: string): string {
    const icons: Record<string, string> = {
      'category': '📁',
      'instrument': '🎹',
      'key': '🎵',
      'bpm': '⏱️'
    };
    return icons[type] || '🔍';
  }
</script>

<div class="search-bar">
  <div class="search-input-wrapper">
    <span class="search-icon" aria-hidden="true">🔍</span>
    <input
      bind:this={inputElement}
      type="text"
      class="search-input"
      placeholder="Search MIDI files by name, category, BPM, key, or instrument..."
      bind:value={query}
      on:input={handleInput}
      on:keydown={handleKeydown}
      on:focus={() => query.length >= 2 && suggestions.length > 0 && (showSuggestions = true)}
      aria-label="Search MIDI files"
      aria-autocomplete="list"
      aria-controls="suggestions-list"
      aria-expanded={showSuggestions}
    />
    {#if query}
      <button
        class="clear-btn"
        on:click={handleClear}
        title="Clear search"
        aria-label="Clear search"
      >
        ✕
      </button>
    {/if}
  </div>

  <button class="search-btn" on:click={handleSearch} aria-label="Search">
    Search
  </button>

  {#if showSuggestions}
    <div class="suggestions" id="suggestions-list" role="listbox">
      {#each suggestions as suggestion, index (suggestion.text)}
        <button
          class="suggestion-item"
          class:selected={index === selectedIndex}
          on:click={() => selectSuggestion(suggestion)}
          on:mouseenter={() => selectedIndex = index}
          role="option"
          aria-selected={index === selectedIndex}
        >
          <span class="suggestion-icon">{getSuggestionIcon(suggestion.type)}</span>
          <span class="suggestion-text">{suggestion.text}</span>
          <span class="suggestion-count">{suggestion.count.toLocaleString()}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .search-bar {
    position: relative;
    display: flex;
    gap: 12px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .search-input-wrapper {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 16px;
    font-size: 20px;
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 14px 48px 14px 48px;
    background: #1e1e1e;
    border: 2px solid #3d3d3d;
    border-radius: 8px;
    color: #e0e0e0;
    font-size: 16px;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: #4a9eff;
    background: #252525;
  }

  .clear-btn {
    position: absolute;
    right: 12px;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #3d3d3d;
    border: none;
    border-radius: 50%;
    color: #e0e0e0;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .clear-btn:hover {
    background: #4d4d4d;
    transform: scale(1.1);
  }

  .search-btn {
    padding: 14px 28px;
    background: #4a9eff;
    border: none;
    border-radius: 8px;
    color: white;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .search-btn:hover {
    background: #357abd;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(74, 158, 255, 0.3);
  }

  .suggestions {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 120px;
    max-height: 400px;
    overflow-y: auto;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    z-index: 1000;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    color: #e0e0e0;
    font-size: 14px;
    cursor: pointer;
    transition: background 0.2s;
    text-align: left;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: #3d3d3d;
  }

  .suggestion-icon {
    font-size: 18px;
  }

  .suggestion-text {
    flex: 1;
  }

  .suggestion-count {
    font-size: 12px;
    color: #808080;
    font-weight: 600;
  }

  /* Custom scrollbar */
  .suggestions::-webkit-scrollbar {
    width: 8px;
  }

  .suggestions::-webkit-scrollbar-track {
    background: #2d2d2d;
  }

  .suggestions::-webkit-scrollbar-thumb {
    background: #4d4d4d;
    border-radius: 4px;
  }
</style>
