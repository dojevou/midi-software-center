<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '../api';
  import type { Track, FileDetails } from '../api';
  import { searchResults } from '../stores';

  // Sequencer state
  let tracks: Track[] = [];
  let isPlaying = false;
  let playbackPosition = 0;
  let currentTempo = 120;
  let timeSignature = { numerator: 4, denominator: 4 };
  let masterVolume = 100;
  let zoom = 1;
  let snapToGrid = true;
  let scrollX = 0;

  // Canvas refs
  let timelineCanvas: HTMLCanvasElement;
  let timelineCtx: CanvasRenderingContext2D | null;

  // Constants
  const trackHeight = 60;
  const trackListWidth = 250;
  const rulerHeight = 40;
  const pixelsPerBeat = 40;
  const colors = [
    '#ff3e00',
    '#4caf50',
    '#2196f3',
    '#9c27b0',
    '#ff9800',
    '#e91e63',
    '#00bcd4',
    '#8bc34a',
    '#ffc107',
    '#795548',
  ];

  // File selection modal
  let showFileModal = false;
  let selectedChannel = 0;

  // Playback update interval
  let playbackInterval: number | null = null;

  // Drag and drop state
  let isDragOverTimeline = false;
  let isDragOverTrackList = false;

  $: beatsPerBar = timeSignature.numerator;
  $: beatWidth = pixelsPerBeat * zoom;
  $: barWidth = beatWidth * beatsPerBar;

  onMount(async () => {
    await loadTracks();
    await loadTempo();

    if (timelineCanvas) {
      timelineCtx = timelineCanvas.getContext('2d');
      resizeCanvas();
      drawTimeline();
    }

    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    if (playbackInterval) {
      clearInterval(playbackInterval);
    }
    window.removeEventListener('resize', handleResize);
  });

  async function loadTracks() {
    try {
      tracks = await api.sequencer.getTracks();
    } catch (error) {
      console.error('Failed to load tracks:', error);
    }
  }

  async function loadTempo() {
    try {
      currentTempo = await api.playback.getTempo();
    } catch (error) {
      console.error('Failed to load tempo:', error);
    }
  }

  async function addTrack() {
    showFileModal = true;
  }

  async function selectFileForTrack(file: FileDetails) {
    try {
      const newTrack = await api.sequencer.addTrack(file.id, selectedChannel);

      // Assign color
      const colorIndex = tracks.length % colors.length;
      await api.sequencer.updateTrack(newTrack.id, {
        color: colors[colorIndex],
        name: file.file_name,
      });

      await loadTracks();
      drawTimeline();
      showFileModal = false;
      selectedChannel = (selectedChannel + 1) % 16;
    } catch (error) {
      console.error('Failed to add track:', error);
      alert('Failed to add track: ' + error);
    }
  }

  async function removeTrack(trackId: number) {
    if (!confirm('Remove this track?')) return;

    try {
      await api.sequencer.removeTrack(trackId);
      await loadTracks();
      drawTimeline();
    } catch (error) {
      console.error('Failed to remove track:', error);
    }
  }

  async function toggleMute(trackId: number) {
    const track = tracks.find((t) => t.id === trackId);
    if (!track) return;

    try {
      await api.sequencer.updateTrack(trackId, { muted: !track.muted });
      await loadTracks();
    } catch (error) {
      console.error('Failed to toggle mute:', error);
    }
  }

  async function toggleSolo(trackId: number) {
    const track = tracks.find((t) => t.id === trackId);
    if (!track) return;

    try {
      await api.sequencer.updateTrack(trackId, { solo: !track.solo });
      await loadTracks();
    } catch (error) {
      console.error('Failed to toggle solo:', error);
    }
  }

  async function updateTrackVolume(trackId: number, volume: number) {
    try {
      await api.sequencer.updateTrack(trackId, { volume: Math.round(volume) });
    } catch (error) {
      console.error('Failed to update volume:', error);
    }
  }

  async function updateTrackName(trackId: number, name: string) {
    try {
      await api.sequencer.updateTrack(trackId, { name });
      await loadTracks();
    } catch (error) {
      console.error('Failed to update name:', error);
    }
  }

  async function startPlayback() {
    try {
      await api.sequencer.start();
      isPlaying = true;
      startPlaybackUpdates();
    } catch (error) {
      console.error('Failed to start playback:', error);
    }
  }

  async function stopPlayback() {
    try {
      await api.sequencer.stop();
      isPlaying = false;
      playbackPosition = 0;
      stopPlaybackUpdates();
      drawTimeline();
    } catch (error) {
      console.error('Failed to stop playback:', error);
    }
  }

  function startPlaybackUpdates() {
    if (playbackInterval) {
      clearInterval(playbackInterval);
    }

    playbackInterval = window.setInterval(async () => {
      try {
        const position = await api.sequencer.getPosition();
        playbackPosition = position.current_tick;
        drawTimeline();
      } catch (error) {
        console.error('Failed to get position:', error);
      }
    }, 50);
  }

  function stopPlaybackUpdates() {
    if (playbackInterval) {
      clearInterval(playbackInterval);
      playbackInterval = null;
    }
  }

  async function setTempo(bpm: number) {
    try {
      await api.playback.setTempo(bpm);
      currentTempo = bpm;
    } catch (error) {
      console.error('Failed to set tempo:', error);
    }
  }

  function handleResize() {
    resizeCanvas();
    drawTimeline();
  }

  function resizeCanvas() {
    if (!timelineCanvas) return;

    timelineCanvas.width = timelineCanvas.offsetWidth;
    timelineCanvas.height = timelineCanvas.offsetHeight;
  }

  function drawTimeline() {
    if (!timelineCtx || !timelineCanvas) return;

    const width = timelineCanvas.width;
    const height = timelineCanvas.height;

    // Clear canvas
    timelineCtx.fillStyle = '#1a1a1a';
    timelineCtx.fillRect(0, 0, width, height);

    // Draw time ruler
    drawTimeRuler(width);

    // Draw tracks
    tracks.forEach((track, index) => {
      drawTrack(track, index, width);
    });

    // Draw playhead
    if (playbackPosition > 0 || isPlaying) {
      drawPlayhead(width, height);
    }

    // Draw grid lines
    drawGrid(width, height);
  }

  function drawTimeRuler(width: number) {
    if (!timelineCtx) return;

    const y = 0;

    // Background
    timelineCtx.fillStyle = '#0a0a0a';
    timelineCtx.fillRect(0, y, width, rulerHeight);

    // Draw bars
    let barNum = 1;
    for (let x = 0; x < width + scrollX; x += barWidth) {
      const drawX = x - scrollX;
      if (drawX < 0 || drawX > width) continue;

      // Bar number
      timelineCtx.fillStyle = '#fff';
      timelineCtx.font = 'bold 12px monospace';
      timelineCtx.fillText(`${barNum}`, drawX + 4, rulerHeight - 10);

      // Bar line
      timelineCtx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
      timelineCtx.lineWidth = 2;
      timelineCtx.beginPath();
      timelineCtx.moveTo(drawX, 20);
      timelineCtx.lineTo(drawX, rulerHeight);
      timelineCtx.stroke();

      // Beat marks
      for (let beat = 1; beat < beatsPerBar; beat++) {
        const beatX = drawX + beat * beatWidth;
        if (beatX < 0 || beatX > width) continue;

        timelineCtx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
        timelineCtx.lineWidth = 1;
        timelineCtx.beginPath();
        timelineCtx.moveTo(beatX, 25);
        timelineCtx.lineTo(beatX, rulerHeight);
        timelineCtx.stroke();

        // Beat number
        timelineCtx.fillStyle = 'rgba(255, 255, 255, 0.6)';
        timelineCtx.font = '10px monospace';
        timelineCtx.fillText(`${beat + 1}`, beatX + 2, rulerHeight - 12);
      }

      barNum++;
    }

    // Border
    timelineCtx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    timelineCtx.lineWidth = 1;
    timelineCtx.strokeRect(0, y, width, rulerHeight);
  }

  function drawTrack(track: Track, index: number, width: number) {
    if (!timelineCtx) return;

    const y = rulerHeight + index * trackHeight;

    // Track background
    const isActive = track.solo || (!tracks.some((t) => t.solo) && !track.muted);
    timelineCtx.fillStyle = isActive ? 'rgba(0, 0, 0, 0.3)' : 'rgba(0, 0, 0, 0.6)';
    timelineCtx.fillRect(0, y, width, trackHeight);

    // Track region (represents the MIDI file)
    const regionWidth = barWidth * 4; // Default 4 bars
    const regionX = 0 - scrollX;
    const regionY = y + 8;
    const regionHeight = trackHeight - 16;

    if (regionX + regionWidth > 0 && regionX < width) {
      // Region background
      const color = track.color || colors[index % colors.length];
      timelineCtx.fillStyle = color + '40'; // 25% opacity
      timelineCtx.fillRect(Math.max(0, regionX), regionY, regionWidth, regionHeight);

      // Region border
      timelineCtx.strokeStyle = color;
      timelineCtx.lineWidth = 2;
      timelineCtx.strokeRect(Math.max(0, regionX), regionY, regionWidth, regionHeight);

      // File name in region
      timelineCtx.fillStyle = '#fff';
      timelineCtx.font = '12px monospace';
      timelineCtx.fillText(track.name, Math.max(4, regionX + 4), regionY + 20);
    }

    // Track separator
    timelineCtx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    timelineCtx.lineWidth = 1;
    timelineCtx.beginPath();
    timelineCtx.moveTo(0, y + trackHeight);
    timelineCtx.lineTo(width, y + trackHeight);
    timelineCtx.stroke();
  }

  function drawGrid(width: number, height: number) {
    if (!timelineCtx) return;

    // Vertical grid lines (bars and beats)
    for (let x = 0; x < width + scrollX; x += beatWidth) {
      const drawX = x - scrollX;
      if (drawX < 0 || drawX > width) continue;

      const isBar = Math.abs(x % barWidth) < 0.1;
      timelineCtx.strokeStyle = isBar ? 'rgba(255, 255, 255, 0.08)' : 'rgba(255, 255, 255, 0.03)';
      timelineCtx.lineWidth = isBar ? 2 : 1;

      timelineCtx.beginPath();
      timelineCtx.moveTo(drawX, rulerHeight);
      timelineCtx.lineTo(drawX, height);
      timelineCtx.stroke();
    }
  }

  function drawPlayhead(width: number, height: number) {
    if (!timelineCtx) return;

    // Convert tick to pixels (assuming 480 ticks per beat)
    const ticksPerBeat = 480;
    const x = (playbackPosition / ticksPerBeat) * beatWidth - scrollX;

    if (x < 0 || x > width) return;

    // Playhead line
    timelineCtx.strokeStyle = 'rgba(255, 0, 0, 0.8)';
    timelineCtx.lineWidth = 2;
    timelineCtx.beginPath();
    timelineCtx.moveTo(x, 0);
    timelineCtx.lineTo(x, height);
    timelineCtx.stroke();

    // Playhead triangle at top
    timelineCtx.fillStyle = 'rgba(255, 0, 0, 0.8)';
    timelineCtx.beginPath();
    timelineCtx.moveTo(x, rulerHeight);
    timelineCtx.lineTo(x - 6, rulerHeight - 10);
    timelineCtx.lineTo(x + 6, rulerHeight - 10);
    timelineCtx.closePath();
    timelineCtx.fill();
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();

    if (e.ctrlKey || e.metaKey) {
      // Zoom
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      zoom = Math.max(0.5, Math.min(4, zoom * delta));
    } else {
      // Horizontal scroll
      scrollX = Math.max(0, scrollX + e.deltaY);
    }

    drawTimeline();
  }

  function zoomIn() {
    zoom = Math.min(4, zoom * 1.1);
    drawTimeline();
  }

  function zoomOut() {
    zoom = Math.max(0.5, zoom * 0.9);
    drawTimeline();
  }

  function resetView() {
    zoom = 1;
    scrollX = 0;
    drawTimeline();
  }

  // Drag and drop handlers for timeline (add new track)
  async function handleTimelineDrop(event: DragEvent) {
    event.preventDefault();
    isDragOverTimeline = false;

    if (!event.dataTransfer) return;

    try {
      const data = JSON.parse(event.dataTransfer.getData('application/json'));

      if (data.type === 'midi-file') {
        // Add track with the dropped file
        const trackName = data.filename.replace(/\.mid$/i, '');
        const newTrack = await api.sequencer.addTrack(data.fileId, selectedChannel);

        // Assign color
        const colorIndex = tracks.length % colors.length;
        await api.sequencer.updateTrack(newTrack.id, {
          color: colors[colorIndex],
          name: trackName,
        });

        await loadTracks();
        drawTimeline();
        selectedChannel = (selectedChannel + 1) % 16;
      }
    } catch (error) {
      console.error('Failed to add track from drop:', error);
      alert('Failed to add track: ' + error);
    }
  }

  function handleTimelineDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
    isDragOverTimeline = true;
  }

  function handleTimelineDragLeave(event: DragEvent) {
    // Only set to false if we're actually leaving the timeline container
    const relatedTarget = event.relatedTarget as HTMLElement;
    if (
      !relatedTarget ||
      !event.currentTarget ||
      !(event.currentTarget as HTMLElement).contains(relatedTarget)
    ) {
      isDragOverTimeline = false;
    }
  }

  // Drag and drop handlers for track reordering
  let draggedTrackId: number | null = null;
  let dropTargetTrackId: number | null = null;

  function handleTrackDragStart(event: DragEvent, trackId: number) {
    if (!event.dataTransfer) return;

    draggedTrackId = trackId;
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData(
      'application/json',
      JSON.stringify({
        type: 'track',
        trackId: trackId,
      })
    );
  }

  function handleTrackDrop(event: DragEvent, targetTrackId: number) {
    event.preventDefault();
    event.stopPropagation();
    dropTargetTrackId = null;
    isDragOverTrackList = false;

    if (!event.dataTransfer) return;

    try {
      const data = JSON.parse(event.dataTransfer.getData('application/json'));

      if (data.type === 'track' && data.trackId !== targetTrackId) {
        // Reorder tracks in the array
        const draggedIndex = tracks.findIndex((t) => t.id === data.trackId);
        const targetIndex = tracks.findIndex((t) => t.id === targetTrackId);

        if (draggedIndex !== -1 && targetIndex !== -1) {
          const newTracks = [...tracks];
          const [draggedTrack] = newTracks.splice(draggedIndex, 1);
          newTracks.splice(targetIndex, 0, draggedTrack);
          tracks = newTracks;
          drawTimeline();
        }
      }
    } catch (error) {
      console.error('Failed to reorder tracks:', error);
    }

    draggedTrackId = null;
  }

  function handleTrackDragOver(event: DragEvent, trackId: number) {
    event.preventDefault();
    event.stopPropagation();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    dropTargetTrackId = trackId;
    isDragOverTrackList = true;
  }

  function handleTrackDragEnd() {
    draggedTrackId = null;
    dropTargetTrackId = null;
    isDragOverTrackList = false;
  }
</script>

<div class="sequencer-container">
  <!-- Header Controls -->
  <div class="sequencer-header">
    <div class="transport-controls">
      {#if !isPlaying}
        <button class="transport-btn play" on:click={startPlayback} title="Play"> ▶️ </button>
      {:else}
        <button class="transport-btn stop" on:click={stopPlayback} title="Stop"> ⏹️ </button>
      {/if}

      <div class="tempo-control">
        <label>BPM:</label>
        <input
          type="number"
          min="40"
          max="240"
          value={currentTempo}
          on:change={(e) => setTempo(Number(e.currentTarget.value))}
        />
      </div>

      <div class="time-sig">
        <span>{timeSignature.numerator}/{timeSignature.denominator}</span>
      </div>

      <div class="master-volume">
        <label>Master:</label>
        <input type="range" min="0" max="100" bind:value={masterVolume} />
        <span>{masterVolume}</span>
      </div>
    </div>

    <div class="view-controls">
      <button
        class="snap-btn {snapToGrid ? 'active' : ''}"
        on:click={() => (snapToGrid = !snapToGrid)}
      >
        {snapToGrid ? '🧲 Snap: ON' : '🧲 Snap: OFF'}
      </button>

      <div class="zoom-controls">
        <button on:click={zoomOut}>🔍-</button>
        <span>{(zoom * 100).toFixed(0)}%</span>
        <button on:click={zoomIn}>🔍+</button>
        <button on:click={resetView}>Reset</button>
      </div>

      <button class="add-track-btn" on:click={addTrack}> ➕ Add Track </button>
    </div>
  </div>

  <!-- Main Content -->
  <div class="sequencer-content">
    <!-- Track List -->
    <div class="track-list" style="width: {trackListWidth}px">
      <div class="track-list-header" style="height: {rulerHeight}px">
        <span>Tracks</span>
      </div>

      {#each tracks as track, index (track.id)}
        <div
          class="track-item {dropTargetTrackId === track.id
            ? 'drop-target'
            : ''} {draggedTrackId === track.id ? 'dragging' : ''}"
          style="height: {trackHeight}px"
          draggable="true"
          on:dragstart={(e) => handleTrackDragStart(e, track.id)}
          on:drop={(e) => handleTrackDrop(e, track.id)}
          on:dragover={(e) => handleTrackDragOver(e, track.id)}
          on:dragend={handleTrackDragEnd}
        >
          <div
            class="track-color"
            style="background-color: {track.color || colors[index % colors.length]}"
          ></div>

          <div class="track-info">
            <input
              type="text"
              class="track-name"
              value={track.name}
              on:blur={(e) => updateTrackName(track.id, e.currentTarget.value)}
            />
            <span class="track-channel">Ch: {track.channel + 1}</span>
          </div>

          <div class="track-controls">
            <button
              class="control-btn mute {track.muted ? 'active' : ''}"
              on:click={() => toggleMute(track.id)}
              title="Mute"
            >
              M
            </button>
            <button
              class="control-btn solo {track.solo ? 'active' : ''}"
              on:click={() => toggleSolo(track.id)}
              title="Solo"
            >
              S
            </button>
          </div>

          <div class="track-volume">
            <input
              type="range"
              min="0"
              max="127"
              value={track.volume}
              on:input={(e) => updateTrackVolume(track.id, Number(e.currentTarget.value))}
            />
            <span>{Math.round((track.volume / 127) * 100)}%</span>
          </div>

          <button
            class="delete-track-btn"
            on:click={() => removeTrack(track.id)}
            title="Delete Track"
          >
            🗑️
          </button>
        </div>
      {/each}

      {#if tracks.length === 0}
        <div class="empty-tracks">
          <p>No tracks yet</p>
          <p class="hint">Click "Add Track" to start</p>
        </div>
      {/if}
    </div>

    <!-- Timeline -->
    <div
      class="timeline-container {isDragOverTimeline ? 'drag-over' : ''}"
      on:drop={handleTimelineDrop}
      on:dragover={handleTimelineDragOver}
      on:dragleave={handleTimelineDragLeave}
    >
      <canvas
        bind:this={timelineCanvas}
        class="timeline-canvas"
        on:wheel={handleWheel}
        style="height: {rulerHeight + tracks.length * trackHeight}px"
      />

      {#if isDragOverTimeline}
        <div class="drop-indicator">
          <div class="drop-indicator-content">
            <span class="drop-icon">🎵</span>
            <span class="drop-text">Drop file here to add track</span>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Status Bar -->
  <div class="sequencer-status">
    <div class="status-left">
      <span>💡 Scroll to pan | Ctrl+Scroll to zoom</span>
    </div>
    <div class="status-right">
      <span>{tracks.length} track{tracks.length !== 1 ? 's' : ''}</span>
      {#if isPlaying}
        <span class="playing-indicator">▶️ Playing</span>
      {/if}
    </div>
  </div>
</div>

<!-- File Selection Modal -->
{#if showFileModal}
  <div class="modal-overlay" on:click={() => (showFileModal = false)}>
    <div class="modal-content" on:click={(e) => e.stopPropagation()}>
      <h3>Select MIDI File for Track</h3>

      <div class="channel-selector">
        <label>MIDI Channel:</label>
        <select bind:value={selectedChannel}>
          {#each Array(16) as _, i}
            <option value={i}>Channel {i + 1}</option>
          {/each}
        </select>
      </div>

      <div class="file-list">
        {#each $searchResults as file}
          <button class="file-item" on:click={() => selectFileForTrack(file)}>
            <span class="file-name">{file.file_name}</span>
            {#if file.bpm}
              <span class="file-info">{file.bpm} BPM</span>
            {/if}
            {#if file.key}
              <span class="file-info">{file.key}</span>
            {/if}
          </button>
        {/each}
      </div>

      <button class="modal-close" on:click={() => (showFileModal = false)}> Cancel </button>
    </div>
  </div>
{/if}

<style>
  .sequencer-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    overflow: hidden;
  }

  .sequencer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.5);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    gap: 1rem;
    flex-wrap: wrap;
  }

  .transport-controls,
  .view-controls {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .transport-btn {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-radius: 6px;
    color: #fff;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .transport-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .transport-btn.play {
    border-color: #4caf50;
    color: #4caf50;
  }

  .transport-btn.stop {
    border-color: #f44336;
    color: #f44336;
  }

  .tempo-control,
  .time-sig,
  .master-volume {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .tempo-control label,
  .master-volume label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .tempo-control input {
    width: 60px;
    padding: 0.4rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
  }

  .time-sig {
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.8);
  }

  .master-volume input[type='range'] {
    width: 80px;
  }

  .master-volume span {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
    min-width: 35px;
  }

  .snap-btn {
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .snap-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .snap-btn.active {
    background: rgba(100, 200, 255, 0.3);
    border-color: rgba(100, 200, 255, 0.5);
  }

  .zoom-controls {
    display: flex;
    gap: 0.25rem;
    align-items: center;
  }

  .zoom-controls button {
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .zoom-controls button:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .zoom-controls span {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
    min-width: 50px;
    text-align: center;
  }

  .add-track-btn {
    padding: 0.5rem 1rem;
    background: rgba(255, 62, 0, 0.2);
    border: 2px solid #ff3e00;
    border-radius: 6px;
    color: #ff3e00;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .add-track-btn:hover {
    background: rgba(255, 62, 0, 0.3);
  }

  .sequencer-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .track-list {
    background: rgba(0, 0, 0, 0.4);
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    overflow-y: auto;
  }

  .track-list-header {
    display: flex;
    align-items: center;
    padding: 0 1rem;
    background: #0a0a0a;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
  }

  .track-item {
    display: flex;
    align-items: center;
    padding: 0.5rem;
    gap: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    transition: background 0.2s;
  }

  .track-item:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .track-item.dragging {
    opacity: 0.5;
    cursor: grabbing;
  }

  .track-item.drop-target {
    background: rgba(59, 130, 246, 0.15);
    border-top: 3px solid #3b82f6;
  }

  .track-item:active {
    cursor: grabbing;
  }

  .track-color {
    width: 4px;
    height: 40px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .track-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .track-name {
    background: transparent;
    border: 1px solid transparent;
    color: #fff;
    font-size: 0.875rem;
    padding: 0.25rem;
    border-radius: 3px;
    width: 100%;
  }

  .track-name:hover {
    border-color: rgba(255, 255, 255, 0.2);
  }

  .track-name:focus {
    outline: none;
    border-color: #ff3e00;
    background: rgba(255, 255, 255, 0.05);
  }

  .track-channel {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .track-controls {
    display: flex;
    gap: 0.25rem;
  }

  .control-btn {
    width: 28px;
    height: 28px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .control-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .control-btn.mute.active {
    background: rgba(255, 152, 0, 0.3);
    border-color: #ff9800;
    color: #ff9800;
  }

  .control-btn.solo.active {
    background: rgba(255, 235, 59, 0.3);
    border-color: #ffeb3b;
    color: #ffeb3b;
  }

  .track-volume {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .track-volume input[type='range'] {
    width: 60px;
  }

  .track-volume span {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.6);
    min-width: 30px;
  }

  .delete-track-btn {
    padding: 0.25rem 0.5rem;
    background: rgba(244, 67, 54, 0.2);
    border: 1px solid rgba(244, 67, 54, 0.3);
    border-radius: 4px;
    color: #f44336;
    cursor: pointer;
    transition: all 0.2s;
  }

  .delete-track-btn:hover {
    background: rgba(244, 67, 54, 0.3);
  }

  .empty-tracks {
    padding: 2rem;
    text-align: center;
    color: rgba(255, 255, 255, 0.5);
  }

  .empty-tracks p {
    margin: 0.5rem 0;
  }

  .empty-tracks .hint {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.3);
  }

  .timeline-container {
    flex: 1;
    overflow: hidden;
    position: relative;
    transition: all 0.2s;
  }

  .timeline-container.drag-over {
    background: rgba(59, 130, 246, 0.1);
    border: 2px dashed #3b82f6;
  }

  .drop-indicator {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    z-index: 100;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .drop-indicator-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem 3rem;
    background: rgba(59, 130, 246, 0.2);
    border: 2px dashed #3b82f6;
    border-radius: 12px;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      transform: scale(1);
      border-color: #3b82f6;
    }
    50% {
      transform: scale(1.05);
      border-color: #60a5fa;
    }
  }

  .drop-icon {
    font-size: 3rem;
    animation: bounce 1s ease-in-out infinite;
  }

  @keyframes bounce {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-10px);
    }
  }

  .drop-text {
    font-size: 1.25rem;
    font-weight: 600;
    color: #3b82f6;
    text-align: center;
  }

  .timeline-canvas {
    width: 100%;
    display: block;
    cursor: grab;
  }

  .timeline-canvas:active:not(.drag-over) {
    cursor: grabbing;
  }

  .sequencer-status {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    background: rgba(0, 0, 0, 0.5);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .playing-indicator {
    color: #4caf50;
    font-weight: 600;
  }

  /* Modal Styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #2d2d2d;
    border: 2px solid rgba(255, 62, 0, 0.5);
    border-radius: 12px;
    padding: 2rem;
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .modal-content h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    color: #ff3e00;
  }

  .channel-selector {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-bottom: 1rem;
  }

  .channel-selector label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .channel-selector select {
    padding: 0.5rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .file-item {
    padding: 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .file-item:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: #ff3e00;
  }

  .file-name {
    flex: 1;
    font-weight: 500;
  }

  .file-info {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.6);
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
  }

  .modal-close {
    width: 100%;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 6px;
    color: #fff;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .modal-close:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  /* Input Range Styling */
  input[type='range'] {
    -webkit-appearance: none;
    appearance: none;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
    height: 4px;
    outline: none;
  }

  input[type='range']::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    background: #ff3e00;
    border-radius: 50%;
    cursor: pointer;
  }

  input[type='range']::-moz-range-thumb {
    width: 14px;
    height: 14px;
    background: #ff3e00;
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }
</style>
