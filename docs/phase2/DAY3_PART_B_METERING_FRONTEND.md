# Day 3, Part 2B: Metering Frontend

**Duration:** 2 hours
**Prerequisites:** Part 2A complete (metering backend working)
**Files to create:** 2

---

## Overview

Create real-time VU meter UI components:
1. VU Meter Svelte component with visual feedback
2. Metering store for event handling
3. Peak hold and clipping indicators
4. Integration with ChannelStrip

---

## Step 1: Metering Store (30 min)

Create `app/src/lib/stores/meteringStore.ts`:

```typescript
import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export interface MeterLevel {
  peakDb: number;
  rmsDb: number;
  peakHoldDb: number;
  peakHoldSamples: number;
}

export interface MeteringConfig {
  trackMeters: Record<number, MeterLevel>;
  masterMeter: MeterLevel;
  sampleRate: number;
  enabled: boolean;
}

export interface MeteringState {
  config: MeteringConfig;
  clipping: boolean;
  clippingTracks: number[];
}

/**
 * Metering Store
 * Manages real-time VU meter state from backend events
 */
class MeteringStore {
  private state: Writable<MeteringState>;

  // Public readable stores
  public readonly config: Readable<MeteringConfig>;
  public readonly clipping: Readable<boolean>;
  public readonly clippingTracks: Readable<number[]>;

  private unlistenMeteringUpdate?: () => void;

  constructor() {
    this.state = writable<MeteringState>({
      config: {
        trackMeters: {},
        masterMeter: {
          peakDb: -100,
          rmsDb: -100,
          peakHoldDb: -100,
          peakHoldSamples: 0,
        },
        sampleRate: 48000,
        enabled: true,
      },
      clipping: false,
      clippingTracks: [],
    });

    this.config = derived(this.state, ($state) => $state.config);
    this.clipping = derived(this.state, ($state) => $state.clipping);
    this.clippingTracks = derived(this.state, ($state) => $state.clippingTracks);
  }

  /**
   * Start listening to metering events from backend
   */
  async startListening(): Promise<void> {
    // Listen for metering updates (emitted ~60fps from backend)
    this.unlistenMeteringUpdate = await listen<MeteringConfig>(
      'metering-update',
      (event) => {
        this.state.update((s) => ({
          ...s,
          config: event.payload,
          clipping: this.checkClipping(event.payload),
          clippingTracks: this.getClippingTracks(event.payload),
        }));
      }
    );

    console.log('Started listening to metering updates');
  }

  /**
   * Stop listening to events
   */
  stopListening(): void {
    if (this.unlistenMeteringUpdate) {
      this.unlistenMeteringUpdate();
      this.unlistenMeteringUpdate = undefined;
      console.log('Stopped listening to metering updates');
    }
  }

  /**
   * Check if master or any track is clipping
   */
  private checkClipping(config: MeteringConfig): boolean {
    if (config.masterMeter.peakDb > 0.0) {
      return true;
    }

    return Object.values(config.trackMeters).some((meter) => meter.peakDb > 0.0);
  }

  /**
   * Get list of clipping track IDs
   */
  private getClippingTracks(config: MeteringConfig): number[] {
    return Object.entries(config.trackMeters)
      .filter(([_, meter]) => meter.peakDb > 0.0)
      .map(([id, _]) => parseInt(id));
  }

  /**
   * Get meter level for specific track
   */
  getTrackMeter(trackId: number): MeterLevel | null {
    const state = this.getCurrentState();
    return state.config.trackMeters[trackId] || null;
  }

  /**
   * Get master meter level
   */
  getMasterMeter(): MeterLevel {
    const state = this.getCurrentState();
    return state.config.masterMeter;
  }

  /**
   * Enable/disable metering
   */
  async setEnabled(enabled: boolean): Promise<void> {
    try {
      await invoke('set_metering_enabled', { enabled });
    } catch (error) {
      console.error('Failed to set metering enabled:', error);
    }
  }

  /**
   * Reset all meters to silence
   */
  async resetMeters(): Promise<void> {
    try {
      await invoke('reset_meters');
    } catch (error) {
      console.error('Failed to reset meters:', error);
    }
  }

  /**
   * Get current state (non-reactive)
   */
  private getCurrentState(): MeteringState {
    let currentState: MeteringState;
    this.state.subscribe((s) => (currentState = s))();
    return currentState!;
  }

  /**
   * Subscribe to state changes
   */
  subscribe(callback: (state: MeteringState) => void) {
    return this.state.subscribe(callback);
  }
}

// Export singleton instance
export const meteringStore = new MeteringStore();
```

---

## Step 2: VU Meter Component (1 hour)

Create `app/src/lib/components/DAW/VUMeter.svelte`:

```svelte
<script lang="ts">
  import type { MeterLevel } from '../../stores/meteringStore';

  export let meterLevel: MeterLevel | null;
  export let orientation: 'vertical' | 'horizontal' = 'vertical';
  export let width = 16;
  export let height = 150;

  // Reactive meter positions
  $: peakPercent = meterLevel ? dbToPercent(meterLevel.peakDb) : 0;
  $: rmsPercent = meterLevel ? dbToPercent(meterLevel.rmsDb) : 0;
  $: peakHoldPercent = meterLevel ? dbToPercent(meterLevel.peakHoldDb) : 0;
  $: clipping = meterLevel ? meterLevel.peakDb > 0.0 : false;

  /**
   * Convert dB to visual percentage (0-100)
   * -60 dB = 0%, 0 dB = 100%
   */
  function dbToPercent(db: number): number {
    if (db <= -60.0) return 0;
    if (db >= 0.0) return 100;
    return ((db + 60.0) / 60.0) * 100;
  }

  /**
   * Get color for meter segment based on dB level
   */
  function getMeterColor(percent: number): string {
    if (percent >= 95) return '#dc2626'; // Red (clipping)
    if (percent >= 75) return '#fbbf24'; // Yellow (hot)
    return '#22c55e'; // Green (normal)
  }
</script>

{#if orientation === 'vertical'}
  <div
    class="vu-meter vertical"
    style="width: {width}px; height: {height}px"
    class:clipping
  >
    <!-- Background scale (optional) -->
    <div class="meter-scale">
      <div class="scale-mark" style="bottom: 100%">0</div>
      <div class="scale-mark" style="bottom: 75%">-6</div>
      <div class="scale-mark" style="bottom: 50%">-12</div>
      <div class="scale-mark" style="bottom: 25%">-18</div>
      <div class="scale-mark" style="bottom: 0%">-∞</div>
    </div>

    <!-- Meter background -->
    <div class="meter-bg"></div>

    <!-- RMS level (average) -->
    <div
      class="meter-rms"
      style="height: {rmsPercent}%; background-color: {getMeterColor(rmsPercent)}"
    ></div>

    <!-- Peak level (transients) -->
    <div
      class="meter-peak"
      style="height: {peakPercent}%; background-color: {getMeterColor(peakPercent)}"
    ></div>

    <!-- Peak hold indicator -->
    {#if peakHoldPercent > 0}
      <div
        class="peak-hold"
        style="bottom: {peakHoldPercent}%; background-color: {getMeterColor(
          peakHoldPercent
        )}"
      ></div>
    {/if}

    <!-- Clipping indicator -->
    {#if clipping}
      <div class="clip-indicator">CLIP</div>
    {/if}
  </div>
{:else}
  <!-- Horizontal orientation (for future use) -->
  <div class="vu-meter horizontal" style="width: {height}px; height: {width}px">
    <div class="meter-bg"></div>
    <div
      class="meter-rms"
      style="width: {rmsPercent}%; background-color: {getMeterColor(rmsPercent)}"
    ></div>
    <div
      class="meter-peak"
      style="width: {peakPercent}%; background-color: {getMeterColor(peakPercent)}"
    ></div>
  </div>
{/if}

<style>
  .vu-meter {
    position: relative;
    border-radius: 4px;
    overflow: hidden;
  }

  .vu-meter.vertical {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
  }

  .vu-meter.horizontal {
    display: flex;
    flex-direction: row;
    align-items: center;
  }

  .meter-scale {
    position: absolute;
    left: -30px;
    top: 0;
    bottom: 0;
    width: 24px;
    font-size: 9px;
    color: #666;
    pointer-events: none;
  }

  .scale-mark {
    position: absolute;
    left: 0;
    transform: translateY(50%);
  }

  .meter-bg {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      to top,
      #1a1a1a 0%,
      #1a1a1a 50%,
      #2a2a2a 75%,
      #3a3a3a 100%
    );
    border: 1px solid #333;
  }

  .meter-rms {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    opacity: 0.6;
    transition: height 0.05s ease-out;
    z-index: 1;
  }

  .meter-peak {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    opacity: 0.9;
    transition: height 0.03s ease-out;
    z-index: 2;
  }

  .peak-hold {
    position: absolute;
    left: 0;
    right: 0;
    height: 2px;
    z-index: 3;
    box-shadow: 0 0 4px currentColor;
  }

  .clip-indicator {
    position: absolute;
    top: 4px;
    left: 50%;
    transform: translateX(-50%);
    background: #dc2626;
    color: #fff;
    font-size: 8px;
    font-weight: bold;
    padding: 2px 4px;
    border-radius: 2px;
    z-index: 4;
    animation: blink 0.5s infinite;
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .vu-meter.clipping {
    box-shadow: 0 0 8px #dc2626;
  }

  /* Horizontal variant */
  .horizontal .meter-rms,
  .horizontal .meter-peak {
    bottom: 0;
    top: 0;
    left: 0;
    right: auto;
    height: 100%;
  }

  .horizontal .meter-rms {
    transition: width 0.05s ease-out;
  }

  .horizontal .meter-peak {
    transition: width 0.03s ease-out;
  }
</style>
```

---

## Step 3: Update ChannelStrip (20 min)

Update `app/src/lib/components/DAW/ChannelStrip.svelte` to use VU meters:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { mixerStore } from '../../stores/mixerStore';
  import { meteringStore } from '../../stores/meteringStore';
  import { GainUtils } from '../../types/daw';
  import VUMeter from './VUMeter.svelte';
  import type { TrackState } from '../../types/daw';

  export let trackId: number;

  // Reactive track state from stores
  $: track = $mixerStore.tracks.find((t) => t.trackId === trackId);
  $: meterLevel = $meteringStore.config.trackMeters[trackId] || null;

  // ... rest of existing script ...
</script>

{#if track}
  <div class="channel-strip" data-track-id={trackId}>
    <!-- Track Name -->
    <div class="track-name" title={track.name}>
      {track.name}
    </div>

    <!-- Mute/Solo Buttons -->
    <!-- ... existing buttons ... -->

    <!-- Pan Knob -->
    <!-- ... existing pan knob ... -->

    <!-- Gain Fader -->
    <!-- ... existing fader ... -->

    <!-- VU Meter (replace placeholder) -->
    <VUMeter {meterLevel} width={16} height={150} />
  </div>
{:else}
  <!-- ... existing empty state ... -->
{/if}

<style>
  /* ... existing styles ... */

  /* Remove meter-placeholder styles */
</style>
```

---

## Step 4: Initialize Metering (10 min)

Update `app/src/lib/components/DAW/MixerPanel.svelte`:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { mixerStore } from '../../stores/mixerStore';
  import { meteringStore } from '../../stores/meteringStore';
  import ChannelStrip from './ChannelStrip.svelte';
  import VUMeter from './VUMeter.svelte';
  import { GainUtils } from '../../types/daw';

  // Load mixer and start metering
  onMount(async () => {
    await mixerStore.loadMixerConfig();
    await meteringStore.startListening();
  });

  // Cleanup
  onDestroy(() => {
    meteringStore.stopListening();
  });

  // Reactive state
  $: tracks = $mixerStore.tracks;
  $: config = $mixerStore.config;
  $: loading = $mixerStore.loading;
  $: masterMeter = $meteringStore.config.masterMeter;
  $: clipping = $meteringStore.clipping;

  // ... rest of existing script ...
</script>

<div class="mixer-panel">
  <!-- Header with clipping indicator -->
  <div class="mixer-header">
    <h2>Mixer</h2>
    {#if clipping}
      <div class="clipping-warning">⚠️ CLIPPING DETECTED</div>
    {/if}
    <!-- ... existing header actions ... -->
  </div>

  <!-- ... existing mixer content ... -->

  <!-- Master Channel with VU Meter -->
  <div class="master-channel">
    <!-- ... existing master controls ... -->

    <!-- Master VU Meter -->
    <VUMeter meterLevel={masterMeter} width={20} height={200} />
  </div>
</div>

<style>
  /* ... existing styles ... */

  .clipping-warning {
    background: #dc2626;
    color: #fff;
    font-size: 12px;
    font-weight: bold;
    padding: 6px 12px;
    border-radius: 4px;
    animation: blink 0.5s infinite;
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
```

---

## Verification (10 min)

```bash
cd app
npm run check
make dev
```

### Test VU Meters:

1. Load a MIDI file and start playback
2. Verify meters respond to audio levels
3. Check peak hold indicators appear and decay
4. Test clipping detection by increasing gain to +12 dB
5. Verify master meter shows combined output

### Performance Check:

Monitor browser performance:
- VU meters should update smoothly at 60fps
- CPU usage should remain reasonable (<10% for metering alone)
- No visual stuttering or lag

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Meters not updating | Check metering events in console, verify `startListening()` called |
| Choppy animation | Reduce event rate from 60fps to 30fps in backend |
| Meters always empty | Ensure audio is playing, check audio callback integration |
| High CPU usage | Optimize CSS transitions, reduce meter update frequency |

---

## What's Next?

✅ **Day 3 Complete! You've built:**
- Real-time VU meters with peak/RMS display
- Peak hold indicators
- Clipping detection and warnings
- Event-driven metering at 60fps

**Next:** [Day 4, Part 3A: Effects Backend](./DAY4_PART_A_EFFECTS_BACKEND.md)
- Effect plugin architecture
- Effect chain management
- Parameter automation
- VST/LV2 integration foundation

**Note:** VU meters are now fully functional and integrated with the mixer UI.
