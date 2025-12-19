<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly } from 'svelte/transition';
  import { toastStore } from '$lib/stores/toastStore';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';

  export let visible = false;
  export let x = 0;
  export let y = 0;
  export let fileId: number;
  export let filename: string;
  export let isFavorite = false;

  const dispatch = createEventDispatcher<{
    close: void;
    addToSequencer: number;
    addToCollection: number;
    showDetails: number;
  }>();

  function close() {
    visible = false;
    dispatch('close');
  }

  async function handleAddToSequencer() {
    dispatch('addToSequencer', fileId);
    close();
  }

  async function handleToggleFavorite() {
    try {
      const newStatus = await Vip3BrowserApi.toggleFavorite(fileId);
      isFavorite = newStatus;
      toastStore.success(newStatus ? `Added "${filename}" to favorites` : `Removed "${filename}" from favorites`);
    } catch (error) {
      toastStore.error('Failed to update favorite status');
    }
    close();
  }

  async function handleAddToCollection() {
    dispatch('addToCollection', fileId);
    close();
  }

  function handleShowDetails() {
    dispatch('showDetails', fileId);
    close();
  }

  function handleCopyFilename() {
    navigator.clipboard.writeText(filename);
    toastStore.info(`Copied "${filename}" to clipboard`);
    close();
  }

  // Close on click outside
  function handleClickOutside(event: MouseEvent) {
    if (visible) {
      close();
    }
  }

  // Close on Escape
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && visible) {
      close();
    }
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

{#if visible}
  <div
    class="context-menu"
    style="left: {x}px; top: {y}px"
    transition:fly={{ y: -10, duration: 150 }}
    role="menu"
    on:click|stopPropagation
  >
    <button class="menu-item" role="menuitem" on:click={handleAddToSequencer}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14"/>
      </svg>
      Add to Sequencer
    </button>

    <button class="menu-item" role="menuitem" on:click={handleToggleFavorite}>
      <svg viewBox="0 0 24 24" fill={isFavorite ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
        <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
      </svg>
      {isFavorite ? 'Remove from Favorites' : 'Add to Favorites'}
    </button>

    <button class="menu-item" role="menuitem" on:click={handleAddToCollection}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      Add to Collection...
    </button>

    <div class="separator"></div>

    <button class="menu-item" role="menuitem" on:click={handleShowDetails}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <path d="M12 16v-4M12 8h.01"/>
      </svg>
      Show Details
    </button>

    <button class="menu-item" role="menuitem" on:click={handleCopyFilename}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
      </svg>
      Copy Filename
    </button>
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 10000;
    background: #252525;
    border: 1px solid #444;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    padding: 6px 0;
    min-width: 200px;
    overflow: hidden;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 14px;
    background: none;
    border: none;
    color: #fff;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .menu-item:hover {
    background: rgba(0, 122, 255, 0.15);
  }

  .menu-item:focus {
    outline: none;
    background: rgba(0, 122, 255, 0.2);
  }

  .menu-item svg {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: #999;
  }

  .menu-item:hover svg {
    color: #007aff;
  }

  .separator {
    height: 1px;
    background: #444;
    margin: 6px 0;
  }
</style>
