/**
 * Tauri Environment Detection Utilities
 *
 * Provides functions to check if running in Tauri context vs browser
 */

// Declare Tauri types for Window
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: unknown;
    isTauri?: boolean;
  }
}

/**
 * Check if we're running inside a Tauri application
 * Uses multiple detection methods for compatibility with Tauri 2.x
 */
export function isTauri(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }

  // Tauri 2.x officially added window.isTauri (since beta.9)
  if ('isTauri' in window && window.isTauri === true) {
    return true;
  }

  // Check for __TAURI_INTERNALS__ (Tauri 2.x internal APIs)
  if (window.__TAURI_INTERNALS__ !== undefined) {
    return true;
  }

  // Check for __TAURI__ (set when withGlobalTauri is enabled)
  if (window.__TAURI__ !== undefined) {
    return true;
  }

  return false;
}

/**
 * Safe invoke wrapper that handles browser context gracefully
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T | null> {
  if (!isTauri()) {
    console.warn(`[Tauri] Cannot invoke '${command}' - not running in Tauri context`);
    return null;
  }

  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(command, args);
}

/**
 * Safe event listener that handles browser context gracefully
 */
export async function safeListen<T>(
  event: string,
  callback: (payload: T) => void
): Promise<(() => void) | null> {
  if (!isTauri()) {
    console.warn(`[Tauri] Cannot listen to '${event}' - not running in Tauri context`);
    return null;
  }

  const { listen } = await import('@tauri-apps/api/event');
  return listen<T>(event, (e) => callback(e.payload));
}

// Expose isTauri globally for debugging
if (typeof window !== 'undefined') {
  (window as unknown as { __IS_TAURI__: boolean }).__IS_TAURI__ = isTauri();
}
