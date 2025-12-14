<script lang="ts">
  import { healthStore, type HealthStatus } from '$lib/stores/healthStore';
  import { onMount, onDestroy } from 'svelte';
  import { scale } from 'svelte/transition';

  export let showDetails = false;
  export let compact = false;

  let expanded = false;

  const statusColors: Record<HealthStatus, string> = {
    healthy: 'var(--color-success, #22c55e)',
    degraded: 'var(--color-warning, #eab308)',
    unhealthy: 'var(--color-error, #ef4444)',
    unknown: 'var(--color-muted, #6b7280)',
  };

  const statusLabels: Record<HealthStatus, string> = {
    healthy: 'All Systems Operational',
    degraded: 'Degraded Performance',
    unhealthy: 'Service Unavailable',
    unknown: 'Checking...',
  };

  function getStatusIcon(status: HealthStatus): string {
    switch (status) {
      case 'healthy': return '●';
      case 'degraded': return '◐';
      case 'unhealthy': return '○';
      default: return '◌';
    }
  }

  function formatLatency(ms: number | null): string {
    if (ms === null) {return 'N/A';}
    if (ms < 1) {return '<1ms';}
    return `${ms}ms`;
  }

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (days > 0) {return `${days}d ${hours}h`;}
    if (hours > 0) {return `${hours}h ${minutes}m`;}
    return `${minutes}m`;
  }

  onMount(() => {
    healthStore.startMonitoring();
  });

  onDestroy(() => {
    healthStore.stopMonitoring();
  });
</script>

<div
  class="health-indicator"
  class:compact
  role="status"
  aria-live="polite"
>
  <button
    class="health-button"
    on:click={() => expanded = !expanded}
    aria-expanded={expanded}
    aria-label="System health status"
  >
    <span
      class="status-dot"
      style="color: {statusColors[$healthStore.health?.overall_status || 'unknown']}"
    >
      {getStatusIcon($healthStore.health?.overall_status || 'unknown')}
    </span>

    {#if !compact}
      <span class="status-label">
        {statusLabels[$healthStore.health?.overall_status || 'unknown']}
      </span>
    {/if}

    {#if $healthStore.isChecking}
      <span class="checking-indicator" aria-label="Checking health">
        <svg class="spinner" viewBox="0 0 24 24" width="14" height="14">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="none" stroke-dasharray="31.4 31.4" />
        </svg>
      </span>
    {/if}
  </button>

  {#if expanded && showDetails && $healthStore.health}
    <div
      class="health-details"
      transition:scale={{ duration: 150, start: 0.95 }}
    >
      <div class="details-header">
        <h3>System Health</h3>
        <span class="uptime">
          Uptime: {formatUptime($healthStore.health.uptime_seconds)}
        </span>
      </div>

      <ul class="services-list">
        {#each $healthStore.health.services as service (service.name)}
          <li class="service-item">
            <span
              class="service-status"
              style="color: {statusColors[service.status]}"
            >
              {getStatusIcon(service.status)}
            </span>
            <span class="service-name">{service.name}</span>
            <span class="service-latency">{formatLatency(service.latency_ms)}</span>
            {#if service.error}
              <span class="service-error" title={service.error}>!</span>
            {/if}
          </li>
        {/each}
      </ul>

      <button
        class="refresh-button"
        on:click={() => healthStore.checkHealth()}
        disabled={$healthStore.isChecking}
      >
        Refresh
      </button>
    </div>
  {/if}
</div>

<style>
  .health-indicator {
    position: relative;
    display: inline-block;
  }

  .health-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: transparent;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    color: inherit;
    transition: background-color 0.15s;
  }

  .health-button:hover {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
  }

  .compact .health-button {
    padding: 2px 6px;
  }

  .status-dot {
    font-size: 10px;
    line-height: 1;
  }

  .status-label {
    white-space: nowrap;
  }

  .checking-indicator {
    display: flex;
    align-items: center;
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .health-details {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    padding: 12px;
    background: var(--panel-bg, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    min-width: 220px;
    z-index: 1000;
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .details-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .uptime {
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  .services-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .service-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 12px;
  }

  .service-status {
    font-size: 8px;
  }

  .service-name {
    flex: 1;
    text-transform: capitalize;
  }

  .service-latency {
    color: var(--text-muted, #888);
    font-family: monospace;
    font-size: 11px;
  }

  .service-error {
    color: var(--color-error, #ef4444);
    font-weight: bold;
    cursor: help;
  }

  .refresh-button {
    width: 100%;
    margin-top: 12px;
    padding: 6px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .refresh-button:hover:not(:disabled) {
    background: var(--button-hover-bg, #444);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
