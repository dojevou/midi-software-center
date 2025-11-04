<script lang="ts">
  /**
   * ControllerEditorWindow - MIDI CC Controller Editor
   *
   * Task-O-Matic: Edit MIDI Continuous Controller (CC) curves with interpolation and smoothing.
   */

  import { onMount, onDestroy } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { playbackStore } from '$lib/stores/playbackStore';

  // ============================================================================
  // Types
  // ============================================================================

  interface CCPoint {
    id: number;
    tick: number;
    value: number; // 0-127
    selected: boolean;
  }

  type SmoothingMode = 'none' | 'linear' | 'bezier';

  interface CCController {
    number: number;
    name: string;
  }

  // ============================================================================
  // Constants
  // ============================================================================

  const CC_CONTROLLERS: CCController[] = [
    { number: 1, name: 'Modulation' },
    { number: 7, name: 'Volume' },
    { number: 10, name: 'Pan' },
    { number: 11, name: 'Expression' },
    { number: 64, name: 'Sustain Pedal' },
    { number: 71, name: 'Resonance' },
    { number: 72, name: 'Release Time' },
    { number: 73, name: 'Attack Time' },
    { number: 74, name: 'Cutoff' },
    { number: 91, name: 'Reverb Send' },
    { number: 93, name: 'Chorus Send' }
  ];

  const TICKS_PER_BEAT = 480;
  const PIXELS_PER_BEAT = 100;
  const TIMELINE_HEIGHT = 200;

  // ============================================================================
  // State
  // ============================================================================

  $: position = $playbackStore.position;
  $: isPlaying = $playbackStore.isPlaying;

  let selectedCC = 7; // Default to Volume
  let points: CCPoint[] = [];
  let selectedPoints: Set<number> = new Set();
  let pointIdCounter = 1;
  let smoothingMode: SmoothingMode = 'linear';
  let zoomH = 1.0;

  let isDragging = false;
  let dragPointId: number | null = null;
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
    return Math.floor(beats * TICKS_PER_BEAT);
  }

  function valueToY(value: number): number {
    return TIMELINE_HEIGHT - ((value / 127) * TIMELINE_HEIGHT);
  }

  function yToValue(y: number): number {
    return Math.max(0, Math.min(127, Math.round((1 - (y / TIMELINE_HEIGHT)) * 127)));
  }

  function getSelectedController(): CCController {
    return CC_CONTROLLERS.find(cc => cc.number === selectedCC) || CC_CONTROLLERS[0];
  }

  function getStats(): { min: number; max: number; current: number } {
    if (points.length === 0) {
      return { min: 0, max: 0, current: 0 };
    }

    const values = points.map(p => p.value);
    const min = Math.min(...values);
    const max = Math.max(...values);

    // Find value at current position
    const currentTick = position.totalTicks;
    let current = 64; // Default mid value

    const sortedPoints = [...points].sort((a, b) => a.tick - b.tick);
    for (let i = 0; i < sortedPoints.length; i++) {
      if (sortedPoints[i].tick <= currentTick) {
        current = sortedPoints[i].value;
      } else {
        break;
      }
    }

    return { min, max, current };
  }

  // ============================================================================
  // Point Management
  // ============================================================================

  function addPoint(tick: number, value: number): void {
    // Check if point already exists at this tick
    const existingPoint = points.find(p => Math.abs(p.tick - tick) < 10);
    if (existingPoint) {
      // Update existing point
      points = points.map(p =>
        p.id === existingPoint.id ? { ...p, value } : p
      );
      return;
    }

    const point: CCPoint = {
      id: pointIdCounter++,
      tick,
      value,
      selected: false
    };

    points = [...points, point].sort((a, b) => a.tick - b.tick);
  }

  function removePoint(pointId: number): void {
    points = points.filter(p => p.id !== pointId);
    selectedPoints.delete(pointId);
  }

  function removeSelectedPoints(): void {
    points = points.filter(p => !selectedPoints.has(p.id));
    selectedPoints.clear();
  }

  function selectPoint(pointId: number, addToSelection: boolean = false): void {
    if (!addToSelection) {
      selectedPoints.clear();
    }

    if (selectedPoints.has(pointId)) {
      selectedPoints.delete(pointId);
    } else {
      selectedPoints.add(pointId);
    }

    points = points.map(p => ({
      ...p,
      selected: selectedPoints.has(p.id)
    }));
  }

  function selectAll(): void {
    selectedPoints.clear();
    points.forEach(p => selectedPoints.add(p.id));
    points = points.map(p => ({ ...p, selected: true }));
  }

  // ============================================================================
  // Curve Operations
  // ============================================================================

  function interpolatePoints(): void {
    if (points.length < 2) return;

    const sortedPoints = [...points].sort((a, b) => a.tick - b.tick);
    const newPoints: CCPoint[] = [];

    for (let i = 0; i < sortedPoints.length - 1; i++) {
      const p1 = sortedPoints[i];
      const p2 = sortedPoints[i + 1];

      newPoints.push(p1);

      const tickDiff = p2.tick - p1.tick;
      const steps = Math.floor(tickDiff / (TICKS_PER_BEAT / 4)); // Interpolate every 16th note

      for (let j = 1; j < steps; j++) {
        const t = j / steps;
        const tick = p1.tick + (tickDiff * t);
        let value: number;

        switch (smoothingMode) {
          case 'linear':
            value = Math.round(p1.value + ((p2.value - p1.value) * t));
            break;
          case 'bezier':
            // Cubic bezier interpolation
            const t2 = t * t;
            const t3 = t2 * t;
            value = Math.round(
              p1.value * (1 - 3*t + 3*t2 - t3) +
              p2.value * (3*t - 6*t2 + 3*t3) +
              p2.value * (3*t2 - 3*t3) +
              p2.value * t3
            );
            break;
          default:
            value = p1.value;
        }

        newPoints.push({
          id: pointIdCounter++,
          tick,
          value,
          selected: false
        });
      }
    }

    newPoints.push(sortedPoints[sortedPoints.length - 1]);
    points = newPoints;
  }

  function smoothCurve(): void {
    if (points.length < 3) return;

    const sortedPoints = [...points].sort((a, b) => a.tick - b.tick);
    const smoothedPoints = sortedPoints.map((point, i) => {
      if (i === 0 || i === sortedPoints.length - 1) {
        return point; // Don't smooth first and last points
      }

      const prev = sortedPoints[i - 1];
      const next = sortedPoints[i + 1];
      const smoothedValue = Math.round((prev.value + point.value + next.value) / 3);

      return { ...point, value: smoothedValue };
    });

    points = smoothedPoints;
  }

  function resetCC(): void {
    if (confirm('Reset all CC points? This cannot be undone.')) {
      points = [];
      selectedPoints.clear();
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

    const tick = xToTicks(x);
    const value = yToValue(y);

    // Check if clicking near an existing point
    const CLICK_THRESHOLD = 10;
    const clickedPoint = points.find(p => {
      const px = ticksToX(p.tick);
      const py = valueToY(p.value);
      const distance = Math.sqrt(Math.pow(x - px, 2) + Math.pow(y - py, 2));
      return distance < CLICK_THRESHOLD;
    });

    if (clickedPoint) {
      selectPoint(clickedPoint.id, e.shiftKey);
    } else {
      addPoint(tick, value);
    }
  }

  function handlePointMouseDown(e: MouseEvent, pointId: number): void {
    e.stopPropagation();
    isDragging = true;
    dragPointId = pointId;
    selectPoint(pointId, e.shiftKey);
  }

  function handlePointMouseMove(e: MouseEvent): void {
    if (!isDragging || dragPointId === null || !timelineContainer) return;

    const rect = timelineContainer.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const tick = Math.max(0, xToTicks(x));
    const value = yToValue(y);

    points = points.map(p =>
      p.id === dragPointId ? { ...p, tick, value } : p
    ).sort((a, b) => a.tick - b.tick);
  }

  function handleMouseUp(): void {
    isDragging = false;
    dragPointId = null;
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
      removeSelectedPoints();
    } else if (e.code === 'KeyA' && e.ctrlKey) {
      e.preventDefault();
      selectAll();
    }
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('mousemove', handlePointMouseMove);

    // Load demo points
    addPoint(0, 64);
    addPoint(TICKS_PER_BEAT * 4, 100);
    addPoint(TICKS_PER_BEAT * 8, 40);
    addPoint(TICKS_PER_BEAT * 12, 80);

    return () => {
      window.removeEventListener('keydown', handleKeydown);
      window.removeEventListener('mouseup', handleMouseUp);
      window.removeEventListener('mousemove', handlePointMouseMove);
    };
  });

  $: stats = getStats();
  $: controller = getSelectedController();
</script>

<WindowBase windowId="controller-editor" title="CC Controller Editor">
  <div class="controller-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="toolbar-section">
        <label class="cc-label">
          Controller:
          <select class="cc-select" bind:value={selectedCC}>
            {#each CC_CONTROLLERS as cc}
              <option value={cc.number}>CC {cc.number} - {cc.name}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="toolbar-section">
        <label class="smoothing-label">
          Smoothing:
          <select class="smoothing-select" bind:value={smoothingMode}>
            <option value="none">None</option>
            <option value="linear">Linear</option>
            <option value="bezier">Bezier</option>
          </select>
        </label>
        <button
          class="action-btn"
          on:click={interpolatePoints}
          disabled={points.length < 2}
          title="Interpolate between points"
        >
          Interpolate
        </button>
        <button
          class="action-btn"
          on:click={smoothCurve}
          disabled={points.length < 3}
          title="Smooth curve using averaging"
        >
          Smooth
        </button>
      </div>

      <div class="toolbar-section">
        <button class="action-btn" on:click={selectAll} title="Select All (Ctrl+A)">
          Select All
        </button>
        <button
          class="action-btn"
          on:click={removeSelectedPoints}
          disabled={selectedPoints.size === 0}
          title="Remove selected points"
        >
          Remove
        </button>
        <button class="action-btn danger" on:click={resetCC} title="Clear all points">
          Reset
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

    <!-- Statistics Panel -->
    <div class="stats-panel">
      <div class="stat-item">
        <span class="stat-label">Controller:</span>
        <span class="stat-value">CC {controller.number} - {controller.name}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Min Value:</span>
        <span class="stat-value">{stats.min}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Max Value:</span>
        <span class="stat-value">{stats.max}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Current Value:</span>
        <span class="stat-value current">{stats.current}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Points:</span>
        <span class="stat-value">{points.length}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Selected:</span>
        <span class="stat-value">{selectedPoints.size}</span>
      </div>
    </div>

    <!-- Timeline Area -->
    <div class="timeline-area">
      <!-- Value Scale -->
      <div class="value-scale">
        {#each [127, 100, 75, 50, 25, 0] as value}
          <div class="scale-mark">
            <span class="scale-label">{value}</span>
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

          {#each [127, 100, 75, 50, 25, 0] as value}
            <div class="value-line" style="top: {valueToY(value)}px" />
          {/each}
        </div>

        <!-- Curve Path -->
        {#if points.length > 1}
          <svg class="curve-svg">
            <path
              d={points
                .sort((a, b) => a.tick - b.tick)
                .map((p, i) => {
                  const x = ticksToX(p.tick);
                  const y = valueToY(p.value);
                  return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
                })
                .join(' ')}
              class="curve-path"
            />
          </svg>
        {/if}

        <!-- Control Points -->
        {#each points as point (point.id)}
          {@const x = ticksToX(point.tick)}
          {@const y = valueToY(point.value)}

          <div
            class="control-point"
            class:selected={point.selected}
            style="left: {x}px; top: {y}px"
            on:mousedown={(e) => handlePointMouseDown(e, point.id)}
            on:contextmenu|preventDefault={() => removePoint(point.id)}
            role="button"
            tabindex="0"
          >
            <div class="point-tooltip">
              <div>Tick: {point.tick}</div>
              <div>Value: {point.value}</div>
            </div>
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

    <!-- Instructions -->
    <div class="instructions">
      <span class="instruction-item">Click to add point</span>
      <span class="instruction-item">Drag to move point</span>
      <span class="instruction-item">Right-click to delete</span>
      <span class="instruction-item">Ctrl+Wheel to zoom</span>
      <span class="instruction-item">Delete key to remove selected</span>
    </div>

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">CC {controller.number} - {controller.name}</span>
      <span class="status-item">Points: {points.length} | Selected: {selectedPoints.size}</span>
      <span class="status-item">Smoothing: {smoothingMode}</span>
      <span class="status-item">Zoom: {zoomH.toFixed(1)}x</span>
    </div>
  </div>
</WindowBase>

<style>
  .controller-window {
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

  .cc-label,
  .smoothing-label,
  .zoom-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .cc-select,
  .smoothing-select {
    padding: 6px 8px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
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

  .action-btn.danger {
    color: #e81123;
  }

  .action-btn.danger:hover:not(:disabled) {
    background: #e81123;
    color: white;
    border-color: #e81123;
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

  /* Statistics Panel */
  .stats-panel {
    display: flex;
    gap: 24px;
    padding: 12px 16px;
    background: var(--stats-bg, #252525);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .stat-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .stat-label {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  .stat-value {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
  }

  .stat-value.current {
    color: var(--accent-color, #0078d4);
    font-size: 14px;
  }

  /* Timeline Area */
  .timeline-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .value-scale {
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

  .value-line {
    position: absolute;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--value-line, #2a2a2a);
  }

  .curve-svg {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  .curve-path {
    fill: none;
    stroke: var(--accent-color, #0078d4);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .control-point {
    position: absolute;
    width: 12px;
    height: 12px;
    background: var(--point-color, #0078d4);
    border: 2px solid var(--point-border, #005a9e);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    cursor: move;
    transition: all 0.15s ease;
    z-index: 10;
  }

  .control-point:hover {
    width: 16px;
    height: 16px;
    background: var(--point-hover, #0084e8);
  }

  .control-point.selected {
    background: var(--point-selected, #f7dc6f);
    border-color: var(--point-selected-border, #f4d03f);
  }

  .point-tooltip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    margin-bottom: 8px;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.9);
    color: white;
    font-size: 10px;
    border-radius: 4px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease;
  }

  .control-point:hover .point-tooltip {
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

  /* Instructions */
  .instructions {
    display: flex;
    gap: 16px;
    padding: 8px 12px;
    background: var(--instructions-bg, #252525);
    border-top: 1px solid var(--border-color, #3e3e3e);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .instruction-item {
    font-size: 11px;
    color: var(--text-tertiary, #666);
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 12px;
    background: var(--status-bg, #2d2d2d);
  }

  .status-item {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Scrollbar */
  .timeline-container::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .timeline-container::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .timeline-container::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
