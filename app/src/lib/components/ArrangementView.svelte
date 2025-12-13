<script lang="ts">
  /**
   * ArrangementView - MPC 3.0 Style Arrangement Wrapper
   *
   * Wraps the existing Sequencer component and adds:
   * - Track type detection and coloring
   * - Integration with arrangementStore for clip editing
   * - Enhanced styling with MPC 3.0 visual design
   * - Split view support for piano roll/automation
   */

  import { onMount, createEventDispatcher } from 'svelte';
  import Sequencer from './Sequencer.svelte';
  import {
    sequencerStore,
    sequencerActions,
    type SequencerTrack,
    type SequencerClip,
  } from '$lib/stores/sequencerStore';
  import {
    arrangementStore,
    arrangementActions,
    type ActiveEditClip,
  } from '$lib/stores/arrangementStore';
  import type { SequencerTrackType } from '$lib/types';
  import { detectTrackType, getTrackTypeCSSVars } from '$lib/utils/trackColors';

  const dispatch = createEventDispatcher<{
    clipEdit: ActiveEditClip;
    trackTypeChange: { trackId: number; trackType: SequencerTrackType };
  }>();

  // Props
  export let showTransport: boolean = true;
  export let showSplitView: boolean = false;

  // Store subscriptions
  $: state = $sequencerStore;
  $: tracks = state.tracks;
  $: arrangement = $arrangementStore;
  $: activeEditClip = arrangement.activeEditClip;
  $: trackTypes = arrangement.trackTypes;
  $: splitViewEnabled = arrangement.splitViewEnabled;
  $: splitViewHeight = arrangement.splitViewHeight;
  $: splitViewContent = arrangement.splitViewContent;

  // Detect track types for any tracks that don't have a type assigned
  $: {
    tracks.forEach((track) => {
      if (!trackTypes[track.id]) {
        const detectedType = detectTrackType({
          name: track.name,
          midiChannel: track.midiChannel,
          hasDrums: track.midiChannel === 10,  // Channel 10 is conventionally drums
        });
        arrangementActions.setTrackType(track.id, detectedType);
      }
    });
  }

  // Apply CSS variables for track colors
  $: trackStyleVars = Object.fromEntries(
    Object.entries(trackTypes).map(([id, type]) => [
      `--track-${id}-color`,
      getTrackTypeCSSVars(type),
    ])
  );

  // Handle double-click on clip to open piano roll
  function handleClipEdit(clip: SequencerClip, track: SequencerTrack) {
    const editClip: ActiveEditClip = {
      clipId: clip.id,
      trackId: track.id,
      fileId: clip.midiFileId,
      clipName: clip.name,
    };

    arrangementActions.setActiveEditClip(editClip);
    dispatch('clipEdit', editClip);

    // If split view isn't showing piano roll, enable it
    if (!splitViewEnabled || splitViewContent !== 'piano-roll') {
      arrangementActions.setSplitViewContent('piano-roll');
    }
  }

  // Handle track type change (user override)
  function handleTrackTypeChange(trackId: number, newType: SequencerTrackType) {
    arrangementActions.setTrackType(trackId, newType);
    dispatch('trackTypeChange', { trackId, trackType: newType });
  }

  // Create a demo project with sample tracks if empty
  function handleCreateDemoProject() {
    if (tracks.length === 0) {
      // Add demo tracks
      const drumTrackId = sequencerActions.addTrack('Drums');
      const bassTrackId = sequencerActions.addTrack('Bass');
      const leadsTrackId = sequencerActions.addTrack('Leads');

      // Set track types
      arrangementActions.setTrackTypes({
        [drumTrackId]: 'drum',
        [bassTrackId]: 'midi',
        [leadsTrackId]: 'midi',
      });

      // Add sample clips
      const ticksPerBar = state.ticksPerBeat * state.project.timeSignature[0];
      sequencerActions.addClip(drumTrackId, 0, ticksPerBar * 4, null, 'Drum Loop');
      sequencerActions.addClip(bassTrackId, ticksPerBar, ticksPerBar * 3, null, 'Bass Line');
      sequencerActions.addClip(leadsTrackId, ticksPerBar * 2, ticksPerBar * 2, null, 'Lead Melody');
    }
  }

  onMount(() => {
    // Setup global event listener for clip edits
    const handleGlobalClipEdit = (event: CustomEvent) => {
      const { clip, track } = event.detail;
      handleClipEdit(clip, track);
    };

    window.addEventListener('sequencer:clipEdit', handleGlobalClipEdit as EventListener);

    return () => {
      window.removeEventListener('sequencer:clipEdit', handleGlobalClipEdit as EventListener);
    };
  });
</script>

<div
  class="arrangement-view"
  class:has-split-view={splitViewEnabled && showSplitView}
  style:--split-view-height="{splitViewHeight}%"
>
  <!-- Main Arrangement Area -->
  <div class="arrangement-main">
    {#if tracks.length === 0}
      <!-- Empty State -->
      <div class="empty-state">
        <div class="empty-state-icon">🎹</div>
        <h3 class="empty-state-title">No Tracks Yet</h3>
        <p class="empty-state-text">
          Drag MIDI files from the browser or create tracks to get started.
        </p>
        <div class="empty-state-actions">
          <button class="action-btn primary" on:click={() => sequencerActions.addTrack('Track 1')}>
            + New Track
          </button>
          <button class="action-btn secondary" on:click={handleCreateDemoProject}>
            Create Demo Project
          </button>
        </div>
      </div>
    {:else}
      <!-- Sequencer with track type styling -->
      <Sequencer />
    {/if}
  </div>

  <!-- Split View Panel (Piano Roll / Automation / Velocity) -->
  {#if splitViewEnabled && showSplitView}
    <div class="split-view-panel">
      <div class="split-view-header">
        <div class="split-view-tabs">
          <button
            class="split-tab"
            class:active={splitViewContent === 'piano-roll'}
            on:click={() => arrangementActions.setSplitViewContent('piano-roll')}
          >
            Piano Roll
          </button>
          <button
            class="split-tab"
            class:active={splitViewContent === 'automation'}
            on:click={() => arrangementActions.setSplitViewContent('automation')}
          >
            Automation
          </button>
          <button
            class="split-tab"
            class:active={splitViewContent === 'velocity'}
            on:click={() => arrangementActions.setSplitViewContent('velocity')}
          >
            Velocity
          </button>
        </div>
        <button class="split-view-close" on:click={() => arrangementActions.setSplitViewContent(null)}>
          ×
        </button>
      </div>
      <div class="split-view-content">
        {#if activeEditClip}
          {#if splitViewContent === 'piano-roll'}
            <div class="placeholder-content">
              <p>Piano Roll: Editing "{activeEditClip.clipName}"</p>
              <span class="hint">Use the Piano Roll window for full editing</span>
            </div>
          {:else if splitViewContent === 'automation'}
            <div class="placeholder-content">
              <p>Automation Lanes</p>
              <span class="hint">Draw automation curves for selected parameters</span>
            </div>
          {:else if splitViewContent === 'velocity'}
            <div class="placeholder-content">
              <p>Velocity Editor</p>
              <span class="hint">Adjust note velocities for "{activeEditClip.clipName}"</span>
            </div>
          {/if}
        {:else}
          <div class="placeholder-content">
            <p>Select a clip to edit</p>
            <span class="hint">Double-click a clip to start editing</span>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .arrangement-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1a1a1a);
    overflow: hidden;
  }

  .arrangement-main {
    flex: 1;
    overflow: hidden;
    min-height: 200px;
  }

  .arrangement-view.has-split-view .arrangement-main {
    flex: 1 1 auto;
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 40px;
    text-align: center;
  }

  .empty-state-icon {
    font-size: 64px;
    margin-bottom: 20px;
    opacity: 0.5;
  }

  .empty-state-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--text-color, #e0e0e0);
    margin: 0 0 8px;
  }

  .empty-state-text {
    font-size: 14px;
    color: var(--text-muted, #888);
    margin: 0 0 24px;
    max-width: 400px;
  }

  .empty-state-actions {
    display: flex;
    gap: 12px;
  }

  .action-btn {
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .action-btn.primary {
    background: var(--primary-color, #3b82f6);
    color: white;
  }

  .action-btn.primary:hover {
    background: var(--primary-dark, #2563eb);
  }

  .action-btn.secondary {
    background: var(--button-bg, #333);
    color: var(--text-color, #e0e0e0);
    border: 1px solid var(--border-color, #444);
  }

  .action-btn.secondary:hover {
    background: var(--button-hover, #444);
  }

  /* Split View Panel */
  .split-view-panel {
    height: var(--split-view-height, 30%);
    min-height: 150px;
    max-height: 50%;
    border-top: 3px solid var(--border-color, #3a3a3a);
    background: var(--window-bg, #1a1a1a);
    display: flex;
    flex-direction: column;
  }

  .split-view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 36px;
    background: var(--menu-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
    flex-shrink: 0;
  }

  .split-view-tabs {
    display: flex;
    gap: 4px;
  }

  .split-tab {
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted, #888);
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .split-tab:hover {
    color: var(--text-color, #e0e0e0);
    background: var(--button-hover, #333);
  }

  .split-tab.active {
    color: var(--primary-color, #3b82f6);
    background: rgba(59, 130, 246, 0.15);
  }

  .split-view-close {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    color: var(--text-muted, #888);
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .split-view-close:hover {
    color: var(--text-color, #e0e0e0);
    background: var(--button-hover, #333);
  }

  .split-view-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .placeholder-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted, #666);
  }

  .placeholder-content p {
    font-size: 14px;
    margin: 0 0 4px;
  }

  .placeholder-content .hint {
    font-size: 12px;
    opacity: 0.7;
  }
</style>
