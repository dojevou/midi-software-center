import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

export interface ServiceHealth {
  name: string;
  status: HealthStatus;
  latency_ms: number | null;
  last_check: number;
  error: string | null;
  details: Record<string, unknown> | null;
}

export interface SystemHealth {
  overall_status: HealthStatus;
  services: ServiceHealth[];
  timestamp: number;
  uptime_seconds: number;
}

interface HealthState {
  health: SystemHealth | null;
  isChecking: boolean;
  lastError: string | null;
  checkInterval: number | null;
}

// Constants
const DEFAULT_CHECK_INTERVAL = 30000; // 30 seconds
const UNHEALTHY_CHECK_INTERVAL = 5000; // 5 seconds when unhealthy

// Store
function createHealthStore() {
  const { subscribe, set, update } = writable<HealthState>({
    health: null,
    isChecking: false,
    lastError: null,
    checkInterval: null,
  });

  let intervalId: ReturnType<typeof setInterval> | null = null;

  async function checkHealth(): Promise<SystemHealth | null> {
    update(s => ({ ...s, isChecking: true, lastError: null }));

    try {
      const health = await invoke<SystemHealth>('check_system_health');
      update(s => ({ ...s, health, isChecking: false }));

      // Emit event for other stores to react
      window.dispatchEvent(new CustomEvent('health-updated', { detail: health }));

      return health;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, isChecking: false, lastError: message }));
      return null;
    }
  }

  async function getCachedHealth(): Promise<SystemHealth | null> {
    try {
      return await invoke<SystemHealth | null>('get_cached_health');
    } catch {
      return null;
    }
  }

  function startMonitoring(interval: number = DEFAULT_CHECK_INTERVAL): void {
    stopMonitoring();

    // Initial check
    checkHealth();

    // Setup interval
    intervalId = setInterval(async () => {
      const health = await checkHealth();

      // Increase check frequency when unhealthy
      if (health && health.overall_status === 'unhealthy') {
        if (get({ subscribe }).checkInterval !== UNHEALTHY_CHECK_INTERVAL) {
          startMonitoring(UNHEALTHY_CHECK_INTERVAL);
        }
      } else if (get({ subscribe }).checkInterval !== interval) {
        startMonitoring(interval);
      }
    }, interval);

    update(s => ({ ...s, checkInterval: interval }));
  }

  function stopMonitoring(): void {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    update(s => ({ ...s, checkInterval: null }));
  }

  function getServiceStatus(serviceName: string): ServiceHealth | null {
    const state = get({ subscribe });
    return state.health?.services.find(s => s.name === serviceName) || null;
  }

  return {
    subscribe,
    checkHealth,
    getCachedHealth,
    startMonitoring,
    stopMonitoring,
    getServiceStatus,

    // Cleanup
    destroy: () => {
      stopMonitoring();
    },
  };
}

export const healthStore = createHealthStore();

// Derived stores for specific services
export const postgresHealth = derived(healthStore, ($health) =>
  $health.health?.services.find(s => s.name === 'postgresql') || null
);

export const meilisearchHealth = derived(healthStore, ($health) =>
  $health.health?.services.find(s => s.name === 'meilisearch') || null
);

export const overallHealth = derived(healthStore, ($health) =>
  $health.health?.overall_status || 'unknown'
);

export const isSystemHealthy = derived(healthStore, ($health) =>
  $health.health?.overall_status === 'healthy'
);
