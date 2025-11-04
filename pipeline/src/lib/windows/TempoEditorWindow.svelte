<script lang="ts">
  /**
   * TempoEditorWindow - Tempo Map Editor
   *
   * Task-O-Matic: Edit tempo changes with instant/ramp transitions and visualization.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { playbackStore, playbackActions } from '$lib/stores/playbackStore';

  // ============================================================================
  // Types
  // ============================================================================

  interface TempoMarker {
    id: number;
    bar: number;
    beat: number;
    tick: number;
    totalTicks: number;
    bpm: number;
    type: 'instant' | 'ramp';
    rampDuration?: number; // In bars (for ramp type)
    selected: boolean;
  }

  // ============================================================================
  // Constants
  // ============================================================================

  const TICKS_PER_BEAT = 480;
  const PIXELS_PER_BEAT = 100;
  const TIMELINE_HEIGHT = 200;
  const MIN_BPM = 20;
  const MAX_BPM = 300;

  // ============================================================================
  // State
  // ============================================================================

  $: position = $playbackStore.position;
  $: currentTempo = $playbackStore.tempo;
  $: timeSignature = $playbackStore.timeSignature;

  let markers: TempoMarker[] = [];
  let selectedMarkers: Set<number> = new Set();
  let markerIdCounter = 1;
  let gridSnap = true;
  let snapDivision = 4; // Snap to beats
  let zoomH = 1.0;

  // Add/Edit mode
  let addMode = false;
  let newTempo = 120;
  let newTempoType: 'instant' | 'ramp' = 'instant';
  let newRampDuration = 4; // bars

  let isDragging = false;
  let dragMarkerId: number | null = null;
  let timelineContainer: HTMLElement;

  // ============================================================================
  // Helper Functions
  // ============================================================================

  function ticksToX(ticks: number): number {
    const beats = ticks / TICKS_PER_BEAT;
    return beats * PIXELS_PER_BEAT * zoomH;
  }

  function xToTicks(x: number): number {
    const beats = x / (PIXELS_PER_BEAT * zoomH);
    let ticks = Math.floor(beats * TICKS_PER_BEAT);

    if (gridSnap) {
      const snapTicks = TICKS_PER_BEAT / snapDivision;
      ticks = Math.round(ticks / snapTicks) * snapTicks;
    }

    return Math.max(0, ticks);
  }

  function bpmToY(bpm: number): number {
    const normalized = (bpm - MIN_BPM) / (MAX_BPM - MIN_BPM);
    return TIMELINE_HEIGHT - (normalized * TIMELINE_HEIGHT);
  }

  function yToBpm(y: number): number {
    const normalized = 1 - (y / TIMELINE_HEIGHT);
    const bpm = MIN_BPM + (normalized * (MAX_BPM - MIN_BPM));
    return Math.max(MIN_BPM, Math.min(MAX_BPM, Math.round(bpm)));
  }

  function ticksToPosition(totalTicks: number): { bar: number; beat: number; tick: number } {
    const ticksPerBeat = TICKS_PER_BEAT;
    const ticksPerBar = ticksPerBeat * timeSignature.numerator;

    const bar = Math.floor(totalTicks / ticksPerBar) + 1;
    const remainingTicks = totalTicks % ticksPerBar;
    const beat = Math.floor(remainingTicks / ticksPerBeat) + 1;
    const tick = remainingTicks % ticksPerBeat;

    return { bar, beat, tick };
  }

  function getTempoAtPosition(ticks: number): number {
    if (markers.length === 0) return currentTempo;

    const sortedMarkers = [...markers].sort((a, b) => a.totalTicks - b.totalTicks);

    // Find the last marker before or at this position
    let lastMarker: TempoMarker | null = null;
    let nextMarker: TempoMarker | null = null;

    for (let i = 0; i < sortedMarkers.length; i++) {
      if (sortedMarkers[i].totalTicks <= ticks) {
        lastMarker = sortedMarkers[i];
        nextMarker = sortedMarkers[i + 1] || null;
      } else {
        break;
      }
    }

    if (!lastMarker) return currentTempo;

    if (lastMarker.type === 'instant') {
      return lastMarker.bpm;
    }

    // Handle ramp
    if (lastMarker.type === 'ramp' && lastMarker.rampDuration && nextMarker) {
      const rampTicks = lastMarker.rampDuration * timeSignature.numerator * TICKS_PER_BEAT;
      const rampEnd = lastMarker.totalTicks + rampTicks;

      if (ticks <= rampEnd) {
        // Interpolate between last and next tempo
        const progress = (ticks - lastMarker.totalTicks) / rampTicks;
        const tempoDiff = nextMarker.bpm - lastMarker.bpm;
        return Math.round(lastMarker.bpm + (tempoDiff * progress));
      }
    }

    return lastMarker.bpm;
  }

  // ============================================================================
  // Marker Management
  // ============================================================================

  function addMarker(ticks: number, bpm: number, type: 'instant' | 'ramp' = 'instant', rampDuration?: number): void {
    const pos = ticksToPosition(ticks);

    const marker: TempoMarker = {
      id: markerIdCounter++,
      bar: pos.bar,
      beat: pos.beat,
      tick: pos.tick,
      totalTicks: ticks,
      bpm,
      type,
      rampDuration: type === 'ramp' ? rampDuration : undefined,
      selected: false
    };

    markers = [...markers, marker].sort((a, b) => a.totalTicks - b.totalTicks);
  }

  function removeMarker(markerId: number): void {
    markers = markers.filter(m => m.id !== markerId);
    selectedMarkers.delete(markerId);
  }

  function removeSelectedMarkers(): void {
    markers = markers.filter(m => !selectedMarkers.has(m.id));
    selectedMarkers.clear();
  }

  function selectMarker(markerId: number, addToSelection: boolean = false): void {
    if (!addToSelection) {
      selectedMarkers.clear();
    }

    if (selectedMarkers.has(markerId)) {
      selectedMarkers.delete(markerId);
    } else {
      selectedMarkers.add(markerId);
    }

    markers = markers.map(m => ({
      ...m,
      selected: selectedMarkers.has(m.id)
    }));
  }

  function updateMarker(markerId: number, updates: Partial<TempoMarker>): void {
    markers = markers.map(m => {
      if (m.id !== markerId) return m;

      const updated = { ...m, ...updates };

      // Validate BPM
      if (updated.bpm) {
        updated.bpm = Math.max(MIN_BPM, Math.min(MAX_BPM, updated.bpm));
      }

      // Recalculate position if ticks changed
      if (updates.totalTicks !== undefined) {
        const pos = ticksToPosition(updated.totalTicks);
        updated.bar = pos.bar;
        updated.beat = pos.beat;
        updated.tick = pos.tick;
      }

      return updated;
    }).sort((a, b) => a.totalTicks - b.totalTicks);
  }

  // ============================================================================
  // Tempo Ramp
  // ============================================================================

  function createTempoRamp(): void {
    if (!addMode) return;

    const ticks = position.totalTicks;
    addMarker(ticks, newTempo, newTempoType, newRampDuration);
    addMode = false;
  }

  async function applyTempoChange(bpm: number): Promise<void> {
    try {
      await playbackActions.setTempo(bpm);
    } catch (error) {
      console.error('Failed to set tempo:', error);
    }
  }

  // ============================================================================
  // Interaction Handlers
  // ============================================================================

  function handleTimelineClick(e: MouseEvent): void {
    if (e.button !== 0) return;

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const ticks = xToTicks(x);
    const bpm = yToBpm(y);

    // Check if clicking near an existing marker
    const CLICK_THRESHOLD = 10;
    const clickedMarker = markers.find(m => {
      const mx = ticksToX(m.totalTicks);
      const my = bpmToY(m.bpm);
      const distance = Math.sqrt(Math.pow(x - mx, 2) + Math.pow(y - my, 2));
      return distance < CLICK_THRESHOLD;
    });

    if (clickedMarker) {
      selectMarker(clickedMarker.id, e.shiftKey);
    } else if (addMode) {
      addMarker(ticks, bpm, newTempoType, newRampDuration);
    }
  }

  function handleMarkerMouseDown(e: MouseEvent, markerId: number): void {
    e.stopPropagation();
    isDragging = true;
    dragMarkerId = markerId;
    selectMarker(markerId, e.shiftKey);
  }

  function handleMarkerMouseMove(e: MouseEvent): void {
    if (!isDragging || dragMarkerId === null || !timelineContainer) return;

    const rect = timelineContainer.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const ticks = xToTicks(x);
    const bpm = yToBpm(y);

    updateMarker(dragMarkerId, { totalTicks: ticks, bpm });
  }

  function handleMouseUp(): void {
    isDragging = false;
    dragMarkerId = null;
  }

  function handleWheel(e: WheelEvent): void {
    if (e.ctrlKey) {
      e.preventDefault();
      if (e.deltaY < 0) {
        zoomH = Math.min(10, zoomH + 0.1);
      } else {
        zoomH = Math.max(0.1, zoomH - 0.1);
      }
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.code === 'Delete' || e.code === 'Backspace') {
      e.preventDefault();
      removeSelectedMarkers();
    } else if (e.code === 'KeyA' && e.ctrlKey) {
      e.preventDefault();
      markers.forEach(m => selectedMarkers.add(m.id));
      markers = markers.map(m => ({ ...m, selected: true }));
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('mousemove', handleMarkerMouseMove);

    // Load demo markers
    addMarker(0, 120, 'instant');
    addMarker(TICKS_PER_BEAT * 16, 140, 'ramp', 4);
    addMarker(TICKS_PER_BEAT * 32, 100, 'instant');

    return () => {
      window.removeEventListener('keydown', handleKeydown);
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('mousemove', handleMarkerMouseMove);
    };
  });

  $: currentPositionTempo = getTempoAtPosition(position.totalTicks);
</script>

<WindowBase windowId="tempo-editor" title="Tempo Editor">
  <div class="tempo-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-section">
        <div class="current-tempo">
          <span class="tempo-label">Current Tempo:</span>
          <span class="tempo-value">{currentPositionTempo} BPM</span>
        </div>
      </div>

      <div class="toolbar-section">
        <button
          class="action-btn"
          class:active={addMode}
          on:click={() => addMode = !addMode}
          title="Toggle add marker mode"
        >
          {addMode ? 'Cancel' : 'Add Marker'}
        </button>

        {#if addMode}
          <label class="input-label">
            BPM:
            <input
              type="number"
              min={MIN_BPM}
              max={MAX_BPM}
              step="1"
              bind:value={newTempo}
              class="tempo-input"
            />
          </label>

          <label class="input-label">
            Type:
            <select class="type-select" bind:value={newTempoType}>
              <option value="instant">Instant</option>
              <option value="ramp">Ramp</option>
            </select>
          </label>

          {#if newTempoType === 'ramp'}
            <label class="input-label">
              Duration (bars):
              <input
                type="number"
                min="1"
                max="64"
                step="1"
                bind:value={newRampDuration}
                class="duration-input"
              />
            </label>
          {/if}

          <button class="action-btn primary" on:click={createTempoRamp}>
            Create
          </button>
        {/if}
      </div>

      <div class="toolbar-section">
        <button
          class="action-btn"
          class:active={gridSnap}
          on:click={() => gridSnap = !gridSnap}
          title="Snap to grid"
        >
          Snap
        </button>

        <select class="snap-select" bind:value={snapDivision} disabled={!gridSnap}>
          <option value={1}>Bar</option>
          <option value={4}>Beat</option>
          <option value={8}>1/8</option>
          <option value={16}>1/16</option>
        </select>
      </div>

      <div class="toolbar-section">
        <button
          class="action-btn"
          on:click={removeSelectedMarkers}
          disabled={selectedMarkers.size === 0}
          title="Remove selected markers"
        >
          Remove
        </button>
      </div>

      <div class="toolbar-section">
        <label class="zoom-label">
          Zoom:
          <input
            type="range"
            min="0.1"
            max="10"
            step="0.1"
            bind:value={zoomH}
            class="zoom-slider"
          />
          <span class="zoom-value">{zoomH.toFixed(1)}x</span>
        </label>
      </div>
    </div>

    <!-- Timeline Area -->
    <div class="timeline-area">
      <!-- BPM Scale -->
      <div class="bpm-scale">
        {#each [300, 250, 200, 150, 100, 50, 20] as bpm}
          <div class="scale-mark">
            <span class="scale-label">{bpm}</span>
            <div class="scale-line" />
          </div>
        {/each}
      </div>

      <!-- Timeline -->
      <div
        class="timeline-container"
        bind:this={timelineContainer}
        on:click={handleTimelineClick}
        on:wheel={handleWheel}
        role="slider"
        tabindex="0"
      >
        <!-- Grid -->
        <div class="grid-background">
          {#each Array(64) as _, bar}
            <div
              class="beat-line"
              class:bar-line={bar % 4 === 0}
              style="left: {ticksToX(bar * TICKS_PER_BEAT)}px"
            />
          {/each}

          {#each [300, 250, 200, 150, 100, 50, 20] as bpm}
            <div class="bpm-line" style="top: {bpmToY(bpm)}px" />
          {/each}
        </div>

        <!-- Tempo Curve -->
        {#if markers.length > 0}
          <svg class="tempo-svg">
            <path
              d={markers
                .map((m, i) => {
                  const x = ticksToX(m.totalTicks);
                  const y = bpmToY(m.bpm);
                  return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
                })
                .join(' ')}
              class="tempo-path"
            />
          </svg>
        {/if}

        <!-- Tempo Markers -->
        {#each markers as marker (marker.id)}
          {@const x = ticksToX(marker.totalTicks)}
          {@const y = bpmToY(marker.bpm)}

          <div
            class="tempo-marker"
            class:selected={marker.selected}
            class:ramp={marker.type === 'ramp'}
            style="left: {x}px; top: {y}px"
            on:mousedown={(e) => handleMarkerMouseDown(e, marker.id)}
            on:contextmenu|preventDefault={() => removeMarker(marker.id)}
            role="button"
            tabindex="0"
          >
            <div class="marker-tooltip">
              <div>Position: {marker.bar}.{marker.beat}.{marker.tick}</div>
              <div>BPM: {marker.bpm}</div>
              <div>Type: {marker.type}</div>
              {#if marker.type === 'ramp' && marker.rampDuration}
                <div>Duration: {marker.rampDuration} bars</div>
              {/if}
            </div>

            {#if marker.type === 'ramp'}
              <div class="ramp-indicator">R</div>
            {/if}
          </div>
        {/each}

        <!-- Playhead -->
        {#if position}
          <div
            class="playhead"
            style="left: {ticksToX(position.totalTicks)}px"
          />
        {/if}
      </div>
    </div>

    <!-- Marker List -->
    <div class="marker-list">
      <div class="list-header">
        <span class="list-title">Tempo Changes</span>
        <span class="list-count">{markers.length} markers</span>
      </div>

      <div class="list-container">
        {#each markers as marker (marker.id)}
          <div
            class="list-item"
            class:selected={marker.selected}
            on:click={() => selectMarker(marker.id, false)}
            role="button"
            tabindex="0"
          >
            <div class="list-position">{marker.bar}.{marker.beat}.{marker.tick}</div>
            <div class="list-bpm">{marker.bpm} BPM</div>
            <div class="list-type">{marker.type}</div>
            <button
              class="list-remove"
              on:click|stopPropagation={() => removeMarker(marker.id)}
              title="Remove marker"
            >
              ×
            </button>
          </div>
        {/each}

        {#if markers.length === 0}
          <div class="list-empty">
            <p>No tempo markers</p>
            <p class="hint">Click "Add Marker" to create tempo changes</p>
          </div>
        {/if}
      </div>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">Markers: {markers.length} | Selected: {selectedMarkers.size}</span>
      <span class="status-item">Snap: {gridSnap ? `1/${snapDivision}` : 'Off'}</span>
      <span class="status-item">Zoom: {zoomH.toFixed(1)}x</span>
      <span class="status-item">Range: {MIN_BPM}-{MAX_BPM} BPM</span>
    </div>
  </div>
</WindowBase>

<style>
  .tempo-window {
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

  .current-tempo {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tempo-label {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .tempo-value {
    font-size: 16px;
    font-weight: 600;
    color: var(--accent-color, #0078d4);
    font-variant-numeric: tabular-nums;
  }

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

  .action-btn:hover:not(:disabled) {
    background: var(--button-hover, #4a4a4a);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  .action-btn.primary {
    background: var(--primary-color, #52b788);
    border-color: var(--primary-color, #52b788);
  }

  .input-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .tempo-input,
  .duration-input {
    width: 60px;
    padding: 6px 8px;
    background: var(--input-bg, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
  }

  .type-select,
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
    gap: 8px;
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

  /* Timeline Area */
  .timeline-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .bpm-scale {
    width: 60px;
    flex-shrink: 0;
    background: var(--scale-bg, #252525);
    border-right: 1px solid var(--border-color, #3e3e3e);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 10px 8px;
  }

  .scale-mark {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .scale-label {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
    min-width: 30px;
    text-align: right;
  }

  .scale-line {
    width: 8px;
    height: 1px;
    background: var(--border-color, #3e3e3e);
  }

  .timeline-container {
    flex: 1;
    position: relative;
    overflow: auto;
    background: var(--timeline-bg, #1a1a1a);
    cursor: crosshair;
    height: 250px;
  }

  .grid-background {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .beat-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--beat-line, #2a2a2a);
  }

  .beat-line.bar-line {
    background: var(--bar-line, #3a3a3a);
    width: 2px;
  }

  .bpm-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--bpm-line, #2a2a2a);
  }

  .tempo-svg {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .tempo-path {
    fill: none;
    stroke: var(--accent-color, #0078d4);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .tempo-marker {
    position: absolute;
    width: 14px;
    height: 14px;
    background: var(--marker-color, #0078d4);
    border: 2px solid var(--marker-border, #005a9e);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    cursor: move;
    transition: all 0.15s ease;
    z-index: 10;
  }

  .tempo-marker:hover {
    width: 18px;
    height: 18px;
    background: var(--marker-hover, #0084e8);
  }

  .tempo-marker.selected {
    background: var(--marker-selected, #f7dc6f);
    border-color: var(--marker-selected-border, #f4d03f);
  }

  .tempo-marker.ramp {
    border-radius: 4px;
  }

  .ramp-indicator {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 9px;
    font-weight: 700;
    color: white;
    pointer-events: none;
  }

  .marker-tooltip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-bottom: 8px;
    padding: 6px 10px;
    background: rgba(0, 0, 0, 0.9);
    color: white;
    font-size: 10px;
    border-radius: 4px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease;
    line-height: 1.4;
  }

  .tempo-marker:hover .marker-tooltip {
    opacity: 1;
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

  /* Marker List */
  .marker-list {
    height: 150px;
    display: flex;
    flex-direction: column;
    background: var(--list-bg, #252525);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .list-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .list-count {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
  }

  .list-item {
    display: grid;
    grid-template-columns: 100px 100px 80px 1fr auto;
    gap: 12px;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .list-item:hover {
    background: var(--item-hover, #2a2a2a);
  }

  .list-item.selected {
    background: var(--item-selected, #3a3a3a);
  }

  .list-position,
  .list-bpm,
  .list-type {
    font-size: 12px;
    color: var(--text-primary, #e0e0e0);
  }

  .list-position {
    font-variant-numeric: tabular-nums;
    color: var(--text-secondary, #888);
  }

  .list-bpm {
    font-weight: 600;
    color: var(--accent-color, #0078d4);
  }

  .list-type {
    text-transform: capitalize;
  }

  .list-remove {
    width: 24px;
    height: 24px;
    background: transparent;
    color: var(--text-secondary, #888);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    transition: all 0.15s ease;
  }

  .list-remove:hover {
    background: #e81123;
    color: white;
  }

  .list-empty {
    padding: 48px 24px;
    text-align: center;
    color: var(--text-secondary, #888);
  }

  .list-empty p {
    margin: 0 0 8px 0;
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

  /* Scrollbars */
  .timeline-container::-webkit-scrollbar,
  .list-container::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .timeline-container::-webkit-scrollbar-track,
  .list-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .timeline-container::-webkit-scrollbar-thumb,
  .list-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
