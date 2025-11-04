/**
 * Phase 7C Frontend Optimization - Public API
 *
 * Main entry point for performance profiling utilities
 *
 * @module profiling
 */

// Export core profiling classes
export {
  BundleAnalyzer,
  RenderProfiler,
  StoreProfiler,
  NetworkProfiler,
  MemoryProfiler,
  PerformanceProfiler,
  VirtualScrollHelper,
  CleanupTracker,
  RequestBatcher,
  createDebouncedStore,
  profiler
} from './performance';

// Export type definitions
export type {
  BundleMetrics,
  RenderMetrics,
  StoreMetrics,
  NetworkMetrics,
  MemoryMetrics,
  PerformanceSnapshot,
  OptimizationRecommendation
} from './performance';

// Export Svelte hooks
export {
  useRenderProfiler,
  useStoreSubscription,
  useMemoryMonitor,
  useFPSMonitor,
  useNetworkTracker,
  useFullProfiler,
  useDevProfiler
} from './hooks';

// Export Vite plugin
export { bundleAnalyzerPlugin } from './vite-plugin';
