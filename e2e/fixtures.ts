import { test as base } from '@playwright/test';

/**
 * Comprehensive Mock Tauri API for testing
 * Handles Tauri 2.0 API structure including @tauri-apps/api/* modules
 * This mock implements all required internals for the Tauri JS API to work
 */
const MOCK_TAURI = `
(function() {
  'use strict';

  // Callback registry for transformCallback
  const callbacks = new Map();
  let callbackId = 0;

  // Register a callback and return its ID
  function transformCallback(callback, once = false) {
    const id = callbackId++;
    if (callback) {
      callbacks.set(id, { callback, once });
    }
    return id;
  }

  // Run a callback by ID
  function runCallback(id, data) {
    const entry = callbacks.get(id);
    if (entry) {
      entry.callback(data);
      if (entry.once) {
        callbacks.delete(id);
      }
    }
  }

  // Unregister a callback
  function unregisterCallback(id) {
    callbacks.delete(id);
  }

  // Event listeners registry
  const eventListeners = new Map();
  let listenerId = 0;

  // Core invoke function with comprehensive mock responses
  async function mockInvoke(cmd, args = {}, options) {
    console.log('[Mock Tauri] invoke:', cmd, JSON.stringify(args).substring(0, 200));

    // Handle plugin:event|listen specially - MUST be before mockResponses check
    if (cmd === 'plugin:event|listen') {
      const id = listenerId++;
      if (args && args.event) {
        eventListeners.set(args.event, { id, handler: args.handler });
      }
      console.log('[Mock Tauri] Event listener registered:', args?.event, '-> id:', id);
      return id;
    }

    // Handle plugin:event|unlisten
    if (cmd === 'plugin:event|unlisten') {
      if (args && args.event) {
        eventListeners.delete(args.event);
      }
      return null;
    }

    // Handle plugin:event|emit
    if (cmd === 'plugin:event|emit') {
      if (args && args.event) {
        const listener = eventListeners.get(args.event);
        if (listener && listener.handler !== undefined) {
          // Call the handler via runCallback
          runCallback(listener.handler, { event: args.event, payload: args.payload });
        }
      }
      return null;
    }

    // Mock responses for all known commands
    const mockResponses = {
      // Preferences and settings
      'get_preferences': { theme: 'DARK', volume: 0.8, sampleRate: 44100 },
      'save_preferences': null,

      // Transport/playback
      'get_transport_state': { isPlaying: false, position: 0, bpm: 120, loop: false, recording: false },
      'play': null,
      'pause': null,
      'stop': null,
      'set_bpm': null,
      'set_loop': null,

      // Project/tracks
      'get_project': { name: 'Untitled', bpm: 120, timeSignature: '4/4', tracks: [] },
      'get_tracks': [],
      'create_track': { id: 'track-1', name: 'Track 1', type: 'midi' },
      'delete_track': null,
      'update_track': null,

      // MIDI I/O
      'midi_io_get_state': { inputs: [], outputs: [], sendClock: false },
      'midi_io_refresh_devices': { inputs: [], outputs: [] },
      'midi_io_connect_input': null,
      'midi_io_connect_output': null,

      // VIP3 Browser
      'vip3_get_filter_counts': { total: 0, bpm: {}, key: {}, instrument: {}, genre: {} },
      'vip3_get_all_files': [],
      'vip3_search': { files: [], total: 0 },
      'vip3_get_files': { files: [], total: 0 },
      'get_vip3_dynamic_filter_counts': { total_matches: 0, bpm_ranges: {}, keys: {}, timbres: {}, styles: {}, articulations: {} },

      // Window/dialog plugins
      'plugin:window|current': { label: 'main' },
      'plugin:window|set_fullscreen': null,
      'plugin:window|is_fullscreen': false,
      'plugin:window|minimize': null,
      'plugin:window|maximize': null,
      'plugin:window|close': null,
      'plugin:window|set_title': null,
      'plugin:dialog|save': '/tmp/test.mid',
      'plugin:dialog|open': null,
      'plugin:dialog|message': null,
      'plugin:fs|write_text_file': null,
      'plugin:fs|read_text_file': '',

      // Sequencer
      'get_sequencer_state': { clips: [], tracks: [], playhead: 0 },

      // Analysis
      'analyze_file': { bpm: 120, key: 'C', duration: 180 },
    };

    // Check for exact match
    if (cmd in mockResponses) {
      return mockResponses[cmd];
    }

    // Default to null for unknown commands
    console.log('[Mock Tauri] Unknown command, returning null:', cmd);
    return null;
  }

  // Set up __TAURI_INTERNALS__ - this is what the Tauri JS API uses
  window.__TAURI_INTERNALS__ = {
    invoke: mockInvoke,
    transformCallback: transformCallback,
    runCallback: runCallback,
    unregisterCallback: unregisterCallback,
    callbacks: callbacks,
    postMessage: (msg) => console.log('[Mock Tauri] postMessage:', msg),
    metadata: {
      currentWindow: { label: 'main' },
      currentWebview: { windowLabel: 'main', label: 'main' }
    },
    plugins: {},
    convertFileSrc: (path, protocol = 'asset') => path,
  };

  // Also set up for event plugin internals
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: (event, id) => {
      eventListeners.delete(event);
    }
  };

  // Legacy __TAURI__ object for backwards compatibility
  window.__TAURI__ = {
    invoke: mockInvoke,
    event: {
      listen: async (event, handler) => {
        const id = listenerId++;
        eventListeners.set(event, { id, handler });
        return () => eventListeners.delete(event);
      },
      once: async (event, handler) => {
        const id = listenerId++;
        eventListeners.set(event, { id, handler, once: true });
        return () => eventListeners.delete(event);
      },
      emit: async (event, payload) => {
        const listener = eventListeners.get(event);
        if (listener) {
          listener.handler({ event, payload });
          if (listener.once) eventListeners.delete(event);
        }
      }
    },
    tauri: { invoke: mockInvoke },
    core: { invoke: mockInvoke, transformCallback },
  };

  // Flag that we're in Tauri environment
  window.isTauri = true;

  console.log('[Mock Tauri] Initialized comprehensive mock with transformCallback support');
})();
`;

/**
 * Extended test fixture with Tauri mock and helper functions
 */
export const test = base.extend({
  page: async ({ page }, use) => {
    // Inject Tauri mock before any page loads
    await page.addInitScript(MOCK_TAURI);

    // Also intercept the main HTML to inject the script even earlier
    await page.route('**/index.html', async (route) => {
      const response = await route.fetch();
      const html = await response.text();
      // Inject mock script right after <head>
      const modifiedHtml = html.replace(
        '<head>',
        `<head><script>${MOCK_TAURI}</script>`
      );
      await route.fulfill({ response, body: modifiedHtml });
    });

    await use(page);
  },
});

export { expect } from '@playwright/test';

/**
 * Helper to wait for the app workspace to be ready
 * Returns true if workspace rendered, false if timeout
 *
 * Note: workspace element has width: 0 because all children are position: fixed
 * So we check for the element's existence rather than visibility,
 * or check for a visible child element (window-base)
 */
export async function waitForWorkspace(
  page: import('@playwright/test').Page,
  timeout = 5000
): Promise<boolean> {
  try {
    // Wait for the workspace element to exist (not necessarily visible since width=0)
    await page.waitForSelector('.workspace', { state: 'attached', timeout });
    // Also wait for at least one window to be visible (the actual UI)
    await page.waitForSelector('.window-base', { state: 'visible', timeout });
    return true;
  } catch {
    return false;
  }
}

/**
 * Helper to check if the app initialized successfully
 * Use this in beforeEach to skip tests when Tauri isn't available
 */
export async function ensureAppReady(
  page: import('@playwright/test').Page,
  test: import('@playwright/test').TestType<unknown, unknown>
): Promise<void> {
  const isReady = await waitForWorkspace(page, 10000);
  if (!isReady) {
    test.skip();
  }
}
