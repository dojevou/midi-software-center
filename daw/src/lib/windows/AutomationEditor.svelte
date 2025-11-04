<script lang="ts">
  // AutomationEditor.svelte
  // Task-O-Matic: Complete UI component for automation editing
  // Features: Timeline, stacked lanes, curve visualization, point interaction

  import { onMount, onDestroy } from 'svelte';
  import { automationStore, automationActions, currentTrackLanes } from '$lib/stores/automationStore';
  import { playbackPosition, tempo } from '$lib/stores/sequencer';
  import {
    parameterTypeToString,
    parameterTypeColor,
    normalizedToDisplay,
    displayToNormalized,
    type AutomationLane,
    type AutomationPoint,
    type ParameterType,
    type CurveType
  } from '$lib/types/automation';

  // Props
  export let track_id: number = 1;
  export let width: number = 1200;
  export let height: number = 600;

  // Component state
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null;
  let dragging_point: { lane_id: number; point_id: number } | null = null;
  let hover_point: { lane_id: number; point_id: number } | null = null;
  let context_menu: { x: number; y: number; lane_id: number; point_id: number } | null = null;

  // Constants
  const LANE_HEIGHT = 120;
  const LANE_HEADER_WIDTH = 120;
  const RULER_HEIGHT = 30;
  const POINT_RADIUS = 6;
  const POINT_RADIUS_HOVER = 8;
  const GRID_COLOR = '#333';
  const TEXT_COLOR = '#ccc';
  const PLAYHEAD_COLOR = '#ff4444';

  // Reactive statements
  $: lanes = $currentTrackLanes;
  $: zoom_h = $automationStore.view.horizontal_zoom;
  $: zoom_v = $automationStore.view.vertical_zoom;
  $: scroll_x = $automationStore.view.scroll_x;
  $: scroll_y = $automationStore.view.scroll_y;
  $: snap_enabled = $automationStore.snap.enabled;
  $: playhead_tick = $playbackPosition.current_tick;

  // Set track on mount
  onMount(() => {
    if (!ctx && canvas) {
      ctx = canvas.getContext('2d');
    }
    automationActions.setTrack(track_id);
    requestAnimationFrame(draw);
  });

  onDestroy(() => {
    // Cleanup
  });

  /**
   * Main draw function
   */
  function draw() {
    if (!ctx || !canvas) return;

    // Clear canvas
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, width, height);

    // Draw ruler
    drawRuler();

    // Draw lanes
    lanes.forEach((lane, index) => {
      const y_offset = RULER_HEIGHT + index * LANE_HEIGHT;
      drawLane(lane, y_offset);
    });

    // Draw playhead
    drawPlayhead();

    requestAnimationFrame(draw);
  }

  /**
   * Draw timeline ruler
   */
  function drawRuler() {
    if (!ctx) return;

    const ruler_width = width - LANE_HEADER_WIDTH;
    const ppq = 384; // Ticks per quarter note
    const beats_per_bar = 4;

    ctx.fillStyle = '#222';
    ctx.fillRect(LANE_HEADER_WIDTH, 0, ruler_width, RULER_HEIGHT);

    ctx.strokeStyle = GRID_COLOR;
    ctx.fillStyle = TEXT_COLOR;
    ctx.font = '11px monospace';
    ctx.textAlign = 'center';

    // Draw bars
    const bar_ticks = ppq * beats_per_bar;
    const visible_start_tick = scroll_x / zoom_h;
    const visible_end_tick = (scroll_x + ruler_width) / zoom_h;
    const start_bar = Math.floor(visible_start_tick / bar_ticks);
    const end_bar = Math.ceil(visible_end_tick / bar_ticks);

    for (let bar = start_bar; bar <= end_bar; bar++) {
      const tick = bar * bar_ticks;
      const x = LANE_HEADER_WIDTH + (tick * zoom_h) - scroll_x;

      if (x >= LANE_HEADER_WIDTH && x <= width) {
        // Draw bar line
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, RULER_HEIGHT);
        ctx.stroke();

        // Draw bar number
        ctx.fillText(`${bar + 1}`, x, RULER_HEIGHT - 5);
      }

      // Draw beats within bar
      for (let beat = 1; beat < beats_per_bar; beat++) {
        const beat_tick = tick + beat * ppq;
        const beat_x = LANE_HEADER_WIDTH + (beat_tick * zoom_h) - scroll_x;

        if (beat_x >= LANE_HEADER_WIDTH && beat_x <= width) {
          ctx.save();
          ctx.globalAlpha = 0.3;
          ctx.beginPath();
          ctx.moveTo(beat_x, RULER_HEIGHT - 10);
          ctx.lineTo(beat_x, RULER_HEIGHT);
          ctx.stroke();
          ctx.restore();
        }
      }
    }
  }

  /**
   * Draw automation lane
   */
  function drawLane(lane: AutomationLane, y_offset: number) {
    if (!ctx) return;

    // Draw lane header
    ctx.fillStyle = '#2a2a2a';
    ctx.fillRect(0, y_offset, LANE_HEADER_WIDTH, LANE_HEIGHT);

    ctx.fillStyle = parameterTypeColor(lane.parameter_type);
    ctx.font = 'bold 13px sans-serif';
    ctx.textAlign = 'left';
    ctx.fillText(
      lane.name || parameterTypeToString(lane.parameter_type),
      10,
      y_offset + 20
    );

    ctx.fillStyle = TEXT_COLOR;
    ctx.font = '11px sans-serif';
    ctx.fillText(`Track ${lane.track_id}`, 10, y_offset + 38);

    // Draw lane background
    ctx.fillStyle = '#1e1e1e';
    ctx.fillRect(LANE_HEADER_WIDTH, y_offset, width - LANE_HEADER_WIDTH, LANE_HEIGHT);

    // Draw grid lines (horizontal)
    ctx.strokeStyle = GRID_COLOR;
    for (let i = 0; i <= 4; i++) {
      const y = y_offset + (i * LANE_HEIGHT / 4);
      ctx.beginPath();
      ctx.moveTo(LANE_HEADER_WIDTH, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw value labels
    ctx.fillStyle = TEXT_COLOR;
    ctx.font = '10px monospace';
    ctx.textAlign = 'right';
    for (let i = 0; i <= 4; i++) {
      const value = 1.0 - (i * 0.25);
      const y = y_offset + (i * LANE_HEIGHT / 4);
      const display = normalizedToDisplay(value, lane.parameter_type);
      ctx.fillText(display.toString(), LANE_HEADER_WIDTH - 5, y + 4);
    }

    // Draw curve
    if (lane.curve.points.length > 0) {
      drawCurve(lane, y_offset);
    }

    // Draw points
    lane.curve.points.forEach(point => {
      drawPoint(lane, point, y_offset);
    });
  }

  /**
   * Draw automation curve
   */
  function drawCurve(lane: AutomationLane, y_offset: number) {
    if (!ctx || lane.curve.points.length === 0) return;

    const color = parameterTypeColor(lane.parameter_type);
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.globalAlpha = 0.7;

    ctx.beginPath();
    let first = true;

    // Sample curve at regular intervals
    const start_tick = scroll_x / zoom_h;
    const end_tick = (scroll_x + width) / zoom_h;
    const sample_interval = 10; // Sample every 10 ticks

    for (let tick = start_tick; tick <= end_tick; tick += sample_interval) {
      const value = interpolateValue(lane, tick);
      if (value === null) continue;

      const x = LANE_HEADER_WIDTH + (tick * zoom_h) - scroll_x;
      const y = y_offset + LANE_HEIGHT - (value * LANE_HEIGHT);

      if (first) {
        ctx.moveTo(x, y);
        first = false;
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();
    ctx.globalAlpha = 1.0;
    ctx.lineWidth = 1;
  }

  /**
   * Interpolate value at tick (client-side preview)
   */
  function interpolateValue(lane: AutomationLane, tick: number): number | null {
    const points = lane.curve.points;
    if (points.length === 0) return null;

    // Find surrounding points
    let prev_idx = -1;
    let next_idx = -1;

    for (let i = 0; i < points.length; i++) {
      if (points[i].time <= tick) {
        prev_idx = i;
      }
      if (points[i].time >= tick && next_idx === -1) {
        next_idx = i;
      }
    }

    // Before first point
    if (prev_idx === -1) return points[0].value;

    // After last point
    if (next_idx === -1) return points[points.length - 1].value;

    // Exact match
    if (prev_idx === next_idx) return points[prev_idx].value;

    // Interpolate
    const p1 = points[prev_idx];
    const p2 = points[next_idx];
    const t = (tick - p1.time) / (p2.time - p1.time);

    switch (lane.curve.curve_type) {
      case 'Linear':
        return p1.value + (p2.value - p1.value) * t;
      case 'Bezier':
        const t_smooth = t * t * (3 - 2 * t);
        return p1.value + (p2.value - p1.value) * t_smooth;
      case 'Exponential':
        const t_exp = p2.value > p1.value ? t * t : 1 - (1 - t) * (1 - t);
        return p1.value + (p2.value - p1.value) * t_exp;
      case 'Step':
        return p1.value;
      default:
        return p1.value;
    }
  }

  /**
   * Draw automation point
   */
  function drawPoint(lane: AutomationLane, point: AutomationPoint, y_offset: number) {
    if (!ctx) return;

    const x = LANE_HEADER_WIDTH + (point.time * zoom_h) - scroll_x;
    const y = y_offset + LANE_HEIGHT - (point.value * LANE_HEIGHT);

    // Skip if off-screen
    if (x < LANE_HEADER_WIDTH || x > width) return;

    const is_selected = $automationStore.selected_points?.point_ids.has(point.id) || false;
    const is_hover = hover_point?.point_id === point.id;
    const radius = is_hover ? POINT_RADIUS_HOVER : POINT_RADIUS;

    // Draw point
    ctx.fillStyle = is_selected ? '#ffff00' : parameterTypeColor(lane.parameter_type);
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();

    // Draw outline
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.lineWidth = 1;
  }

  /**
   * Draw playhead indicator
   */
  function drawPlayhead() {
    if (!ctx) return;

    const x = LANE_HEADER_WIDTH + (playhead_tick * zoom_h) - scroll_x;

    if (x >= LANE_HEADER_WIDTH && x <= width) {
      ctx.strokeStyle = PLAYHEAD_COLOR;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(x, RULER_HEIGHT);
      ctx.lineTo(x, height);
      ctx.stroke();
      ctx.lineWidth = 1;
    }
  }

  /**
   * Canvas mouse handlers
   */
  function handleMouseDown(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Check if clicked on point
    const clicked = findPointAtPosition(x, y);
    if (clicked) {
      dragging_point = clicked;
      automationActions.selectPoint(clicked.lane_id, clicked.point_id, e.shiftKey);
      return;
    }

    // Click in lane to add point
    const lane_index = Math.floor((y - RULER_HEIGHT) / LANE_HEIGHT);
    if (lane_index >= 0 && lane_index < lanes.length && x > LANE_HEADER_WIDTH) {
      const lane = lanes[lane_index];
      const tick = Math.floor((x - LANE_HEADER_WIDTH + scroll_x) / zoom_h);
      const lane_y = y - RULER_HEIGHT - lane_index * LANE_HEIGHT;
      const value = 1.0 - (lane_y / LANE_HEIGHT);

      automationActions.addPoint(track_id, lane.parameter_type, tick, value);
    }
  }

  function handleMouseMove(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Update hover state
    hover_point = findPointAtPosition(x, y);
    canvas.style.cursor = hover_point ? 'pointer' : 'default';

    // Drag point
    if (dragging_point && e.buttons === 1) {
      const lane_index = lanes.findIndex(l => l.id === dragging_point!.lane_id);
      if (lane_index === -1) return;

      const lane = lanes[lane_index];
      const tick = Math.floor((x - LANE_HEADER_WIDTH + scroll_x) / zoom_h);
      const lane_y = y - RULER_HEIGHT - lane_index * LANE_HEIGHT;
      const value = Math.max(0, Math.min(1, 1.0 - (lane_y / LANE_HEIGHT)));

      automationActions.movePoint(
        track_id,
        lane.parameter_type,
        dragging_point.point_id,
        tick,
        value
      );
    }
  }

  function handleMouseUp(e: MouseEvent) {
    dragging_point = null;
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const clicked = findPointAtPosition(x, y);
    if (clicked) {
      context_menu = {
        x: e.clientX,
        y: e.clientY,
        lane_id: clicked.lane_id,
        point_id: clicked.point_id
      };
    }
  }

  /**
   * Find point at screen position
   */
  function findPointAtPosition(x: number, y: number): { lane_id: number; point_id: number } | null {
    for (let i = 0; i < lanes.length; i++) {
      const lane = lanes[i];
      const y_offset = RULER_HEIGHT + i * LANE_HEIGHT;

      for (const point of lane.curve.points) {
        const px = LANE_HEADER_WIDTH + (point.time * zoom_h) - scroll_x;
        const py = y_offset + LANE_HEIGHT - (point.value * LANE_HEIGHT);

        const dx = x - px;
        const dy = y - py;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist <= POINT_RADIUS_HOVER) {
          return { lane_id: lane.id, point_id: point.id };
        }
      }
    }
    return null;
  }

  /**
   * Context menu actions
   */
  function deletePoint() {
    if (!context_menu) return;
    const lane = lanes.find(l => l.id === context_menu!.lane_id);
    if (!lane) return;

    automationActions.removePoint(track_id, lane.parameter_type, context_menu.point_id);
    context_menu = null;
  }

  function setCurveType(type: CurveType) {
    if (!context_menu) return;
    const lane = lanes.find(l => l.id === context_menu!.lane_id);
    if (!lane) return;

    automationActions.setCurveType(track_id, lane.parameter_type, type);
    context_menu = null;
  }

  /**
   * Keyboard shortcuts
   */
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      const selected = $automationStore.selected_points;
      if (selected) {
        const lane = lanes.find(l => l.id === selected.lane_id);
        if (lane) {
          selected.point_ids.forEach(point_id => {
            automationActions.removePoint(track_id, lane.parameter_type, point_id);
          });
        }
      }
    } else if (e.key === 'Escape') {
      automationActions.clearSelection();
      context_menu = null;
    }
  }

  /**
   * Zoom controls
   */
  function zoomIn() {
    automationActions.setZoom(zoom_h * 1.2, zoom_v * 1.2);
  }

  function zoomOut() {
    automationActions.setZoom(zoom_h / 1.2, zoom_v / 1.2);
  }

  function toggleSnap() {
    automationActions.toggleSnap();
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="automation-editor">
  <!-- Toolbar -->
  <div class="toolbar">
    <div class="toolbar-group">
      <button on:click={zoomIn} title="Zoom In">
        <span>🔍+</span>
      </button>
      <button on:click={zoomOut} title="Zoom Out">
        <span>🔍-</span>
      </button>
    </div>
    <div class="toolbar-group">
      <button
        class:active={snap_enabled}
        on:click={toggleSnap}
        title="Snap to Grid"
      >
        <span>🧲 Snap</span>
      </button>
    </div>
    <div class="toolbar-group">
      <span class="info">Track {track_id} | {lanes.length} lanes</span>
    </div>
  </div>

  <!-- Canvas -->
  <canvas
    bind:this={canvas}
    {width}
    {height}
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseup={handleMouseUp}
    on:contextmenu={handleContextMenu}
  />

  <!-- Context Menu -->
  {#if context_menu}
    <div
      class="context-menu"
      style="left: {context_menu.x}px; top: {context_menu.y}px;"
      on:click={() => context_menu = null}
    >
      <div class="menu-item" on:click={deletePoint}>Delete Point</div>
      <div class="menu-separator"></div>
      <div class="menu-item" on:click={() => setCurveType('Linear')}>Linear</div>
      <div class="menu-item" on:click={() => setCurveType('Bezier')}>Bezier</div>
      <div class="menu-item" on:click={() => setCurveType('Exponential')}>Exponential</div>
      <div class="menu-item" on:click={() => setCurveType('Step')}>Step</div>
    </div>
  {/if}

  <!-- Error Display -->
  {#if $automationStore.error}
    <div class="error">
      {$automationStore.error}
    </div>
  {/if}
</div>

<style>
  .automation-editor {
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    color: #ccc;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    position: relative;
  }

  .toolbar {
    display: flex;
    gap: 16px;
    padding: 12px;
    background: #252525;
    border-bottom: 1px solid #333;
    align-items: center;
  }

  .toolbar-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  button {
    padding: 6px 12px;
    background: #333;
    border: 1px solid #444;
    border-radius: 4px;
    color: #ccc;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  button:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  button.active {
    background: #4a6fa5;
    border-color: #5a7fb5;
    color: #fff;
  }

  .info {
    font-size: 12px;
    color: #888;
  }

  canvas {
    cursor: default;
  }

  .context-menu {
    position: fixed;
    background: #2a2a2a;
    border: 1px solid #444;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    min-width: 140px;
  }

  .menu-item {
    padding: 8px 12px;
    cursor: pointer;
    font-size: 13px;
  }

  .menu-item:hover {
    background: #3a3a3a;
  }

  .menu-separator {
    height: 1px;
    background: #444;
    margin: 4px 0;
  }

  .error {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    padding: 12px 20px;
    background: #ff4444;
    color: white;
    border-radius: 4px;
    font-size: 13px;
    z-index: 100;
  }
</style>
