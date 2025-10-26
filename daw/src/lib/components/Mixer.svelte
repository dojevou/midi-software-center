<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '../api';
  import type { Track } from '../api';

  // Mixer state
  let tracks: Track[] = [];
  let masterVolume = 75;
  let masterMuted = false;

  // Animation frame for meter updates
  let meterUpdateInterval: number | null = null;

  // Load tracks on mount
  onMount(async () => {
    await loadTracks();
    startMeterUpdates();
  });

  onDestroy(() => {
    stopMeterUpdates();
  });

  async function loadTracks() {
    try {
      tracks = await api.sequencer.getTracks();
    } catch (error) {
      console.error('Failed to load tracks:', error);
    }
  }

  function startMeterUpdates() {
    // Simulate meter updates for now - in production this would read actual audio levels
    meterUpdateInterval = window.setInterval(() => {
      tracks = tracks.map((track) => ({
        ...track,
        // Simulate varying levels based on whether track is playing
        level: track.muted ? 0 : Math.random() * 80 + 20,
      }));
    }, 100);
  }

  function stopMeterUpdates() {
    if (meterUpdateInterval) {
      clearInterval(meterUpdateInterval);
      meterUpdateInterval = null;
    }
  }

  async function updateTrackVolume(trackId: number, volume: number) {
    try {
      // Convert 0-100 display to 0-127 MIDI
      const midiVolume = Math.round((volume / 100) * 127);
      await api.sequencer.updateTrack(trackId, { volume: midiVolume });

      // Update local state
      const track = tracks.find((t) => t.id === trackId);
      if (track) track.volume = midiVolume;
    } catch (error) {
      console.error('Failed to update volume:', error);
    }
  }

  async function updateTrackPan(trackId: number, pan: number) {
    try {
      // Convert -100 to +100 display to 0-127 MIDI (64 = center)
      const midiPan = Math.round(((pan + 100) / 200) * 127);
      await api.sequencer.updateTrack(trackId, { pan: midiPan });

      // Update local state
      const track = tracks.find((t) => t.id === trackId);
      if (track) track.pan = midiPan;
    } catch (error) {
      console.error('Failed to update pan:', error);
    }
  }

  async function toggleMute(trackId: number) {
    const track = tracks.find((t) => t.id === trackId);
    if (!track) return;

    try {
      const newMuted = !track.muted;
      await api.sequencer.updateTrack(trackId, { muted: newMuted });
      track.muted = newMuted;
      tracks = tracks; // Trigger reactivity
    } catch (error) {
      console.error('Failed to toggle mute:', error);
    }
  }

  async function toggleSolo(trackId: number, exclusive: boolean = false) {
    const track = tracks.find((t) => t.id === trackId);
    if (!track) return;

    try {
      if (exclusive) {
        // Unsolo all other tracks
        for (const t of tracks) {
          if (t.id !== trackId && t.solo) {
            await api.sequencer.updateTrack(t.id, { solo: false });
            t.solo = false;
          }
        }
      }

      const newSolo = !track.solo;
      await api.sequencer.updateTrack(trackId, { solo: newSolo });
      track.solo = newSolo;
      tracks = tracks; // Trigger reactivity
    } catch (error) {
      console.error('Failed to toggle solo:', error);
    }
  }

  function getMeterColor(level: number): string {
    if (level > 90) return '#ef4444'; // red
    if (level > 70) return '#f59e0b'; // yellow
    return '#22c55e'; // green
  }

  function resetVolume(trackId: number) {
    updateTrackVolume(trackId, 75);
  }

  function resetPan(trackId: number) {
    updateTrackPan(trackId, 0);
  }

  // Convert MIDI volume (0-127) to display percentage (0-100)
  function volumeToPercent(volume: number): number {
    return Math.round((volume / 127) * 100);
  }

  // Convert MIDI pan (0-127, 64=center) to display (-100 to +100)
  function panToDisplay(pan: number): number {
    return Math.round((pan / 127) * 200 - 100);
  }
</script>

<div class="mixer">
  <div class="track-strips">
    {#each tracks as track, index}
      <div class="track-strip" class:muted={track.muted} class:solo={track.solo}>
        <!-- Track Name -->
        <div class="track-name" title={track.name}>{track.name}</div>

        <!-- Volume Meter -->
        <div class="meter">
          <div
            class="meter-fill"
            style="height: {track.level || 0}%; background: {getMeterColor(track.level || 0)}"
          ></div>
        </div>

        <!-- Volume Fader -->
        <input
          type="range"
          min="0"
          max="100"
          value={volumeToPercent(track.volume)}
          on:input={(e) => updateTrackVolume(track.id, Number(e.currentTarget.value))}
          on:dblclick={() => resetVolume(track.id)}
          class="fader"
          title="Double-click to reset to 75"
        />
        <div class="volume-label">{volumeToPercent(track.volume)}</div>

        <!-- Pan Control -->
        <div class="pan-control">
          <span class="pan-label">Pan</span>
          <input
            type="range"
            min="-100"
            max="100"
            value={panToDisplay(track.pan)}
            on:input={(e) => updateTrackPan(track.id, Number(e.currentTarget.value))}
            on:dblclick={() => resetPan(track.id)}
            class="pan-slider"
            title="Double-click to center"
          />
          <span class="pan-value"
            >{panToDisplay(track.pan) > 0 ? 'R' : panToDisplay(track.pan) < 0 ? 'L' : 'C'}</span
          >
        </div>

        <!-- Mute/Solo Buttons -->
        <div class="buttons">
          <button
            class="mute-btn"
            class:active={track.muted}
            on:click={() => toggleMute(track.id)}
            title="Mute"
          >
            M
          </button>
          <button
            class="solo-btn"
            class:active={track.solo}
            on:click={(e) => toggleSolo(track.id, e.shiftKey)}
            title="Solo (Shift+click for exclusive)"
          >
            S
          </button>
        </div>

        <!-- Track Color Indicator -->
        <div class="track-color" style="background: {track.color || '#666'}"></div>
      </div>
    {/each}

    <!-- Master Strip -->
    <div class="track-strip master">
      <div class="track-name">Master</div>

      <!-- Master Meter -->
      <div class="meter">
        <div class="meter-fill" style="height: 75%; background: #22c55e"></div>
      </div>

      <!-- Master Fader -->
      <input
        type="range"
        min="0"
        max="100"
        bind:value={masterVolume}
        on:dblclick={() => (masterVolume = 75)}
        class="fader"
        title="Double-click to reset to 75"
      />
      <div class="volume-label">{masterVolume}</div>

      <!-- Master Mute -->
      <div class="buttons">
        <button
          class="mute-btn master-mute"
          class:active={masterMuted}
          on:click={() => (masterMuted = !masterMuted)}
          title="Master Mute"
        >
          M
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .mixer {
    background: #1a1a1a;
    padding: 1em;
    overflow-x: auto;
    overflow-y: hidden;
    height: 100%;
    display: flex;
  }

  .track-strips {
    display: flex;
    gap: 8px;
    min-height: 450px;
    height: 100%;
  }

  .track-strip {
    width: 70px;
    display: flex;
    flex-direction: column;
    align-items: center;
    background: linear-gradient(180deg, rgba(40, 40, 40, 0.8) 0%, rgba(25, 25, 25, 0.9) 100%);
    border-radius: 6px;
    padding: 0.75em 0.5em;
    position: relative;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 0.2s;
  }

  .track-strip:hover {
    background: linear-gradient(180deg, rgba(45, 45, 45, 0.8) 0%, rgba(30, 30, 30, 0.9) 100%);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .track-strip.muted {
    opacity: 0.4;
    background: linear-gradient(180deg, rgba(30, 30, 30, 0.6) 0%, rgba(20, 20, 20, 0.7) 100%);
  }

  .track-strip.solo {
    border: 2px solid #f59e0b;
    box-shadow: 0 0 12px rgba(245, 158, 11, 0.3);
  }

  .track-strip.master {
    background: linear-gradient(180deg, rgba(59, 80, 130, 0.3) 0%, rgba(35, 50, 90, 0.4) 100%);
    border: 1px solid rgba(59, 130, 246, 0.4);
    box-shadow: 0 0 16px rgba(59, 130, 246, 0.2);
    width: 80px;
  }

  .track-name {
    font-size: 0.7em;
    font-weight: 600;
    margin-bottom: 0.75em;
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    width: 100%;
    color: rgba(255, 255, 255, 0.9);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    letter-spacing: 0.3px;
  }

  .meter {
    width: 16px;
    height: 180px;
    background: linear-gradient(180deg, #1a1a1a 0%, #0a0a0a 100%);
    border-radius: 3px;
    position: relative;
    overflow: hidden;
    margin-bottom: 0.75em;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.5);
    border: 1px solid rgba(0, 0, 0, 0.8);
  }

  .meter-fill {
    position: absolute;
    bottom: 0;
    width: 100%;
    transition:
      height 0.05s linear,
      background 0.1s;
    box-shadow: 0 0 8px currentColor;
  }

  .fader {
    writing-mode: bt-lr; /* Vertical */
    -webkit-appearance: slider-vertical;
    appearance: none;
    height: 180px;
    width: 32px;
    margin: 0.75em 0;
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.08) 0%,
      rgba(255, 255, 255, 0.03) 100%
    );
    border-radius: 16px;
    outline: none;
    cursor: ns-resize;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(0, 0, 0, 0.5);
  }

  .fader::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 28px;
    height: 12px;
    background: linear-gradient(180deg, #ff5722 0%, #d84315 100%);
    border-radius: 6px;
    cursor: ns-resize;
    box-shadow:
      0 2px 6px rgba(0, 0, 0, 0.4),
      0 0 8px rgba(255, 87, 34, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .fader::-moz-range-thumb {
    width: 28px;
    height: 12px;
    background: linear-gradient(180deg, #ff5722 0%, #d84315 100%);
    border-radius: 6px;
    cursor: ns-resize;
    box-shadow:
      0 2px 6px rgba(0, 0, 0, 0.4),
      0 0 8px rgba(255, 87, 34, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .fader:hover::-webkit-slider-thumb {
    background: linear-gradient(180deg, #ff6e40 0%, #e64a19 100%);
    box-shadow:
      0 2px 8px rgba(0, 0, 0, 0.5),
      0 0 12px rgba(255, 87, 34, 0.5);
  }

  .fader:hover::-moz-range-thumb {
    background: linear-gradient(180deg, #ff6e40 0%, #e64a19 100%);
    box-shadow:
      0 2px 8px rgba(0, 0, 0, 0.5),
      0 0 12px rgba(255, 87, 34, 0.5);
  }

  .volume-label {
    font-size: 0.65em;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 600;
    margin-bottom: 0.5em;
    min-height: 14px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  }

  .pan-control {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: 0.5em 0;
    gap: 0.25em;
  }

  .pan-label {
    font-size: 0.6em;
    color: rgba(255, 255, 255, 0.6);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .pan-slider {
    width: 50px;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0.05) 100%);
    border-radius: 2px;
    outline: none;
    cursor: ew-resize;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .pan-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 10px;
    height: 10px;
    background: linear-gradient(135deg, #4caf50 0%, #388e3c 100%);
    border-radius: 50%;
    cursor: ew-resize;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .pan-slider::-moz-range-thumb {
    width: 10px;
    height: 10px;
    background: linear-gradient(135deg, #4caf50 0%, #388e3c 100%);
    border-radius: 50%;
    cursor: ew-resize;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  .pan-value {
    font-size: 0.6em;
    color: rgba(255, 255, 255, 0.5);
    font-weight: 600;
  }

  .buttons {
    display: flex;
    gap: 4px;
    margin-top: 0.75em;
  }

  .mute-btn,
  .solo-btn {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 4px;
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.12) 0%,
      rgba(255, 255, 255, 0.08) 100%
    );
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    font-size: 0.75em;
    font-weight: bold;
    transition: all 0.15s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .mute-btn:hover,
  .solo-btn:hover {
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.18) 0%,
      rgba(255, 255, 255, 0.12) 100%
    );
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .mute-btn:active,
  .solo-btn:active {
    transform: translateY(0);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .mute-btn.active {
    background: linear-gradient(180deg, #ef4444 0%, #dc2626 100%);
    color: white;
    box-shadow:
      0 0 12px rgba(239, 68, 68, 0.4),
      0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .solo-btn.active {
    background: linear-gradient(180deg, #f59e0b 0%, #d97706 100%);
    color: white;
    box-shadow:
      0 0 12px rgba(245, 158, 11, 0.4),
      0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .master-mute {
    width: 60px;
    margin-top: 1em;
  }

  .track-color {
    width: 100%;
    height: 4px;
    margin-top: 0.75em;
    border-radius: 2px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  /* Scrollbar styling */
  .mixer::-webkit-scrollbar {
    height: 8px;
  }

  .mixer::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }

  .mixer::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .mixer::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
