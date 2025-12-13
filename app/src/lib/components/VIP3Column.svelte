<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let title: string;
  export let icon: string = '';
  export let options: { value: string; label: string; icon?: string; count?: number }[] = [];
  export let selected: string[] = [];
  export let multiSelect: boolean = true;
  export let collapsed: boolean = false;
  export let showCounts: boolean = true;
  export let showAll: boolean = true;

  const dispatch = createEventDispatcher<{
    select: { value: string; selected: boolean };
    clear: void;
    toggle: { collapsed: boolean };
  }>();

  function handleOptionClick(value: string) {
    if (multiSelect) {
      const isSelected = selected.includes(value);
      dispatch('select', { value, selected: !isSelected });
    } else {
      // Single select - toggle off if already selected
      if (selected.includes(value)) {
        dispatch('clear');
      } else {
        dispatch('select', { value, selected: true });
      }
    }
  }

  function handleClear() {
    dispatch('clear');
  }

  function handleToggleCollapse() {
    dispatch('toggle', { collapsed: !collapsed });
  }

  $: selectedCount = selected.length;
  $: isAllSelected = selected.length === 0;
</script>

<div class="vip3-column" class:collapsed>
  <button
    type="button"
    class="column-header"
    on:click={handleToggleCollapse}
    aria-expanded={!collapsed}
  >
    <span class="column-title">
      {#if icon}<span class="column-icon">{icon}</span>{/if}
      <span class="column-text">{title}</span>
    </span>
    {#if selectedCount > 0}
      <span class="filter-badge">{selectedCount}</span>
    {/if}
    <span class="collapse-icon" aria-hidden="true">{collapsed ? '▶' : '▼'}</span>
  </button>

  {#if !collapsed}
    <div class="column-content" role="listbox" aria-label="{title} filter options">
      {#if showAll}
        <button
          type="button"
          class="filter-option"
          class:active={isAllSelected}
          on:click={handleClear}
          role="option"
          aria-selected={isAllSelected}
        >
          <span class="option-check">{isAllSelected ? '●' : '○'}</span>
          <span class="option-label">All</span>
        </button>
        <div class="divider" aria-hidden="true"></div>
      {/if}

      {#each options as option (option.value)}
        {@const isSelected = selected.includes(option.value)}
        <button
          type="button"
          class="filter-option"
          class:active={isSelected}
          on:click={() => handleOptionClick(option.value)}
          role="option"
          aria-selected={isSelected}
        >
          <span class="option-check">
            {#if multiSelect}
              {isSelected ? '☑' : '☐'}
            {:else}
              {isSelected ? '●' : '○'}
            {/if}
          </span>
          {#if option.icon}
            <span class="option-icon">{option.icon}</span>
          {/if}
          <span class="option-label">{option.label}</span>
          {#if showCounts && option.count !== undefined}
            <span class="option-count">({option.count.toLocaleString()})</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .vip3-column {
    display: flex;
    flex-direction: column;
    width: 130px;
    min-width: 130px;
    border-right: 1px solid var(--gray-700, #3f3f46);
    background: var(--gray-900, #18181b);
  }

  .vip3-column:last-child {
    border-right: none;
  }

  .vip3-column.collapsed {
    width: 40px;
    min-width: 40px;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 4px;
    padding: 10px 8px;
    background: var(--gray-800, #27272a);
    border: none;
    border-bottom: 1px solid var(--gray-700, #3f3f46);
    cursor: pointer;
    width: 100%;
    text-align: left;
    color: inherit;
    transition: background-color 0.15s ease;
  }

  .column-header:hover {
    background: var(--gray-700, #3f3f46);
  }

  .column-header:focus-visible {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
  }

  .column-title {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .column-icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .column-text {
    font-size: 11px;
    font-weight: 600;
    color: var(--gray-100, #f4f4f5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .collapsed .column-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
  }

  .filter-badge {
    background: var(--primary-500, #3b82f6);
    color: white;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 10px;
    min-width: 18px;
    text-align: center;
    flex-shrink: 0;
  }

  .collapse-icon {
    font-size: 10px;
    color: var(--gray-500, #71717a);
    flex-shrink: 0;
  }

  .column-content {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  /* Custom scrollbar */
  .column-content::-webkit-scrollbar {
    width: 6px;
  }

  .column-content::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .column-content::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 3px;
  }

  .column-content::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }

  .filter-option {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: var(--gray-400, #a1a1aa);
    font-size: 11px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .filter-option:hover {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-100, #f4f4f5);
  }

  .filter-option:focus-visible {
    outline: 2px solid var(--primary-500, #3b82f6);
    outline-offset: -2px;
    background: var(--gray-700, #3f3f46);
  }

  .filter-option.active {
    background: var(--primary-600, #2563eb);
    color: white;
  }

  .filter-option.active:hover {
    background: var(--primary-500, #3b82f6);
  }

  .option-check {
    font-size: 12px;
    width: 14px;
    flex-shrink: 0;
  }

  .option-icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .option-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .option-count {
    font-size: 10px;
    color: var(--gray-500, #71717a);
    flex-shrink: 0;
  }

  .filter-option.active .option-count {
    color: var(--gray-300, #d4d4d8);
  }

  .divider {
    height: 1px;
    background: var(--gray-700, #3f3f46);
    margin: 4px 8px;
  }
</style>
