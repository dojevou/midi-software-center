<script lang="ts">
  import { filterCounts, vip3Filters, vip3Actions } from '$lib/stores/vip3Store';

  const bpmRanges = [
    { label: '60-80', min: 60, max: 80 },
    { label: '80-100', min: 80, max: 100 },
    { label: '100-120', min: 100, max: 120 },
    { label: '120-140', min: 120, max: 140 },
    { label: '140-160', min: 140, max: 160 },
    { label: '160-180', min: 160, max: 180 },
    { label: '180+', min: 180, max: 999 }
  ];

  $: bpmCounts = $filterCounts?.bpm_counts ?? {};
  $: activeBpm = $vip3Filters.bpm_min !== undefined && $vip3Filters.bpm_max !== undefined;

  function selectRange(min: number, max: number) {
    vip3Actions.setFilter('bpm_min', min);
    vip3Actions.setFilter('bpm_max', max);
  }

  function clearBpm() {
    vip3Actions.setFilter('bpm_min', undefined);
    vip3Actions.setFilter('bpm_max', undefined);
  }

  function isActive(min: number, max: number): boolean {
    return $vip3Filters.bpm_min === min && $vip3Filters.bpm_max === max;
  }
</script>

<div class="vip3-column">
  <div class="column-header">
    <h3 class="column-title">BPM</h3>
    {#if activeBpm}
      <button class="clear-btn" on:click={clearBpm}>Clear</button>
    {/if}
  </div>

  <div class="column-items">
    {#each bpmRanges as range (range.label)}
      {@const count = bpmCounts[range.label] ?? 0}
      {@const active = isActive(range.min, range.max)}
      {@const disabled = count === 0 && !active}

      <button
        class="filter-item"
        class:active
        class:disabled
        on:click={() => selectRange(range.min, range.max)}
        {disabled}
      >
        <span class="item-name">{range.label} BPM</span>
        <span class="item-count" class:zero={count === 0}>
          {count.toLocaleString()}
        </span>
      </button>
    {/each}
  </div>
</div>

<style>
  /* Same styles as VIP3Column.svelte */
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

  .clear-btn {
    font-size: 12px;
    color: #60a5fa;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px 8px;
  }

  .clear-btn:hover {
    color: #93c5fd;
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
</style>
