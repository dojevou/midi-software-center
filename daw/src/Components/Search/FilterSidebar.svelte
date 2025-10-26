<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  export let filters: {
    categories: string[];
    bpmRange: [number, number];
    keySignature: string[];
    timeSignature: string[];
    instruments: string[];
    duration: [number, number];
  };

  const dispatch = createEventDispatcher();

  interface FilterOption {
    value: string;
    label: string;
    count: number;
  }

  let availableCategories: FilterOption[] = [];
  let availableKeys: FilterOption[] = [];
  let availableTimeSignatures: FilterOption[] = [];
  let availableInstruments: FilterOption[] = [];

  onMount(async () => {
    try {
      availableCategories = await invoke<FilterOption[]>('get_categories');
      availableKeys = await invoke<FilterOption[]>('get_key_signatures');
      availableTimeSignatures = await invoke<FilterOption[]>('get_time_signatures');
      availableInstruments = await invoke<FilterOption[]>('get_instruments');
    } catch (error) {
      console.error('Failed to load filter options:', error);
    }
  });

  function toggleFilter(filterArray: string[], value: string) {
    const index = filterArray.indexOf(value);
    if (index >= 0) {
      filterArray.splice(index, 1);
    } else {
      filterArray.push(value);
    }
    filters = filters; // Trigger reactivity
    dispatch('filterChange', filters);
  }

  function handleBpmChange() {
    dispatch('filterChange', filters);
  }

  function handleDurationChange() {
    dispatch('filterChange', filters);
  }

  function clearAllFilters() {
    filters.categories = [];
    filters.bpmRange = [0, 300];
    filters.keySignature = [];
    filters.timeSignature = [];
    filters.instruments = [];
    filters.duration = [0, 600];
    dispatch('filterChange', filters);
  }
</script>

<aside class="filter-sidebar">
  <div class="sidebar-header">
    <h2>Filters</h2>
    <button class="btn-clear" on:click={clearAllFilters}>Clear All</button>
  </div>

  <div class="filter-sections">
    <!-- Category Filter -->
    <div class="filter-section">
      <h3 class="filter-title">📁 Category</h3>
      <div class="filter-options">
        {#each availableCategories as option}
          <label class="filter-option">
            <input
              type="checkbox"
              checked={filters.categories.includes(option.value)}
              on:change={() => toggleFilter(filters.categories, option.value)}
            />
            <span class="option-label">{option.label}</span>
            <span class="option-count">{option.count}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- BPM Range -->
    <div class="filter-section">
      <h3 class="filter-title">⏱️ BPM</h3>
      <div class="range-inputs">
        <input
          type="number"
          class="range-input"
          placeholder="Min"
          bind:value={filters.bpmRange[0]}
          on:change={handleBpmChange}
          min="0"
          max="300"
        />
        <span class="range-separator">to</span>
        <input
          type="number"
          class="range-input"
          placeholder="Max"
          bind:value={filters.bpmRange[1]}
          on:change={handleBpmChange}
          min="0"
          max="300"
        />
      </div>
      <input
        type="range"
        class="dual-slider"
        min="0"
        max="300"
        bind:value={filters.bpmRange[0]}
        on:input={handleBpmChange}
      />
    </div>

    <!-- Key Signature -->
    <div class="filter-section">
      <h3 class="filter-title">🎵 Key Signature</h3>
      <div class="filter-options">
        {#each availableKeys as option}
          <label class="filter-option">
            <input
              type="checkbox"
              checked={filters.keySignature.includes(option.value)}
              on:change={() => toggleFilter(filters.keySignature, option.value)}
            />
            <span class="option-label">{option.label}</span>
            <span class="option-count">{option.count}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Time Signature -->
    <div class="filter-section">
      <h3 class="filter-title">🎼 Time Signature</h3>
      <div class="filter-options">
        {#each availableTimeSignatures as option}
          <label class="filter-option">
            <input
              type="checkbox"
              checked={filters.timeSignature.includes(option.value)}
              on:change={() => toggleFilter(filters.timeSignature, option.value)}
            />
            <span class="option-label">{option.label}</span>
            <span class="option-count">{option.count}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Instruments -->
    <div class="filter-section">
      <h3 class="filter-title">🎹 Instruments</h3>
      <div class="filter-options">
        {#each availableInstruments as option}
          <label class="filter-option">
            <input
              type="checkbox"
              checked={filters.instruments.includes(option.value)}
              on:change={() => toggleFilter(filters.instruments, option.value)}
            />
            <span class="option-label">{option.label}</span>
            <span class="option-count">{option.count}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Duration -->
    <div class="filter-section">
      <h3 class="filter-title">⏳ Duration (seconds)</h3>
      <div class="range-inputs">
        <input
          type="number"
          class="range-input"
          placeholder="Min"
          bind:value={filters.duration[0]}
          on:change={handleDurationChange}
          min="0"
        />
        <span class="range-separator">to</span>
        <input
          type="number"
          class="range-input"
          placeholder="Max"
          bind:value={filters.duration[1]}
          on:change={handleDurationChange}
          min="0"
        />
      </div>
    </div>
  </div>
</aside>

<style>
  .filter-sidebar {
    width: 300px;
    height: 100%;
    background: #1e1e1e;
    border-right: 1px solid #3d3d3d;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #3d3d3d;
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: #e0e0e0;
  }

  .btn-clear {
    padding: 6px 12px;
    background: transparent;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #b0b0b0;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-clear:hover {
    background: #2d2d2d;
    border-color: #4a9eff;
    color: #4a9eff;
  }

  .filter-sections {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .filter-section {
    margin-bottom: 24px;
  }

  .filter-title {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: #e0e0e0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .filter-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .filter-option {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .filter-option:hover {
    background: #252525;
  }

  .filter-option input[type="checkbox"] {
    width: 16px;
    height: 16px;
    cursor: pointer;
    accent-color: #4a9eff;
  }

  .option-label {
    flex: 1;
    font-size: 13px;
    color: #e0e0e0;
  }

  .option-count {
    font-size: 11px;
    color: #808080;
    font-weight: 600;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .range-input {
    flex: 1;
    padding: 8px;
    background: #252525;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    color: #e0e0e0;
    font-size: 13px;
  }

  .range-input:focus {
    outline: none;
    border-color: #4a9eff;
  }

  .range-separator {
    font-size: 12px;
    color: #808080;
  }

  .dual-slider {
    -webkit-appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: #3d3d3d;
    outline: none;
  }

  .dual-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
  }

  .dual-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #4a9eff;
    cursor: pointer;
    border: none;
  }

  /* Custom scrollbar */
  .filter-sections::-webkit-scrollbar {
    width: 6px;
  }

  .filter-sections::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .filter-sections::-webkit-scrollbar-thumb {
    background: #3d3d3d;
    border-radius: 3px;
  }
</style>
