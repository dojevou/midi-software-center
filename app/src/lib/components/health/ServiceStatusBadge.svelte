<script lang="ts">
  import type { HealthStatus } from '$lib/stores/healthStore';

  export let status: HealthStatus;
  export let serviceName: string;
  export let latency: number | null = null;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  const statusStyles: Record<HealthStatus, { bg: string; text: string }> = {
    healthy: { bg: 'rgba(34, 197, 94, 0.15)', text: '#22c55e' },
    degraded: { bg: 'rgba(234, 179, 8, 0.15)', text: '#eab308' },
    unhealthy: { bg: 'rgba(239, 68, 68, 0.15)', text: '#ef4444' },
    unknown: { bg: 'rgba(107, 114, 128, 0.15)', text: '#6b7280' },
  };

  $: style = statusStyles[status];
</script>

<span
  class="badge size-{size}"
  style="background-color: {style.bg}; color: {style.text}"
  role="status"
  aria-label="{serviceName} status: {status}"
>
  <span class="dot" />
  <span class="name">{serviceName}</span>
  {#if latency !== null}
    <span class="latency">{latency}ms</span>
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 9999px;
    font-weight: 500;
    text-transform: capitalize;
  }

  .size-sm {
    font-size: 10px;
    padding: 2px 8px;
    gap: 4px;
  }

  .size-md {
    font-size: 12px;
  }

  .size-lg {
    font-size: 14px;
    padding: 6px 14px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: currentColor;
  }

  .size-sm .dot {
    width: 4px;
    height: 4px;
  }

  .size-lg .dot {
    width: 8px;
    height: 8px;
  }

  .name {
    white-space: nowrap;
  }

  .latency {
    opacity: 0.7;
    font-family: monospace;
    font-size: 0.9em;
  }
</style>
