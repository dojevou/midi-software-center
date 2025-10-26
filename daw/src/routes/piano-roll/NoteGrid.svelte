<script lang="ts">
  import type { MidiNote } from '../../lib/trusty/notes';
  import { createEventDispatcher } from 'svelte';

  export let notes: MidiNote[] = [];
  export let selectedNotes: MidiNote[] = [];
  export let gridDivisor: number = 4;
  export let pixelsPerBeat: number = 100;
  export let beatsPerMeasure: number = 4;

  const dispatch = createEventDispatcher<{
    noteClick: MidiNote;
    gridClick: { time: number; pitch: number };
    selectionStart: { x: number; y: number };
    selectionEnd: { x: number; y: number };
  }>();

  // MIDI pitch range for display
  const MIN_PITCH = 21; // A0
  const MAX_PITCH = 108; // C8
  const TOTAL_PITCHES = MAX_PITCH - MIN_PITCH + 1;

  // Calculate dimensions
  const noteHeight = 20; // pixels per pitch
  const gridHeight = TOTAL_PITCHES * noteHeight;

  // Calculate grid width based on typical content (8 measures)
  const beatsToDisplay = beatsPerMeasure * 8;
  const gridWidth = beatsToDisplay * pixelsPerBeat;

  // Grid line spacing
  const gridUnitPixels = pixelsPerBeat / gridDivisor;
  const measurePixels = beatsPerMeasure * pixelsPerBeat;

  let isSelecting = false;
  let selectionStart: { x: number; y: number } | null = null;

  function handleCanvasMouseDown(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    isSelecting = true;
    selectionStart = { x, y };
    dispatch('selectionStart', { x, y });
  }

  function handleCanvasMouseUp(event: MouseEvent) {
    if (!isSelecting || !selectionStart) return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    isSelecting = false;
    dispatch('selectionEnd', { x, y });
    selectionStart = null;
  }

  function handleCanvasClick(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Convert pixel position to time and pitch (approximate)
    const time = x / pixelsPerBeat;
    const pitch = MAX_PITCH - Math.floor(y / noteHeight);

    dispatch('gridClick', { time, pitch });
  }

  function handleNoteClick(event: MouseEvent, note: MidiNote) {
    event.stopPropagation();
    dispatch('noteClick', note);
  }

  function isNoteSelected(note: MidiNote): boolean {
    return selectedNotes.some((n) => n.id === note.id);
  }

  let canvas: HTMLElement;
</script>

<div class="note-grid-container">
  <div
    class="note-grid"
    bind:this={canvas}
    on:mousedown={handleCanvasMouseDown}
    on:mouseup={handleCanvasMouseUp}
    on:click={handleCanvasClick}
    role="presentation"
  >
    <!-- SVG for grid lines and notes -->
    <svg width={gridWidth} height={gridHeight} class="grid-canvas">
      <!-- Measure lines (vertical, darker) -->
      {#each Array.from({ length: Math.ceil(beatsToDisplay / beatsPerMeasure) + 1 }) as _, measureIndex}
        {@const x = measureIndex * measurePixels}
        {#if x <= gridWidth}
          <line
            x1={x}
            y1="0"
            x2={x}
            y2={gridHeight}
            class="measure-line"
            stroke="#333"
            stroke-width="2"
          />
        {/if}
      {/each}

      <!-- Beat grid lines (vertical, lighter) -->
      {#each Array.from({ length: Math.ceil(beatsToDisplay * gridDivisor) + 1 }) as _, gridIndex}
        {@const x = gridIndex * gridUnitPixels}
        {@const isMeasureLine = gridIndex % (beatsPerMeasure * gridDivisor) === 0}
        {#if !isMeasureLine && x <= gridWidth}
          <line
            x1={x}
            y1="0"
            x2={x}
            y2={gridHeight}
            class="grid-line"
            stroke="#222"
            stroke-width="1"
          />
        {/if}
      {/each}

      <!-- Pitch lines (horizontal) -->
      {#each Array.from({ length: TOTAL_PITCHES + 1 }) as _, pitchIndex}
        {@const y = pitchIndex * noteHeight}
        <line
          x1="0"
          y1={y}
          x2={gridWidth}
          y2={y}
          class="pitch-line"
          stroke="#1a1a1a"
          stroke-width="1"
        />
      {/each}

      <!-- Note rectangles -->
      {#each notes as note (note.id)}
        {@const x = note.startTime * pixelsPerBeat}
        {@const y = (MAX_PITCH - note.pitch) * noteHeight}
        {@const width = note.duration * pixelsPerBeat}
        {@const isSelected = isNoteSelected(note)}

        <g 
          on:click={(e) => handleNoteClick(e, note)}
          role="button" 
          tabindex="0" 
          class="note-group"
        >
          <rect
            {x}
            {y}
            {width}
            height={noteHeight}
            class="note-rect"
            class:selected={isSelected}
            fill={isSelected ? '#90ee90' : '#4a7c59'}
            stroke={isSelected ? '#00aa00' : '#2d5a3d'}
            stroke-width="1"
            rx="2"
          />
          <text
            x={x + width / 2}
            y={y + noteHeight / 2}
            class="note-label"
            text-anchor="middle"
            dominant-baseline="middle"
            font-size="10"
            fill="#000"
            pointer-events="none"
          >
            {note.velocity}
          </text>
        </g>
      {/each}
    </svg>
  </div>
</div>

<style>
  .note-grid-container {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: #0d0d0d;
    border: 1px solid #333;
    border-radius: 4px;
    overflow: hidden;
  }

  .note-grid {
    flex: 1;
    overflow: auto;
    position: relative;
    background-color: #1a1a1a;
    cursor: crosshair;
  }

  .grid-canvas {
    display: block;
    background-color: #1a1a1a;
  }

  :global(.measure-line) {
    stroke: #333 !important;
    opacity: 0.8;
  }

  :global(.grid-line) {
    stroke: #222 !important;
    opacity: 0.6;
  }

  :global(.pitch-line) {
    stroke: #1a1a1a !important;
    opacity: 0.8;
  }

  :global(.note-group) {
    cursor: pointer;
  }

  :global(.note-group:hover .note-rect) {
    filter: brightness(1.2);
  }

  :global(.note-rect.selected) {
    filter: brightness(1.15);
    opacity: 0.9;
  }

  :global(.note-label) {
    font-weight: 600;
    font-size: 10px;
    user-select: none;
    pointer-events: none;
  }
</style>
