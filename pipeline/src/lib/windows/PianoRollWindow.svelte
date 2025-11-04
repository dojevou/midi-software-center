<script lang="ts">
  /**
   * PianoRollWindow - MIDI Piano Roll Editor
   *
   * Task-O-Matic: Visual MIDI note editor with timeline, keyboard, velocity editing.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    playbackStore,
    playbackActions,
    type Position
  } from '$lib/stores/playbackStore';
  import {
    projectStore,
    selectedTrack
  } from '$lib/stores/projectStore';

  // ============================================================================
  // Types
  // ============================================================================

  interface Note {
    id: number;
    pitch: number; // MIDI note number (0-127)
    startTick: number;
    endTick: number;
    velocity: number; // 0-127
    selected: boolean;
  }

  interface GridSettings {
    snapEnabled: boolean;
    snapDivision: number; // 1, 2, 4, 8, 16, 32
    zoomH: number; // Horizontal zoom (0.1 to 10)
    zoomV: number; // Vertical zoom (0.5 to 2)
  }

  // ============================================================================
  // Constants
  // ============================================================================

  const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const OCTAVES = Array.from({ length: 11 }, (_, i) => i - 1); // C-1 to C9
  const TOTAL_NOTES = OCTAVES.length * 12;
  const NOTE_HEIGHT = 20;
  const PIANO_KEY_WIDTH = 60;
  const TICKS_PER_BEAT = 480;
  const PIXELS_PER_BEAT = 100;

  // ============================================================================
  // State
  // ============================================================================

  // Reactive state from stores
  $: position = $playbackStore.position;
  $: isPlaying = $playbackStore.isPlaying;
  $: tempo = $playbackStore.tempo;
  $: currentTrack = $selectedTrack;

  // Notes data
  let notes: Note[] = [];
  let selectedNotes: Set<number> = new Set();
  let noteIdCounter = 1;

  // Grid settings
  let gridSettings: GridSettings = {
    snapEnabled: true,
    snapDivision: 16,
    zoomH: 1.0,
    zoomV: 1.0
  };

  // UI state
  let scrollX = 0;
  let scrollY = 0;
  let showVelocity = true;
  let isDragging = false;
  let isSelecting = false;
  let selectionBox: { x: number; y: number; width: number; height: number } | null = null;
  let dragStartX = 0;
  let dragStartY = 0;

  // Canvas refs
  let noteGridContainer: HTMLElement;
  let velocityContainer: HTMLElement;

  // ============================================================================
  // Helper Functions
  // ============================================================================

  function pitchToNoteName(pitch: number): string {
    const octave = Math.floor(pitch / 12) - 1;
    const noteIndex = pitch % 12;
    return `${NOTE_NAMES[noteIndex]}${octave}`;
  }

  function isBlackKey(pitch: number): boolean {
    const noteIndex = pitch % 12;
    return [1, 3, 6, 8, 10].includes(noteIndex);
  }

  function ticksToX(ticks: number): number {
    const beats = ticks / TICKS_PER_BEAT;
    return beats * PIXELS_PER_BEAT * gridSettings.zoomH;
  }

  function xToTicks(x: number): number {
    const beats = x / (PIXELS_PER_BEAT * gridSettings.zoomH);
    return Math.floor(beats * TICKS_PER_BEAT);
  }

  function pitchToY(pitch: number): number {
    return (TOTAL_NOTES - pitch - 1) * NOTE_HEIGHT * gridSettings.zoomV;
  }

  function yToPitch(y: number): number {
    const pitch = TOTAL_NOTES - Math.floor(y / (NOTE_HEIGHT * gridSettings.zoomV)) - 1;
    return Math.max(0, Math.min(127, pitch));
  }

  function snapToGrid(ticks: number): number {
    if (!gridSettings.snapEnabled) return ticks;
    const division = TICKS_PER_BEAT / gridSettings.snapDivision;
    return Math.round(ticks / division) * division;
  }

  function velocityToDb(velocity: number): string {
    if (velocity === 0) return '-∞';
    const normalized = velocity / 127;
    const db = 20 * Math.log10(normalized);
    return db.toFixed(1);
  }

  // ============================================================================
  // Note Management
  // ============================================================================

  function addNote(pitch: number, startTick: number, duration: number = TICKS_PER_BEAT): void {
    const snappedStart = snapToGrid(startTick);
    const snappedDuration = snapToGrid(duration);

    const note: Note = {
      id: noteIdCounter++,
      pitch,
      startTick: snappedStart,
      endTick: snappedStart + snappedDuration,
      velocity: 100,
      selected: false
    };

    notes = [...notes, note];
  }

  function removeNote(noteId: number): void {
    notes = notes.filter(n => n.id !== noteId);
    selectedNotes.delete(noteId);
  }

  function removeSelectedNotes(): void {
    notes = notes.filter(n => !selectedNotes.has(n.id));
    selectedNotes.clear();
  }

  function selectNote(noteId: number, addToSelection: boolean = false): void {
    if (!addToSelection) {
      selectedNotes.clear();
      notes = notes.map(n => ({ ...n, selected: false }));
    }

    selectedNotes.add(noteId);
    notes = notes.map(n => n.id === noteId ? { ...n, selected: true } : n);
  }

  function selectAll(): void {
    selectedNotes.clear();
    notes = notes.map(n => {
      selectedNotes.add(n.id);
      return { ...n, selected: true };
    });
  }

  function invertSelection(): void {
    const newSelected = new Set<number>();
    notes = notes.map(n => {
      const newState = !selectedNotes.has(n.id);
      if (newState) newSelected.add(n.id);
      return { ...n, selected: newState };
    });
    selectedNotes = newSelected;
  }

  function updateNoteVelocity(noteId: number, velocity: number): void {
    const clampedVelocity = Math.max(0, Math.min(127, velocity));
    notes = notes.map(n => n.id === noteId ? { ...n, velocity: clampedVelocity } : n);
  }

  function moveSelectedNotes(deltaPitch: number, deltaTicks: number): void {
    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;

      const newPitch = Math.max(0, Math.min(127, n.pitch + deltaPitch));
      const newStart = Math.max(0, n.startTick + deltaTicks);
      const duration = n.endTick - n.startTick;

      return {
        ...n,
        pitch: newPitch,
        startTick: snapToGrid(newStart),
        endTick: snapToGrid(newStart) + duration
      };
    });
  }

  // ============================================================================
  // Actions
  // ============================================================================

  function handleQuantize(): void {
    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;

      const duration = n.endTick - n.startTick;
      const snappedStart = snapToGrid(n.startTick);

      return {
        ...n,
        startTick: snappedStart,
        endTick: snappedStart + duration
      };
    });
  }

  function handleTranspose(semitones: number): void {
    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;
      const newPitch = Math.max(0, Math.min(127, n.pitch + semitones));
      return { ...n, pitch: newPitch };
    });
  }

  function handleZoomH(delta: number): void {
    gridSettings.zoomH = Math.max(0.1, Math.min(10, gridSettings.zoomH + delta));
  }

  function handleZoomV(delta: number): void {
    gridSettings.zoomV = Math.max(0.5, Math.min(2, gridSettings.zoomV + delta));
  }

  function handleGridClick(e: MouseEvent): void {
    if (e.button !== 0) return; // Left click only

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left + scrollX;
    const y = e.clientY - rect.top + scrollY;

    const pitch = yToPitch(y);
    const ticks = xToTicks(x);

    // Check if clicking on existing note
    const clickedNote = notes.find(n =>
      n.pitch === pitch &&
      ticks >= ticksToX(n.startTick) &&
      ticks <= ticksToX(n.endTick)
    );

    if (clickedNote) {
      selectNote(clickedNote.id, e.shiftKey);
    } else {
      // Add new note
      if (!e.shiftKey) {
        selectedNotes.clear();
        notes = notes.map(n => ({ ...n, selected: false }));
      }
      addNote(pitch, ticks);
    }
  }

  function handleScroll(e: WheelEvent): void {
    if (e.ctrlKey) {
      e.preventDefault();
      if (e.deltaY < 0) {
        handleZoomH(0.1);
      } else {
        handleZoomH(-0.1);
      }
    } else {
      scrollY += e.deltaY;
      scrollX += e.deltaX;
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.code === 'Delete' || e.code === 'Backspace') {
      e.preventDefault();
      removeSelectedNotes();
    } else if (e.code === 'KeyA' && e.ctrlKey) {
      e.preventDefault();
      selectAll();
    } else if (e.code === 'KeyI' && e.ctrlKey) {
      e.preventDefault();
      invertSelection();
    } else if (e.code === 'KeyV') {
      e.preventDefault();
      showVelocity = !showVelocity;
    } else if (e.code === 'ArrowUp') {
      e.preventDefault();
      moveSelectedNotes(1, 0);
    } else if (e.code === 'ArrowDown') {
      e.preventDefault();
      moveSelectedNotes(-1, 0);
    } else if (e.code === 'ArrowLeft') {
      e.preventDefault();
      const snapAmount = TICKS_PER_BEAT / gridSettings.snapDivision;
      moveSelectedNotes(0, -snapAmount);
    } else if (e.code === 'ArrowRight') {
      e.preventDefault();
      const snapAmount = TICKS_PER_BEAT / gridSettings.snapDivision;
      moveSelectedNotes(0, snapAmount);
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);

    // Load demo notes
    addNote(60, 0, TICKS_PER_BEAT);
    addNote(64, TICKS_PER_BEAT, TICKS_PER_BEAT);
    addNote(67, TICKS_PER_BEAT * 2, TICKS_PER_BEAT);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<WindowBase windowId="pianoroll" title="Piano Roll">
  <div class="pianoroll-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-section">
        <button
          class="tool-btn"
          class:active={gridSettings.snapEnabled}
          on:click={() => gridSettings.snapEnabled = !gridSettings.snapEnabled}
          title="Grid Snapping (G)"
        >
          Grid
        </button>

        <select
          class="snap-select"
          bind:value={gridSettings.snapDivision}
          disabled={!gridSettings.snapEnabled}
        >
          <option value={1}>1/1</option>
          <option value={2}>1/2</option>
          <option value={4}>1/4</option>
          <option value={8}>1/8</option>
          <option value={16}>1/16</option>
          <option value={32}>1/32</option>
        </select>
      </div>

      <div class="toolbar-section">
        <label class="zoom-label">
          Zoom H:
          <input
            type="range"
            min="0.1"
            max="10"
            step="0.1"
            bind:value={gridSettings.zoomH}
            class="zoom-slider"
          />
          <span class="zoom-value">{gridSettings.zoomH.toFixed(1)}x</span>
        </label>

        <label class="zoom-label">
          Zoom V:
          <input
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            bind:value={gridSettings.zoomV}
            class="zoom-slider"
          />
          <span class="zoom-value">{gridSettings.zoomV.toFixed(1)}x</span>
        </label>
      </div>

      <div class="toolbar-section">
        <button class="action-btn" on:click={handleQuantize} title="Quantize selected notes">
          Quantize
        </button>
        <button class="action-btn" on:click={() => handleTranspose(12)} title="Transpose up 1 octave">
          +Oct
        </button>
        <button class="action-btn" on:click={() => handleTranspose(-12)} title="Transpose down 1 octave">
          -Oct
        </button>
      </div>

      <div class="toolbar-section">
        <button class="action-btn" on:click={selectAll} title="Select All (Ctrl+A)">
          Select All
        </button>
        <button class="action-btn" on:click={invertSelection} title="Invert Selection (Ctrl+I)">
          Invert
        </button>
        <button
          class="action-btn"
          class:active={showVelocity}
          on:click={() => showVelocity = !showVelocity}
          title="Show/Hide Velocity (V)"
        >
          Velocity
        </button>
      </div>

      <div class="toolbar-section info">
        <span class="info-text">Selected: {selectedNotes.size}</span>
        <span class="info-text">Total: {notes.length}</span>
        <span class="info-text">Track: {currentTrack?.label || 'None'}</span>
      </div>
    </div>

    <!-- Main Content -->
    <div class="content-area">
      <!-- Timeline -->
      <div class="timeline-header">
        <div class="timeline-spacer" style="width: {PIANO_KEY_WIDTH}px;" />
        <div class="timeline" on:wheel={handleScroll}>
          <div class="timeline-markers" style="transform: scaleX({gridSettings.zoomH})">
            {#each Array(64) as _, bar}
              <div class="bar-marker" style="min-width: {PIXELS_PER_BEAT * 4}px">
                <span class="bar-number">{bar + 1}</span>
              </div>
            {/each}
          </div>

          <!-- Playhead in timeline -->
          {#if position}
            <div
              class="playhead-timeline"
              style="left: {ticksToX(position.totalTicks)}px"
            />
          {/if}
        </div>
      </div>

      <!-- Piano Roll Grid -->
      <div class="grid-container">
        <!-- Piano Keyboard -->
        <div class="piano-keyboard">
          {#each Array(TOTAL_NOTES) as _, i}
            {@const pitch = TOTAL_NOTES - i - 1}
            {@const isBlack = isBlackKey(pitch)}
            <div
              class="piano-key"
              class:black={isBlack}
              style="height: {NOTE_HEIGHT * gridSettings.zoomV}px"
            >
              <span class="note-name">{pitchToNoteName(pitch)}</span>
            </div>
          {/each}
        </div>

        <!-- Note Grid -->
        <div
          class="note-grid"
          bind:this={noteGridContainer}
          on:click={handleGridClick}
          on:wheel={handleScroll}
          role="grid"
          tabindex="0"
        >
          <!-- Grid lines -->
          <div class="grid-lines">
            {#each Array(TOTAL_NOTES) as _, i}
              {@const isBlack = isBlackKey(TOTAL_NOTES - i - 1)}
              <div
                class="grid-row"
                class:black-key-row={isBlack}
                style="height: {NOTE_HEIGHT * gridSettings.zoomV}px"
              />
            {/each}
          </div>

          <!-- Beat markers -->
          <div class="beat-markers" style="transform: scaleX({gridSettings.zoomH})">
            {#each Array(256) as _, beat}
              <div
                class="beat-line"
                class:bar-line={beat % 4 === 0}
                style="left: {beat * PIXELS_PER_BEAT}px"
              />
            {/each}
          </div>

          <!-- Notes -->
          <div class="notes-layer">
            {#each notes as note (note.id)}
              {@const x = ticksToX(note.startTick)}
              {@const y = pitchToY(note.pitch)}
              {@const width = ticksToX(note.endTick - note.startTick)}
              <div
                class="note"
                class:selected={note.selected}
                style="
                  left: {x}px;
                  top: {y}px;
                  width: {width}px;
                  height: {NOTE_HEIGHT * gridSettings.zoomV}px;
                  opacity: {note.velocity / 127};
                "
                on:click|stopPropagation={() => selectNote(note.id, false)}
                on:contextmenu|preventDefault={() => removeNote(note.id)}
                role="gridcell"
                tabindex="0"
              >
                <div class="note-content">
                  <span class="note-vel">{note.velocity}</span>
                </div>
              </div>
            {/each}
          </div>

          <!-- Playhead -->
          {#if position}
            <div
              class="playhead"
              style="left: {ticksToX(position.totalTicks)}px"
            />
          {/if}
        </div>
      </div>

      <!-- Velocity Editor -->
      {#if showVelocity}
        <div class="velocity-editor" bind:this={velocityContainer}>
          <div class="velocity-spacer" style="width: {PIANO_KEY_WIDTH}px;">
            <span class="velocity-label">Velocity</span>
          </div>

          <div class="velocity-lanes">
            {#each notes as note (note.id)}
              {@const x = ticksToX(note.startTick)}
              {@const width = ticksToX(note.endTick - note.startTick)}
              {@const height = (note.velocity / 127) * 80}

              <div
                class="velocity-strip"
                class:selected={note.selected}
                style="
                  left: {x}px;
                  width: {width}px;
                  height: {height}px;
                "
              >
                <div class="velocity-db">{velocityToDb(note.velocity)} dB</div>
              </div>
            {/each}

            <!-- Playhead in velocity -->
            {#if position}
              <div
                class="playhead-velocity"
                style="left: {ticksToX(position.totalTicks)}px"
              />
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">Snap: {gridSettings.snapEnabled ? `1/${gridSettings.snapDivision}` : 'Off'}</span>
      <span class="status-item">Zoom: {gridSettings.zoomH.toFixed(1)}x / {gridSettings.zoomV.toFixed(1)}x</span>
      <span class="status-item">Position: {position.bar}.{position.beat}.{position.tick}</span>
      <span class="status-item">Tempo: {tempo} BPM</span>
    </div>
  </div>
</WindowBase>

<style>
  .pianoroll-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
    overflow: hidden;
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 12px;
    background: var(--toolbar-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    flex-wrap: wrap;
  }

  .toolbar-section {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-right: 16px;
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .toolbar-section:last-child {
    border-right: none;
  }

  .toolbar-section.info {
    margin-left: auto;
  }

  .tool-btn,
  .action-btn {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s ease;
  }

  .tool-btn:hover,
  .action-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .tool-btn.active,
  .action-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .snap-select {
    padding: 6px 8px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .snap-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .zoom-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .zoom-slider {
    width: 80px;
    height: 4px;
    -webkit-appearance: none;
    background: var(--slider-bg, #3a3a3a);
    border-radius: 2px;
    outline: none;
  }

  .zoom-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
  }

  .zoom-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .zoom-value {
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    min-width: 35px;
    color: var(--text-primary, #e0e0e0);
  }

  .info-text {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  /* Content Area */
  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Timeline */
  .timeline-header {
    display: flex;
    height: 40px;
    background: var(--timeline-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .timeline-spacer {
    flex-shrink: 0;
    background: var(--piano-bg, #2a2a2a);
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .timeline {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .timeline-markers {
    display: flex;
    height: 100%;
    transform-origin: left;
  }

  .bar-marker {
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

  .playhead-timeline {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #e81123;
    pointer-events: none;
    z-index: 10;
  }

  /* Grid Container */
  .grid-container {
    flex: 1;
    display: flex;
    overflow: auto;
  }

  /* Piano Keyboard */
  .piano-keyboard {
    width: 60px;
    flex-shrink: 0;
    background: var(--piano-bg, #2a2a2a);
    border-right: 1px solid var(--border-color, #3e3e3e);
    overflow: hidden;
  }

  .piano-key {
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    border-bottom: 1px solid #ccc;
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .piano-key:hover {
    background: #f0f0f0;
  }

  .piano-key.black {
    background: #2a2a2a;
    color: white;
  }

  .piano-key.black:hover {
    background: #3a3a3a;
  }

  .note-name {
    font-size: 11px;
    font-weight: 500;
  }

  /* Note Grid */
  .note-grid {
    flex: 1;
    position: relative;
    overflow: auto;
    cursor: crosshair;
    background: var(--grid-bg, #1a1a1a);
  }

  .grid-lines {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }

  .grid-row {
    border-bottom: 1px solid var(--grid-line, #2a2a2a);
  }

  .grid-row.black-key-row {
    background: rgba(255, 255, 255, 0.02);
  }

  .beat-markers {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    transform-origin: left;
  }

  .beat-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--beat-line, #333);
    pointer-events: none;
  }

  .beat-line.bar-line {
    background: var(--bar-line, #555);
    width: 2px;
  }

  .notes-layer {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .note {
    position: absolute;
    background: var(--note-color, #0078d4);
    border: 1px solid var(--note-border, #005a9e);
    border-radius: 3px;
    cursor: move;
    transition: background 0.15s ease;
  }

  .note:hover {
    background: var(--note-hover, #0084e8);
  }

  .note.selected {
    background: var(--note-selected, #f7dc6f);
    border-color: var(--note-selected-border, #f4d03f);
  }

  .note-content {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 0 4px;
  }

  .note-vel {
    font-size: 10px;
    font-weight: 600;
    color: rgba(0, 0, 0, 0.6);
  }

  .playhead {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #e81123;
    pointer-events: none;
    z-index: 100;
  }

  .playhead::before {
    content: '';
    position: absolute;
    top: 0;
    left: -4px;
    width: 0;
    height: 0;
    border-left: 4px solid transparent;
    border-right: 4px solid transparent;
    border-top: 8px solid #e81123;
  }

  /* Velocity Editor */
  .velocity-editor {
    height: 100px;
    display: flex;
    background: var(--velocity-bg, #252525);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .velocity-spacer {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--piano-bg, #2a2a2a);
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .velocity-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary, #888);
    transform: rotate(-90deg);
  }

  .velocity-lanes {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .velocity-strip {
    position: absolute;
    bottom: 0;
    background: var(--velocity-color, #0078d4);
    border: 1px solid var(--velocity-border, #005a9e);
    cursor: ns-resize;
    transition: background 0.15s ease;
  }

  .velocity-strip:hover {
    background: var(--velocity-hover, #0084e8);
  }

  .velocity-strip.selected {
    background: var(--velocity-selected, #f7dc6f);
    border-color: var(--velocity-selected-border, #f4d03f);
  }

  .velocity-db {
    position: absolute;
    top: -20px;
    left: 50%;
    transform: translateX(-50%);
    font-size: 10px;
    color: var(--text-secondary, #888);
    white-space: nowrap;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .velocity-strip:hover .velocity-db {
    opacity: 1;
  }

  .playhead-velocity {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    background: #e81123;
    pointer-events: none;
    z-index: 10;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 12px;
    background: var(--status-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .status-item {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Scrollbars */
  .note-grid::-webkit-scrollbar,
  .grid-container::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .note-grid::-webkit-scrollbar-track,
  .grid-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .note-grid::-webkit-scrollbar-thumb,
  .grid-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
