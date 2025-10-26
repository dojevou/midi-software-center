<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { api } from '../api';
  import { selectedFile, selectedPattern } from '../stores';
  import type { MidiEvent } from '../api';

  const dispatch = createEventDispatcher();

  // Props
  export let fileData: any = null;
  export let playbackPosition: number = 0;

  // Canvas refs
  let canvas: HTMLCanvasElement;
  let timeRulerCanvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null;
  let rulerCtx: CanvasRenderingContext2D | null;

  // Tool types
  type Tool = 'select' | 'draw' | 'erase';
  type SnapValue = 0 | 4 | 8 | 16;

  // Piano roll settings
  let zoom = 1;
  let scrollX = 0;
  let scrollY = 0;
  const pixelsPerBeat = 40;
  const noteHeight = 12;
  const pianoWidth = 60;
  const rulerHeight = 40;
  const minNoteWidth = 2;

  // Interaction state
  let currentTool: Tool = 'select';
  let snapToGrid: SnapValue = 16;
  let selectedNoteIndex: number | null = null;
  let isDragging = false;
  let isResizing = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let draggedNote: MidiEvent | null = null;
  let resizeHandle: 'start' | 'end' | null = null;

  // MIDI note names
  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  // Performance optimization - cache rendered notes
  let cachedNotes: Array<{
    event: MidiEvent;
    x: number;
    y: number;
    width: number;
    noteOff: MidiEvent | null;
  }> = [];

  $: if ($selectedFile && canvas) {
    loadPattern();
  }

  $: if (canvas && ctx) {
    drawPianoRoll();
  }

  $: if (timeRulerCanvas && rulerCtx) {
    drawTimeRuler();
  }

  async function loadPattern() {
    if (!$selectedFile) return;

    try {
      const pattern = await api.files.loadPattern($selectedFile.id);
      $selectedPattern = pattern;
      cacheNotes();
      drawPianoRoll();
      drawTimeRuler();
    } catch (error) {
      console.error('Failed to load pattern:', error);
    }
  }

  function cacheNotes() {
    if (!$selectedPattern) return;

    const noteEvents = $selectedPattern.events.filter(
      (e: MidiEvent) => e.event_type === 'NoteOn' && e.note !== null
    );

    const ticksPerPixel = $selectedPattern.ticks_per_beat / pixelsPerBeat / zoom;

    cachedNotes = noteEvents.map((event: MidiEvent) => {
      const noteOff =
        $selectedPattern!.events.find(
          (e: MidiEvent) =>
            e.event_type === 'NoteOff' &&
            e.note === event.note &&
            e.tick > event.tick &&
            e.channel === event.channel
        ) || null;

      const x = pianoWidth + event.tick / ticksPerPixel;
      const y = (127 - (event.note || 0)) * noteHeight;
      const duration = noteOff ? (noteOff.tick - event.tick) / ticksPerPixel : 20;

      return {
        event,
        x,
        y,
        width: Math.max(duration, minNoteWidth),
        noteOff,
      };
    });
  }

  function drawPianoRoll() {
    if (!ctx || !$selectedPattern) return;

    const width = canvas.width;
    const height = canvas.height;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    // Draw grid
    drawGrid(width, height);

    // Draw piano keys
    drawPianoKeys();

    // Draw playhead
    if (playbackPosition > 0) {
      drawPlayhead();
    }

    // Draw notes
    drawNotes();
  }

  function drawGrid(width: number, height: number) {
    if (!ctx) return;

    const beatWidth = pixelsPerBeat * zoom;
    const barWidth = beatWidth * 4;

    // Vertical lines (beats and bars)
    for (let x = pianoWidth; x < width + scrollX; x += beatWidth) {
      const isBar = Math.abs((x - pianoWidth) % barWidth) < 0.1;
      ctx.strokeStyle = isBar ? 'rgba(255, 255, 255, 0.15)' : 'rgba(255, 255, 255, 0.05)';
      ctx.lineWidth = isBar ? 2 : 1;

      const drawX = x - scrollX;
      ctx.beginPath();
      ctx.moveTo(drawX, 0);
      ctx.lineTo(drawX, height);
      ctx.stroke();
    }

    // Horizontal lines (notes)
    for (let y = 0; y < height + scrollY; y += noteHeight) {
      const noteNum = 127 - Math.floor((y + scrollY) / noteHeight);
      const isWhiteKey = ![1, 3, 6, 8, 10].includes(noteNum % 12);

      ctx.strokeStyle = isWhiteKey ? 'rgba(255, 255, 255, 0.05)' : 'rgba(255, 255, 255, 0.02)';
      ctx.lineWidth = 1;

      const drawY = y - scrollY;
      ctx.beginPath();
      ctx.moveTo(pianoWidth, drawY);
      ctx.lineTo(width, drawY);
      ctx.stroke();
    }
  }

  function drawPianoKeys() {
    if (!ctx) return;

    const height = canvas.height;

    for (let y = -scrollY; y < height + scrollY; y += noteHeight) {
      const noteNum = 127 - Math.floor((y + scrollY) / noteHeight);
      if (noteNum < 0 || noteNum > 127) continue;

      const octave = Math.floor(noteNum / 12) - 1;
      const noteName = noteNames[noteNum % 12];
      const isWhiteKey = ![1, 3, 6, 8, 10].includes(noteNum % 12);

      const drawY = y - scrollY;

      // Draw key background
      ctx.fillStyle = isWhiteKey ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.3)';
      ctx.fillRect(0, drawY, pianoWidth, noteHeight);

      // Draw key border
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
      ctx.lineWidth = 1;
      ctx.strokeRect(0, drawY, pianoWidth, noteHeight);

      // Draw note name for C notes
      if (noteName === 'C') {
        ctx.fillStyle = '#fff';
        ctx.font = '10px monospace';
        ctx.fillText(`${noteName}${octave}`, 5, drawY + noteHeight - 2);
      }
    }
  }

  function drawNotes() {
    if (!ctx || cachedNotes.length === 0) return;

    const width = canvas.width;
    const height = canvas.height;

    // Only draw visible notes for performance
    cachedNotes.forEach((note, index) => {
      const drawX = note.x - scrollX;
      const drawY = note.y - scrollY;

      // Culling - skip notes outside viewport
      if (drawX + note.width < pianoWidth || drawX > width) return;
      if (drawY + noteHeight < 0 || drawY > height) return;

      const isSelected = index === selectedNoteIndex;
      const velocity = note.event.velocity || 100;

      // Velocity-based color coding (darker = quieter)
      const alpha = velocity / 127;
      const brightness = Math.floor(255 * alpha);

      // Color gradient from dark orange to bright orange based on velocity
      const r = 255;
      const g = Math.floor(62 + 193 * (1 - alpha)); // 62 to 255
      const b = 0;

      if (!ctx) return;

      // Fill note
      if (isSelected) {
        ctx.fillStyle = `rgba(100, 200, 255, 0.8)`;
      } else {
        ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${alpha * 0.7})`;
      }
      ctx.fillRect(drawX, drawY, note.width, noteHeight - 2);

      // Note border
      ctx.strokeStyle = isSelected ? `rgba(100, 200, 255, 1)` : `rgba(${r}, ${g}, ${b}, ${alpha})`;
      ctx.lineWidth = isSelected ? 2 : 1;
      ctx.strokeRect(drawX, drawY, note.width, noteHeight - 2);

      // Draw resize handles for selected note
      if (isSelected) {
        ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
        // Start handle
        ctx.fillRect(drawX - 2, drawY + noteHeight / 2 - 3, 4, 6);
        // End handle
        ctx.fillRect(drawX + note.width - 2, drawY + noteHeight / 2 - 3, 4, 6);
      }
    });
  }

  function drawPlayhead() {
    if (!ctx || !$selectedPattern) return;

    const ticksPerPixel = $selectedPattern.ticks_per_beat / pixelsPerBeat / zoom;
    const x = pianoWidth + playbackPosition / ticksPerPixel - scrollX;

    ctx.strokeStyle = 'rgba(255, 0, 0, 0.8)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, canvas.height);
    ctx.stroke();

    // Playhead triangle
    ctx.fillStyle = 'rgba(255, 0, 0, 0.8)';
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x - 6, 10);
    ctx.lineTo(x + 6, 10);
    ctx.closePath();
    ctx.fill();
  }

  function drawTimeRuler() {
    if (!rulerCtx || !$selectedPattern) return;

    const width = timeRulerCanvas.width;
    const height = rulerHeight;

    // Clear
    rulerCtx.fillStyle = '#0a0a0a';
    rulerCtx.fillRect(0, 0, width, height);

    const beatWidth = pixelsPerBeat * zoom;
    const barWidth = beatWidth * 4;
    const ticksPerBeat = $selectedPattern.ticks_per_beat;

    // Draw bars and beats
    let barNum = 1;
    for (let x = pianoWidth; x < width + scrollX; x += barWidth) {
      const drawX = x - scrollX;

      // Bar number
      rulerCtx.fillStyle = '#fff';
      rulerCtx.font = 'bold 12px monospace';
      rulerCtx.fillText(`${barNum}`, drawX + 4, 16);

      // Bar line
      rulerCtx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
      rulerCtx.lineWidth = 2;
      rulerCtx.beginPath();
      rulerCtx.moveTo(drawX, 20);
      rulerCtx.lineTo(drawX, height);
      rulerCtx.stroke();

      // Beat marks
      for (let beat = 1; beat < 4; beat++) {
        const beatX = drawX + beat * beatWidth;
        rulerCtx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
        rulerCtx.lineWidth = 1;
        rulerCtx.beginPath();
        rulerCtx.moveTo(beatX, 25);
        rulerCtx.lineTo(beatX, height);
        rulerCtx.stroke();

        // Beat number
        rulerCtx.fillStyle = 'rgba(255, 255, 255, 0.6)';
        rulerCtx.font = '10px monospace';
        rulerCtx.fillText(`${beat + 1}`, beatX + 2, 32);
      }

      barNum++;
    }

    // Draw playhead in ruler
    if (playbackPosition > 0) {
      const ticksPerPixel = ticksPerBeat / pixelsPerBeat / zoom;
      const x = pianoWidth + playbackPosition / ticksPerPixel - scrollX;

      rulerCtx.strokeStyle = 'rgba(255, 0, 0, 0.8)';
      rulerCtx.lineWidth = 2;
      rulerCtx.beginPath();
      rulerCtx.moveTo(x, 0);
      rulerCtx.lineTo(x, height);
      rulerCtx.stroke();
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();

    if (e.ctrlKey || e.metaKey) {
      // Zoom
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      zoom = Math.max(0.5, Math.min(4, zoom * delta));
      cacheNotes();
    } else if (e.shiftKey) {
      // Horizontal scroll
      scrollX = Math.max(0, scrollX + e.deltaY);
    } else {
      // Vertical scroll
      scrollY = Math.max(0, scrollY + e.deltaY);
    }

    drawPianoRoll();
    drawTimeRuler();
  }

  function handleMouseDown(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Check if clicking on piano keys
    if (x < pianoWidth) {
      return;
    }

    // Find clicked note
    const clickedNoteIndex = findNoteAt(x + scrollX, y + scrollY);

    if (currentTool === 'select') {
      if (clickedNoteIndex !== null) {
        selectedNoteIndex = clickedNoteIndex;

        // Check if clicking on resize handles
        const note = cachedNotes[clickedNoteIndex];
        const noteX = note.x - scrollX;
        const noteEndX = noteX + note.width;

        if (Math.abs(x - noteX) < 6) {
          isResizing = true;
          resizeHandle = 'start';
        } else if (Math.abs(x - noteEndX) < 6) {
          isResizing = true;
          resizeHandle = 'end';
        } else {
          isDragging = true;
          draggedNote = note.event;
        }

        dragStartX = x;
        dragStartY = y;
      } else {
        selectedNoteIndex = null;
      }
    } else if (currentTool === 'draw') {
      // Add new note
      addNote(x + scrollX, y + scrollY);
    } else if (currentTool === 'erase') {
      // Delete note
      if (clickedNoteIndex !== null) {
        deleteNote(clickedNoteIndex);
      }
    }

    drawPianoRoll();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging && !isResizing) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (isDragging && selectedNoteIndex !== null && draggedNote) {
      const deltaX = x - dragStartX;
      const deltaY = y - dragStartY;

      const note = cachedNotes[selectedNoteIndex];

      // Update cached position
      note.x += deltaX;
      note.y += deltaY;

      // Snap to grid
      if (snapToGrid > 0) {
        const snapPixels = (pixelsPerBeat * zoom) / (snapToGrid / 4);
        note.x = Math.round(note.x / snapPixels) * snapPixels;
      }
      note.y = Math.round(note.y / noteHeight) * noteHeight;

      dragStartX = x;
      dragStartY = y;

      drawPianoRoll();
    } else if (isResizing && selectedNoteIndex !== null) {
      const deltaX = x - dragStartX;
      const note = cachedNotes[selectedNoteIndex];

      if (resizeHandle === 'start') {
        note.x += deltaX;
        note.width -= deltaX;
      } else if (resizeHandle === 'end') {
        note.width += deltaX;
      }

      note.width = Math.max(note.width, minNoteWidth);

      dragStartX = x;
      drawPianoRoll();
    }
  }

  function handleMouseUp() {
    if ((isDragging || isResizing) && selectedNoteIndex !== null) {
      const note = cachedNotes[selectedNoteIndex];

      // Calculate new tick and note values
      const ticksPerPixel = $selectedPattern!.ticks_per_beat / pixelsPerBeat / zoom;
      const newTick = Math.round((note.x - pianoWidth) * ticksPerPixel);
      const newNote = 127 - Math.floor(note.y / noteHeight);
      const newDuration = Math.round(note.width * ticksPerPixel);

      // Emit note-changed event
      dispatch('note-changed', {
        oldNote: note.event,
        newTick,
        newNote,
        newDuration,
      });
    }

    isDragging = false;
    isResizing = false;
    resizeHandle = null;
    draggedNote = null;
  }

  function findNoteAt(x: number, y: number): number | null {
    for (let i = cachedNotes.length - 1; i >= 0; i--) {
      const note = cachedNotes[i];
      if (x >= note.x && x <= note.x + note.width && y >= note.y && y <= note.y + noteHeight) {
        return i;
      }
    }
    return null;
  }

  function addNote(x: number, y: number) {
    if (!$selectedPattern) return;

    const ticksPerPixel = $selectedPattern.ticks_per_beat / pixelsPerBeat / zoom;
    const tick = Math.round((x - pianoWidth) * ticksPerPixel);
    const note = 127 - Math.floor(y / noteHeight);

    // Snap to grid
    let snappedTick = tick;
    if (snapToGrid > 0) {
      const snapTicks = $selectedPattern.ticks_per_beat / (snapToGrid / 4);
      snappedTick = Math.round(tick / snapTicks) * snapTicks;
    }

    const defaultDuration = $selectedPattern.ticks_per_beat / 4; // Quarter note

    dispatch('note-added', {
      tick: snappedTick,
      note,
      velocity: 100,
      duration: defaultDuration,
    });

    // Refresh pattern
    loadPattern();
  }

  function deleteNote(index: number) {
    const note = cachedNotes[index];
    dispatch('note-deleted', { note: note.event });

    // Refresh pattern
    loadPattern();
  }

  function getSelectedNoteInfo(): string {
    if (selectedNoteIndex === null) return '';

    const note = cachedNotes[selectedNoteIndex];
    const noteNum = note.event.note || 0;
    const octave = Math.floor(noteNum / 12) - 1;
    const noteName = noteNames[noteNum % 12];
    const velocity = note.event.velocity || 0;

    const ticksPerPixel = $selectedPattern!.ticks_per_beat / pixelsPerBeat / zoom;
    const durationTicks = note.noteOff ? note.noteOff.tick - note.event.tick : 0;
    const durationBeats = (durationTicks / $selectedPattern!.ticks_per_beat).toFixed(2);

    return `${noteName}${octave} | Vel: ${velocity} | Dur: ${durationBeats} beats`;
  }

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      canvas.width = canvas.offsetWidth;
      canvas.height = canvas.offsetHeight;
    }

    if (timeRulerCanvas) {
      rulerCtx = timeRulerCanvas.getContext('2d');
      timeRulerCanvas.width = timeRulerCanvas.offsetWidth;
      timeRulerCanvas.height = rulerHeight;
    }

    if ($selectedFile) {
      loadPattern();
    }

    // Redraw on window resize
    const handleResize = () => {
      if (canvas) {
        canvas.width = canvas.offsetWidth;
        canvas.height = canvas.offsetHeight;
        drawPianoRoll();
      }
      if (timeRulerCanvas) {
        timeRulerCanvas.width = timeRulerCanvas.offsetWidth;
        timeRulerCanvas.height = rulerHeight;
        drawTimeRuler();
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  });
</script>

<div class="piano-roll-container">
  {#if $selectedFile}
    <div class="piano-roll-header">
      <h2>{$selectedFile.file_name}</h2>
      <div class="controls">
        <!-- Tool Selector -->
        <div class="tool-group">
          <button
            class:active={currentTool === 'select'}
            on:click={() => (currentTool = 'select')}
            title="Select Tool (V)"
          >
            ✋
          </button>
          <button
            class:active={currentTool === 'draw'}
            on:click={() => (currentTool = 'draw')}
            title="Draw Tool (D)"
          >
            ✏️
          </button>
          <button
            class:active={currentTool === 'erase'}
            on:click={() => (currentTool = 'erase')}
            title="Erase Tool (E)"
          >
            🗑️
          </button>
        </div>

        <!-- Snap Controls -->
        <div class="snap-group">
          <label>Snap:</label>
          <button class:active={snapToGrid === 0} on:click={() => (snapToGrid = 0)}> Off </button>
          <button class:active={snapToGrid === 4} on:click={() => (snapToGrid = 4)}> 1/4 </button>
          <button class:active={snapToGrid === 8} on:click={() => (snapToGrid = 8)}> 1/8 </button>
          <button class:active={snapToGrid === 16} on:click={() => (snapToGrid = 16)}>
            1/16
          </button>
        </div>

        <!-- Zoom Controls -->
        <div class="zoom-group">
          <button
            on:click={() => {
              zoom = Math.max(0.5, zoom * 0.9);
              cacheNotes();
              drawPianoRoll();
              drawTimeRuler();
            }}
          >
            🔍-
          </button>
          <span>{(zoom * 100).toFixed(0)}%</span>
          <button
            on:click={() => {
              zoom = Math.min(4, zoom * 1.1);
              cacheNotes();
              drawPianoRoll();
              drawTimeRuler();
            }}
          >
            🔍+
          </button>
        </div>

        <button
          on:click={() => {
            zoom = 1;
            scrollX = 0;
            scrollY = 0;
            cacheNotes();
            drawPianoRoll();
            drawTimeRuler();
          }}
        >
          Reset
        </button>
      </div>
    </div>

    <!-- Time Ruler -->
    <div class="time-ruler-container">
      <canvas
        bind:this={timeRulerCanvas}
        class="time-ruler-canvas"
        style="height: {rulerHeight}px"
      />
    </div>

    <!-- Piano Roll Canvas -->
    <canvas
      bind:this={canvas}
      on:wheel={handleWheel}
      on:mousedown={handleMouseDown}
      on:mousemove={handleMouseMove}
      on:mouseup={handleMouseUp}
      on:mouseleave={handleMouseUp}
      class="piano-roll-canvas"
      class:select-cursor={currentTool === 'select'}
      class:draw-cursor={currentTool === 'draw'}
      class:erase-cursor={currentTool === 'erase'}
    />

    <!-- Status Bar -->
    <div class="piano-roll-status">
      <div class="status-left">
        <span>💡 Scroll: Move | Shift+Scroll: Horizontal | Ctrl+Scroll: Zoom</span>
      </div>
      <div class="status-right">
        {#if selectedNoteIndex !== null}
          <span class="note-info">🎵 {getSelectedNoteInfo()}</span>
        {:else}
          <span class="note-info">No note selected</span>
        {/if}
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <p>🎹 Select a file to view its piano roll</p>
    </div>
  {/if}
</div>

<style>
  .piano-roll-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 8px;
    overflow: hidden;
  }

  .piano-roll-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.5);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .piano-roll-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: #fff;
  }

  .controls {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .tool-group,
  .snap-group,
  .zoom-group {
    display: flex;
    gap: 0.25rem;
    align-items: center;
  }

  .snap-group label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
    margin-right: 0.25rem;
  }

  .controls button {
    padding: 0.4rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .controls button:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .controls button.active {
    background: rgba(100, 200, 255, 0.3);
    border-color: rgba(100, 200, 255, 0.5);
  }

  .zoom-group span {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
    min-width: 50px;
    text-align: center;
  }

  .time-ruler-container {
    background: #0a0a0a;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .time-ruler-canvas {
    width: 100%;
    display: block;
  }

  .piano-roll-canvas {
    flex: 1;
    width: 100%;
    display: block;
  }

  .piano-roll-canvas.select-cursor {
    cursor: default;
  }

  .piano-roll-canvas.draw-cursor {
    cursor: crosshair;
  }

  .piano-roll-canvas.erase-cursor {
    cursor: not-allowed;
  }

  .piano-roll-status {
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

  .note-info {
    color: rgba(100, 200, 255, 0.9);
    font-weight: 500;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.5);
  }

  .empty-state p {
    font-size: 1.125rem;
  }
</style>
