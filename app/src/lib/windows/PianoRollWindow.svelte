<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { PlaybackPosition } from '$lib/types';
  import NoteGrid from '$lib/components/NoteGrid.svelte';
  import VirtualKeyboard from '$lib/components/VirtualKeyboard.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import { safeListen } from '$lib/utils/tauri';
  import { arrangementStore, arrangementActions, type ActiveEditClip } from '$lib/stores/arrangementStore';

  // Props - can be passed directly or derived from arrangement store
  export let trackId: number | undefined = undefined;
  export let clipId: number | undefined = undefined;
  export let showHeader: boolean = true;

  // Detach state from arrangement store
  $: isDetached = $arrangementStore.pianoRollDetached;
  $: activeClip = $arrangementStore.activeEditClip;

  // Use props if provided, otherwise use active clip from store
  $: effectiveTrackId = trackId ?? activeClip?.trackId ?? 0;
  $: effectiveClipName = activeClip?.clipName ?? 'Untitled Clip';

  // State using Svelte 4 let bindings
  let notes: Note[] = [];
  let currentTool = 'select';
  let gridSize = 32; // 32nd notes
  let zoomLevel = 1;
  let snapToGrid = true;
  const loopRange = { start: 0, end: 1920 }; // 4 bars at 480 PPQN
  let playbackPosition = 0;
  let selectedNotes: Set<number> = new Set();
  let isLoading = false;
  let unlistenPlayback: (() => void) | null = null;
  let unlistenMidi: (() => void) | null = null;

  interface Note {
    id: number;
    pitch: number;
    velocity: number;
    startTick: number;
    duration: number;
    channel: number;
  }

  interface NoteInput {
    pitch: number;
    velocity: number;
    startTick: number;
    duration: number;
    channel: number;
  }

  interface ToolbarItem {
    id: string;
    icon: string;
    tooltip: string;
    action?: () => void;
    separator?: boolean;
  }

  onMount(async () => {
    await loadTrackNotes();

    // Listen for playback position updates (only in Tauri context)
    unlistenPlayback = await safeListen<{ position: number }>(
      'playback-position-update',
      (payload) => {
        playbackPosition = payload.position;
      }
    );

    // Listen for MIDI input (only in Tauri context)
    unlistenMidi = await safeListen<{ type: string; note: number; velocity: number }>(
      'midi-input',
      (payload) => {
        handleMidiInput(payload);
      }
    );
  });

  onDestroy(() => {
    if (unlistenPlayback) {
      unlistenPlayback();
    }
    if (unlistenMidi) {
      unlistenMidi();
    }
  });

  async function loadTrackNotes() {
    isLoading = true;
    try {
      // Backend: daw/src-tauri/src/commands/piano_roll.rs:925
      notes = await api.pianoRoll.getTrackNotes(effectiveTrackId);
    } catch (error) {
      console.error('Failed to load notes:', error);
      notes = [];
    } finally {
      isLoading = false;
    }
  }

  function handleMidiInput(midiEvent: { type: string; note: number; velocity: number }) {
    if (currentTool === 'draw' && midiEvent.type === 'noteon') {
      addNote({
        pitch: midiEvent.note,
        velocity: midiEvent.velocity,
        startTick: calculateCurrentTick(),
        duration: gridSize * 2,
        channel: effectiveTrackId,
      });
    }
  }

  function calculateCurrentTick(): number {
    return playbackPosition;
  }

  async function addNote(noteData: NoteInput) {
    try {
      const newNoteId = await api.pianoRoll.addNote(effectiveTrackId, noteData);
      const newNote: Note = {
        id: newNoteId,
        ...noteData,
      };
      notes = [...notes, newNote];
      selectedNotes = new Set([newNote.id]);
    } catch (error) {
      console.error('Failed to add note:', error);
    }
  }

  async function updateNotes(updatedNotes: Array<{ id: number } & Partial<Note>>) {
    try {
      await api.pianoRoll.updateNotesBatch(updatedNotes);
      updatedNotes.forEach((updated) => {
        const index = notes.findIndex((n) => n.id === updated.id);
        if (index !== -1) {
          notes[index] = { ...notes[index], ...updated };
        }
      });
      notes = [...notes];
    } catch (error) {
      console.error('Failed to update notes:', error);
    }
  }

  async function deleteSelectedNotes() {
    if (selectedNotes.size === 0) {
      return;
    }

    try {
      await api.pianoRoll.deleteNotes(effectiveTrackId, Array.from(selectedNotes));
      notes = notes.filter((note) => !selectedNotes.has(note.id));
      selectedNotes = new Set();
    } catch (error) {
      console.error('Failed to delete notes:', error);
    }
  }

  function handleNoteDrag(noteId: number, deltaX: number, deltaY: number) {
    const note = notes.find((n) => n.id === noteId);
    if (!note) {
      return;
    }

    const gridUnit = 480 / (gridSize / 4);
    const deltaTicks = Math.round(deltaX / gridUnit) * gridUnit;
    const deltaPitch = Math.round(deltaY / 10);

    if (snapToGrid && deltaTicks !== 0) {
      updateNotes([
        {
          id: noteId,
          startTick: note.startTick + deltaTicks,
          pitch: Math.min(127, Math.max(0, note.pitch - deltaPitch)),
        },
      ]);
    }
  }

  function handleRangeSelect(startX: number, startY: number, endX: number, endY: number) {
    const startTick = pixelsToTicks(Math.min(startX, endX));
    const endTick = pixelsToTicks(Math.max(startX, endX));
    const lowPitch = Math.min(startY, endY);
    const highPitch = Math.max(startY, endY);

    const selected = notes.filter(
      (note) =>
        note.startTick >= startTick &&
        note.startTick <= endTick &&
        note.pitch >= lowPitch &&
        note.pitch <= highPitch
    );

    selectedNotes = new Set(selected.map((n) => n.id));
  }

  function pixelsToTicks(pixels: number): number {
    const pixelsPerBeat = 100 * zoomLevel;
    const beats = pixels / pixelsPerBeat;
    return beats * 480;
  }

  function quantizeSelected() {
    const updated: Array<{ id: number } & Partial<Note>> = [];
    selectedNotes.forEach((noteId) => {
      const note = notes.find((n) => n.id === noteId);
      if (note) {
        const quantizedTick = Math.round(note.startTick / gridSize) * gridSize;
        updated.push({ id: noteId, startTick: quantizedTick });
      }
    });
    updateNotes(updated);
  }

  function transposeSelected(semitones: number) {
    const updated: Array<{ id: number } & Partial<Note>> = [];
    selectedNotes.forEach((noteId) => {
      const note = notes.find((n) => n.id === noteId);
      if (note) {
        const newPitch = Math.min(127, Math.max(0, note.pitch + semitones));
        updated.push({ id: noteId, pitch: newPitch });
      }
    });
    updateNotes(updated);
  }

  // ============================================================================
  // SLICE TOOL
  // ============================================================================

  async function sliceNoteAtPosition(noteId: number, sliceTick: number) {
    try {
      const result = await api.pianoRoll.sliceNote(effectiveTrackId, noteId, sliceTick);
      // result is a tuple of [firstNote, secondNote]
      const note = notes.find((n) => n.id === noteId);
      if (note) {
        // Update local state: modify original note and add new note
        notes = notes.map((n) => (n.id === noteId ? { ...n, duration: sliceTick - n.startTick } : n));
        notes = [
          ...notes,
          {
            id: result[1].id,
            pitch: note.pitch,
            velocity: note.velocity,
            startTick: sliceTick,
            duration: note.startTick + note.duration - sliceTick,
            channel: note.channel,
          },
        ];
      }
    } catch (error) {
      console.error('Failed to slice note:', error);
    }
  }

  function handleGridClick(event: MouseEvent) {
    if (currentTool === 'slice') {
      // Find if click is on a note
      const clickTick = pixelsToTicks(event.offsetX);
      const clickPitch = 127 - Math.floor(event.offsetY / 10);

      const targetNote = notes.find(
        (n) =>
          clickTick > n.startTick &&
          clickTick < n.startTick + n.duration &&
          n.pitch === clickPitch
      );

      if (targetNote) {
        const snappedTick = snapToGrid
          ? Math.round(clickTick / gridSize) * gridSize
          : clickTick;
        sliceNoteAtPosition(targetNote.id, snappedTick);
      }
    }
  }

  // ============================================================================
  // STRETCH TOOL
  // ============================================================================

  let stretchRatio = 1.0;
  let stretchAnchor: 'start' | 'end' | 'center' = 'start';
  let showStretchDialog = false;

  async function stretchSelectedNotes() {
    if (selectedNotes.size === 0) return;

    try {
      const noteIds = Array.from(selectedNotes);
      const result = await api.pianoRoll.stretchNotes(effectiveTrackId, noteIds, stretchRatio, stretchAnchor);

      // Update local state with stretched notes
      result.forEach((stretchedNote: Note) => {
        const idx = notes.findIndex((n) => n.id === stretchedNote.id);
        if (idx !== -1) {
          notes[idx] = stretchedNote;
        }
      });
      notes = [...notes];
      showStretchDialog = false;
    } catch (error) {
      console.error('Failed to stretch notes:', error);
    }
  }

  // ============================================================================
  // SCALE QUANTIZE
  // ============================================================================

  type ScaleType = 'Major' | 'Minor' | 'Dorian' | 'Phrygian' | 'Lydian' | 'Mixolydian' | 'Locrian' | 'MelodicMinor' | 'HarmonicMinor' | 'Pentatonic' | 'Blues' | 'Chromatic';

  const scaleTypes: ScaleType[] = [
    'Major', 'Minor', 'Dorian', 'Phrygian', 'Lydian', 'Mixolydian',
    'Locrian', 'MelodicMinor', 'HarmonicMinor', 'Pentatonic', 'Blues', 'Chromatic'
  ];

  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  let selectedScaleRoot = 0; // C
  let selectedScaleType: ScaleType = 'Major';
  let showScaleQuantizeDialog = false;

  async function scaleQuantizeSelected() {
    if (selectedNotes.size === 0) return;

    try {
      const result = await api.pianoRoll.scaleQuantizeNotes(effectiveTrackId, selectedScaleRoot, selectedScaleType);

      // Update local state with quantized notes
      result.forEach((quantizedNote: Note) => {
        const idx = notes.findIndex((n) => n.id === quantizedNote.id);
        if (idx !== -1) {
          notes[idx] = quantizedNote;
        }
      });
      notes = [...notes];
      showScaleQuantizeDialog = false;
    } catch (error) {
      console.error('Failed to scale quantize notes:', error);
    }
  }

  function handleNoteClick(
    event: CustomEvent<{ noteId: number; ctrlKey: boolean; shiftKey: boolean }>
  ) {
    const { noteId, ctrlKey, shiftKey } = event.detail;
    if (ctrlKey) {
      if (selectedNotes.has(noteId)) {
        selectedNotes.delete(noteId);
      } else {
        selectedNotes.add(noteId);
      }
      selectedNotes = new Set(selectedNotes);
    } else if (shiftKey) {
      selectedNotes.add(noteId);
      selectedNotes = new Set(selectedNotes);
    } else {
      selectedNotes = new Set([noteId]);
    }
  }

  function handleNoteResize(event: CustomEvent<{ noteId: number; newDuration: number }>) {
    updateNotes([{ id: event.detail.noteId, duration: event.detail.newDuration }]);
  }

  function handleKeyboardNote(note: number, velocity: number) {
    if (currentTool === 'draw') {
      addNote({
        pitch: note,
        velocity: Math.round(velocity * 127),
        startTick: playbackPosition,
        duration: gridSize * 2,
        channel: effectiveTrackId,
      });
    }
  }

  const toolbarItems: ToolbarItem[] = [
    { id: 'select', icon: 'mouse-pointer', tooltip: 'Select Tool' },
    { id: 'draw', icon: 'edit-3', tooltip: 'Draw Tool' },
    { id: 'erase', icon: 'eraser', tooltip: 'Eraser Tool' },
    { id: 'slice', icon: 'scissors', tooltip: 'Slice Tool (click on note to split)' },
    { id: 'stretch', icon: 'maximize-2', tooltip: 'Stretch Tool' },
    { id: 'separator-1', icon: '', tooltip: '', separator: true },
    { id: 'quantize', icon: 'grid', action: quantizeSelected, tooltip: 'Quantize Selected (Grid)' },
    {
      id: 'scale-quantize',
      icon: 'music',
      action: () => (showScaleQuantizeDialog = true),
      tooltip: 'Scale Quantize (snap to scale)',
    },
    {
      id: 'stretch-dialog',
      icon: 'move-horizontal',
      action: () => (showStretchDialog = true),
      tooltip: 'Stretch Selected Notes',
    },
    { id: 'separator-2', icon: '', tooltip: '', separator: true },
    {
      id: 'transpose-up',
      icon: 'chevron-up',
      action: () => transposeSelected(1),
      tooltip: 'Transpose Up',
    },
    {
      id: 'transpose-down',
      icon: 'chevron-down',
      action: () => transposeSelected(-1),
      tooltip: 'Transpose Down',
    },
    { id: 'delete', icon: 'trash-2', action: deleteSelectedNotes, tooltip: 'Delete Selected' },
  ];

  // Reactive statement for status bar (Svelte 4 pattern)
  $: selectedCount = selectedNotes.size;

  // Reload notes when track changes
  $: if (effectiveTrackId > 0) {
    loadTrackNotes();
  }
</script>

<div class="piano-roll-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header with Clip Name and Detach/Dock -->
  {#if showHeader}
    <div class="piano-roll-header flex items-center justify-between px-3 py-2 dark:bg-menu border-b dark:border-window-border">
      <div class="flex items-center gap-2">
        <span class="icon">🎹</span>
        <span class="clip-name font-medium text-sm">{effectiveClipName}</span>
        {#if effectiveTrackId > 0}
          <span class="track-id text-xs dark:text-gray-500">(Track {effectiveTrackId})</span>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <button
          class="detach-btn px-2 py-1 text-xs rounded dark:bg-secondary hover:dark:bg-button-hover transition-colors flex items-center gap-1"
          on:click={() => arrangementActions.togglePianoRollDetached()}
          title={isDetached ? 'Dock Piano Roll' : 'Detach Piano Roll'}
        >
          <span>{isDetached ? '📌' : '🔲'}</span>
          <span>{isDetached ? 'Dock' : 'Detach'}</span>
        </button>
      </div>
    </div>
  {/if}

  <!-- Toolbar -->
  <Toolbar items={toolbarItems} bind:selectedTool={currentTool} />

  <div class="flex flex-1 overflow-hidden">
    <!-- Piano Keyboard -->
    <div class="keyboard-container w-16 flex-shrink-0 border-r dark:border-window-border">
      <VirtualKeyboard
        octaves={[2, 3, 4, 5, 6]}
        on:noteOn={(e) => handleKeyboardNote(e.detail.note, e.detail.velocity)}
      />
    </div>

    <!-- Main Grid Area -->
    <div class="grid-container flex-1 overflow-auto relative">
      <!-- Time Ruler -->
      <div
        class="time-ruler h-6 dark:bg-window-subtle border-b dark:border-window-border sticky top-0 z-10 flex"
      >
        {#each Array(16) as _, bar (bar)}
          <div
            class="bar-marker text-xs dark:text-gray-400 px-2 border-r dark:border-window-border"
            style="min-width: {100 * zoomLevel}px"
          >
            {bar + 1}
          </div>
        {/each}
      </div>

      <!-- Note Grid -->
      <div class="grid-area relative" style="width: {1920 * zoomLevel}px; height: 800px">
        {#if isLoading}
          <div class="absolute inset-0 flex items-center justify-center dark:text-gray-500">
            Loading notes...
          </div>
        {:else}
          <NoteGrid
            {notes}
            {gridSize}
            {zoomLevel}
            {playbackPosition}
            {selectedNotes}
            {loopRange}
            on:noteDrag={(e) => handleNoteDrag(e.detail.noteId, e.detail.deltaX, e.detail.deltaY)}
            on:noteClick={handleNoteClick}
            on:noteResize={handleNoteResize}
          />
        {/if}

        <!-- Playback Cursor -->
        <div
          class="playback-cursor absolute top-0 bottom-0 w-px bg-red-500 pointer-events-none z-20"
          style="left: {(playbackPosition * zoomLevel) / 4}px"
        ></div>

        <!-- Loop Range -->
        <div
          class="loop-range absolute top-0 bottom-0 bg-blue-500 opacity-10 pointer-events-none"
          style="left: {(loopRange.start * zoomLevel) / 4}px; width: {((loopRange.end -
            loopRange.start) *
            zoomLevel) /
            4}px"
        ></div>
      </div>
    </div>

    <!-- Velocity Editor Sidebar -->
    <div
      class="velocity-editor w-32 flex-shrink-0 border-l dark:border-window-border overflow-y-auto"
    >
      <h3
        class="text-sm dark:text-gray-300 p-2 border-b dark:border-window-border sticky top-0 dark:bg-window"
      >
        Velocities
      </h3>
      <div class="p-2 space-y-2">
        {#each Array.from(selectedNotes) as noteId (noteId)}
          {@const note = notes.find((n) => n.id === noteId)}
          {#if note}
            <div class="velocity-item">
              <div class="text-xs dark:text-gray-400 mb-1">Note {note.pitch}</div>
              <input
                type="range"
                min="1"
                max="127"
                value={note.velocity}
                on:input={(e) =>
                  updateNotes([{ id: note.id, velocity: parseInt(e.currentTarget.value) }])}
                class="w-full"
              />
              <div class="text-xs dark:text-gray-300 text-center">{note.velocity}</div>
            </div>
          {/if}
        {/each}
        {#if selectedNotes.size === 0}
          <div class="text-xs dark:text-gray-500 text-center py-4">
            Select notes to edit velocity
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Status Bar -->
  <div
    class="status-bar h-6 dark:bg-window-subtle border-t dark:border-window-border flex items-center px-2 text-xs dark:text-gray-400"
  >
    <div class="flex-1">
      Selected: {selectedCount} notes | Grid: 1/{gridSize} | Zoom: {Math.round(zoomLevel * 100)}% |
      Snap: {snapToGrid ? 'On' : 'Off'}
    </div>
    <div class="flex gap-2 items-center">
      <button
        on:click={() => (snapToGrid = !snapToGrid)}
        class="px-2 py-0.5 rounded text-xs"
        class:dark:bg-primary={snapToGrid}
        class:dark:bg-secondary={!snapToGrid}
      >
        Snap
      </button>
      <select
        bind:value={gridSize}
        class="text-xs dark:bg-input dark:border-window-border rounded px-1"
      >
        <option value={4}>1/4</option>
        <option value={8}>1/8</option>
        <option value={16}>1/16</option>
        <option value={32}>1/32</option>
        <option value={64}>1/64</option>
      </select>
      <input type="range" min="0.25" max="4" step="0.25" bind:value={zoomLevel} class="w-20" />
    </div>
  </div>
</div>

<!-- Stretch Dialog -->
{#if showStretchDialog}
  <div class="dialog-overlay fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="dialog dark:bg-window dark:border-window-border border rounded-lg p-4 w-80 shadow-xl">
      <h3 class="text-lg font-semibold dark:text-app-text mb-4">Stretch Notes</h3>

      <div class="space-y-4">
        <div>
          <label class="block text-sm dark:text-gray-300 mb-1">Stretch Ratio</label>
          <div class="flex items-center gap-2">
            <input
              type="range"
              min="0.25"
              max="4"
              step="0.25"
              bind:value={stretchRatio}
              class="flex-1"
            />
            <span class="text-sm dark:text-gray-300 w-12 text-right">{stretchRatio}x</span>
          </div>
          <div class="flex gap-2 mt-2">
            <button
              class="px-2 py-1 text-xs dark:bg-secondary rounded"
              on:click={() => (stretchRatio = 0.5)}
            >
              0.5x
            </button>
            <button
              class="px-2 py-1 text-xs dark:bg-secondary rounded"
              on:click={() => (stretchRatio = 1.0)}
            >
              1x
            </button>
            <button
              class="px-2 py-1 text-xs dark:bg-secondary rounded"
              on:click={() => (stretchRatio = 2.0)}
            >
              2x
            </button>
          </div>
        </div>

        <div>
          <label class="block text-sm dark:text-gray-300 mb-1">Anchor Point</label>
          <select
            bind:value={stretchAnchor}
            class="w-full dark:bg-input dark:border-window-border rounded px-2 py-1"
          >
            <option value="start">Start (notes stretch forward)</option>
            <option value="end">End (notes stretch backward)</option>
            <option value="center">Center (notes stretch both ways)</option>
          </select>
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-4">
        <button
          class="px-3 py-1.5 dark:bg-secondary rounded text-sm"
          on:click={() => (showStretchDialog = false)}
        >
          Cancel
        </button>
        <button
          class="px-3 py-1.5 dark:bg-primary rounded text-sm"
          on:click={stretchSelectedNotes}
          disabled={selectedNotes.size === 0}
        >
          Apply Stretch
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Scale Quantize Dialog -->
{#if showScaleQuantizeDialog}
  <div class="dialog-overlay fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="dialog dark:bg-window dark:border-window-border border rounded-lg p-4 w-80 shadow-xl">
      <h3 class="text-lg font-semibold dark:text-app-text mb-4">Scale Quantize</h3>
      <p class="text-sm dark:text-gray-400 mb-4">Snap selected notes to fit a musical scale.</p>

      <div class="space-y-4">
        <div>
          <label class="block text-sm dark:text-gray-300 mb-1">Root Note</label>
          <select
            bind:value={selectedScaleRoot}
            class="w-full dark:bg-input dark:border-window-border rounded px-2 py-1"
          >
            {#each noteNames as name, i}
              <option value={i}>{name}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-sm dark:text-gray-300 mb-1">Scale Type</label>
          <select
            bind:value={selectedScaleType}
            class="w-full dark:bg-input dark:border-window-border rounded px-2 py-1"
          >
            {#each scaleTypes as scale}
              <option value={scale}>{scale}</option>
            {/each}
          </select>
        </div>

        <div class="text-xs dark:text-gray-400 p-2 dark:bg-window-subtle rounded">
          Notes will be moved to the nearest note in the {noteNames[selectedScaleRoot]} {selectedScaleType} scale.
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-4">
        <button
          class="px-3 py-1.5 dark:bg-secondary rounded text-sm"
          on:click={() => (showScaleQuantizeDialog = false)}
        >
          Cancel
        </button>
        <button
          class="px-3 py-1.5 dark:bg-primary rounded text-sm"
          on:click={scaleQuantizeSelected}
          disabled={selectedNotes.size === 0}
        >
          Apply Scale Quantize
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .piano-roll-window {
    min-height: 400px;
  }

  .grid-container {
    scroll-behavior: smooth;
  }

  .keyboard-container {
    background: linear-gradient(to bottom, var(--window-bg), var(--window-subtle));
  }

  .playback-cursor {
    box-shadow: 0 0 4px rgba(239, 68, 68, 0.5);
  }

  .velocity-item input[type='range'] {
    accent-color: var(--primary-color);
  }
</style>
