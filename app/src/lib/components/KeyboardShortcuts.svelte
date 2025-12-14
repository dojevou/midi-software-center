<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { keyboardStore, shortcutsByCategory, hasConflicts, comboToString } from '$lib/stores/keyboardStore';
  import type { ShortcutAction, ShortcutCategory } from '$lib/stores/keyboardStore';

  export let visible = false;

  let searchQuery = '';
  let selectedCategory: ShortcutCategory | 'all' = 'all';
  let editingId: string | null = null;

  $: filteredShortcuts = Object.values($keyboardStore.shortcuts).filter(shortcut => {
    if (selectedCategory !== 'all' && shortcut.category !== selectedCategory) {
      return false;
    }
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return shortcut.name.toLowerCase().includes(query) ||
             shortcut.description.toLowerCase().includes(query) ||
             comboToString(shortcut.currentCombo).toLowerCase().includes(query);
    }
    return true;
  });

  function handleKeyDown(event: KeyboardEvent) {
    // Escape to close modal or cancel editing
    if (event.key === 'Escape') {
      if (editingId) {
        keyboardStore.stopRecording();
        editingId = null;
      } else {
        visible = false;
      }
      return;
    }

    // Let keyboard store handle it when editing
    if (editingId && $keyboardStore.isRecording) {
      keyboardStore.handleKeyDown(event);
      editingId = null;
    }
  }

  function startEditing(id: string) {
    editingId = id;
    keyboardStore.startRecording(id);
  }

  function resetShortcut(id: string) {
    keyboardStore.resetShortcut(id);
  }

  function handleOverlayClick() {
    visible = false;
  }

  function handleModalClick(event: MouseEvent) {
    event.stopPropagation();
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if visible}
  <div
    class="keyboard-shortcuts-overlay"
    on:click={handleOverlayClick}
    on:keydown={handleKeyDown}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div
      class="keyboard-shortcuts-modal"
      on:click={handleModalClick}
      on:keydown={() => {}}
      role="document"
    >
      <header>
        <h2>Keyboard Shortcuts</h2>
        <button class="close-btn" on:click={() => visible = false} aria-label="Close">&times;</button>
      </header>

      {#if $hasConflicts}
        <div class="conflicts-warning" role="alert">
          <span class="icon">Warning</span>
          <span>Some shortcuts have conflicts. Review highlighted items.</span>
        </div>
      {/if}

      <div class="search-bar">
        <input
          type="text"
          placeholder="Search shortcuts..."
          bind:value={searchQuery}
          aria-label="Search shortcuts"
        />
        <select bind:value={selectedCategory} aria-label="Filter by category">
          <option value="all">All Categories</option>
          <option value="transport">Transport</option>
          <option value="file">File</option>
          <option value="editing">Editing</option>
          <option value="navigation">Navigation</option>
          <option value="windows">Windows</option>
          <option value="view">View</option>
          <option value="tools">Tools</option>
        </select>
      </div>

      <div class="shortcuts-list" role="list">
        {#each filteredShortcuts as shortcut (shortcut.id)}
          {@const isConflict = $keyboardStore.conflicts.some(c => c.shortcutIds.includes(shortcut.id))}
          {@const isEditing = editingId === shortcut.id}
          <div
            class="shortcut-row"
            class:conflict={isConflict}
            class:editing={isEditing}
            class:disabled={!shortcut.enabled}
            role="listitem"
          >
            <div class="shortcut-info">
              <span class="name">{shortcut.name}</span>
              <span class="description">{shortcut.description}</span>
              {#if shortcut.context}
                <span class="context">{shortcut.context}</span>
              {/if}
            </div>

            <div class="shortcut-combo">
              {#if isEditing}
                <span class="recording">Press keys...</span>
              {:else}
                <kbd class="combo">{comboToString(shortcut.currentCombo)}</kbd>
              {/if}
            </div>

            <div class="shortcut-actions">
              <button
                class="edit-btn"
                on:click={() => startEditing(shortcut.id)}
                disabled={isEditing}
              >
                Edit
              </button>
              {#if comboToString(shortcut.currentCombo) !== comboToString(shortcut.defaultCombo)}
                <button
                  class="reset-btn"
                  on:click={() => resetShortcut(shortcut.id)}
                >
                  Reset
                </button>
              {/if}
              <label class="enable-toggle">
                <input
                  type="checkbox"
                  checked={shortcut.enabled}
                  on:change={(e) => keyboardStore.setEnabled(shortcut.id, e.currentTarget.checked)}
                />
              </label>
            </div>
          </div>
        {/each}
      </div>

      <footer>
        <button class="reset-all-btn" on:click={() => keyboardStore.resetAll()}>
          Reset All to Defaults
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .keyboard-shortcuts-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .keyboard-shortcuts-modal {
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 8px;
    width: 800px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #333);
  }

  header h2 {
    margin: 0;
    font-size: 18px;
    color: var(--text-primary, #fff);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    font-size: 24px;
    cursor: pointer;
  }

  .conflicts-warning {
    background: rgba(255, 193, 7, 0.15);
    border: 1px solid #ffc107;
    color: #ffc107;
    padding: 10px 16px;
    margin: 16px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-bar {
    display: flex;
    gap: 12px;
    padding: 16px 20px;
  }

  .search-bar input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-primary, #fff);
  }

  .search-bar select {
    padding: 8px 12px;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-primary, #fff);
  }

  .shortcuts-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 20px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    padding: 12px;
    border-bottom: 1px solid var(--border-color, #333);
    gap: 16px;
  }

  .shortcut-row.conflict {
    background: rgba(220, 53, 69, 0.1);
    border-color: rgba(220, 53, 69, 0.3);
  }

  .shortcut-row.editing {
    background: rgba(0, 123, 255, 0.1);
    border-color: rgba(0, 123, 255, 0.3);
  }

  .shortcut-row.disabled {
    opacity: 0.5;
  }

  .shortcut-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .shortcut-info .name {
    font-weight: 500;
    color: var(--text-primary, #fff);
  }

  .shortcut-info .description {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .shortcut-info .context {
    font-size: 10px;
    color: var(--accent-color, #007bff);
    background: rgba(0, 123, 255, 0.1);
    padding: 2px 6px;
    border-radius: 3px;
    width: fit-content;
    margin-top: 4px;
  }

  .shortcut-combo {
    min-width: 120px;
    text-align: center;
  }

  .combo {
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    padding: 4px 8px;
    font-family: monospace;
    font-size: 12px;
    color: var(--text-primary, #fff);
  }

  .recording {
    color: var(--accent-color, #007bff);
    font-style: italic;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .shortcut-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .edit-btn, .reset-btn {
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .edit-btn {
    background: var(--accent-color, #007bff);
    border: none;
    color: white;
  }

  .reset-btn {
    background: none;
    border: 1px solid var(--border-color, #444);
    color: var(--text-secondary, #888);
  }

  footer {
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, #333);
    display: flex;
    justify-content: flex-end;
  }

  .reset-all-btn {
    background: rgba(220, 53, 69, 0.2);
    border: 1px solid #dc3545;
    color: #dc3545;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
