<script lang="ts">
  import { sequencerActions } from '$lib/stores/sequencerStore';

  // Props
  export let zoom: number;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  export let bpm: number;
  export let timeSignature: [number, number];
  export let lengthBars: number;
  export let loopEnabled: boolean;
  export let loopStartBar: number;
  export let loopEndBar: number;
  export let playheadTicks: number;
  export let ticksPerBeat: number;

  // Calculate derived values
  $: pxPerTick = zoom / ticksPerBeat;
  $: pxPerBeat = zoom;
  $: pxPerBar = pxPerBeat * timeSignature[0];
  $: totalWidth = lengthBars * pxPerBar;

  // Playhead position in pixels
  $: playheadX = playheadTicks * pxPerTick;

  // Loop region in pixels
  $: loopStartX = (loopStartBar - 1) * pxPerBar;
  $: loopEndX = loopEndBar * pxPerBar;

  // Generate bar markers
  $: barMarkers = Array.from({ length: lengthBars + 1 }, (_, i) => ({
    bar: i + 1,
    x: i * pxPerBar,
    isMajor: (i + 1) % 4 === 1, // Every 4 bars is major
  }));

  // Generate beat markers within visible area
  $: beatMarkers = generateBeatMarkers();

  function generateBeatMarkers() {
    const markers: Array<{ x: number; isBeat: boolean }> = [];
    const beatsPerBar = timeSignature[0];

    for (let bar = 0; bar < lengthBars; bar++) {
      for (let beat = 0; beat < beatsPerBar; beat++) {
        const x = bar * pxPerBar + beat * pxPerBeat;
        markers.push({ x, isBeat: beat === 0 });
      }
    }

    return markers;
  }

  // Local state for dragging
  let isDraggingPlayhead = false;
  let isDraggingLoopStart = false;
  let isDraggingLoopEnd = false;

  // Handle click on timeline to set playhead
  function handleClick(event: MouseEvent) {
    if (isDraggingPlayhead || isDraggingLoopStart || isDraggingLoopEnd) { return; }

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    const tick = Math.max(0, Math.floor(x / pxPerTick));
    sequencerActions.setPlayheadPosition(tick);
  }

  // Handle playhead drag
  function handlePlayheadMouseDown(event: MouseEvent) {
    event.stopPropagation();
    isDraggingPlayhead = true;
    window.addEventListener('mousemove', handlePlayheadMove);
    window.addEventListener('mouseup', handlePlayheadUp);
  }

  function handlePlayheadMove(event: MouseEvent) {
    const timeline = document.querySelector('.timeline-ruler') as HTMLElement;
    if (!timeline) { return; }
    const rect = timeline.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const tick = Math.max(0, Math.floor(x / pxPerTick));
    sequencerActions.setPlayheadPosition(tick);
  }

  function handlePlayheadUp() {
    isDraggingPlayhead = false;
    window.removeEventListener('mousemove', handlePlayheadMove);
    window.removeEventListener('mouseup', handlePlayheadUp);
  }

  // Handle loop start drag
  function handleLoopStartMouseDown(event: MouseEvent) {
    event.stopPropagation();
    isDraggingLoopStart = true;
    window.addEventListener('mousemove', handleLoopStartMove);
    window.addEventListener('mouseup', handleLoopStartUp);
  }

  function handleLoopStartMove(event: MouseEvent) {
    const timeline = document.querySelector('.timeline-ruler') as HTMLElement;
    if (!timeline) { return; }
    const rect = timeline.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const bar = Math.max(1, Math.min(loopEndBar - 1, Math.round(x / pxPerBar) + 1));
    sequencerActions.setLoopRange(bar, loopEndBar);
  }

  function handleLoopStartUp() {
    isDraggingLoopStart = false;
    window.removeEventListener('mousemove', handleLoopStartMove);
    window.removeEventListener('mouseup', handleLoopStartUp);
  }

  // Handle loop end drag
  function handleLoopEndMouseDown(event: MouseEvent) {
    event.stopPropagation();
    isDraggingLoopEnd = true;
    window.addEventListener('mousemove', handleLoopEndMove);
    window.addEventListener('mouseup', handleLoopEndUp);
  }

  function handleLoopEndMove(event: MouseEvent) {
    const timeline = document.querySelector('.timeline-ruler') as HTMLElement;
    if (!timeline) { return; }
    const rect = timeline.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const bar = Math.max(loopStartBar + 1, Math.min(lengthBars, Math.round(x / pxPerBar)));
    sequencerActions.setLoopRange(loopStartBar, bar);
  }

  function handleLoopEndUp() {
    isDraggingLoopEnd = false;
    window.removeEventListener('mousemove', handleLoopEndMove);
    window.removeEventListener('mouseup', handleLoopEndUp);
  }

  // Double-click to add marker
  function handleDoubleClick(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    const tick = Math.floor(x / pxPerTick);
    const markerName = prompt('Marker name:');
    if (markerName) {
      sequencerActions.addMarker(markerName, tick);
    }
  }
</script>

<div
  class="timeline-ruler"
  style="width: {totalWidth}px;"
  on:click={handleClick}
  on:dblclick={handleDoubleClick}
  role="slider"
  aria-label="Timeline"
  aria-valuemin="0"
  aria-valuemax={lengthBars * timeSignature[0] * ticksPerBeat}
  aria-valuenow={playheadTicks}
>
  <!-- Background grid -->
  <div class="timeline-grid">
    <!-- Beat lines -->
    {#each beatMarkers as marker (marker.x)}
      <div
        class="beat-line"
        class:bar-line={marker.isBeat}
        style="left: {marker.x}px;"
      />
    {/each}
  </div>

  <!-- Bar numbers -->
  <div class="bar-numbers">
    {#each barMarkers as marker (marker.bar)}
      <div
        class="bar-number"
        class:major={marker.isMajor}
        style="left: {marker.x}px;"
      >
        {marker.bar}
      </div>
    {/each}
  </div>

  <!-- Loop region -->
  {#if loopEnabled}
    <div
      class="loop-region"
      style="left: {loopStartX}px; width: {loopEndX - loopStartX}px;"
    >
      <!-- Loop start handle -->
      <div
        class="loop-handle loop-start-handle"
        on:mousedown={handleLoopStartMouseDown}
        role="slider"
        aria-label="Loop start"
      >
        <span class="loop-label">L</span>
      </div>

      <!-- Loop end handle -->
      <div
        class="loop-handle loop-end-handle"
        on:mousedown={handleLoopEndMouseDown}
        role="slider"
        aria-label="Loop end"
      >
        <span class="loop-label">R</span>
      </div>
    </div>
  {/if}

  <!-- Playhead -->
  <div
    class="playhead"
    style="left: {playheadX}px;"
    on:mousedown={handlePlayheadMouseDown}
    role="slider"
    aria-label="Playhead position"
  >
    <div class="playhead-head" />
    <div class="playhead-line" />
  </div>
</div>

<style>
  .timeline-ruler {
    position: relative;
    height: 100%;
    background: var(--menu-bg, #2a2a2a);
    cursor: pointer;
    user-select: none;
    overflow: visible;
  }

  .timeline-grid {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }

  .beat-line {
    position: absolute;
    top: 28px;
    bottom: 0;
    width: 1px;
    background: var(--border-color, #3a3a3a);
    opacity: 0.3;
  }

  .beat-line.bar-line {
    opacity: 0.6;
    top: 20px;
  }

  .bar-numbers {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 20px;
    background: var(--menu-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
  }

  .bar-number {
    position: absolute;
    transform: translateX(-50%);
    font-size: 10px;
    color: var(--text-muted, #888);
    padding-top: 4px;
  }

  .bar-number.major {
    color: var(--text-color, #e0e0e0);
    font-weight: 600;
  }

  .loop-region {
    position: absolute;
    top: 20px;
    height: 12px;
    background: var(--primary-color, #3b82f6);
    opacity: 0.6;
    border-radius: 2px;
  }

  .loop-handle {
    position: absolute;
    top: -2px;
    width: 14px;
    height: 16px;
    background: var(--primary-color, #3b82f6);
    border-radius: 2px;
    cursor: ew-resize;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .loop-handle:hover {
    background: var(--primary-dark, #2563eb);
  }

  .loop-start-handle {
    left: -7px;
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }

  .loop-end-handle {
    right: -7px;
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }

  .loop-label {
    font-size: 8px;
    color: white;
    font-weight: 600;
  }

  .playhead {
    position: absolute;
    top: 0;
    z-index: 50;
    cursor: ew-resize;
  }

  .playhead-head {
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-top: 10px solid var(--error-color, #ef4444);
    transform: translateX(-6px);
  }

  .playhead-head:hover {
    border-top-color: var(--error-dark, #dc2626);
  }

  .playhead-line {
    position: absolute;
    top: 10px;
    left: -0.5px;
    width: 1px;
    height: 30px;
    background: var(--error-color, #ef4444);
  }
</style>
