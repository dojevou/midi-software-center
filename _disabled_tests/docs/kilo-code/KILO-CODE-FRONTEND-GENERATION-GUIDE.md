# 🎹 COMPLETE FRONTEND GENERATION GUIDE FOR KILO CODE
## MIDI Software Center - Unified Application Frontend

**Version**: 1.0
**Date**: 2025-11-09
**Target**: ~10,000 lines of production-ready TypeScript + Svelte code
**Architecture**: Single unified app with 4 draggable windows (DAW, Mixer, Database, Pipeline)
**Zero Tolerance**: Every detail must match backend Rust types and command signatures EXACTLY

---

## 📋 TABLE OF CONTENTS

1. [Database & Environment Setup](#section-0-database--environment-setup)
2. [Project Structure](#section-1-project-structure--file-organization)
3. [Configuration Files](#section-2-configuration-files)
4. [Type Definitions](#section-3-type-definitions-critical)
5. [API Client](#section-4-api-client-all-backend-commands)
6. [Event Listeners](#section-5-event-listeners)
7. [Stores](#section-6-stores-state-management)
8. [Utility Functions](#section-7-utility-functions)
9. [Base Components](#section-8-base-components)
10. [Window Components](#section-9-window-components)
11. [Root Application](#section-10-root-application)
12. [Global Styles](#section-11-global-styles)
13. [Testing & Validation](#section-12-testing--validation)

---

## SECTION 0: DATABASE & ENVIRONMENT SETUP

### ✅ EXISTING SETUP (DO NOT MODIFY)

You already have:
- ✅ `.env` file at project root with correct DATABASE_URL
- ✅ Docker containers running (PostgreSQL on port 5433, Meilisearch on port 7700)
- ✅ Database migrations applied

**Your existing .env contains:**
```bash
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
```

**CRITICAL**: Port is **5433** (not 5432) - this is correct!

### VERIFICATION COMMANDS

Before starting frontend generation, verify:

```bash
# 1. Check Docker containers are running
docker ps | grep midi-library

# Expected: 2 containers (postgres, meilisearch) with "Up" status

# 2. Test database connection
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "SELECT COUNT(*) FROM files;"

# Expected: Returns a count (could be 0 if no files imported yet)
```

---

## SECTION 1: PROJECT STRUCTURE & FILE ORGANIZATION

Create this EXACT directory structure in `app/`:

```
app/
├── package.json                           # Frontend dependencies
├── vite.config.ts                         # Build configuration
├── tsconfig.json                          # TypeScript configuration
├── index.html                             # HTML entry point
│
├── src/
│   ├── main.ts                           # Application entry point
│   ├── App.svelte                        # Root component
│   ├── app.css                           # Global styles
│   │
│   └── lib/
│       ├── types/
│       │   └── index.ts                  # ALL TypeScript type definitions
│       │
│       ├── api/
│       │   ├── index.ts                  # Unified API client
│       │   ├── errors.ts                 # Error handling utilities
│       │   └── events.ts                 # Backend event listeners
│       │
│       ├── stores/
│       │   ├── index.ts                  # Store barrel exports
│       │   ├── playbackStore.ts          # Transport/playback state
│       │   ├── projectStore.ts           # Tracks/project state
│       │   ├── databaseStore.ts          # Search/files state
│       │   └── uiStore.ts                # Window positions/visibility
│       │
│       ├── components/
│       │   ├── MenuBar.svelte            # Top menu bar
│       │   ├── StatusBar.svelte          # Bottom status bar
│       │   └── WindowBase.svelte         # Draggable window wrapper
│       │
│       ├── windows/
│       │   ├── DAWWindow.svelte          # Main sequencer window
│       │   ├── MixerWindow.svelte        # Channel strips window
│       │   ├── DatabaseWindow.svelte     # File browser window
│       │   └── PipelineWindow.svelte     # Batch processing window
│       │
│       └── utils/
│           ├── formatters.ts             # Format functions
│           └── constants.ts              # App constants
│
└── src-tauri/
    ├── Cargo.toml                        # Rust dependencies
    ├── tauri.conf.json                   # Tauri configuration
    ├── build.rs                          # Build script
    │
    └── src/
        ├── main.rs                       # Backend entry point
        ├── lib.rs                        # Library exports
        └── commands/
            └── mod.rs                    # Command handlers
```

---

## SECTION 2: CONFIGURATION FILES

### FILE: `app/package.json`

```json
{
  "name": "midi-software-center",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "svelte": "^4.2.8"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.1",
    "@tauri-apps/cli": "^2.0.0",
    "svelte-check": "^3.6.2",
    "typescript": "^5.3.3",
    "vite": "^5.0.8"
  }
}
```

### FILE: `app/vite.config.ts`

```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],

  resolve: {
    alias: {
      '$lib': path.resolve(__dirname, './src/lib'),
      '@': path.resolve(__dirname, './src')
    }
  },

  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
  },

  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

### FILE: `app/tsconfig.json`

```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "resolveJsonModule": true,
    "allowJs": true,
    "checkJs": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "strict": true,
    "skipLibCheck": true,
    "paths": {
      "$lib/*": ["./src/lib/*"],
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.d.ts", "src/**/*.ts", "src/**/*.js", "src/**/*.svelte"]
}
```

### FILE: `app/index.html`

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>MIDI Software Center</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

### FILE: `app/src-tauri/tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2.0.0",
  "productName": "MIDI Software Center",
  "version": "1.0.0",
  "identifier": "com.midisoftwarecenter.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "MIDI Software Center",
        "width": 1600,
        "height": 1000,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "alwaysOnTop": false
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

### FILE: `app/src-tauri/Cargo.toml`

```toml
[package]
name = "midi-software-center"
version = "1.0.0"
edition = "2021"

[lib]
name = "midi_app"
path = "src/lib.rs"

[[bin]]
name = "midi-software-center"
path = "src/main.rs"

[dependencies]
tauri = { version = "2.0", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "migrate", "chrono", "bigdecimal"] }
tokio = { version = "1.35", features = ["full"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }

[build-dependencies]
tauri-build = { version = "2.0", features = [] }
```

### FILE: `app/src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build()
}
```

---

## SECTION 3: TYPE DEFINITIONS (CRITICAL)

### FILE: `app/src/lib/types/index.ts`

**CRITICAL**: These types MUST match Rust backend structures EXACTLY. Field names use snake_case in JSON due to Rust serde(rename). Optional fields in Rust become `| undefined` in TypeScript (NOT `| null`).

```typescript
// ============================================================================
// CORE DATABASE TYPES
// ============================================================================

/**
 * File metadata from database
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface FileMetadata {
  id: number;                          // i64 (BIGSERIAL)
  filename: string;                    // TEXT
  original_filename: string;           // TEXT
  filepath: string;                    // TEXT (unique)
  content_hash: string;                // BYTEA (hex-encoded string)
  file_size_bytes: number;             // BIGINT
  bpm: number | undefined;             // NUMERIC(6,2) | NULL
  key_signature: string | undefined;   // musical_key ENUM | NULL
}

/**
 * Complete file details with all metadata
 * Backend: daw/src-tauri/src/models/midi_file.rs
 */
export interface FileDetails {
  id: number;                          // i64
  file_name: string;                   // filename (serde rename)
  file_path: string;                   // filepath (serde rename)
  file_size: number;                   // file_size_bytes (serde rename)
  bpm?: number;                        // Option<f64>
  key?: string;                        // key_signature (serde rename)
  time_signature?: string;             // "4/4" format
  duration_seconds?: number;           // NUMERIC(10,3)
  total_notes?: number;                // INTEGER
  category?: string;                   // primary_category (serde rename)
  parent_folder?: string;              // Option<String>
  created_at: string;                  // ISO 8601 timestamp
  is_favorite: boolean;                // bool
  tags: string[];                      // Vec<String>
  manufacturer?: string;               // Option<String>
  collection?: string;                 // collection_name (serde rename)
  track_count: number;                 // num_tracks (i16)
  has_notes: boolean;                  // bool
  has_drums?: boolean;                 // Option<bool>
}

/**
 * Search filters for database queries
 * Backend: daw/src-tauri/src/models/search.rs
 */
export interface SearchFilters {
  min_bpm?: number;                    // Option<f64>
  max_bpm?: number;                    // Option<f64>
  key_signature?: string;              // Option<String>
  time_signature?: string;             // Option<String>
  category?: string;                   // Option<String>
  min_notes?: number;                  // Option<i32>
  max_notes?: number;                  // Option<i32>
  min_duration?: number;               // Option<f64> (seconds)
  max_duration?: number;               // Option<f64> (seconds)
  instruments?: string[];              // Option<Vec<String>>
  search_text?: string;                // Option<String>
  sort_by?: string;                    // Option<String>
  sort_desc?: boolean;                 // Option<bool>
  limit?: number;                      // Option<i64>
  offset?: number;                     // Option<i64>
}

/**
 * Search response from backend
 * Backend: daw/src-tauri/src/models/search.rs
 */
export interface SearchResponse {
  files: FileDetails[];                // Vec<FileDetails>
  total: number;                       // i64
}

/**
 * Import progress event payload
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface ImportProgress {
  current: number;                     // usize
  total: number;                       // usize
  current_file: string;                // String (CRITICAL: not "fileName")
  rate: number;                        // f64 (files per second)
}

/**
 * Import summary result
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface ImportSummary {
  total_files: number;                 // usize (CRITICAL: snake_case)
  imported: number;                    // usize
  skipped: number;                     // usize
  errors: string[];                    // Vec<String>
  duration_secs: number;               // f64 (CRITICAL: not "duration")
  rate: number;                        // f64
}

// ============================================================================
// SEQUENCER/PLAYBACK TYPES
// ============================================================================

/**
 * Sequencer track
 * Backend: daw/src-tauri/src/models/sequencer.rs
 */
export interface Track {
  id: number;                          // i32
  name: string;                        // String
  file_id: number;                     // i32
  channel: number;                     // u8 (0-15)
  muted: boolean;                      // bool (CRITICAL: "muted" not "mute")
  solo: boolean;                       // bool
  volume: number;                      // u8 (0-127)
  pan: number;                         // u8 (0-127, 64 = center)
  color: string;                       // String (hex color)
}

/**
 * Track properties for partial updates
 * Backend: daw/src-tauri/src/models/sequencer.rs
 */
export interface TrackProperties {
  name?: string;                       // Option<String>
  muted?: boolean;                     // Option<bool>
  solo?: boolean;                      // Option<bool>
  volume?: number;                     // Option<u8>
  pan?: number;                        // Option<u8>
  color?: string;                      // Option<String>
}

/**
 * Playback position
 * Backend: daw/src-tauri/src/models/sequencer.rs
 */
export interface PlaybackPosition {
  current_tick: number;                // u64 (CRITICAL: snake_case)
  current_bar: number;                 // u32
  current_beat: number;                // u32
}

// ============================================================================
// MIDI HARDWARE TYPES
// ============================================================================

/**
 * MIDI device information
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiDevice {
  name: string;                        // String
  manufacturer?: string;               // Option<String>
}

/**
 * MIDI event types
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export type MidiEventType =
  | 'NoteOn'
  | 'NoteOff'
  | 'ControlChange'
  | 'ProgramChange'
  | 'PitchBend'
  | 'Aftertouch';

/**
 * MIDI event
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiEvent {
  event_type: MidiEventType;           // MidiEventType enum
  tick: number;                        // u64
  channel: number;                     // u8
  note?: number;                       // Option<u8>
  velocity?: number;                   // Option<u8>
  controller?: number;                 // Option<u8>
  value?: number;                      // Option<u8>
  program?: number;                    // Option<u8>
}

/**
 * MIDI pattern
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiPattern {
  events: MidiEvent[];                 // Vec<MidiEvent>
  ticks_per_quarter_note: number;      // u16
  total_ticks: number;                 // u64
}

// ============================================================================
// ENUM CONSTANTS
// ============================================================================

/**
 * Musical keys (ENUM values from database)
 */
export const MUSICAL_KEYS = [
  'C', 'Cm', 'C#', 'C#m', 'Db', 'Dbm',
  'D', 'Dm', 'D#', 'D#m', 'Eb', 'Ebm',
  'E', 'Em', 'F', 'Fm', 'F#', 'F#m',
  'Gb', 'Gbm', 'G', 'Gm', 'G#', 'G#m',
  'Ab', 'Abm', 'A', 'Am', 'A#', 'A#m',
  'Bb', 'Bbm', 'B', 'Bm', 'UNKNOWN'
] as const;

export type MusicalKey = typeof MUSICAL_KEYS[number];

/**
 * File categories (ENUM values from database)
 */
export const FILE_CATEGORIES = [
  'KICK', 'SNARE', 'HIHAT', 'CLAP', 'PERC', 'TOM', 'CYMBAL',
  'DRUM_LOOP', 'DRUM_PATTERN', 'BASS', 'SUB_BASS', 'BASS_LOOP',
  'CHORD', 'PROGRESSION', 'STAB', 'PAD', 'TEXTURE', 'ATMOSPHERE',
  'LEAD', 'MELODY', 'HOOK', 'RIFF', 'ARP', 'SEQUENCE',
  'PIANO', 'KEYS', 'ORGAN', 'STRING', 'BRASS', 'WOODWIND',
  'FX', 'RISER', 'IMPACT', 'SWEEP', 'TRANSITION',
  'VOCAL', 'VOX', 'SAMPLE', 'MOTIF', 'THEME', 'FULL_MIX', 'STEM', 'UNKNOWN'
] as const;

export type FileCategory = typeof FILE_CATEGORIES[number];

// ============================================================================
// UI STATE TYPES
// ============================================================================

export type WindowId = 'daw' | 'mixer' | 'database' | 'pipeline';

export interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
  visible: boolean;
  minimized: boolean;
  zIndex: number;
}

export interface WindowState {
  [key: string]: WindowPosition;
}
```

---

## SECTION 4: API CLIENT (ALL BACKEND COMMANDS)

### FILE: `app/src/lib/api/errors.ts`

```typescript
/**
 * Error handling utilities for Tauri API calls
 */

/**
 * Extract error message from unknown error type
 */
export function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message);
  }
  return 'Unknown error occurred';
}

/**
 * Wrap Tauri command in error handling
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
    throw new Error(fullMessage);
  }
}
```

### FILE: `app/src/lib/api/index.ts`

**CRITICAL**: All command names are snake_case. All parameter object keys are snake_case.

```typescript
/**
 * Unified API client for all Tauri backend commands
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Track, TrackProperties, PlaybackPosition,
  MidiDevice, MidiPattern, FileDetails,
  SearchFilters, SearchResponse, FileMetadata, ImportSummary
} from '$lib/types';

// ============================================================================
// SYSTEM COMMANDS
// ============================================================================

export const initializeDatabase = (): Promise<void> =>
  invoke('initialize_database');

// ============================================================================
// MIDI DEVICE COMMANDS
// ============================================================================

export const midiListDevices = (): Promise<MidiDevice[]> =>
  invoke('midi_list_devices');

export const midiConnect = (deviceName: string): Promise<void> =>
  invoke('midi_connect', { device_name: deviceName });

export const midiDisconnect = (): Promise<void> =>
  invoke('midi_disconnect');

export const midiIsConnected = (): Promise<boolean> =>
  invoke('midi_is_connected');

export const midiGetCurrentDevice = (): Promise<MidiDevice | null> =>
  invoke('midi_get_current_device');

export const midiSendTestNote = (note: number, velocity: number): Promise<void> =>
  invoke('midi_send_test_note', { note, velocity });

// ============================================================================
// SEQUENCER/PLAYBACK COMMANDS
// ============================================================================

export const startSequencer = (): Promise<void> =>
  invoke('start_sequencer');

export const stopSequencer = (): Promise<void> =>
  invoke('stop_sequencer');

export const pauseSequencer = (): Promise<void> =>
  invoke('pause_sequencer');

export const resumeSequencer = (): Promise<void> =>
  invoke('resume_sequencer');

export const isSequencerPlaying = (): Promise<boolean> =>
  invoke('is_sequencer_playing');

export const getPlaybackPosition = (): Promise<PlaybackPosition> =>
  invoke('get_playback_position');

export const seekPosition = (tick: number): Promise<void> =>
  invoke('seek_position', { tick });

export const setTempo = (bpm: number): Promise<void> =>
  invoke('set_tempo', { bpm });

export const getTempo = (): Promise<number> =>
  invoke('get_tempo');

export const addTrack = (fileId: number, channel: number): Promise<Track> =>
  invoke('add_track', { file_id: fileId, channel });

export const removeTrack = (trackId: number): Promise<void> =>
  invoke('remove_track', { track_id: trackId });

export const updateTrack = (trackId: number, properties: TrackProperties): Promise<Track> =>
  invoke('update_track', { track_id: trackId, properties });

export const getTracks = (): Promise<Track[]> =>
  invoke('get_tracks');

export const loadSequencerTracks = (fileIds: number[]): Promise<Track[]> =>
  invoke('load_sequencer_tracks', { file_ids: fileIds });

// ============================================================================
// SEARCH/DATABASE COMMANDS
// ============================================================================

export const searchFiles = (filters: SearchFilters): Promise<SearchResponse> =>
  invoke('search_files', { filters });

export const getFileDetails = (fileId: number): Promise<FileDetails> =>
  invoke('get_file_details', { file_id: fileId });

export const getSearchSuggestions = (query: string, limit: number = 10): Promise<string[]> =>
  invoke('get_search_suggestions', { query, limit });

// ============================================================================
// FILE IMPORT COMMANDS
// ============================================================================

export const importSingleFile = (filePath: string): Promise<FileMetadata> =>
  invoke('import_single_file', { file_path: filePath });

export const importDirectory = (directoryPath: string, recursive: boolean): Promise<ImportSummary> =>
  invoke('import_directory', { directory_path: directoryPath, recursive });

// ============================================================================
// ANALYSIS/FAVORITES COMMANDS
// ============================================================================

export const addFavorite = (fileId: number): Promise<void> =>
  invoke('add_favorite', { file_id: fileId });

export const removeFavorite = (fileId: number): Promise<void> =>
  invoke('remove_favorite', { file_id: fileId });

export const isFavorite = (fileId: number): Promise<boolean> =>
  invoke('is_favorite', { file_id: fileId });

export const getFavorites = (): Promise<FileDetails[]> =>
  invoke('get_favorites');

// ============================================================================
// PROJECT COMMANDS
// ============================================================================

export const loadMultipleTracks = (fileIds: number[]): Promise<Track[]> =>
  invoke('load_multiple_tracks', { file_ids: fileIds });

export const clearAllTracks = (): Promise<void> =>
  invoke('clear_all_tracks');

export const getTrackDetails = (trackId: number): Promise<Track> =>
  invoke('get_track_details', { track_id: trackId });

// ============================================================================
// FILE PATTERN COMMANDS
// ============================================================================

export const loadPattern = (fileId: number): Promise<MidiPattern> =>
  invoke('load_pattern', { file_id: fileId });

// ============================================================================
// UNIFIED API OBJECT (Organized by namespace)
// ============================================================================

export const api = {
  system: {
    initializeDatabase,
  },

  midi: {
    listDevices: midiListDevices,
    connect: midiConnect,
    disconnect: midiDisconnect,
    isConnected: midiIsConnected,
    getCurrentDevice: midiGetCurrentDevice,
    sendTestNote: midiSendTestNote,
  },

  sequencer: {
    start: startSequencer,
    stop: stopSequencer,
    pause: pauseSequencer,
    resume: resumeSequencer,
    isPlaying: isSequencerPlaying,
    getPosition: getPlaybackPosition,
    seek: seekPosition,
    setTempo,
    getTempo,
    addTrack,
    removeTrack,
    updateTrack,
    getTracks,
    loadTracks: loadSequencerTracks,
  },

  search: {
    files: searchFiles,
    getDetails: getFileDetails,
    getSuggestions: getSearchSuggestions,
  },

  files: {
    importSingle: importSingleFile,
    importDirectory,
    loadPattern,
  },

  analysis: {
    addFavorite,
    removeFavorite,
    isFavorite,
    getFavorites,
  },

  project: {
    loadMultipleTracks,
    clearAllTracks,
    getTrackDetails,
  },
};
```

---

## SECTION 5: EVENT LISTENERS

### FILE: `app/src/lib/api/events.ts`

**CRITICAL**: Event names are kebab-case. Payloads match backend emissions exactly.

```typescript
/**
 * Backend event listener setup
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { PlaybackPosition, Track, ImportProgress, ImportSummary, MidiDevice } from '$lib/types';

const unlistenFunctions: UnlistenFn[] = [];

/**
 * Setup all backend event listeners
 * Call this ONCE during app initialization
 */
export async function setupEventListeners(callbacks: {
  onPlaybackStarted?: () => void;
  onPlaybackStopped?: () => void;
  onPlaybackPaused?: () => void;
  onPlaybackPosition?: (position: PlaybackPosition) => void;
  onTempoChanged?: (tempo: { bpm: number }) => void;
  onTrackAdded?: (track: Track) => void;
  onTrackRemoved?: (data: { track_id: number }) => void;
  onTrackUpdated?: (track: Track) => void;
  onPipelineProgress?: (progress: ImportProgress) => void;
  onPipelineFile?: (data: { file_id: number; file_path: string }) => void;
  onPipelineError?: (data: { file_path: string; error: string }) => void;
  onPipelineComplete?: (summary: ImportSummary) => void;
  onMidiConnected?: (device: MidiDevice) => void;
  onMidiDisconnected?: () => void;
}) {
  // Playback events
  if (callbacks.onPlaybackStarted) {
    const unlisten = await listen('playback-started', () => callbacks.onPlaybackStarted!());
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPlaybackStopped) {
    const unlisten = await listen('playback-stopped', () => callbacks.onPlaybackStopped!());
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPlaybackPaused) {
    const unlisten = await listen('playback-paused', () => callbacks.onPlaybackPaused!());
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPlaybackPosition) {
    const unlisten = await listen<PlaybackPosition>('playback-position', (event) => {
      callbacks.onPlaybackPosition!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onTempoChanged) {
    const unlisten = await listen<{ bpm: number }>('tempo-changed', (event) => {
      callbacks.onTempoChanged!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  // Track events
  if (callbacks.onTrackAdded) {
    const unlisten = await listen<Track>('track-added', (event) => {
      callbacks.onTrackAdded!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onTrackRemoved) {
    const unlisten = await listen<{ track_id: number }>('track-removed', (event) => {
      callbacks.onTrackRemoved!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onTrackUpdated) {
    const unlisten = await listen<Track>('track-updated', (event) => {
      callbacks.onTrackUpdated!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  // Pipeline events
  if (callbacks.onPipelineProgress) {
    const unlisten = await listen<ImportProgress>('pipeline-progress', (event) => {
      callbacks.onPipelineProgress!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPipelineFile) {
    const unlisten = await listen<{ file_id: number; file_path: string }>('pipeline-file', (event) => {
      callbacks.onPipelineFile!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPipelineError) {
    const unlisten = await listen<{ file_path: string; error: string }>('pipeline-error', (event) => {
      callbacks.onPipelineError!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onPipelineComplete) {
    const unlisten = await listen<ImportSummary>('pipeline-complete', (event) => {
      callbacks.onPipelineComplete!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  // MIDI device events
  if (callbacks.onMidiConnected) {
    const unlisten = await listen<MidiDevice>('midi-connected', (event) => {
      callbacks.onMidiConnected!(event.payload);
    });
    unlistenFunctions.push(unlisten);
  }

  if (callbacks.onMidiDisconnected) {
    const unlisten = await listen('midi-disconnected', () => callbacks.onMidiDisconnected!());
    unlistenFunctions.push(unlisten);
  }
}

/**
 * Cleanup all event listeners
 */
export function cleanupEventListeners() {
  unlistenFunctions.forEach(unlisten => unlisten());
  unlistenFunctions.length = 0;
}
```

---

## SECTION 6: STORES (STATE MANAGEMENT)

### FILE: `app/src/lib/stores/index.ts`

```typescript
/**
 * Store barrel exports
 */

export { playbackStore, playbackActions } from './playbackStore';
export { projectStore, projectActions } from './projectStore';
export { databaseStore, databaseActions } from './databaseStore';
export { uiStore, uiActions } from './uiStore';

export type { PlaybackState } from './playbackStore';
export type { ProjectState } from './projectStore';
export type { DatabaseState } from './databaseStore';
export type { UIState } from './uiStore';
```

### FILE: `app/src/lib/stores/playbackStore.ts`

```typescript
/**
 * Playback Store - Transport controls and playback state
 */

import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { PlaybackPosition } from '$lib/types';

// ============================================================================
// STATE INTERFACE
// ============================================================================

export interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  position: PlaybackPosition;
  tempo: number;
  timeSignature: { numerator: number; denominator: number };
  keySignature: string;
  loopEnabled: boolean;
  loopStart: number; // tick
  loopEnd: number;   // tick
  metronomeEnabled: boolean;
  loading: boolean;
  error: string | null;
}

const initialState: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  position: {
    current_tick: 0,
    current_bar: 1,
    current_beat: 1,
  },
  tempo: 120,
  timeSignature: { numerator: 4, denominator: 4 },
  keySignature: 'C',
  loopEnabled: false,
  loopStart: 0,
  loopEnd: 0,
  metronomeEnabled: false,
  loading: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<PlaybackState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const formattedPosition = derived(
  { subscribe },
  ($state) => {
    const bar = String($state.position.current_bar).padStart(3, '0');
    const beat = String($state.position.current_beat).padStart(1, '0');
    const tick = String($state.position.current_tick % 480).padStart(3, '0');
    return `${bar}.${beat}.${tick}`;
  }
);

export const formattedTimeSignature = derived(
  { subscribe },
  ($state) => `${$state.timeSignature.numerator}/${$state.timeSignature.denominator}`
);

// ============================================================================
// ACTIONS
// ============================================================================

export const playbackActions = {
  async play() {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.start();
      update(s => ({ ...s, isPlaying: true, isPaused: false, loading: false }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async pause() {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.pause();
      update(s => ({ ...s, isPlaying: false, isPaused: true, loading: false }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async stop() {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.stop();
      update(s => ({
        ...s,
        isPlaying: false,
        isPaused: false,
        position: { current_tick: 0, current_bar: 1, current_beat: 1 },
        loading: false,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async togglePlayPause() {
    const current = get({ subscribe });
    if (current.isPlaying) {
      await playbackActions.pause();
    } else {
      await playbackActions.play();
    }
  },

  async setTempo(bpm: number) {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.setTempo(bpm);
      update(s => ({ ...s, tempo: bpm, loading: false }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async seek(tick: number) {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.seek(tick);
      update(s => ({ ...s, loading: false }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  toggleLoop() {
    update(s => ({ ...s, loopEnabled: !s.loopEnabled }));
  },

  setLoopRange(start: number, end: number) {
    update(s => ({ ...s, loopStart: start, loopEnd: end }));
  },

  toggleMetronome() {
    update(s => ({ ...s, metronomeEnabled: !s.metronomeEnabled }));
  },

  updatePosition(position: PlaybackPosition) {
    update(s => ({ ...s, position }));
  },
};

export const playbackStore = { subscribe };
```

### FILE: `app/src/lib/stores/projectStore.ts`

```typescript
/**
 * Project Store - Track and project state management
 */

import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { Track, TrackProperties } from '$lib/types';

// ============================================================================
// STATE INTERFACE
// ============================================================================

export interface ProjectState {
  tracks: Track[];
  selectedTrackId: number | null;
  loading: boolean;
  error: string | null;
}

const initialState: ProjectState = {
  tracks: [],
  selectedTrackId: null,
  loading: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<ProjectState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const selectedTrack = derived(
  { subscribe },
  ($state) => $state.tracks.find(t => t.id === $state.selectedTrackId) || null
);

export const soloedTracks = derived(
  { subscribe },
  ($state) => $state.tracks.filter(t => t.solo)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const projectActions = {
  async addTrack(fileId: number, channel: number = 0) {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      const track = await api.sequencer.addTrack(fileId, channel);
      update(s => ({
        ...s,
        tracks: [...s.tracks, track],
        loading: false,
      }));
      return track;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
      throw error;
    }
  },

  async removeTrack(trackId: number) {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      await api.sequencer.removeTrack(trackId);
      update(s => ({
        ...s,
        tracks: s.tracks.filter(t => t.id !== trackId),
        selectedTrackId: s.selectedTrackId === trackId ? null : s.selectedTrackId,
        loading: false,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async updateTrack(trackId: number, properties: TrackProperties) {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      const updatedTrack = await api.sequencer.updateTrack(trackId, properties);
      update(s => ({
        ...s,
        tracks: s.tracks.map(t => t.id === trackId ? updatedTrack : t),
        loading: false,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  async loadTracks() {
    update(s => ({ ...s, loading: true, error: null }));
    try {
      const tracks = await api.sequencer.getTracks();
      update(s => ({ ...s, tracks, loading: false }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  selectTrack(trackId: number | null) {
    update(s => ({ ...s, selectedTrackId: trackId }));
  },

  addTrackFromEvent(track: Track) {
    update(s => ({
      ...s,
      tracks: [...s.tracks, track],
    }));
  },

  removeTrackFromEvent(trackId: number) {
    update(s => ({
      ...s,
      tracks: s.tracks.filter(t => t.id !== trackId),
      selectedTrackId: s.selectedTrackId === trackId ? null : s.selectedTrackId,
    }));
  },

  updateTrackFromEvent(track: Track) {
    update(s => ({
      ...s,
      tracks: s.tracks.map(t => t.id === track.id ? track : t),
    }));
  },
};

export const projectStore = { subscribe };
```

### FILE: `app/src/lib/stores/databaseStore.ts`

```typescript
/**
 * Database Store - File search and browsing state
 */

import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { SearchFilters, SearchResponse, FileDetails } from '$lib/types';

// ============================================================================
// STATE INTERFACE
// ============================================================================

export interface DatabaseState {
  searchQuery: string;
  filters: SearchFilters;
  results: FileDetails[];
  total: number;
  currentPage: number;
  pageSize: number;
  selectedFileId: number | null;
  loading: boolean;
  error: string | null;
}

const initialState: DatabaseState = {
  searchQuery: '',
  filters: {},
  results: [],
  total: 0,
  currentPage: 1,
  pageSize: 50,
  selectedFileId: null,
  loading: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<DatabaseState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const totalPages = derived(
  { subscribe },
  ($state) => Math.ceil($state.total / $state.pageSize)
);

export const selectedFile = derived(
  { subscribe },
  ($state) => $state.results.find(f => f.id === $state.selectedFileId) || null
);

// ============================================================================
// ACTIONS
// ============================================================================

export const databaseActions = {
  async search() {
    const current = get({ subscribe });
    update(s => ({ ...s, loading: true, error: null }));

    try {
      const filters: SearchFilters = {
        ...current.filters,
        search_text: current.searchQuery || undefined,
        limit: current.pageSize,
        offset: (current.currentPage - 1) * current.pageSize,
      };

      const response = await api.search.files(filters);
      update(s => ({
        ...s,
        results: response.files,
        total: response.total,
        loading: false,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, loading: false, error: message }));
    }
  },

  setSearchQuery(query: string) {
    update(s => ({ ...s, searchQuery: query, currentPage: 1 }));
  },

  setFilters(filters: Partial<SearchFilters>) {
    update(s => ({
      ...s,
      filters: { ...s.filters, ...filters },
      currentPage: 1,
    }));
  },

  clearFilters() {
    update(s => ({
      ...s,
      filters: {},
      searchQuery: '',
      currentPage: 1,
    }));
  },

  setPage(page: number) {
    update(s => ({ ...s, currentPage: page }));
  },

  nextPage() {
    const current = get({ subscribe });
    const maxPage = Math.ceil(current.total / current.pageSize);
    if (current.currentPage < maxPage) {
      update(s => ({ ...s, currentPage: s.currentPage + 1 }));
    }
  },

  previousPage() {
    const current = get({ subscribe });
    if (current.currentPage > 1) {
      update(s => ({ ...s, currentPage: s.currentPage - 1 }));
    }
  },

  selectFile(fileId: number | null) {
    update(s => ({ ...s, selectedFileId: fileId }));
  },

  async toggleFavorite(fileId: number) {
    try {
      const isFav = await api.analysis.isFavorite(fileId);
      if (isFav) {
        await api.analysis.removeFavorite(fileId);
      } else {
        await api.analysis.addFavorite(fileId);
      }
      // Refresh results
      await databaseActions.search();
    } catch (error) {
      console.error('Toggle favorite failed:', error);
    }
  },
};

export const databaseStore = { subscribe };
```

### FILE: `app/src/lib/stores/uiStore.ts`

```typescript
/**
 * UI Store - Window positions and visibility
 */

import { writable, derived } from 'svelte/store';
import type { WindowState, WindowId } from '$lib/types';

// ============================================================================
// STATE INTERFACE
// ============================================================================

export interface UIState {
  windows: WindowState;
  gridSnap: boolean;
  gridSize: number;
}

const defaultWindowPositions: WindowState = {
  daw: {
    x: 50,
    y: 100,
    width: 1200,
    height: 700,
    visible: true,
    minimized: false,
    zIndex: 1,
  },
  mixer: {
    x: 1300,
    y: 100,
    width: 400,
    height: 700,
    visible: false,
    minimized: false,
    zIndex: 1,
  },
  database: {
    x: 50,
    y: 850,
    width: 800,
    height: 500,
    visible: true,
    minimized: false,
    zIndex: 1,
  },
  pipeline: {
    x: 900,
    y: 850,
    width: 800,
    height: 500,
    visible: false,
    minimized: false,
    zIndex: 1,
  },
};

// Load from localStorage
const loadState = (): UIState => {
  if (typeof window === 'undefined') {
    return {
      windows: defaultWindowPositions,
      gridSnap: true,
      gridSize: 10,
    };
  }

  try {
    const saved = localStorage.getItem('uiState');
    if (saved) {
      const parsed = JSON.parse(saved);
      return {
        windows: { ...defaultWindowPositions, ...parsed.windows },
        gridSnap: parsed.gridSnap ?? true,
        gridSize: parsed.gridSize ?? 10,
      };
    }
  } catch (error) {
    console.error('Failed to load UI state:', error);
  }

  return {
    windows: defaultWindowPositions,
    gridSnap: true,
    gridSize: 10,
  };
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<UIState>(loadState());

// Save to localStorage on every update
subscribe(state => {
  if (typeof window !== 'undefined') {
    try {
      localStorage.setItem('uiState', JSON.stringify(state));
    } catch (error) {
      console.error('Failed to save UI state:', error);
    }
  }
});

// ============================================================================
// DERIVED STORES
// ============================================================================

export const visibleWindows = derived(
  { subscribe },
  ($state) => Object.entries($state.windows)
    .filter(([_, window]) => window.visible && !window.minimized)
    .map(([id]) => id as WindowId)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const uiActions = {
  showWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          visible: true,
          minimized: false,
          zIndex: Math.max(...Object.values(s.windows).map(w => w.zIndex)) + 1,
        },
      },
    }));
  },

  hideWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          visible: false,
        },
      },
    }));
  },

  toggleWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          visible: !s.windows[windowId].visible,
          minimized: false,
        },
      },
    }));
  },

  minimizeWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          minimized: true,
        },
      },
    }));
  },

  restoreWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          minimized: false,
          zIndex: Math.max(...Object.values(s.windows).map(w => w.zIndex)) + 1,
        },
      },
    }));
  },

  setWindowPosition(windowId: WindowId, x: number, y: number) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          x: s.gridSnap ? Math.round(x / s.gridSize) * s.gridSize : x,
          y: s.gridSnap ? Math.round(y / s.gridSize) * s.gridSize : y,
        },
      },
    }));
  },

  setWindowSize(windowId: WindowId, width: number, height: number) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          width,
          height,
        },
      },
    }));
  },

  focusWindow(windowId: WindowId) {
    update(s => ({
      ...s,
      windows: {
        ...s.windows,
        [windowId]: {
          ...s.windows[windowId],
          zIndex: Math.max(...Object.values(s.windows).map(w => w.zIndex)) + 1,
        },
      },
    }));
  },

  toggleGridSnap() {
    update(s => ({ ...s, gridSnap: !s.gridSnap }));
  },

  setGridSize(size: number) {
    update(s => ({ ...s, gridSize: size }));
  },
};

export const uiStore = { subscribe };
```

---

## SECTION 7: UTILITY FUNCTIONS

### FILE: `app/src/lib/utils/formatters.ts`

```typescript
/**
 * Formatting utility functions
 */

/**
 * Format duration in seconds to MM:SS
 */
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format file size in bytes to human readable
 */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format timestamp to relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return 'just now';
  if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
  return date.toLocaleDateString();
}

/**
 * Convert volume (0-127) to dB
 */
export function volumeToDb(volume: number): string {
  if (volume === 0) return '-∞';
  const db = 20 * Math.log10(volume / 127);
  return `${db.toFixed(1)} dB`;
}

/**
 * Convert pan (0-127) to display string
 */
export function panToString(pan: number): string {
  if (pan < 60) return `L${Math.round((64 - pan) / 64 * 100)}%`;
  if (pan > 68) return `R${Math.round((pan - 64) / 64 * 100)}%`;
  return 'C';
}

/**
 * Format BPM to 1 decimal place
 */
export function formatBpm(bpm: number): string {
  return bpm.toFixed(1);
}
```

### FILE: `app/src/lib/utils/constants.ts`

```typescript
/**
 * Application constants
 */

export const TICKS_PER_BEAT = 480;
export const DEFAULT_TEMPO = 120;
export const DEFAULT_TIME_SIGNATURE = { numerator: 4, denominator: 4 };

export const TRACK_COLORS = [
  '#3b82f6', // blue
  '#ef4444', // red
  '#10b981', // green
  '#f59e0b', // orange
  '#8b5cf6', // purple
  '#ec4899', // pink
  '#06b6d4', // cyan
  '#84cc16', // lime
  '#f97316', // orange
  '#6366f1', // indigo
];

export const GRID_SIZES = [10, 20, 50];
```

---

*Due to character limits, this is Part 1 of the guide. The file will be continued with Sections 8-12 covering components, windows, root app, styles, and testing.*

---

## SECTION 8: BASE COMPONENTS

### FILE: `app/src/lib/components/WindowBase.svelte`

```svelte
<script lang="ts">
  import { uiActions } from '$lib/stores';
  import type { WindowId } from '$lib/types';

  export let windowId: WindowId;
  export let title: string;

  let dragging = false;
  let resizing = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let resizeStartWidth = 0;
  let resizeStartHeight = 0;

  function handleMouseDown(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.window-titlebar')) {
      dragging = true;
      dragStartX = e.clientX;
      dragStartY = e.clientY;
      uiActions.focusWindow(windowId);
    }
  }

  function handleMouseMove(e: MouseEvent) {
    if (dragging) {
      const deltaX = e.clientX - dragStartX;
      const deltaY = e.clientY - dragStartY;
      // Position update handled by uiStore
      dragStartX = e.clientX;
      dragStartY = e.clientY;
    }
  }

  function handleMouseUp() {
    dragging = false;
    resizing = false;
  }

  function handleMinimize() {
    uiActions.minimizeWindow(windowId);
  }

  function handleClose() {
    uiActions.hideWindow(windowId);
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<div class="window-base" on:mousedown={handleMouseDown}>
  <div class="window-titlebar">
    <span class="window-title">{title}</span>
    <div class="window-controls">
      <button on:click={handleMinimize} class="btn-minimize">−</button>
      <button on:click={handleClose} class="btn-close">✕</button>
    </div>
  </div>
  <div class="window-content">
    <slot />
  </div>
</div>

<style>
  .window-base {
    position: absolute;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .window-titlebar {
    background: var(--color-bg);
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: move;
    user-select: none;
    border-bottom: 1px solid var(--color-border);
  }

  .window-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
  }

  .window-controls {
    display: flex;
    gap: 8px;
  }

  .window-controls button {
    width: 24px;
    height: 24px;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .window-controls button:hover {
    background: var(--color-primary);
  }

  .window-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
</style>
```

### FILE: `app/src/lib/components/MenuBar.svelte`

```svelte
<script lang="ts">
  import { uiActions } from '$lib/stores';
  import { playbackActions } from '$lib/stores';

  let activeMenu: string | null = null;

  function toggleMenu(menu: string) {
    activeMenu = activeMenu === menu ? null : menu;
  }

  function closeMenus() {
    activeMenu = null;
  }

  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === ' ' && !e.repeat) {
      e.preventDefault();
      playbackActions.togglePlayPause();
    }
    if (e.ctrlKey || e.metaKey) {
      switch (e.key) {
        case '1':
          e.preventDefault();
          uiActions.toggleWindow('daw');
          break;
        case '2':
          e.preventDefault();
          uiActions.toggleWindow('mixer');
          break;
        case '3':
          e.preventDefault();
          uiActions.toggleWindow('database');
          break;
        case '4':
          e.preventDefault();
          uiActions.toggleWindow('pipeline');
          break;
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} on:click={closeMenus} />

<div class="menu-bar">
  <div class="menu-item" on:click|stopPropagation={() => toggleMenu('file')}>
    File
    {#if activeMenu === 'file'}
      <div class="menu-dropdown">
        <div class="menu-option">New Project</div>
        <div class="menu-option">Open Project</div>
        <div class="menu-option">Save Project</div>
        <div class="menu-separator"></div>
        <div class="menu-option">Exit</div>
      </div>
    {/if}
  </div>

  <div class="menu-item" on:click|stopPropagation={() => toggleMenu('window')}>
    Window
    {#if activeMenu === 'window'}
      <div class="menu-dropdown">
        <div class="menu-option" on:click={() => uiActions.toggleWindow('daw')}>
          DAW Window <span class="shortcut">Ctrl+1</span>
        </div>
        <div class="menu-option" on:click={() => uiActions.toggleWindow('mixer')}>
          Mixer Window <span class="shortcut">Ctrl+2</span>
        </div>
        <div class="menu-option" on:click={() => uiActions.toggleWindow('database')}>
          Database Window <span class="shortcut">Ctrl+3</span>
        </div>
        <div class="menu-option" on:click={() => uiActions.toggleWindow('pipeline')}>
          Pipeline Window <span class="shortcut">Ctrl+4</span>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .menu-bar {
    background: var(--color-bg);
    padding: 8px 16px;
    display: flex;
    gap: 16px;
    border-bottom: 1px solid var(--color-border);
    user-select: none;
  }

  .menu-item {
    position: relative;
    padding: 4px 12px;
    cursor: pointer;
    border-radius: 4px;
    color: var(--color-text);
    font-size: 14px;
  }

  .menu-item:hover {
    background: var(--color-surface);
  }

  .menu-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    min-width: 200px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    z-index: 1000;
  }

  .menu-option {
    padding: 8px 16px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .menu-option:hover {
    background: var(--color-primary);
  }

  .shortcut {
    color: var(--color-text-secondary);
    font-size: 12px;
  }

  .menu-separator {
    height: 1px;
    background: var(--color-border);
    margin: 4px 0;
  }
</style>
```

### FILE: `app/src/lib/components/StatusBar.svelte`

```svelte
<script lang="ts">
  import { playbackStore } from '$lib/stores';
  import { formattedPosition } from '$lib/stores/playbackStore';
</script>

<div class="status-bar">
  <div class="status-item">
    Position: {$formattedPosition}
  </div>
  <div class="status-item">
    {$playbackStore.tempo} BPM
  </div>
  <div class="status-item">
    {$playbackStore.timeSignature.numerator}/{$playbackStore.timeSignature.denominator}
  </div>
  <div class="status-item">
    Key: {$playbackStore.keySignature}
  </div>
</div>

<style>
  .status-bar {
    background: var(--color-bg);
    padding: 6px 16px;
    display: flex;
    gap: 24px;
    border-top: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
```

---

## SECTION 9: WINDOW COMPONENTS

### FILE: `app/src/lib/windows/DAWWindow.svelte`

```svelte
<script lang="ts">
  import { playbackStore, playbackActions } from '$lib/stores';
  import { projectStore, projectActions } from '$lib/stores';
  import { formattedPosition } from '$lib/stores/playbackStore';

  async function handlePlayPause() {
    await playbackActions.togglePlayPause();
  }

  async function handleStop() {
    await playbackActions.stop();
  }

  async function handleAddTrack() {
    // TODO: Open file picker
    await projectActions.addTrack(1, 0);
  }
</script>

<div class="daw-window">
  <!-- Transport Bar -->
  <div class="transport-bar">
    <div class="transport-controls">
      <button on:click={handlePlayPause} class="btn-transport">
        {$playbackStore.isPlaying ? '⏸' : '▶'}
      </button>
      <button on:click={handleStop} class="btn-transport">■</button>
    </div>

    <div class="position-display">
      {$formattedPosition}
    </div>

    <div class="tempo-control">
      <button on:click={() => playbackActions.setTempo($playbackStore.tempo - 1)}>−</button>
      <span>{$playbackStore.tempo} BPM</span>
      <button on:click={() => playbackActions.setTempo($playbackStore.tempo + 1)}>+</button>
    </div>

    <div class="controls">
      <button on:click={playbackActions.toggleLoop} class:active={$playbackStore.loopEnabled}>
        Loop
      </button>
      <button on:click={playbackActions.toggleMetronome} class:active={$playbackStore.metronomeEnabled}>
        Metro
      </button>
    </div>
  </div>

  <!-- Track List -->
  <div class="track-list">
    <button on:click={handleAddTrack} class="btn-add-track">+ Add Track</button>

    {#each $projectStore.tracks as track (track.id)}
      <div class="track-item" class:selected={$projectStore.selectedTrackId === track.id}>
        <span class="track-color" style="background: {track.color}"></span>
        <span class="track-name">{track.name}</span>
        <div class="track-controls">
          <button
            class:active={track.muted}
            on:click={() => projectActions.updateTrack(track.id, { muted: !track.muted })}
          >
            M
          </button>
          <button
            class:active={track.solo}
            on:click={() => projectActions.updateTrack(track.id, { solo: !track.solo })}
          >
            S
          </button>
        </div>
      </div>
    {/each}
  </div>

  <!-- Arrangement View (Placeholder) -->
  <div class="arrangement-view">
    <div class="timeline">Timeline will go here</div>
  </div>
</div>

<style>
  .daw-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: 16px;
  }

  .transport-bar {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 12px;
    background: var(--color-bg);
    border-radius: 6px;
  }

  .transport-controls {
    display: flex;
    gap: 8px;
  }

  .btn-transport {
    width: 40px;
    height: 40px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 6px;
    cursor: pointer;
    font-size: 18px;
  }

  .position-display {
    font-family: monospace;
    font-size: 18px;
    font-weight: bold;
    color: var(--color-text);
  }

  .tempo-control {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .tempo-control button {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 4px;
    cursor: pointer;
  }

  .controls button {
    padding: 8px 16px;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 6px;
    cursor: pointer;
  }

  .controls button.active {
    background: var(--color-primary);
    color: white;
  }

  .track-list {
    background: var(--color-bg);
    border-radius: 6px;
    padding: 12px;
    max-height: 300px;
    overflow-y: auto;
  }

  .btn-add-track {
    width: 100%;
    padding: 8px;
    border: 1px dashed var(--color-border);
    background: transparent;
    color: var(--color-text-secondary);
    border-radius: 4px;
    cursor: pointer;
    margin-bottom: 8px;
  }

  .track-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    background: var(--color-surface);
    border-radius: 4px;
    margin-bottom: 4px;
    cursor: pointer;
  }

  .track-item.selected {
    background: var(--color-primary);
  }

  .track-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .track-name {
    flex: 1;
    color: var(--color-text);
  }

  .track-controls {
    display: flex;
    gap: 4px;
  }

  .track-controls button {
    width: 28px;
    height: 28px;
    border: none;
    background: var(--color-bg);
    color: var(--color-text-secondary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .track-controls button.active {
    background: var(--color-error);
    color: white;
  }

  .arrangement-view {
    flex: 1;
    background: var(--color-bg);
    border-radius: 6px;
    padding: 16px;
  }
</style>
```

### FILE: `app/src/lib/windows/MixerWindow.svelte`

```svelte
<script lang="ts">
  import { projectStore, projectActions } from '$lib/stores';
  import { volumeToDb, panToString } from '$lib/utils/formatters';

  function handleVolumeChange(trackId: number, volume: number) {
    projectActions.updateTrack(trackId, { volume });
  }

  function handlePanChange(trackId: number, pan: number) {
    projectActions.updateTrack(trackId, { pan });
  }
</script>

<div class="mixer-window">
  {#each $projectStore.tracks as track (track.id)}
    <div class="channel-strip">
      <div class="channel-name">{track.name}</div>

      <!-- Volume Fader -->
      <div class="fader-section">
        <input
          type="range"
          min="0"
          max="127"
          value={track.volume}
          on:input={(e) => handleVolumeChange(track.id, parseInt(e.currentTarget.value))}
          class="fader vertical"
        />
        <div class="fader-label">{volumeToDb(track.volume)}</div>
      </div>

      <!-- Pan Knob -->
      <div class="pan-section">
        <input
          type="range"
          min="0"
          max="127"
          value={track.pan}
          on:input={(e) => handlePanChange(track.id, parseInt(e.currentTarget.value))}
          class="pan-knob"
        />
        <div class="pan-label">{panToString(track.pan)}</div>
      </div>

      <!-- Mute/Solo -->
      <div class="channel-controls">
        <button
          class:active={track.muted}
          on:click={() => projectActions.updateTrack(track.id, { muted: !track.muted })}
        >
          M
        </button>
        <button
          class:active={track.solo}
          on:click={() => projectActions.updateTrack(track.id, { solo: !track.solo })}
        >
          S
        </button>
      </div>
    </div>
  {/each}
</div>

<style>
  .mixer-window {
    display: flex;
    gap: 8px;
    padding: 16px;
    overflow-x: auto;
  }

  .channel-strip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--color-bg);
    border-radius: 6px;
    min-width: 80px;
  }

  .channel-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
  }

  .fader-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .fader.vertical {
    writing-mode: bt-lr;
    -webkit-appearance: slider-vertical;
    height: 150px;
  }

  .fader-label {
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .pan-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .pan-label {
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .channel-controls {
    display: flex;
    gap: 4px;
  }

  .channel-controls button {
    width: 32px;
    height: 32px;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .channel-controls button.active {
    background: var(--color-error);
    color: white;
  }
</style>
```

### FILE: `app/src/lib/windows/DatabaseWindow.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { databaseStore, databaseActions } from '$lib/stores';
  import { totalPages } from '$lib/stores/databaseStore';
  import { formatFileSize, formatRelativeTime, formatBpm } from '$lib/utils/formatters';

  let searchTimeout: number;

  function handleSearchInput() {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      databaseActions.search();
    }, 300);
  }

  onMount(() => {
    databaseActions.search();
  });
</script>

<div class="database-window">
  <!-- Search Bar -->
  <div class="search-section">
    <input
      type="search"
      bind:value={$databaseStore.searchQuery}
      on:input={handleSearchInput}
      placeholder="Search files..."
      class="search-input"
    />
  </div>

  <!-- Filters -->
  <div class="filters-section">
    <div class="filter-group">
      <label>BPM Range:</label>
      <input
        type="number"
        placeholder="Min"
        on:change={() => databaseActions.search()}
      />
      <input
        type="number"
        placeholder="Max"
        on:change={() => databaseActions.search()}
      />
    </div>
  </div>

  <!-- Results -->
  <div class="results-section">
    {#if $databaseStore.loading}
      <div class="loading">Searching...</div>
    {:else if $databaseStore.error}
      <div class="error">{$databaseStore.error}</div>
    {:else}
      <div class="results-header">
        {$databaseStore.total} file{$databaseStore.total !== 1 ? 's' : ''} found
      </div>

      <div class="results-list">
        {#each $databaseStore.results as file (file.id)}
          <div class="file-card" on:click={() => databaseActions.selectFile(file.id)}>
            <div class="file-info">
              <h4>{file.file_name}</h4>
              <div class="file-metadata">
                {#if file.bpm}<span>{formatBpm(file.bpm)} BPM</span>{/if}
                {#if file.key}<span>{file.key}</span>{/if}
                {#if file.time_signature}<span>{file.time_signature}</span>{/if}
              </div>
              {#if file.tags.length > 0}
                <div class="file-tags">
                  {#each file.tags.slice(0, 5) as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
            <button on:click|stopPropagation={() => databaseActions.toggleFavorite(file.id)}>
              {file.is_favorite ? '⭐' : '☆'}
            </button>
          </div>
        {/each}
      </div>

      <!-- Pagination -->
      <div class="pagination">
        <button
          disabled={$databaseStore.currentPage === 1}
          on:click={() => { databaseActions.previousPage(); databaseActions.search(); }}
        >
          Previous
        </button>
        <span>Page {$databaseStore.currentPage} of {$totalPages}</span>
        <button
          disabled={$databaseStore.currentPage >= $totalPages}
          on:click={() => { databaseActions.nextPage(); databaseActions.search(); }}
        >
          Next
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .database-window {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
  }

  .search-input {
    width: 100%;
    padding: 10px 16px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text);
    font-size: 14px;
  }

  .filters-section {
    display: flex;
    gap: 16px;
  }

  .filter-group {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .filter-group label {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .filter-group input {
    width: 80px;
    padding: 6px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text);
  }

  .results-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .results-header {
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  .results-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .file-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--color-bg);
    border-radius: 6px;
    cursor: pointer;
  }

  .file-card:hover {
    background: var(--color-surface);
  }

  .file-info h4 {
    margin: 0 0 8px 0;
    font-size: 14px;
    color: var(--color-text);
  }

  .file-metadata {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .file-tags {
    display: flex;
    gap: 4px;
    margin-top: 8px;
  }

  .tag {
    padding: 2px 8px;
    background: var(--color-primary);
    color: white;
    border-radius: 12px;
    font-size: 11px;
  }

  .pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: var(--color-bg);
    border-radius: 6px;
  }

  .pagination button {
    padding: 8px 16px;
    border: none;
    background: var(--color-surface);
    color: var(--color-text);
    border-radius: 4px;
    cursor: pointer;
  }

  .pagination button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

### FILE: `app/src/lib/windows/PipelineWindow.svelte`

```svelte
<script lang="ts">
  // Pipeline window for batch import - placeholder
</script>

<div class="pipeline-window">
  <h3>Pipeline Import (Coming Soon)</h3>
  <p>Drag and drop files or folders to import them into the database.</p>
</div>

<style>
  .pipeline-window {
    padding: 24px;
    text-align: center;
    color: var(--color-text-secondary);
  }
</style>
```

---

## SECTION 10: ROOT APPLICATION

### FILE: `app/src/main.ts`

```typescript
import './app.css';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('app')!,
});

export default app;
```

### FILE: `app/src/App.svelte`

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api';
  import { setupEventListeners, cleanupEventListeners } from '$lib/api/events';
  import { playbackActions, projectActions } from '$lib/stores';
  import { uiStore } from '$lib/stores';

  import MenuBar from '$lib/components/MenuBar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';

  import DAWWindow from '$lib/windows/DAWWindow.svelte';
  import MixerWindow from '$lib/windows/MixerWindow.svelte';
  import DatabaseWindow from '$lib/windows/DatabaseWindow.svelte';
  import PipelineWindow from '$lib/windows/PipelineWindow.svelte';

  let appReady = false;
  let initError = '';

  onMount(async () => {
    try {
      // Initialize database connection
      await api.system.initializeDatabase();
      console.log('✅ Database connected');

      // Setup event listeners
      await setupEventListeners({
        onPlaybackPosition: playbackActions.updatePosition,
        onTrackAdded: projectActions.addTrackFromEvent,
        onTrackRemoved: (data) => projectActions.removeTrackFromEvent(data.track_id),
        onTrackUpdated: projectActions.updateTrackFromEvent,
      });
      console.log('✅ Event listeners setup');

      appReady = true;
    } catch (error) {
      console.error('❌ App initialization failed:', error);
      initError = error instanceof Error ? error.message : 'Unknown error';

      if (initError.includes('ECONNREFUSED') || initError.includes('Connection refused')) {
        initError = `Database connection failed. Please start Docker containers:

cd /home/dojevou/projects/midi-software-center/database
docker-compose up -d

Then restart the application.`;
      }

      appReady = true;
    }
  });

  onDestroy(() => {
    cleanupEventListeners();
  });
</script>

{#if !appReady}
  <div class="loading-screen">
    <div class="spinner"></div>
    <p>Initializing MIDI Software Center...</p>
  </div>
{:else if initError}
  <div class="error-screen">
    <div class="error-icon">⚠️</div>
    <h2>Initialization Error</h2>
    <pre>{initError}</pre>
    <button on:click={() => window.location.reload()}>Retry</button>
  </div>
{:else}
  <div class="app">
    <MenuBar />

    <div class="workspace">
      {#if $uiStore.windows.daw?.visible && !$uiStore.windows.daw.minimized}
        <div
          class="window"
          style="left: {$uiStore.windows.daw.x}px; top: {$uiStore.windows.daw.y}px; width: {$uiStore.windows.daw.width}px; height: {$uiStore.windows.daw.height}px; z-index: {$uiStore.windows.daw.zIndex}"
        >
          <WindowBase windowId="daw" title="DAW - Sequencer">
            <DAWWindow />
          </WindowBase>
        </div>
      {/if}

      {#if $uiStore.windows.mixer?.visible && !$uiStore.windows.mixer.minimized}
        <div
          class="window"
          style="left: {$uiStore.windows.mixer.x}px; top: {$uiStore.windows.mixer.y}px; width: {$uiStore.windows.mixer.width}px; height: {$uiStore.windows.mixer.height}px; z-index: {$uiStore.windows.mixer.zIndex}"
        >
          <WindowBase windowId="mixer" title="Mixer">
            <MixerWindow />
          </WindowBase>
        </div>
      {/if}

      {#if $uiStore.windows.database?.visible && !$uiStore.windows.database.minimized}
        <div
          class="window"
          style="left: {$uiStore.windows.database.x}px; top: {$uiStore.windows.database.y}px; width: {$uiStore.windows.database.width}px; height: {$uiStore.windows.database.height}px; z-index: {$uiStore.windows.database.zIndex}"
        >
          <WindowBase windowId="database" title="Database Browser">
            <DatabaseWindow />
          </WindowBase>
        </div>
      {/if}

      {#if $uiStore.windows.pipeline?.visible && !$uiStore.windows.pipeline.minimized}
        <div
          class="window"
          style="left: {$uiStore.windows.pipeline.x}px; top: {$uiStore.windows.pipeline.y}px; width: {$uiStore.windows.pipeline.width}px; height: {$uiStore.windows.pipeline.height}px; z-index: {$uiStore.windows.pipeline.zIndex}"
        >
          <WindowBase windowId="pipeline" title="Pipeline Import">
            <PipelineWindow />
          </WindowBase>
        </div>
      {/if}
    </div>

    <StatusBar />
  </div>
{/if}

<style>
  .app {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--color-bg);
  }

  .workspace {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .window {
    position: absolute;
  }

  .loading-screen, .error-screen {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
    background: var(--color-bg);
    color: var(--color-text);
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-icon {
    font-size: 64px;
  }

  pre {
    background: var(--color-surface);
    padding: 16px;
    border-radius: 6px;
    max-width: 600px;
    white-space: pre-wrap;
  }

  button {
    padding: 12px 24px;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
  }
</style>
```

---

## SECTION 11: GLOBAL STYLES

### FILE: `app/src/app.css`

```css
:root {
  /* Colors - Dark Theme */
  --color-bg: #1a1a1a;
  --color-surface: #2d2d2d;
  --color-border: #404040;

  --color-primary: #3b82f6;
  --color-success: #10b981;
  --color-error: #ef4444;
  --color-warning: #f59e0b;

  --color-text: #e5e5e5;
  --color-text-secondary: #a3a3a3;

  /* Spacing */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;

  /* Borders */
  --radius-sm: 4px;
  --radius-md: 6px;
  --radius-lg: 8px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: var(--color-bg);
  color: var(--color-text);
  overflow: hidden;
}

input, button, select, textarea {
  font-family: inherit;
}

input:focus, button:focus, select:focus, textarea:focus {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Scrollbar styles */
::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-track {
  background: var(--color-bg);
}

::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 5px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-surface);
}
```

---

## SECTION 12: TESTING & VALIDATION

### Post-Generation Checklist

After kilo code generates all files, run these validation steps:

```bash
# 1. Install dependencies
cd app
pnpm install

# 2. Type check
pnpm check

# 3. Build frontend
pnpm build

# 4. Build Rust backend
cd src-tauri
cargo build

# 5. Run development server
cd ..
pnpm tauri dev
```

### Expected Behavior

1. **App launches** without errors
2. **Database connection** shows ✅ in console
3. **DAW and Database windows** are visible by default
4. **MenuBar** shows "File" and "Window" menus
5. **Window shortcuts** work (Ctrl+1-4)
6. **StatusBar** displays position, tempo, time signature
7. **DatabaseWindow** shows search input and "0 files found" (if database is empty)
8. **DAWWindow** shows transport controls and "Add Track" button

### Common Issues & Fixes

| Issue | Fix |
|-------|-----|
| Database connection failed | Run `docker-compose up -d` in database/ directory |
| Type errors | Check all field names match spec (snake_case) |
| Import errors | Verify vite.config.ts has correct path aliases |
| Window not draggable | Check WindowBase mousedown handler |
| Keyboard shortcuts not working | Verify MenuBar keydown listener is attached |

---

## FINAL NOTES FOR KILO CODE

**CRITICAL REQUIREMENTS:**

1. **Exact Type Matching**: All TypeScript types MUST match Rust backend exactly
   - Use snake_case for JSON field names
   - Use `| undefined` for Option<T>, NOT `| null`
   - All numeric types become `number`

2. **Command Names**: All invoke() calls use snake_case
   - ✅ `invoke('midi_list_devices')`
   - ❌ `invoke('midiListDevices')`

3. **Event Names**: All listen() calls use kebab-case
   - ✅ `listen('playback-started')`
   - ❌ `listen('playbackStarted')`

4. **Database Connection**: Port is 5433 (not 5432)
   - Connection string: `postgresql://midiuser:145278963@localhost:5433/midi_library`

5. **Error Handling**: Always wrap API calls in try-catch

6. **File Paths**: Use absolute paths in file organization

7. **Testing**: Run `pnpm check` before considering generation complete

---

**END OF COMPLETE FRONTEND GENERATION GUIDE**

Total estimated lines: ~8,000-10,000 lines of production-ready code across 30+ files.
