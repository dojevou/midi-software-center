# 🎹 KILO CODE GUIDE V2.0 - PART 2 (Sections 6-15)
## Continuation from KILO-CODE-GUIDE-V2-CORRECTED.md

**IMPORTANT**: Read KILO-CODE-GUIDE-V2-CORRECTED.md first (Sections 0-5)

---

## SECTION 6: EVENT LISTENERS

### FILE: `app/src/lib/api/events.ts`

```typescript
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ImportProgress, ImportSummary, PlaybackPosition } from '$lib/types';

// ============================================================================
// EVENT PAYLOAD TYPES
// ============================================================================

export interface AnalysisProgressPayload {
  current: number;
  total: number;
  current_file: string;
  rate: number;
}

export interface AnalysisSummaryPayload {
  total_analyzed: number;
  success: number;
  failed: number;
  duration_secs: number;
}

export interface ArchiveProgressPayload {
  current_file: string;
  extracted_count: number;
  total_count: number;
}

export interface ProgressStatePayload {
  state: 'idle' | 'running' | 'paused' | 'complete' | 'error';
  progress: number; // 0.0-1.0
  message: string;
}

export interface PlaybackPositionPayload {
  position: PlaybackPosition;
}

// ============================================================================
// EVENT LISTENER SETUP
// ============================================================================

export interface EventCallbacks {
  // Pipeline Events
  onPipelineProgress?: (progress: ImportProgress) => void;
  onPipelineComplete?: (summary: ImportSummary) => void;
  onAnalysisProgress?: (progress: AnalysisProgressPayload) => void;
  onAnalysisComplete?: (summary: AnalysisSummaryPayload) => void;
  onArchiveProgress?: (progress: ArchiveProgressPayload) => void;
  onArchiveError?: (error: { path: string; error: string }) => void;
  onProgressUpdate?: (state: ProgressStatePayload) => void;

  // Sequencer Events
  onPlaybackStarted?: () => void;
  onPlaybackStopped?: () => void;
  onPlaybackPaused?: () => void;
  onPlaybackPosition?: (payload: PlaybackPositionPayload) => void;
  onTrackAdded?: (trackId: number) => void;
  onTrackRemoved?: (trackId: number) => void;

  // Window Events
  onCommandToggleSidebar?: () => void;
  onCommandToggleInspector?: () => void;
}

/**
 * Setup all event listeners
 * Returns cleanup function to unlisten all events
 */
export async function setupEventListeners(callbacks: EventCallbacks): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  // Pipeline events
  if (callbacks.onPipelineProgress) {
    const unlisten = await listen<ImportProgress>('pipeline-progress', (event) => {
      callbacks.onPipelineProgress!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onPipelineComplete) {
    const unlisten = await listen<ImportSummary>('pipeline-complete', (event) => {
      callbacks.onPipelineComplete!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onAnalysisProgress) {
    const unlisten = await listen<AnalysisProgressPayload>('analysis-progress', (event) => {
      callbacks.onAnalysisProgress!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onAnalysisComplete) {
    const unlisten = await listen<AnalysisSummaryPayload>('analysis-complete', (event) => {
      callbacks.onAnalysisComplete!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onArchiveProgress) {
    const unlisten = await listen<ArchiveProgressPayload>('archive-progress', (event) => {
      callbacks.onArchiveProgress!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onArchiveError) {
    const unlisten = await listen<{ path: string; error: string }>('archive-error', (event) => {
      callbacks.onArchiveError!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onProgressUpdate) {
    const unlisten = await listen<ProgressStatePayload>('progress-update', (event) => {
      callbacks.onProgressUpdate!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  // Sequencer events
  if (callbacks.onPlaybackStarted) {
    const unlisten = await listen('playback-started', () => {
      callbacks.onPlaybackStarted!();
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onPlaybackStopped) {
    const unlisten = await listen('playback-stopped', () => {
      callbacks.onPlaybackStopped!();
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onPlaybackPaused) {
    const unlisten = await listen('playback-paused', () => {
      callbacks.onPlaybackPaused!();
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onPlaybackPosition) {
    const unlisten = await listen<PlaybackPositionPayload>('playback-position', (event) => {
      callbacks.onPlaybackPosition!(event.payload);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onTrackAdded) {
    const unlisten = await listen<{ track_id: number }>('track-added', (event) => {
      callbacks.onTrackAdded!(event.payload.track_id);
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onTrackRemoved) {
    const unlisten = await listen<{ track_id: number }>('track-removed', (event) => {
      callbacks.onTrackRemoved!(event.payload.track_id);
    });
    unlisteners.push(unlisten);
  }

  // Window command events
  if (callbacks.onCommandToggleSidebar) {
    const unlisten = await listen('command:toggle-sidebar', () => {
      callbacks.onCommandToggleSidebar!();
    });
    unlisteners.push(unlisten);
  }

  if (callbacks.onCommandToggleInspector) {
    const unlisten = await listen('command:toggle-inspector', () => {
      callbacks.onCommandToggleInspector!();
    });
    unlisteners.push(unlisten);
  }

  // Return cleanup function
  return () => {
    unlisteners.forEach(unlisten => unlisten());
  };
}
```

### FILE: `app/src/lib/api/errors.ts`

```typescript
/**
 * Custom error class for Tauri command failures
 */
export class TauriCommandError extends Error {
  constructor(
    message: string,
    public readonly command: string,
    public readonly originalError?: unknown
  ) {
    super(message);
    this.name = 'TauriCommandError';
  }
}

/**
 * Extract error message from unknown error type
 */
function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message);
  }
  return 'Unknown error occurred';
}

/**
 * Wrapper for Tauri commands with consistent error handling
 *
 * Usage:
 * const result = await handleTauriCommand(
 *   api.midi.listDevices(),
 *   'Failed to list MIDI devices'
 * );
 */
export async function handleTauriCommand<T>(
  commandPromise: Promise<T>,
  errorContext?: string
): Promise<T> {
  try {
    return await commandPromise;
  } catch (error) {
    const message = extractErrorMessage(error);
    const fullMessage = errorContext ? `${errorContext}: ${message}` : message;

    console.error(fullMessage, error);
    throw new TauriCommandError(fullMessage, errorContext || 'unknown', error);
  }
}
```

---

## SECTION 7: STORES (STATE MANAGEMENT)

### FILE: `app/src/lib/stores/index.ts`

```typescript
// Barrel exports for all stores

export * from './playbackStore';
export * from './projectStore';
export * from './databaseStore';
export * from './uiStore';
```

### FILE: `app/src/lib/stores/playbackStore.ts`

```typescript
import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { PlaybackPosition } from '$lib/types';

// ============================================================================
// PLAYBACK STATE
// ============================================================================

export interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  tempo: number;
  timeSignature: [number, number];
  keySignature: string;
  position: PlaybackPosition;
  loopEnabled: boolean;
  loopStart: number;
  loopEnd: number;
  metronomeEnabled: boolean;
  metronomeVolume: number;
}

const initialState: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  tempo: 120,
  timeSignature: [4, 4],
  keySignature: 'C',
  position: {
    current_tick: 0,
    current_bar: 0,
    current_beat: 0,
  },
  loopEnabled: false,
  loopStart: 0,
  loopEnd: 0,
  metronomeEnabled: false,
  metronomeVolume: 0.7,
};

export const playbackStore = writable<PlaybackState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const isPlayingOrPaused = derived(
  playbackStore,
  ($playback) => $playback.isPlaying || $playback.isPaused
);

export const formattedPosition = derived(
  playbackStore,
  ($playback) => {
    const { current_bar, current_beat } = $playback.position;
    return `${current_bar + 1}:${current_beat + 1}`;
  }
);

// ============================================================================
// ACTIONS
// ============================================================================

export const playbackActions = {
  async play() {
    await api.sequencer.start();
    playbackStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
  },

  async stop() {
    await api.sequencer.stop();
    playbackStore.update(state => ({
      ...state,
      isPlaying: false,
      isPaused: false,
      position: { current_tick: 0, current_bar: 0, current_beat: 0 }
    }));
  },

  async pause() {
    await api.sequencer.pause();
    playbackStore.update(state => ({ ...state, isPlaying: false, isPaused: true }));
  },

  async resume() {
    await api.sequencer.resume();
    playbackStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
  },

  async seek(bar: number, beat: number) {
    await api.sequencer.seekPosition(bar, beat);
    // Position update will come from backend event
  },

  async setTempo(bpm: number) {
    await api.sequencer.setTempo(bpm);
    playbackStore.update(state => ({ ...state, tempo: bpm }));
  },

  async setTimeSignature(numerator: number, denominator: number) {
    await api.window.setTimeSignature(numerator, denominator);
    playbackStore.update(state => ({ ...state, timeSignature: [numerator, denominator] }));
  },

  async setKeySignature(key: string) {
    await api.window.setKeySignature(key);
    playbackStore.update(state => ({ ...state, keySignature: key }));
  },

  async toggleLoop() {
    const currentState = get(playbackStore);
    await api.window.setLoopEnabled(!currentState.loopEnabled);
    playbackStore.update(state => ({ ...state, loopEnabled: !state.loopEnabled }));
  },

  async setLoopRange(start: number, end: number) {
    await api.window.setLoopRange(start, end);
    playbackStore.update(state => ({ ...state, loopStart: start, loopEnd: end }));
  },

  async toggleMetronome() {
    const currentState = get(playbackStore);
    await api.window.setMetronomeEnabled(!currentState.metronomeEnabled);
    playbackStore.update(state => ({ ...state, metronomeEnabled: !state.metronomeEnabled }));
  },

  async setMetronomeVolume(volume: number) {
    await api.window.setMetronomeVolume(volume);
    playbackStore.update(state => ({ ...state, metronomeVolume: volume }));
  },

  updatePosition(position: PlaybackPosition) {
    playbackStore.update(state => ({ ...state, position }));
  },
};
```

### FILE: `app/src/lib/stores/projectStore.ts`

```typescript
import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { Track, MidiPattern } from '$lib/types';

// ============================================================================
// PROJECT STATE ✅ NEW - Completely missing from v1.0
// ============================================================================

export interface ProjectState {
  tracks: Track[];
  selectedTrackId: number | null;
  clipboardContent: MidiPattern | null;
  hasUnsavedChanges: boolean;
  projectName: string;
}

const initialState: ProjectState = {
  tracks: [],
  selectedTrackId: null,
  clipboardContent: null,
  hasUnsavedChanges: false,
  projectName: 'Untitled Project',
};

export const projectStore = writable<ProjectState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const selectedTrack = derived(
  projectStore,
  ($project) => $project.tracks.find(t => t.id === $project.selectedTrackId) || null
);

export const trackCount = derived(
  projectStore,
  ($project) => $project.tracks.length
);

export const hasSelection = derived(
  projectStore,
  ($project) => $project.selectedTrackId !== null
);

// ============================================================================
// ACTIONS
// ============================================================================

export const projectActions = {
  async addTrack(fileId: number, channel: number) {
    const track = await api.sequencer.addTrack(fileId, channel);
    projectStore.update(state => ({
      ...state,
      tracks: [...state.tracks, track],
      hasUnsavedChanges: true,
    }));
    return track;
  },

  async removeTrack(trackId: number) {
    await api.sequencer.removeTrack(trackId);
    projectStore.update(state => ({
      ...state,
      tracks: state.tracks.filter(t => t.id !== trackId),
      selectedTrackId: state.selectedTrackId === trackId ? null : state.selectedTrackId,
      hasUnsavedChanges: true,
    }));
  },

  async updateTrack(trackId: number, properties: Partial<Track>) {
    await api.sequencer.updateTrack(trackId, properties);
    projectStore.update(state => ({
      ...state,
      tracks: state.tracks.map(t =>
        t.id === trackId ? { ...t, ...properties } : t
      ),
      hasUnsavedChanges: true,
    }));
  },

  selectTrack(trackId: number | null) {
    projectStore.update(state => ({ ...state, selectedTrackId: trackId }));
  },

  copyPattern(pattern: MidiPattern) {
    projectStore.update(state => ({ ...state, clipboardContent: pattern }));
  },

  pastePattern(): MidiPattern | null {
    const state = get(projectStore);
    return state.clipboardContent;
  },

  async loadTracks() {
    const tracks = await api.sequencer.getTracks();
    projectStore.update(state => ({ ...state, tracks }));
  },

  async clearAllTracks() {
    await api.project.clearAllTracks();
    projectStore.update(state => ({
      ...state,
      tracks: [],
      selectedTrackId: null,
      hasUnsavedChanges: true,
    }));
  },

  markSaved() {
    projectStore.update(state => ({ ...state, hasUnsavedChanges: false }));
  },

  markUnsaved() {
    projectStore.update(state => ({ ...state, hasUnsavedChanges: true }));
  },

  setProjectName(name: string) {
    projectStore.update(state => ({ ...state, projectName: name, hasUnsavedChanges: true }));
  },
};
```

### FILE: `app/src/lib/stores/databaseStore.ts`

```typescript
import { writable, derived } from 'svelte/store';
import { api } from '$lib/api';
import type { FileDetails, SearchFilters, SearchResponse } from '$lib/types';

// ============================================================================
// DATABASE STATE
// ============================================================================

export interface DatabaseState {
  searchResults: FileDetails[];
  totalCount: number;
  currentPage: number;
  pageSize: number;
  filters: SearchFilters;
  selectedFile: FileDetails | null;
  favorites: FileDetails[];
  isLoading: boolean;
  searchQuery: string;
}

const initialState: DatabaseState = {
  searchResults: [],
  totalCount: 0,
  currentPage: 0,
  pageSize: 50,
  filters: {},
  selectedFile: null,
  favorites: [],
  isLoading: false,
  searchQuery: '',
};

export const databaseStore = writable<DatabaseState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const totalPages = derived(
  databaseStore,
  ($database) => Math.ceil($database.totalCount / $database.pageSize)
);

export const hasResults = derived(
  databaseStore,
  ($database) => $database.searchResults.length > 0
);

export const hasSelection = derived(
  databaseStore,
  ($database) => $database.selectedFile !== null
);

// ============================================================================
// ACTIONS
// ============================================================================

export const databaseActions = {
  async search(filters?: SearchFilters, page?: number) {
    databaseStore.update(state => ({ ...state, isLoading: true }));

    try {
      const currentState = get(databaseStore);
      const searchFilters: SearchFilters = {
        ...currentState.filters,
        ...filters,
        limit: currentState.pageSize,
        offset: (page ?? currentState.currentPage) * currentState.pageSize,
      };

      const response: SearchResponse = await api.search.files(searchFilters);

      databaseStore.update(state => ({
        ...state,
        searchResults: response.files,
        totalCount: response.total,
        currentPage: page ?? state.currentPage,
        filters: searchFilters,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Search failed:', error);
      databaseStore.update(state => ({ ...state, isLoading: false }));
      throw error;
    }
  },

  async loadFile(fileId: number) {
    databaseStore.update(state => ({ ...state, isLoading: true }));

    try {
      const file = await api.search.getDetails(fileId);
      databaseStore.update(state => ({
        ...state,
        selectedFile: file,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load file:', error);
      databaseStore.update(state => ({ ...state, isLoading: false }));
      throw error;
    }
  },

  async addToFavorites(fileId: number) {
    await api.analysis.addFavorite(fileId);
    await databaseActions.loadFavorites();
  },

  async removeFromFavorites(fileId: number) {
    await api.analysis.removeFavorite(fileId);
    await databaseActions.loadFavorites();
  },

  async loadFavorites() {
    const favorites = await api.analysis.getFavorites();
    databaseStore.update(state => ({ ...state, favorites }));
  },

  setFilters(filters: Partial<SearchFilters>) {
    databaseStore.update(state => ({
      ...state,
      filters: { ...state.filters, ...filters },
    }));
  },

  clearSearch() {
    databaseStore.update(state => ({
      ...state,
      searchResults: [],
      totalCount: 0,
      currentPage: 0,
      searchQuery: '',
    }));
  },

  resetFilters() {
    databaseStore.update(state => ({
      ...state,
      filters: {},
      searchQuery: '',
    }));
  },

  nextPage() {
    const state = get(databaseStore);
    if (state.currentPage < Math.ceil(state.totalCount / state.pageSize) - 1) {
      databaseActions.search(state.filters, state.currentPage + 1);
    }
  },

  previousPage() {
    const state = get(databaseStore);
    if (state.currentPage > 0) {
      databaseActions.search(state.filters, state.currentPage - 1);
    }
  },

  setSearchQuery(query: string) {
    databaseStore.update(state => ({ ...state, searchQuery: query }));
  },
};

// Need to import get
import { get } from 'svelte/store';
```

### FILE: `app/src/lib/stores/uiStore.ts`

```typescript
import { writable, derived } from 'svelte/store';
import type { WindowId, WindowPosition } from '$lib/types';

// ============================================================================
// UI STATE
// ============================================================================

export interface UIState {
  windows: Record<WindowId, WindowPosition>;
  sidebarVisible: boolean;
  inspectorVisible: boolean;
  theme: 'dark' | 'light';
}

const initialState: UIState = {
  windows: {
    daw: {
      x: 50,
      y: 50,
      width: 800,
      height: 600,
      z_index: 1,
      visible: true,
    },
    mixer: {
      x: 900,
      y: 50,
      width: 400,
      height: 600,
      z_index: 2,
      visible: false,
    },
    database: {
      x: 50,
      y: 700,
      width: 600,
      height: 400,
      z_index: 3,
      visible: false,
    },
    pipeline: {
      x: 700,
      y: 700,
      width: 600,
      height: 400,
      z_index: 4,
      visible: false,
    },
  },
  sidebarVisible: true,
  inspectorVisible: true,
  theme: 'dark',
};

export const uiStore = writable<UIState>(initialState);

// Load from localStorage
if (typeof window !== 'undefined') {
  const saved = localStorage.getItem('ui-state');
  if (saved) {
    try {
      const parsed = JSON.parse(saved);
      uiStore.set({ ...initialState, ...parsed });
    } catch (e) {
      console.error('Failed to load UI state:', e);
    }
  }
}

// Save to localStorage on changes
uiStore.subscribe(state => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('ui-state', JSON.stringify(state));
  }
});

// ============================================================================
// DERIVED STORES
// ============================================================================

export const visibleWindows = derived(
  uiStore,
  ($ui) => Object.entries($ui.windows)
    .filter(([_, pos]) => pos.visible)
    .map(([id]) => id as WindowId)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const uiActions = {
  toggleWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: !state.windows[windowId].visible,
        },
      },
    }));
  },

  showWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: true,
        },
      },
    }));
  },

  hideWindow(windowId: WindowId) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          visible: false,
        },
      },
    }));
  },

  setWindowPosition(windowId: WindowId, x: number, y: number) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          x,
          y,
        },
      },
    }));
  },

  setWindowSize(windowId: WindowId, width: number, height: number) {
    uiStore.update(state => ({
      ...state,
      windows: {
        ...state.windows,
        [windowId]: {
          ...state.windows[windowId],
          width,
          height,
        },
      },
    }));
  },

  bringToFront(windowId: WindowId) {
    uiStore.update(state => {
      const maxZ = Math.max(...Object.values(state.windows).map(w => w.z_index));
      return {
        ...state,
        windows: {
          ...state.windows,
          [windowId]: {
            ...state.windows[windowId],
            z_index: maxZ + 1,
          },
        },
      };
    });
  },

  toggleSidebar() {
    uiStore.update(state => ({ ...state, sidebarVisible: !state.sidebarVisible }));
  },

  toggleInspector() {
    uiStore.update(state => ({ ...state, inspectorVisible: !state.inspectorVisible }));
  },

  setTheme(theme: 'dark' | 'light') {
    uiStore.update(state => ({ ...state, theme }));
  },
};
```

---

## SECTION 8: UTILITY FUNCTIONS

### FILE: `app/src/lib/utils/formatters.ts`

```typescript
/**
 * Format bytes to human-readable file size
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Format duration in seconds to MM:SS
 */
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format BPM with decimal places
 */
export function formatBPM(bpm: number | undefined): string {
  if (bpm === undefined) return 'N/A';
  return bpm.toFixed(2) + ' BPM';
}

/**
 * Format tick position to bar:beat:tick
 */
export function formatTick(tick: number, tpqn: number = 480): string {
  const ticksPerBeat = tpqn / 4; // Assuming 4/4
  const bar = Math.floor(tick / (tpqn * 4));
  const beat = Math.floor((tick % (tpqn * 4)) / ticksPerBeat);
  const tickInBeat = tick % ticksPerBeat;
  return `${bar + 1}:${beat + 1}:${Math.floor(tickInBeat)}`;
}

/**
 * Convert MIDI note number to name (e.g., 60 -> "C4")
 */
export function formatMidiNote(note: number): string {
  const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const octave = Math.floor(note / 12) - 1;
  const noteName = NOTE_NAMES[note % 12];
  return `${noteName}${octave}`;
}

/**
 * Format MIDI velocity as percentage
 */
export function formatVelocity(velocity: number): string {
  const percentage = Math.round((velocity / 127) * 100);
  return `${percentage}%`;
}

/**
 * Format MIDI channel (0-15 to 1-16)
 */
export function formatChannel(channel: number): string {
  return `Ch ${channel + 1}`;
}

/**
 * Format timestamp to local time
 */
export function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleString();
}
```

### FILE: `app/src/lib/utils/constants.ts` ✅ NEW - Missing from v1.0

```typescript
// ============================================================================
// MIDI CONSTANTS
// ============================================================================

export const MIDI_CHANNELS = 16;
export const NOTES_PER_OCTAVE = 12;
export const DEFAULT_TPQN = 480; // Ticks per quarter note
export const MIN_BPM = 30;
export const MAX_BPM = 300;
export const DEFAULT_BPM = 120;

export const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export const MUSICAL_KEYS = [
  'C', 'Cm', 'C#', 'C#m', 'D', 'Dm', 'D#', 'D#m',
  'E', 'Em', 'F', 'Fm', 'F#', 'F#m', 'G', 'Gm',
  'G#', 'G#m', 'A', 'Am', 'A#', 'A#m', 'B', 'Bm'
];

export const TIME_SIGNATURES = [
  [4, 4], [3, 4], [6, 8], [5, 4], [7, 8], [2, 4], [12, 8]
];

// ============================================================================
// FILE CATEGORIES
// ============================================================================

export const FILE_CATEGORIES = [
  'KICK', 'SNARE', 'HIHAT', 'PERCUSSION', 'BASS', 'LEAD',
  'PAD', 'CHORD', 'ARP', 'FX', 'VOCAL', 'LOOP', 'UNKNOWN'
] as const;

// ============================================================================
// UI CONSTANTS
// ============================================================================

export const DEFAULT_WINDOW_WIDTH = 800;
export const DEFAULT_WINDOW_HEIGHT = 600;
export const MIN_WINDOW_WIDTH = 400;
export const MIN_WINDOW_HEIGHT = 300;

// ============================================================================
// SEARCH CONSTANTS
// ============================================================================

export const DEFAULT_PAGE_SIZE = 50;
export const MAX_SEARCH_RESULTS = 500;

// ============================================================================
// PERFORMANCE CONSTANTS
// ============================================================================

export const DEBOUNCE_DELAY = 300; // ms
export const THROTTLE_INTERVAL = 100; // ms
```

---

**End of Part 2**

**Sections completed: 6-8 (Event Listeners, Stores, Utilities)**

**Next file**: Continue with Part 3 (Sections 9-15: Components, Windows, Testing, Security, Deployment)
