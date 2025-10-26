<script lang="ts">
  import { writable } from 'svelte/store';

  import TopBar from './TopBar.svelte';
  import SearchBar from '../Components/Search/SearchBar.svelte';
  import FilterSidebar from '../Components/Search/FilterSidebar.svelte';
  import ResultsView from '../Components/Search/ResultsView.svelte';
  import SequencerPanel from '../Components/Sequencer/SequencerPanel.svelte';
  import StatusBar from './StatusBar.svelte';

  export let dbConnected: boolean;
  export let totalFiles: number;

  interface SearchFilters {
    categories: string[];
    bpmRange: [number, number];
    keySignature: string[];
    timeSignature: string[];
    instruments: string[];
    duration: [number, number];
  }

  let showFilters = true;
  let showSequencer = false;
  let searchQuery = '';
  let filters: SearchFilters = {
    categories: [],
    bpmRange: [0, 300],
    keySignature: [],
    timeSignature: [],
    instruments: [],
    duration: [0, 600]
  };

  function toggleFilters() {
    showFilters = !showFilters;
  }

  function toggleSequencer() {
    showSequencer = !showSequencer;
  }
</script>

<div class="main-layout">
  <TopBar />

  <div class="search-bar-container">
    <SearchBar bind:query={searchQuery} on:search />
  </div>

  <div class="content-area">
    {#if showFilters}
      <FilterSidebar bind:filters on:filterChange />
    {/if}

    <div class="main-content">
      <div class="toolbar">
        <button class="toolbar-btn" on:click={toggleFilters}>
          {showFilters ? '◀' : '▶'} Filters
        </button>
        <button class="toolbar-btn" on:click={toggleSequencer}>
          {showSequencer ? '▼' : '▲'} Sequencer
        </button>
      </div>

      <ResultsView {searchQuery} {filters} />
    </div>
  </div>

  {#if showSequencer}
    <div class="sequencer-container">
      <SequencerPanel />
    </div>
  {/if}

  <StatusBar {dbConnected} {totalFiles} />
</div>

<style>
  .main-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1a1a1a;
  }

  .search-bar-container {
    padding: 16px 24px;
    background: #252525;
    border-bottom: 1px solid #3d3d3d;
  }

  .content-area {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    background: #1e1e1e;
    border-bottom: 1px solid #3d3d3d;
  }

  .toolbar-btn {
    padding: 8px 16px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .toolbar-btn:hover {
    background: #3d3d3d;
    border-color: #4a9eff;
  }

  .sequencer-container {
    height: 300px;
    background: #1e1e1e;
    border-top: 1px solid #3d3d3d;
  }
</style>
