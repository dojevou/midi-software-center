# Day 2, Part 2B: Automation Frontend

**Duration:** 2.5 hours
**Prerequisites:** Parts 2A complete (automation playback)
**Files to create:** 3

---

## Overview

Build automation UI components:
1. TypeScript types for automation
2. Automation API client
3. AutomationLane Svelte component
4. AutomationEditor with visual curve editing

---

## Step 1: TypeScript Types (15 min)

Create `app/src/lib/types/automation.ts`:

```typescript
export type CurveType = 'Linear' | 'Step' | 'Cubic';

export type AutomationMode = 'Read' | 'Write' | 'Latch' | 'Touch' | 'Off';

export interface AutomationPoint {
  time_ticks: number;
  value: number;
  curve_type: CurveType;
}

export interface AutomationLane {
  id: string;
  track_id: number;
  parameter_id: string;
  name: string;
  points: AutomationPoint[];
  enabled: boolean;
  mode: AutomationMode;
}

export class AutomationUtils {
  /**
   * Convert normalized value (0-1) to parameter-specific value
   */
  static normalizedToValue(
    normalized: number,
    paramType: string
  ): { value: number; display: string } {
    switch (paramType) {
      case 'gain':
        const db = normalized * 72 - 60; // -60dB to +12dB
        return {
          value: db,
          display: db <= -60 ? '-∞ dB' : `${db.toFixed(1)} dB`,
        };

      case 'pan':
        const pan = normalized * 2 - 1; // -1 to +1
        if (pan === 0) return { value: pan, display: 'C' };
        const percent = Math.abs(pan * 100).toFixed(0);
        return {
          value: pan,
          display: pan < 0 ? `${percent}L` : `${percent}R`,
        };

      default:
        return {
          value: normalized,
          display: `${(normalized * 100).toFixed(0)}%`,
        };
    }
  }

  /**
   * Convert parameter value to normalized (0-1)
   */
  static valueToNormalized(value: number, paramType: string): number {
    switch (paramType) {
      case 'gain':
        return (value + 60) / 72; // -60dB to +12dB -> 0 to 1

      case 'pan':
        return (value + 1) / 2; // -1 to +1 -> 0 to 1

      default:
        return value;
    }
  }

  /**
   * Get color for automation lane
   */
  static getColorForParameter(paramId: string): string {
    const colors: Record<string, string> = {
      gain: '#22c55e',
      pan: '#3b82f6',
      volume: '#f59e0b',
      mute: '#ef4444',
      solo: '#eab308',
    };

    return colors[paramId] || '#8b5cf6';
  }
}
```

---

## Step 2: Automation API (20 min)

Create `app/src/lib/api/automationApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type {
  AutomationLane,
  AutomationMode,
  AutomationPoint,
  CurveType,
} from '../types/automation';

export class AutomationApi {
  static async createLane(
    trackId: number,
    parameterId: string,
    name: string
  ): Promise<AutomationLane> {
    return await invoke<AutomationLane>('create_automation_lane', {
      trackId,
      parameterId,
      name,
    });
  }

  static async getLane(laneId: string): Promise<AutomationLane | null> {
    return await invoke<AutomationLane | null>('get_automation_lane', {
      laneId,
    });
  }

  static async getTrackLanes(trackId: number): Promise<AutomationLane[]> {
    return await invoke<AutomationLane[]>('get_track_automation_lanes', {
      trackId,
    });
  }

  static async setMode(laneId: string, mode: AutomationMode): Promise<void> {
    await invoke('set_automation_mode', { laneId, mode });
  }

  static async recordPoint(
    laneId: string,
    value: number,
    isTouching: boolean
  ): Promise<void> {
    await invoke('record_automation', { laneId, value, isTouching });
  }

  static async addPoint(
    laneId: string,
    timeTicks: number,
    value: number,
    curveType: CurveType = 'Linear'
  ): Promise<void> {
    await invoke('add_automation_point', {
      laneId,
      timeTicks,
      value,
      curveType,
    });
  }

  static async removePoint(laneId: string, timeTicks: number): Promise<void> {
    await invoke('remove_automation_point', { laneId, timeTicks });
  }

  static async clearLane(laneId: string): Promise<void> {
    await invoke('clear_automation_lane', { laneId });
  }

  static async deleteLane(laneId: string): Promise<void> {
    await invoke('delete_automation_lane', { laneId });
  }

  static async setEnabled(enabled: boolean): Promise<void> {
    await invoke('set_automation_enabled', { enabled });
  }

  static async getValue(laneId: string): Promise<number | null> {
    return await invoke<number | null>('get_automation_value', { laneId });
  }

  static async getTrackValues(
    trackId: number,
    positionTicks: number
  ): Promise<Record<string, number>> {
    return await invoke<Record<string, number>>('get_track_automation_values', {
      trackId,
      positionTicks,
    });
  }
}
```

---

## Step 3: Automation Lane Component (1 hour)

Create `app/src/lib/components/DAW/AutomationLane.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { AutomationApi } from '../../api/automationApi';
  import { AutomationUtils } from '../../types/automation';
  import type { AutomationLane, AutomationPoint, CurveType } from '../../types/automation';

  export let lane: AutomationLane;
  export let timelineWidth: number = 800;
  export let maxTicks: number = 1920; // One bar at 480 PPQ
  export let height: number = 100;

  let canvas: HTMLCanvasElement;
  let isDragging = false;
  let draggedPointIndex: number | null = null;
  let hoveredPointIndex: number | null = null;

  $: paramColor = AutomationUtils.getColorForParameter(lane.parameter_id);

  onMount(() => {
    drawCurve();
  });

  $: if (canvas && lane.points) {
    drawCurve();
  }

  function drawCurve() {
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, timelineWidth, height);

    if (lane.points.length === 0) return;

    // Draw grid lines
    ctx.strokeStyle = '#333';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = (height / 4) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(timelineWidth, y);
      ctx.stroke();
    }

    // Draw automation curve
    ctx.strokeStyle = paramColor;
    ctx.lineWidth = 2;
    ctx.beginPath();

    for (let i = 0; i < lane.points.length - 1; i++) {
      const p1 = lane.points[i];
      const p2 = lane.points[i + 1];

      const x1 = (p1.time_ticks / maxTicks) * timelineWidth;
      const y1 = height - p1.value * height;

      const x2 = (p2.time_ticks / maxTicks) * timelineWidth;
      const y2 = height - p2.value * height;

      if (i === 0) {
        ctx.moveTo(x1, y1);
      }

      if (p1.curve_type === 'Step') {
        ctx.lineTo(x2, y1);
        ctx.lineTo(x2, y2);
      } else if (p1.curve_type === 'Linear') {
        ctx.lineTo(x2, y2);
      } else if (p1.curve_type === 'Cubic') {
        // Bezier curve
        const cpx1 = x1 + (x2 - x1) / 3;
        const cpy1 = y1;
        const cpx2 = x2 - (x2 - x1) / 3;
        const cpy2 = y2;
        ctx.bezierCurveTo(cpx1, cpy1, cpx2, cpy2, x2, y2);
      }
    }
    ctx.stroke();

    // Draw points
    lane.points.forEach((point, index) => {
      const x = (point.time_ticks / maxTicks) * timelineWidth;
      const y = height - point.value * height;

      ctx.fillStyle = index === hoveredPointIndex ? '#fff' : paramColor;
      ctx.beginPath();
      ctx.arc(x, y, 5, 0, Math.PI * 2);
      ctx.fill();

      // Draw outline
      ctx.strokeStyle = '#000';
      ctx.lineWidth = 2;
      ctx.stroke();
    });
  }

  function handleMouseDown(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Check if clicking on existing point
    const pointIndex = findPointAtPosition(x, y);

    if (pointIndex !== null) {
      isDragging = true;
      draggedPointIndex = pointIndex;
    } else {
      // Add new point
      addPointAtPosition(x, y);
    }
  }

  function handleMouseMove(e: MouseEvent) {
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    if (isDragging && draggedPointIndex !== null) {
      movePoint(draggedPointIndex, x, y);
    } else {
      // Update hovered point
      hoveredPointIndex = findPointAtPosition(x, y);
      drawCurve();
    }
  }

  function handleMouseUp() {
    isDragging = false;
    draggedPointIndex = null;
  }

  function findPointAtPosition(x: number, y: number): number | null {
    const threshold = 10;

    for (let i = 0; i < lane.points.length; i++) {
      const point = lane.points[i];
      const px = (point.time_ticks / maxTicks) * timelineWidth;
      const py = height - point.value * height;

      const distance = Math.sqrt((x - px) ** 2 + (y - py) ** 2);
      if (distance <= threshold) {
        return i;
      }
    }

    return null;
  }

  async function addPointAtPosition(x: number, y: number) {
    const timeTicks = Math.round((x / timelineWidth) * maxTicks);
    const value = Math.max(0, Math.min(1, 1 - y / height));

    try {
      await AutomationApi.addPoint(lane.id, timeTicks, value);
      // Reload lane
      const updated = await AutomationApi.getLane(lane.id);
      if (updated) {
        lane = updated;
      }
    } catch (error) {
      console.error('Failed to add point:', error);
    }
  }

  async function movePoint(index: number, x: number, y: number) {
    const oldPoint = lane.points[index];
    const newTimeTicks = Math.round((x / timelineWidth) * maxTicks);
    const newValue = Math.max(0, Math.min(1, 1 - y / height));

    // Remove old point
    try {
      await AutomationApi.removePoint(lane.id, oldPoint.time_ticks);
      await AutomationApi.addPoint(lane.id, newTimeTicks, newValue, oldPoint.curve_type);

      // Reload lane
      const updated = await AutomationApi.getLane(lane.id);
      if (updated) {
        lane = updated;
      }
    } catch (error) {
      console.error('Failed to move point:', error);
    }
  }

  async function handleRightClick(e: MouseEvent) {
    e.preventDefault();

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const pointIndex = findPointAtPosition(x, y);
    if (pointIndex !== null) {
      const point = lane.points[pointIndex];

      if (confirm('Delete this point?')) {
        try {
          await AutomationApi.removePoint(lane.id, point.time_ticks);
          const updated = await AutomationApi.getLane(lane.id);
          if (updated) {
            lane = updated;
          }
        } catch (error) {
          console.error('Failed to delete point:', error);
        }
      }
    }
  }

  async function handleModeChange(e: Event) {
    const select = e.target as HTMLSelectElement;
    const mode = select.value as any;

    try {
      await AutomationApi.setMode(lane.id, mode);
      lane.mode = mode;
    } catch (error) {
      console.error('Failed to set mode:', error);
    }
  }

  async function handleClear() {
    if (!confirm('Clear all automation points?')) return;

    try {
      await AutomationApi.clearLane(lane.id);
      const updated = await AutomationApi.getLane(lane.id);
      if (updated) {
        lane = updated;
      }
    } catch (error) {
      console.error('Failed to clear lane:', error);
    }
  }
</script>

<div class="automation-lane">
  <div class="lane-header">
    <div class="lane-info">
      <div class="lane-name" style="color: {paramColor}">{lane.name}</div>
      <div class="point-count">{lane.points.length} points</div>
    </div>

    <div class="lane-controls">
      <select class="mode-select" value={lane.mode} on:change={handleModeChange}>
        <option value="Read">Read</option>
        <option value="Write">Write</option>
        <option value="Latch">Latch</option>
        <option value="Touch">Touch</option>
        <option value="Off">Off</option>
      </select>

      <button class="clear-btn" on:click={handleClear} title="Clear all points">
        Clear
      </button>
    </div>
  </div>

  <canvas
    bind:this={canvas}
    width={timelineWidth}
    {height}
    class="automation-canvas"
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:mouseup={handleMouseUp}
    on:mouseleave={handleMouseUp}
    on:contextmenu={handleRightClick}
  />
</div>

<style>
  .automation-lane {
    background: #1a1a1a;
    border-radius: 6px;
    padding: 8px;
    margin-bottom: 8px;
  }

  .lane-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .lane-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .lane-name {
    font-size: 14px;
    font-weight: 600;
  }

  .point-count {
    font-size: 11px;
    color: #666;
  }

  .lane-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .mode-select {
    padding: 4px 8px;
    background: #252525;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 12px;
  }

  .clear-btn {
    padding: 4px 12px;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .clear-btn:hover {
    background: #b91c1c;
  }

  .automation-canvas {
    display: block;
    background: #0a0a0a;
    border: 1px solid #333;
    border-radius: 4px;
    cursor: crosshair;
  }
</style>
```

---

## Step 4: Automation Editor Container (40 min)

Create `app/src/lib/components/DAW/AutomationEditor.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { AutomationApi } from '../../api/automationApi';
  import AutomationLane from './AutomationLane.svelte';
  import type { AutomationLane as AutomationLaneType } from '../../types/automation';

  export let trackId: number;
  export let timelineWidth: number = 800;
  export let maxTicks: number = 1920;

  let lanes: AutomationLaneType[] = [];
  let showAddDialog = false;
  let newParamId = '';
  let newParamName = '';

  const commonParameters = [
    { id: 'gain', name: 'Gain' },
    { id: 'pan', name: 'Pan' },
    { id: 'volume', name: 'Volume' },
  ];

  onMount(async () => {
    await loadLanes();
  });

  async function loadLanes() {
    try {
      lanes = await AutomationApi.getTrackLanes(trackId);
    } catch (error) {
      console.error('Failed to load automation lanes:', error);
    }
  }

  async function handleAddLane() {
    if (!newParamId || !newParamName) return;

    try {
      const lane = await AutomationApi.createLane(trackId, newParamId, newParamName);
      lanes = [...lanes, lane];
      showAddDialog = false;
      newParamId = '';
      newParamName = '';
    } catch (error) {
      console.error('Failed to create lane:', error);
    }
  }

  async function handleDeleteLane(laneId: string) {
    if (!confirm('Delete this automation lane?')) return;

    try {
      await AutomationApi.deleteLane(laneId);
      lanes = lanes.filter((l) => l.id !== laneId);
    } catch (error) {
      console.error('Failed to delete lane:', error);
    }
  }

  function selectCommonParam(paramId: string, paramName: string) {
    newParamId = paramId;
    newParamName = paramName;
  }
</script>

<div class="automation-editor">
  <div class="editor-header">
    <h3>Automation - Track {trackId}</h3>
    <button class="add-btn" on:click={() => (showAddDialog = !showAddDialog)}>
      {showAddDialog ? '✕ Cancel' : '+ Add Lane'}
    </button>
  </div>

  {#if showAddDialog}
    <div class="add-dialog">
      <h4>Add Automation Lane</h4>

      <div class="common-params">
        {#each commonParameters as param}
          <button
            class="param-btn"
            class:selected={newParamId === param.id}
            on:click={() => selectCommonParam(param.id, param.name)}
          >
            {param.name}
          </button>
        {/each}
      </div>

      <div class="form-row">
        <input
          type="text"
          placeholder="Parameter ID"
          bind:value={newParamId}
          class="param-input"
        />
        <input
          type="text"
          placeholder="Display Name"
          bind:value={newParamName}
          class="param-input"
        />
      </div>

      <button class="create-btn" on:click={handleAddLane} disabled={!newParamId || !newParamName}>
        Create Lane
      </button>
    </div>
  {/if}

  {#if lanes.length === 0}
    <div class="empty-state">
      <p>No automation lanes yet.</p>
      <p>Click "+ Add Lane" to get started.</p>
    </div>
  {:else}
    <div class="lanes-container">
      {#each lanes as lane (lane.id)}
        <AutomationLane {lane} {timelineWidth} {maxTicks} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .automation-editor {
    background: #161616;
    border-radius: 8px;
    padding: 16px;
    min-height: 200px;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .editor-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .add-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
  }

  .add-btn:hover {
    background: #2563eb;
  }

  .add-dialog {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 16px;
    margin-bottom: 16px;
  }

  .add-dialog h4 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #fff;
  }

  .common-params {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .param-btn {
    padding: 8px 16px;
    background: #252525;
    color: #999;
    border: 1px solid #333;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .param-btn:hover {
    background: #2a2a2a;
    border-color: #444;
  }

  .param-btn.selected {
    background: #3b82f6;
    color: #fff;
    border-color: #3b82f6;
  }

  .form-row {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
  }

  .param-input {
    flex: 1;
    padding: 8px;
    background: #0a0a0a;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
  }

  .create-btn {
    padding: 8px 16px;
    background: #22c55e;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .create-btn:hover {
    background: #16a34a;
  }

  .create-btn:disabled {
    background: #333;
    color: #666;
    cursor: not-allowed;
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: #666;
  }

  .empty-state p {
    margin: 4px 0;
  }

  .lanes-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
</style>
```

---

## Verification (15 min)

```bash
npm run check
make dev
```

Test automation editor:
1. Open automation editor for a track
2. Click "+ Add Lane" and select "Gain"
3. Click on canvas to add points
4. Drag points to move them
5. Right-click to delete points
6. Change automation mode (Read, Write, Latch, Touch)
7. Clear all points
8. Verify curve interpolation (linear, step, cubic)

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Canvas not drawing | Check canvas dimensions, verify drawCurve is called |
| Points not clickable | Adjust threshold in findPointAtPosition |
| Curve interpolation wrong | Verify curve type handling in drawCurve |
| Mode changes not persisting | Ensure API call succeeds, reload lane after change |

---

## What's Next?

✅ **Day 1-2 Complete! You've built:**
- Complete automation system with models, recording, playback
- 12 Tauri commands for automation control
- Visual automation editor with curve editing
- 4 automation modes (Read, Write, Latch, Touch)
- Point add/move/delete with mouse interaction

**Next:** [Day 3, Part 3A: Preset Backend](./DAY3_PART_A_PRESET_BACKEND.md)
- Preset models (track, mixer, effect, project templates)
- Preset repository for save/load
- Preset serialization
- Tauri commands for preset management
