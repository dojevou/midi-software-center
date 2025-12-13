<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let value: string = '';
  export let placeholder: string = 'Search files...';
  export let suggestions: string[] = [];
  export let showSuggestions: boolean = false;
  export let recentSearches: string[] = [];
  export let isLoading: boolean = false;

  const dispatch = createEventDispatcher<{
    search: { query: string };
    input: { value: string };
    clear: void;
    selectSuggestion: { suggestion: string };
  }>();

  let inputElement: HTMLInputElement;
  let showDropdown = false;
  let selectedIndex = -1;

  $: combinedSuggestions = [
    ...suggestions.map(s => ({ type: 'suggestion' as const, value: s })),
    ...(value.length === 0 ? recentSearches.map(s => ({ type: 'recent' as const, value: s })) : [])
  ];

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    selectedIndex = -1;
    showDropdown = true;
    dispatch('input', { value });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!showDropdown || combinedSuggestions.length === 0) {
      if (event.key === 'Enter') {
        event.preventDefault();
        dispatch('search', { query: value });
      }
      return;
    }

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, combinedSuggestions.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, -1);
        break;
      case 'Enter':
        event.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < combinedSuggestions.length) {
          selectSuggestion(combinedSuggestions[selectedIndex].value);
        } else {
          dispatch('search', { query: value });
        }
        showDropdown = false;
        break;
      case 'Escape':
        showDropdown = false;
        selectedIndex = -1;
        break;
    }
  }

  function selectSuggestion(suggestion: string) {
    value = suggestion;
    showDropdown = false;
    selectedIndex = -1;
    dispatch('selectSuggestion', { suggestion });
    dispatch('search', { query: suggestion });
  }

  function handleClear() {
    value = '';
    showDropdown = false;
    selectedIndex = -1;
    dispatch('clear');
    inputElement?.focus();
  }

  function handleFocus() {
    if (recentSearches.length > 0 || suggestions.length > 0) {
      showDropdown = true;
    }
  }

  function handleBlur(event: FocusEvent) {
    // Delay to allow click on suggestion
    setTimeout(() => {
      const relatedTarget = event.relatedTarget as HTMLElement;
      if (!relatedTarget?.closest('.vip3-search-dropdown')) {
        showDropdown = false;
        selectedIndex = -1;
      }
    }, 150);
  }

  function handleSubmit(event: Event) {
    event.preventDefault();
    dispatch('search', { query: value });
    showDropdown = false;
  }
</script>

<form class="vip3-search-bar" on:submit={handleSubmit}>
  <div class="search-input-wrapper">
    <span class="search-icon" aria-hidden="true">🔍</span>
    <input
      bind:this={inputElement}
      type="text"
      class="search-input"
      {placeholder}
      {value}
      on:input={handleInput}
      on:keydown={handleKeydown}
      on:focus={handleFocus}
      on:blur={handleBlur}
      aria-label="Search"
      aria-autocomplete="list"
      aria-expanded={showDropdown && combinedSuggestions.length > 0}
      aria-controls="search-suggestions"
      autocomplete="off"
    />
    {#if isLoading}
      <span class="loading-spinner" aria-label="Loading">⟳</span>
    {:else if value.length > 0}
      <button
        type="button"
        class="clear-button"
        on:click={handleClear}
        aria-label="Clear search"
      >
        ✕
      </button>
    {/if}
  </div>

  {#if showDropdown && combinedSuggestions.length > 0}
    <div
      id="search-suggestions"
      class="vip3-search-dropdown"
      role="listbox"
      aria-label="Search suggestions"
    >
      {#each combinedSuggestions as item, index (item.value + index)}
        <button
          type="button"
          class="suggestion-item"
          class:selected={index === selectedIndex}
          class:recent={item.type === 'recent'}
          role="option"
          aria-selected={index === selectedIndex}
          on:click={() => selectSuggestion(item.value)}
          on:mouseenter={() => selectedIndex = index}
        >
          <span class="suggestion-icon">
            {item.type === 'recent' ? '🕐' : '🔍'}
          </span>
          <span class="suggestion-text">{item.value}</span>
          {#if item.type === 'recent'}
            <span class="suggestion-badge">Recent</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</form>

<style>
  .vip3-search-bar {
    position: relative;
    width: 100%;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--gray-800, #27272a);
    border: 1px solid var(--gray-700, #3f3f46);
    border-radius: 6px;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .search-input-wrapper:focus-within {
    border-color: var(--primary-500, #3b82f6);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
  }

  .search-icon {
    font-size: 14px;
    color: var(--gray-500, #71717a);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--gray-100, #f4f4f5);
    font-size: 13px;
    outline: none;
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--gray-500, #71717a);
  }

  .loading-spinner {
    font-size: 14px;
    color: var(--primary-500, #3b82f6);
    animation: spin 1s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .clear-button {
    background: transparent;
    border: none;
    color: var(--gray-500, #71717a);
    cursor: pointer;
    padding: 2px;
    font-size: 12px;
    line-height: 1;
    border-radius: 3px;
    transition: color 0.15s ease, background-color 0.15s ease;
    flex-shrink: 0;
  }

  .clear-button:hover {
    color: var(--gray-100, #f4f4f5);
    background: var(--gray-700, #3f3f46);
  }

  .vip3-search-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background: var(--gray-800, #27272a);
    border: 1px solid var(--gray-700, #3f3f46);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    max-height: 240px;
    overflow-y: auto;
    z-index: 100;
  }

  .suggestion-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--gray-300, #d4d4d8);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.15s ease;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-100, #f4f4f5);
  }

  .suggestion-item.recent {
    color: var(--gray-400, #a1a1aa);
  }

  .suggestion-icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .suggestion-text {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .suggestion-badge {
    font-size: 10px;
    color: var(--gray-500, #71717a);
    background: var(--gray-900, #18181b);
    padding: 2px 6px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  /* Custom scrollbar */
  .vip3-search-dropdown::-webkit-scrollbar {
    width: 6px;
  }

  .vip3-search-dropdown::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .vip3-search-dropdown::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 3px;
  }

  .vip3-search-dropdown::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }
</style>
