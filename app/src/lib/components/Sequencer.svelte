<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    sequencerStore,
    sequencerActions,
    pixelsPerTick,
    totalProjectLength,
    type SequencerTrack as TrackType,
  } from '$lib/stores/sequencerStore';
  import SequencerTrack from './SequencerTrack.svelte';
  import SequencerTimeline from './SequencerTimeline.svelte';
  import SequencerTransport from './SequencerTransport.svelte';
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';
  import { toastStore } from '$lib/stores/toastStore';

  // Props
  export let width: number = 800;
  export let height: number = 600;

  // Local state
  let containerRef: HTMLDivElement;
  let tracksContainerRef: HTMLDivElement;
  let isDragOver = false;
  let isLoadingFile = false;
  let loadingFileName = '';

  // Reactive state from store
  $: state = $sequencerStore;
  $: tracks = state.tracks;
  $: project = state.project;
  $: playhead = state.playhead;
  $: zoom = state.zoom;
  $: scrollX = state.scrollX;
  $: scrollY = state.scrollY;
  $: pxPerTick = $pixelsPerTick;
  $: totalLength = $totalProjectLength;

  // Calculate total content width based on project length
  $: contentWidth = totalLength * pxPerTick;

  // Calculate playhead position in pixels
  $: playheadX = playhead.ticks * pxPerTick - scrollX;

  // Track header width constant
  const TRACK_HEADER_WIDTH = 200;

  onMount(() => {
    void sequencerActions.initListeners();
  });

  onDestroy(() => {
    sequencerActions.cleanupListeners();
  });

  // Handle scroll sync between timeline and tracks
  function handleTracksScroll(event: Event) {
    const target = event.target as HTMLDivElement;
    sequencerActions.setScrollX(target.scrollLeft);
    sequencerActions.setScrollY(target.scrollTop);
  }

  // Handle zoom with mouse wheel
  function handleWheel(event: WheelEvent) {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      if (event.deltaY < 0) {
        sequencerActions.zoomIn();
      } else {
        sequencerActions.zoomOut();
      }
    }
  }

  // Handle keyboard shortcuts
  function handleKeyDown(event: KeyboardEvent) {
    // Don't handle if typing in an input
    if ((event.target as HTMLElement).tagName === 'INPUT') { return; }

    switch (event.key) {
      case ' ':
        event.preventDefault();
        if (state.isPlaying) {
          void sequencerActions.pause();
        } else {
          void sequencerActions.play();
        }
        break;
      case 'Enter':
        event.preventDefault();
        void sequencerActions.stop();
        break;
      case 'Home':
        event.preventDefault();
        sequencerActions.gotoStart();
        break;
      case 'End':
        event.preventDefault();
        sequencerActions.gotoEnd();
        break;
      case 'Delete':
      case 'Backspace':
        if (state.selectedClipIds.length > 0) {
          event.preventDefault();
          sequencerActions.deleteSelectedClips();
        }
        break;
      case 'd':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          sequencerActions.duplicateSelectedClips();
        }
        break;
      case 'a':
        if (event.ctrlKey || event.metaKey) {
          event.preventDefault();
          // Select all clips
          const allClipIds: number[] = [];
          for (const track of tracks) {
            for (const clip of track.clips) {
              allClipIds.push(clip.id);
            }
          }
          allClipIds.forEach((id, i) =>
            sequencerActions.selectClip(id, i > 0)
          );
        }
        break;
      case 'l':
        if (!event.ctrlKey && !event.metaKey) {
          event.preventDefault();
          sequencerActions.setLoopEnabled(!project.loopEnabled);
        }
        break;
      case 'r':
        if (!event.ctrlKey && !event.metaKey) {
          event.preventDefault();
          if (state.isRecording) {
            void sequencerActions.stop();
          } else {
            void sequencerActions.record();
          }
        }
        break;
    }
  }

  // Handle drag and drop for MIDI files
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
    isDragOver = true;
  }

  function handleDragLeave(event: DragEvent) {
    const relatedTarget = event.relatedTarget as HTMLElement | null;
    const currentTarget = event.currentTarget as HTMLElement;
    if (!relatedTarget || !currentTarget.contains(relatedTarget)) { isDragOver = false; }
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;

    if (!event.dataTransfer) { return; }

    const jsonData = event.dataTransfer.getData('application/json');
    if (!jsonData) { return; }

    try {
      const data = JSON.parse(jsonData);
      if (data.type !== 'midi-file') { return; }

      // Show loading state
      isLoadingFile = true;
      loadingFileName = data.filename;

      // Calculate drop position
      const rect = tracksContainerRef.getBoundingClientRect();
      const x = event.clientX - rect.left + scrollX - TRACK_HEADER_WIDTH;
      const y = event.clientY - rect.top + scrollY;

      // Find target track based on y position
      let targetTrack: TrackType | null = null;
      let accumulatedHeight = 0;
      for (const track of tracks) {
        if (y >= accumulatedHeight && y < accumulatedHeight + track.height) {
          targetTrack = track;
          break;
        }
        accumulatedHeight += track.height;
      }

      // Calculate start tick from x position
      const startTick = Math.max(0, Math.floor(x / pxPerTick));

      // Create new track if dropped below existing tracks
      let trackId: number;
      if (targetTrack) {
        trackId = targetTrack.id;
      } else {
        const newTrackName = data.filename.replace(/\.(mid|midi)$/i, '');
        trackId = sequencerActions.addTrack(newTrackName);
      }

      // Load MIDI file into the DAW engine
      try {
        const loadedTrackId = await Vip3BrowserApi.loadFileToDaw(data.id);
        console.log(`Loaded file ${data.id} to DAW as track ${loadedTrackId}`);

        // Add clip to track for visual representation
        const clipName = data.filename.replace(/\.(mid|midi)$/i, '');
        // Default clip length: 4 bars
        const clipLength = project.timeSignature[0] * state.ticksPerBeat * 4;
        sequencerActions.addClip(trackId, startTick, clipLength, data.id, clipName);

        console.log(`Added clip "${clipName}" to track at tick ${startTick}`);

        // Success notification
        toastStore.success(`Successfully loaded "${data.filename}"`);
      } catch (loadError) {
        console.error('Failed to load MIDI file to DAW:', loadError);
        // Error notification
        toastStore.error(`Failed to load "${data.filename}": ${loadError}`);
        throw loadError;
      } finally {
        isLoadingFile = false;
        loadingFileName = '';
      }
    } catch (error) {
      console.error('Failed to process dropped file:', error);
      isLoadingFile = false;
      loadingFileName = '';
    }
  }

  // Handle click on empty area to deselect
  function handleBackgroundClick(event: MouseEvent) {
    if (event.target === tracksContainerRef) {
      sequencerActions.deselectAllClips();
      sequencerActions.deselectAllTracks();
    }
  }

  // Add new track
  function handleAddTrack() {
    const trackName = prompt('Track name:') || `Track ${tracks.length + 1}`;
    sequencerActions.addTrack(trackName);
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div
  bind:this={containerRef}
  class="sequencer"
  class:drag-over={isDragOver}
  on:wheel={handleWheel}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="application"
  aria-label="Sequencer"
  style="--track-header-width: {TRACK_HEADER_WIDTH}px;"
>
  <!-- Transport Bar -->
  <SequencerTransport />

  <!-- Main Content Area -->
  <div class="sequencer-content">
    <!-- Track Headers + Timeline -->
    <div class="header-row">
      <div class="track-header-spacer">
        <span class="track-count">{tracks.length} Tracks</span>
      </div>
      <div class="timeline-wrapper" style="margin-left: -{scrollX}px;">
        <SequencerTimeline
          {zoom}
          bpm={project.bpm}
          timeSignature={project.timeSignature}
          lengthBars={project.lengthBars}
          loopEnabled={project.loopEnabled}
          loopStartBar={project.loopStartBar}
          loopEndBar={project.loopEndBar}
          playheadTicks={playhead.ticks}
          ticksPerBeat={state.ticksPerBeat}
        />
      </div>
    </div>

    <!-- Track List with Clips -->
    <div
      bind:this={tracksContainerRef}
      class="tracks-container"
      on:scroll={handleTracksScroll}
      on:click={handleBackgroundClick}
      role="list"
    >
      <!-- Playhead Line -->
      {#if playheadX >= 0}
        <div
          class="playhead-line"
          style="left: {playheadX + TRACK_HEADER_WIDTH}px;"
        />
      {/if}

      <!-- Loop Region -->
      {#if project.loopEnabled}
        {@const loopStartX = (project.loopStartBar - 1) * project.timeSignature[0] * state.ticksPerBeat * pxPerTick - scrollX}
        {@const loopEndX = (project.loopEndBar) * project.timeSignature[0] * state.ticksPerBeat * pxPerTick - scrollX}
        <div
          class="loop-region"
          style="left: {loopStartX + TRACK_HEADER_WIDTH}px; width: {loopEndX - loopStartX}px;"
        />
      {/if}

      <!-- Tracks -->
      {#each tracks as track, index (track.id)}
        <SequencerTrack
          {track}
          {index}
          {zoom}
          {scrollX}
          ticksPerBeat={state.ticksPerBeat}
          snapValue={state.snapValue}
          headerWidth={TRACK_HEADER_WIDTH}
          contentWidth={contentWidth}
        />
      {/each}

      <!-- Add Track Button -->
      <div class="add-track-row">
        <div class="track-header add-track-header">
          <button class="add-track-btn" on:click={handleAddTrack}>
            + Add Track
          </button>
        </div>
        <div class="track-clips add-track-area">
          <span class="drop-hint">Drop MIDI files here to create tracks</span>
        </div>
      </div>

      <!-- Grid background (virtual scroll placeholder) -->
      <div
        class="grid-background"
        style="width: {contentWidth + TRACK_HEADER_WIDTH}px;"
      />
    </div>
  </div>

  <!-- Loading Overlay -->
  {#if isLoadingFile}
    <div class="loading-overlay">
      <div class="loading-spinner">
        <div class="spinner"></div>
        <p>Loading {loadingFileName}...</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .sequencer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1a1a1a);
    color: var(--text-color, #e0e0e0);
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    overflow: hidden;
    user-select: none;
  }

  .sequencer.drag-over {
    outline: 2px dashed var(--primary-color, #3b82f6);
    outline-offset: -2px;
  }

  .sequencer-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header-row {
    display: flex;
    height: 40px;
    background: var(--menu-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
    flex-shrink: 0;
  }

  .track-header-spacer {
    width: var(--track-header-width);
    min-width: var(--track-header-width);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--menu-bg, #2a2a2a);
    border-right: 1px solid var(--border-color, #3a3a3a);
  }

  .track-count {
    font-size: 11px;
    color: var(--text-muted, #888);
    font-weight: 500;
  }

  .timeline-wrapper {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .tracks-container {
    flex: 1;
    overflow: auto;
    position: relative;
    background: var(--window-subtle, #222);
  }

  .playhead-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--error-color, #ef4444);
    z-index: 100;
    pointer-events: none;
  }

  .playhead-line::before {
    content: '';
    position: absolute;
    top: 0;
    left: -5px;
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-top: 8px solid var(--error-color, #ef4444);
  }

  .loop-region {
    position: absolute;
    top: 0;
    bottom: 0;
    background: rgba(59, 130, 246, 0.1);
    border-left: 2px solid var(--primary-color, #3b82f6);
    border-right: 2px solid var(--primary-color, #3b82f6);
    z-index: 1;
    pointer-events: none;
  }

  .add-track-row {
    display: flex;
    min-height: 60px;
    border-bottom: 1px solid var(--border-color, #3a3a3a);
  }

  .add-track-header {
    width: var(--track-header-width);
    min-width: var(--track-header-width);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--menu-bg, #2a2a2a);
    border-right: 1px solid var(--border-color, #3a3a3a);
  }

  .add-track-btn {
    padding: 8px 16px;
    background: var(--primary-color, #3b82f6);
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s;
  }

  .add-track-btn:hover {
    background: var(--primary-dark, #2563eb);
  }

  .add-track-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--window-subtle, #222);
    border: 2px dashed var(--border-color, #3a3a3a);
    margin: 8px;
    border-radius: 8px;
  }

  .drop-hint {
    color: var(--text-muted, #666);
    font-size: 12px;
  }

  .track-clips {
    flex: 1;
    position: relative;
  }

  .grid-background {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    pointer-events: none;
    background-image:
      linear-gradient(to right, var(--border-color, #3a3a3a) 1px, transparent 1px),
      linear-gradient(to bottom, var(--border-color, #3a3a3a) 1px, transparent 1px);
    background-size: 40px 80px; /* Will be dynamic based on zoom */
    opacity: 0.3;
    z-index: 0;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .loading-spinner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 32px;
    background: var(--menu-bg, #2a2a2a);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 8px;
  }

  .loading-spinner p {
    margin: 0;
    color: var(--text-color, #e0e0e0);
    font-size: 14px;
    font-weight: 500;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-color, #3a3a3a);
    border-top: 3px solid var(--primary-color, #3b82f6);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
