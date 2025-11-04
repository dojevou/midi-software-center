<script lang="ts">
  /**
   * MidiDeviceManagerWindow - MIDI hardware device management
   *
   * Task-O-Matic: Manage MIDI inputs/outputs, configure devices, test connections.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  // ============================================================================
  // Types
  // ============================================================================

  interface MidiDevice {
    id: string;
    name: string;
    portCount: number;
    latencyMs: number;
    status: 'connected' | 'disconnected' | 'error';
    manufacturer?: string;
    version?: string;
    isConfigured: boolean;
    alias?: string;
  }

  interface DeviceSettings {
    deviceId: string;
    alias: string;
    mapping: string;
    enabled: boolean;
  }

  type FilterMode = 'all' | 'connected' | 'configured';

  // ============================================================================
  // State
  // ============================================================================

  // Device lists
  let inputDevices: MidiDevice[] = [];
  let outputDevices: MidiDevice[] = [];

  // UI state
  let loading = false;
  let error: string | null = null;
  let selectedInputId: string | null = null;
  let selectedOutputId: string | null = null;
  let filterMode: FilterMode = 'all';
  let refreshing = false;
  let testingDeviceId: string | null = null;

  // Detail panel
  let showDetailsPanel = false;
  let detailsDevice: MidiDevice | null = null;
  let detailsType: 'input' | 'output' | null = null;

  // Settings modal
  let showSettingsModal = false;
  let settingsDevice: MidiDevice | null = null;
  let settingsForm: DeviceSettings = {
    deviceId: '',
    alias: '',
    mapping: '',
    enabled: true
  };

  // Event listeners
  let deviceDetectionUnlisten: (() => void) | null = null;

  // ============================================================================
  // Reactive
  // ============================================================================

  $: filteredInputs = filterDevices(inputDevices, filterMode);
  $: filteredOutputs = filterDevices(outputDevices, filterMode);
  $: selectedInput = inputDevices.find(d => d.id === selectedInputId) || null;
  $: selectedOutput = outputDevices.find(d => d.id === selectedOutputId) || null;

  // ============================================================================
  // Filtering
  // ============================================================================

  function filterDevices(devices: MidiDevice[], mode: FilterMode): MidiDevice[] {
    switch (mode) {
      case 'connected':
        return devices.filter(d => d.status === 'connected');
      case 'configured':
        return devices.filter(d => d.isConfigured);
      case 'all':
      default:
        return devices;
    }
  }

  // ============================================================================
  // Device Management
  // ============================================================================

  async function loadDevices(): Promise<void> {
    loading = true;
    error = null;

    try {
      const [inputs, outputs] = await Promise.all([
        invoke<MidiDevice[]>('get_midi_inputs'),
        invoke<MidiDevice[]>('get_midi_outputs')
      ]);

      inputDevices = inputs;
      outputDevices = outputs;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load MIDI devices';
      console.error('Device load error:', err);
    } finally {
      loading = false;
    }
  }

  async function refreshDevices(): Promise<void> {
    refreshing = true;
    try {
      await invoke('rescan_midi_devices');
      await loadDevices();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to refresh devices';
      console.error('Refresh error:', err);
    } finally {
      refreshing = false;
    }
  }

  async function connectDevice(deviceId: string, type: 'input' | 'output'): Promise<void> {
    try {
      await invoke('connect_midi_device', { deviceId, deviceType: type });
      await loadDevices();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to connect device';
      console.error('Connect error:', err);
    }
  }

  async function disconnectDevice(deviceId: string, type: 'input' | 'output'): Promise<void> {
    try {
      await invoke('disconnect_midi_device', { deviceId, deviceType: type });
      await loadDevices();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to disconnect device';
      console.error('Disconnect error:', err);
    }
  }

  async function testDevice(deviceId: string): Promise<void> {
    testingDeviceId = deviceId;
    try {
      // Send test note: Middle C (60) with velocity 100, 500ms duration
      await invoke('send_test_note', {
        deviceId,
        note: 60,
        velocity: 100,
        durationMs: 500
      });

      // Visual feedback - clear after note duration
      setTimeout(() => {
        testingDeviceId = null;
      }, 600);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to test device';
      console.error('Test error:', err);
      testingDeviceId = null;
    }
  }

  // ============================================================================
  // Device Details
  // ============================================================================

  function showDetails(device: MidiDevice, type: 'input' | 'output'): void {
    detailsDevice = device;
    detailsType = type;
    showDetailsPanel = true;
  }

  function closeDetails(): void {
    showDetailsPanel = false;
    detailsDevice = null;
    detailsType = null;
  }

  // ============================================================================
  // Device Settings
  // ============================================================================

  function openSettings(device: MidiDevice): void {
    settingsDevice = device;
    settingsForm = {
      deviceId: device.id,
      alias: device.alias || device.name,
      mapping: '',
      enabled: device.status === 'connected'
    };
    showSettingsModal = true;
  }

  function closeSettings(): void {
    showSettingsModal = false;
    settingsDevice = null;
    settingsForm = {
      deviceId: '',
      alias: '',
      mapping: '',
      enabled: true
    };
  }

  async function saveSettings(): Promise<void> {
    if (!settingsDevice) return;

    try {
      await invoke('update_device_settings', { settings: settingsForm });
      await loadDevices();
      closeSettings();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to save settings';
      console.error('Settings save error:', err);
    }
  }

  // ============================================================================
  // Device Selection
  // ============================================================================

  function selectInput(deviceId: string): void {
    selectedInputId = selectedInputId === deviceId ? null : deviceId;
  }

  function selectOutput(deviceId: string): void {
    selectedOutputId = selectedOutputId === deviceId ? null : deviceId;
  }

  // ============================================================================
  // Status Helpers
  // ============================================================================

  function getStatusColor(status: MidiDevice['status']): string {
    switch (status) {
      case 'connected': return '#4caf50'; // Green
      case 'disconnected': return '#9e9e9e'; // Gray
      case 'error': return '#f44336'; // Red
    }
  }

  function getStatusText(status: MidiDevice['status']): string {
    switch (status) {
      case 'connected': return 'Connected';
      case 'disconnected': return 'Disconnected';
      case 'error': return 'Error';
    }
  }

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  function handleKeydown(e: KeyboardEvent): void {
    if (showSettingsModal) {
      if (e.key === 'Escape') {
        closeSettings();
      } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        saveSettings();
      }
      return;
    }

    if (showDetailsPanel && e.key === 'Escape') {
      closeDetails();
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(async () => {
    await loadDevices();

    // Listen for real-time device detection events
    deviceDetectionUnlisten = await listen('midi-device-changed', async () => {
      await loadDevices();
    });

    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    if (deviceDetectionUnlisten) {
      deviceDetectionUnlisten();
    }
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<WindowBase windowId="midi-devices" title="MIDI Device Manager">
  <div class="device-manager">
    <!-- Header -->
    <div class="header">
      <div class="filter-buttons">
        <button
          class="filter-btn"
          class:active={filterMode === 'all'}
          on:click={() => filterMode = 'all'}
        >
          Show All
        </button>
        <button
          class="filter-btn"
          class:active={filterMode === 'connected'}
          on:click={() => filterMode = 'connected'}
        >
          Connected Only
        </button>
        <button
          class="filter-btn"
          class:active={filterMode === 'configured'}
          on:click={() => filterMode = 'configured'}
        >
          Configured Only
        </button>
      </div>

      <div class="header-actions">
        <button
          class="refresh-btn"
          on:click={refreshDevices}
          disabled={refreshing || loading}
        >
          {refreshing ? '⟳ Refreshing...' : '⟳ Refresh Devices'}
        </button>
      </div>
    </div>

    <!-- Error Display -->
    {#if error}
      <div class="error-banner">
        <span class="error-icon">⚠</span>
        <span class="error-text">{error}</span>
        <button class="error-close" on:click={() => error = null}>×</button>
      </div>
    {/if}

    <!-- Main Content: Two Columns -->
    <div class="columns">
      <!-- Input Devices Column -->
      <div class="column">
        <div class="column-header">
          <h3 class="column-title">Input Devices</h3>
          <span class="device-count">{filteredInputs.length} device{filteredInputs.length !== 1 ? 's' : ''}</span>
        </div>

        <div class="device-list">
          {#if loading}
            <div class="loading-state">
              <p>Loading devices...</p>
            </div>
          {:else if filteredInputs.length === 0}
            <div class="empty-state">
              <p>No input devices found</p>
              <button class="refresh-btn-small" on:click={refreshDevices}>
                Scan for Devices
              </button>
            </div>
          {:else}
            {#each filteredInputs as device (device.id)}
              <div
                class="device-card"
                class:selected={selectedInputId === device.id}
                on:click={() => selectInput(device.id)}
              >
                <div class="device-header">
                  <div class="device-status">
                    <span
                      class="status-indicator"
                      style="background: {getStatusColor(device.status)}"
                    ></span>
                    <span class="status-text">{getStatusText(device.status)}</span>
                  </div>
                  <button
                    class="settings-icon-btn"
                    on:click|stopPropagation={() => openSettings(device)}
                    title="Settings"
                  >
                    ⚙
                  </button>
                </div>

                <div class="device-name">
                  {device.alias || device.name}
                  {#if device.alias && device.alias !== device.name}
                    <span class="device-name-original">({device.name})</span>
                  {/if}
                </div>

                <div class="device-info">
                  <div class="info-item">
                    <span class="info-label">Latency:</span>
                    <span class="info-value">{device.latencyMs.toFixed(1)}ms</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">Ports:</span>
                    <span class="info-value">{device.portCount}</span>
                  </div>
                </div>

                <div class="device-actions">
                  {#if device.status === 'connected'}
                    <button
                      class="btn btn-secondary"
                      on:click|stopPropagation={() => disconnectDevice(device.id, 'input')}
                    >
                      Disconnect
                    </button>
                  {:else}
                    <button
                      class="btn btn-primary"
                      on:click|stopPropagation={() => connectDevice(device.id, 'input')}
                    >
                      Connect
                    </button>
                  {/if}
                  <button
                    class="btn btn-info"
                    on:click|stopPropagation={() => showDetails(device, 'input')}
                  >
                    Details
                  </button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>

      <!-- Output Devices Column -->
      <div class="column">
        <div class="column-header">
          <h3 class="column-title">Output Devices</h3>
          <span class="device-count">{filteredOutputs.length} device{filteredOutputs.length !== 1 ? 's' : ''}</span>
        </div>

        <div class="device-list">
          {#if loading}
            <div class="loading-state">
              <p>Loading devices...</p>
            </div>
          {:else if filteredOutputs.length === 0}
            <div class="empty-state">
              <p>No output devices found</p>
              <button class="refresh-btn-small" on:click={refreshDevices}>
                Scan for Devices
              </button>
            </div>
          {:else}
            {#each filteredOutputs as device (device.id)}
              <div
                class="device-card"
                class:selected={selectedOutputId === device.id}
                class:testing={testingDeviceId === device.id}
                on:click={() => selectOutput(device.id)}
              >
                <div class="device-header">
                  <div class="device-status">
                    <span
                      class="status-indicator"
                      style="background: {getStatusColor(device.status)}"
                    ></span>
                    <span class="status-text">{getStatusText(device.status)}</span>
                  </div>
                  <button
                    class="settings-icon-btn"
                    on:click|stopPropagation={() => openSettings(device)}
                    title="Settings"
                  >
                    ⚙
                  </button>
                </div>

                <div class="device-name">
                  {device.alias || device.name}
                  {#if device.alias && device.alias !== device.name}
                    <span class="device-name-original">({device.name})</span>
                  {/if}
                </div>

                <div class="device-info">
                  <div class="info-item">
                    <span class="info-label">Latency:</span>
                    <span class="info-value">{device.latencyMs.toFixed(1)}ms</span>
                  </div>
                  <div class="info-item">
                    <span class="info-label">Ports:</span>
                    <span class="info-value">{device.portCount}</span>
                  </div>
                </div>

                <div class="device-actions">
                  {#if device.status === 'connected'}
                    <button
                      class="btn btn-secondary"
                      on:click|stopPropagation={() => disconnectDevice(device.id, 'output')}
                    >
                      Disconnect
                    </button>
                    <button
                      class="btn btn-test"
                      on:click|stopPropagation={() => testDevice(device.id)}
                      disabled={testingDeviceId === device.id}
                    >
                      {testingDeviceId === device.id ? '♪ Testing...' : '♪ Test'}
                    </button>
                  {:else}
                    <button
                      class="btn btn-primary"
                      on:click|stopPropagation={() => connectDevice(device.id, 'output')}
                    >
                      Connect
                    </button>
                  {/if}
                  <button
                    class="btn btn-info"
                    on:click|stopPropagation={() => showDetails(device, 'output')}
                  >
                    Details
                  </button>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>

    <!-- Device Details Panel (Side Panel) -->
    {#if showDetailsPanel && detailsDevice}
      <div class="details-overlay" on:click={closeDetails}>
        <div class="details-panel" on:click|stopPropagation>
          <div class="details-header">
            <h3 class="details-title">Device Details</h3>
            <button class="close-btn" on:click={closeDetails}>×</button>
          </div>

          <div class="details-content">
            <div class="detail-row">
              <span class="detail-label">Type:</span>
              <span class="detail-value">{detailsType === 'input' ? 'Input' : 'Output'}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Name:</span>
              <span class="detail-value">{detailsDevice.name}</span>
            </div>

            {#if detailsDevice.alias}
              <div class="detail-row">
                <span class="detail-label">Alias:</span>
                <span class="detail-value">{detailsDevice.alias}</span>
              </div>
            {/if}

            <div class="detail-row">
              <span class="detail-label">Status:</span>
              <span class="detail-value" style="color: {getStatusColor(detailsDevice.status)}">
                {getStatusText(detailsDevice.status)}
              </span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Device ID:</span>
              <span class="detail-value mono">{detailsDevice.id}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Port Count:</span>
              <span class="detail-value">{detailsDevice.portCount}</span>
            </div>

            <div class="detail-row">
              <span class="detail-label">Latency:</span>
              <span class="detail-value">{detailsDevice.latencyMs.toFixed(2)} ms</span>
            </div>

            {#if detailsDevice.manufacturer}
              <div class="detail-row">
                <span class="detail-label">Manufacturer:</span>
                <span class="detail-value">{detailsDevice.manufacturer}</span>
              </div>
            {/if}

            {#if detailsDevice.version}
              <div class="detail-row">
                <span class="detail-label">Version:</span>
                <span class="detail-value">{detailsDevice.version}</span>
              </div>
            {/if}

            <div class="detail-row">
              <span class="detail-label">Configured:</span>
              <span class="detail-value">{detailsDevice.isConfigured ? 'Yes' : 'No'}</span>
            </div>
          </div>

          <div class="details-actions">
            <button class="btn btn-primary" on:click={closeDetails}>
              Close
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Settings Modal -->
    {#if showSettingsModal && settingsDevice}
      <div class="modal-overlay" on:click={closeSettings}>
        <div class="modal" on:click|stopPropagation>
          <div class="modal-header">
            <h3 class="modal-title">Device Settings</h3>
            <button class="close-btn" on:click={closeSettings}>×</button>
          </div>

          <div class="modal-content">
            <div class="form-group">
              <label class="form-label" for="device-name">Device Name:</label>
              <input
                type="text"
                id="device-name"
                class="form-input"
                value={settingsDevice.name}
                disabled
              />
            </div>

            <div class="form-group">
              <label class="form-label" for="device-alias">Alias (Custom Name):</label>
              <input
                type="text"
                id="device-alias"
                class="form-input"
                bind:value={settingsForm.alias}
                placeholder="Enter custom name..."
              />
            </div>

            <div class="form-group">
              <label class="form-label" for="device-mapping">Mapping Configuration:</label>
              <textarea
                id="device-mapping"
                class="form-textarea"
                bind:value={settingsForm.mapping}
                placeholder="Enter mapping configuration (e.g., channel remapping)..."
                rows="4"
              ></textarea>
            </div>

            <div class="form-group">
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  bind:checked={settingsForm.enabled}
                />
                <span>Enable device on startup</span>
              </label>
            </div>

            {#if error}
              <div class="modal-error">
                {error}
              </div>
            {/if}
          </div>

          <div class="modal-actions">
            <button class="btn btn-secondary" on:click={closeSettings}>
              Cancel
            </button>
            <button class="btn btn-primary" on:click={saveSettings}>
              Save Settings
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</WindowBase>

<style>
  .device-manager {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Header */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .filter-buttons {
    display: flex;
    gap: 8px;
  }

  .filter-btn {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .filter-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .filter-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .refresh-btn {
    padding: 6px 16px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.15s ease;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--button-hover, #4a4a4a);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Error Banner */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--error-bg, #5c1a1a);
    border-bottom: 1px solid var(--error-border, #8b2e2e);
  }

  .error-icon {
    font-size: 18px;
    color: var(--error-color, #f44336);
  }

  .error-text {
    flex: 1;
    color: var(--error-text, #ffcdd2);
    font-size: 13px;
  }

  .error-close {
    padding: 4px 8px;
    background: transparent;
    color: var(--error-text, #ffcdd2);
    border: none;
    cursor: pointer;
    font-size: 18px;
  }

  /* Columns Layout */
  .columns {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .column {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .column:last-child {
    border-right: none;
  }

  .column-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--panel-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .column-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .device-count {
    font-size: 12px;
    color: var(--text-secondary, #9e9e9e);
  }

  /* Device List */
  .device-list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary, #9e9e9e);
    font-size: 14px;
  }

  .empty-state {
    gap: 16px;
  }

  .refresh-btn-small {
    padding: 8px 16px;
    background: var(--accent-color, #0078d4);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .refresh-btn-small:hover {
    background: var(--accent-hover, #005a9e);
  }

  /* Device Card */
  .device-card {
    padding: 12px;
    margin-bottom: 12px;
    background: var(--card-bg, #2d2d2d);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .device-card:hover {
    background: var(--card-hover, #353535);
    border-color: var(--accent-color, #0078d4);
  }

  .device-card.selected {
    background: var(--card-selected, #1a3a52);
    border-color: var(--accent-color, #0078d4);
  }

  .device-card.testing {
    animation: pulse 0.6s ease-in-out;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.02); box-shadow: 0 0 10px var(--accent-color, #0078d4); }
  }

  .device-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .device-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-text {
    font-size: 12px;
    color: var(--text-secondary, #9e9e9e);
  }

  .settings-icon-btn {
    padding: 4px 8px;
    background: transparent;
    color: var(--text-secondary, #9e9e9e);
    border: none;
    cursor: pointer;
    font-size: 14px;
    transition: color 0.15s ease;
  }

  .settings-icon-btn:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .device-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 8px;
  }

  .device-name-original {
    font-size: 12px;
    font-weight: 400;
    color: var(--text-secondary, #9e9e9e);
    margin-left: 4px;
  }

  .device-info {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
  }

  .info-item {
    display: flex;
    gap: 4px;
    font-size: 12px;
  }

  .info-label {
    color: var(--text-secondary, #9e9e9e);
  }

  .info-value {
    color: var(--text-primary, #e0e0e0);
    font-weight: 500;
  }

  .device-actions {
    display: flex;
    gap: 6px;
  }

  /* Buttons */
  .btn {
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s ease;
  }

  .btn-primary {
    background: var(--accent-color, #0078d4);
    color: white;
  }

  .btn-primary:hover {
    background: var(--accent-hover, #005a9e);
  }

  .btn-secondary {
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
  }

  .btn-secondary:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .btn-info {
    background: var(--info-bg, #1976d2);
    color: white;
  }

  .btn-info:hover {
    background: var(--info-hover, #1565c0);
  }

  .btn-test {
    background: var(--success-bg, #388e3c);
    color: white;
  }

  .btn-test:hover:not(:disabled) {
    background: var(--success-hover, #2e7d32);
  }

  .btn-test:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Details Panel */
  .details-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .details-panel {
    width: 500px;
    max-height: 80vh;
    background: var(--modal-bg, #2d2d2d);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--header-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .details-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .close-btn {
    padding: 4px 8px;
    background: transparent;
    color: var(--text-secondary, #9e9e9e);
    border: none;
    cursor: pointer;
    font-size: 24px;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .details-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 10px 0;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary, #9e9e9e);
  }

  .detail-value {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    text-align: right;
  }

  .detail-value.mono {
    font-family: monospace;
    font-size: 12px;
  }

  .details-actions {
    padding: 16px;
    background: var(--header-bg, #252525);
    border-top: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    justify-content: flex-end;
  }

  /* Modal */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1001;
  }

  .modal {
    width: 600px;
    max-height: 80vh;
    background: var(--modal-bg, #2d2d2d);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: var(--header-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .modal-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .form-input,
  .form-textarea {
    width: 100%;
    padding: 8px 12px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
  }

  .form-input:focus,
  .form-textarea:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .form-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-textarea {
    resize: vertical;
    min-height: 80px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
  }

  .modal-error {
    padding: 12px;
    background: var(--error-bg, #5c1a1a);
    color: var(--error-text, #ffcdd2);
    border: 1px solid var(--error-border, #8b2e2e);
    border-radius: 4px;
    font-size: 13px;
    margin-top: 16px;
  }

  .modal-actions {
    padding: 16px;
    background: var(--header-bg, #252525);
    border-top: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
