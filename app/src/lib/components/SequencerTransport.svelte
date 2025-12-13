<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    sequencerStore,
    sequencerActions,
    formattedPlayhead,
    isPlaying,
    isRecording,
    loopEnabled,
    currentBpm,
    currentTimeSignature,
    projectName,
    isProjectDirty,
    type SyncSource,
    type SnapValue,
  } from '$lib/stores/sequencerStore';

  // Reactive values from store
  $: state = $sequencerStore;
  $: playing = $isPlaying;
  $: recording = $isRecording;
  $: loop = $loopEnabled;
  $: bpm = $currentBpm;
  $: timeSig = $currentTimeSignature;
  $: position = $formattedPlayhead;
  $: name = $projectName;
  $: dirty = $isProjectDirty;

  // Local state for editing
  let editingBpm = false;
  let editBpmValue = bpm;

  // Snap value options
  const snapOptions: { value: SnapValue; label: string }[] = [
    { value: '1/1', label: '1 Bar' },
    { value: '1/2', label: '1/2' },
    { value: '1/4', label: '1/4' },
    { value: '1/8', label: '1/8' },
    { value: '1/16', label: '1/16' },
    { value: '1/32', label: '1/32' },
    { value: 'off', label: 'Off' },
  ];

  // Sync source options
  const syncOptions: { value: SyncSource; label: string }[] = [
    { value: 'internal', label: 'Internal' },
    { value: 'external', label: 'External' },
    { value: 'midi_clock', label: 'MIDI Clock' },
  ];

  // Time signature options
  const timeSignatures = ['4/4', '3/4', '6/8', '5/4', '7/8', '12/8'];

  // Transport button handlers
  function handleRewind() {
    sequencerActions.rewind();
  }

  function handleGotoStart() {
    sequencerActions.gotoStart();
  }

  function handlePlay() {
    if (playing) {
      void sequencerActions.pause();
    } else {
      void sequencerActions.play();
    }
  }

  function handleStop() {
    void sequencerActions.stop();
  }

  function handleForward() {
    sequencerActions.forward();
  }

  function handleGotoEnd() {
    sequencerActions.gotoEnd();
  }

  function handleRecord() {
    if (recording) {
      void sequencerActions.stop();
    } else {
      void sequencerActions.record();
    }
  }

  function handleLoopToggle() {
    sequencerActions.setLoopEnabled(!loop);
  }

  // BPM editing
  function handleBpmClick() {
    editingBpm = true;
    editBpmValue = bpm;
  }

  function handleBpmBlur() {
    editingBpm = false;
    if (editBpmValue !== bpm) {
      sequencerActions.setBpm(editBpmValue);
    }
  }

  function handleBpmKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      (event.target as HTMLInputElement).blur();
    } else if (event.key === 'Escape') {
      editBpmValue = bpm;
      editingBpm = false;
    }
  }

  // Tap tempo
  let tapTimes: number[] = [];
  function handleTapTempo() {
    const now = Date.now();
    tapTimes.push(now);

    // Keep only last 4 taps
    if (tapTimes.length > 4) {
      tapTimes = tapTimes.slice(-4);
    }

    // Calculate average BPM from tap intervals
    if (tapTimes.length >= 2) {
      let totalInterval = 0;
      for (let i = 1; i < tapTimes.length; i++) {
        totalInterval += tapTimes[i] - tapTimes[i - 1];
      }
      const avgInterval = totalInterval / (tapTimes.length - 1);
      const newBpm = Math.round(60000 / avgInterval);
      if (newBpm >= 20 && newBpm <= 300) {
        sequencerActions.setBpm(newBpm);
        editBpmValue = newBpm;
      }
    }

    // Reset if no tap for 2 seconds
    setTimeout(() => {
      const now2 = Date.now();
      if (tapTimes.length > 0 && now2 - tapTimes[tapTimes.length - 1] > 2000) {
        tapTimes = [];
      }
    }, 2000);
  }

  // Time signature change
  function handleTimeSignatureChange(event: Event) {
    const value = (event.target as HTMLSelectElement).value;
    const [num, denom] = value.split('/').map(Number);
    sequencerActions.setTimeSignature(num, denom);
  }

  // Snap value change
  function handleSnapChange(event: Event) {
    const value = (event.target as HTMLSelectElement).value as SnapValue;
    sequencerActions.setSnapValue(value);
  }

  // Sync source change
  function handleSyncChange(event: Event) {
    const value = (event.target as HTMLSelectElement).value as SyncSource;
    sequencerActions.setSyncSource(value);
  }

  // Project actions
  function handleNewProject() {
    if (dirty && !confirm('Unsaved changes will be lost. Continue?')) {
      return;
    }
    sequencerActions.newProject();
  }

  function handleSaveProject() {
    void sequencerActions.saveProject();
  }

  async function handleOpenProject() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'MIDI Project', extensions: ['mproj', 'json'] }]
    });
    if (selected && typeof selected === 'string') {
      await sequencerActions.loadProjectFromFile(selected);
    }
  }

  // Calculate time from position
  $: projectLengthSeconds =
    (state.project.lengthBars * timeSig[0] * 60) / bpm;
  $: currentTimeSeconds = (state.playhead.ticks / state.ticksPerBeat / bpm) * 60;

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  }
</script>

<div class="transport-bar">
  <!-- Project info -->
  <div class="project-section">
    <span class="project-name" title={name}>
      {name}{dirty ? '*' : ''}
    </span>
    <div class="project-buttons">
      <button
        class="project-btn"
        on:click={handleNewProject}
        title="New Project (Cmd+N)"
      >
        New
      </button>
      <button
        class="project-btn"
        on:click={handleOpenProject}
        title="Open Project (Cmd+O)"
      >
        Open
      </button>
      <button
        class="project-btn save-btn"
        on:click={handleSaveProject}
        title="Save Project (Cmd+S)"
        class:dirty
      >
        Save
      </button>
    </div>
  </div>

  <!-- Transport controls -->
  <div class="transport-section">
    <button
      class="transport-btn"
      on:click={handleGotoStart}
      title="Go to Start (Home)"
    >
      <span class="icon">⏮</span>
    </button>
    <button
      class="transport-btn"
      on:click={handleRewind}
      title="Rewind (,)"
    >
      <span class="icon">⏪</span>
    </button>
    <button
      class="transport-btn play-btn"
      class:playing
      on:click={handlePlay}
      title="Play/Pause (Space)"
    >
      <span class="icon">{playing ? '⏸' : '▶'}</span>
    </button>
    <button
      class="transport-btn stop-btn"
      on:click={handleStop}
      title="Stop (Enter)"
    >
      <span class="icon">⏹</span>
    </button>
    <button
      class="transport-btn"
      on:click={handleForward}
      title="Forward (.)"
    >
      <span class="icon">⏩</span>
    </button>
    <button
      class="transport-btn"
      on:click={handleGotoEnd}
      title="Go to End (End)"
    >
      <span class="icon">⏭</span>
    </button>

    <div class="transport-divider" />

    <button
      class="transport-btn record-btn"
      class:recording
      on:click={handleRecord}
      title="Record (R)"
    >
      <span class="icon">●</span>
      REC
    </button>
    <button
      class="transport-btn loop-btn"
      class:active={loop}
      on:click={handleLoopToggle}
      title="Toggle Loop (L)"
    >
      <span class="icon">↺</span>
      LOOP
    </button>
  </div>

  <!-- Position display -->
  <div class="position-section">
    <div class="position-display">
      <span class="position-label">POS:</span>
      <span class="position-value">{position}</span>
    </div>
    <div class="time-display">
      <span class="time-current">{formatTime(currentTimeSeconds)}</span>
      <span class="time-separator">/</span>
      <span class="time-total">{formatTime(projectLengthSeconds)}</span>
    </div>
  </div>

  <!-- Tempo section -->
  <div class="tempo-section">
    <div class="tempo-control">
      <span class="tempo-label">BPM:</span>
      {#if editingBpm}
        <input
          type="number"
          class="tempo-input"
          bind:value={editBpmValue}
          min="20"
          max="300"
          step="0.1"
          on:blur={handleBpmBlur}
          on:keydown={handleBpmKeydown}
          autofocus
        />
      {:else}
        <button
          class="tempo-value"
          on:click={handleBpmClick}
          title="Click to edit BPM"
        >
          {bpm.toFixed(1)}
        </button>
      {/if}
      <button
        class="tap-btn"
        on:click={handleTapTempo}
        title="Tap Tempo"
      >
        TAP
      </button>
    </div>

    <div class="time-sig-control">
      <span class="control-label">TIME:</span>
      <select
        class="time-sig-select"
        value="{timeSig[0]}/{timeSig[1]}"
        on:change={handleTimeSignatureChange}
      >
        {#each timeSignatures as ts (ts)}
          <option value={ts}>{ts}</option>
        {/each}
      </select>
    </div>
  </div>

  <!-- Grid & Sync section -->
  <div class="grid-section">
    <div class="snap-control">
      <span class="control-label">SNAP:</span>
      <select
        class="snap-select"
        value={state.snapValue}
        on:change={handleSnapChange}
      >
        {#each snapOptions as opt (opt.value)}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <div class="sync-control">
      <span class="control-label">SYNC:</span>
      <select
        class="sync-select"
        value={state.project.syncSource}
        on:change={handleSyncChange}
      >
        {#each syncOptions as opt (opt.value)}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
  </div>
</div>

<style>
  .transport-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 16px;
    background: var(--menu-bg, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #3a3a3a);
    flex-shrink: 0;
  }

  /* Project section */
  .project-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 120px;
  }

  .project-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-color, #e0e0e0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-buttons {
    display: flex;
    gap: 4px;
  }

  .project-btn {
    padding: 2px 8px;
    font-size: 10px;
    background: var(--button-bg, #444);
    border: 1px solid var(--border-color, #555);
    border-radius: 3px;
    color: var(--text-color, #e0e0e0);
    cursor: pointer;
  }

  .project-btn:hover {
    background: var(--button-hover, #555);
  }

  .save-btn.dirty {
    background: var(--primary-color, #3b82f6);
    border-color: var(--primary-color, #3b82f6);
    color: white;
  }

  /* Transport section */
  .transport-section {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--window-bg, #1a1a1a);
    border-radius: 6px;
  }

  .transport-btn {
    width: 32px;
    height: 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    background: var(--button-bg, #444);
    border: 1px solid var(--border-color, #555);
    border-radius: 4px;
    color: var(--text-color, #e0e0e0);
    font-size: 9px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .transport-btn .icon {
    font-size: 14px;
  }

  .transport-btn:hover {
    background: var(--button-hover, #555);
  }

  .play-btn.playing {
    background: var(--success-color, #22c55e);
    border-color: var(--success-color, #22c55e);
    color: white;
  }

  .stop-btn:hover {
    background: var(--error-color, #ef4444);
    border-color: var(--error-color, #ef4444);
    color: white;
  }

  .record-btn {
    width: auto;
    padding: 0 8px;
    gap: 4px;
  }

  .record-btn .icon {
    color: var(--error-color, #ef4444);
  }

  .record-btn.recording {
    background: var(--error-color, #ef4444);
    border-color: var(--error-color, #ef4444);
    color: white;
    animation: pulse 1s ease-in-out infinite;
  }

  .record-btn.recording .icon {
    color: white;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .loop-btn {
    width: auto;
    padding: 0 8px;
  }

  .loop-btn.active {
    background: var(--primary-color, #3b82f6);
    border-color: var(--primary-color, #3b82f6);
    color: white;
  }

  .transport-divider {
    width: 1px;
    height: 24px;
    background: var(--border-color, #3a3a3a);
    margin: 0 4px;
  }

  /* Position section */
  .position-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 4px 12px;
    background: var(--window-bg, #1a1a1a);
    border-radius: 4px;
    min-width: 100px;
  }

  .position-display {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .position-label {
    font-size: 9px;
    color: var(--text-muted, #888);
  }

  .position-value {
    font-family: 'JetBrains Mono', monospace;
    font-size: 14px;
    font-weight: 600;
    color: var(--success-color, #22c55e);
  }

  .time-display {
    font-family: 'JetBrains Mono', monospace;
    font-size: 10px;
    color: var(--text-muted, #888);
  }

  .time-separator {
    margin: 0 2px;
  }

  /* Tempo section */
  .tempo-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tempo-control {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tempo-label,
  .control-label {
    font-size: 10px;
    color: var(--text-muted, #888);
    min-width: 32px;
  }

  .tempo-value {
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-color, #e0e0e0);
    background: var(--input-bg, #333);
    border: 1px solid var(--border-color, #3a3a3a);
    padding: 2px 8px;
    border-radius: 3px;
    cursor: pointer;
    min-width: 60px;
    text-align: center;
  }

  .tempo-value:hover {
    border-color: var(--primary-color, #3b82f6);
  }

  .tempo-input {
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    width: 60px;
    padding: 2px 8px;
    background: var(--input-bg, #333);
    border: 1px solid var(--primary-color, #3b82f6);
    border-radius: 3px;
    color: var(--text-color, #e0e0e0);
    text-align: center;
  }

  .tap-btn {
    padding: 2px 8px;
    font-size: 9px;
    font-weight: 600;
    background: var(--button-bg, #444);
    border: 1px solid var(--border-color, #555);
    border-radius: 3px;
    color: var(--text-color, #e0e0e0);
    cursor: pointer;
  }

  .tap-btn:hover {
    background: var(--primary-color, #3b82f6);
    border-color: var(--primary-color, #3b82f6);
    color: white;
  }

  .tap-btn:active {
    transform: scale(0.95);
  }

  .time-sig-control {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .time-sig-select {
    font-size: 11px;
    padding: 2px 4px;
    background: var(--input-bg, #333);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 3px;
    color: var(--text-color, #e0e0e0);
    cursor: pointer;
  }

  /* Grid section */
  .grid-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .snap-control,
  .sync-control {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .snap-select,
  .sync-select {
    font-size: 11px;
    padding: 2px 4px;
    background: var(--input-bg, #333);
    border: 1px solid var(--border-color, #3a3a3a);
    border-radius: 3px;
    color: var(--text-color, #e0e0e0);
    cursor: pointer;
    min-width: 70px;
  }

  select:hover {
    border-color: var(--primary-color, #3b82f6);
  }
</style>
