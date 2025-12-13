<script lang="ts">
  import { midiClockStore, syncStatus, type SyncMode } from '$lib/stores/midiClockStore';
  import { onMount } from 'svelte';

  export let showModeSelector = true;

  let selectedMode: SyncMode = 'Internal';
  let expanded = false;

  $: selectedMode = $syncStatus.mode;

  const modeLabels: Record<SyncMode, string> = {
    Internal: 'Internal',
    External: 'External MIDI',
    MidiTimecode: 'MIDI Timecode',
  };

  const modeDescriptions: Record<SyncMode, string> = {
    Internal: 'This device is the clock master',
    External: 'Sync to external MIDI clock',
    MidiTimecode: 'Sync to MIDI Time Code',
  };

  async function handleModeChange(mode: SyncMode) {
    await midiClockStore.setSyncMode(mode.toLowerCase());
    expanded = false;
  }

  // Helper to cast string to SyncMode for template usage
  const syncModes = Object.keys(modeLabels) as SyncMode[];

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.mode-selector')) {
      expanded = false;
    }
  }

  onMount(() => {
    midiClockStore.fetchSyncStatus();
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="sync-status">
  <div class="sync-header">
    <span class="sync-label">SYNC</span>

    <span
      class="sync-indicator"
      class:locked={$syncStatus.is_locked}
      class:external={$syncStatus.mode !== 'Internal'}
      title={$syncStatus.is_locked ? 'Locked to external clock' : 'Not locked'}
    >
      {#if $syncStatus.mode === 'Internal'}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v4M12 18v4M2 12h4M18 12h4" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {:else if $syncStatus.is_locked}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 9.9-1" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      {/if}
    </span>
  </div>

  {#if showModeSelector}
    <div class="mode-selector">
      <button
        class="mode-button"
        on:click|stopPropagation={() => expanded = !expanded}
        aria-expanded={expanded}
        aria-haspopup="listbox"
      >
        {modeLabels[$syncStatus.mode]}
        <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
          <polyline points="6 9 12 15 18 9" stroke="currentColor" stroke-width="2" fill="none" />
        </svg>
      </button>

      {#if expanded}
        <ul class="mode-dropdown" role="listbox">
          {#each syncModes as mode (mode)}
            <li>
              <button
                role="option"
                aria-selected={mode === $syncStatus.mode}
                class:selected={mode === $syncStatus.mode}
                on:click|stopPropagation={() => handleModeChange(mode)}
              >
                <span class="mode-name">{modeLabels[mode]}</span>
                <span class="mode-desc">{modeDescriptions[mode]}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}

  {#if $syncStatus.mode !== 'Internal'}
    <div class="sync-details">
      {#if $syncStatus.external_bpm}
        <span class="external-bpm">
          EXT: {$syncStatus.external_bpm.toFixed(1)} BPM
        </span>
      {/if}

      {#if $syncStatus.drift_ms > 0}
        <span
          class="drift"
          class:warning={$syncStatus.drift_ms > 10}
        >
          Drift: {$syncStatus.drift_ms.toFixed(1)}ms
        </span>
      {/if}

      {#if $syncStatus.clock_source}
        <span class="clock-source">
          Source: {$syncStatus.clock_source}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sync-status {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    background: var(--panel-bg, #1a1a1a);
    border-radius: 6px;
    font-size: 12px;
  }

  .sync-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sync-label {
    font-size: 10px;
    font-weight: bold;
    text-transform: uppercase;
    color: var(--text-muted, #888);
    letter-spacing: 1px;
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    color: var(--text-muted, #666);
  }

  .sync-indicator.external {
    color: var(--color-warning, #eab308);
  }

  .sync-indicator.locked {
    color: var(--color-success, #22c55e);
  }

  .mode-selector {
    position: relative;
  }

  .mode-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .mode-button:hover {
    background: var(--button-hover-bg, #444);
  }

  .mode-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    padding: 4px;
    background: var(--dropdown-bg, #222);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    list-style: none;
    z-index: 100;
    min-width: 180px;
  }

  .mode-dropdown li {
    margin: 0;
    padding: 0;
  }

  .mode-dropdown button {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .mode-dropdown button:hover {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
  }

  .mode-dropdown button.selected {
    background: var(--selected-bg, rgba(59, 130, 246, 0.2));
  }

  .mode-name {
    font-weight: 500;
  }

  .mode-desc {
    font-size: 10px;
    color: var(--text-muted, #888);
    margin-top: 2px;
  }

  .sync-details {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--border-color, #333);
  }

  .external-bpm {
    font-family: monospace;
    color: var(--color-success, #22c55e);
  }

  .drift {
    font-family: monospace;
    color: var(--text-muted, #888);
  }

  .drift.warning {
    color: var(--color-warning, #eab308);
  }

  .clock-source {
    color: var(--text-muted, #888);
  }
</style>
