<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  interface Note {
    id: number;
    pitch: number;
    velocity: number;
    startTick: number;
    duration: number;
    channel: number;
  }

  interface LoopRange {
    start: number;
    end: number;
  }

  export let notes: Note[] = [];
  export let gridSize: number = 32;
  export let zoomLevel: number = 1;
  export let playbackPosition: number = 0;
  export let selectedNotes: Set<number> = new Set();
  export let loopRange: LoopRange = { start: 0, end: 1920 };
  export let showVelocity: boolean = false;
  export let snapToGrid: boolean = true;

  const dispatch = createEventDispatcher<{
    noteDrag: { noteId: number; deltaX: number; deltaY: number };
    noteClick: { noteId: number; ctrlKey: boolean; shiftKey: boolean };
    noteResize: { noteId: number; newDuration: number };
    gridClick: { tick: number; pitch: number };
  }>();

  let gridElement: HTMLDivElement;
  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragNoteId: number | null = null;

  const PPQN = 480;
  const NOTE_HEIGHT = 10;
  const TOTAL_PITCHES = 128;

  function ticksToPixels(ticks: number): number {
    return (ticks / PPQN) * 100 * zoomLevel;
  }

  function pixelsToTicks(pixels: number): number {
    return (pixels / (100 * zoomLevel)) * PPQN;
  }

  function pitchToY(pitch: number): number {
    return (127 - pitch) * NOTE_HEIGHT;
  }

  function yToPitch(y: number): number {
    return 127 - Math.floor(y / NOTE_HEIGHT);
  }

  function handleNoteMouseDown(event: MouseEvent, noteId: number) {
    event.stopPropagation();
    isDragging = true;
    dragStartX = event.clientX;
    dragStartY = event.clientY;
    dragNoteId = noteId;

    dispatch('noteClick', {
      noteId,
      ctrlKey: event.ctrlKey,
      shiftKey: event.shiftKey,
    });
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging || dragNoteId === null) {
      return;
    }

    const deltaX = event.clientX - dragStartX;
    const deltaY = event.clientY - dragStartY;

    dispatch('noteDrag', {
      noteId: dragNoteId,
      deltaX: pixelsToTicks(deltaX),
      deltaY,
    });

    dragStartX = event.clientX;
    dragStartY = event.clientY;
  }

  function handleMouseUp() {
    isDragging = false;
    dragNoteId = null;
  }

  function handleGridClick(event: MouseEvent) {
    if (isDragging) {
      return;
    }

    const rect = gridElement.getBoundingClientRect();
    const x = event.clientX - rect.left + gridElement.scrollLeft;
    const y = event.clientY - rect.top + gridElement.scrollTop;

    const tick = pixelsToTicks(x);
    const pitch = yToPitch(y);

    const snappedTick = snapToGrid ? Math.round(tick / gridSize) * gridSize : tick;

    dispatch('gridClick', { tick: snappedTick, pitch });
  }

  function handleNoteResize(event: MouseEvent, noteId: number) {
    event.stopPropagation();
    // Note resize logic would go here
  }

  function getNoteColor(velocity: number): string {
    const intensity = Math.floor((velocity / 127) * 100);
    return `hsl(210, 80%, ${30 + intensity * 0.4}%)`;
  }

  function isNoteSelected(noteId: number): boolean {
    return selectedNotes.has(noteId);
  }
</script>

<div
  bind:this={gridElement}
  class="note-grid relative overflow-auto"
  style="height: {TOTAL_PITCHES * NOTE_HEIGHT}px"
  on:mousemove={handleMouseMove}
  on:mouseup={handleMouseUp}
  on:mouseleave={handleMouseUp}
  on:click={handleGridClick}
  role="grid"
  tabindex="0"
>
  <!-- Grid Background -->
  <div class="grid-background absolute inset-0">
    <!-- Horizontal lines (pitch rows) -->
    {#each Array(TOTAL_PITCHES) as _, pitch (pitch)}
      <div
        class="pitch-row absolute w-full border-b"
        class:dark:border-gray-700={pitch % 12 !== 0}
        class:dark:border-gray-500={pitch % 12 === 0}
        class:dark:bg-gray-800={(127 - pitch) % 12 === 0}
        style="top: {pitch * NOTE_HEIGHT}px; height: {NOTE_HEIGHT}px"
      ></div>
    {/each}

    <!-- Vertical lines (beat markers) -->
    {#each Array(Math.ceil(loopRange.end / PPQN) + 4) as _, beat (beat)}
      <div
        class="beat-line absolute top-0 bottom-0 border-l"
        class:dark:border-gray-600={beat % 4 !== 0}
        class:dark:border-gray-400={beat % 4 === 0}
        style="left: {ticksToPixels(beat * PPQN)}px"
      ></div>
    {/each}
  </div>

  <!-- Notes -->
  {#each notes as note (note.id)}
    <div
      class="note absolute rounded cursor-pointer transition-colors"
      class:ring-2={isNoteSelected(note.id)}
      class:ring-white={isNoteSelected(note.id)}
      style="
        left: {ticksToPixels(note.startTick)}px;
        top: {pitchToY(note.pitch)}px;
        width: {ticksToPixels(note.duration)}px;
        height: {NOTE_HEIGHT - 1}px;
        background-color: {getNoteColor(note.velocity)};
      "
      on:mousedown={(e) => handleNoteMouseDown(e, note.id)}
      role="button"
      tabindex="0"
      aria-label="Note {note.pitch} at tick {note.startTick}"
    >
      <!-- Velocity indicator -->
      {#if showVelocity}
        <div
          class="velocity-bar absolute bottom-0 left-0 right-0 dark:bg-white opacity-30"
          style="height: {(note.velocity / 127) * 100}%"
        ></div>
      {/if}

      <!-- Resize handle -->
      <div
        class="resize-handle absolute right-0 top-0 bottom-0 w-2 cursor-ew-resize opacity-0 hover:opacity-100 dark:bg-white dark:bg-opacity-50"
        on:mousedown|stopPropagation={(e) => handleNoteResize(e, note.id)}
        role="separator"
        aria-orientation="vertical"
      ></div>
    </div>
  {/each}

  <!-- Playback cursor -->
  <div
    class="playback-cursor absolute top-0 bottom-0 w-px dark:bg-red-500 pointer-events-none z-10"
    style="left: {ticksToPixels(playbackPosition)}px"
  ></div>

  <!-- Loop range indicators -->
  <div
    class="loop-start absolute top-0 bottom-0 w-px dark:bg-blue-500 pointer-events-none"
    style="left: {ticksToPixels(loopRange.start)}px"
  ></div>
  <div
    class="loop-end absolute top-0 bottom-0 w-px dark:bg-blue-500 pointer-events-none"
    style="left: {ticksToPixels(loopRange.end)}px"
  ></div>
</div>

<style>
  .note-grid {
    background-color: var(--window-subtle);
  }

  .note:hover {
    filter: brightness(1.2);
  }

  .playback-cursor {
    box-shadow: 0 0 4px rgba(239, 68, 68, 0.5);
  }
</style>
