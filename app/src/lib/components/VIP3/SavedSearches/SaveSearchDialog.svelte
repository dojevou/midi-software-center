<script lang="ts">
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import type { Vip3Filters } from '$lib/types/vip3';

  export let onClose: () => void = () => {};
  export let onSave: () => void = () => {};
  export let currentFilters: Vip3Filters = {};

  let name = '';
  let description = '';
  let icon = '';
  let color = '#007aff';
  let saving = false;
  let error: string | null = null;

  const iconOptions = ['🔍', '⭐', '🎵', '🎹', '🥁', '🎸', '🎺', '🎻', '📁', '💾'];
  const colorOptions = [
    '#007aff', // Blue
    '#34c759', // Green
    '#ff9500', // Orange
    '#ff3b30', // Red
    '#af52de', // Purple
    '#5ac8fa', // Cyan
    '#ffcc00', // Yellow
    '#ff2d55', // Pink
  ];

  async function handleSave() {
    if (!name.trim()) {
      error = 'Please enter a name';
      return;
    }

    try {
      saving = true;
      error = null;

      await Vip3BrowserApi.saveSearch({
        name: name.trim(),
        description: description.trim() || undefined,
        filters: currentFilters,
        icon: icon || undefined,
        color: color || undefined,
      });

      onSave();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to save search';
      console.error('Error saving search:', e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    } else if (event.key === 'Enter' && event.ctrlKey) {
      handleSave();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-overlay" on:click={onClose}>
  <div class="modal" on:click|stopPropagation>
    <div class="modal-header">
      <h3>Save Search</h3>
      <button class="btn-close" on:click={onClose}>×</button>
    </div>

    <div class="modal-body">
      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <div class="form-group">
        <label for="name">Name <span class="required">*</span></label>
        <input
          id="name"
          type="text"
          bind:value={name}
          placeholder="e.g., Jazz Piano 120 BPM"
          maxlength="100"
          disabled={saving}
          autofocus
        />
      </div>

      <div class="form-group">
        <label for="description">Description</label>
        <textarea
          id="description"
          bind:value={description}
          placeholder="Optional description..."
          rows="3"
          maxlength="500"
          disabled={saving}
        />
      </div>

      <div class="form-group">
        <label>Icon</label>
        <div class="icon-picker">
          {#each iconOptions as iconOption}
            <button
              class="icon-option"
              class:selected={icon === iconOption}
              on:click={() => icon = iconOption}
              disabled={saving}
            >
              {iconOption}
            </button>
          {/each}
          <button
            class="icon-option clear"
            class:selected={!icon}
            on:click={() => icon = ''}
            disabled={saving}
            title="No icon"
          >
            ∅
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>Color</label>
        <div class="color-picker">
          {#each colorOptions as colorOption}
            <button
              class="color-option"
              class:selected={color === colorOption}
              style:background-color={colorOption}
              on:click={() => color = colorOption}
              disabled={saving}
              title={colorOption}
            />
          {/each}
        </div>
      </div>

      <div class="filter-preview">
        <strong>Current Filters:</strong>
        <div class="filter-summary">
          {#if currentFilters.folder_ids?.length}
            <span class="filter-tag">📁 {currentFilters.folder_ids.length} folders</span>
          {/if}
          {#if currentFilters.instrument_ids?.length}
            <span class="filter-tag">🎹 {currentFilters.instrument_ids.length} instruments</span>
          {/if}
          {#if currentFilters.timbre_ids?.length}
            <span class="filter-tag">🎵 {currentFilters.timbre_ids.length} timbres</span>
          {/if}
          {#if currentFilters.style_ids?.length}
            <span class="filter-tag">🎨 {currentFilters.style_ids.length} styles</span>
          {/if}
          {#if !currentFilters.folder_ids?.length &&
               !currentFilters.instrument_ids?.length &&
               !currentFilters.timbre_ids?.length &&
               !currentFilters.style_ids?.length}
            <span class="no-filters">No filters applied</span>
          {/if}
        </div>
      </div>
    </div>

    <div class="modal-footer">
      <button class="btn-secondary" on:click={onClose} disabled={saving}>
        Cancel
      </button>
      <button class="btn-primary" on:click={handleSave} disabled={saving || !name.trim()}>
        {saving ? 'Saving...' : 'Save Search'}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 12px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid var(--color-border, #3a3a3a);
  }

  h3 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: var(--color-text, #fff);
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 32px;
    color: var(--color-text-secondary, #999);
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .btn-close:hover {
    color: var(--color-text, #fff);
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .error-message {
    padding: 12px;
    background: var(--color-error-bg, #3a1f1f);
    border-left: 4px solid var(--color-error, #ff3b30);
    border-radius: 4px;
    color: var(--color-error, #ff3b30);
    margin-bottom: 16px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  label {
    display: block;
    margin-bottom: 8px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text, #fff);
  }

  .required {
    color: var(--color-error, #ff3b30);
  }

  input,
  textarea {
    width: 100%;
    padding: 10px 12px;
    background: var(--color-bg-tertiary, #333);
    border: 1px solid var(--color-border, #3a3a3a);
    border-radius: 6px;
    color: var(--color-text, #fff);
    font-size: 14px;
    font-family: inherit;
  }

  input:focus,
  textarea:focus {
    outline: none;
    border-color: var(--color-primary, #007aff);
  }

  textarea {
    resize: vertical;
  }

  .icon-picker,
  .color-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .icon-option {
    width: 44px;
    height: 44px;
    padding: 0;
    background: var(--color-bg-tertiary, #333);
    border: 2px solid transparent;
    border-radius: 8px;
    font-size: 24px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .icon-option:hover {
    background: var(--color-bg-hover, #3a3a3a);
  }

  .icon-option.selected {
    border-color: var(--color-primary, #007aff);
    background: var(--color-bg-hover, #3a3a3a);
  }

  .icon-option.clear {
    font-size: 20px;
    color: var(--color-text-secondary, #999);
  }

  .color-option {
    width: 40px;
    height: 40px;
    padding: 0;
    border: 3px solid transparent;
    border-radius: 50%;
    cursor: pointer;
    transition: all 0.2s;
  }

  .color-option:hover {
    transform: scale(1.1);
  }

  .color-option.selected {
    border-color: white;
    box-shadow: 0 0 0 2px var(--color-bg-secondary, #2a2a2a);
  }

  .filter-preview {
    padding: 12px;
    background: var(--color-bg-tertiary, #333);
    border-radius: 6px;
    font-size: 13px;
  }

  .filter-preview strong {
    display: block;
    margin-bottom: 8px;
    color: var(--color-text, #fff);
  }

  .filter-summary {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .filter-tag {
    padding: 4px 8px;
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-text-secondary, #999);
  }

  .no-filters {
    color: var(--color-text-tertiary, #666);
    font-style: italic;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 20px;
    border-top: 1px solid var(--color-border, #3a3a3a);
  }

  .btn-secondary,
  .btn-primary {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-secondary {
    background: var(--color-bg-tertiary, #333);
    color: var(--color-text, #fff);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-bg-hover, #3a3a3a);
  }

  .btn-primary {
    background: var(--color-primary, #007aff);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #0056b3);
  }

  .btn-secondary:disabled,
  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
