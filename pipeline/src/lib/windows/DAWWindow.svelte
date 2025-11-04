<script lang="ts">
  /**
   * DAWWindow - Main DAW interface
   *
   * Task-O-Matic: Transport controls, timeline, track list, and arrangement view.
   */

  import { onMount } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    playbackStore,
    playbackActions,
    positionFormatted,
    tempoDisplay,
    timeSignatureDisplay
  } from '$lib/stores/playbackStore';
  import {
    projectStore,
    projectActions,
    tracksList,
    selectedTrack
  } from '$lib/stores/projectStore';
  import { uiStore, uiActions } from '$lib/stores/uiStore';

  // Local state
  let gridSnapping = true;
  let snapValue = 16; // 16th notes
  let zoomLevel = 1.0;
  let scrollPosition = 0;

  // Reactive state
  $: isPlaying = $playbackStore.isPlaying;
  $: isRecording = $playbackStore.isRecording;
  $: position = $playbackStore.position;
  $: tracks = $tracksList;
  $: currentTrack = $selectedTrack;
  $: tempo = $playbackStore.tempo;
  $: loopEnabled = $playbackStore.loop.enabled;
  $: metronomeEnabled = $playbackStore.metronome.enabled;

  // Transport actions
  async function handlePlay() {
    if (isPlaying) {
      await playbackActions.pause();
    } else {
      await playbackActions.play();
    }
  }

  async function handleStop() {
    await playbackActions.stop();
  }

  async function handleRecord() {
    await playbackActions.record();
  }

  // Track actions
  async function handleAddTrack() {
    await projectActions.addTrack();
  }

  async function handleSelectTrack(trackId: number) {
    projectActions.selectTrack(trackId);
  }

  async function handleRemoveTrack(trackId: number) {
    if (confirm('Are you sure you want to delete this track?')) {
      await projectActions.removeTrack(trackId);
    }
  }

  async function handleMuteTrack(trackId: number, muted: boolean) {
    await projectActions.setTrackMute(trackId, muted);
  }

  async function handleSoloTrack(trackId: number, solo: boolean) {
    await projectActions.setTrackSolo(trackId, solo);
  }

  async function handleArmTrack(trackId: number, armed: boolean) {
    await projectActions.setTrackArm(trackId, armed);
  }

  // Timeline navigation
  function handleZoomIn() {
    zoomLevel = Math.min(zoomLevel * 1.5, 10);
  }

  function handleZoomOut() {
    zoomLevel = Math.max(zoomLevel / 1.5, 0.1);
  }

  function handleScroll(e: WheelEvent) {
    if (e.ctrlKey) {
      e.preventDefault();
      if (e.deltaY < 0) {
        handleZoomIn();
      } else {
        handleZoomOut();
      }
    } else {
      scrollPosition += e.deltaY;
    }
  }

  // Grid snapping
  function toggleGridSnapping() {
    gridSnapping = !gridSnapping;
  }

  // Loop controls
  function toggleLoop() {
    playbackActions.enableLoop(!loopEnabled);
  }

  // Metronome controls
  function toggleMetronome() {
    playbackActions.enableMetronome(!metronomeEnabled);
  }

  // Tempo adjustment
  async function handleTempoChange(delta: number) {
    const newTempo = tempo + delta;
    if (newTempo >= 20 && newTempo <= 300) {
      await playbackActions.setTempo(newTempo);
    }
  }

  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    if (e.code === 'Space') {
      e.preventDefault();
      handlePlay();
    } else if (e.code === 'KeyR' && e.ctrlKey) {
      e.preventDefault();
      handleRecord();
    } else if (e.code === 'KeyL' && e.ctrlKey) {
      e.preventDefault();
      toggleLoop();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<WindowBase windowId="daw" title="DAW">
  <div class="daw-window">
    <!-- Transport Bar -->
    <div class="transport-bar">
      <div class="transport-controls">
        <button
          class="transport-btn"
          on:click={handlePlay}
          title={isPlaying ? 'Pause (Space)' : 'Play (Space)'}
        >
          {isPlaying ? '⏸' : '▶'}
        </button>
        <button
          class="transport-btn"
          on:click={handleStop}
          title="Stop"
        >
          ⏹
        </button>
        <button
          class="transport-btn record"
          class:recording={isRecording}
          on:click={handleRecord}
          title="Record (Ctrl+R)"
        >
          ⏺
        </button>
        <button
          class="transport-btn"
          class:active={loopEnabled}
          on:click={toggleLoop}
          title="Toggle Loop (Ctrl+L)"
        >
          🔁
        </button>
        <button
          class="transport-btn"
          class:active={metronomeEnabled}
          on:click={toggleMetronome}
          title="Toggle Metronome"
        >
          🎵
        </button>
      </div>

      <div class="position-display">
        <div class="position-value">{$positionFormatted}</div>
        <div class="position-label">Position</div>
      </div>

      <div class="tempo-control">
        <button
          class="tempo-btn"
          on:click={() => handleTempoChange(-1)}
        >
          -
        </button>
        <div class="tempo-display">
          <div class="tempo-value">{tempo}</div>
          <div class="tempo-label">BPM</div>
        </div>
        <button
          class="tempo-btn"
          on:click={() => handleTempoChange(1)}
        >
          +
        </button>
      </div>

      <div class="time-sig-display">
        <div class="time-sig-value">{$timeSignatureDisplay}</div>
        <div class="time-sig-label">Time Sig</div>
      </div>

      <div class="transport-options">
        <button
          class="option-btn"
          class:active={gridSnapping}
          on:click={toggleGridSnapping}
          title="Grid Snapping"
        >
          Grid
        </button>
        <select class="snap-select" bind:value={snapValue}>
          <option value={1}>1/1</option>
          <option value={2}>1/2</option>
          <option value={4}>1/4</option>
          <option value={8}>1/8</option>
          <option value={16}>1/16</option>
          <option value={32}>1/32</option>
        </select>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="main-content">
      <!-- Track List -->
      <div class="track-list">
        <div class="track-list-header">
          <span class="track-list-title">Tracks</span>
          <button
            class="add-track-btn"
            on:click={handleAddTrack}
            title="Add Track (Ctrl+T)"
          >
            +
          </button>
        </div>

        <div class="tracks-container">
          {#each tracks as track (track.id)}
            <div
              class="track-header"
              class:selected={currentTrack?.id === track.id}
              on:click={() => handleSelectTrack(track.id)}
              role="button"
              tabindex="0"
              style="border-left: 4px solid {track.color}"
            >
              <div class="track-name">{track.label}</div>
              <div class="track-controls">
                <button
                  class="track-btn mute"
                  class:active={track.muted}
                  on:click|stopPropagation={() => handleMuteTrack(track.id, !track.muted)}
                  title="Mute"
                >
                  M
                </button>
                <button
                  class="track-btn solo"
                  class:active={track.solo}
                  on:click|stopPropagation={() => handleSoloTrack(track.id, !track.solo)}
                  title="Solo"
                >
                  S
                </button>
                <button
                  class="track-btn arm"
                  class:active={track.armed}
                  on:click|stopPropagation={() => handleArmTrack(track.id, !track.armed)}
                  title="Arm"
                >
                  R
                </button>
                <button
                  class="track-btn delete"
                  on:click|stopPropagation={() => handleRemoveTrack(track.id)}
                  title="Delete Track"
                >
                  ×
                </button>
              </div>
            </div>
          {/each}

          {#if tracks.length === 0}
            <div class="no-tracks">
              <p>No tracks yet</p>
              <button class="add-first-track" on:click={handleAddTrack}>
                Add First Track
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Arrangement View -->
      <div class="arrangement-view" on:wheel={handleScroll}>
        <!-- Timeline -->
        <div class="timeline">
          <div class="timeline-markers" style="transform: scale({zoomLevel}, 1)">
            {#each Array(64) as _, bar}
              <div class="bar-marker">
                <span class="bar-number">{bar + 1}</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Tracks Arrangement -->
        <div class="tracks-arrangement" style="transform: scale({zoomLevel}, 1)">
          {#each tracks as track (track.id)}
            <div class="track-lane" style="background: {track.color}10">
              <!-- Track regions/clips will be rendered here -->
              <div class="track-lane-content">
                <!-- Placeholder for clips -->
              </div>
            </div>
          {/each}
        </div>

        <!-- Playhead -->
        {#if position}
          <div
            class="playhead"
            style="left: {(position.totalTicks / 480 / 4) * 100 * zoomLevel}px"
          />
        {/if}
      </div>
    </div>

    <!-- Zoom Controls -->
    <div class="zoom-controls">
      <button class="zoom-btn" on:click={handleZoomOut}>-</button>
      <span class="zoom-value">{(zoomLevel * 100).toFixed(0)}%</span>
      <button class="zoom-btn" on:click={handleZoomIn}>+</button>
    </div>
  </div>
</WindowBase>

<style>
  .daw-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Transport Bar */
  .transport-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background: var(--transport-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .transport-controls {
    display: flex;
    gap: 8px;
  }

  .transport-btn {
    width: 40px;
    height: 40px;
    border: none;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    font-size: 18px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .transport-btn:hover {
    background: var(--button-hover, #4a4a4a);
    transform: scale(1.05);
  }

  .transport-btn.active {
    background: var(--accent-color, #0078d4);
  }

  .transport-btn.record.recording {
    background: #e81123;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .position-display,
  .tempo-control,
  .time-sig-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .position-value,
  .tempo-value,
  .time-sig-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
  }

  .position-label,
  .tempo-label,
  .time-sig-label {
    font-size: 10px;
    color: var(--text-secondary, #888);
    text-transform: uppercase;
  }

  .tempo-control {
    flex-direction: row;
    gap: 8px;
  }

  .tempo-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .tempo-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .transport-options {
    display: flex;
    gap: 8px;
    margin-left: auto;
  }

  .option-btn {
    padding: 8px 16px;
    border: 1px solid var(--border-color, #3e3e3e);
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .option-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .snap-select {
    padding: 8px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
  }

  /* Main Content */
  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* Track List */
  .track-list {
    width: 240px;
    background: var(--tracklist-bg, #252525);
    border-right: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    flex-direction: column;
  }

  .track-list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .track-list-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .add-track-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: var(--accent-color, #0078d4);
    color: white;
    border-radius: 4px;
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
  }

  .add-track-btn:hover {
    background: var(--accent-hover, #0063b1);
  }

  .tracks-container {
    flex: 1;
    overflow-y: auto;
  }

  .track-header {
    padding: 12px;
    background: var(--track-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .track-header:hover {
    background: var(--track-hover, #323232);
  }

  .track-header.selected {
    background: var(--track-selected, #3a3a3a);
  }

  .track-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 8px;
  }

  .track-controls {
    display: flex;
    gap: 6px;
  }

  .track-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border-color, #3e3e3e);
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
  }

  .track-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .track-btn.mute.active {
    background: #f7dc6f;
    color: #1e1e1e;
  }

  .track-btn.solo.active {
    background: #52b788;
    color: white;
  }

  .track-btn.arm.active {
    background: #e81123;
    color: white;
  }

  .track-btn.delete:hover {
    background: #e81123;
    color: white;
    border-color: #e81123;
  }

  .no-tracks {
    padding: 48px 24px;
    text-align: center;
    color: var(--text-secondary, #888);
  }

  .add-first-track {
    margin-top: 16px;
    padding: 12px 24px;
    background: var(--accent-color, #0078d4);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  /* Arrangement View */
  .arrangement-view {
    flex: 1;
    overflow: auto;
    position: relative;
    background: var(--arrangement-bg, #1e1e1e);
  }

  .timeline {
    height: 40px;
    background: var(--timeline-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    overflow: hidden;
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .timeline-markers {
    display: flex;
    height: 100%;
    transform-origin: left;
  }

  .bar-marker {
    min-width: 100px;
    border-left: 1px solid var(--marker-color, #4e4e4e);
    display: flex;
    align-items: center;
    padding-left: 8px;
  }

  .bar-number {
    font-size: 12px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  .tracks-arrangement {
    transform-origin: left;
  }

  .track-lane {
    height: 80px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    position: relative;
  }

  .track-lane-content {
    height: 100%;
    padding: 8px;
  }

  .playhead {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #e81123;
    z-index: 100;
    pointer-events: none;
  }

  .playhead::before {
    content: '';
    position: absolute;
    top: 0;
    left: -6px;
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-top: 10px solid #e81123;
  }

  /* Zoom Controls */
  .zoom-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--transport-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .zoom-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .zoom-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .zoom-value {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
    min-width: 50px;
    text-align: center;
  }

  /* Scrollbars */
  .tracks-container::-webkit-scrollbar,
  .arrangement-view::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .tracks-container::-webkit-scrollbar-track,
  .arrangement-view::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .tracks-container::-webkit-scrollbar-thumb,
  .arrangement-view::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
