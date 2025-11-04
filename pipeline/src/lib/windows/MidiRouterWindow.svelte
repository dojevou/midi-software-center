<script lang="ts">
  /**
   * MidiRouterWindow - MIDI routing and transformation
   *
   * Task-O-Matic: Create, edit, and manage MIDI routing rules with filters and transforms.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/api/dialog';
  import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

  // ============================================================================
  // Types
  // ============================================================================

  type MessageType =
    | 'NoteOn'
    | 'NoteOff'
    | 'ControlChange'
    | 'PitchBend'
    | 'ProgramChange'
    | 'All';

  interface MidiFilter {
    channelFilter: number | 'all'; // 1-16 or 'all'
    messageTypes: MessageType[];
    noteRangeMin: number; // 0-127
    noteRangeMax: number; // 0-127
    velocityMin: number; // 0-127
    velocityMax: number; // 0-127
  }

  interface MidiTransform {
    transpose: number; // -24 to +24 semitones
    velocityScale: number; // 0.5 to 2.0
    channelRemap: number | null; // 1-16 or null (no remap)
  }

  interface MidiRoute {
    id: string;
    sourceDeviceId: string;
    sourceName: string;
    destinationDeviceId: string;
    destinationName: string;
    filter: MidiFilter;
    transform: MidiTransform;
    enabled: boolean;
    lastActivity: number | null; // timestamp
  }

  interface MidiDevice {
    id: string;
    name: string;
  }

  type SortMode = 'source' | 'destination' | 'activity';

  // ============================================================================
  // Constants
  // ============================================================================

  const DEFAULT_FILTER: MidiFilter = {
    channelFilter: 'all',
    messageTypes: ['All'],
    noteRangeMin: 0,
    noteRangeMax: 127,
    velocityMin: 0,
    velocityMax: 127
  };

  const DEFAULT_TRANSFORM: MidiTransform = {
    transpose: 0,
    velocityScale: 1.0,
    channelRemap: null
  };

  const MESSAGE_TYPES: MessageType[] = [
    'All',
    'NoteOn',
    'NoteOff',
    'ControlChange',
    'PitchBend',
    'ProgramChange'
  ];

  // ============================================================================
  // State
  // ============================================================================

  // Routes
  let routes: MidiRoute[] = [];
  let expandedRouteId: string | null = null;
  let sortMode: SortMode = 'source';

  // Devices
  let inputDevices: MidiDevice[] = [];
  let outputDevices: MidiDevice[] = [];

  // UI state
  let loading = false;
  let error: string | null = null;

  // Modal state
  let showAddModal = false;
  let showEditModal = false;
  let editingRoute: MidiRoute | null = null;

  // Form state
  let formSourceId = '';
  let formDestinationId = '';
  let formFilter: MidiFilter = { ...DEFAULT_FILTER };
  let formTransform: MidiTransform = { ...DEFAULT_TRANSFORM };

  // Testing
  let testingRouteId: string | null = null;

  // ============================================================================
  // Reactive
  // ============================================================================

  $: sortedRoutes = sortRoutes(routes, sortMode);
  $: activeRouteCount = routes.filter(r => r.enabled).length;

  // ============================================================================
  // Sorting
  // ============================================================================

  function sortRoutes(routeList: MidiRoute[], mode: SortMode): MidiRoute[] {
    const sorted = [...routeList];
    switch (mode) {
      case 'source':
        return sorted.sort((a, b) => a.sourceName.localeCompare(b.sourceName));
      case 'destination':
        return sorted.sort((a, b) => a.destinationName.localeCompare(b.destinationName));
      case 'activity':
        return sorted.sort((a, b) => {
          if (a.lastActivity === null) return 1;
          if (b.lastActivity === null) return -1;
          return b.lastActivity - a.lastActivity;
        });
      default:
        return sorted;
    }
  }

  // ============================================================================
  // Route Management
  // ============================================================================

  async function loadRoutes(): Promise<void> {
    loading = true;
    error = null;

    try {
      const [loadedRoutes, inputs, outputs] = await Promise.all([
        invoke<MidiRoute[]>('get_midi_routes'),
        invoke<MidiDevice[]>('get_midi_inputs'),
        invoke<MidiDevice[]>('get_midi_outputs')
      ]);

      routes = loadedRoutes;
      inputDevices = inputs;
      outputDevices = outputs;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load routes';
      console.error('Load routes error:', err);
    } finally {
      loading = false;
    }
  }

  async function saveRoute(): Promise<void> {
    if (!formSourceId || !formDestinationId) {
      error = 'Please select both source and destination devices';
      return;
    }

    const sourceName = inputDevices.find(d => d.id === formSourceId)?.name || 'Unknown';
    const destinationName = outputDevices.find(d => d.id === formDestinationId)?.name || 'Unknown';

    const route: MidiRoute = {
      id: editingRoute?.id || crypto.randomUUID(),
      sourceDeviceId: formSourceId,
      sourceName,
      destinationDeviceId: formDestinationId,
      destinationName,
      filter: { ...formFilter },
      transform: { ...formTransform },
      enabled: true,
      lastActivity: null
    };

    try {
      if (editingRoute) {
        await invoke('update_midi_route', { route });
      } else {
        await invoke('create_midi_route', { route });
      }

      await loadRoutes();
      closeModals();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to save route';
      console.error('Save route error:', err);
    }
  }

  async function deleteRoute(routeId: string): Promise<void> {
    if (!confirm('Are you sure you want to delete this route?')) {
      return;
    }

    try {
      await invoke('delete_midi_route', { routeId });
      await loadRoutes();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to delete route';
      console.error('Delete route error:', err);
    }
  }

  async function toggleRouteEnabled(routeId: string): Promise<void> {
    const route = routes.find(r => r.id === routeId);
    if (!route) return;

    try {
      await invoke('toggle_midi_route', { routeId, enabled: !route.enabled });
      await loadRoutes();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to toggle route';
      console.error('Toggle route error:', err);
    }
  }

  async function testRoute(routeId: string): Promise<void> {
    testingRouteId = routeId;

    try {
      // Send test note through route: Middle C (60), velocity 100, 500ms
      await invoke('test_midi_route', {
        routeId,
        note: 60,
        velocity: 100,
        durationMs: 500
      });

      setTimeout(() => {
        testingRouteId = null;
      }, 600);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to test route';
      console.error('Test route error:', err);
      testingRouteId = null;
    }
  }

  // ============================================================================
  // Modal Management
  // ============================================================================

  function openAddModal(): void {
    resetForm();
    showAddModal = true;
  }

  function openEditModal(route: MidiRoute): void {
    editingRoute = route;
    formSourceId = route.sourceDeviceId;
    formDestinationId = route.destinationDeviceId;
    formFilter = { ...route.filter };
    formTransform = { ...route.transform };
    showEditModal = true;
  }

  function closeModals(): void {
    showAddModal = false;
    showEditModal = false;
    editingRoute = null;
    resetForm();
  }

  function resetForm(): void {
    formSourceId = '';
    formDestinationId = '';
    formFilter = { ...DEFAULT_FILTER };
    formTransform = { ...DEFAULT_TRANSFORM };
  }

  // ============================================================================
  // Import/Export
  // ============================================================================

  async function exportRoutes(): Promise<void> {
    try {
      const filePath = await save({
        defaultPath: 'midi-routes.json',
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (filePath) {
        const json = JSON.stringify(routes, null, 2);
        await writeTextFile(filePath, json);
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to export routes';
      console.error('Export error:', err);
    }
  }

  async function importRoutes(): Promise<void> {
    try {
      const filePath = await open({
        multiple: false,
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }]
      });

      if (filePath && typeof filePath === 'string') {
        const json = await readTextFile(filePath);
        const importedRoutes = JSON.parse(json) as MidiRoute[];

        // Import routes to backend
        await invoke('import_midi_routes', { routes: importedRoutes });
        await loadRoutes();
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to import routes';
      console.error('Import error:', err);
    }
  }

  // ============================================================================
  // Route Expansion
  // ============================================================================

  function toggleExpanded(routeId: string): void {
    expandedRouteId = expandedRouteId === routeId ? null : routeId;
  }

  // ============================================================================
  // Formatting
  // ============================================================================

  function formatLastActivity(timestamp: number | null): string {
    if (!timestamp) return 'Never';

    const now = Date.now();
    const diff = now - timestamp;

    if (diff < 1000) return 'Just now';
    if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return `${Math.floor(diff / 86400000)}d ago`;
  }

  function formatMessageTypes(types: MessageType[]): string {
    if (types.includes('All')) return 'All';
    return types.join(', ');
  }

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  function handleKeydown(e: KeyboardEvent): void {
    if (showAddModal || showEditModal) {
      if (e.key === 'Escape') {
        closeModals();
      } else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        saveRoute();
      }
      return;
    }

    if (e.key === 'n' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      openAddModal();
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(async () => {
    await loadRoutes();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<WindowBase windowId="midi-router" title="MIDI Router">
  <div class="router-window">
    <!-- Header -->
    <div class="header">
      <div class="header-left">
        <button class="btn btn-primary" on:click={openAddModal}>
          + Add Route
        </button>

        <div class="sort-controls">
          <label class="sort-label">Sort by:</label>
          <select class="sort-select" bind:value={sortMode}>
            <option value="source">Source</option>
            <option value="destination">Destination</option>
            <option value="activity">Activity</option>
          </select>
        </div>
      </div>

      <div class="header-right">
        <button class="btn btn-secondary" on:click={importRoutes}>
          Import Routes
        </button>
        <button class="btn btn-secondary" on:click={exportRoutes}>
          Export Routes
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

    <!-- Stats Bar -->
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-label">Total Routes:</span>
        <span class="stat-value">{routes.length}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Active:</span>
        <span class="stat-value">{activeRouteCount}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Inactive:</span>
        <span class="stat-value">{routes.length - activeRouteCount}</span>
      </div>
    </div>

    <!-- Route List -->
    <div class="route-list">
      {#if loading}
        <div class="loading-state">
          <p>Loading routes...</p>
        </div>
      {:else if routes.length === 0}
        <div class="empty-state">
          <p>No routes configured</p>
          <button class="btn btn-primary" on:click={openAddModal}>
            Create First Route
          </button>
        </div>
      {:else}
        <table class="route-table">
          <thead>
            <tr>
              <th class="col-enabled">Enabled</th>
              <th class="col-source">Source</th>
              <th class="col-arrow"></th>
              <th class="col-destination">Destination</th>
              <th class="col-filter">Filter</th>
              <th class="col-transform">Transform</th>
              <th class="col-activity">Last Activity</th>
              <th class="col-actions">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each sortedRoutes as route (route.id)}
              <tr
                class="route-row"
                class:expanded={expandedRouteId === route.id}
                class:disabled={!route.enabled}
                class:testing={testingRouteId === route.id}
              >
                <td class="col-enabled">
                  <input
                    type="checkbox"
                    checked={route.enabled}
                    on:change={() => toggleRouteEnabled(route.id)}
                  />
                </td>
                <td class="col-source" on:click={() => toggleExpanded(route.id)}>
                  {route.sourceName}
                </td>
                <td class="col-arrow" on:click={() => toggleExpanded(route.id)}>
                  →
                </td>
                <td class="col-destination" on:click={() => toggleExpanded(route.id)}>
                  {route.destinationName}
                </td>
                <td class="col-filter" on:click={() => toggleExpanded(route.id)}>
                  Ch {route.filter.channelFilter === 'all' ? 'All' : route.filter.channelFilter}
                </td>
                <td class="col-transform" on:click={() => toggleExpanded(route.id)}>
                  {#if route.transform.transpose !== 0}
                    T{route.transform.transpose > 0 ? '+' : ''}{route.transform.transpose}
                  {:else if route.transform.velocityScale !== 1.0}
                    V×{route.transform.velocityScale}
                  {:else if route.transform.channelRemap !== null}
                    Ch→{route.transform.channelRemap}
                  {:else}
                    None
                  {/if}
                </td>
                <td class="col-activity">
                  {formatLastActivity(route.lastActivity)}
                </td>
                <td class="col-actions">
                  <button
                    class="btn-icon"
                    on:click|stopPropagation={() => testRoute(route.id)}
                    disabled={!route.enabled || testingRouteId === route.id}
                    title="Test Route"
                  >
                    ♪
                  </button>
                  <button
                    class="btn-icon"
                    on:click|stopPropagation={() => openEditModal(route)}
                    title="Edit Route"
                  >
                    ✎
                  </button>
                  <button
                    class="btn-icon btn-delete"
                    on:click|stopPropagation={() => deleteRoute(route.id)}
                    title="Delete Route"
                  >
                    ×
                  </button>
                </td>
              </tr>

              {#if expandedRouteId === route.id}
                <tr class="route-details">
                  <td colspan="8">
                    <div class="details-content">
                      <div class="details-section">
                        <h4 class="details-heading">Filter Configuration</h4>
                        <div class="details-grid">
                          <div class="detail-item">
                            <span class="detail-label">Channel:</span>
                            <span class="detail-value">
                              {route.filter.channelFilter === 'all' ? 'All Channels' : `Channel ${route.filter.channelFilter}`}
                            </span>
                          </div>
                          <div class="detail-item">
                            <span class="detail-label">Message Types:</span>
                            <span class="detail-value">{formatMessageTypes(route.filter.messageTypes)}</span>
                          </div>
                          <div class="detail-item">
                            <span class="detail-label">Note Range:</span>
                            <span class="detail-value">
                              {route.filter.noteRangeMin} - {route.filter.noteRangeMax}
                            </span>
                          </div>
                          <div class="detail-item">
                            <span class="detail-label">Velocity Range:</span>
                            <span class="detail-value">
                              {route.filter.velocityMin} - {route.filter.velocityMax}
                            </span>
                          </div>
                        </div>
                      </div>

                      <div class="details-section">
                        <h4 class="details-heading">Transform Configuration</h4>
                        <div class="details-grid">
                          <div class="detail-item">
                            <span class="detail-label">Transpose:</span>
                            <span class="detail-value">
                              {route.transform.transpose === 0 ? 'None' : `${route.transform.transpose > 0 ? '+' : ''}${route.transform.transpose} semitones`}
                            </span>
                          </div>
                          <div class="detail-item">
                            <span class="detail-label">Velocity Scale:</span>
                            <span class="detail-value">×{route.transform.velocityScale.toFixed(2)}</span>
                          </div>
                          <div class="detail-item">
                            <span class="detail-label">Channel Remap:</span>
                            <span class="detail-value">
                              {route.transform.channelRemap === null ? 'None' : `Channel ${route.transform.channelRemap}`}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      {/if}
    </div>

    <!-- Add/Edit Route Modal -->
    {#if showAddModal || showEditModal}
      <div class="modal-overlay" on:click={closeModals}>
        <div class="modal" on:click|stopPropagation>
          <div class="modal-header">
            <h3 class="modal-title">
              {showAddModal ? 'Add New Route' : 'Edit Route'}
            </h3>
            <button class="close-btn" on:click={closeModals}>×</button>
          </div>

          <div class="modal-content">
            <!-- Device Selection -->
            <div class="form-section">
              <h4 class="section-title">Devices</h4>

              <div class="form-group">
                <label class="form-label" for="source-device">Source (Input):</label>
                <select
                  id="source-device"
                  class="form-select"
                  bind:value={formSourceId}
                >
                  <option value="">Select input device...</option>
                  {#each inputDevices as device}
                    <option value={device.id}>{device.name}</option>
                  {/each}
                </select>
              </div>

              <div class="form-group">
                <label class="form-label" for="dest-device">Destination (Output):</label>
                <select
                  id="dest-device"
                  class="form-select"
                  bind:value={formDestinationId}
                >
                  <option value="">Select output device...</option>
                  {#each outputDevices as device}
                    <option value={device.id}>{device.name}</option>
                  {/each}
                </select>
              </div>
            </div>

            <!-- Filter Configuration -->
            <div class="form-section">
              <h4 class="section-title">Filter</h4>

              <div class="form-row">
                <div class="form-group">
                  <label class="form-label" for="filter-channel">Channel:</label>
                  <select
                    id="filter-channel"
                    class="form-select"
                    bind:value={formFilter.channelFilter}
                  >
                    <option value="all">All Channels</option>
                    {#each Array(16) as _, i}
                      <option value={i + 1}>Channel {i + 1}</option>
                    {/each}
                  </select>
                </div>

                <div class="form-group">
                  <label class="form-label">Message Types:</label>
                  <div class="checkbox-group">
                    {#each MESSAGE_TYPES as msgType}
                      <label class="checkbox-label">
                        <input
                          type="checkbox"
                          checked={formFilter.messageTypes.includes(msgType)}
                          on:change={(e) => {
                            if (e.currentTarget.checked) {
                              formFilter.messageTypes = [...formFilter.messageTypes, msgType];
                            } else {
                              formFilter.messageTypes = formFilter.messageTypes.filter(t => t !== msgType);
                            }
                          }}
                        />
                        <span>{msgType}</span>
                      </label>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="form-row">
                <div class="form-group">
                  <label class="form-label">Note Range:</label>
                  <div class="range-inputs">
                    <input
                      type="number"
                      class="form-input-small"
                      min="0"
                      max="127"
                      bind:value={formFilter.noteRangeMin}
                    />
                    <span>to</span>
                    <input
                      type="number"
                      class="form-input-small"
                      min="0"
                      max="127"
                      bind:value={formFilter.noteRangeMax}
                    />
                  </div>
                </div>

                <div class="form-group">
                  <label class="form-label">Velocity Range:</label>
                  <div class="range-inputs">
                    <input
                      type="number"
                      class="form-input-small"
                      min="0"
                      max="127"
                      bind:value={formFilter.velocityMin}
                    />
                    <span>to</span>
                    <input
                      type="number"
                      class="form-input-small"
                      min="0"
                      max="127"
                      bind:value={formFilter.velocityMax}
                    />
                  </div>
                </div>
              </div>
            </div>

            <!-- Transform Configuration -->
            <div class="form-section">
              <h4 class="section-title">Transform</h4>

              <div class="form-row">
                <div class="form-group">
                  <label class="form-label" for="transpose">Transpose (semitones):</label>
                  <input
                    type="number"
                    id="transpose"
                    class="form-input"
                    min="-24"
                    max="24"
                    bind:value={formTransform.transpose}
                  />
                </div>

                <div class="form-group">
                  <label class="form-label" for="velocity-scale">Velocity Scale:</label>
                  <input
                    type="number"
                    id="velocity-scale"
                    class="form-input"
                    min="0.5"
                    max="2.0"
                    step="0.1"
                    bind:value={formTransform.velocityScale}
                  />
                </div>
              </div>

              <div class="form-group">
                <label class="form-label" for="channel-remap">Channel Remap:</label>
                <select
                  id="channel-remap"
                  class="form-select"
                  bind:value={formTransform.channelRemap}
                >
                  <option value={null}>No Remap</option>
                  {#each Array(16) as _, i}
                    <option value={i + 1}>Channel {i + 1}</option>
                  {/each}
                </select>
              </div>
            </div>

            {#if error}
              <div class="modal-error">
                {error}
              </div>
            {/if}
          </div>

          <div class="modal-actions">
            <button class="btn btn-secondary" on:click={closeModals}>
              Cancel
            </button>
            <button class="btn btn-primary" on:click={saveRoute}>
              {showAddModal ? 'Create Route' : 'Save Changes'}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</WindowBase>

<style>
  .router-window {
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

  .header-left,
  .header-right {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .sort-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sort-label {
    font-size: 13px;
    color: var(--text-secondary, #9e9e9e);
  }

  .sort-select {
    padding: 6px 10px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
  }

  .sort-select:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  /* Buttons */
  .btn {
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
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

  /* Stats Bar */
  .stats-bar {
    display: flex;
    gap: 24px;
    padding: 10px 16px;
    background: var(--panel-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .stat {
    display: flex;
    gap: 6px;
    font-size: 13px;
  }

  .stat-label {
    color: var(--text-secondary, #9e9e9e);
  }

  .stat-value {
    color: var(--text-primary, #e0e0e0);
    font-weight: 600;
  }

  /* Route List */
  .route-list {
    flex: 1;
    overflow-y: auto;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
    color: var(--text-secondary, #9e9e9e);
    font-size: 14px;
  }

  .route-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .route-table thead {
    position: sticky;
    top: 0;
    background: var(--header-bg, #2d2d2d);
    z-index: 1;
  }

  .route-table th {
    padding: 10px 12px;
    text-align: left;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .route-table td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    color: var(--text-primary, #e0e0e0);
  }

  .route-row {
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .route-row:hover {
    background: var(--row-hover, #2d2d2d);
  }

  .route-row.expanded {
    background: var(--row-selected, #1a3a52);
  }

  .route-row.disabled {
    opacity: 0.5;
  }

  .route-row.testing {
    animation: pulse 0.6s ease-in-out;
  }

  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.01); }
  }

  .col-enabled {
    width: 60px;
    text-align: center;
  }

  .col-source,
  .col-destination {
    min-width: 150px;
  }

  .col-arrow {
    width: 30px;
    text-align: center;
    color: var(--accent-color, #0078d4);
    font-weight: bold;
  }

  .col-filter,
  .col-transform {
    min-width: 100px;
  }

  .col-activity {
    width: 120px;
    font-size: 12px;
    color: var(--text-secondary, #9e9e9e);
  }

  .col-actions {
    width: 120px;
    text-align: right;
  }

  .btn-icon {
    padding: 4px 8px;
    background: transparent;
    color: var(--text-secondary, #9e9e9e);
    border: none;
    cursor: pointer;
    font-size: 14px;
    transition: color 0.15s ease;
  }

  .btn-icon:hover:not(:disabled) {
    color: var(--text-primary, #e0e0e0);
  }

  .btn-icon:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .btn-icon.btn-delete:hover {
    color: var(--error-color, #f44336);
  }

  /* Route Details */
  .route-details {
    background: var(--panel-bg, #252525);
  }

  .details-content {
    padding: 16px 20px;
  }

  .details-section {
    margin-bottom: 20px;
  }

  .details-section:last-child {
    margin-bottom: 0;
  }

  .details-heading {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .details-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .detail-label {
    font-size: 12px;
    color: var(--text-secondary, #9e9e9e);
  }

  .detail-value {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
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
    z-index: 1000;
  }

  .modal {
    width: 700px;
    max-height: 85vh;
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

  .modal-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .form-section {
    margin-bottom: 24px;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .form-section:last-child {
    margin-bottom: 0;
    padding-bottom: 0;
    border-bottom: none;
  }

  .section-title {
    margin: 0 0 16px 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .form-label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .form-select,
  .form-input,
  .form-input-small {
    width: 100%;
    padding: 8px 12px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 13px;
  }

  .form-select:focus,
  .form-input:focus,
  .form-input-small:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .form-input-small {
    width: auto;
    padding: 6px 10px;
  }

  .range-inputs {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .checkbox-group {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
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
