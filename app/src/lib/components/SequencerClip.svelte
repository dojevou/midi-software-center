<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    sequencerActions,
    type SequencerClip,
    type SnapValue,
    getSnapTicks,
    snapToGrid,
  } from '$lib/stores/sequencerStore';
  import { playbackStore } from '$lib/stores/playbackStore';

  // Props
  export let clip: SequencerClip;
  export let trackId: number;
  export let pxPerTick: number;
  export let ticksPerBeat: number;
  export let snapValue: SnapValue;
  export let trackColor: string;

  const dispatch = createEventDispatcher();

  // Calculate clip position and dimensions
  $: left = clip.startTick * pxPerTick;
  $: width = Math.max(20, clip.lengthTicks * pxPerTick);

  // Local drag state
  let isDragging = false;
  let isResizingStart = false;
  let isResizingEnd = false;
  let dragStartX = 0;
  let originalStartTick = 0;
  let originalLengthTicks = 0;

  // Get snap ticks for current snap value
  $: snapTicks = getSnapTicks(snapValue, ticksPerBeat);

  // Handle clip selection
  function handleClick(event: MouseEvent) {
    event.stopPropagation();
    sequencerActions.selectClip(clip.id, event.shiftKey || event.ctrlKey || event.metaKey);
  }

  // Handle double-click to open editor
  function handleDoubleClick(event: MouseEvent) {
    event.stopPropagation();
    // Dispatch edit event to parent component (Sequencer handles opening piano roll)
    console.log('Opening clip in piano roll editor:', clip.id, 'track:', trackId);
    dispatch('edit', { clipId: clip.id, trackId, fileId: clip.midiFileId, clipName: clip.name });
  }

  // Handle drag start
  function handleMouseDown(event: MouseEvent) {
    if (event.button !== 0) { return; } // Only left click
    event.stopPropagation();

    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;

    // Determine if resizing from edges (10px from each edge)
    if (x <= 10) {
      isResizingStart = true;
    } else if (x >= width - 10) {
      isResizingEnd = true;
    } else {
      isDragging = true;
    }

    dragStartX = event.clientX;
    originalStartTick = clip.startTick;
    originalLengthTicks = clip.lengthTicks;

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(event: MouseEvent) {
    const deltaX = event.clientX - dragStartX;
    const deltaTicks = Math.round(deltaX / pxPerTick);

    if (isDragging) {
      // Move clip
      let newStartTick = originalStartTick + deltaTicks;
      newStartTick = snapToGrid(newStartTick, snapValue, ticksPerBeat);
      newStartTick = Math.max(0, newStartTick);
      sequencerActions.updateClip(trackId, clip.id, { startTick: newStartTick });
    } else if (isResizingStart) {
      // Resize from start
      let newStartTick = originalStartTick + deltaTicks;
      newStartTick = snapToGrid(newStartTick, snapValue, ticksPerBeat);
      newStartTick = Math.max(0, newStartTick);
      const newLength = originalLengthTicks - (newStartTick - originalStartTick);
      if (newLength >= snapTicks) {
        sequencerActions.updateClip(trackId, clip.id, {
          startTick: newStartTick,
          lengthTicks: newLength,
        });
      }
    } else if (isResizingEnd) {
      // Resize from end
      let newLength = originalLengthTicks + deltaTicks;
      newLength = snapToGrid(newLength, snapValue, ticksPerBeat);
      newLength = Math.max(snapTicks, newLength);
      sequencerActions.updateClip(trackId, clip.id, { lengthTicks: newLength });
    }
  }

  function handleMouseUp() {
    isDragging = false;
    isResizingStart = false;
    isResizingEnd = false;
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
  }

  // Handle context menu - clip options
  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    // Select the clip and show context options
    sequencerActions.selectClip(clip.id, false);
    // Log available actions for debugging - full context menu UI can be added later
    const options = [
      { label: 'Open in Piano Roll', action: () => dispatch('edit', { clipId: clip.id, trackId, fileId: clip.midiFileId, clipName: clip.name }) },
      { label: 'Duplicate Clip', action: () => duplicateClip() },
      { label: 'Delete Clip', action: () => sequencerActions.removeClip(trackId, clip.id) },
      { label: 'Split at Playhead', action: () => splitAtPlayhead() },
    ];
    console.log('Clip context menu:', clip.id, 'Options:', options.map(o => o.label));
  }

  // Duplicate clip functionality
  function duplicateClip() {
    // Place duplicated clip right after the current one
    const newStartTick = clip.startTick + clip.lengthTicks;
    sequencerActions.addClip(trackId, newStartTick, clip.lengthTicks, clip.midiFileId, `${clip.name} (copy)`);
  }

  // Split clip at the current playhead position
  function splitAtPlayhead() {
    const playheadTick = $playbackStore.position.current_tick;
    const clipEnd = clip.startTick + clip.lengthTicks;

    // Check if playhead is within clip boundaries (excluding edges)
    if (playheadTick <= clip.startTick || playheadTick >= clipEnd) {
      console.log('Playhead not within clip bounds - cannot split');
      return;
    }

    // Calculate the lengths for both clips
    const firstClipLength = playheadTick - clip.startTick;
    const secondClipLength = clipEnd - playheadTick;

    // Ensure both clips would have meaningful length
    if (firstClipLength < snapTicks || secondClipLength < snapTicks) {
      console.log('Split would create clips too small - minimum is', snapTicks, 'ticks');
      return;
    }

    // Update the original clip to end at the playhead
    sequencerActions.updateClip(trackId, clip.id, { lengthTicks: firstClipLength });

    // Create a new clip starting at the playhead with the remaining content
    sequencerActions.addClip(
      trackId,
      playheadTick,
      secondClipLength,
      clip.midiFileId,
      `${clip.name} (split)`
    );

    console.log('Split clip at tick', playheadTick, '- first:', firstClipLength, 'second:', secondClipLength);
  }

  // Handle delete key when focused
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Delete' || event.key === 'Backspace') {
      event.preventDefault();
      sequencerActions.removeClip(trackId, clip.id);
    }
  }

  // Generate waveform pattern (simple visualization)
  function generateWaveform(): string {
    // Generate random-ish pattern based on clip ID for visual variety
    const seed = clip.id % 100;
    const bars: string[] = [];
    const barCount = Math.max(4, Math.floor(width / 8));

    for (let i = 0; i < barCount; i++) {
      const height = 30 + ((seed * (i + 1)) % 40);
      bars.push(`${i * 8},${50 - height / 2} ${i * 8},${50 + height / 2}`);
    }

    return `M ${bars.join(' L ')}`;
  }

  // Determine cursor based on position
  function getCursor(event: MouseEvent): string {
    if (!event.currentTarget) { return 'grab'; }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left;
    if (x <= 10 || x >= width - 10) {
      return 'ew-resize';
    }
    return 'grab';
  }
</script>

<div
  class="sequencer-clip"
  class:selected={clip.selected}
  class:dragging={isDragging}
  class:resizing={isResizingStart || isResizingEnd}
  style="
    left: {left}px;
    width: {width}px;
    background-color: {clip.color || trackColor};
  "
  on:click={handleClick}
  on:dblclick={handleDoubleClick}
  on:mousedown={handleMouseDown}
  on:contextmenu={handleContextMenu}
  on:keydown={handleKeyDown}
  role="button"
  tabindex="0"
  aria-label="Clip: {clip.name}"
>
  <!-- Clip header -->
  <div class="clip-header">
    <span class="clip-name" title={clip.name}>{clip.name}</span>
    {#if clip.loopCount > 1}
      <span class="loop-count">x{clip.loopCount}</span>
    {/if}
  </div>

  <!-- Clip waveform visualization -->
  <div class="clip-waveform">
    <svg width="100%" height="100%" preserveAspectRatio="none">
      <path
        d={generateWaveform()}
        stroke="rgba(255,255,255,0.5)"
        stroke-width="1"
        fill="none"
      />
    </svg>
  </div>

  <!-- Resize handles -->
  <div class="resize-handle resize-start" />
  <div class="resize-handle resize-end" />

  <!-- Loop indicators -->
  {#if clip.loopCount > 1}
    {#each Array(clip.loopCount - 1) as _, i (i)}
      <div
        class="loop-marker"
        style="left: {((i + 1) * 100) / clip.loopCount}%;"
      />
    {/each}
  {/if}

  <!-- Transpose/velocity indicators -->
  {#if clip.transpose !== 0 || clip.velocityScale !== 100}
    <div class="clip-modifiers">
      {#if clip.transpose !== 0}
        <span class="modifier">
          {clip.transpose > 0 ? '+' : ''}{clip.transpose}
        </span>
      {/if}
      {#if clip.velocityScale !== 100}
        <span class="modifier">{clip.velocityScale}%</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sequencer-clip {
    position: absolute;
    top: 4px;
    bottom: 4px;
    border-radius: 4px;
    overflow: hidden;
    cursor: grab;
    transition: box-shadow 0.15s, transform 0.05s;
    z-index: 10;
  }

  .sequencer-clip:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    z-index: 20;
  }

  .sequencer-clip.selected {
    box-shadow: 0 0 0 2px white, 0 2px 8px rgba(0, 0, 0, 0.4);
    z-index: 30;
  }

  .sequencer-clip.dragging,
  .sequencer-clip.resizing {
    cursor: grabbing;
    opacity: 0.9;
    z-index: 40;
  }

  .clip-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    background: rgba(0, 0, 0, 0.3);
    font-size: 10px;
    font-weight: 500;
    color: white;
    white-space: nowrap;
    overflow: hidden;
  }

  .clip-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  }

  .loop-count {
    font-size: 9px;
    background: rgba(255, 255, 255, 0.2);
    padding: 1px 4px;
    border-radius: 2px;
  }

  .clip-waveform {
    position: absolute;
    top: 18px;
    left: 0;
    right: 0;
    bottom: 0;
    opacity: 0.7;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 10px;
    cursor: ew-resize;
    z-index: 5;
  }

  .resize-start {
    left: 0;
  }

  .resize-end {
    right: 0;
  }

  .resize-handle:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .loop-marker {
    position: absolute;
    top: 18px;
    bottom: 0;
    width: 1px;
    background: rgba(255, 255, 255, 0.3);
    pointer-events: none;
  }

  .clip-modifiers {
    position: absolute;
    bottom: 2px;
    right: 4px;
    display: flex;
    gap: 4px;
  }

  .modifier {
    font-size: 8px;
    background: rgba(0, 0, 0, 0.5);
    color: white;
    padding: 1px 3px;
    border-radius: 2px;
  }
</style>
