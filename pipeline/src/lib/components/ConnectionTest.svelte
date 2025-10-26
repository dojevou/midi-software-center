<!--
  ConnectionTest.svelte
  ARCHETYPE: CONTAINER (Task-O-Matic)
  PURPOSE: Test database connection end-to-end with visual feedback

  Container Pattern Compliance:
  ✅ Composes UI elements and handles user interactions
  ✅ Delegates ALL I/O to API functions (no direct invoke() calls)
  ✅ No business logic (uses API for all operations)
  ✅ Manages local UI state only
-->

<script lang="ts">
  import {
    testDatabaseConnection,
    pingDatabase,
    getFileCount,
    getDatabaseSize,
    checkDatabaseHealth,
  } from '$lib/api';

  // Component state
  let isLoading = false;
  let isConnected: boolean | null = null;
  let errorMessage = '';

  // Connection stats (only populated on success)
  let fileCount: number | null = null;
  let databaseSize: string | null = null;
  let healthStatus: 'good' | 'warning' | 'error' | null = null;

  /**
   * Test the database connection and gather stats
   * Container responsibility: Orchestrate UI flow and API calls
   */
  async function testConnection() {
    // Reset state
    isLoading = true;
    isConnected = null;
    errorMessage = '';
    fileCount = null;
    databaseSize = null;
    healthStatus = null;

    try {
      // Step 1: Test basic connection
      const connected = await testDatabaseConnection();

      if (!connected) {
        isConnected = false;
        errorMessage = 'Database connection failed. Please check if the database is running.';
        return;
      }

      // Step 2: Verify with ping
      await pingDatabase();

      // Step 3: Gather statistics (only if connected)
      const [count, size, health] = await Promise.all([
        getFileCount(),
        getDatabaseSize(),
        checkDatabaseHealth(),
      ]);

      // Success - update all stats
      isConnected = true;
      fileCount = count;
      databaseSize = size;
      healthStatus = health;

    } catch (error) {
      // Handle any errors
      isConnected = false;
      errorMessage = error instanceof Error
        ? error.message
        : 'An unknown error occurred while testing the connection.';

      console.error('Connection test failed:', error);
    } finally {
      isLoading = false;
    }
  }

  /**
   * Retry the connection test
   * Container responsibility: Handle user action
   */
  function retry() {
    testConnection();
  }

  // Reactive: Format file count for display
  $: formattedFileCount = formatFileCount(fileCount);

  /**
   * Pure function: Format file count with units
   * Delegate to pure function for formatting logic
   */
  function formatFileCount(count: number | null): string {
    if (count === null) return 'N/A';
    if (count === 0) return '0 files';
    if (count === 1) return '1 file';
    if (count < 1000) return `${count} files`;
    if (count < 1000000) return `${(count / 1000).toFixed(1)}K files`;
    return `${(count / 1000000).toFixed(1)}M files`;
  }

  /**
   * Pure function: Get health status icon
   */
  function getHealthIcon(status: 'good' | 'warning' | 'error' | null): string {
    if (status === null) return '❓';
    switch (status) {
      case 'good': return '✓';
      case 'warning': return '⚠️';
      case 'error': return '✗';
      default: return '❓';
    }
  }

  /**
   * Pure function: Get health status label
   */
  function getHealthLabel(status: 'good' | 'warning' | 'error' | null): string {
    if (status === null) return 'Unknown';
    switch (status) {
      case 'good': return 'Good';
      case 'warning': return 'Warning';
      case 'error': return 'Error';
      default: return 'Unknown';
    }
  }

  /**
   * Pure function: Get health status CSS class
   */
  function getHealthClass(status: 'good' | 'warning' | 'error' | null): string {
    if (status === null) return 'health-unknown';
    return `health-${status}`;
  }
</script>

<div class="connection-test">
  <!-- Header -->
  <div class="test-header">
    <h2 class="test-title">Database Connection Test</h2>
    <p class="test-description">
      Test the connection to the PostgreSQL database and verify it's operational.
    </p>
  </div>

  <!-- Test Button -->
  <div class="test-actions">
    <button
      class="test-button"
      class:loading={isLoading}
      on:click={testConnection}
      disabled={isLoading}
      aria-label="Test database connection"
    >
      {#if isLoading}
        <span class="button-icon spinning">⚙️</span>
        <span class="button-text">Testing Connection...</span>
      {:else}
        <span class="button-icon">🔌</span>
        <span class="button-text">Test Connection</span>
      {/if}
    </button>
  </div>

  <!-- Results -->
  {#if isConnected === true}
    <!-- Success State -->
    <div class="test-result success">
      <div class="result-header">
        <span class="result-icon">✓</span>
        <h3 class="result-title">Connection Successful</h3>
      </div>

      <!-- Stats Grid -->
      <div class="stats-grid">
        <!-- File Count -->
        <div class="stat-card">
          <div class="stat-icon">📊</div>
          <div class="stat-content">
            <div class="stat-label">Total Files</div>
            <div class="stat-value">{formattedFileCount}</div>
          </div>
        </div>

        <!-- Database Size -->
        <div class="stat-card">
          <div class="stat-icon">💾</div>
          <div class="stat-content">
            <div class="stat-label">Database Size</div>
            <div class="stat-value">{databaseSize ?? 'N/A'}</div>
          </div>
        </div>

        <!-- Health Status -->
        <div class="stat-card">
          <div class="stat-icon {getHealthClass(healthStatus)}">
            {getHealthIcon(healthStatus)}
          </div>
          <div class="stat-content">
            <div class="stat-label">Health Status</div>
            <div class="stat-value {getHealthClass(healthStatus)}">
              {getHealthLabel(healthStatus)}
            </div>
          </div>
        </div>
      </div>

      <!-- Success Message -->
      <div class="success-message">
        <p>
          Database is connected and operational. All systems ready.
        </p>
      </div>
    </div>
  {:else if isConnected === false}
    <!-- Error State -->
    <div class="test-result error">
      <div class="result-header">
        <span class="result-icon">✗</span>
        <h3 class="result-title">Connection Failed</h3>
      </div>

      <!-- Error Message -->
      <div class="error-message">
        <p>{errorMessage}</p>
      </div>

      <!-- Troubleshooting Tips -->
      <div class="troubleshooting">
        <h4 class="troubleshooting-title">Troubleshooting:</h4>
        <ul class="troubleshooting-list">
          <li>Verify the database is running (check Docker containers)</li>
          <li>Check database connection settings in configuration</li>
          <li>Ensure network connectivity to the database host</li>
          <li>Verify database credentials are correct</li>
        </ul>
      </div>

      <!-- Retry Button -->
      <div class="retry-actions">
        <button
          class="retry-button"
          on:click={retry}
          aria-label="Retry connection test"
        >
          <span class="button-icon">🔄</span>
          <span class="button-text">Retry</span>
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Container */
  .connection-test {
    max-width: 600px;
    margin: 0 auto;
    padding: var(--spacing-xl, 32px);
  }

  /* Header */
  .test-header {
    text-align: center;
    margin-bottom: var(--spacing-xl, 32px);
  }

  .test-title {
    font-size: var(--font-size-2xl, 24px);
    font-weight: 700;
    color: var(--color-text, #e5e5e5);
    margin: 0 0 var(--spacing-sm, 8px) 0;
  }

  .test-description {
    font-size: var(--font-size-sm, 14px);
    color: var(--color-text-secondary, #a3a3a3);
    margin: 0;
  }

  /* Test Actions */
  .test-actions {
    display: flex;
    justify-content: center;
    margin-bottom: var(--spacing-xl, 32px);
  }

  .test-button {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-md, 16px) var(--spacing-xl, 32px);
    background: var(--color-primary, #3b82f6);
    color: white;
    border: none;
    border-radius: var(--radius-md, 8px);
    font-size: var(--font-size-base, 16px);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .test-button:hover:not(:disabled) {
    background: var(--color-primary-hover, #2563eb);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  }

  .test-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .test-button.loading {
    background: var(--color-primary-hover, #2563eb);
  }

  .button-icon {
    font-size: 20px;
    line-height: 1;
  }

  .button-text {
    line-height: 1;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Results */
  .test-result {
    padding: var(--spacing-xl, 32px);
    border-radius: var(--radius-lg, 12px);
    border: 2px solid;
  }

  .test-result.success {
    background: rgba(16, 185, 129, 0.05);
    border-color: var(--color-success, #10b981);
  }

  .test-result.error {
    background: rgba(239, 68, 68, 0.05);
    border-color: var(--color-error, #ef4444);
  }

  /* Result Header */
  .result-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md, 16px);
    margin-bottom: var(--spacing-xl, 32px);
  }

  .result-icon {
    font-size: 32px;
    line-height: 1;
  }

  .test-result.success .result-icon {
    color: var(--color-success, #10b981);
  }

  .test-result.error .result-icon {
    color: var(--color-error, #ef4444);
  }

  .result-title {
    font-size: var(--font-size-xl, 20px);
    font-weight: 700;
    color: var(--color-text, #e5e5e5);
    margin: 0;
  }

  /* Stats Grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--spacing-md, 16px);
    margin-bottom: var(--spacing-xl, 32px);
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-md, 16px);
    padding: var(--spacing-md, 16px);
    background: var(--color-surface, #2d2d2d);
    border-radius: var(--radius-md, 8px);
    border: 1px solid var(--color-border, #404040);
  }

  .stat-icon {
    font-size: 28px;
    line-height: 1;
  }

  .stat-content {
    flex: 1;
    min-width: 0;
  }

  .stat-label {
    font-size: var(--font-size-xs, 12px);
    color: var(--color-text-secondary, #a3a3a3);
    margin-bottom: var(--spacing-xs, 4px);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: var(--font-size-lg, 18px);
    font-weight: 700;
    color: var(--color-text, #e5e5e5);
  }

  /* Health Status Colors */
  .health-good {
    color: var(--color-success, #10b981);
  }

  .health-warning {
    color: var(--color-warning, #f59e0b);
  }

  .health-error {
    color: var(--color-error, #ef4444);
  }

  .health-unknown {
    color: var(--color-text-secondary, #a3a3a3);
  }

  /* Messages */
  .success-message,
  .error-message {
    padding: var(--spacing-md, 16px);
    border-radius: var(--radius-md, 8px);
    margin-bottom: var(--spacing-md, 16px);
  }

  .success-message {
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid var(--color-success, #10b981);
  }

  .success-message p {
    margin: 0;
    color: var(--color-success, #10b981);
    font-size: var(--font-size-sm, 14px);
    text-align: center;
  }

  .error-message {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid var(--color-error, #ef4444);
  }

  .error-message p {
    margin: 0;
    color: var(--color-error, #ef4444);
    font-size: var(--font-size-sm, 14px);
  }

  /* Troubleshooting */
  .troubleshooting {
    margin-bottom: var(--spacing-xl, 32px);
  }

  .troubleshooting-title {
    font-size: var(--font-size-sm, 14px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0 0 var(--spacing-sm, 8px) 0;
  }

  .troubleshooting-list {
    margin: 0;
    padding-left: var(--spacing-lg, 24px);
    color: var(--color-text-secondary, #a3a3a3);
    font-size: var(--font-size-sm, 14px);
  }

  .troubleshooting-list li {
    margin-bottom: var(--spacing-xs, 4px);
  }

  /* Retry Actions */
  .retry-actions {
    display: flex;
    justify-content: center;
  }

  .retry-button {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px) var(--spacing-lg, 24px);
    background: var(--color-surface, #2d2d2d);
    color: var(--color-text, #e5e5e5);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-md, 8px);
    font-size: var(--font-size-sm, 14px);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button:hover {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
    transform: translateY(-1px);
  }

  .retry-button .button-icon {
    font-size: 16px;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .connection-test {
      padding: var(--spacing-lg, 24px);
    }

    .test-result {
      padding: var(--spacing-lg, 24px);
    }

    .stats-grid {
      grid-template-columns: 1fr;
    }
  }

  /* Dark mode support (uses CSS variables) */
  @media (prefers-color-scheme: dark) {
    .connection-test {
      /* Already using dark mode variables */
    }
  }
</style>
