<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    sequencerActions,
    type SequencerTrack,
    type SnapValue,
  } from '$lib/stores/sequencerStore';
  import { arrangementStore } from '$lib/stores/arrangementStore';
  import type { SequencerTrackType } from '$lib/types';
  import { getTrackTypeColor, getTrackTypeColorWithOpacity, detectTrackType } from '$lib/utils/trackColors';
  import SequencerClip from './SequencerClip.svelte';
  import TrackTypeIcon from './TrackTypeIcon.svelte';

  const dispatch = createEventDispatcher<{
    trackTypeChange: { trackId: number; trackType: SequencerTrackType };
  }>();

  // Props
  export let track: SequencerTrack;
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  export let index: number;
  export let zoom: number;
  export let scrollX: number;
  export let ticksPerBeat: number;
  export let snapValue: SnapValue;
  export let headerWidth: number;
  export let contentWidth: number;

  // Calculate pixels per tick
  $: pxPerTick = zoom / ticksPerBeat;

  // Local state
  let isResizing = false;
  let resizeStartY = 0;
  let resizeStartHeight = 0;

  // Channel options 1-16
  const channels = Array.from({ length: 16 }, (_, i) => i + 1);

  // Track type options
  const trackTypes: SequencerTrackType[] = ['midi', 'drum', 'audio', 'bus'];

  // Get track type from arrangement store, or detect it
  $: arrangementState = $arrangementStore;
  $: trackType = arrangementState.trackTypes[track.id] || detectTrackType({
    name: track.name,
    midiChannel: track.midiChannel,
    hasDrums: track.midiChannel === 10,  // Channel 10 is conventionally drums
  });

  // Compute track colors based on type
  $: trackTypeColor = getTrackTypeColor(trackType);
  $: trackTypeBgColor = getTrackTypeColorWithOpacity(trackType, 0.15);
  $: trackTypeBorderColor = getTrackTypeColorWithOpacity(trackType, 0.5);

  // Handle track type change
  function handleTrackTypeChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    const newType = select.value as SequencerTrackType;
    dispatch('trackTypeChange', { trackId: track.id, trackType: newType });
  }

  // Handle mute toggle
  function handleMuteClick() {
    sequencerActions.toggleTrackMute(track.id);
  }

  // Handle solo toggle
  function handleSoloClick() {
    sequencerActions.toggleTrackSolo(track.id);
  }

  // Handle record arm toggle
  function handleArmClick() {
    sequencerActions.toggleTrackArmed(track.id);
  }

  // Handle collapse toggle
  function handleCollapseClick() {
    sequencerActions.toggleTrackCollapsed(track.id);
  }

  // Handle track selection
  function handleTrackClick(event: MouseEvent) {
    sequencerActions.selectTrack(track.id, event.shiftKey || event.ctrlKey || event.metaKey);
  }

  // Handle track name change
  function handleNameChange(event: Event) {
    const input = event.target as HTMLInputElement;
    sequencerActions.updateTrack(track.id, { name: input.value });
  }

  // Handle channel change
  function handleChannelChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    sequencerActions.updateTrack(track.id, { midiChannel: parseInt(select.value) });
  }

  // Handle track resize
  function handleResizeStart(event: MouseEvent) {
    event.preventDefault();
    isResizing = true;
    resizeStartY = event.clientY;
    resizeStartHeight = track.height;
    window.addEventListener('mousemove', handleResizeMove);
    window.addEventListener('mouseup', handleResizeEnd);
  }

  function handleResizeMove(event: MouseEvent) {
    if (!isResizing) { return; }
    const deltaY = event.clientY - resizeStartY;
    const newHeight = Math.max(40, Math.min(200, resizeStartHeight + deltaY));
    sequencerActions.setTrackHeight(track.id, newHeight);
  }

  function handleResizeEnd() {
    isResizing = false;
    window.removeEventListener('mousemove', handleResizeMove);
    window.removeEventListener('mouseup', handleResizeEnd);
  }

  // Handle context menu - show simple browser context menu with track options
  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    // Create a simple dropdown menu at cursor position
    const options = [
      { label: 'Duplicate Track', action: () => duplicateTrack() },
      { label: 'Delete Track', action: () => handleRemoveTrack() },
      { label: track.muted ? 'Unmute' : 'Mute', action: () => handleMuteClick() },
      { label: track.solo ? 'Unsolo' : 'Solo', action: () => handleSoloClick() },
    ];
    // For now, just select the track on right-click to provide visual feedback
    sequencerActions.selectTrack(track.id, false);
    console.log('Track context menu:', track.id, 'Options:', options.map(o => o.label));
  }

  // Duplicate track functionality
  function duplicateTrack() {
    const newTrackId = sequencerActions.addTrack(`${track.name} (copy)`);
    if (newTrackId) {
      // Copy track settings
      sequencerActions.updateTrack(newTrackId, {
        color: track.color,
        midiChannel: track.midiChannel,
        height: track.height,
      });
      // Copy clips
      track.clips.forEach(clip => {
        sequencerActions.addClip(newTrackId, clip.startTick, clip.lengthTicks, clip.midiFileId, clip.name);
      });
    }
  }

  // Handle double-click on track area to add clip
  function handleTrackDoubleClick(event: MouseEvent) {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const x = event.clientX - rect.left + scrollX;
    const startTick = Math.floor(x / pxPerTick);
    const clipLength = ticksPerBeat * 4; // Default 1 bar
    sequencerActions.addClip(track.id, startTick, clipLength);
  }

  // Handle removing track
  function handleRemoveTrack() {
    if (confirm(`Remove track "${track.name}"?`)) {
      sequencerActions.removeTrack(track.id);
    }
  }
</script>

<div
  class="sequencer-track"
  class:muted={track.muted}
  class:solo={track.solo}
  class:armed={track.armed}
  class:collapsed={track.collapsed}
  style="height: {track.collapsed ? 24 : track.height}px; --track-type-color: {trackTypeColor}; --track-type-bg: {trackTypeBgColor}; --track-type-border: {trackTypeBorderColor};"
  on:contextmenu={handleContextMenu}
  role="listitem"
>
  <!-- Track Type Color Bar -->
  <div class="track-type-bar" style="background-color: {trackTypeColor};" />

  <!-- Track Header -->
  <div
    class="track-header"
    style="width: {headerWidth}px;"
    on:click={handleTrackClick}
    role="button"
    tabindex="0"
  >
    <!-- Collapse toggle -->
    <button
      class="collapse-btn"
      on:click|stopPropagation={handleCollapseClick}
      title={track.collapsed ? 'Expand track' : 'Collapse track'}
    >
      {track.collapsed ? '►' : '▼'}
    </button>

    <!-- Track type icon -->
    <TrackTypeIcon {trackType} size="sm" showBackground={false} />

    {#if !track.collapsed}
      <div class="track-info">
        <input
          type="text"
          class="track-name"
          value={track.name}
          on:change={handleNameChange}
          on:click|stopPropagation
        />
        <div class="track-routing">
          <span class="routing-label">Type:</span>
          <select
            class="type-select"
            value={trackType}
            on:change={handleTrackTypeChange}
            on:click|stopPropagation
          >
            {#each trackTypes as type (type)}
              <option value={type}>{type.charAt(0).toUpperCase() + type.slice(1)}</option>
            {/each}
          </select>
          <span class="routing-label">Ch:</span>
          <select
            class="channel-select"
            value={track.midiChannel}
            on:change={handleChannelChange}
            on:click|stopPropagation
          >
            {#each channels as ch (ch)}
              <option value={ch}>{ch}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Track controls -->
      <div class="track-controls">
        <button
          class="control-btn mute-btn"
          class:active={track.muted}
          on:click|stopPropagation={handleMuteClick}
          title="Mute (M)"
        >
          M
        </button>
        <button
          class="control-btn solo-btn"
          class:active={track.solo}
          on:click|stopPropagation={handleSoloClick}
          title="Solo (S)"
        >
          S
        </button>
        <button
          class="control-btn arm-btn"
          class:active={track.armed}
          on:click|stopPropagation={handleArmClick}
          title="Arm for recording"
        >
          R
        </button>
      </div>
    {:else}
      <span class="track-name-collapsed">{track.name}</span>
      <span class="clip-count">[{track.clips.length} clips]</span>
    {/if}
  </div>

  <!-- Track Clips Area -->
  <div
    class="track-clips"
    style="width: {contentWidth}px; transform: translateX(-{scrollX}px);"
    on:dblclick={handleTrackDoubleClick}
    role="region"
  >
    {#each track.clips as clip (clip.id)}
      <SequencerClip
        {clip}
        trackId={track.id}
        {pxPerTick}
        {ticksPerBeat}
        {snapValue}
        trackColor={track.color}
      />
    {/each}
  </div>

  <!-- Resize handle -->
  {#if !track.collapsed}
    <div
      class="resize-handle"
      on:mousedown={handleResizeStart}
      role="separator"
      aria-orientation="horizontal"
    />
  {/if}
</div>

<style>
  .sequencer-track {
    display: flex;
    position: relative;
    background: var(--window-bg, #1a1a1a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
    transition: height 0.15s ease;
  }

  .sequencer-track.muted {
    opacity: 0.6;
  }

  .sequencer-track.armed {
    background: rgba(239, 68, 68, 0.1);
  }

  /* Track Type Color Bar - MPC 3.0 Style */
  .track-type-bar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    z-index: 10;
  }

  .track-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px 4px 12px; /* Extra left padding for color bar */
    background: var(--track-type-bg, var(--menu-bg, #2a2a2a));
    border-right: 1px solid var(--border-color, #3a3a3a);
    flex-shrink: 0;
    cursor: pointer;
    transition: background 0.15s;
  }

  .track-header:hover {
    background: var(--hover-bg, #333);
  }

  .collapse-btn {
    width: 16px;
    height: 16px;
    padding: 0;
    background: none;
    border: none;
    color: var(--text-muted, #888);
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .collapse-btn:hover {
    color: var(--text-color, #e0e0e0);
  }

  .track-icon {
    font-size: 14px;
    width: 20px;
    text-align: center;
  }

  .track-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .track-name {
    background: transparent;
    border: none;
    color: var(--text-color, #e0e0e0);
    font-size: 12px;
    font-weight: 500;
    padding: 2px 4px;
    width: 100%;
    border-radius: 2px;
  }

  .track-name:focus {
    background: var(--input-bg, #333);
    outline: 1px solid var(--primary-color, #3b82f6);
  }

  .track-name-collapsed {
    flex: 1;
    font-size: 11px;
    color: var(--text-color, #e0e0e0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .clip-count {
    font-size: 10px;
    color: var(--text-muted, #888);
    margin-left: auto;
  }

  .track-routing {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .routing-label {
    font-size: 10px;
    color: var(--text-muted, #888);
  }

  .type-select,
  .channel-select {
    background: var(--input-bg, #333);
    border: 1px solid var(--border-color, #3a3a3a);
    color: var(--text-color, #e0e0e0);
    font-size: 10px;
    padding: 2px 4px;
    border-radius: 2px;
    cursor: pointer;
  }

  .type-select {
    min-width: 50px;
  }

  .track-controls {
    display: flex;
    gap: 2px;
    margin-left: auto;
  }

  .control-btn {
    width: 20px;
    height: 20px;
    padding: 0;
    background: var(--button-bg, #444);
    border: 1px solid var(--border-color, #555);
    border-radius: 3px;
    color: var(--text-muted, #888);
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .control-btn:hover {
    background: var(--button-hover, #555);
    color: var(--text-color, #e0e0e0);
  }

  .mute-btn.active {
    background: var(--error-color, #ef4444);
    color: white;
    border-color: var(--error-color, #ef4444);
  }

  .solo-btn.active {
    background: var(--warning-color, #f59e0b);
    color: white;
    border-color: var(--warning-color, #f59e0b);
  }

  .arm-btn.active {
    background: var(--error-color, #ef4444);
    color: white;
    border-color: var(--error-color, #ef4444);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .track-clips {
    flex: 1;
    position: relative;
    background: var(--window-subtle, #222);
    min-height: 100%;
    transition: transform 0.05s linear;
  }

  .sequencer-track.collapsed .track-clips {
    background: var(--menu-bg, #2a2a2a);
  }

  .resize-handle {
    position: absolute;
    left: 0;
    right: 0;
    bottom: -2px;
    height: 4px;
    cursor: ns-resize;
    z-index: 10;
  }

  .resize-handle:hover {
    background: var(--primary-color, #3b82f6);
    opacity: 0.5;
  }
</style>
