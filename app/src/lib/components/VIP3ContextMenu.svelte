<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import type { FileDetails } from '$lib/types';

  export let file: FileDetails | null = null;
  export let x: number = 0;
  export let y: number = 0;
  export let visible: boolean = false;

  const dispatch = createEventDispatcher<{
    preview: { file: FileDetails };
    addToSequencer: { file: FileDetails };
    addToTrack: { file: FileDetails; trackIndex: number };
    toggleFavorite: { file: FileDetails };
    editTags: { file: FileDetails };
    showInFolder: { file: FileDetails };
    copyPath: { file: FileDetails };
    delete: { file: FileDetails };
    close: void;
  }>();

  let menuElement: HTMLDivElement;

  // Available tracks for "Add to Track" submenu
  const tracks = [
    { index: 0, name: 'Track 1 - Bass' },
    { index: 1, name: 'Track 2 - Drums' },
    { index: 2, name: 'Track 3 - Keys' },
    { index: 3, name: 'Track 4 - Melody' },
  ];

  // Adjust position to keep menu on screen
  $: if (visible && menuElement) {
    const rect = menuElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    if (x + rect.width > viewportWidth) {
      x = viewportWidth - rect.width - 10;
    }
    if (y + rect.height > viewportHeight) {
      y = viewportHeight - rect.height - 10;
    }
  }

  function handleAction(
    action: 'preview' | 'addToSequencer' | 'toggleFavorite' | 'editTags' | 'showInFolder' | 'copyPath' | 'delete'
  ) {
    if (file) {
      dispatch(action, { file });
    }
    dispatch('close');
  }

  function handleAddToTrack(trackIndex: number) {
    if (file) {
      dispatch('addToTrack', { file, trackIndex });
    }
    dispatch('close');
  }

  function handleClickOutside(event: MouseEvent) {
    if (visible && menuElement && !menuElement.contains(event.target as Node)) {
      dispatch('close');
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      dispatch('close');
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
    document.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if visible && file}
  <div
    bind:this={menuElement}
    class="context-menu"
    style="left: {x}px; top: {y}px;"
    role="menu"
    aria-label="File actions"
  >
    <button type="button" class="menu-item" on:click={() => handleAction('preview')} role="menuitem">
      <span class="menu-icon">▶</span>
      <span class="menu-label">Preview</span>
      <span class="menu-shortcut">Space</span>
    </button>

    <div class="menu-divider" role="separator"></div>

    <button type="button" class="menu-item" on:click={() => handleAction('addToSequencer')} role="menuitem">
      <span class="menu-icon">➕</span>
      <span class="menu-label">Add to Sequencer</span>
    </button>

    <div class="menu-item submenu-trigger" role="menuitem" aria-haspopup="true">
      <span class="menu-icon">➕</span>
      <span class="menu-label">Add to Track</span>
      <span class="submenu-arrow">▶</span>
      <div class="submenu" role="menu">
        {#each tracks as track}
          <button
            type="button"
            class="menu-item"
            on:click={() => handleAddToTrack(track.index)}
            role="menuitem"
          >
            <span class="menu-label">{track.name}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="menu-divider" role="separator"></div>

    <button type="button" class="menu-item" on:click={() => handleAction('toggleFavorite')} role="menuitem">
      <span class="menu-icon">{file.is_favorite ? '★' : '☆'}</span>
      <span class="menu-label">{file.is_favorite ? 'Remove from Favorites' : 'Add to Favorites'}</span>
    </button>

    <button type="button" class="menu-item" on:click={() => handleAction('editTags')} role="menuitem">
      <span class="menu-icon">🏷</span>
      <span class="menu-label">Edit Tags...</span>
    </button>

    <div class="menu-divider" role="separator"></div>

    <button type="button" class="menu-item" on:click={() => handleAction('showInFolder')} role="menuitem">
      <span class="menu-icon">📁</span>
      <span class="menu-label">Show in Folder</span>
    </button>

    <button type="button" class="menu-item" on:click={() => handleAction('copyPath')} role="menuitem">
      <span class="menu-icon">📋</span>
      <span class="menu-label">Copy Path</span>
    </button>

    <div class="menu-divider" role="separator"></div>

    <button type="button" class="menu-item danger" on:click={() => handleAction('delete')} role="menuitem">
      <span class="menu-icon">🗑</span>
      <span class="menu-label">Delete</span>
    </button>
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 10000;
    min-width: 200px;
    background: var(--gray-800, #27272a);
    border: 1px solid var(--gray-700, #3f3f46);
    border-radius: 8px;
    box-shadow:
      0 10px 25px rgba(0, 0, 0, 0.4),
      0 4px 10px rgba(0, 0, 0, 0.3);
    padding: 4px 0;
    overflow: hidden;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--gray-200, #e4e4e7);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.1s ease;
    position: relative;
  }

  .menu-item:hover {
    background: var(--gray-700, #3f3f46);
  }

  .menu-item:focus {
    outline: none;
    background: var(--gray-700, #3f3f46);
  }

  .menu-item:focus-visible {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }

  .menu-item.danger {
    color: var(--red-400, #f87171);
  }

  .menu-item.danger:hover {
    background: var(--red-900, #7f1d1d);
    color: var(--red-200, #fecaca);
  }

  .menu-icon {
    width: 16px;
    font-size: 14px;
    flex-shrink: 0;
    text-align: center;
  }

  .menu-label {
    flex: 1;
  }

  .menu-shortcut {
    font-size: 10px;
    color: var(--gray-500, #71717a);
    margin-left: auto;
  }

  .menu-divider {
    height: 1px;
    background: var(--gray-700, #3f3f46);
    margin: 4px 0;
  }

  /* Submenu styles */
  .submenu-trigger {
    position: relative;
  }

  .submenu-arrow {
    font-size: 8px;
    color: var(--gray-500, #71717a);
    margin-left: auto;
  }

  .submenu {
    display: none;
    position: absolute;
    left: 100%;
    top: 0;
    min-width: 160px;
    background: var(--gray-800, #27272a);
    border: 1px solid var(--gray-700, #3f3f46);
    border-radius: 8px;
    box-shadow:
      0 10px 25px rgba(0, 0, 0, 0.4),
      0 4px 10px rgba(0, 0, 0, 0.3);
    padding: 4px 0;
    margin-left: 2px;
  }

  .submenu-trigger:hover .submenu {
    display: block;
  }

  .submenu .menu-item {
    padding: 6px 12px;
  }
</style>
