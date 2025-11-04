/**
 * Type augmentations for Performance API extensions
 *
 * These types are for Chrome-specific APIs that may not be available
 * in all browsers. Code should check for existence before using.
 */

declare global {
  interface Performance {
    /**
     * Memory API (Chrome only)
     * Check availability: if (performance.memory) { ... }
     */
    memory?: {
      /** The maximum size of the heap, in bytes, available to the context */
      jsHeapSizeLimit: number;
      /** The total allocated heap size, in bytes */
      totalJSHeapSize: number;
      /** The currently active segment of JS heap, in bytes */
      usedJSHeapSize: number;
    };
  }
}

export {};
