<script lang="ts">
  /**
   * ArrangementWindow - MPC 3.0 Style Main Window
   *
   * The primary window of the application, featuring:
   * - Linear arrangement timeline (MPC 3.0 style)
   * - Color-coded tracks by type (MIDI/Drum/Audio)
   * - Transport controls
   * - Integrated split view for piano roll/automation
   * - Toolbar with view options and actions
   *
   * This window is designed to be the main view, always visible,
   * with Mixer and Piano Roll as detachable companions.
   */

  import WindowBase from '$lib/components/WindowBase.svelte';
  import ArrangementView from '$lib/components/ArrangementView.svelte';
  import {
    sequencerStore,
    sequencerActions,
  } from '$lib/stores/sequencerStore';
  import {
    arrangementStore,
    arrangementActions,
    hasDetachedWindows,
    type ActiveEditClip,
  } from '$lib/stores/arrangementStore';
  import { uiActions } from '$lib/stores/uiStore';
  import type { SequencerTrackType } from '$lib/types';
  import TrackTypeIcon from '$lib/components/TrackTypeIcon.svelte';

  // Store subscriptions
  $: state = $sequencerStore;
  $: arrangement = $arrangementStore;
  $: isPlaying = state.isPlaying;
  $: isRecording = state.isRecording;
  $: mixerDetached = arrangement.mixerDetached;
  $: pianoRollDetached = arrangement.pianoRollDetached;
  $: showTrackTypeIcons = arrangement.showTrackTypeIcons;
  $: showClipNames = arrangement.showClipNames;
  $: showClipPreviews = arrangement.showClipPreviews;
  $: trackColorMode = arrangement.trackColorMode;
  $: splitViewEnabled = arrangement.splitViewEnabled;

  // View menu open state
  let viewMenuOpen = false;

  // Handle clip edit event
  function handleClipEdit(event: CustomEvent<ActiveEditClip>) {
    const clip = event.detail;
    console.log('Opening clip for editing:', clip);

    // If piano roll is detached, open it as a window
    if (pianoRollDetached) {
      uiActions.openWindow('piano-roll');
    }
  }

  // Handle track type change
  function handleTrackTypeChange(
    event: CustomEvent<{ trackId: number; trackType: SequencerTrackType }>
  ) {
    console.log('Track type changed:', event.detail);
  }

  // Toggle view menu
  function toggleViewMenu() {
    viewMenuOpen = !viewMenuOpen;
  }

  // Close view menu when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.view-menu-container')) {
      viewMenuOpen = false;
    }
  }

  // Open/toggle mixer window
  function handleToggleMixer() {
    if (mixerDetached) {
      uiActions.toggleWindow('mixer');
    } else {
      // Dock mixer at bottom (future implementation)
      arrangementActions.toggleMixerDetached();
      uiActions.openWindow('mixer');
    }
  }

  // Open/toggle piano roll window
  function handleTogglePianoRoll() {
    if (pianoRollDetached) {
      uiActions.toggleWindow('piano-roll');
    } else {
      // Show in split view
      arrangementActions.toggleSplitView();
    }
  }

  // Keyboard shortcuts for the window
  function handleKeyDown(event: KeyboardEvent) {
    // Don't handle if typing in an input
    if ((event.target as HTMLElement).tagName === 'INPUT') return;

    // Check for modifier keys
    const isModifier = event.ctrlKey || event.metaKey;

    switch (event.key.toLowerCase()) {
      case 'm':
        if (isModifier) {
          event.preventDefault();
          handleToggleMixer();
        }
        break;
      case 'p':
        if (isModifier) {
          event.preventDefault();
          handleTogglePianoRoll();
        }
        break;
      case 'e':
        if (isModifier) {
          event.preventDefault();
          arrangementActions.toggleSplitView();
        }
        break;
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} on:click={handleClickOutside} />

<WindowBase
  title="Arrangement"
  windowId="arrangement"
  width={1200}
  height={700}
  minWidth={600}
  minHeight={400}
  maximizable={true}
  closable={false}
>
  <div class="arrangement-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <!-- Left: View Options -->
      <div class="toolbar-section toolbar-left">
        <div class="view-menu-container">
          <button
            class="toolbar-btn"
            class:active={viewMenuOpen}
            on:click={toggleViewMenu}
            title="View Options"
          >
            <span class="icon">👁</span>
            <span class="label">View</span>
            <span class="chevron">▼</span>
          </button>

          {#if viewMenuOpen}
            <div class="dropdown-menu">
              <label class="menu-item checkbox">
                <input
                  type="checkbox"
                  checked={showTrackTypeIcons}
                  on:change={() => arrangementActions.toggleTrackTypeIcons()}
                />
                <span>Track Type Icons</span>
              </label>
              <label class="menu-item checkbox">
                <input
                  type="checkbox"
                  checked={showClipNames}
                  on:change={() => arrangementActions.toggleClipNames()}
                />
                <span>Clip Names</span>
              </label>
              <label class="menu-item checkbox">
                <input
                  type="checkbox"
                  checked={showClipPreviews}
                  on:change={() => arrangementActions.toggleClipPreviews()}
                />
                <span>Clip Previews</span>
              </label>
              <div class="menu-divider" />
              <button
                class="menu-item"
                on:click={() => {
                  arrangementActions.setTrackColorMode(trackColorMode === 'type' ? 'custom' : 'type');
                  viewMenuOpen = false;
                }}
              >
                <span>Color Mode: {trackColorMode === 'type' ? 'By Type' : 'Custom'}</span>
              </button>
            </div>
          {/if}
        </div>

        <!-- Track Type Legend -->
        <div class="track-type-legend">
          <div class="legend-item">
            <TrackTypeIcon trackType="midi" size="sm" showTooltip={true} />
            <span>MIDI</span>
          </div>
          <div class="legend-item">
            <TrackTypeIcon trackType="drum" size="sm" showTooltip={true} />
            <span>Drum</span>
          </div>
          <div class="legend-item">
            <TrackTypeIcon trackType="audio" size="sm" showTooltip={true} />
            <span>Audio</span>
          </div>
        </div>
      </div>

      <!-- Center: Status -->
      <div class="toolbar-section toolbar-center">
        {#if isRecording}
          <span class="status-indicator recording">● Recording</span>
        {:else if isPlaying}
          <span class="status-indicator playing">▶ Playing</span>
        {:else}
          <span class="status-indicator stopped">⏹ Stopped</span>
        {/if}
      </div>

      <!-- Right: Window Toggle Buttons -->
      <div class="toolbar-section toolbar-right">
        <button
          class="toolbar-btn"
          class:active={!pianoRollDetached && splitViewEnabled}
          on:click={handleTogglePianoRoll}
          title="Toggle Piano Roll (Ctrl+P)"
        >
          <span class="icon">🎹</span>
          <span class="label">Piano Roll</span>
        </button>

        <button
          class="toolbar-btn"
          class:active={mixerDetached}
          on:click={handleToggleMixer}
          title="Toggle Mixer (Ctrl+M)"
        >
          <span class="icon">🎚</span>
          <span class="label">Mixer</span>
        </button>

        <button
          class="toolbar-btn"
          on:click={() => uiActions.toggleWindow('database')}
          title="Toggle Database Browser"
        >
          <span class="icon">📁</span>
          <span class="label">Browser</span>
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="main-content">
      <ArrangementView
        showTransport={true}
        showSplitView={true}
        on:clipEdit={handleClipEdit}
        on:trackTypeChange={handleTrackTypeChange}
      />
    </div>
  </div>
</WindowBase>

<style>
  .arrangement-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1a1a1a);
    overflow: hidden;
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--menu-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
    gap: 12px;
    flex-shrink: 0;
  }

  .toolbar-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-left {
    flex: 1;
    justify-content: flex-start;
  }

  .toolbar-center {
    flex: 0 0 auto;
  }

  .toolbar-right {
    flex: 1;
    justify-content: flex-end;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--button-bg, #333);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    color: var(--text-color, #e0e0e0);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .toolbar-btn:hover {
    background: var(--button-hover, #444);
  }

  .toolbar-btn.active {
    background: var(--primary-color, #3b82f6);
    border-color: var(--primary-color, #3b82f6);
    color: white;
  }

  .toolbar-btn .icon {
    font-size: 14px;
  }

  .toolbar-btn .label {
    font-weight: 500;
  }

  .toolbar-btn .chevron {
    font-size: 8px;
    opacity: 0.7;
  }

  /* View Menu Dropdown */
  .view-menu-container {
    position: relative;
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    min-width: 180px;
    background: var(--menu-bg, #2a2a2a);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    padding: 4px 0;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    color: var(--text-color, #e0e0e0);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.15s;
    border: none;
    background: transparent;
    width: 100%;
    text-align: left;
  }

  .menu-item:hover {
    background: var(--button-hover, #333);
  }

  .menu-item.checkbox {
    gap: 8px;
  }

  .menu-item input[type='checkbox'] {
    accent-color: var(--primary-color, #3b82f6);
  }

  .menu-divider {
    height: 1px;
    background: var(--border-color, #3a3a3a);
    margin: 4px 0;
  }

  /* Track Type Legend */
  .track-type-legend {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: 16px;
    padding-left: 16px;
    border-left: 1px solid var(--border-color, #3a3a3a);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  /* Status Indicator */
  .status-indicator {
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .status-indicator.recording {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    animation: pulse 1s infinite;
  }

  .status-indicator.playing {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
  }

  .status-indicator.stopped {
    background: rgba(100, 100, 100, 0.2);
    color: var(--text-muted, #888);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  /* Main Content */
  .main-content {
    flex: 1;
    overflow: hidden;
  }
</style>
