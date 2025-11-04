<script lang="ts">
  /**
   * PipelineWindow - MIDI file processing pipeline
   *
   * Task-O-Matic: Drag-drop import, batch processing, progress tracking.
   */

  import { writable } from 'svelte/store';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/api/dialog';

  // Processing state
  interface ProcessingState {
    isProcessing: boolean;
    isPaused: boolean;
    currentFile: string | null;
    processedCount: number;
    totalCount: number;
    errorCount: number;
    duplicateCount: number;
    progress: number;
    errors: string[];
    mode: 'quick' | 'standard' | 'analysis';
    workerCount: number;
  }

  const processingState = writable<ProcessingState>({
    isProcessing: false,
    isPaused: false,
    currentFile: null,
    processedCount: 0,
    totalCount: 0,
    errorCount: 0,
    duplicateCount: 0,
    progress: 0,
    errors: [],
    mode: 'standard',
    workerCount: 4
  });

  // Reactive state
  $: state = $processingState;
  $: canStart = !state.isProcessing && state.totalCount > 0;
  $: canPause = state.isProcessing && !state.isPaused;
  $: canResume = state.isProcessing && state.isPaused;
  $: canStop = state.isProcessing;

  // Drag and drop
  let isDragging = false;

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    if (!e.dataTransfer) return;

    const files = Array.from(e.dataTransfer.files);
    const paths = files.map(f => f.path);

    if (paths.length > 0) {
      await addFilesToQueue(paths);
    }
  }

  // File selection
  async function handleSelectFiles() {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'MIDI Files',
        extensions: ['mid', 'midi']
      }]
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await addFilesToQueue(paths);
    }
  }

  async function handleSelectFolder() {
    const selected = await open({
      directory: true,
      multiple: false
    });

    if (selected && typeof selected === 'string') {
      await addFilesToQueue([selected]);
    }
  }

  // Queue management
  async function addFilesToQueue(paths: string[]) {
    try {
      const fileCount = await invoke<number>('add_to_processing_queue', { paths });

      processingState.update(s => ({
        ...s,
        totalCount: s.totalCount + fileCount
      }));
    } catch (error) {
      console.error('Failed to add files:', error);
      processingState.update(s => ({
        ...s,
        errors: [...s.errors, `Failed to add files: ${error}`]
      }));
    }
  }

  // Processing controls
  async function handleStart() {
    try {
      await invoke('start_pipeline_processing', {
        mode: state.mode,
        workerCount: state.workerCount
      });

      processingState.update(s => ({
        ...s,
        isProcessing: true,
        isPaused: false,
        processedCount: 0,
        errorCount: 0,
        duplicateCount: 0,
        errors: []
      }));
    } catch (error) {
      console.error('Failed to start processing:', error);
      processingState.update(s => ({
        ...s,
        errors: [...s.errors, `Failed to start: ${error}`]
      }));
    }
  }

  async function handlePause() {
    try {
      await invoke('pause_pipeline_processing');
      processingState.update(s => ({ ...s, isPaused: true }));
    } catch (error) {
      console.error('Failed to pause processing:', error);
    }
  }

  async function handleResume() {
    try {
      await invoke('resume_pipeline_processing');
      processingState.update(s => ({ ...s, isPaused: false }));
    } catch (error) {
      console.error('Failed to resume processing:', error);
    }
  }

  async function handleStop() {
    try {
      await invoke('stop_pipeline_processing');
      processingState.update(s => ({
        ...s,
        isProcessing: false,
        isPaused: false,
        currentFile: null
      }));
    } catch (error) {
      console.error('Failed to stop processing:', error);
    }
  }

  function handleClearQueue() {
    processingState.update(s => ({
      ...s,
      totalCount: 0,
      processedCount: 0,
      errorCount: 0,
      duplicateCount: 0,
      errors: [],
      currentFile: null
    }));
  }

  // Mode and worker count
  function handleModeChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    processingState.update(s => ({
      ...s,
      mode: target.value as ProcessingState['mode']
    }));
  }

  function handleWorkerCountChange(e: Event) {
    const target = e.target as HTMLInputElement;
    processingState.update(s => ({
      ...s,
      workerCount: parseInt(target.value)
    }));
  }

  // Event listeners
  let unlistenProgress: (() => void) | null = null;
  let unlistenFile: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;

  async function setupEventListeners() {
    unlistenProgress = await listen<{ processed: number; total: number }>('pipeline-progress', (event) => {
      processingState.update(s => ({
        ...s,
        processedCount: event.payload.processed,
        totalCount: event.payload.total,
        progress: (event.payload.processed / event.payload.total) * 100
      }));
    });

    unlistenFile = await listen<{ filename: string }>('pipeline-file', (event) => {
      processingState.update(s => ({
        ...s,
        currentFile: event.payload.filename
      }));
    });

    unlistenError = await listen<{ error: string }>('pipeline-error', (event) => {
      processingState.update(s => ({
        ...s,
        errorCount: s.errorCount + 1,
        errors: [...s.errors, event.payload.error]
      }));
    });

    unlistenComplete = await listen('pipeline-complete', () => {
      processingState.update(s => ({
        ...s,
        isProcessing: false,
        isPaused: false,
        currentFile: null,
        progress: 100
      }));
    });
  }

  function cleanupEventListeners() {
    if (unlistenProgress) unlistenProgress();
    if (unlistenFile) unlistenFile();
    if (unlistenError) unlistenError();
    if (unlistenComplete) unlistenComplete();
  }

  // Lifecycle
  import { onMount, onDestroy } from 'svelte';

  onMount(() => {
    setupEventListeners();
  });

  onDestroy(() => {
    cleanupEventListeners();
  });
</script>

<WindowBase windowId="pipeline" title="Pipeline">
  <div class="pipeline-window">
    <!-- Drop Zone -->
    <div
      class="drop-zone"
      class:dragging={isDragging}
      on:dragover={handleDragOver}
      on:dragleave={handleDragLeave}
      on:drop={handleDrop}
      role="button"
      tabindex="0"
    >
      <div class="drop-zone-content">
        <div class="drop-icon">📁</div>
        <p class="drop-title">Drag & Drop Files or Folders</p>
        <p class="drop-subtitle">or</p>
        <div class="drop-buttons">
          <button class="select-btn" on:click={handleSelectFiles}>
            Select Files
          </button>
          <button class="select-btn" on:click={handleSelectFolder}>
            Select Folder
          </button>
        </div>
      </div>
    </div>

    <!-- Settings -->
    <div class="settings-panel">
      <div class="setting-group">
        <label class="setting-label">Processing Mode:</label>
        <select
          class="mode-select"
          value={state.mode}
          on:change={handleModeChange}
          disabled={state.isProcessing}
        >
          <option value="quick">Quick (Import Only)</option>
          <option value="standard">Standard (Import + Basic Analysis)</option>
          <option value="analysis">Full Analysis (BPM, Key, Tags)</option>
        </select>
      </div>

      <div class="setting-group">
        <label class="setting-label">Workers: {state.workerCount}</label>
        <input
          type="range"
          class="worker-slider"
          min="1"
          max="16"
          value={state.workerCount}
          on:input={handleWorkerCountChange}
          disabled={state.isProcessing}
        />
      </div>
    </div>

    <!-- Progress Section -->
    {#if state.totalCount > 0}
      <div class="progress-section">
        <div class="progress-header">
          <span class="progress-title">Processing Queue</span>
          {#if !state.isProcessing}
            <button class="clear-btn" on:click={handleClearQueue}>Clear</button>
          {/if}
        </div>

        <div class="progress-stats">
          <div class="stat">
            <span class="stat-label">Total:</span>
            <span class="stat-value">{state.totalCount}</span>
          </div>
          <div class="stat">
            <span class="stat-label">Processed:</span>
            <span class="stat-value">{state.processedCount}</span>
          </div>
          <div class="stat">
            <span class="stat-label">Errors:</span>
            <span class="stat-value error">{state.errorCount}</span>
          </div>
          <div class="stat">
            <span class="stat-label">Duplicates:</span>
            <span class="stat-value">{state.duplicateCount}</span>
          </div>
        </div>

        {#if state.isProcessing}
          <div class="progress-bar-container">
            <div class="progress-bar">
              <div
                class="progress-fill"
                style="width: {state.progress}%"
              />
            </div>
            <span class="progress-percent">{state.progress.toFixed(1)}%</span>
          </div>

          {#if state.currentFile}
            <div class="current-file">
              <span class="current-file-label">Processing:</span>
              <span class="current-file-name">{state.currentFile}</span>
            </div>
          {/if}

          {#if state.isPaused}
            <div class="paused-indicator">
              <span class="paused-icon">⏸</span>
              <span>Paused</span>
            </div>
          {/if}
        {/if}
      </div>
    {/if}

    <!-- Control Buttons -->
    <div class="controls">
      {#if !state.isProcessing}
        <button
          class="control-btn start"
          on:click={handleStart}
          disabled={!canStart}
        >
          ▶ Start Processing
        </button>
      {:else}
        {#if state.isPaused}
          <button class="control-btn resume" on:click={handleResume}>
            ▶ Resume
          </button>
        {:else}
          <button class="control-btn pause" on:click={handlePause}>
            ⏸ Pause
          </button>
        {/if}
        <button class="control-btn stop" on:click={handleStop}>
          ⏹ Stop
        </button>
      {/if}
    </div>

    <!-- Error Log -->
    {#if state.errors.length > 0}
      <div class="error-log">
        <div class="error-log-header">
          <span class="error-log-title">Errors ({state.errors.length})</span>
        </div>
        <div class="error-log-content">
          {#each state.errors as error, index}
            <div class="error-item">
              <span class="error-number">{index + 1}.</span>
              <span class="error-message">{error}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</WindowBase>

<style>
  .pipeline-window {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Drop Zone */
  .drop-zone {
    border: 2px dashed var(--border-color, #3e3e3e);
    border-radius: 12px;
    padding: 48px 24px;
    background: var(--dropzone-bg, #252525);
    transition: all 0.3s ease;
  }

  .drop-zone.dragging {
    border-color: var(--accent-color, #0078d4);
    background: rgba(0, 120, 212, 0.1);
  }

  .drop-zone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .drop-icon {
    font-size: 64px;
  }

  .drop-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .drop-subtitle {
    margin: 0;
    font-size: 14px;
    color: var(--text-secondary, #888);
  }

  .drop-buttons {
    display: flex;
    gap: 12px;
  }

  .select-btn {
    padding: 12px 24px;
    background: var(--accent-color, #0078d4);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.15s ease;
  }

  .select-btn:hover {
    background: var(--accent-hover, #0063b1);
    transform: translateY(-2px);
  }

  /* Settings Panel */
  .settings-panel {
    display: flex;
    gap: 24px;
    padding: 16px;
    background: var(--panel-bg, #252525);
    border-radius: 8px;
    border: 1px solid var(--border-color, #3e3e3e);
  }

  .setting-group {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .setting-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .mode-select {
    padding: 8px 12px;
    background: var(--input-bg, #1e1e1e);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
  }

  .mode-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .worker-slider {
    width: 100%;
    height: 6px;
    -webkit-appearance: none;
    background: var(--slider-bg, #3a3a3a);
    border-radius: 3px;
    outline: none;
  }

  .worker-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
  }

  .worker-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    background: var(--accent-color, #0078d4);
    border-radius: 50%;
    cursor: pointer;
    border: none;
  }

  .worker-slider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Progress Section */
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    background: var(--panel-bg, #252525);
    border-radius: 8px;
    border: 1px solid var(--border-color, #3e3e3e);
  }

  .progress-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .progress-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .clear-btn {
    padding: 4px 12px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .progress-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .stat-value {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
  }

  .stat-value.error {
    color: #e81123;
  }

  .progress-bar-container {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .progress-bar {
    flex: 1;
    height: 24px;
    background: var(--progress-bg, #1e1e1e);
    border-radius: 12px;
    overflow: hidden;
    border: 1px solid var(--border-color, #3e3e3e);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(
      90deg,
      var(--accent-color, #0078d4) 0%,
      var(--accent-hover, #0063b1) 100%
    );
    transition: width 0.3s ease;
  }

  .progress-percent {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    font-variant-numeric: tabular-nums;
    min-width: 50px;
    text-align: right;
  }

  .current-file {
    display: flex;
    gap: 8px;
    padding: 8px 12px;
    background: var(--file-bg, #1e1e1e);
    border-radius: 6px;
  }

  .current-file-label {
    font-size: 13px;
    color: var(--text-secondary, #888);
  }

  .current-file-name {
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .paused-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: rgba(247, 220, 111, 0.1);
    color: #f7dc6f;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
  }

  .paused-icon {
    font-size: 16px;
  }

  /* Controls */
  .controls {
    display: flex;
    gap: 12px;
  }

  .control-btn {
    flex: 1;
    padding: 16px 24px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 600;
    transition: all 0.15s ease;
  }

  .control-btn.start {
    background: var(--accent-color, #0078d4);
    color: white;
  }

  .control-btn.start:hover:not(:disabled) {
    background: var(--accent-hover, #0063b1);
    transform: translateY(-2px);
  }

  .control-btn.pause {
    background: #f7dc6f;
    color: #1e1e1e;
  }

  .control-btn.resume {
    background: #52b788;
    color: white;
  }

  .control-btn.stop {
    background: #e81123;
    color: white;
  }

  .control-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Error Log */
  .error-log {
    display: flex;
    flex-direction: column;
    max-height: 200px;
    background: var(--panel-bg, #252525);
    border-radius: 8px;
    border: 1px solid #e81123;
    overflow: hidden;
  }

  .error-log-header {
    padding: 12px 16px;
    background: rgba(232, 17, 35, 0.1);
    border-bottom: 1px solid #e81123;
  }

  .error-log-title {
    font-size: 14px;
    font-weight: 600;
    color: #e81123;
  }

  .error-log-content {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
  }

  .error-item {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 13px;
    color: var(--text-primary, #e0e0e0);
  }

  .error-number {
    color: #e81123;
    font-weight: 600;
  }

  .error-message {
    flex: 1;
    word-break: break-word;
  }

  /* Scrollbar */
  .error-log-content::-webkit-scrollbar {
    width: 8px;
  }

  .error-log-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .error-log-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 4px;
  }
</style>
