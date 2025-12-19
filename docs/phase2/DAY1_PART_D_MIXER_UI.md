# Day 1, Part 1D: Mixer UI Components

**Duration:** 2.5 hours
**Prerequisites:** Parts 1A, 1B, 1C complete
**Files to create:** 2

---

## Overview

Create Svelte components for mixer UI:
1. MixerPanel - Main mixer container
2. ChannelStrip - Individual track controls (fader, pan, mute, solo)
3. Visual feedback and animations
4. Responsive layout

---

## Step 1: ChannelStrip Component (1 hour 15 min)

Create `app/src/lib/components/DAW/ChannelStrip.svelte`:

```svelte
<script lang="ts">
  import { mixerStore } from '../../stores/mixerStore';
  import { GainUtils } from '../../types/daw';
  import type { TrackState } from '../../types/daw';

  export let trackId: number;

  // Reactive track state from store
  $: track = $mixerStore.tracks.find((t) => t.trackId === trackId);

  // Visual state
  let draggingFader = false;
  let draggingPan = false;

  // Gain fader handling (vertical drag)
  function handleFaderMouseDown(e: MouseEvent) {
    if (!track) return;
    draggingFader = true;

    const fader = e.currentTarget as HTMLElement;
    const rect = fader.getBoundingClientRect();
    const height = rect.height;

    const handleMouseMove = (e: MouseEvent) => {
      const y = Math.max(0, Math.min(height, rect.bottom - e.clientY));
      const percent = y / height;

      // Map 0-1 to -60 dB to +12 dB
      const gainDb = (percent * 72.0) - 60.0;

      mixerStore.setTrackGain(trackId, gainDb);
    };

    const handleMouseUp = () => {
      draggingFader = false;
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  // Pan knob handling (horizontal drag)
  function handlePanMouseDown(e: MouseEvent) {
    if (!track) return;
    draggingPan = true;

    const knob = e.currentTarget as HTMLElement;
    const rect = knob.getBoundingClientRect();
    const width = rect.width;

    const handleMouseMove = (e: MouseEvent) => {
      const x = Math.max(0, Math.min(width, e.clientX - rect.left));
      const percent = x / width;

      // Map 0-1 to -1.0 to +1.0
      const pan = (percent * 2.0) - 1.0;

      mixerStore.setTrackPan(trackId, pan);
    };

    const handleMouseUp = () => {
      draggingPan = false;
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  // Reset to defaults on double-click
  function handleFaderDoubleClick() {
    mixerStore.setTrackGain(trackId, 0.0);
  }

  function handlePanDoubleClick() {
    mixerStore.setTrackPan(trackId, 0.0);
  }

  // Mute/solo toggles
  function handleMuteClick() {
    mixerStore.toggleTrackMute(trackId);
  }

  function handleSoloClick() {
    mixerStore.toggleTrackSolo(trackId);
  }

  // Calculate fader position (0 at bottom, 1 at top)
  function getFaderPosition(gainDb: number): number {
    return ((gainDb + 60.0) / 72.0) * 100;
  }

  // Calculate pan knob position (0 at left, 1 at right)
  function getPanPosition(pan: number): number {
    return ((pan + 1.0) / 2.0) * 100;
  }
</script>

{#if track}
  <div class="channel-strip" data-track-id={trackId}>
    <!-- Track Name -->
    <div class="track-name" title={track.name}>
      {track.name}
    </div>

    <!-- Mute/Solo Buttons -->
    <div class="button-row">
      <button
        class="mute-btn"
        class:active={track.muted}
        on:click={handleMuteClick}
        title="Mute (M)"
      >
        M
      </button>
      <button
        class="solo-btn"
        class:active={track.soloed}
        on:click={handleSoloClick}
        title="Solo (S)"
      >
        S
      </button>
    </div>

    <!-- Pan Knob -->
    <div class="pan-section">
      <div class="pan-label">
        {GainUtils.formatPan(track.pan)}
      </div>
      <div
        class="pan-knob-container"
        on:mousedown={handlePanMouseDown}
        on:dblclick={handlePanDoubleClick}
        role="slider"
        tabindex="0"
        aria-label="Pan"
        aria-valuenow={track.pan}
        aria-valuemin={-1}
        aria-valuemax={1}
      >
        <div class="pan-track"></div>
        <div
          class="pan-knob"
          class:dragging={draggingPan}
          style="left: {getPanPosition(track.pan)}%"
        ></div>
      </div>
    </div>

    <!-- Gain Fader -->
    <div class="fader-section">
      <div
        class="fader-container"
        on:mousedown={handleFaderMouseDown}
        on:dblclick={handleFaderDoubleClick}
        role="slider"
        tabindex="0"
        aria-label="Gain"
        aria-valuenow={track.gainDb}
        aria-valuemin={-60}
        aria-valuemax={12}
      >
        <div class="fader-track"></div>
        <div
          class="fader-thumb"
          class:dragging={draggingFader}
          style="bottom: {getFaderPosition(track.gainDb)}%"
        ></div>
      </div>
      <div class="gain-label">
        {GainUtils.formatDb(track.gainDb)}
      </div>
    </div>

    <!-- Meter Placeholder (will be implemented in Part 2B) -->
    <div class="meter-placeholder">
      <div class="meter-bar"></div>
    </div>
  </div>
{:else}
  <div class="channel-strip empty">
    <div class="track-name">Track {trackId}</div>
    <p class="error">Track not found</p>
  </div>
{/if}

<style>
  .channel-strip {
    width: 80px;
    background: linear-gradient(to bottom, #2a2a2a, #1a1a1a);
    border-right: 1px solid #333;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 8px;
    gap: 12px;
    user-select: none;
  }

  .track-name {
    font-size: 12px;
    font-weight: 600;
    color: #fff;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* Mute/Solo Buttons */
  .button-row {
    display: flex;
    gap: 4px;
  }

  .mute-btn,
  .solo-btn {
    width: 30px;
    height: 24px;
    border: 1px solid #444;
    border-radius: 4px;
    background: #333;
    color: #999;
    font-size: 11px;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .mute-btn:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  .mute-btn.active {
    background: #dc2626;
    color: #fff;
    border-color: #dc2626;
  }

  .solo-btn:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  .solo-btn.active {
    background: #fbbf24;
    color: #000;
    border-color: #fbbf24;
  }

  /* Pan Knob */
  .pan-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 100%;
  }

  .pan-label {
    font-size: 10px;
    color: #999;
    font-weight: 600;
  }

  .pan-knob-container {
    position: relative;
    width: 60px;
    height: 20px;
    cursor: pointer;
  }

  .pan-track {
    position: absolute;
    top: 50%;
    left: 0;
    right: 0;
    height: 3px;
    background: #444;
    border-radius: 2px;
    transform: translateY(-50%);
  }

  .pan-track::before {
    content: '';
    position: absolute;
    left: 50%;
    top: -2px;
    bottom: -2px;
    width: 1px;
    background: #666;
    transform: translateX(-50%);
  }

  .pan-knob {
    position: absolute;
    top: 50%;
    width: 12px;
    height: 16px;
    background: linear-gradient(to bottom, #666, #444);
    border: 1px solid #555;
    border-radius: 2px;
    transform: translate(-50%, -50%);
    transition: background 0.1s ease;
  }

  .pan-knob.dragging {
    background: linear-gradient(to bottom, #3b82f6, #2563eb);
    border-color: #3b82f6;
  }

  /* Gain Fader */
  .fader-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 200px;
  }

  .fader-container {
    position: relative;
    width: 40px;
    flex: 1;
    cursor: pointer;
    display: flex;
    justify-content: center;
  }

  .fader-track {
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 4px;
    background: linear-gradient(to top, #dc2626, #fbbf24, #22c55e);
    border-radius: 2px;
    transform: translateX(-50%);
  }

  .fader-thumb {
    position: absolute;
    left: 50%;
    width: 30px;
    height: 20px;
    background: linear-gradient(to bottom, #666, #444);
    border: 2px solid #555;
    border-radius: 4px;
    transform: translateX(-50%);
    transition: background 0.1s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .fader-thumb.dragging {
    background: linear-gradient(to bottom, #3b82f6, #2563eb);
    border-color: #3b82f6;
  }

  .gain-label {
    font-size: 11px;
    color: #999;
    font-weight: 600;
    min-height: 16px;
  }

  /* Meter Placeholder */
  .meter-placeholder {
    width: 12px;
    height: 150px;
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 2px;
  }

  .meter-bar {
    width: 100%;
    height: 0%;
    background: linear-gradient(to top, #22c55e, #fbbf24, #dc2626);
    transition: height 0.1s ease;
  }

  /* Empty State */
  .channel-strip.empty {
    opacity: 0.5;
  }

  .error {
    font-size: 10px;
    color: #dc2626;
    text-align: center;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .channel-strip {
      width: 60px;
      padding: 8px 4px;
    }

    .track-name {
      font-size: 10px;
    }

    .mute-btn,
    .solo-btn {
      width: 26px;
      height: 20px;
      font-size: 10px;
    }
  }
</style>
```

---

## Step 2: MixerPanel Component (1 hour)

Create `app/src/lib/components/DAW/MixerPanel.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { mixerStore } from '../../stores/mixerStore';
  import ChannelStrip from './ChannelStrip.svelte';
  import { GainUtils } from '../../types/daw';

  // Load mixer state on mount
  onMount(async () => {
    await mixerStore.loadMixerConfig();
  });

  // Reactive state
  $: tracks = $mixerStore.tracks;
  $: config = $mixerStore.config;
  $: loading = $mixerStore.loading;

  // Master fader
  let draggingMaster = false;

  function handleMasterFaderMouseDown(e: MouseEvent) {
    draggingMaster = true;

    const fader = e.currentTarget as HTMLElement;
    const rect = fader.getBoundingClientRect();
    const height = rect.height;

    const handleMouseMove = (e: MouseEvent) => {
      const y = Math.max(0, Math.min(height, rect.bottom - e.clientY));
      const percent = y / height;
      const gainDb = (percent * 72.0) - 60.0;

      mixerStore.setMasterGain(gainDb);
    };

    const handleMouseUp = () => {
      draggingMaster = false;
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMasterFaderDoubleClick() {
    mixerStore.setMasterGain(0.0);
  }

  function handleMasterMuteClick() {
    mixerStore.toggleMasterMute();
  }

  function handleResetMixerClick() {
    if (confirm('Reset all mixer settings to defaults?')) {
      mixerStore.resetMixer();
    }
  }

  function handleMuteAllClick() {
    mixerStore.muteAllTracks();
  }

  function handleUnmuteAllClick() {
    mixerStore.unmuteAllTracks();
  }

  function handleClearSolosClick() {
    mixerStore.clearAllSolos();
  }

  function getMasterFaderPosition(gainDb: number): number {
    return ((gainDb + 60.0) / 72.0) * 100;
  }
</script>

<div class="mixer-panel">
  <!-- Header -->
  <div class="mixer-header">
    <h2>Mixer</h2>
    <div class="header-actions">
      <button class="action-btn" on:click={handleMuteAllClick} title="Mute all tracks">
        🔇 All
      </button>
      <button class="action-btn" on:click={handleUnmuteAllClick} title="Unmute all tracks">
        🔊 All
      </button>
      <button class="action-btn" on:click={handleClearSolosClick} title="Clear all solos">
        Clear S
      </button>
      <button class="action-btn danger" on:click={handleResetMixerClick} title="Reset mixer">
        Reset
      </button>
    </div>
  </div>

  <!-- Mixer Content -->
  {#if loading}
    <div class="loading-state">
      <p>Loading mixer...</p>
    </div>
  {:else if tracks.length === 0}
    <div class="empty-state">
      <p>No tracks loaded</p>
      <p class="hint">Load a MIDI file to see tracks</p>
    </div>
  {:else}
    <div class="mixer-content">
      <!-- Track Channels -->
      <div class="track-channels">
        {#each tracks as track (track.trackId)}
          <ChannelStrip trackId={track.trackId} />
        {/each}
      </div>

      <!-- Master Channel -->
      <div class="master-channel">
        <div class="track-name">Master</div>

        <button
          class="mute-btn"
          class:active={config.masterMuted}
          on:click={handleMasterMuteClick}
          title="Mute master"
        >
          M
        </button>

        <div class="fader-section">
          <div
            class="fader-container"
            on:mousedown={handleMasterFaderMouseDown}
            on:dblclick={handleMasterFaderDoubleClick}
            role="slider"
            tabindex="0"
            aria-label="Master Gain"
          >
            <div class="fader-track"></div>
            <div
              class="fader-thumb"
              class:dragging={draggingMaster}
              style="bottom: {getMasterFaderPosition(config.masterGainDb)}%"
            ></div>
          </div>
          <div class="gain-label">
            {GainUtils.formatDb(config.masterGainDb)}
          </div>
        </div>

        <div class="meter-placeholder">
          <div class="meter-bar"></div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .mixer-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1a1a1a;
    color: #fff;
  }

  /* Header */
  .mixer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #252525;
    border-bottom: 1px solid #333;
  }

  .mixer-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    padding: 6px 12px;
    background: #333;
    color: #fff;
    border: 1px solid #444;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  .action-btn.danger {
    color: #dc2626;
  }

  .action-btn.danger:hover {
    background: #dc2626;
    color: #fff;
  }

  /* Content */
  .mixer-content {
    flex: 1;
    display: flex;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .track-channels {
    display: flex;
    flex: 1;
  }

  /* Master Channel */
  .master-channel {
    width: 100px;
    background: linear-gradient(to bottom, #3a3a3a, #2a2a2a);
    border-left: 2px solid #fbbf24;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 8px;
    gap: 12px;
  }

  .master-channel .track-name {
    font-size: 14px;
    font-weight: 700;
    color: #fbbf24;
    text-align: center;
  }

  .master-channel .mute-btn {
    width: 40px;
    height: 28px;
    border: 1px solid #444;
    border-radius: 4px;
    background: #333;
    color: #999;
    font-size: 12px;
    font-weight: bold;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .master-channel .mute-btn:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  .master-channel .mute-btn.active {
    background: #dc2626;
    color: #fff;
    border-color: #dc2626;
  }

  .master-channel .fader-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 200px;
  }

  .master-channel .fader-container {
    position: relative;
    width: 50px;
    flex: 1;
    cursor: pointer;
    display: flex;
    justify-content: center;
  }

  .master-channel .fader-track {
    position: absolute;
    left: 50%;
    top: 0;
    bottom: 0;
    width: 6px;
    background: linear-gradient(to top, #dc2626, #fbbf24, #22c55e);
    border-radius: 3px;
    transform: translateX(-50%);
  }

  .master-channel .fader-thumb {
    position: absolute;
    left: 50%;
    width: 40px;
    height: 24px;
    background: linear-gradient(to bottom, #fbbf24, #f59e0b);
    border: 2px solid #fbbf24;
    border-radius: 4px;
    transform: translateX(-50%);
    transition: background 0.1s ease;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
  }

  .master-channel .fader-thumb.dragging {
    background: linear-gradient(to bottom, #fcd34d, #fbbf24);
  }

  .master-channel .gain-label {
    font-size: 12px;
    color: #fbbf24;
    font-weight: 700;
  }

  .master-channel .meter-placeholder {
    width: 16px;
    height: 150px;
    background: #1a1a1a;
    border: 2px solid #fbbf24;
    border-radius: 3px;
  }

  /* Loading/Empty States */
  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    color: #666;
  }

  .empty-state .hint {
    margin-top: 8px;
    font-size: 14px;
    color: #888;
  }
</style>
```

---

## Verification (15 min)

### 1. Component Check

```bash
cd app
npm run check
```

**Expected:** No errors

### 2. Visual Test

```bash
make dev
```

Navigate to mixer view and verify:
- Track channel strips appear
- Faders are draggable
- Pan knobs work
- Mute/solo buttons toggle
- Master channel displays
- Visual feedback on interaction

### 3. Test Interactions

In browser:
1. Drag a track fader up/down
2. Double-click fader to reset to 0 dB
3. Drag pan knob left/right
4. Double-click pan to center
5. Click mute button (should turn red)
6. Click solo button (should turn yellow)
7. Verify labels update correctly

### 4. Test Batch Operations

Click header buttons:
- "Mute All" - all tracks should mute
- "Unmute All" - all tracks should unmute
- "Clear S" - all solos should clear
- "Reset" - entire mixer should reset to defaults

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Fader not dragging | Check mouse event handlers, ensure window listeners added |
| Values not updating | Verify mixerStore.subscribe() is working |
| Visual glitches | Check CSS transform calculations |
| Pan knob jumpy | Ensure mouse position calculated relative to knob rect |

---

## What's Next?

✅ **Day 1 Complete! You've built:**
- Complete mixer data models (Rust)
- 17 Tauri commands for mixer operations
- TypeScript types and API client
- Reactive Svelte stores
- Full mixer UI with faders, pan knobs, mute/solo

**Next:** [Day 3, Part 2A: Metering Backend](./DAY3_PART_A_METERING_BACKEND.md)
- Peak and RMS level detection
- Real-time audio analysis
- Metering events to frontend
- VU meter integration

**Note:** Day 2 is reserved for testing and refinement. Proceed directly to Day 3 when ready.
