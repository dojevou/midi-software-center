<script lang="ts">
  // ARCHETYPE: CONTAINER (Task-O-Matic)
  // Purpose: Display real-time import progress from backend events
  // Listens to: "import-progress" Tauri events

  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  // TypeScript interface for progress event payload
  interface ProgressState {
    current_file: string;
    current_index: number;
    total_files: number;
    percentage: number;
    phase: string;
    files_per_second: number;
    errors_count: number;
    duplicates_found: number;
    estimated_time_remaining: number;
  }

  // Component state
  let progressData: ProgressState = {
    current_file: '',
    current_index: 0,
    total_files: 0,
    percentage: 0,
    phase: 'idle',
    files_per_second: 0,
    errors_count: 0,
    duplicates_found: 0,
    estimated_time_remaining: 0
  };

  let unlistenFn: UnlistenFn | null = null;
  let isActive = false;

  // Reactive values
  $: remainingFiles = progressData.total_files - progressData.current_index;
  $: isPaused = isActive && progressData.phase === 'paused';
  $: isComplete = progressData.phase === 'complete';
  $: isIdle = progressData.phase === 'idle';

  // Format time in seconds to human-readable string
  function formatTime(seconds: number): string {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    if (seconds < 3600) {
      const mins = Math.floor(seconds / 60);
      const secs = Math.round(seconds % 60);
      return `${mins}m ${secs}s`;
    }
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }

  // Get display label for phase
  function getPhaseLabel(phase: string): string {
    const labels: Record<string, string> = {
      'idle': '⏸️ Idle',
      'scanning': '🔍 Scanning Files',
      'decompressing': '📦 Decompressing Archives',
      'analyzing': '🎵 Analyzing MIDI',
      'writing': '💾 Writing to Database',
      'complete': '✅ Complete',
      'paused': '⏸️ Paused'
    };
    return labels[phase] || phase;
  }

  // Get color for phase
  function getPhaseColor(phase: string): string {
    const colors: Record<string, string> = {
      'idle': '#808080',
      'scanning': '#4a9eff',
      'decompressing': '#ff9800',
      'analyzing': '#9c27b0',
      'writing': '#4caf50',
      'complete': '#4caf50',
      'paused': '#ff9800'
    };
    return colors[phase] || '#4a9eff';
  }

  // Set up event listener on mount
  onMount(async () => {
    try {
      unlistenFn = await listen<ProgressState>('import-progress', (event) => {
        progressData = event.payload;

        // Set active state based on phase
        if (progressData.phase !== 'idle' && progressData.phase !== 'complete') {
          isActive = true;
        } else if (progressData.phase === 'complete') {
          isActive = false;
        }
      });
    } catch (error) {
      console.error('Failed to set up progress listener:', error);
    }
  });

  // Clean up event listener on destroy
  onDestroy(() => {
    if (unlistenFn) {
      unlistenFn();
    }
  });
</script>

<div class="progress-monitor" class:active={isActive} class:paused={isPaused} class:complete={isComplete}>
  {#if isIdle && !isComplete}
    <!-- Idle State -->
    <div class="idle-state">
      <div class="idle-icon">⏱️</div>
      <p>No import in progress</p>
      <p class="idle-hint">Start an import to see progress here</p>
    </div>
  {:else}
    <!-- Active/Complete State -->
    {#if isPaused}
      <div class="paused-banner">
        <span class="pause-icon">⏸️</span>
        <span>Import Paused</span>
      </div>
    {/if}

    {#if isComplete}
      <div class="complete-banner">
        <span class="complete-icon">✅</span>
        <span>Import Complete!</span>
      </div>
    {/if}

    <!-- Progress Bar Section -->
    <div class="progress-section">
      <div class="progress-header">
        <span class="phase-label" style="color: {getPhaseColor(progressData.phase)}">
          {getPhaseLabel(progressData.phase)}
        </span>
        <span class="progress-percentage">{progressData.percentage.toFixed(1)}%</span>
      </div>

      <div class="progress-bar">
        <div
          class="progress-fill"
          style="width: {progressData.percentage}%; background: {getPhaseColor(progressData.phase)}"
        ></div>
      </div>

      <div class="progress-stats">
        <span>{progressData.current_index.toLocaleString()} / {progressData.total_files.toLocaleString()} files</span>
        <span>{remainingFiles.toLocaleString()} remaining</span>
      </div>
    </div>

    <!-- Current File -->
    {#if progressData.current_file && !isComplete}
      <div class="current-file">
        <div class="file-label">Current File:</div>
        <div class="file-path" title={progressData.current_file}>
          {progressData.current_file}
        </div>
      </div>
    {/if}

    <!-- Metrics Grid -->
    <div class="metrics-grid">
      <div class="metric-card">
        <div class="metric-icon">⚡</div>
        <div class="metric-content">
          <div class="metric-label">Speed</div>
          <div class="metric-value">{progressData.files_per_second.toFixed(1)} files/s</div>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon">⏱️</div>
        <div class="metric-content">
          <div class="metric-label">ETA</div>
          <div class="metric-value">
            {progressData.estimated_time_remaining > 0
              ? formatTime(progressData.estimated_time_remaining)
              : '--'}
          </div>
        </div>
      </div>

      <div class="metric-card" class:has-errors={progressData.errors_count > 0}>
        <div class="metric-icon">❌</div>
        <div class="metric-content">
          <div class="metric-label">Errors</div>
          <div class="metric-value">{progressData.errors_count.toLocaleString()}</div>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon">🔄</div>
        <div class="metric-content">
          <div class="metric-label">Duplicates</div>
          <div class="metric-value">{progressData.duplicates_found.toLocaleString()}</div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .progress-monitor {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    background: #2d2d2d;
    border-radius: 8px;
    border: 1px solid #3d3d3d;
    transition: border-color 0.3s ease;
  }

  .progress-monitor.active {
    border-color: #4a9eff;
  }

  .progress-monitor.paused {
    border-color: #ff9800;
  }

  .progress-monitor.complete {
    border-color: #4caf50;
  }

  /* Idle State */
  .idle-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px;
    gap: 12px;
  }

  .idle-icon {
    font-size: 48px;
    opacity: 0.5;
  }

  .idle-state p {
    color: #e0e0e0;
    margin: 0;
    font-size: 16px;
  }

  .idle-hint {
    color: #808080;
    font-size: 14px !important;
  }

  /* Paused Banner */
  .paused-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: rgba(255, 152, 0, 0.1);
    border: 1px solid #ff9800;
    border-radius: 6px;
    color: #ff9800;
    font-weight: 600;
  }

  .pause-icon {
    font-size: 20px;
  }

  /* Complete Banner */
  .complete-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: rgba(76, 175, 80, 0.1);
    border: 1px solid #4caf50;
    border-radius: 6px;
    color: #4caf50;
    font-weight: 600;
  }

  .complete-icon {
    font-size: 20px;
  }

  /* Progress Section */
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .phase-label {
    font-size: 16px;
    font-weight: 600;
    transition: color 0.3s ease;
  }

  .progress-percentage {
    font-size: 20px;
    font-weight: 700;
    color: #e0e0e0;
  }

  .progress-bar {
    height: 12px;
    background: #1e1e1e;
    border-radius: 6px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    transition: width 0.3s ease, background 0.3s ease;
    border-radius: 6px;
  }

  .progress-stats {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: #b0b0b0;
  }

  /* Current File */
  .current-file {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px;
    background: #1e1e1e;
    border-radius: 6px;
  }

  .file-label {
    font-size: 12px;
    color: #808080;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .file-path {
    font-size: 13px;
    color: #e0e0e0;
    font-family: 'Courier New', monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Metrics Grid */
  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }

  .metric-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: #1e1e1e;
    border-radius: 8px;
    border: 1px solid #3d3d3d;
    transition: border-color 0.2s ease;
  }

  .metric-card.has-errors {
    border-color: #f44336;
  }

  .metric-icon {
    font-size: 28px;
  }

  .metric-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .metric-label {
    font-size: 11px;
    color: #808080;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .metric-value {
    font-size: 18px;
    color: #e0e0e0;
    font-weight: 700;
  }

  .has-errors .metric-value {
    color: #f44336;
  }

  /* Dark mode support (already dark, but ensuring consistency) */
  @media (prefers-color-scheme: dark) {
    .progress-monitor {
      background: #2d2d2d;
      border-color: #3d3d3d;
    }
  }
</style>
