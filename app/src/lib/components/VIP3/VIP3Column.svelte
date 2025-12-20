<script lang="ts">
  import { filterCounts, vip3Filters, vip3Actions } from '$lib/stores/vip3Store';

  export let title: string;
  export let items: Array<{ id: number; name: string }>;
  export let filterKey: 'folder_ids' | 'instrument_ids' | 'timbre_ids' | 'style_ids' | 'articulation_ids';
  export let countKey: 'folder_counts' | 'instrument_counts' | 'timbre_counts' | 'style_counts' | 'articulation_counts';

  // Get counts for this column's items
  $: counts = $filterCounts?.[countKey] ?? {};

  // Get active selections for this column
  $: activeIds = ($vip3Filters[filterKey] as number[] | undefined) ?? [];

  // Check if an item is selected
  function isActive(id: number): boolean {
    return activeIds.includes(id);
  }

  // Get count for an item (0 if not found)
  function getCount(id: number): number {
    return counts[id] ?? 0;
  }

  // Check if item should be disabled (count is 0 and not currently selected)
  function isDisabled(id: number): boolean {
    return getCount(id) === 0 && !isActive(id);
  }

  // Toggle item selection
  function toggleItem(id: number) {
    if (isDisabled(id)) return;

    vip3Filters.update((filters) => {
      const current = (filters[filterKey] as number[] | undefined) ?? [];
      const updated = current.includes(id)
        ? current.filter((itemId) => itemId !== id)
        : [...current, id];

      return {
        ...filters,
        [filterKey]: updated.length > 0 ? updated : undefined
      };
    });

    // Trigger count and result refresh
    vip3Actions.refreshCounts();
    vip3Actions.search();
  }
</script>

<div class="vip3-column">
  <div class="column-header">
    <h3 class="column-title">{title}</h3>
    {#if activeIds.length > 0}
      <span class="active-count">{activeIds.length} selected</span>
    {/if}
  </div>

  <div class="column-items">
    {#each items as item (item.id)}
      {@const count = getCount(item.id)}
      {@const active = isActive(item.id)}
      {@const disabled = isDisabled(item.id)}

      <button
        class="filter-item"
        class:active
        class:disabled
        on:click={() => toggleItem(item.id)}
        disabled={disabled}
      >
        <span class="item-name">{item.name}</span>
        <span class="item-count" class:zero={count === 0}>
          {count.toLocaleString()}
        </span>
      </button>
    {/each}
  </div>
</div>

<style>
  .vip3-column {
    display: flex;
    flex-direction: column;
    border-right: 1px solid #333;
    background: #1a1a1a;
    min-width: 200px;
    max-width: 250px;
  }

  .column-header {
    padding: 12px 16px;
    border-bottom: 1px solid #333;
    background: #252525;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .column-title {
    font-size: 14px;
    font-weight: 600;
    color: #fff;
    margin: 0;
  }

  .active-count {
    font-size: 12px;
    color: #60a5fa;
    background: rgba(96, 165, 250, 0.1);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .column-items {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .filter-item {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    margin: 2px 0;
    border: none;
    background: transparent;
    color: #ccc;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s ease;
    text-align: left;
  }

  .filter-item:hover:not(.disabled) {
    background: #2a2a2a;
    color: #fff;
  }

  .filter-item.active {
    background: #3b82f6;
    color: #fff;
  }

  .filter-item.active:hover {
    background: #2563eb;
  }

  .filter-item.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .item-name {
    flex: 1;
    font-size: 13px;
  }

  .item-count {
    font-size: 12px;
    color: #999;
    font-weight: 500;
    margin-left: 8px;
  }

  .filter-item.active .item-count {
    color: rgba(255, 255, 255, 0.9);
  }

  .item-count.zero {
    color: #666;
  }

  /* Scrollbar styling */
  .column-items::-webkit-scrollbar {
    width: 8px;
  }

  .column-items::-webkit-scrollbar-track {
    background: #1a1a1a;
  }

  .column-items::-webkit-scrollbar-thumb {
    background: #444;
    border-radius: 4px;
  }

  .column-items::-webkit-scrollbar-thumb:hover {
    background: #555;
  }
</style>
