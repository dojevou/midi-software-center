# Day 1, Part 1D: Filter Counts UI Components

**Duration:** 1.5 hours
**Prerequisites:** Parts 1A, 1B, 1C completed
**Files to modify:** 1

---

## Overview

You've completed the backend (Parts A & B) and frontend API/store (Part C). Now you'll:
1. Update VIP3Column component to display filter counts
2. Add visual indicators for active filters
3. Disable filters with 0 results
4. Add real-time count updates

---

## Step 1: Update VIP3Column Component (1 hour)

Update or create `app/src/lib/components/VIP3/VIP3Column.svelte`:

```svelte
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
```

---

## Step 2: Create BPM Range Column Component (20 min)

Create `app/src/lib/components/VIP3/VIP3BpmColumn.svelte`:

```svelte
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
```

---

## Step 3: Update VIP3Browser to Use Columns (10 min)

Update `app/src/lib/components/VIP3/VIP3Browser.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import VIP3Column from './VIP3Column.svelte';
  import VIP3BpmColumn from './VIP3BpmColumn.svelte';
  import {
    vip3Actions,
    timbres,
    styles,
    articulations,
    loadingCounts,
    totalMatches
  } from '$lib/stores/vip3Store';

  // Mock folders for now (in Phase 3, load from database)
  const folders = [
    { id: 1, name: 'Drums' },
    { id: 2, name: 'Bass' },
    { id: 3, name: 'Keys' },
    { id: 4, name: 'Synths' }
  ];

  // Mock instruments (in Phase 3, load from tags)
  const instruments = [
    { id: 5, name: 'Kick' },
    { id: 12, name: 'Snare' },
    { id: 18, name: 'Hi-Hat' },
    { id: 25, name: 'Piano' }
  ];

  onMount(async () => {
    await vip3Actions.initialize();
  });
</script>

<div class="vip3-browser">
  <div class="browser-header">
    <h2>VIP3 Browser</h2>
    {#if $loadingCounts}
      <span class="loading">Loading counts...</span>
    {:else}
      <span class="total-matches">
        {$totalMatches.toLocaleString()} files
      </span>
    {/if}
  </div>

  <div class="filter-columns">
    <VIP3Column
      title="Folders"
      items={folders}
      filterKey="folder_ids"
      countKey="folder_counts"
    />

    <VIP3Column
      title="Instruments"
      items={instruments}
      filterKey="instrument_ids"
      countKey="instrument_counts"
    />

    <VIP3Column
      title="Timbre"
      items={$timbres}
      filterKey="timbre_ids"
      countKey="timbre_counts"
    />

    <VIP3Column
      title="Style"
      items={$styles}
      filterKey="style_ids"
      countKey="style_counts"
    />

    <VIP3Column
      title="Articulation"
      items={$articulations}
      filterKey="articulation_ids"
      countKey="articulation_counts"
    />

    <VIP3BpmColumn />
  </div>

  <div class="results-panel">
    <!-- Results grid will be added in Phase 2 -->
    <p class="placeholder">Search results will appear here</p>
  </div>
</div>

<style>
  .vip3-browser {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #151515;
    color: #fff;
  }

  .browser-header {
    padding: 16px 24px;
    border-bottom: 1px solid #333;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #1f1f1f;
  }

  .browser-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .loading {
    font-size: 14px;
    color: #999;
  }

  .total-matches {
    font-size: 14px;
    color: #60a5fa;
    font-weight: 500;
  }

  .filter-columns {
    display: flex;
    border-bottom: 1px solid #333;
    background: #1a1a1a;
    overflow-x: auto;
  }

  .results-panel {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .placeholder {
    color: #666;
    text-align: center;
    margin-top: 48px;
  }
</style>
```

---

## Verification (10 min)

### 1. Visual Check

Start the dev server:

```bash
make dev
```

Navigate to the VIP3 browser page and verify:

**✓ Initial State:**
- All filter columns visible
- Counts appear next to each filter option
- Total matches shown in header

**✓ Interaction:**
- Click a folder → count updates, other columns update
- Click multiple filters → counts decrease appropriately
- Click filter with 0 count → nothing happens (disabled)
- Hover over filters → background changes

**✓ Active State:**
- Selected filters highlighted in blue
- "X selected" badge appears in column header
- Active filter counts show white text

**✓ Performance:**
- Check browser console for timing logs
- Filter count updates should complete <50ms
- No lag when clicking filters

### 2. Console Verification

Open browser console and run:

```javascript
// Check filter state
console.log('Current filters:', window.$vip3Filters);

// Check counts
console.log('Filter counts:', window.$filterCounts);

// Toggle a folder
vip3Actions.toggleFolder(1);

// Verify counts updated
setTimeout(() => {
  console.log('Updated counts:', window.$filterCounts);
}, 100);
```

### 3. Expected Behavior

| Action | Expected Result |
|--------|----------------|
| Click "Drums" folder | Drums highlighted blue, other counts update |
| Click "Kick" instrument | Both filters active, counts narrow down |
| Click "100-120 BPM" | BPM range selected, counts update |
| Click active filter again | Filter removed, counts expand |
| Click filter with 0 count | Nothing happens (disabled state) |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Counts not updating | Check Part 1C store is imported correctly |
| All counts showing 0 | Verify database has files and Part 1B command works |
| Filters not clickable | Check disabled state logic in `isDisabled()` |
| Performance >50ms | Continue to Day 2 (Database Optimization) |
| Columns not scrolling | Add `overflow-y: auto` to `.column-items` |
| Active state not showing | Verify `class:active` binding in button |

---

## What's Next?

✅ **Day 1 Complete! You've implemented:**
- ✅ Part 1A: Backend models (FilterCounts, Vip3Filters, VIP3Repository)
- ✅ Part 1B: Tauri commands with parallel execution
- ✅ Part 1C: Frontend API and Svelte store
- ✅ Part 1D: UI components with real-time counts

**Next:** [Day 2: Database Optimization](./DAY2_DATABASE_OPTIMIZATION.md)
- Create partial indexes for VIP3 queries
- Add composite indexes for common filter combinations
- Optimize WHERE clause building
- Performance testing to hit <50ms target
- Query plan analysis

**Current Status:**
- Filter counts system is functional ✓
- May be >50ms without indexes (expected)
- Day 2 will optimize to <50ms target

**Testing Checklist:**
- [ ] Can click filters and see counts update
- [ ] Filters with 0 results are disabled
- [ ] Active filters highlighted in blue
- [ ] Total matches updates correctly
- [ ] Multiple filters work together
- [ ] Performance logged in console
