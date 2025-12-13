<script lang="ts">
  import { onMount } from 'svelte';
  import { midiDeviceStore, midiDeviceActions } from '$lib/stores/midiDeviceStore';
  import type { MIDIDevice } from '$lib/stores/midiDeviceStore';

  // Routing configuration for each MIDI channel
  interface ChannelRoute {
    channel: number;
    enabled: boolean;
    outputDeviceId: string | null;
    transpose: number;
    velocityScale: number;
    filterNotes: boolean;
    filterCC: boolean;
    filterProgramChange: boolean;
    filterPitchBend: boolean;
    filterAftertouch: boolean;
    channelRemap: number | null; // null = keep original, 1-16 = remap to specific channel
  }

  // Global routing settings
  interface RoutingSettings {
    masterEnabled: boolean;
    defaultOutputDeviceId: string | null;
    midiThruEnabled: boolean;
    panicOnDisconnect: boolean;
    autoReconnect: boolean;
  }

  // Initialize channel routes (1-16)
  let channelRoutes: ChannelRoute[] = Array.from({ length: 16 }, (_, i) => ({
    channel: i + 1,
    enabled: true,
    outputDeviceId: null,
    transpose: 0,
    velocityScale: 100,
    filterNotes: false,
    filterCC: false,
    filterProgramChange: false,
    filterPitchBend: false,
    filterAftertouch: false,
    channelRemap: null,
  }));

  let routingSettings: RoutingSettings = {
    masterEnabled: true,
    defaultOutputDeviceId: null,
    midiThruEnabled: false,
    panicOnDisconnect: true,
    autoReconnect: true,
  };

  // View mode: 'matrix' for grid view, 'list' for detailed list
  let viewMode: 'matrix' | 'list' = 'matrix';

  // Selected channel for detailed editing
  let selectedChannel: number | null = null;

  // Filter for showing only enabled channels
  let showOnlyEnabled = false;

  // Get output devices from store
  $: outputDevices = $midiDeviceStore.devices.filter((d: MIDIDevice) => d.type === 'output' || d.type === 'both');
  // inputDevices available for future use
  $: _inputDevices = $midiDeviceStore.devices.filter((d: MIDIDevice) => d.type === 'input' || d.type === 'both');

  // Filtered channel routes
  $: filteredRoutes = showOnlyEnabled ? channelRoutes.filter((r) => r.enabled) : channelRoutes;

  // Selected route for detail panel (reactive - allows binding)
  $: selectedRoute = selectedChannel !== null ? channelRoutes.find((r) => r.channel === selectedChannel) : undefined;

  // Count active routes
  $: activeRouteCount = channelRoutes.filter((r) => r.enabled && r.outputDeviceId).length;

  // Load saved routing configuration
  onMount(() => {
    loadRoutingConfig();
    // Scan for devices on mount
    void midiDeviceActions.scanDevices();
  });

  function loadRoutingConfig() {
    try {
      const saved = localStorage.getItem('midi-io-routing');
      if (saved) {
        const config = JSON.parse(saved);
        if (config.channelRoutes) {
          channelRoutes = config.channelRoutes;
        }
        if (config.routingSettings) {
          routingSettings = config.routingSettings;
        }
      }
    } catch (e) {
      console.error('Failed to load routing config:', e);
    }
  }

  function saveRoutingConfig() {
    try {
      localStorage.setItem(
        'midi-io-routing',
        JSON.stringify({
          channelRoutes,
          routingSettings,
        })
      );
    } catch (e) {
      console.error('Failed to save routing config:', e);
    }
  }

  // Save on any change
  $: if (channelRoutes || routingSettings) {
    saveRoutingConfig();
  }

  function toggleChannel(channel: number) {
    const route = channelRoutes.find((r) => r.channel === channel);
    if (route) {
      route.enabled = !route.enabled;
      channelRoutes = [...channelRoutes];
    }
  }

  function setChannelOutput(channel: number, deviceId: string | null) {
    const route = channelRoutes.find((r) => r.channel === channel);
    if (route) {
      route.outputDeviceId = deviceId;
      channelRoutes = [...channelRoutes];
    }
  }

  function selectChannel(channel: number) {
    selectedChannel = selectedChannel === channel ? null : channel;
  }

  function getDeviceName(deviceId: string | null): string {
    if (!deviceId) {
      return 'None';
    }
    const device = outputDevices.find((d: MIDIDevice) => d.id === deviceId);
    return device?.name || 'Unknown';
  }

  function getDeviceShortName(deviceId: string | null): string {
    const name = getDeviceName(deviceId);
    return name.length > 12 ? name.substring(0, 10) + '...' : name;
  }

  function enableAllChannels() {
    channelRoutes = channelRoutes.map((r) => ({ ...r, enabled: true }));
  }

  function disableAllChannels() {
    channelRoutes = channelRoutes.map((r) => ({ ...r, enabled: false }));
  }

  function routeAllToDevice(deviceId: string | null) {
    channelRoutes = channelRoutes.map((r) => ({ ...r, outputDeviceId: deviceId }));
  }

  function resetAllRoutes() {
    channelRoutes = Array.from({ length: 16 }, (_, i) => ({
      channel: i + 1,
      enabled: true,
      outputDeviceId: null,
      transpose: 0,
      velocityScale: 100,
      filterNotes: false,
      filterCC: false,
      filterProgramChange: false,
      filterPitchBend: false,
      filterAftertouch: false,
      channelRemap: null,
    }));
  }

  function sendPanic() {
    void midiDeviceActions.panic();
  }

  function refreshDevices() {
    void midiDeviceActions.scanDevices();
  }

  // Get channel color based on routing status
  function getChannelColor(route: ChannelRoute): string {
    if (!route.enabled) {
      return 'var(--channel-disabled, #444)';
    }
    if (!route.outputDeviceId) {
      return 'var(--channel-unrouted, #666)';
    }
    return 'var(--channel-routed, #4a9eff)';
  }

  // Format transpose value
  function formatTranspose(value: number): string {
    if (value === 0) {
      return '0';
    }
    return value > 0 ? `+${value}` : `${value}`;
  }
</script>

<div class="midi-io-setup">
  <!-- Header with global controls -->
  <div class="header">
    <div class="header-left">
      <label class="master-toggle">
        <input type="checkbox" bind:checked={routingSettings.masterEnabled} />
        <span class="toggle-label">Master Enable</span>
      </label>
      <span class="route-count">{activeRouteCount}/16 Active</span>
    </div>

    <div class="header-center">
      <div class="view-toggle">
        <button class:active={viewMode === 'matrix'} on:click={() => (viewMode = 'matrix')} title="Matrix View">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="1" width="6" height="6" />
            <rect x="9" y="1" width="6" height="6" />
            <rect x="1" y="9" width="6" height="6" />
            <rect x="9" y="9" width="6" height="6" />
          </svg>
        </button>
        <button class:active={viewMode === 'list'} on:click={() => (viewMode = 'list')} title="List View">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="2" width="14" height="3" />
            <rect x="1" y="7" width="14" height="3" />
            <rect x="1" y="12" width="14" height="3" />
          </svg>
        </button>
      </div>
    </div>

    <div class="header-right">
      <button class="btn-small" on:click={refreshDevices} title="Refresh Devices">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 3a5 5 0 0 0-5 5h2l-3 3-3-3h2a7 7 0 0 1 14 0h-2a5 5 0 0 0-5-5z" />
          <path d="M8 13a5 5 0 0 0 5-5h-2l3-3 3 3h-2a7 7 0 0 1-14 0h2a5 5 0 0 0 5 5z" />
        </svg>
      </button>
      <button class="btn-small btn-panic" on:click={sendPanic} title="MIDI Panic (All Notes Off)"> PANIC </button>
    </div>
  </div>

  <!-- Quick Actions Bar -->
  <div class="quick-actions">
    <button class="btn-action" on:click={enableAllChannels}>Enable All</button>
    <button class="btn-action" on:click={disableAllChannels}>Disable All</button>
    <select class="route-all-select" on:change={(e) => routeAllToDevice(e.currentTarget.value || null)}>
      <option value="">Route All To...</option>
      {#each outputDevices as device (device.id)}
        <option value={device.id}>{device.name}</option>
      {/each}
    </select>
    <button class="btn-action btn-reset" on:click={resetAllRoutes}>Reset All</button>
    <label class="filter-toggle">
      <input type="checkbox" bind:checked={showOnlyEnabled} />
      <span>Show Only Enabled</span>
    </label>
  </div>

  <!-- Main Content -->
  <div class="content">
    {#if viewMode === 'matrix'}
      <!-- Matrix View -->
      <div class="matrix-view">
        <div class="matrix-grid">
          <!-- Header row with output devices -->
          <div class="matrix-header">
            <div class="matrix-corner">CH</div>
            <div class="matrix-device-header">None</div>
            {#each outputDevices as device (device.id)}
              <div class="matrix-device-header" title={device.name}>
                {getDeviceShortName(device.id)}
              </div>
            {/each}
          </div>

          <!-- Channel rows -->
          {#each filteredRoutes as route (route.channel)}
            <div class="matrix-row" class:disabled={!route.enabled}>
              <div
                class="matrix-channel"
                class:selected={selectedChannel === route.channel}
                on:click={() => selectChannel(route.channel)}
                on:keydown={(e) => e.key === 'Enter' && selectChannel(route.channel)}
                role="button"
                tabindex="0"
                style="background-color: {getChannelColor(route)}"
              >
                <span class="channel-number">{route.channel}</span>
                <button class="channel-toggle" on:click|stopPropagation={() => toggleChannel(route.channel)} title={route.enabled ? 'Disable' : 'Enable'}>
                  {route.enabled ? '●' : '○'}
                </button>
              </div>

              <!-- None option -->
              <div
                class="matrix-cell"
                class:active={route.outputDeviceId === null}
                on:click={() => setChannelOutput(route.channel, null)}
                on:keydown={(e) => e.key === 'Enter' && setChannelOutput(route.channel, null)}
                role="button"
                tabindex="0"
              >
                {#if route.outputDeviceId === null}
                  <span class="cell-indicator">●</span>
                {/if}
              </div>

              <!-- Device options -->
              {#each outputDevices as device (device.id)}
                <div
                  class="matrix-cell"
                  class:active={route.outputDeviceId === device.id}
                  class:device-connected={device.isConnected}
                  on:click={() => setChannelOutput(route.channel, device.id)}
                  on:keydown={(e) => e.key === 'Enter' && setChannelOutput(route.channel, device.id)}
                  role="button"
                  tabindex="0"
                >
                  {#if route.outputDeviceId === device.id}
                    <span class="cell-indicator">●</span>
                  {/if}
                </div>
              {/each}
            </div>
          {/each}
        </div>

        <!-- Channel Detail Panel (shown when channel selected) -->
        {#if selectedChannel !== null && selectedRoute}
            <div class="channel-detail">
              <div class="detail-header">
                <h3>Channel {selectedChannel} Settings</h3>
                <button class="btn-close" on:click={() => (selectedChannel = null)}>×</button>
              </div>

              <div class="detail-content">
                <div class="detail-row">
                  <label>Output Device</label>
                  <select bind:value={selectedRoute.outputDeviceId} on:change={() => (channelRoutes = [...channelRoutes])}>
                    <option value={null}>None</option>
                    {#each outputDevices as device (device.id)}
                      <option value={device.id}>{device.name}</option>
                    {/each}
                  </select>
                </div>

                <div class="detail-row">
                  <label>Channel Remap</label>
                  <select bind:value={selectedRoute.channelRemap} on:change={() => (channelRoutes = [...channelRoutes])}>
                    <option value={null}>Keep Original ({selectedChannel})</option>
                    {#each Array.from({ length: 16 }, (_, i) => i + 1) as ch (ch)}
                      <option value={ch}>Channel {ch}</option>
                    {/each}
                  </select>
                </div>

                <div class="detail-row">
                  <label>Transpose</label>
                  <div class="transpose-control">
                    <button on:click={() => { selectedRoute.transpose = Math.max(-48, selectedRoute.transpose - 12); channelRoutes = [...channelRoutes]; }}>-Oct</button>
                    <button on:click={() => { selectedRoute.transpose = Math.max(-48, selectedRoute.transpose - 1); channelRoutes = [...channelRoutes]; }}>-1</button>
                    <span class="transpose-value">{formatTranspose(selectedRoute.transpose)}</span>
                    <button on:click={() => { selectedRoute.transpose = Math.min(48, selectedRoute.transpose + 1); channelRoutes = [...channelRoutes]; }}>+1</button>
                    <button on:click={() => { selectedRoute.transpose = Math.min(48, selectedRoute.transpose + 12); channelRoutes = [...channelRoutes]; }}>+Oct</button>
                  </div>
                </div>

                <div class="detail-row">
                  <label>Velocity Scale</label>
                  <div class="velocity-control">
                    <input type="range" min="0" max="200" bind:value={selectedRoute.velocityScale} on:input={() => (channelRoutes = [...channelRoutes])} />
                    <span class="velocity-value">{selectedRoute.velocityScale}%</span>
                  </div>
                </div>

                <div class="detail-section">
                  <h4>Message Filters</h4>
                  <div class="filter-grid">
                    <label>
                      <input type="checkbox" bind:checked={selectedRoute.filterNotes} on:change={() => (channelRoutes = [...channelRoutes])} />
                      Block Notes
                    </label>
                    <label>
                      <input type="checkbox" bind:checked={selectedRoute.filterCC} on:change={() => (channelRoutes = [...channelRoutes])} />
                      Block CC
                    </label>
                    <label>
                      <input type="checkbox" bind:checked={selectedRoute.filterProgramChange} on:change={() => (channelRoutes = [...channelRoutes])} />
                      Block Program Change
                    </label>
                    <label>
                      <input type="checkbox" bind:checked={selectedRoute.filterPitchBend} on:change={() => (channelRoutes = [...channelRoutes])} />
                      Block Pitch Bend
                    </label>
                    <label>
                      <input type="checkbox" bind:checked={selectedRoute.filterAftertouch} on:change={() => (channelRoutes = [...channelRoutes])} />
                      Block Aftertouch
                    </label>
                  </div>
                </div>
              </div>
            </div>
        {/if}
      </div>
    {:else}
      <!-- List View -->
      <div class="list-view">
        {#each filteredRoutes as route (route.channel)}
          <div class="list-item" class:disabled={!route.enabled} class:selected={selectedChannel === route.channel}>
            <div class="list-item-header">
              <div class="list-channel" style="background-color: {getChannelColor(route)}">
                <span>{route.channel}</span>
              </div>

              <label class="list-enable">
                <input type="checkbox" bind:checked={route.enabled} on:change={() => (channelRoutes = [...channelRoutes])} />
                <span>Enabled</span>
              </label>

              <div class="list-output">
                <label>Output:</label>
                <select bind:value={route.outputDeviceId} on:change={() => (channelRoutes = [...channelRoutes])}>
                  <option value={null}>None</option>
                  {#each outputDevices as device (device.id)}
                    <option value={device.id}>{device.name}</option>
                  {/each}
                </select>
              </div>

              <div class="list-remap">
                <label>Remap:</label>
                <select bind:value={route.channelRemap} on:change={() => (channelRoutes = [...channelRoutes])}>
                  <option value={null}>-</option>
                  {#each Array.from({ length: 16 }, (_, i) => i + 1) as ch (ch)}
                    <option value={ch}>{ch}</option>
                  {/each}
                </select>
              </div>

              <div class="list-transpose">
                <label>Trans:</label>
                <input type="number" min="-48" max="48" bind:value={route.transpose} on:input={() => (channelRoutes = [...channelRoutes])} />
              </div>

              <div class="list-velocity">
                <label>Vel:</label>
                <input type="number" min="0" max="200" bind:value={route.velocityScale} on:input={() => (channelRoutes = [...channelRoutes])} />
                <span>%</span>
              </div>

              <button class="list-expand" on:click={() => selectChannel(route.channel)} title="Expand/Collapse">
                {selectedChannel === route.channel ? '▼' : '▶'}
              </button>
            </div>

            {#if selectedChannel === route.channel}
              <div class="list-item-detail">
                <div class="filter-section">
                  <span class="filter-label">Block:</span>
                  <label><input type="checkbox" bind:checked={route.filterNotes} on:change={() => (channelRoutes = [...channelRoutes])} /> Notes</label>
                  <label><input type="checkbox" bind:checked={route.filterCC} on:change={() => (channelRoutes = [...channelRoutes])} /> CC</label>
                  <label><input type="checkbox" bind:checked={route.filterProgramChange} on:change={() => (channelRoutes = [...channelRoutes])} /> PC</label>
                  <label><input type="checkbox" bind:checked={route.filterPitchBend} on:change={() => (channelRoutes = [...channelRoutes])} /> Bend</label>
                  <label><input type="checkbox" bind:checked={route.filterAftertouch} on:change={() => (channelRoutes = [...channelRoutes])} /> AT</label>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Footer with global settings -->
  <div class="footer">
    <div class="footer-settings">
      <label>
        <input type="checkbox" bind:checked={routingSettings.midiThruEnabled} />
        <span>MIDI Thru</span>
      </label>
      <label>
        <input type="checkbox" bind:checked={routingSettings.panicOnDisconnect} />
        <span>Panic on Disconnect</span>
      </label>
      <label>
        <input type="checkbox" bind:checked={routingSettings.autoReconnect} />
        <span>Auto-Reconnect</span>
      </label>
    </div>

    <div class="footer-default">
      <label>Default Output:</label>
      <select bind:value={routingSettings.defaultOutputDeviceId}>
        <option value={null}>None</option>
        {#each outputDevices as device (device.id)}
          <option value={device.id}>{device.name}</option>
        {/each}
      </select>
    </div>
  </div>
</div>

<style>
  .midi-io-setup {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--bg-primary, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    font-size: 12px;
  }

  /* Header */
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background-color: var(--bg-secondary, #252525);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .header-left,
  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .master-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .master-toggle input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-color, #4a9eff);
  }

  .toggle-label {
    font-weight: 600;
  }

  .route-count {
    color: var(--text-secondary, #888);
    font-size: 11px;
  }

  .view-toggle {
    display: flex;
    gap: 2px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-radius: 4px;
    padding: 2px;
  }

  .view-toggle button {
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    padding: 4px 8px;
    cursor: pointer;
    border-radius: 3px;
    transition: all 0.15s;
  }

  .view-toggle button:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .view-toggle button.active {
    background-color: var(--accent-color, #4a9eff);
    color: white;
  }

  .btn-small {
    background-color: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s;
  }

  .btn-small:hover {
    background-color: var(--bg-hover, #353535);
  }

  .btn-panic {
    background-color: var(--panic-bg, #8b0000);
    border-color: var(--panic-border, #a00);
    color: white;
    font-weight: 600;
  }

  .btn-panic:hover {
    background-color: var(--panic-hover, #a00);
  }

  /* Quick Actions */
  .quick-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-bottom: 1px solid var(--border-color, #333);
    flex-wrap: wrap;
  }

  .btn-action {
    background-color: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 10px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 11px;
    transition: all 0.15s;
  }

  .btn-action:hover {
    background-color: var(--bg-hover, #353535);
  }

  .btn-reset {
    color: var(--warning-color, #ff9800);
    border-color: var(--warning-color, #ff9800);
  }

  .route-all-select {
    background-color: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 8px;
    border-radius: 3px;
    font-size: 11px;
  }

  .filter-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    cursor: pointer;
    color: var(--text-secondary, #888);
    font-size: 11px;
  }

  .filter-toggle input {
    accent-color: var(--accent-color, #4a9eff);
  }

  /* Content */
  .content {
    flex: 1;
    overflow: auto;
    padding: 8px;
  }

  /* Matrix View */
  .matrix-view {
    display: flex;
    gap: 12px;
  }

  .matrix-grid {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .matrix-header {
    display: flex;
    gap: 2px;
    position: sticky;
    top: 0;
    background-color: var(--bg-primary, #1a1a1a);
    z-index: 10;
    padding-bottom: 4px;
  }

  .matrix-corner {
    width: 60px;
    min-width: 60px;
    padding: 4px;
    font-weight: 600;
    text-align: center;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-radius: 3px;
  }

  .matrix-device-header {
    flex: 1;
    min-width: 60px;
    max-width: 100px;
    padding: 4px;
    text-align: center;
    font-size: 10px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-radius: 3px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .matrix-row {
    display: flex;
    gap: 2px;
  }

  .matrix-row.disabled {
    opacity: 0.5;
  }

  .matrix-channel {
    width: 60px;
    min-width: 60px;
    padding: 4px 6px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .matrix-channel:hover {
    filter: brightness(1.2);
  }

  .matrix-channel.selected {
    outline: 2px solid var(--accent-color, #4a9eff);
    outline-offset: 1px;
  }

  .channel-number {
    font-weight: 600;
    font-size: 11px;
  }

  .channel-toggle {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 10px;
    padding: 0;
    opacity: 0.7;
  }

  .channel-toggle:hover {
    opacity: 1;
  }

  .matrix-cell {
    flex: 1;
    min-width: 60px;
    max-width: 100px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-secondary, #252525);
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.15s;
    border: 1px solid transparent;
  }

  .matrix-cell:hover {
    background-color: var(--bg-hover, #353535);
    border-color: var(--border-color, #444);
  }

  .matrix-cell.active {
    background-color: var(--accent-color, #4a9eff);
    color: white;
  }

  .matrix-cell.device-connected {
    border-color: var(--success-color, #4caf50);
  }

  .cell-indicator {
    font-size: 10px;
  }

  /* Channel Detail Panel */
  .channel-detail {
    width: 280px;
    min-width: 280px;
    background-color: var(--bg-secondary, #252525);
    border-radius: 6px;
    border: 1px solid var(--border-color, #333);
    overflow: hidden;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .detail-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .btn-close {
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    font-size: 18px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .btn-close:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .detail-content {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .detail-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .detail-row label {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .detail-row select {
    background-color: var(--bg-tertiary, #1e1e1e);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 6px 8px;
    border-radius: 3px;
    font-size: 12px;
  }

  .transpose-control {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .transpose-control button {
    background-color: var(--bg-tertiary, #1e1e1e);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 8px;
    border-radius: 3px;
    cursor: pointer;
    font-size: 10px;
  }

  .transpose-control button:hover {
    background-color: var(--bg-hover, #353535);
  }

  .transpose-value {
    min-width: 40px;
    text-align: center;
    font-weight: 600;
    font-family: monospace;
  }

  .velocity-control {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .velocity-control input[type='range'] {
    flex: 1;
    accent-color: var(--accent-color, #4a9eff);
  }

  .velocity-value {
    min-width: 40px;
    text-align: right;
    font-family: monospace;
  }

  .detail-section {
    border-top: 1px solid var(--border-color, #333);
    padding-top: 12px;
  }

  .detail-section h4 {
    margin: 0 0 8px 0;
    font-size: 11px;
    color: var(--text-secondary, #888);
    text-transform: uppercase;
  }

  .filter-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .filter-grid label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .filter-grid input {
    accent-color: var(--accent-color, #4a9eff);
  }

  /* List View */
  .list-view {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .list-item {
    background-color: var(--bg-secondary, #252525);
    border-radius: 4px;
    border: 1px solid var(--border-color, #333);
    overflow: hidden;
  }

  .list-item.disabled {
    opacity: 0.6;
  }

  .list-item.selected {
    border-color: var(--accent-color, #4a9eff);
  }

  .list-item-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
  }

  .list-channel {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    font-weight: 700;
    font-size: 14px;
  }

  .list-enable {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    font-size: 11px;
  }

  .list-enable input {
    accent-color: var(--accent-color, #4a9eff);
  }

  .list-output,
  .list-remap,
  .list-transpose,
  .list-velocity {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .list-output label,
  .list-remap label,
  .list-transpose label,
  .list-velocity label {
    font-size: 10px;
    color: var(--text-secondary, #888);
  }

  .list-output select,
  .list-remap select {
    background-color: var(--bg-tertiary, #1e1e1e);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 6px;
    border-radius: 3px;
    font-size: 11px;
  }

  .list-output select {
    min-width: 120px;
  }

  .list-remap select {
    width: 50px;
  }

  .list-transpose input,
  .list-velocity input {
    width: 50px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 6px;
    border-radius: 3px;
    font-size: 11px;
    text-align: center;
  }

  .list-velocity span {
    font-size: 10px;
    color: var(--text-secondary, #888);
  }

  .list-expand {
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    cursor: pointer;
    padding: 4px;
    margin-left: auto;
  }

  .list-expand:hover {
    color: var(--text-primary, #e0e0e0);
  }

  .list-item-detail {
    padding: 8px 12px;
    background-color: var(--bg-tertiary, #1e1e1e);
    border-top: 1px solid var(--border-color, #333);
  }

  .filter-section {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .filter-label {
    font-size: 10px;
    color: var(--text-secondary, #888);
    font-weight: 600;
  }

  .filter-section label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .filter-section input {
    accent-color: var(--accent-color, #4a9eff);
  }

  /* Footer */
  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background-color: var(--bg-secondary, #252525);
    border-top: 1px solid var(--border-color, #333);
  }

  .footer-settings {
    display: flex;
    gap: 16px;
  }

  .footer-settings label {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .footer-settings input {
    accent-color: var(--accent-color, #4a9eff);
  }

  .footer-default {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .footer-default label {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .footer-default select {
    background-color: var(--bg-tertiary, #1e1e1e);
    border: 1px solid var(--border-color, #444);
    color: var(--text-primary, #e0e0e0);
    padding: 4px 8px;
    border-radius: 3px;
    font-size: 11px;
  }
</style>
