/**
 * Phase 7C Frontend Optimization - Svelte Hooks for Performance Monitoring
 *
 * Custom Svelte hooks and utilities for component-level performance tracking
 *
 * Architecture: Trusty Module (reusable utilities)
 * Usage: Import in components to enable automatic profiling
 *
 * @module profiling/hooks
 */

import { onMount, onDestroy } from 'svelte';
import { profiler } from './performance';
import { CleanupTracker } from './performance';

/**
 * Hook to track component render performance
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useRenderProfiler } from '$lib/profiling/hooks';
 *   useRenderProfiler('MyComponent');
 * </script>
 * ```
 */
export function useRenderProfiler(componentName: string): void {
  const { render } = profiler.getProfilers();
  let renderCount = 0;

  onMount(() => {
    renderCount++;
    const stopTracking = render.startRender(`${componentName}-mount-${renderCount}`);
    stopTracking();
  });
}

/**
 * Hook to track store subscriptions with automatic cleanup
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useStoreSubscription } from '$lib/profiling/hooks';
 *   import { myStore } from '$lib/stores';
 *
 *   const cleanup = useStoreSubscription('myStore');
 *   const unsubscribe = myStore.subscribe(value => {
 *     // handle value
 *   });
 *   cleanup.registerSubscription(unsubscribe);
 * </script>
 * ```
 */
export function useStoreSubscription(componentName: string): CleanupTracker {
  const cleanup = new CleanupTracker(componentName);

  onDestroy(() => {
    cleanup.cleanup();
  });

  return cleanup;
}

/**
 * Hook to monitor memory usage during component lifecycle
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useMemoryMonitor } from '$lib/profiling/hooks';
 *   useMemoryMonitor('MyComponent');
 * </script>
 * ```
 */
export function useMemoryMonitor(componentName: string): void {
  const { memory } = profiler.getProfilers();
  let stopMonitoring: (() => void) | null = null;

  onMount(() => {
    console.log(`[${componentName}] Starting memory monitoring`);
    const snapshot = memory.takeSnapshot();
    if (snapshot) {
      console.log(`[${componentName}] Initial memory: ${(snapshot.usedJSHeapSize / 1024 / 1024).toFixed(2)} MB`);
    }

    stopMonitoring = memory.startMonitoring(10000); // Every 10 seconds
  });

  onDestroy(() => {
    if (stopMonitoring) {
      stopMonitoring();
    }

    const snapshot = memory.takeSnapshot();
    if (snapshot) {
      console.log(`[${componentName}] Final memory: ${(snapshot.usedJSHeapSize / 1024 / 1024).toFixed(2)} MB`);
    }

    // Check for leaks
    const leakDetection = memory.detectMemoryLeaks();
    if (leakDetection.leakDetected) {
      console.warn(`[${componentName}] Possible memory leak: ${leakDetection.analysis}`);
    }
  });
}

/**
 * Hook to monitor FPS during component lifecycle
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useFPSMonitor } from '$lib/profiling/hooks';
 *   useFPSMonitor('MyComponent');
 * </script>
 * ```
 */
export function useFPSMonitor(componentName: string): void {
  const { render } = profiler.getProfilers();
  let stopMonitoring: (() => void) | null = null;
  let intervalId: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    console.log(`[${componentName}] Starting FPS monitoring`);
    stopMonitoring = render.startFPSMonitoring();

    intervalId = setInterval(() => {
      const fps = render.getCurrentFPS();
      if (fps < 30) {
        console.warn(`[${componentName}] Low FPS detected: ${fps}`);
      }
    }, 5000);
  });

  onDestroy(() => {
    if (stopMonitoring) {
      stopMonitoring();
    }
    if (intervalId) {
      clearInterval(intervalId);
    }

    const finalFPS = render.getCurrentFPS();
    console.log(`[${componentName}] Final FPS: ${finalFPS}`);
  });
}

/**
 * Hook to track network requests in a component
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useNetworkTracker } from '$lib/profiling/hooks';
 *   const trackRequest = useNetworkTracker('MyComponent');
 *
 *   async function loadData() {
 *     await trackRequest(async () => {
 *       const result = await invoke('get_files');
 *       return result;
 *     });
 *   }
 * </script>
 * ```
 */
export function useNetworkTracker(componentName: string) {
  const { network } = profiler.getProfilers();
  let requestCount = 0;

  onMount(() => {
    console.log(`[${componentName}] Network tracking enabled`);
  });

  onDestroy(() => {
    const metrics = network.getMetrics();
    console.log(`[${componentName}] Network stats: ${requestCount} requests, avg latency: ${metrics.averageLatency.toFixed(0)}ms`);
  });

  return async <T>(requestFn: () => Promise<T>, size?: number): Promise<T> => {
    requestCount++;
    return network.trackRequest(requestFn, size);
  };
}

/**
 * Combined hook for full component profiling
 *
 * Usage:
 * ```svelte
 * <script>
 *   import { useFullProfiler } from '$lib/profiling/hooks';
 *   const { trackRequest, cleanup } = useFullProfiler('MyComponent');
 * </script>
 * ```
 */
export function useFullProfiler(componentName: string) {
  useRenderProfiler(componentName);
  useFPSMonitor(componentName);
  useMemoryMonitor(componentName);

  const cleanup = useStoreSubscription(componentName);
  const trackRequest = useNetworkTracker(componentName);

  return {
    cleanup,
    trackRequest
  };
}

/**
 * Development-only profiling wrapper
 * Only enables profiling in development mode
 */
export function useDevProfiler(componentName: string, enabled?: boolean) {
  // Default to development mode check
  const isDev = enabled !== undefined ? enabled : (typeof import !== 'undefined' && import.meta?.env?.DEV);

  if (!isDev) return;

  useRenderProfiler(componentName);
  useFPSMonitor(componentName);
  useMemoryMonitor(componentName);
}
