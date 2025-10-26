<script lang="ts">
  export let dbConnected: boolean;
  export let totalFiles: number;
</script>

<footer class="status-bar">
  <div class="status-item">
    <span class="status-label">Database:</span>
    <span class="status-value" class:connected={dbConnected} class:disconnected={!dbConnected}>
      {dbConnected ? 'Connected' : 'Disconnected'}
    </span>
  </div>

  <div class="status-item">
    <span class="status-label">Total Files:</span>
    <span class="status-value">{totalFiles.toLocaleString()}</span>
  </div>

  <div class="status-item">
    <span class="status-label">Ready</span>
  </div>
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 20px;
    height: 32px;
    padding: 0 16px;
    background: #1e1e1e;
    border-top: 1px solid #3d3d3d;
    font-size: 12px;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status-label {
    color: #808080;
  }

  .status-value {
    color: #e0e0e0;
    font-weight: 600;
  }

  .status-value.connected {
    color: #4ade80;
  }

  .status-value.disconnected {
    color: #ef4444;
  }
</style>
