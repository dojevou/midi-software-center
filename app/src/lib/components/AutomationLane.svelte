<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { AutomationPoint as SharedAutomationPoint } from '$lib/types';

  // Local interface that extends or maps to shared type
  interface AutomationPoint {
    id: number;
    time: number;
    value: number;
    curve?: 'linear' | 'exponential' | 'logarithmic' | 'smooth';
  }

  // Convert from shared type (tick) to local type (time)
  function toLocalPoint(p: SharedAutomationPoint | AutomationPoint): AutomationPoint {
    // Check if it's a SharedAutomationPoint (has tick) or local (has time)
    if ('tick' in p) {
      return { id: p.id, time: p.tick, value: p.value };
    }
    return p;
  }

  export let points: (AutomationPoint | SharedAutomationPoint)[] = [];

  // Normalize all points to local format
  $: normalizedPoints = points.map(toLocalPoint);

  export let minValue: number = 0;
  export let maxValue: number = 127;
  export let height: number = 100;
  export let width: number = 800;
  export let color: string = '#3b82f6';
  export let gridSize: number = 480; // One beat in ticks
  export let zoomLevel: number = 1;
  export let selectedPoints: Set<number> = new Set();
  export let parameterName: string = 'Parameter';

  const dispatch = createEventDispatcher<{
    pointAdd: { time: number; value: number };
    pointMove: { pointId: number; time: number; value: number };
    pointDelete: { pointId: number };
    pointSelect: { pointId: number; ctrlKey: boolean };
  }>();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let isDragging = false;
  let dragPointId: number | null = null;

  const POINT_RADIUS = 6;
  const PPQN = 480;

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
      draw();
    }
  });

  function draw() {
    if (!ctx || !canvas) {
      return;
    }

    const w = canvas.width;
    const h = canvas.height;

    ctx.clearRect(0, 0, w, h);

    // Draw grid
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
    ctx.lineWidth = 1;

    // Vertical grid lines (beats)
    for (let tick = 0; tick < w / zoomLevel; tick += gridSize) {
      const x = tickToX(tick);
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();
    }

    // Horizontal grid lines (value divisions)
    const divisions = 4;
    for (let i = 1; i < divisions; i++) {
      const y = (h / divisions) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }

    // Draw automation curve
    if (normalizedPoints.length > 0) {
      ctx.strokeStyle = color;
      ctx.lineWidth = 2;
      ctx.beginPath();

      const sortedPoints = [...normalizedPoints].sort((a, b) => a.time - b.time);

      for (let i = 0; i < sortedPoints.length; i++) {
        const point = sortedPoints[i];
        const x = tickToX(point.time);
        const y = valueToY(point.value);

        if (i === 0) {
          // Draw from left edge to first point
          ctx.moveTo(0, y);
          ctx.lineTo(x, y);
        } else {
          const prevPoint = sortedPoints[i - 1];
          const prevX = tickToX(prevPoint.time);
          const prevY = valueToY(prevPoint.value);

          // Draw curve based on curve type
          switch (prevPoint.curve) {
            case 'exponential':
              drawExponentialCurve(ctx, prevX, prevY, x, y);
              break;
            case 'logarithmic':
              drawLogarithmicCurve(ctx, prevX, prevY, x, y);
              break;
            case 'smooth':
              drawSmoothCurve(ctx, prevX, prevY, x, y);
              break;
            default:
              ctx.lineTo(x, y);
          }
        }

        if (i === sortedPoints.length - 1) {
          // Draw from last point to right edge
          ctx.lineTo(w, y);
        }
      }
      ctx.stroke();

      // Draw points
      for (const point of sortedPoints) {
        const x = tickToX(point.time);
        const y = valueToY(point.value);
        const isSelected = selectedPoints.has(point.id);

        ctx.beginPath();
        ctx.arc(x, y, POINT_RADIUS, 0, Math.PI * 2);
        ctx.fillStyle = isSelected ? '#ffffff' : color;
        ctx.fill();

        if (isSelected) {
          ctx.strokeStyle = color;
          ctx.lineWidth = 2;
          ctx.stroke();
        }
      }
    }
  }

  function drawExponentialCurve(
    ctx: CanvasRenderingContext2D,
    x1: number,
    y1: number,
    x2: number,
    y2: number
  ) {
    const steps = 20;
    for (let i = 1; i <= steps; i++) {
      const t = i / steps;
      const x = x1 + (x2 - x1) * t;
      const y = y1 + (y2 - y1) * (t * t);
      ctx.lineTo(x, y);
    }
  }

  function drawLogarithmicCurve(
    ctx: CanvasRenderingContext2D,
    x1: number,
    y1: number,
    x2: number,
    y2: number
  ) {
    const steps = 20;
    for (let i = 1; i <= steps; i++) {
      const t = i / steps;
      const x = x1 + (x2 - x1) * t;
      const y = y1 + (y2 - y1) * Math.sqrt(t);
      ctx.lineTo(x, y);
    }
  }

  function drawSmoothCurve(
    ctx: CanvasRenderingContext2D,
    x1: number,
    y1: number,
    x2: number,
    y2: number
  ) {
    const cx1 = x1 + (x2 - x1) / 3;
    const cx2 = x1 + ((x2 - x1) * 2) / 3;
    ctx.bezierCurveTo(cx1, y1, cx2, y2, x2, y2);
  }

  function tickToX(tick: number): number {
    return (tick / PPQN) * 100 * zoomLevel;
  }

  function xToTick(x: number): number {
    return (x / (100 * zoomLevel)) * PPQN;
  }

  function valueToY(value: number): number {
    const normalized = (value - minValue) / (maxValue - minValue);
    return height - normalized * height;
  }

  function yToValue(y: number): number {
    const normalized = 1 - y / height;
    return minValue + normalized * (maxValue - minValue);
  }

  function handleCanvasClick(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Check if clicking on existing point
    for (const point of normalizedPoints) {
      const px = tickToX(point.time);
      const py = valueToY(point.value);
      const distance = Math.sqrt(Math.pow(x - px, 2) + Math.pow(y - py, 2));

      if (distance <= POINT_RADIUS * 1.5) {
        dispatch('pointSelect', { pointId: point.id, ctrlKey: event.ctrlKey });
        return;
      }
    }

    // Add new point
    const tick = xToTick(x);
    const value = Math.round(yToValue(y));
    dispatch('pointAdd', { time: tick, value });
  }

  function handleMouseDown(event: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    for (const point of normalizedPoints) {
      const px = tickToX(point.time);
      const py = valueToY(point.value);
      const distance = Math.sqrt(Math.pow(x - px, 2) + Math.pow(y - py, 2));

      if (distance <= POINT_RADIUS * 1.5) {
        isDragging = true;
        dragPointId = point.id;
        break;
      }
    }
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging || dragPointId === null) {
      return;
    }

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    const tick = Math.max(0, xToTick(x));
    const value = Math.round(Math.min(maxValue, Math.max(minValue, yToValue(y))));

    dispatch('pointMove', { pointId: dragPointId, time: tick, value });
  }

  function handleMouseUp() {
    isDragging = false;
    dragPointId = null;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Delete' || event.key === 'Backspace') {
      selectedPoints.forEach((pointId) => {
        dispatch('pointDelete', { pointId });
      });
    }
  }

  $: {
    normalizedPoints;
    selectedPoints;
    zoomLevel;
    if (ctx) {
      draw();
    }
  }
</script>

<div class="automation-lane dark:bg-window-subtle rounded border dark:border-window-border">
  <div class="header flex items-center justify-between p-2 border-b dark:border-window-border">
    <span class="text-sm dark:text-gray-300">{parameterName}</span>
    <div class="flex items-center gap-2 text-xs dark:text-gray-400">
      <span>Min: {minValue}</span>
      <span>Max: {maxValue}</span>
    </div>
  </div>

  <canvas
    bind:this={canvas}
    {width}
    {height}
    class="cursor-crosshair"
    on:click={handleCanvasClick}
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseup={handleMouseUp}
    on:mouseleave={handleMouseUp}
    on:keydown={handleKeyDown}
    tabindex="0"
    role="application"
    aria-label="Automation lane for {parameterName}"
  ></canvas>
</div>

<style>
  canvas {
    display: block;
  }
</style>
