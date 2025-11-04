<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    categoryChange: string;
    search: string;
    close: void;
  }>();

  export let activeCategory: string = 'general';

  let searchQuery: string = '';

  const categories = [
    { id: 'general', label: 'General' },
    { id: 'audio', label: 'Audio' },
    { id: 'display', label: 'Display' },
    { id: 'keyboard', label: 'Keyboard' },
    { id: 'midi', label: 'MIDI' },
    { id: 'mixer', label: 'Mixer' },
    { id: 'track', label: 'Track' },
    { id: 'importexport', label: 'Import/Export' },
    { id: 'performance', label: 'Performance' },
    { id: 'library', label: 'Library' },
    { id: 'playback', label: 'Playback' },
    { id: 'recording', label: 'Recording' },
    { id: 'sync', label: 'Sync' },
    { id: 'privacy', label: 'Privacy' },
    { id: 'advanced', label: 'Advanced' }
  ];

  function handleCategoryClick(categoryId: string) {
    activeCategory = categoryId;
    dispatch('categoryChange', categoryId);
  }

  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
    dispatch('search', searchQuery);
  }

  function handleClose() {
    dispatch('close');
  }
</script>

<header class="settings-header">
  <div class="header-top">
    <h1 class="title">Settings</h1>
    <button class="close-btn" on:click={handleClose} aria-label="Close settings">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </button>
  </div>

  <div class="search-container">
    <svg class="search-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8"/>
      <path d="M21 21l-4.35-4.35"/>
    </svg>
    <input
      type="text"
      class="search-input"
      placeholder="Search settings..."
      value={searchQuery}
      on:input={handleSearchInput}
    />
  </div>

  <nav class="categories-nav">
    {#each categories as category}
      <button
        class="category-btn"
        class:active={activeCategory === category.id}
        on:click={() => handleCategoryClick(category.id)}
      >
        {category.label}
      </button>
    {/each}
  </nav>
</header>

<style>
  .settings-header {
    background: var(--bg-secondary, #1e1e1e);
    border-bottom: 1px solid var(--border-color, #333);
    padding: 1.5rem;
  }

  .header-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .title {
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin: 0;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary, #a0a0a0);
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    background: var(--bg-hover, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
  }

  .search-container {
    position: relative;
    margin-bottom: 1.5rem;
  }

  .search-icon {
    position: absolute;
    left: 1rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-secondary, #a0a0a0);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 0.75rem 1rem 0.75rem 3rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .search-input::placeholder {
    color: var(--text-secondary, #a0a0a0);
  }

  .categories-nav {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .category-btn {
    padding: 0.5rem 1rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-secondary, #a0a0a0);
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .category-btn:hover {
    background: var(--bg-hover, #2a2a2a);
    border-color: var(--border-hover, #444);
    color: var(--text-primary, #e0e0e0);
  }

  .category-btn.active {
    background: var(--accent-color, #4a9eff);
    border-color: var(--accent-color, #4a9eff);
    color: #fff;
  }
</style>
