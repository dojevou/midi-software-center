<script lang="ts">
  /**
   * MixerWindow - Audio mixer interface
   *
   * Task-O-Matic: Channel strips with faders, pan, mute/solo, and metering.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    projectStore,
    projectActions,
    tracksList
  } from '$lib/stores/projectStore';
  import { playbackStore } from '$lib/stores/playbackStore';

  // Reactive state
  $: tracks = $tracksList;
  $: isPlaying = $playbackStore.isPlaying;

  // Master channel state
  let masterVolume = 0.8;
  let masterPan = 0.0;
  let masterMuted = false;

  // Metering state
  let meterLevels = new Map<number, number>();
  let masterMeterLevel = 0;
  let meterUpdateInterval: number | null = null;

  // Fader interaction
  let draggingFader: number | null = null;
  let draggingMasterFader = false;

  // Volume control
  async function handleVolumeChange(trackId: number, volume: number) {
    await projectActions.setTrackVolume(trackId, volume);
  }

  async function handlePanChange(trackId: number, pan: number) {
    await projectActions.setTrackPan(trackId, pan);
  }

  function handleMasterVolumeChange(volume: number) {
    masterVolume = volume;
    // TODO: Send to backend
  }

  function handleMasterPanChange(pan: number) {
    masterPan = pan;
    // TODO: Send to backend
  }

  // Mute/Solo
  async function handleMute(trackId: number, muted: boolean) {
    await projectActions.setTrackMute(trackId, muted);
  }

  async function handleSolo(trackId: number, solo: boolean) {
    await projectActions.setTrackSolo(trackId, solo);
  }

  function handleMasterMute() {
    masterMuted = !masterMuted;
    // TODO: Send to backend
  }

  // Fader dragging
  function startDragging(trackId: number | 'master') {
    if (trackId === 'master') {
      draggingMasterFader = true;
    } else {
      draggingFader = trackId;
    }
  }

  function stopDragging() {
    draggingFader = null;
    draggingMasterFader = false;
  }

  function handleFaderMouseMove(e: MouseEvent, trackId: number, faderHeight: number) {
    if (draggingFader === trackId) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const y = e.clientY - rect.top;
      const volume = Math.max(0, Math.min(1, 1 - (y / faderHeight)));
      handleVolumeChange(trackId, volume);
    }
  }

  function handleMasterFaderMouseMove(e: MouseEvent, faderHeight: number) {
    if (draggingMasterFader) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const y = e.clientY - rect.top;
      const volume = Math.max(0, Math.min(1, 1 - (y / faderHeight)));
      handleMasterVolumeChange(volume);
    }
  }

  // Metering simulation
  function updateMeters() {
    if (isPlaying) {
      // Simulate meter levels (in production, get from audio engine)
      tracks.forEach(track => {
        const level = track.muted ? 0 : Math.random() * (track.volume);
        meterLevels.set(track.id, level);
      });

      masterMeterLevel = masterMuted ? 0 : Math.random() * masterVolume * 0.8;
      meterLevels = new Map(meterLevels); // Trigger reactivity
    } else {
      // Decay meters when not playing
      tracks.forEach(track => {
        const current = meterLevels.get(track.id) || 0;
        meterLevels.set(track.id, current * 0.9);
      });
      masterMeterLevel *= 0.9;
      meterLevels = new Map(meterLevels);
    }
  }

  // Db conversion for display
  function volumeToDb(volume: number): string {
    if (volume === 0) return '-∞';
    const db = 20 * Math.log10(volume);
    return db.toFixed(1);
  }

  // Pan display
  function panToDisplay(pan: number): string {
    if (pan === 0) return 'C';
    if (pan < 0) return `${Math.abs(pan * 100).toFixed(0)}L`;
    return `${(pan * 100).toFixed(0)}R`;
  }

  onMount(() => {
    document.addEventListener('mouseup', stopDragging);
    meterUpdateInterval = window.setInterval(updateMeters, 50);
  });

  onDestroy(() => {
    document.removeEventListener('mouseup', stopDragging);
    if (meterUpdateInterval !== null) {
      clearInterval(meterUpdateInterval);
    }
  });
</script>

<WindowBase windowId="mixer" title="Mixer">
  <div class="mixer-window">
    <div class="mixer-header">
      <h3 class="mixer-title">Channel Mixer</h3>
      <div class="mixer-controls">
        <button class="reset-btn" title="Reset All">Reset</button>
      </div>
    </div>

    <div class="channels-container">
      <!-- Track Channels -->
      {#each tracks as track (track.id)}
        <div class="channel-strip" style="border-top: 3px solid {track.color}">
          <!-- Track Label -->
          <div class="channel-label">{track.label}</div>

          <!-- Meter -->
          <div class="meter">
            <div class="meter-bar">
              <div
                class="meter-level"
                style="height: {(meterLevels.get(track.id) || 0) * 100}%"
                class:clip={(meterLevels.get(track.id) || 0) > 0.95}
              />
            </div>
          </div>

          <!-- Pan Control -->
          <div class="pan-control">
            <input
              type="range"
              min="-1"
              max="1"
              step="0.01"
              value={track.pan}
              on:input={(e) => handlePanChange(track.id, parseFloat(e.currentTarget.value))}
              class="pan-slider"
            />
            <div class="pan-value">{panToDisplay(track.pan)}</div>
          </div>

          <!-- Fader -->
          <div
            class="fader-container"
            on:mousedown={() => startDragging(track.id)}
            on:mousemove={(e) => handleFaderMouseMove(e, track.id, 200)}
            role="slider"
            tabindex="0"
            aria-label="Volume fader"
            aria-valuenow={track.volume * 100}
          >
            <div class="fader-track">
              <div
                class="fader-thumb"
                style="bottom: {track.volume * 100}%"
              />
            </div>
          </div>

          <!-- Volume Display -->
          <div class="volume-display">
            {volumeToDb(track.volume)} dB
          </div>

          <!-- Mute/Solo -->
          <div class="channel-buttons">
            <button
              class="channel-btn mute"
              class:active={track.muted}
              on:click={() => handleMute(track.id, !track.muted)}
            >
              M
            </button>
            <button
              class="channel-btn solo"
              class:active={track.solo}
              on:click={() => handleSolo(track.id, !track.solo)}
            >
              S
            </button>
          </div>
        </div>
      {/each}

      <!-- Master Channel -->
      <div class="channel-strip master">
        <div class="channel-label master-label">Master</div>

        <!-- Master Meter -->
        <div class="meter">
          <div class="meter-bar">
            <div
              class="meter-level"
              style="height: {masterMeterLevel * 100}%"
              class:clip={masterMeterLevel > 0.95}
            />
          </div>
        </div>

        <!-- Master Pan -->
        <div class="pan-control">
          <input
            type="range"
            min="-1"
            max="1"
            step="0.01"
            value={masterPan}
            on:input={(e) => handleMasterPanChange(parseFloat(e.currentTarget.value))}
            class="pan-slider"
          />
          <div class="pan-value">{panToDisplay(masterPan)}</div>
        </div>

        <!-- Master Fader -->
        <div
          class="fader-container"
          on:mousedown={() => startDragging('master')}
          on:mousemove={(e) => handleMasterFaderMouseMove(e, 200)}
          role="slider"
          tabindex="0"
          aria-label="Master volume fader"
          aria-valuenow={masterVolume * 100}
        >
          <div class="fader-track">
            <div
              class="fader-thumb master-thumb"
              style="bottom: {masterVolume * 100}%"
            />
          </div>
        </div>

        <!-- Master Volume Display -->
        <div class="volume-display master-volume">
          {volumeToDb(masterVolume)} dB
        </div>

        <!-- Master Mute -->
        <div class="channel-buttons">
          <button
            class="channel-btn mute master-mute"
            class:active={masterMuted}
            on:click={handleMasterMute}
          >
            M
          </button>
        </div>
      </div>
    </div>

    {#if tracks.length === 0}
      <div class="no-tracks-message">
        <p>No tracks to mix</p>
        <p class="hint">Add tracks in the DAW window to get started</p>
      </div>
    {/if}
  </div>
</WindowBase>

<style>
  .mixer-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--mixer-bg, #1e1e1e);
  }

  .mixer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .mixer-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .mixer-controls {
    display: flex;
    gap: 8px;
  }

  .reset-btn {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .reset-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .channels-container {
    flex: 1;
    display: flex;
    gap: 16px;
    padding: 24px 16px;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .channel-strip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    min-width: 80px;
    padding: 16px 12px;
    background: var(--channel-bg, #252525);
    border-radius: 8px;
    border: 1px solid var(--border-color, #3e3e3e);
  }

  .channel-strip.master {
    min-width: 100px;
    background: var(--master-bg, #2a2a2a);
    border: 2px solid var(--accent-color, #0078d4);
  }

  .channel-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    text-align: center;
    word-break: break-word;
    max-width: 100%;
  }

  .channel-label.master-label {
    font-size: 14px;
    color: var(--accent-color, #0078d4);
  }

  /* Meter */
  .meter {
    width: 20px;
    height: 120px;
    background: var(--meter-bg, #1a1a1a);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    overflow: hidden;
    position: relative;
  }

  .meter-bar {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 100%;
  }

  .meter-level {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: linear-gradient(
      to top,
      #52b788 0%,
      #52b788 70%,
      #f7dc6f 70%,
      #f7dc6f 85%,
      #e81123 85%
    );
    transition: height 0.05s ease-out;
  }

  .meter-level.clip {
    animation: clip-flash 0.1s;
  }

  @keyframes clip-flash {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  /* Pan Control */
  .pan-control {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: 100%;
  }

  .pan-slider {
    width: 100%;
    height: 4px;
    -webkit-appearance: none;
    background: var(--slider-bg, #3a3a3a);
    border-radius: 2px;
    outline: none;
  }

  .pan-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 16px;
    background: var(--accent-color, #0078d4);
    border-radius: 2px;
    cursor: pointer;
  }

  .pan-slider::-moz-range-thumb {
    width: 12px;
    height: 16px;
    background: var(--accent-color, #0078d4);
    border-radius: 2px;
    cursor: pointer;
    border: none;
  }

  .pan-value {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Fader */
  .fader-container {
    width: 40px;
    height: 200px;
    cursor: pointer;
    user-select: none;
  }

  .fader-track {
    width: 8px;
    height: 100%;
    background: var(--fader-track, #3a3a3a);
    border-radius: 4px;
    margin: 0 auto;
    position: relative;
  }

  .fader-thumb {
    position: absolute;
    left: 50%;
    transform: translateX(-50%) translateY(50%);
    width: 32px;
    height: 16px;
    background: var(--fader-thumb, #0078d4);
    border: 2px solid var(--fader-thumb-border, #005a9e);
    border-radius: 4px;
    cursor: grab;
  }

  .fader-thumb:active {
    cursor: grabbing;
  }

  .fader-thumb.master-thumb {
    background: var(--master-fader, #0078d4);
    border-color: var(--master-fader-border, #005a9e);
    width: 40px;
    height: 20px;
  }

  .volume-display {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
    text-align: center;
  }

  .volume-display.master-volume {
    font-weight: 600;
    color: var(--accent-color, #0078d4);
  }

  /* Channel Buttons */
  .channel-buttons {
    display: flex;
    gap: 6px;
    width: 100%;
    justify-content: center;
  }

  .channel-btn {
    width: 32px;
    height: 28px;
    border: 1px solid var(--border-color, #3e3e3e);
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    transition: all 0.15s ease;
  }

  .channel-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .channel-btn.mute.active {
    background: #f7dc6f;
    color: #1e1e1e;
    border-color: #f7dc6f;
  }

  .channel-btn.solo.active {
    background: #52b788;
    color: white;
    border-color: #52b788;
  }

  .channel-btn.master-mute {
    width: 100%;
  }

  .no-tracks-message {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary, #888);
    gap: 8px;
  }

  .no-tracks-message p {
    margin: 0;
  }

  .hint {
    font-size: 13px;
    color: var(--text-tertiary, #666);
  }

  /* Scrollbar */
  .channels-container::-webkit-scrollbar {
    height: 12px;
  }

  .channels-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .channels-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
