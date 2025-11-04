/**
 * Phase 7C Frontend Optimization - Performance Monitoring & Optimization
 *
 * Comprehensive frontend profiling system with:
 * - Bundle size reduction strategies
 * - Rendering performance optimization
 * - Store optimization patterns
 * - Network request optimization
 * - Memory leak detection
 *
 * Architecture: Trusty Module (pure functions, measurements)
 * Dependencies: None (standalone utility)
 *
 * @module profiling/performance
 */

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

export interface BundleMetrics {
  totalSize: number;
  jsSize: number;
  cssSize: number;
  assetSize: number;
  chunkCount: number;
  largestChunks: Array<{ name: string; size: number }>;
  timestamp: number;
}

export interface RenderMetrics {
  componentName: string;
  renderCount: number;
  averageRenderTime: number;
  maxRenderTime: number;
  minRenderTime: number;
  totalRenderTime: number;
  lastRenderTimestamp: number;
}

export interface StoreMetrics {
  storeName: string;
  subscriptionCount: number;
  updateCount: number;
  averageUpdateTime: number;
  lastUpdateTimestamp: number;
  memoryUsage?: number;
}

export interface NetworkMetrics {
  requestCount: number;
  failedRequests: number;
  averageLatency: number;
  maxLatency: number;
  minLatency: number;
  totalDataTransferred: number;
  cacheHitRate: number;
}

export interface MemoryMetrics {
  jsHeapSize: number;
  jsHeapSizeLimit: number;
  totalJSHeapSize: number;
  usedJSHeapSize: number;
  percentUsed: number;
  timestamp: number;
}

export interface PerformanceSnapshot {
  bundle: BundleMetrics;
  renders: Map<string, RenderMetrics>;
  stores: Map<string, StoreMetrics>;
  network: NetworkMetrics;
  memory: MemoryMetrics;
  fps: number;
  timestamp: number;
}

export interface OptimizationRecommendation {
  category: 'bundle' | 'render' | 'store' | 'network' | 'memory';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  impact: string;
  solution: string;
  estimatedGain: string;
}

// ============================================================================
// CONSTANTS
// ============================================================================

const TARGET_FPS = 60;
const TARGET_FRAME_TIME = 16.67; // milliseconds (1000 / 60)
const MAX_BUNDLE_SIZE = 500 * 1024; // 500KB
const MAX_CHUNK_SIZE = 200 * 1024; // 200KB
const MAX_MEMORY_PERCENT = 80; // 80% of heap limit
const MIN_CACHE_HIT_RATE = 0.7; // 70%
const MAX_RENDER_TIME = 16; // milliseconds
const DEBOUNCE_THRESHOLD = 50; // milliseconds

// ============================================================================
// 1. BUNDLE SIZE REDUCTION (300 LINES)
// ============================================================================

/**
 * Analyzes bundle size and identifies optimization opportunities
 */
export class BundleAnalyzer {
  private metrics: BundleMetrics[] = [];

  /**
   * Records current bundle metrics
   */
  recordBundleMetrics(manifest: Record<string, any>): BundleMetrics {
    const metrics: BundleMetrics = {
      totalSize: 0,
      jsSize: 0,
      cssSize: 0,
      assetSize: 0,
      chunkCount: 0,
      largestChunks: [],
      timestamp: Date.now()
    };

    const chunks: Array<{ name: string; size: number }> = [];

    // Analyze manifest entries
    for (const [file, info] of Object.entries(manifest)) {
      const size = info.size || 0;
      metrics.totalSize += size;

      if (file.endsWith('.js')) {
        metrics.jsSize += size;
        metrics.chunkCount++;
        chunks.push({ name: file, size });
      } else if (file.endsWith('.css')) {
        metrics.cssSize += size;
      } else {
        metrics.assetSize += size;
      }
    }

    // Sort and get largest chunks
    chunks.sort((a, b) => b.size - a.size);
    metrics.largestChunks = chunks.slice(0, 10);

    this.metrics.push(metrics);
    return metrics;
  }

  /**
   * Analyzes bundle and provides optimization recommendations
   */
  analyzeBundleOptimizations(metrics: BundleMetrics): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];

    // Check total bundle size
    if (metrics.totalSize > MAX_BUNDLE_SIZE) {
      recommendations.push({
        category: 'bundle',
        severity: 'high',
        title: 'Bundle size exceeds target',
        description: `Total bundle size is ${this.formatBytes(metrics.totalSize)}, exceeds ${this.formatBytes(MAX_BUNDLE_SIZE)} target`,
        impact: 'Slower initial load time, poor user experience on slow networks',
        solution: 'Implement code splitting, lazy loading, and tree shaking',
        estimatedGain: `Reduce by ${this.formatBytes(metrics.totalSize - MAX_BUNDLE_SIZE)}`
      });
    }

    // Check for large chunks
    const oversizedChunks = metrics.largestChunks.filter(c => c.size > MAX_CHUNK_SIZE);
    if (oversizedChunks.length > 0) {
      recommendations.push({
        category: 'bundle',
        severity: 'medium',
        title: `${oversizedChunks.length} oversized chunks detected`,
        description: `Chunks: ${oversizedChunks.map(c => `${c.name} (${this.formatBytes(c.size)})`).join(', ')}`,
        impact: 'Inefficient caching, unnecessary code loaded',
        solution: 'Split large chunks using dynamic imports and route-based splitting',
        estimatedGain: 'Up to 40% faster initial load'
      });
    }

    // Check JS to CSS ratio
    const jsRatio = metrics.jsSize / metrics.totalSize;
    if (jsRatio > 0.8) {
      recommendations.push({
        category: 'bundle',
        severity: 'low',
        title: 'High JavaScript to CSS ratio',
        description: `JS accounts for ${(jsRatio * 100).toFixed(1)}% of bundle`,
        impact: 'CSS-in-JS may be inefficient',
        solution: 'Consider extracting CSS to separate files',
        estimatedGain: '10-15% smaller bundle size'
      });
    }

    return recommendations;
  }

  /**
   * Generates code splitting recommendations based on route analysis
   */
  generateCodeSplittingPlan(routes: string[]): Record<string, string[]> {
    const splittingPlan: Record<string, string[]> = {
      // Core bundle - always loaded
      core: [
        'src/routes/+layout.svelte',
        'src/lib/stores/index.ts',
        'src/lib/api.ts'
      ],

      // Window-specific chunks - lazy loaded
      database: [
        'src/lib/windows/DatabaseWindow.svelte',
        'src/lib/stores/databaseStore.ts',
        'src/lib/components/FileBrowser/**'
      ],

      project: [
        'src/lib/windows/ProjectWindow.svelte',
        'src/lib/stores/projectStore.ts',
        'src/lib/components/FileDetails/**'
      ],

      playback: [
        'src/lib/windows/PlaybackWindow.svelte',
        'src/lib/stores/playbackStore.ts'
      ],

      import: [
        'src/lib/components/Import/**',
        'src/lib/stores/import.ts'
      ],

      search: [
        'src/lib/components/Search/**',
        'src/lib/stores/searchStore.ts'
      ],

      tags: [
        'src/lib/components/Tags/**',
        'src/lib/stores/tags.ts'
      ]
    };

    return splittingPlan;
  }

  /**
   * Identifies unused code that can be tree-shaken
   */
  identifyUnusedCode(): string[] {
    // This would integrate with build tools to find unused exports
    // For now, return common candidates
    return [
      'Unused lodash methods (use lodash-es with named imports)',
      'Unused Lucide icons (import only used icons)',
      'Unused utility functions in utils/',
      'Dead code in conditional branches',
      'Unused CSS classes and styles'
    ];
  }

  /**
   * Calculates estimated load time based on bundle size
   */
  estimateLoadTime(metrics: BundleMetrics, connectionSpeed: 'slow-3g' | '4g' | 'fiber'): number {
    const speeds = {
      'slow-3g': 50 * 1024, // 50 KB/s
      '4g': 1.5 * 1024 * 1024, // 1.5 MB/s
      'fiber': 10 * 1024 * 1024 // 10 MB/s
    };

    const bytesPerSecond = speeds[connectionSpeed];
    const downloadTime = metrics.totalSize / bytesPerSecond;
    const parseTime = metrics.jsSize / (1024 * 1024) * 100; // ~100ms per MB of JS

    return downloadTime + parseTime;
  }

  /**
   * Format bytes to human-readable string
   */
  private formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  /**
   * Get bundle metrics history
   */
  getMetricsHistory(): BundleMetrics[] {
    return [...this.metrics];
  }

  /**
   * Compare two bundle metrics
   */
  compareMetrics(before: BundleMetrics, after: BundleMetrics): Record<string, number> {
    return {
      totalSizeDelta: after.totalSize - before.totalSize,
      jsSizeDelta: after.jsSize - before.jsSize,
      cssSizeDelta: after.cssSize - before.cssSize,
      chunkCountDelta: after.chunkCount - before.chunkCount,
      percentChange: ((after.totalSize - before.totalSize) / before.totalSize) * 100
    };
  }
}

// ============================================================================
// 2. RENDERING PERFORMANCE (400 LINES)
// ============================================================================

/**
 * Monitors and optimizes component rendering performance
 */
export class RenderProfiler {
  private renderMetrics = new Map<string, RenderMetrics>();
  private frameTimestamps: number[] = [];
  private rafId: number | null = null;

  /**
   * Start tracking a component render
   */
  startRender(componentName: string): () => void {
    const startTime = performance.now();

    return () => {
      const endTime = performance.now();
      const duration = endTime - startTime;
      this.recordRender(componentName, duration);
    };
  }

  /**
   * Record a component render
   */
  private recordRender(componentName: string, duration: number): void {
    const existing = this.renderMetrics.get(componentName);

    if (existing) {
      const newRenderCount = existing.renderCount + 1;
      const newTotalTime = existing.totalRenderTime + duration;

      this.renderMetrics.set(componentName, {
        componentName,
        renderCount: newRenderCount,
        averageRenderTime: newTotalTime / newRenderCount,
        maxRenderTime: Math.max(existing.maxRenderTime, duration),
        minRenderTime: Math.min(existing.minRenderTime, duration),
        totalRenderTime: newTotalTime,
        lastRenderTimestamp: Date.now()
      });
    } else {
      this.renderMetrics.set(componentName, {
        componentName,
        renderCount: 1,
        averageRenderTime: duration,
        maxRenderTime: duration,
        minRenderTime: duration,
        totalRenderTime: duration,
        lastRenderTimestamp: Date.now()
      });
    }
  }

  /**
   * Start FPS monitoring
   */
  startFPSMonitoring(): () => void {
    let lastFrameTime = performance.now();

    const measureFrame = () => {
      const now = performance.now();
      const delta = now - lastFrameTime;
      this.frameTimestamps.push(delta);

      // Keep only last 60 frames
      if (this.frameTimestamps.length > 60) {
        this.frameTimestamps.shift();
      }

      lastFrameTime = now;
      this.rafId = requestAnimationFrame(measureFrame);
    };

    this.rafId = requestAnimationFrame(measureFrame);

    return () => {
      if (this.rafId !== null) {
        cancelAnimationFrame(this.rafId);
        this.rafId = null;
      }
    };
  }

  /**
   * Calculate current FPS
   */
  getCurrentFPS(): number {
    if (this.frameTimestamps.length === 0) return 0;

    const averageFrameTime = this.frameTimestamps.reduce((a, b) => a + b, 0) / this.frameTimestamps.length;
    return Math.round(1000 / averageFrameTime);
  }

  /**
   * Get all render metrics
   */
  getRenderMetrics(): Map<string, RenderMetrics> {
    return new Map(this.renderMetrics);
  }

  /**
   * Identify slow components
   */
  identifySlowComponents(threshold = MAX_RENDER_TIME): RenderMetrics[] {
    return Array.from(this.renderMetrics.values())
      .filter(m => m.averageRenderTime > threshold)
      .sort((a, b) => b.averageRenderTime - a.averageRenderTime);
  }

  /**
   * Identify frequently re-rendering components
   */
  identifyFrequentRerenders(threshold = 10, timeWindow = 5000): RenderMetrics[] {
    const now = Date.now();
    return Array.from(this.renderMetrics.values())
      .filter(m => {
        const isRecent = now - m.lastRenderTimestamp < timeWindow;
        return isRecent && m.renderCount > threshold;
      })
      .sort((a, b) => b.renderCount - a.renderCount);
  }

  /**
   * Generate rendering optimization recommendations
   */
  analyzeRenderingOptimizations(): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];
    const slowComponents = this.identifySlowComponents();
    const frequentComponents = this.identifyFrequentRerenders();
    const fps = this.getCurrentFPS();

    // Check FPS
    if (fps < TARGET_FPS) {
      recommendations.push({
        category: 'render',
        severity: 'critical',
        title: 'Low FPS detected',
        description: `Current FPS: ${fps}, target: ${TARGET_FPS}`,
        impact: 'Janky animations, poor user experience',
        solution: 'Optimize render-heavy components, use virtual scrolling, reduce DOM updates',
        estimatedGain: `Improve to ${TARGET_FPS} FPS`
      });
    }

    // Check slow components
    if (slowComponents.length > 0) {
      const topSlow = slowComponents.slice(0, 3);
      recommendations.push({
        category: 'render',
        severity: 'high',
        title: `${slowComponents.length} slow components detected`,
        description: `Top offenders: ${topSlow.map(c => `${c.componentName} (${c.averageRenderTime.toFixed(2)}ms)`).join(', ')}`,
        impact: 'Delayed UI updates, poor responsiveness',
        solution: 'Use memoization, optimize expensive calculations, implement virtualization',
        estimatedGain: 'Up to 50% faster rendering'
      });
    }

    // Check frequent re-renders
    if (frequentComponents.length > 0) {
      const topFrequent = frequentComponents.slice(0, 3);
      recommendations.push({
        category: 'render',
        severity: 'medium',
        title: `${frequentComponents.length} components re-rendering frequently`,
        description: `Top offenders: ${topFrequent.map(c => `${c.componentName} (${c.renderCount} renders)`).join(', ')}`,
        impact: 'Unnecessary CPU usage, battery drain',
        solution: 'Use derived stores, debounce updates, prevent unnecessary subscriptions',
        estimatedGain: 'Reduce render count by 60-80%'
      });
    }

    return recommendations;
  }

  /**
   * Clear all metrics
   */
  clearMetrics(): void {
    this.renderMetrics.clear();
    this.frameTimestamps = [];
  }
}

/**
 * Virtual scrolling helper for large lists
 */
export class VirtualScrollHelper {
  private containerHeight: number;
  private itemHeight: number;
  private totalItems: number;
  private overscan: number;

  constructor(containerHeight: number, itemHeight: number, totalItems: number, overscan = 3) {
    this.containerHeight = containerHeight;
    this.itemHeight = itemHeight;
    this.totalItems = totalItems;
    this.overscan = overscan;
  }

  /**
   * Calculate visible range based on scroll position
   */
  calculateVisibleRange(scrollTop: number): { start: number; end: number; visibleItems: number } {
    const startIndex = Math.max(0, Math.floor(scrollTop / this.itemHeight) - this.overscan);
    const visibleCount = Math.ceil(this.containerHeight / this.itemHeight);
    const endIndex = Math.min(this.totalItems, startIndex + visibleCount + this.overscan * 2);

    return {
      start: startIndex,
      end: endIndex,
      visibleItems: endIndex - startIndex
    };
  }

  /**
   * Calculate total content height
   */
  getTotalHeight(): number {
    return this.totalItems * this.itemHeight;
  }

  /**
   * Calculate offset for visible items
   */
  getOffsetY(startIndex: number): number {
    return startIndex * this.itemHeight;
  }

  /**
   * Update total items count
   */
  updateTotalItems(totalItems: number): void {
    this.totalItems = totalItems;
  }
}

// ============================================================================
// 3. STORE OPTIMIZATION (300 LINES)
// ============================================================================

/**
 * Monitors and optimizes Svelte store performance
 */
export class StoreProfiler {
  private storeMetrics = new Map<string, StoreMetrics>();

  /**
   * Track store subscription
   */
  trackSubscription(storeName: string): () => void {
    const existing = this.storeMetrics.get(storeName);

    if (existing) {
      existing.subscriptionCount++;
    } else {
      this.storeMetrics.set(storeName, {
        storeName,
        subscriptionCount: 1,
        updateCount: 0,
        averageUpdateTime: 0,
        lastUpdateTimestamp: Date.now()
      });
    }

    // Return unsubscribe function
    return () => {
      const metrics = this.storeMetrics.get(storeName);
      if (metrics && metrics.subscriptionCount > 0) {
        metrics.subscriptionCount--;
      }
    };
  }

  /**
   * Track store update
   */
  trackUpdate(storeName: string, updateFn: () => void): void {
    const startTime = performance.now();
    updateFn();
    const duration = performance.now() - startTime;

    const existing = this.storeMetrics.get(storeName);
    if (existing) {
      const newUpdateCount = existing.updateCount + 1;
      const totalTime = existing.averageUpdateTime * existing.updateCount + duration;

      existing.updateCount = newUpdateCount;
      existing.averageUpdateTime = totalTime / newUpdateCount;
      existing.lastUpdateTimestamp = Date.now();
    } else {
      this.storeMetrics.set(storeName, {
        storeName,
        subscriptionCount: 0,
        updateCount: 1,
        averageUpdateTime: duration,
        lastUpdateTimestamp: Date.now()
      });
    }
  }

  /**
   * Get all store metrics
   */
  getStoreMetrics(): Map<string, StoreMetrics> {
    return new Map(this.storeMetrics);
  }

  /**
   * Identify stores with high subscription count
   */
  identifyHighSubscriptionStores(threshold = 10): StoreMetrics[] {
    return Array.from(this.storeMetrics.values())
      .filter(m => m.subscriptionCount > threshold)
      .sort((a, b) => b.subscriptionCount - a.subscriptionCount);
  }

  /**
   * Identify frequently updating stores
   */
  identifyFrequentUpdates(threshold = 20, timeWindow = 5000): StoreMetrics[] {
    const now = Date.now();
    return Array.from(this.storeMetrics.values())
      .filter(m => {
        const isRecent = now - m.lastUpdateTimestamp < timeWindow;
        return isRecent && m.updateCount > threshold;
      })
      .sort((a, b) => b.updateCount - a.updateCount);
  }

  /**
   * Identify slow store updates
   */
  identifySlowUpdates(threshold = 5): StoreMetrics[] {
    return Array.from(this.storeMetrics.values())
      .filter(m => m.averageUpdateTime > threshold)
      .sort((a, b) => b.averageUpdateTime - a.averageUpdateTime);
  }

  /**
   * Generate store optimization recommendations
   */
  analyzeStoreOptimizations(): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];
    const highSubscriptions = this.identifyHighSubscriptionStores();
    const frequentUpdates = this.identifyFrequentUpdates();
    const slowUpdates = this.identifySlowUpdates();

    // Check high subscription counts
    if (highSubscriptions.length > 0) {
      recommendations.push({
        category: 'store',
        severity: 'medium',
        title: `${highSubscriptions.length} stores with high subscription count`,
        description: `Top stores: ${highSubscriptions.slice(0, 3).map(s => `${s.storeName} (${s.subscriptionCount} subs)`).join(', ')}`,
        impact: 'Memory overhead, potential memory leaks',
        solution: 'Check for unsubscribed listeners, use derived stores to reduce subscriptions',
        estimatedGain: 'Reduce memory usage by 20-30%'
      });
    }

    // Check frequent updates
    if (frequentUpdates.length > 0) {
      recommendations.push({
        category: 'store',
        severity: 'high',
        title: `${frequentUpdates.length} stores updating too frequently`,
        description: `Top stores: ${frequentUpdates.slice(0, 3).map(s => `${s.storeName} (${s.updateCount} updates)`).join(', ')}`,
        impact: 'Excessive re-renders, poor performance',
        solution: 'Debounce updates, batch state changes, use throttling',
        estimatedGain: 'Reduce updates by 70-90%'
      });
    }

    // Check slow updates
    if (slowUpdates.length > 0) {
      recommendations.push({
        category: 'store',
        severity: 'medium',
        title: `${slowUpdates.length} stores with slow updates`,
        description: `Top stores: ${slowUpdates.slice(0, 3).map(s => `${s.storeName} (${s.averageUpdateTime.toFixed(2)}ms)`).join(', ')}`,
        impact: 'Delayed state updates, UI lag',
        solution: 'Optimize update logic, avoid synchronous expensive operations',
        estimatedGain: 'Up to 80% faster updates'
      });
    }

    return recommendations;
  }

  /**
   * Clear all metrics
   */
  clearMetrics(): void {
    this.storeMetrics.clear();
  }
}

/**
 * Debounced store update helper
 */
export function createDebouncedStore<T>(initialValue: T, delay = DEBOUNCE_THRESHOLD) {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let subscribers: Array<(value: T) => void> = [];
  let currentValue = initialValue;

  return {
    subscribe(fn: (value: T) => void): () => void {
      subscribers.push(fn);
      fn(currentValue); // Initial call

      return () => {
        subscribers = subscribers.filter(sub => sub !== fn);
      };
    },

    set(value: T): void {
      if (timeoutId !== null) {
        clearTimeout(timeoutId);
      }

      timeoutId = setTimeout(() => {
        currentValue = value;
        subscribers.forEach(fn => fn(value));
        timeoutId = null;
      }, delay);
    },

    update(fn: (value: T) => T): void {
      this.set(fn(currentValue));
    }
  };
}

// ============================================================================
// 4. NETWORK OPTIMIZATION (200 LINES)
// ============================================================================

/**
 * Monitors and optimizes network requests
 */
export class NetworkProfiler {
  private metrics: NetworkMetrics = {
    requestCount: 0,
    failedRequests: 0,
    averageLatency: 0,
    maxLatency: 0,
    minLatency: Infinity,
    totalDataTransferred: 0,
    cacheHitRate: 0
  };

  private latencies: number[] = [];
  private cacheHits = 0;
  private cacheMisses = 0;

  /**
   * Track a network request
   */
  async trackRequest<T>(
    requestFn: () => Promise<T>,
    size?: number
  ): Promise<T> {
    const startTime = performance.now();

    try {
      const result = await requestFn();
      const latency = performance.now() - startTime;

      this.recordSuccess(latency, size);
      return result;
    } catch (error) {
      const latency = performance.now() - startTime;
      this.recordFailure(latency);
      throw error;
    }
  }

  /**
   * Record successful request
   */
  private recordSuccess(latency: number, size?: number): void {
    this.metrics.requestCount++;
    this.latencies.push(latency);

    // Keep only last 100 latencies
    if (this.latencies.length > 100) {
      this.latencies.shift();
    }

    this.metrics.averageLatency = this.latencies.reduce((a, b) => a + b, 0) / this.latencies.length;
    this.metrics.maxLatency = Math.max(this.metrics.maxLatency, latency);
    this.metrics.minLatency = Math.min(this.metrics.minLatency, latency);

    if (size !== undefined) {
      this.metrics.totalDataTransferred += size;
    }
  }

  /**
   * Record failed request
   */
  private recordFailure(latency: number): void {
    this.metrics.requestCount++;
    this.metrics.failedRequests++;
    this.latencies.push(latency);
  }

  /**
   * Record cache hit
   */
  recordCacheHit(): void {
    this.cacheHits++;
    this.updateCacheHitRate();
  }

  /**
   * Record cache miss
   */
  recordCacheMiss(): void {
    this.cacheMisses++;
    this.updateCacheHitRate();
  }

  /**
   * Update cache hit rate
   */
  private updateCacheHitRate(): void {
    const total = this.cacheHits + this.cacheMisses;
    this.metrics.cacheHitRate = total > 0 ? this.cacheHits / total : 0;
  }

  /**
   * Get network metrics
   */
  getMetrics(): NetworkMetrics {
    return { ...this.metrics };
  }

  /**
   * Generate network optimization recommendations
   */
  analyzeNetworkOptimizations(): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];

    // Check high latency
    if (this.metrics.averageLatency > 1000) {
      recommendations.push({
        category: 'network',
        severity: 'high',
        title: 'High average latency',
        description: `Average latency: ${this.metrics.averageLatency.toFixed(0)}ms`,
        impact: 'Slow data loading, poor user experience',
        solution: 'Implement request batching, reduce payload size, add caching',
        estimatedGain: 'Reduce latency by 40-60%'
      });
    }

    // Check cache hit rate
    if (this.metrics.cacheHitRate < MIN_CACHE_HIT_RATE) {
      recommendations.push({
        category: 'network',
        severity: 'medium',
        title: 'Low cache hit rate',
        description: `Cache hit rate: ${(this.metrics.cacheHitRate * 100).toFixed(1)}%, target: ${MIN_CACHE_HIT_RATE * 100}%`,
        impact: 'Redundant network requests, higher data usage',
        solution: 'Implement proper caching strategy, use ETags, add service worker',
        estimatedGain: 'Reduce network requests by 30-50%'
      });
    }

    // Check failed requests
    const failureRate = this.metrics.requestCount > 0
      ? this.metrics.failedRequests / this.metrics.requestCount
      : 0;

    if (failureRate > 0.05) {
      recommendations.push({
        category: 'network',
        severity: 'high',
        title: 'High request failure rate',
        description: `${(failureRate * 100).toFixed(1)}% of requests failing`,
        impact: 'Data loss, poor reliability',
        solution: 'Add retry logic, improve error handling, check backend stability',
        estimatedGain: 'Improve success rate to >95%'
      });
    }

    return recommendations;
  }

  /**
   * Clear all metrics
   */
  clearMetrics(): void {
    this.metrics = {
      requestCount: 0,
      failedRequests: 0,
      averageLatency: 0,
      maxLatency: 0,
      minLatency: Infinity,
      totalDataTransferred: 0,
      cacheHitRate: 0
    };
    this.latencies = [];
    this.cacheHits = 0;
    this.cacheMisses = 0;
  }
}

/**
 * Request batcher for combining multiple requests
 */
export class RequestBatcher<T, R> {
  private queue: Array<{ data: T; resolve: (result: R) => void; reject: (error: any) => void }> = [];
  private timeoutId: ReturnType<typeof setTimeout> | null = null;
  private batchFn: (batch: T[]) => Promise<R[]>;
  private delay: number;

  constructor(batchFn: (batch: T[]) => Promise<R[]>, delay = 50) {
    this.batchFn = batchFn;
    this.delay = delay;
  }

  /**
   * Add request to batch
   */
  async request(data: T): Promise<R> {
    return new Promise((resolve, reject) => {
      this.queue.push({ data, resolve, reject });

      if (this.timeoutId !== null) {
        clearTimeout(this.timeoutId);
      }

      this.timeoutId = setTimeout(() => this.flush(), this.delay);
    });
  }

  /**
   * Flush the batch
   */
  private async flush(): Promise<void> {
    if (this.queue.length === 0) return;

    const batch = this.queue.splice(0);
    const data = batch.map(item => item.data);

    try {
      const results = await this.batchFn(data);

      batch.forEach((item, index) => {
        item.resolve(results[index]);
      });
    } catch (error) {
      batch.forEach(item => {
        item.reject(error);
      });
    }
  }
}

// ============================================================================
// 5. MEMORY MANAGEMENT (300 LINES)
// ============================================================================

/**
 * Monitors memory usage and detects leaks
 */
export class MemoryProfiler {
  private snapshots: MemoryMetrics[] = [];
  private intervalId: ReturnType<typeof setInterval> | null = null;

  /**
   * Start memory monitoring
   */
  startMonitoring(interval = 5000): () => void {
    this.intervalId = setInterval(() => {
      this.takeSnapshot();
    }, interval);

    return () => {
      if (this.intervalId !== null) {
        clearInterval(this.intervalId);
        this.intervalId = null;
      }
    };
  }

  /**
   * Take a memory snapshot
   */
  takeSnapshot(): MemoryMetrics | null {
    if (!performance.memory) {
      console.warn('Performance memory API not available');
      return null;
    }

    const metrics: MemoryMetrics = {
      jsHeapSize: performance.memory.jsHeapSizeLimit,
      jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
      totalJSHeapSize: performance.memory.totalJSHeapSize,
      usedJSHeapSize: performance.memory.usedJSHeapSize,
      percentUsed: (performance.memory.usedJSHeapSize / performance.memory.jsHeapSizeLimit) * 100,
      timestamp: Date.now()
    };

    this.snapshots.push(metrics);

    // Keep only last 100 snapshots
    if (this.snapshots.length > 100) {
      this.snapshots.shift();
    }

    return metrics;
  }

  /**
   * Get all snapshots
   */
  getSnapshots(): MemoryMetrics[] {
    return [...this.snapshots];
  }

  /**
   * Detect memory leaks
   */
  detectMemoryLeaks(threshold = 10): { leakDetected: boolean; trend: number; analysis: string } {
    if (this.snapshots.length < 10) {
      return {
        leakDetected: false,
        trend: 0,
        analysis: 'Insufficient data for leak detection'
      };
    }

    // Calculate trend over last 10 snapshots
    const recentSnapshots = this.snapshots.slice(-10);
    const first = recentSnapshots[0];
    const last = recentSnapshots[recentSnapshots.length - 1];

    const memoryGrowth = last.usedJSHeapSize - first.usedJSHeapSize;
    const percentGrowth = (memoryGrowth / first.usedJSHeapSize) * 100;

    const leakDetected = percentGrowth > threshold;

    let analysis = '';
    if (leakDetected) {
      analysis = `Memory increased by ${this.formatBytes(memoryGrowth)} (${percentGrowth.toFixed(1)}%) over ${recentSnapshots.length} samples. `;
      analysis += 'Possible causes: unsubscribed event listeners, retained DOM references, closure leaks.';
    } else {
      analysis = `Memory usage stable: ${percentGrowth.toFixed(1)}% growth over ${recentSnapshots.length} samples.`;
    }

    return {
      leakDetected,
      trend: percentGrowth,
      analysis
    };
  }

  /**
   * Generate memory optimization recommendations
   */
  analyzeMemoryOptimizations(): OptimizationRecommendation[] {
    const recommendations: OptimizationRecommendation[] = [];
    const latest = this.snapshots[this.snapshots.length - 1];

    if (!latest) {
      return recommendations;
    }

    // Check high memory usage
    if (latest.percentUsed > MAX_MEMORY_PERCENT) {
      recommendations.push({
        category: 'memory',
        severity: 'critical',
        title: 'High memory usage',
        description: `Using ${latest.percentUsed.toFixed(1)}% of available heap (${this.formatBytes(latest.usedJSHeapSize)} / ${this.formatBytes(latest.jsHeapSizeLimit)})`,
        impact: 'Risk of out-of-memory errors, slow performance',
        solution: 'Clear caches, unsubscribe listeners, limit data retention',
        estimatedGain: 'Reduce memory usage by 30-40%'
      });
    }

    // Check for memory leaks
    const leakDetection = this.detectMemoryLeaks();
    if (leakDetection.leakDetected) {
      recommendations.push({
        category: 'memory',
        severity: 'high',
        title: 'Potential memory leak detected',
        description: leakDetection.analysis,
        impact: 'Progressive memory increase, eventual crashes',
        solution: 'Audit event listeners, check store subscriptions, review component lifecycle',
        estimatedGain: 'Stabilize memory usage'
      });
    }

    return recommendations;
  }

  /**
   * Format bytes to human-readable string
   */
  private formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  /**
   * Clear snapshots
   */
  clearSnapshots(): void {
    this.snapshots = [];
  }
}

/**
 * Cleanup tracker for managing subscriptions and event listeners
 */
export class CleanupTracker {
  private cleanupFunctions: Array<() => void> = [];
  private componentName: string;

  constructor(componentName: string) {
    this.componentName = componentName;
  }

  /**
   * Register a cleanup function
   */
  register(cleanup: () => void): void {
    this.cleanupFunctions.push(cleanup);
  }

  /**
   * Register a store subscription
   */
  registerSubscription(unsubscribe: () => void): void {
    this.register(unsubscribe);
  }

  /**
   * Register an event listener
   */
  registerEventListener(
    target: EventTarget,
    event: string,
    handler: EventListener,
    options?: AddEventListenerOptions
  ): void {
    target.addEventListener(event, handler, options);
    this.register(() => {
      target.removeEventListener(event, handler, options);
    });
  }

  /**
   * Run all cleanup functions
   */
  cleanup(): void {
    this.cleanupFunctions.forEach(fn => {
      try {
        fn();
      } catch (error) {
        console.error(`Cleanup error in ${this.componentName}:`, error);
      }
    });
    this.cleanupFunctions = [];
  }

  /**
   * Get count of registered cleanup functions
   */
  getCleanupCount(): number {
    return this.cleanupFunctions.length;
  }
}

// ============================================================================
// MASTER PROFILER - COMBINES ALL PROFILING
// ============================================================================

/**
 * Master performance profiler that combines all optimization systems
 */
export class PerformanceProfiler {
  private bundleAnalyzer: BundleAnalyzer;
  private renderProfiler: RenderProfiler;
  private storeProfiler: StoreProfiler;
  private networkProfiler: NetworkProfiler;
  private memoryProfiler: MemoryProfiler;

  constructor() {
    this.bundleAnalyzer = new BundleAnalyzer();
    this.renderProfiler = new RenderProfiler();
    this.storeProfiler = new StoreProfiler();
    this.networkProfiler = new NetworkProfiler();
    this.memoryProfiler = new MemoryProfiler();
  }

  /**
   * Get all profilers
   */
  getProfilers() {
    return {
      bundle: this.bundleAnalyzer,
      render: this.renderProfiler,
      store: this.storeProfiler,
      network: this.networkProfiler,
      memory: this.memoryProfiler
    };
  }

  /**
   * Take a complete performance snapshot
   */
  async takeSnapshot(bundleManifest?: Record<string, any>): Promise<PerformanceSnapshot> {
    const bundleMetrics = bundleManifest
      ? this.bundleAnalyzer.recordBundleMetrics(bundleManifest)
      : {
          totalSize: 0,
          jsSize: 0,
          cssSize: 0,
          assetSize: 0,
          chunkCount: 0,
          largestChunks: [],
          timestamp: Date.now()
        };

    const memoryMetrics = this.memoryProfiler.takeSnapshot() || {
      jsHeapSize: 0,
      jsHeapSizeLimit: 0,
      totalJSHeapSize: 0,
      usedJSHeapSize: 0,
      percentUsed: 0,
      timestamp: Date.now()
    };

    return {
      bundle: bundleMetrics,
      renders: this.renderProfiler.getRenderMetrics(),
      stores: this.storeProfiler.getStoreMetrics(),
      network: this.networkProfiler.getMetrics(),
      memory: memoryMetrics,
      fps: this.renderProfiler.getCurrentFPS(),
      timestamp: Date.now()
    };
  }

  /**
   * Get all optimization recommendations
   */
  getAllRecommendations(): OptimizationRecommendation[] {
    return [
      ...this.renderProfiler.analyzeRenderingOptimizations(),
      ...this.storeProfiler.analyzeStoreOptimizations(),
      ...this.networkProfiler.analyzeNetworkOptimizations(),
      ...this.memoryProfiler.analyzeMemoryOptimizations()
    ].sort((a, b) => {
      const severityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
      return severityOrder[b.severity] - severityOrder[a.severity];
    });
  }

  /**
   * Generate comprehensive performance report
   */
  generateReport(): string {
    const recommendations = this.getAllRecommendations();
    const fps = this.renderProfiler.getCurrentFPS();
    const memory = this.memoryProfiler.getSnapshots();
    const latestMemory = memory[memory.length - 1];

    let report = '# Frontend Performance Report\n\n';
    report += `Generated: ${new Date().toISOString()}\n\n`;

    // Summary
    report += '## Summary\n\n';
    report += `- FPS: ${fps}\n`;
    if (latestMemory) {
      report += `- Memory Usage: ${latestMemory.percentUsed.toFixed(1)}%\n`;
    }
    report += `- Total Recommendations: ${recommendations.length}\n`;
    report += `  - Critical: ${recommendations.filter(r => r.severity === 'critical').length}\n`;
    report += `  - High: ${recommendations.filter(r => r.severity === 'high').length}\n`;
    report += `  - Medium: ${recommendations.filter(r => r.severity === 'medium').length}\n`;
    report += `  - Low: ${recommendations.filter(r => r.severity === 'low').length}\n\n`;

    // Recommendations
    if (recommendations.length > 0) {
      report += '## Optimization Recommendations\n\n';

      for (const rec of recommendations) {
        report += `### [${rec.severity.toUpperCase()}] ${rec.title}\n\n`;
        report += `**Category:** ${rec.category}\n\n`;
        report += `**Description:** ${rec.description}\n\n`;
        report += `**Impact:** ${rec.impact}\n\n`;
        report += `**Solution:** ${rec.solution}\n\n`;
        report += `**Estimated Gain:** ${rec.estimatedGain}\n\n`;
        report += '---\n\n';
      }
    } else {
      report += '## No Critical Issues Found\n\n';
      report += 'Performance is within acceptable parameters.\n\n';
    }

    return report;
  }

  /**
   * Clear all profiling data
   */
  clearAll(): void {
    this.renderProfiler.clearMetrics();
    this.storeProfiler.clearMetrics();
    this.networkProfiler.clearMetrics();
    this.memoryProfiler.clearSnapshots();
  }
}

// Export singleton instance
export const profiler = new PerformanceProfiler();
