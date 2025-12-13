<script lang="ts">
  import { onMount } from 'svelte';
  import { CommandInvoker } from '$lib/api/commands';
  import type { MidiPort } from '$lib/types';

  export let compact: boolean = false;

  const invoker = new CommandInvoker();

  let clockEnabled = false;
  let transportEnabled = false;
  let ports: MidiPort[] = [];
  let isLoading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadState();
  });

  async function loadState() {
    isLoading = true;
    error = null;
    try {
      const state = await invoker.midiIO.getState();
      ports = Object.values(state.ports).filter(p => p.direction === 'output');
      // Master state would need to be fetched from mixer
      const master = await invoker.mixer.getMaster() as { clock_enabled?: boolean; transport_enabled?: boolean };
      clockEnabled = master?.clock_enabled ?? false;
      transportEnabled = master?.transport_enabled ?? false;
    } catch (err) {
      console.error('Failed to load MIDI sync state:', err);
      error = err instanceof Error ? err.message : 'Failed to load state';
    } finally {
      isLoading = false;
    }
  }

  async function toggleMasterClock() {
    const newValue = !clockEnabled;
    try {
      await invoker.mixer.setMasterClockEnabled(newValue);
      clockEnabled = newValue;
    } catch (err) {
      console.error('Failed to toggle master clock:', err);
    }
  }

  async function toggleMasterTransport() {
    const newValue = !transportEnabled;
    try {
      await invoker.mixer.setMasterTransportEnabled(newValue);
      transportEnabled = newValue;
    } catch (err) {
      console.error('Failed to toggle master transport:', err);
    }
  }

  async function togglePortClock(port: MidiPort) {
    const newValue = !port.send_clock;
    try {
      const updated = await invoker.midiIO.updatePort(port.id, { sendClock: newValue });
      ports = ports.map(p => p.id === port.id ? updated : p);
    } catch (err) {
      console.error('Failed to toggle port clock:', err);
    }
  }

  async function togglePortTransport(port: MidiPort) {
    const newValue = !port.send_transport;
    try {
      const updated = await invoker.midiIO.updatePort(port.id, { sendTransport: newValue });
      ports = ports.map(p => p.id === port.id ? updated : p);
    } catch (err) {
      console.error('Failed to toggle port transport:', err);
    }
  }
</script>

<div class="midi-sync-controls" class:compact>
  <div class="header">
    <h3 class="title">MIDI Sync</h3>
    <button type="button" class="refresh-btn" on:click={loadState} title="Refresh">
      ↻
    </button>
  </div>

  {#if isLoading}
    <div class="loading">
      <span class="spinner">⏳</span>
      <span>Loading...</span>
    </div>
  {:else if error}
    <div class="error">
      <span>⚠️ {error}</span>
      <button type="button" on:click={loadState}>Retry</button>
    </div>
  {:else}
    <div class="master-section">
      <h4 class="section-title">Master Output</h4>
      <div class="control-row">
        <label class="toggle-label">
          <input type="checkbox" checked={clockEnabled} on:change={toggleMasterClock} />
          <span class="toggle-text">Send MIDI Clock</span>
        </label>
      </div>
      <div class="control-row">
        <label class="toggle-label">
          <input type="checkbox" checked={transportEnabled} on:change={toggleMasterTransport} />
          <span class="toggle-text">Send Transport (Start/Stop/Continue)</span>
        </label>
      </div>
    </div>

    {#if ports.length > 0}
      <div class="ports-section">
        <h4 class="section-title">Per-Port Settings</h4>
        <div class="port-list">
          {#each ports as port (port.id)}
            <div class="port-row">
              <span class="port-name" title={port.device_name || port.name}>
                {port.display_name || port.alias || port.name}
              </span>
              <div class="port-controls">
                <button
                  type="button"
                  class="port-btn"
                  class:active={port.send_clock}
                  on:click={() => togglePortClock(port)}
                  title="Send Clock"
                >
                  CLK
                </button>
                <button
                  type="button"
                  class="port-btn"
                  class:active={port.send_transport}
                  on:click={() => togglePortTransport(port)}
                  title="Send Transport"
                >
                  TRS
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-ports">
        No MIDI output ports configured
      </div>
    {/if}

    <div class="info-section">
      <p class="info-text">
        When enabled, MIDI Clock (24 PPQN) and transport messages will be sent to external devices for synchronization.
      </p>
    </div>
  {/if}
</div>

<style>
  .midi-sync-controls {
    display: flex;
    flex-direction: column;
    background: var(--gray-900, #18181b);
    border-radius: 8px;
    overflow: hidden;
    min-width: 280px;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--gray-850, #1f1f23);
    border-bottom: 1px solid var(--gray-700, #3f3f46);
  }

  .title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--gray-100, #f4f4f5);
  }

  .refresh-btn {
    background: transparent;
    border: none;
    color: var(--gray-400, #a1a1aa);
    cursor: pointer;
    font-size: 16px;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .refresh-btn:hover {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-200, #e4e4e7);
  }

  .loading,
  .error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px 16px;
    color: var(--gray-400, #a1a1aa);
    gap: 8px;
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error {
    color: var(--red-400, #f87171);
  }

  .error button {
    margin-top: 8px;
    background: var(--gray-700, #3f3f46);
    border: none;
    color: var(--gray-200, #e4e4e7);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .master-section,
  .ports-section {
    padding: 16px;
    border-bottom: 1px solid var(--gray-800, #27272a);
  }

  .section-title {
    margin: 0 0 12px 0;
    font-size: 12px;
    font-weight: 600;
    color: var(--gray-400, #a1a1aa);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .control-row {
    margin-bottom: 8px;
  }

  .control-row:last-child {
    margin-bottom: 0;
  }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--primary-500, #3b82f6);
  }

  .toggle-text {
    font-size: 13px;
    color: var(--gray-200, #e4e4e7);
  }

  .port-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .port-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--gray-800, #27272a);
    border-radius: 6px;
  }

  .port-name {
    font-size: 13px;
    color: var(--gray-200, #e4e4e7);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    margin-right: 12px;
  }

  .port-controls {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .port-btn {
    padding: 4px 8px;
    font-size: 10px;
    font-weight: 600;
    border: 1px solid var(--gray-600, #52525b);
    border-radius: 4px;
    background: transparent;
    color: var(--gray-400, #a1a1aa);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .port-btn:hover {
    background: var(--gray-700, #3f3f46);
    border-color: var(--gray-500, #71717a);
    color: var(--gray-200, #e4e4e7);
  }

  .port-btn.active {
    background: var(--primary-600, #2563eb);
    border-color: var(--primary-500, #3b82f6);
    color: white;
  }

  .empty-ports {
    padding: 16px;
    text-align: center;
    color: var(--gray-500, #71717a);
    font-size: 13px;
  }

  .info-section {
    padding: 12px 16px;
  }

  .info-text {
    margin: 0;
    font-size: 11px;
    color: var(--gray-500, #71717a);
    line-height: 1.5;
  }

  /* Compact mode */
  .compact .header {
    padding: 8px 12px;
  }

  .compact .title {
    font-size: 12px;
  }

  .compact .master-section,
  .compact .ports-section {
    padding: 12px;
  }

  .compact .section-title {
    font-size: 10px;
    margin-bottom: 8px;
  }

  .compact .toggle-text {
    font-size: 12px;
  }

  .compact .port-row {
    padding: 6px 10px;
  }

  .compact .port-name {
    font-size: 12px;
  }
</style>
