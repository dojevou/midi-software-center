<script lang="ts">
  import { appStateStore } from '../stores/appState';
  import { statsStore } from '../stores/stats';
  import { processingStore } from '../stores/processing';

  // Subscribe to stores
  $: isConnected = $appStateStore.isConnected;
  $: fileCount = $statsStore.totalFiles;
  $: processingStatus = $processingStore.status;
  $: isProcessing = $processingStore.isProcessing;
  $: currentFile = $processingStore.currentFile;
  $: progress = $processingStore.progress;

  // Format file count for display
  function formatFileCount(count: number): string {
    if (count === 0) return '0 files';
    if (count === 1) return '1 file';
    if (count < 1000) return `${count} files`;
    if (count < 1000000) return `${(count / 1000).toFixed(1)}K files`;
    return `${(count / 1000000).toFixed(1)}M files`;
  }

  // Get status icon
  function getStatusIcon(connected: boolean): string {
    return connected ? '✓' : '✗';
  }

  // Get status color class
  function getStatusClass(connected: boolean): string {
    return connected ? 'status-success' : 'status-error';
  }
</script>

<footer class="statusbar">
  <div class="statusbar-left">
    <!-- Connection status -->
    <div class="status-item {getStatusClass(isConnected)}">
      <span class="status-icon">{getStatusIcon(isConnected)}</span>
      <span class="status-label">
        {isConnected ? 'Database Connected' : 'Database Disconnected'}
      </span>
    </div>

    <!-- File count -->
    <div class="status-item">
      <span class="status-icon">📊</span>
      <span class="status-label">{formatFileCount(fileCount)}</span>
    </div>
  </div>

  <div class="statusbar-center">
    {#if isProcessing}
      <!-- Processing status -->
      <div class="processing-status">
        <div class="processing-info">
          <span class="processing-icon">⚙️</span>
          <span class="processing-text">{processingStatus}</span>
          {#if currentFile}
            <span class="processing-file">• {currentFile}</span>
          {/if}
        </div>
        {#if progress !== null}
          <div class="progress-bar">
            <div class="progress-fill" style="width: {progress}%"></div>
          </div>
        {/if}
      </div>
    {:else}
      <!-- Ready status -->
      <div class="status-item status-ready">
        <span class="status-icon">✓</span>
        <span class="status-label">Ready</span>
      </div>
    {/if}
  </div>

  <div class="statusbar-right">
    <!-- Quick action buttons -->
    <button
      class="action-btn"
      on:click={() => statsStore.refresh()}
      title="Refresh statistics"
      aria-label="Refresh statistics"
      disabled={isProcessing}
    >
      🔄
    </button>

    <button
      class="action-btn"
      on:click={() => appStateStore.toggleSettings()}
      title="Open settings"
      aria-label="Open settings"
    >
      ⚙️
    </button>

    <!-- Version info -->
    <div class="version-info">
      <span class="version-label">v1.0.0</span>
    </div>
  </div>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--statusbar-height, 40px);
    padding: 0 var(--spacing-md, 16px);
    background: var(--color-surface, #2d2d2d);
    color: var(--color-text, #e5e5e5);
    font-size: var(--font-size-xs, 12px);
    gap: var(--spacing-md, 16px);
  }

  .statusbar-left,
  .statusbar-center,
  .statusbar-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-md, 16px);
    flex: 1;
  }

  .statusbar-center {
    justify-content: center;
  }

  .statusbar-right {
    justify-content: flex-end;
  }

  /* Status items */
  .status-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs, 4px);
    padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border-radius: var(--radius-sm, 4px);
    white-space: nowrap;
  }

  .status-icon {
    font-size: 14px;
    line-height: 1;
  }

  .status-label {
    color: var(--color-text-secondary, #a3a3a3);
    font-weight: 500;
  }

  .status-item.status-success .status-icon,
  .status-item.status-success .status-label {
    color: var(--color-success, #10b981);
  }

  .status-item.status-error .status-icon,
  .status-item.status-error .status-label {
    color: var(--color-error, #ef4444);
  }

  .status-item.status-ready {
    background: transparent;
  }

  .status-item.status-ready .status-label {
    color: var(--color-success, #10b981);
  }

  /* Processing status */
  .processing-status {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 4px);
    max-width: 400px;
    width: 100%;
  }

  .processing-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs, 4px);
    color: var(--color-text-secondary, #a3a3a3);
  }

  .processing-icon {
    font-size: 14px;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .processing-text {
    font-weight: 600;
    color: var(--color-primary, #3b82f6);
  }

  .processing-file {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .progress-bar {
    width: 100%;
    height: 3px;
    background: var(--color-bg, #1a1a1a);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-primary, #3b82f6);
    transition: width 0.3s ease;
    border-radius: 2px;
  }

  /* Action buttons */
  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text, #e5e5e5);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
    transform: scale(1.05);
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Version info */
  .version-info {
    padding: var(--spacing-xs, 4px) var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border-radius: var(--radius-sm, 4px);
  }

  .version-label {
    color: var(--color-text-secondary, #a3a3a3);
    font-weight: 500;
    font-family: monospace;
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .statusbar {
      padding: 0 var(--spacing-sm, 8px);
      gap: var(--spacing-sm, 8px);
    }

    .statusbar-left,
    .statusbar-right {
      flex: 0 1 auto;
    }

    .status-label {
      display: none;
    }

    .processing-file {
      display: none;
    }

    .version-info {
      display: none;
    }
  }
</style>
