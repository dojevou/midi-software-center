# Day 5, Part 4B: Routing Frontend

**Duration:** 1.5 hours
**Prerequisites:** Part 4A complete
**Files to create:** 2

---

## Overview

Create routing matrix UI:
1. Routing matrix component
2. Send controls
3. Bus management
4. Visual routing display

---

## Step 1: TypeScript Types (10 min)

Add to `app/src/lib/types/daw.ts`:

```typescript
export type BusType = 'Master' | 'Aux' | 'Group';

export interface Bus {
  id: number;
  name: string;
  busType: BusType;
  gainDb: number;
  muted: boolean;
  enabled: boolean;
}

export interface Send {
  fromTrack: number;
  toBus: number;
  level: number;
  preFader: boolean;
  enabled: boolean;
}

export interface RoutingConfig {
  buses: Record<number, Bus>;
  sends: Send[];
  trackOutputs: Record<number, number>;
}
```

---

## Step 2: Routing API (10 min)

Create `app/src/lib/api/routingApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Bus, BusType, Send, RoutingConfig } from '../types/daw';

export class RoutingApi {
  static async getRoutingConfig(): Promise<RoutingConfig> {
    return await invoke<RoutingConfig>('get_routing_config');
  }

  static async addBus(busId: number, name: string, busType: BusType): Promise<Bus> {
    return await invoke<Bus>('add_bus', { busId, name, busType });
  }

  static async removeBus(busId: number): Promise<Bus> {
    return await invoke<Bus>('remove_bus', { busId });
  }

  static async addSend(
    fromTrack: number,
    toBus: number,
    level: number,
    preFader: boolean
  ): Promise<Send> {
    return await invoke<Send>('add_send', { fromTrack, toBus, level, preFader });
  }

  static async removeSend(fromTrack: number, toBus: number): Promise<void> {
    await invoke('remove_send', { fromTrack, toBus });
  }

  static async setSendLevel(
    fromTrack: number,
    toBus: number,
    level: number
  ): Promise<void> {
    await invoke('set_send_level', { fromTrack, toBus, level });
  }

  static async setTrackOutput(trackId: number, busId: number): Promise<void> {
    await invoke('set_track_output', { trackId, busId });
  }

  static async createDefaultAuxes(count: number): Promise<void> {
    await invoke('create_default_auxes', { count });
  }
}
```

---

## Step 3: Routing Matrix Component (50 min)

Create `app/src/lib/components/DAW/RoutingMatrix.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { RoutingApi } from '../../api/routingApi';
  import { mixerStore } from '../../stores/mixerStore';
  import type { RoutingConfig, Send } from '../../types/daw';

  let routingConfig: RoutingConfig | null = null;
  let showAddBus = false;
  let newBusName = '';
  let newBusType: 'Aux' | 'Group' = 'Aux';

  $: tracks = $mixerStore.tracks;
  $: buses = routingConfig ? Object.values(routingConfig.buses) : [];

  onMount(async () => {
    await loadRouting();

    // Create default auxes if none exist
    const auxCount = buses.filter((b) => b.busType === 'Aux').length;
    if (auxCount === 0) {
      await RoutingApi.createDefaultAuxes(4);
      await loadRouting();
    }
  });

  async function loadRouting() {
    try {
      routingConfig = await RoutingApi.getRoutingConfig();
    } catch (error) {
      console.error('Failed to load routing:', error);
    }
  }

  async function handleAddBus() {
    if (!newBusName.trim()) return;

    const busId = Math.max(...buses.map((b) => b.id), 0) + 1;

    try {
      await RoutingApi.addBus(busId, newBusName, newBusType);
      await loadRouting();
      newBusName = '';
      showAddBus = false;
    } catch (error) {
      console.error('Failed to add bus:', error);
    }
  }

  async function handleRemoveBus(busId: number) {
    if (!confirm(`Remove bus ${busId}?`)) return;

    try {
      await RoutingApi.removeBus(busId);
      await loadRouting();
    } catch (error) {
      console.error('Failed to remove bus:', error);
    }
  }

  async function toggleSend(trackId: number, busId: number) {
    if (!routingConfig) return;

    const existingSend = routingConfig.sends.find(
      (s) => s.fromTrack === trackId && s.toBus === busId
    );

    try {
      if (existingSend) {
        await RoutingApi.removeSend(trackId, busId);
      } else {
        await RoutingApi.addSend(trackId, busId, 0.5, false);
      }
      await loadRouting();
    } catch (error) {
      console.error('Failed to toggle send:', error);
    }
  }

  async function handleSendLevelChange(trackId: number, busId: number, level: number) {
    try {
      await RoutingApi.setSendLevel(trackId, busId, level);
    } catch (error) {
      console.error('Failed to set send level:', error);
    }
  }

  function getSend(trackId: number, busId: number): Send | null {
    if (!routingConfig) return null;
    return (
      routingConfig.sends.find(
        (s) => s.fromTrack === trackId && s.toBus === busId
      ) || null
    );
  }

  function getTrackOutput(trackId: number): number {
    if (!routingConfig) return 0;
    return routingConfig.trackOutputs[trackId] || 0;
  }

  async function handleTrackOutputChange(trackId: number, busId: number) {
    try {
      await RoutingApi.setTrackOutput(trackId, busId);
      await loadRouting();
    } catch (error) {
      console.error('Failed to set track output:', error);
    }
  }
</script>

<div class="routing-matrix">
  <div class="matrix-header">
    <h3>Routing Matrix</h3>
    <button class="add-btn" on:click={() => (showAddBus = !showAddBus)}>
      {showAddBus ? '✕' : '+'} Add Bus
    </button>
  </div>

  {#if showAddBus}
    <div class="add-bus-form">
      <input
        type="text"
        placeholder="Bus name..."
        bind:value={newBusName}
        on:keydown={(e) => e.key === 'Enter' && handleAddBus()}
      />
      <select bind:value={newBusType}>
        <option value="Aux">Aux</option>
        <option value="Group">Group</option>
      </select>
      <button on:click={handleAddBus}>Add</button>
    </div>
  {/if}

  {#if routingConfig && tracks.length > 0}
    <div class="matrix-container">
      <table class="matrix-table">
        <thead>
          <tr>
            <th class="track-header">Track</th>
            <th class="output-header">Output</th>
            {#each buses as bus (bus.id)}
              {#if bus.busType !== 'Master'}
                <th class="bus-header">
                  <div class="bus-name">{bus.name}</div>
                  {#if bus.id !== 0}
                    <button
                      class="remove-bus-btn"
                      on:click={() => handleRemoveBus(bus.id)}
                      title="Remove bus"
                    >
                      ✕
                    </button>
                  {/if}
                </th>
              {/if}
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each tracks as track (track.trackId)}
            <tr>
              <td class="track-name">{track.name}</td>

              <!-- Track output selector -->
              <td class="output-cell">
                <select
                  value={getTrackOutput(track.trackId)}
                  on:change={(e) =>
                    handleTrackOutputChange(
                      track.trackId,
                      parseInt(e.currentTarget.value)
                    )}
                >
                  {#each buses as bus (bus.id)}
                    <option value={bus.id}>{bus.name}</option>
                  {/each}
                </select>
              </td>

              <!-- Send matrix -->
              {#each buses as bus (bus.id)}
                {#if bus.busType !== 'Master'}
                  {@const send = getSend(track.trackId, bus.id)}
                  <td class="send-cell">
                    {#if send}
                      <div class="send-active">
                        <input
                          type="range"
                          min="0"
                          max="1"
                          step="0.01"
                          value={send.level}
                          on:input={(e) =>
                            handleSendLevelChange(
                              track.trackId,
                              bus.id,
                              parseFloat(e.currentTarget.value)
                            )}
                          title="Send level: {Math.round(send.level * 100)}%"
                        />
                        <button
                          class="send-btn active"
                          on:click={() => toggleSend(track.trackId, bus.id)}
                        >
                          ●
                        </button>
                      </div>
                    {:else}
                      <button
                        class="send-btn"
                        on:click={() => toggleSend(track.trackId, bus.id)}
                        title="Add send"
                      >
                        ○
                      </button>
                    {/if}
                  </td>
                {/if}
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <p class="empty-state">No tracks loaded</p>
  {/if}
</div>

<style>
  .routing-matrix {
    background: #1a1a1a;
    border-radius: 8px;
    padding: 16px;
    overflow: auto;
  }

  .matrix-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .matrix-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .add-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .add-btn:hover {
    background: #2563eb;
  }

  .add-bus-form {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    padding: 12px;
    background: #252525;
    border-radius: 4px;
  }

  .add-bus-form input {
    flex: 1;
    padding: 8px;
    background: #333;
    color: #fff;
    border: 1px solid #444;
    border-radius: 4px;
  }

  .add-bus-form select {
    padding: 8px;
    background: #333;
    color: #fff;
    border: 1px solid #444;
    border-radius: 4px;
  }

  .add-bus-form button {
    padding: 8px 16px;
    background: #22c55e;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .matrix-container {
    overflow-x: auto;
  }

  .matrix-table {
    width: 100%;
    border-collapse: collapse;
    min-width: 600px;
  }

  .matrix-table th,
  .matrix-table td {
    padding: 8px;
    border: 1px solid #333;
    text-align: center;
  }

  .track-header,
  .output-header {
    background: #252525;
    color: #fff;
    font-weight: 600;
    position: sticky;
    left: 0;
    z-index: 2;
  }

  .bus-header {
    background: #252525;
    color: #fff;
    font-weight: 600;
    position: relative;
  }

  .bus-name {
    font-size: 12px;
  }

  .remove-bus-btn {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 16px;
    height: 16px;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 2px;
    font-size: 10px;
    cursor: pointer;
    padding: 0;
  }

  .track-name {
    background: #252525;
    color: #fff;
    font-weight: 500;
    text-align: left;
    position: sticky;
    left: 0;
    z-index: 1;
  }

  .output-cell select {
    width: 100%;
    padding: 4px;
    background: #333;
    color: #fff;
    border: 1px solid #444;
    border-radius: 4px;
    font-size: 12px;
  }

  .send-cell {
    background: #1a1a1a;
  }

  .send-active {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .send-active input[type='range'] {
    width: 60px;
  }

  .send-btn {
    width: 24px;
    height: 24px;
    border: 2px solid #444;
    border-radius: 50%;
    background: #333;
    color: #666;
    cursor: pointer;
    font-size: 16px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .send-btn:hover {
    border-color: #666;
  }

  .send-btn.active {
    border-color: #3b82f6;
    color: #3b82f6;
  }

  .empty-state {
    padding: 48px;
    text-align: center;
    color: #666;
  }
</style>
```

---

## Step 4: Integration (10 min)

Add routing matrix to DAW view:

```svelte
<!-- In main DAW view or separate routing panel -->
<script lang="ts">
  import RoutingMatrix from './RoutingMatrix.svelte';
</script>

<div class="routing-panel">
  <RoutingMatrix />
</div>
```

---

## Verification (10 min)

```bash
npm run check
make dev
```

Test routing matrix:
1. View routing matrix showing all tracks and buses
2. Click send buttons to enable/disable sends
3. Adjust send levels with sliders
4. Change track output routing
5. Add new aux/group buses
6. Remove buses (verify sends are cleaned up)
7. Verify sends persist across reloads

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Matrix not loading | Check routing API, verify backend integration |
| Sends not toggling | Check send API calls, verify track/bus IDs valid |
| Cannot remove bus | Master bus (ID 0) cannot be removed |
| UI not responsive | Table may need horizontal scroll for many buses |

---

## Phase 2 Complete! 🎉

✅ **Congratulations! You've built a complete DAW mixer system:**

### Day 1-2: Basic Mixer
- ✅ Mixer models (TrackState, MixerConfig)
- ✅ 17 mixer commands (gain, pan, mute, solo, master)
- ✅ Frontend mixer API and stores
- ✅ Full mixer UI with faders, pan knobs

### Day 3: VU Metering
- ✅ Peak/RMS level detection
- ✅ Real-time metering at 60fps
- ✅ VU meter components
- ✅ Clipping detection

### Day 4: Effect Chain
- ✅ Effect models (gain, EQ, reverb, delay, compressor, limiter)
- ✅ Effect chain management
- ✅ Effect rack UI with parameter controls
- ✅ Drag-and-drop reordering

### Day 5: Audio Routing
- ✅ Bus system (master, aux, group)
- ✅ Send/return routing
- ✅ Routing matrix UI
- ✅ Track output routing

---

## Testing Checklist

Before moving to Phase 3, verify:

### Mixer Controls
- [ ] Track gain faders work (-60 dB to +12 dB)
- [ ] Pan knobs work (-1.0 to +1.0)
- [ ] Mute buttons silence tracks
- [ ] Solo buttons work correctly
- [ ] Master fader affects all tracks

### VU Meters
- [ ] Meters respond to audio
- [ ] Peak hold indicators appear
- [ ] Clipping detection works
- [ ] Meters update smoothly

### Effect Chain
- [ ] Can add effects to tracks
- [ ] Can reorder effects by dragging
- [ ] Can adjust effect parameters
- [ ] Can bypass individual effects
- [ ] Can remove effects

### Routing
- [ ] Can create aux buses
- [ ] Can add sends to auxes
- [ ] Can adjust send levels
- [ ] Can change track outputs
- [ ] Can remove buses (sends cleaned up)

---

## Next Steps

**Phase 3: DAW Advanced Features**
- Audio recording
- MIDI editing
- Automation
- Project save/load
- Export/render

See [Implementation Roadmap](../IMPLEMENTATION_ROADMAP.md) for Phase 3 details.

---

## Performance Notes

If you experience performance issues:
- Reduce metering update rate from 60fps to 30fps
- Limit number of effects per track
- Use batch updates for multiple parameter changes
- Consider using Web Workers for heavy processing

---

## Architecture Review

Your DAW mixer now has:
- **Backend:** Rust with async state management
- **Frontend:** Svelte with reactive stores
- **Communication:** Tauri IPC + event streaming
- **Real-time:** 60fps metering updates
- **Modular:** Separate concerns (mixer, metering, effects, routing)

This architecture is production-ready and scalable for advanced features.
