<script lang="ts">
  /**
   * VelocityEditorWindow - MIDI Velocity Editor
   *
   * Task-O-Matic: Edit note velocities with faders, humanization, and curve shaping.
   */

  import { onMount } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';

  // ============================================================================
  // Types
  // ============================================================================

  interface NoteVelocity {
    id: number;
    noteName: string;
    position: string; // Bar.Beat.Tick
    velocity: number; // 0-127
    selected: boolean;
  }

  type VelocityCurve = 'linear' | 'exponential' | 'logarithmic';

  // ============================================================================
  // State
  // ============================================================================

  let notes: NoteVelocity[] = [];
  let selectedNotes: Set<number> = new Set();
  let humanizeAmount = 0; // 0-50%
  let velocityCurve: VelocityCurve = 'linear';
  let undoStack: NoteVelocity[][] = [];
  let redoStack: NoteVelocity[][] = [];
  let draggingNoteId: number | null = null;

  // ============================================================================
  // Helper Functions
  // ============================================================================

  function velocityToDb(velocity: number): string {
    if (velocity === 0) return '-∞';
    const normalized = velocity / 127;
    const db = 20 * Math.log10(normalized);
    return db.toFixed(1);
  }

  function getVelocityStats(): { min: number; max: number; avg: number } {
    if (notes.length === 0) {
      return { min: 0, max: 0, avg: 0 };
    }

    const velocities = notes.map(n => n.velocity);
    const min = Math.min(...velocities);
    const max = Math.max(...velocities);
    const avg = Math.round(velocities.reduce((a, b) => a + b, 0) / velocities.length);

    return { min, max, avg };
  }

  function getSelectedVelocityStats(): { min: number; max: number; avg: number } {
    const selectedVelocities = notes
      .filter(n => selectedNotes.has(n.id))
      .map(n => n.velocity);

    if (selectedVelocities.length === 0) {
      return { min: 0, max: 0, avg: 0 };
    }

    const min = Math.min(...selectedVelocities);
    const max = Math.max(...selectedVelocities);
    const avg = Math.round(selectedVelocities.reduce((a, b) => a + b, 0) / selectedVelocities.length);

    return { min, max, avg };
  }

  // ============================================================================
  // Undo/Redo
  // ============================================================================

  function saveToUndo(): void {
    undoStack.push(JSON.parse(JSON.stringify(notes)));
    if (undoStack.length > 50) {
      undoStack.shift();
    }
    redoStack = [];
  }

  function undo(): void {
    if (undoStack.length === 0) return;

    const previous = undoStack.pop();
    if (previous) {
      redoStack.push(JSON.parse(JSON.stringify(notes)));
      notes = previous;
    }
  }

  function redo(): void {
    if (redoStack.length === 0) return;

    const next = redoStack.pop();
    if (next) {
      undoStack.push(JSON.parse(JSON.stringify(notes)));
      notes = next;
    }
  }

  // ============================================================================
  // Velocity Manipulation
  // ============================================================================

  function setVelocity(noteId: number, velocity: number): void {
    const clampedVelocity = Math.max(0, Math.min(127, Math.round(velocity)));
    notes = notes.map(n => n.id === noteId ? { ...n, velocity: clampedVelocity } : n);
  }

  function setSelectedVelocities(velocity: number): void {
    saveToUndo();
    notes = notes.map(n =>
      selectedNotes.has(n.id)
        ? { ...n, velocity: Math.max(0, Math.min(127, Math.round(velocity))) }
        : n
    );
  }

  function scaleSelectedVelocities(factor: number): void {
    saveToUndo();
    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;
      const newVelocity = Math.max(0, Math.min(127, Math.round(n.velocity * factor)));
      return { ...n, velocity: newVelocity };
    });
  }

  function humanizeVelocities(): void {
    if (humanizeAmount === 0) return;

    saveToUndo();
    const randomRange = (humanizeAmount / 100) * 127;

    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;

      const randomOffset = (Math.random() - 0.5) * randomRange;
      const newVelocity = Math.max(0, Math.min(127, Math.round(n.velocity + randomOffset)));

      return { ...n, velocity: newVelocity };
    });
  }

  function applyCurve(): void {
    if (selectedNotes.size === 0) return;

    saveToUndo();
    const selectedNotesList = notes.filter(n => selectedNotes.has(n.id));
    const count = selectedNotesList.length;

    selectedNotesList.forEach((note, index) => {
      const position = index / (count - 1 || 1); // 0 to 1
      let curveValue: number;

      switch (velocityCurve) {
        case 'linear':
          curveValue = position;
          break;
        case 'exponential':
          curveValue = Math.pow(position, 2);
          break;
        case 'logarithmic':
          curveValue = Math.sqrt(position);
          break;
        default:
          curveValue = position;
      }

      const velocity = Math.round(curveValue * 127);
      setVelocity(note.id, velocity);
    });
  }

  function normalizeVelocities(): void {
    const stats = getSelectedVelocityStats();
    if (stats.max === 0) return;

    saveToUndo();
    const factor = 127 / stats.max;

    notes = notes.map(n => {
      if (!selectedNotes.has(n.id)) return n;
      const newVelocity = Math.round(n.velocity * factor);
      return { ...n, velocity: Math.min(127, newVelocity) };
    });
  }

  // ============================================================================
  // Selection
  // ============================================================================

  function selectNote(noteId: number, addToSelection: boolean = false): void {
    if (!addToSelection) {
      selectedNotes.clear();
    }

    if (selectedNotes.has(noteId)) {
      selectedNotes.delete(noteId);
    } else {
      selectedNotes.add(noteId);
    }

    notes = notes.map(n => ({
      ...n,
      selected: selectedNotes.has(n.id)
    }));
  }

  function selectAll(): void {
    selectedNotes.clear();
    notes.forEach(n => selectedNotes.add(n.id));
    notes = notes.map(n => ({ ...n, selected: true }));
  }

  function clearSelection(): void {
    selectedNotes.clear();
    notes = notes.map(n => ({ ...n, selected: false }));
  }

  // ============================================================================
  // Fader Interaction
  // ============================================================================

  function handleFaderMouseDown(noteId: number): void {
    draggingNoteId = noteId;
    saveToUndo();
  }

  function handleFaderMouseMove(e: MouseEvent, noteId: number): void {
    if (draggingNoteId !== noteId) return;

    const faderHeight = 200;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const y = e.clientY - rect.top;
    const velocity = Math.max(0, Math.min(127, Math.round((1 - (y / faderHeight)) * 127)));

    setVelocity(noteId, velocity);
  }

  function handleFaderMouseUp(): void {
    draggingNoteId = null;
  }

  // ============================================================================
  // Keyboard Shortcuts
  // ============================================================================

  function handleKeydown(e: KeyboardEvent): void {
    if (e.ctrlKey && e.code === 'KeyA') {
      e.preventDefault();
      selectAll();
    } else if (e.ctrlKey && e.code === 'KeyZ') {
      e.preventDefault();
      undo();
    } else if (e.ctrlKey && e.code === 'KeyY') {
      e.preventDefault();
      redo();
    } else if (e.code === 'Escape') {
      e.preventDefault();
      clearSelection();
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('mouseup', handleFaderMouseUp);

    // Load demo notes
    notes = [
      { id: 1, noteName: 'C4', position: '1.1.0', velocity: 100, selected: false },
      { id: 2, noteName: 'E4', position: '1.2.0', velocity: 90, selected: false },
      { id: 3, noteName: 'G4', position: '1.3.0', velocity: 80, selected: false },
      { id: 4, noteName: 'C5', position: '1.4.0', velocity: 110, selected: false },
      { id: 5, noteName: 'E5', position: '2.1.0', velocity: 95, selected: false },
      { id: 6, noteName: 'G5', position: '2.2.0', velocity: 85, selected: false },
      { id: 7, noteName: 'C6', position: '2.3.0', velocity: 105, selected: false },
      { id: 8, noteName: 'E6', position: '2.4.0', velocity: 100, selected: false }
    ];

    return () => {
      window.removeEventListener('keydown', handleKeydown);
      window.removeEventListener('mouseup', handleFaderMouseUp);
    };
  });

  $: allStats = getVelocityStats();
  $: selectedStats = getSelectedVelocityStats();
</script>

<WindowBase windowId="velocity-editor" title="Velocity Editor">
  <div class="velocity-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-section">
        <button class="action-btn" on:click={selectAll} title="Select All (Ctrl+A)">
          Select All
        </button>
        <button class="action-btn" on:click={clearSelection} title="Clear Selection (Esc)">
          Clear
        </button>
      </div>

      <div class="toolbar-section">
        <label class="humanize-label">
          Humanize:
          <input
            type="range"
            min="0"
            max="50"
            step="1"
            bind:value={humanizeAmount}
            class="humanize-slider"
          />
          <span class="humanize-value">{humanizeAmount}%</span>
        </label>
        <button
          class="action-btn"
          on:click={humanizeVelocities}
          disabled={selectedNotes.size === 0 || humanizeAmount === 0}
          title="Apply random variation to selected notes"
        >
          Apply
        </button>
      </div>

      <div class="toolbar-section">
        <label class="curve-label">
          Curve:
          <select class="curve-select" bind:value={velocityCurve}>
            <option value="linear">Linear</option>
            <option value="exponential">Exponential</option>
            <option value="logarithmic">Logarithmic</option>
          </select>
        </label>
        <button
          class="action-btn"
          on:click={applyCurve}
          disabled={selectedNotes.size === 0}
          title="Apply velocity curve to selected notes"
        >
          Apply Curve
        </button>
      </div>

      <div class="toolbar-section">
        <button
          class="action-btn"
          on:click={normalizeVelocities}
          disabled={selectedNotes.size === 0}
          title="Normalize selected velocities to max"
        >
          Normalize
        </button>
        <button
          class="action-btn"
          on:click={() => scaleSelectedVelocities(1.1)}
          disabled={selectedNotes.size === 0}
          title="Increase velocities by 10%"
        >
          +10%
        </button>
        <button
          class="action-btn"
          on:click={() => scaleSelectedVelocities(0.9)}
          disabled={selectedNotes.size === 0}
          title="Decrease velocities by 10%"
        >
          -10%
        </button>
      </div>

      <div class="toolbar-section">
        <button
          class="undo-btn"
          on:click={undo}
          disabled={undoStack.length === 0}
          title="Undo (Ctrl+Z)"
        >
          ↶ Undo
        </button>
        <button
          class="redo-btn"
          on:click={redo}
          disabled={redoStack.length === 0}
          title="Redo (Ctrl+Y)"
        >
          ↷ Redo
        </button>
      </div>
    </div>

    <!-- Statistics Panel -->
    <div class="stats-panel">
      <div class="stats-section">
        <h4 class="stats-title">All Notes</h4>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">Min:</span>
            <span class="stat-value">{allStats.min}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Max:</span>
            <span class="stat-value">{allStats.max}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Avg:</span>
            <span class="stat-value">{allStats.avg}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Count:</span>
            <span class="stat-value">{notes.length}</span>
          </div>
        </div>
      </div>

      {#if selectedNotes.size > 0}
        <div class="stats-section">
          <h4 class="stats-title">Selected Notes</h4>
          <div class="stats-grid">
            <div class="stat-item">
              <span class="stat-label">Min:</span>
              <span class="stat-value">{selectedStats.min}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Max:</span>
              <span class="stat-value">{selectedStats.max}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Avg:</span>
              <span class="stat-value">{selectedStats.avg}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Count:</span>
              <span class="stat-value">{selectedNotes.size}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Faders Container -->
    <div class="faders-container">
      {#each notes as note (note.id)}
        <div class="fader-lane" class:selected={note.selected}>
          <!-- Note Info -->
          <div
            class="note-info"
            on:click={() => selectNote(note.id, false)}
            role="button"
            tabindex="0"
          >
            <div class="note-name">{note.noteName}</div>
            <div class="note-position">{note.position}</div>
          </div>

          <!-- Fader -->
          <div
            class="fader-track"
            on:mousedown={() => handleFaderMouseDown(note.id)}
            on:mousemove={(e) => handleFaderMouseMove(e, note.id)}
            role="slider"
            aria-label="Velocity fader"
            aria-valuenow={note.velocity}
            aria-valuemin="0"
            aria-valuemax="127"
            tabindex="0"
          >
            <div class="fader-scale">
              {#each [127, 100, 75, 50, 25, 0] as mark}
                <div class="scale-mark">
                  <span class="scale-value">{mark}</span>
                </div>
              {/each}
            </div>

            <div class="fader-fill" style="height: {(note.velocity / 127) * 100}%" />

            <div
              class="fader-thumb"
              style="bottom: {(note.velocity / 127) * 100}%"
            />
          </div>

          <!-- Velocity Display -->
          <div class="velocity-display">
            <div class="velocity-value">{note.velocity}</div>
            <div class="velocity-db">{velocityToDb(note.velocity)} dB</div>
          </div>
        </div>
      {/each}

      {#if notes.length === 0}
        <div class="no-notes">
          <p>No notes to edit</p>
          <p class="hint">Select notes in the Piano Roll to edit their velocities</p>
        </div>
      {/if}
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">Selected: {selectedNotes.size} / {notes.length}</span>
      <span class="status-item">Humanize: {humanizeAmount}%</span>
      <span class="status-item">Curve: {velocityCurve}</span>
      <span class="status-item">Undo: {undoStack.length} | Redo: {redoStack.length}</span>
    </div>
  </div>
</WindowBase>

<style>
  .velocity-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
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

  .action-btn,
  .undo-btn,
  .redo-btn {
    padding: 6px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s ease;
  }

  .action-btn:hover:not(:disabled),
  .undo-btn:hover:not(:disabled),
  .redo-btn:hover:not(:disabled) {
    background: var(--button-hover, #4a4a4a);
  }

  .action-btn:disabled,
  .undo-btn:disabled,
  .redo-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .humanize-label,
  .curve-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .humanize-slider {
    width: 100px;
    height: 4px;
    -webkit-appearance: none;
    background: var(--slider-bg, #3a3a3a);
    border-radius: 2px;
    outline: none;
  }

  .humanize-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
  }

  .humanize-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .humanize-value {
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    min-width: 30px;
    color: var(--text-primary, #e0e0e0);
  }

  .curve-select {
    padding: 6px 8px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  /* Statistics Panel */
  .stats-panel {
    display: flex;
    gap: 24px;
    padding: 12px 16px;
    background: var(--stats-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .stats-section {
    flex: 1;
  }

  .stats-title {
    margin: 0 0 8px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-label {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .stat-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--accent-color, #0078d4);
    font-variant-numeric: tabular-nums;
  }

  /* Faders Container */
  .faders-container {
    flex: 1;
    display: flex;
    gap: 16px;
    padding: 24px 16px;
    overflow-x: auto;
    overflow-y: hidden;
    background: var(--faders-bg, #1a1a1a);
  }

  .fader-lane {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    min-width: 80px;
    padding: 16px 12px;
    background: var(--lane-bg, #252525);
    border-radius: 8px;
    border: 2px solid transparent;
    transition: border-color 0.15s ease;
  }

  .fader-lane.selected {
    border-color: var(--accent-color, #0078d4);
    background: var(--lane-selected, #2a2a2a);
  }

  .note-info {
    text-align: center;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    transition: background 0.15s ease;
  }

  .note-info:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .note-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .note-position {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Fader */
  .fader-track {
    position: relative;
    width: 50px;
    height: 200px;
    cursor: ns-resize;
    user-select: none;
  }

  .fader-scale {
    position: absolute;
    left: -30px;
    top: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .scale-mark {
    position: relative;
  }

  .scale-value {
    font-size: 10px;
    color: var(--text-tertiary, #666);
    font-variant-numeric: tabular-nums;
  }

  .fader-fill {
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 8px;
    background: linear-gradient(
      to top,
      #52b788 0%,
      #52b788 70%,
      #f7dc6f 70%,
      #f7dc6f 85%,
      #e81123 85%
    );
    border-radius: 4px;
    transition: height 0.1s ease-out;
  }

  .fader-thumb {
    position: absolute;
    left: 50%;
    transform: translateX(-50%) translateY(50%);
    width: 40px;
    height: 20px;
    background: var(--fader-thumb, #0078d4);
    border: 2px solid var(--fader-thumb-border, #005a9e);
    border-radius: 4px;
    cursor: grab;
    transition: bottom 0.1s ease-out;
  }

  .fader-thumb:active {
    cursor: grabbing;
  }

  .velocity-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .velocity-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
  }

  .velocity-db {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  .no-notes {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary, #888);
    gap: 8px;
  }

  .no-notes p {
    margin: 0;
  }

  .hint {
    font-size: 13px;
    color: var(--text-tertiary, #666);
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

  /* Scrollbar */
  .faders-container::-webkit-scrollbar {
    height: 12px;
  }

  .faders-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .faders-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
