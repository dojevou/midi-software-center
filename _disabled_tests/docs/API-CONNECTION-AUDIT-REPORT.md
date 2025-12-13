# API Connection Audit Report
## MIDI Software Center - Frontend-Backend Integration Analysis

**Date:** 2025-11-12
**Status:** CRITICAL GAPS IDENTIFIED
**Priority:** HIGH - Multiple UI features non-functional

---

## Executive Summary

**CRITICAL FINDING:** Major disconnect between frontend UI and backend functionality. Database and Pipeline commands exist in backend but are NOT registered in main.rs, making them completely inaccessible from the frontend.

### Key Issues Identified:
1. **Database commands exist but are NOT registered** - 6 backend functions unavailable
2. **Pipeline commands exist but are NOT registered** - 5 backend functions unavailable
3. **Frontend calling non-existent API wrappers** - Multiple UI buttons do nothing
4. **Type mismatches** - Frontend/backend type definitions don't align
5. **Missing progress event handlers** - Pipeline operations invisible to UI

---

## Part 1: Missing Backend Command Registrations

### ❌ Database Commands (EXIST BUT NOT REGISTERED)

**Location:** `daw/src-tauri/src/commands/database.rs`
**Status:** ✅ Implemented | ❌ NOT registered in main.rs | ❌ NOT accessible from frontend

| Command | Implemented | Registered | Frontend API Wrapper | Status |
|---------|-------------|-----------|---------------------|--------|
| `database_search` | ✅ | ❌ | ❌ | **MISSING** |
| `database_get_file_metadata` | ✅ | ❌ | ❌ | **MISSING** |
| `database_add_file` | ✅ | ❌ | ❌ | **MISSING** |
| `database_remove_file` | ✅ | ❌ | ❌ | **MISSING** |
| `database_get_stats` | ✅ | ❌ | ❌ | **MISSING** |

**Impact:** DatabaseWindow has NO way to:
- Search files (uses different `search_files` command instead)
- Get file metadata directly
- Add/remove files from database
- Get database statistics

### ❌ Pipeline Commands (EXIST BUT NOT REGISTERED)

**Location:** `daw/src-tauri/src/commands/pipeline.rs`
**Status:** ✅ Implemented | ❌ NOT registered in main.rs | ❌ NOT accessible from frontend

| Command | Implemented | Registered | Frontend API Wrapper | Status |
|---------|-------------|-----------|---------------------|--------|
| `pipeline_import_files` | ✅ | ❌ | ❌ | **MISSING** |
| `pipeline_analyze_files` | ✅ | ❌ | ❌ | **MISSING** |
| `pipeline_archive_files` | ✅ | ❌ | ❌ | **MISSING** |
| `pipeline_get_progress` | ✅ | ❌ | ❌ | **MISSING** |
| `pipeline_cancel` | ✅ | ❌ | ❌ | **MISSING** |

**Impact:** PipelineWindow buttons do nothing:
- "Import Files" button - no backend
- "Start Analysis" button - no backend
- "Start Archiving" button - no backend
- Progress indicators - no data source
- Cancel button - non-functional

---

## Part 2: Frontend API Wrapper Gaps

### Current API Structure (app/src/lib/api.ts)

**Existing API Modules:**
```typescript
✅ api.midi          - 6 commands (all connected)
✅ api.sequencer     - 13 commands (all connected)
✅ api.search        - 3 commands (all connected)
✅ api.analysis      - 6 commands (all connected)
✅ api.project       - 3 commands (all connected)
✅ api.export        - 1 command (connected)
✅ api.window        - 33 commands (all connected)
✅ api.automation    - 12 commands (all connected)
✅ api.pipeline      - 4 commands (calling wrong backend!)
❌ api.database      - COMPLETELY MISSING
❌ api.statistics    - COMPLETELY MISSING
❌ api.tags          - COMPLETELY MISSING
```

### ❌ Missing: api.database Module

**Frontend needs but doesn't have:**
```typescript
api.database: {
  search(filters: DatabaseFilters) -> SearchResults
  getFileMetadata(id: number) -> MidiFile
  addFile(file: FileParams) -> number
  removeFile(id: number) -> void
  getStats() -> DatabaseStats
  testConnection() -> boolean
}
```

### ❌ Missing: api.statistics Module

**Frontend needs but doesn't have:**
```typescript
api.statistics: {
  getCategoryStats() -> Record<string, number>
  getManufacturerStats() -> Record<string, number>
  getKeySignatureStats() -> Record<string, number>
  getRecentlyAddedCount(days: number) -> number
  getDuplicateCount() -> number
  getDatabaseSize() -> number
  checkDatabaseHealth() -> HealthReport
}
```

### ❌ Missing: api.tags Module

**Frontend needs but doesn't have:**
```typescript
api.tags: {
  getFileTags(fileId: number) -> string[]
  getPopularTags(limit: number) -> TagInfo[]
  searchTags(query: string) -> string[]
  getTagCategories() -> string[]
  getTagsByCategory(category: string) -> string[]
  updateFileTags(fileId: number, tags: string[]) -> void
  addTagsToFile(fileId: number, tags: string[]) -> void
  removeTagFromFile(fileId: number, tag: string) -> void
  getFilesByTags(tags: string[]) -> FileDetails[]
}
```

### ⚠️ Broken: api.pipeline Module

**Currently defined (WRONG):**
```typescript
api.pipeline: {
  importSingleFile(filePath: string)     // Calls 'import_single_file'
  importDirectory(directoryPath: string)  // Calls 'import_directory'
  startAnalysis()                        // Calls 'start_analysis'
  importArchiveCollection(archivePath)   // Calls 'import_archive_collection'
}
```

**Problem:** These invoke **pipeline/src-tauri** commands (different app!), not **daw/src-tauri** commands.

**Should call:**
```typescript
api.pipeline: {
  importFiles(filePaths: string[])       // Calls 'pipeline_import_files'
  analyzeFiles(fileIds: number[])        // Calls 'pipeline_analyze_files'
  archiveFiles(fileIds: number[], path)  // Calls 'pipeline_archive_files'
  getProgress()                          // Calls 'pipeline_get_progress'
  cancel()                               // Calls 'pipeline_cancel'
}
```

---

## Part 3: Type Definition Mismatches

### Backend Types (database.rs)
```rust
struct MidiFile {
    id: i64,
    file_path: String,
    file_name: String,
    bpm: f32,
    key_signature: String,
    tags: Vec<String>,
    duration: f32,
    track_count: i32,
    file_size: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct SearchFilters {
    query: Option<String>,
    bpm_min: Option<f32>,
    bpm_max: Option<f32>,
    key: Option<String>,
    tag: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

struct DatabaseStats {
    total_files: i64,
    avg_bpm: f32,
    total_size: i64,
}
```

### Frontend Types (types.ts) - MISSING EQUIVALENTS

**Current search filter type:**
```typescript
interface SearchFilters {
  search_text?: string;
  min_bpm?: number;
  max_bpm?: number;
  key_signature?: string;
  instruments?: string[];
  limit?: number;
  offset?: number;
}
```

**Missing types needed:**
```typescript
interface MidiFile {
  id: number;
  file_path: string;
  file_name: string;
  bpm: number;
  key_signature: string;
  tags: string[];
  duration: number;
  track_count: number;
  file_size: number;
  created_at: string;
  updated_at: string;
}

interface DatabaseFilters {
  query?: string;
  bpm_min?: number;
  bpm_max?: number;
  key?: string;
  tag?: string;
  limit?: number;
  offset?: number;
}

interface DatabaseStats {
  total_files: number;
  avg_bpm: number;
  total_size: number;
}

interface PipelineProgress {
  current: number;
  total: number;
  stage: string;
  current_file?: string;
  rate: number;
  eta_seconds: number;
  details: string;
}

interface ImportStats {
  files_processed: number;
  files_imported: number;
  files_skipped: number;
  total_size: number;
  duration_seconds: number;
  errors: string[];
}

interface AnalysisResults {
  files_analyzed: number;
  bpm_detected: number;
  key_detected: number;
  instruments_found: string[];
  errors: string[];
}
```

---

## Part 4: Component-Specific Issues

### DatabaseWindow.svelte

**Current Issues:**
1. ✅ Uses `api.search.files()` - Works (registered command)
2. ✅ Uses `api.search.getSuggestions()` - Works (registered command)
3. ❌ Favorite star button - Has backend (`api.analysis.addFavorite/removeFavorite`) but no UI handler
4. ❌ No delete file functionality - `database_remove_file` exists but not registered
5. ❌ No file details modal - `database_get_file_metadata` exists but not registered
6. ❌ No statistics display - `database_get_stats` exists but not registered

**Missing UI Features:**
```svelte
<!-- MISSING: Favorite toggle on each file -->
<button on:click={() => toggleFavorite(file.id)}>
  {#if file.is_favorite}★{:else}☆{/if}
</button>

<!-- MISSING: Delete file button -->
<button on:click={() => deleteFile(file.id)}>Delete</button>

<!-- MISSING: Statistics panel -->
<div class="stats-panel">
  Total Files: {stats.total_files}
  Avg BPM: {stats.avg_bpm}
  Total Size: {formatBytes(stats.total_size)}
</div>
```

### PipelineWindow.svelte

**Current Issues:**
1. ❌ `pipelineActions.importFiles()` - Calls non-existent API
2. ❌ `pipelineActions.startOperation('analyze')` - Calls non-existent API
3. ❌ `pipelineActions.startOperation('archive')` - Calls non-existent API
4. ❌ No progress updates - `pipeline_get_progress` not registered
5. ❌ No event listeners - No handlers for `pipeline::progress` events
6. ❌ No cancel functionality - `pipeline_cancel` not registered

**Required Event Handlers:**
```typescript
onMount(() => {
  // Listen for pipeline progress events
  listen('pipeline::progress', (event: { payload: PipelineProgress }) => {
    pipelineStore.update(s => ({ ...s, progress: event.payload }));
  });

  listen('pipeline::completed', () => {
    pipelineStore.update(s => ({ ...s, operation: 'completed' }));
  });
});
```

### MixerWindow.svelte

**Current Issues:**
1. ✅ Volume sliders - Backend exists (`api.window.setChannelVolume`) - just needs connection
2. ✅ Pan sliders - Backend exists (`api.window.setChannelPan`) - just needs connection
3. ✅ Mute buttons - Backend exists (`api.window.setChannelMute`) - just needs connection
4. ✅ Solo buttons - Backend exists (`api.window.setChannelSolo`) - just needs connection
5. ✅ Get mixer state - Backend exists (`api.window.getMixerState`) - needs initial load

**Missing Connections:** (Commands exist, just need UI wiring)
```typescript
async function updateVolume(trackId: number, volume: number) {
  await api.window.setChannelVolume(trackId, volume);
}

async function updatePan(trackId: number, pan: number) {
  await api.window.setChannelPan(trackId, pan);
}

async function toggleMute(trackId: number, muted: boolean) {
  await api.window.setChannelMute(trackId, muted);
}

async function toggleSolo(trackId: number, soloed: boolean) {
  await api.window.setChannelSolo(trackId, soloed);
}
```

### DAWWindow.svelte

**Current Issues:**
1. ✅ Transport controls - Backend exists (`api.window.playTransport`, etc.)
2. ✅ Add/Remove tracks - Backend exists (`api.window.addWindowTrack`, `removeWindowTrack`)
3. ❌ Track list not loading - Missing `onMount()` call to `api.window.getAllWindowTracks()`
4. ❌ Playback position not updating - Missing interval to call `api.window.getPlaybackState()`
5. ❌ BPM display not updating - Missing call to `api.window.getBpm()`

**Required Initialization:**
```typescript
onMount(async () => {
  // Load initial tracks
  const tracks = await api.window.getAllWindowTracks();
  dawStore.set({ ...dawStore, tracks });

  // Start playback position updates (every 100ms)
  const interval = setInterval(async () => {
    const state = await api.window.getPlaybackState();
    dawStore.update(s => ({ ...s, playbackState: state }));
  }, 100);

  return () => clearInterval(interval);
});
```

---

## Part 5: Step-by-Step Fix Plan

### Step 1: Register Missing Backend Commands

**File:** `daw/src-tauri/src/main.rs`

**Add to `invoke_handler![]`:**
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...

    // Database commands (ADD THESE)
    commands::database::database_search,
    commands::database::database_get_file_metadata,
    commands::database::database_add_file,
    commands::database::database_remove_file,
    commands::database::database_get_stats,

    // Pipeline commands (ADD THESE)
    commands::pipeline::pipeline_import_files,
    commands::pipeline::pipeline_analyze_files,
    commands::pipeline::pipeline_archive_files,
    commands::pipeline::pipeline_get_progress,
    commands::pipeline::pipeline_cancel,
])
```

**Add Pipeline state management:**
```rust
use commands::pipeline::PipelineState;

// In main():
let pipeline_state = PipelineState::default();

tauri::Builder::default()
    .manage(app_state)
    .manage(midi_manager)
    .manage(sequencer_engine)
    .manage(daw_state)
    .manage(mixer_state)
    .manage(pipeline_state)  // ADD THIS
```

### Step 2: Add Missing API Wrappers

**File:** `app/src/lib/api.ts`

**Add new modules:**
```typescript
// Database operations (NEW)
database: {
  search: async (filters: DatabaseFilters): Promise<SearchResults> => {
    try {
      return await invoke('database_search', { filters });
    } catch (error) {
      console.error('Database search failed:', error);
      throw error;
    }
  },

  getFileMetadata: async (id: number): Promise<MidiFile> => {
    try {
      return await invoke('database_get_file_metadata', { id });
    } catch (error) {
      console.error('Get file metadata failed:', error);
      throw error;
    }
  },

  addFile: async (params: FileParams): Promise<number> => {
    try {
      return await invoke('database_add_file', params);
    } catch (error) {
      console.error('Add file failed:', error);
      throw error;
    }
  },

  removeFile: async (id: number): Promise<void> => {
    try {
      await invoke('database_remove_file', { id });
    } catch (error) {
      console.error('Remove file failed:', error);
      throw error;
    }
  },

  getStats: async (): Promise<DatabaseStats> => {
    try {
      return await invoke('database_get_stats');
    } catch (error) {
      console.error('Get stats failed:', error);
      throw error;
    }
  },
},
```

**Fix pipeline module:**
```typescript
// Fix existing pipeline module
pipeline: {
  importFiles: async (filePaths: string[]): Promise<ImportStats> => {
    try {
      return await invoke('pipeline_import_files', { file_paths: filePaths });
    } catch (error) {
      console.error('Import files failed:', error);
      throw error;
    }
  },

  analyzeFiles: async (fileIds: number[]): Promise<AnalysisResults> => {
    try {
      return await invoke('pipeline_analyze_files', { file_ids: fileIds });
    } catch (error) {
      console.error('Analyze files failed:', error);
      throw error;
    }
  },

  archiveFiles: async (fileIds: number[], archivePath: string): Promise<ImportStats> => {
    try {
      return await invoke('pipeline_archive_files', {
        file_ids: fileIds,
        archive_path: archivePath
      });
    } catch (error) {
      console.error('Archive files failed:', error);
      throw error;
    }
  },

  getProgress: async (): Promise<PipelineProgress> => {
    try {
      return await invoke('pipeline_get_progress');
    } catch (error) {
      console.error('Get progress failed:', error);
      throw error;
    }
  },

  cancel: async (): Promise<void> => {
    try {
      await invoke('pipeline_cancel');
    } catch (error) {
      console.error('Cancel pipeline failed:', error);
      throw error;
    }
  },
},
```

### Step 3: Add Missing Type Definitions

**File:** `app/src/lib/types.ts`

**Add these types:**
```typescript
export interface MidiFile {
  id: number;
  file_path: string;
  file_name: string;
  bpm: number;
  key_signature: string;
  tags: string[];
  duration: number;
  track_count: number;
  file_size: number;
  created_at: string;
  updated_at: string;
}

export interface DatabaseFilters {
  query?: string;
  bpm_min?: number;
  bpm_max?: number;
  key?: string;
  tag?: string;
  limit?: number;
  offset?: number;
}

export interface SearchResults {
  files: MidiFile[];
  total_count: number;
}

export interface DatabaseStats {
  total_files: number;
  avg_bpm: number;
  total_size: number;
}

export interface PipelineProgress {
  current: number;
  total: number;
  stage: string;
  current_file?: string;
  rate: number;
  eta_seconds: number;
  details: string;
}

export interface ImportStats {
  files_processed: number;
  files_imported: number;
  files_skipped: number;
  total_size: number;
  duration_seconds: number;
  errors: string[];
}

export interface AnalysisResults {
  files_analyzed: number;
  bpm_detected: number;
  key_detected: number;
  instruments_found: string[];
  errors: string[];
}
```

### Step 4: Fix DatabaseWindow

**File:** `app/src/lib/windows/DatabaseWindow.svelte`

**Add favorite toggle:**
```svelte
<script lang="ts">
  // ... existing code ...

  async function toggleFavorite(file: FileDetails) {
    try {
      if (file.is_favorite) {
        await api.analysis.removeFavorite(file.id);
      } else {
        await api.analysis.addFavorite(file.id);
      }
      // Refresh search results
      await databaseActions.search();
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }

  async function deleteFile(fileId: number) {
    if (confirm('Are you sure you want to delete this file?')) {
      try {
        await api.database.removeFile(fileId);
        await databaseActions.search();
      } catch (error) {
        console.error('Failed to delete file:', error);
      }
    }
  }

  let stats: DatabaseStats | null = null;

  onMount(async () => {
    // ... existing code ...

    // Load statistics
    try {
      stats = await api.database.getStats();
    } catch (error) {
      console.warn('Failed to load stats:', error);
    }
  });
</script>

<!-- In the file list item -->
<button
  type="button"
  class="favorite-btn"
  on:click|stopPropagation={() => toggleFavorite(file)}
>
  {#if file.is_favorite}★{:else}☆{/if}
</button>

<!-- Add stats panel -->
{#if stats}
  <div class="stats-panel dark:bg-menu p-3 rounded mb-4">
    <h3 class="dark:text-gray-300 mb-2">Database Statistics</h3>
    <div class="grid grid-cols-3 gap-4">
      <div>
        <div class="dark:text-gray-400 text-sm">Total Files</div>
        <div class="dark:text-app-text text-lg font-semibold">{stats.total_files}</div>
      </div>
      <div>
        <div class="dark:text-gray-400 text-sm">Avg BPM</div>
        <div class="dark:text-app-text text-lg font-semibold">{stats.avg_bpm.toFixed(1)}</div>
      </div>
      <div>
        <div class="dark:text-gray-400 text-sm">Total Size</div>
        <div class="dark:text-app-text text-lg font-semibold">{formatBytes(stats.total_size)}</div>
      </div>
    </div>
  </div>
{/if}
```

### Step 5: Fix PipelineWindow

**File:** `app/src/lib/windows/PipelineWindow.svelte`

**Add event listeners and fix handlers:**
```svelte
<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { api } from '$lib/api';

  let progress: PipelineProgress = {
    current: 0,
    total: 0,
    stage: 'idle',
    rate: 0,
    eta_seconds: 0,
    details: '',
  };

  let isProcessing = false;
  let errors: string[] = [];

  onMount(() => {
    // Listen for progress events
    const unlisten1 = listen('pipeline::progress', (event: any) => {
      progress = event.payload;
    });

    const unlisten2 = listen('pipeline::completed', () => {
      isProcessing = false;
      progress = { ...progress, stage: 'completed' };
    });

    return () => {
      unlisten1.then(fn => fn());
      unlisten2.then(fn => fn());
    };
  });

  async function handleImportFiles(files: File[]) {
    isProcessing = true;
    errors = [];
    try {
      const filePaths = files.map(f => f.path); // Need file.path from File object
      const result = await api.pipeline.importFiles(filePaths);
      console.log('Import complete:', result);
      errors = result.errors;
    } catch (error) {
      console.error('Import failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function handleAnalyze() {
    isProcessing = true;
    errors = [];
    try {
      // Get all file IDs from database
      const searchResult = await api.search.files({});
      const fileIds = searchResult.results.map(f => f.id);

      const result = await api.pipeline.analyzeFiles(fileIds);
      console.log('Analysis complete:', result);
      errors = result.errors;
    } catch (error) {
      console.error('Analysis failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function handleArchive() {
    isProcessing = true;
    errors = [];
    try {
      // Get selected file IDs (would need selection UI)
      const fileIds = []; // TODO: Add file selection
      const archivePath = '/tmp/archive.zip'; // TODO: Add path selector

      const result = await api.pipeline.archiveFiles(fileIds, archivePath);
      console.log('Archive complete:', result);
      errors = result.errors;
    } catch (error) {
      console.error('Archive failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function cancelOperation() {
    try {
      await api.pipeline.cancel();
    } catch (error) {
      console.error('Cancel failed:', error);
    }
  }
</script>

<!-- Update progress bar -->
{#if isProcessing}
  <div class="progress-container">
    <div class="progress-bar">
      <div
        class="progress-fill"
        style="width: {(progress.current / progress.total) * 100}%"
      />
    </div>
    <div class="progress-info">
      <span>{progress.current} / {progress.total}</span>
      <span>{progress.details}</span>
      <span>Rate: {progress.rate.toFixed(1)} files/sec</span>
      <span>ETA: {formatETA(progress.eta_seconds)}</span>
    </div>
    <button on:click={cancelOperation}>Cancel</button>
  </div>
{/if}
```

### Step 6: Fix MixerWindow

**File:** `app/src/lib/windows/MixerWindow.svelte`

**Add mixer controls:**
```svelte
<script lang="ts">
  import { api } from '$lib/api';
  import type { MixerState } from '$lib/types';

  let mixerState: MixerState | null = null;

  onMount(async () => {
    try {
      mixerState = await api.window.getMixerState();
    } catch (error) {
      console.error('Failed to load mixer state:', error);
    }
  });

  async function updateVolume(trackId: number, volume: number) {
    try {
      await api.window.setChannelVolume(trackId, volume);
    } catch (error) {
      console.error('Failed to set volume:', error);
    }
  }

  async function updatePan(trackId: number, pan: number) {
    try {
      await api.window.setChannelPan(trackId, pan);
    } catch (error) {
      console.error('Failed to set pan:', error);
    }
  }

  async function toggleMute(trackId: number) {
    try {
      const track = mixerState?.channels.find(c => c.track_id === trackId);
      if (track) {
        await api.window.setChannelMute(trackId, !track.muted);
        mixerState = await api.window.getMixerState();
      }
    } catch (error) {
      console.error('Failed to toggle mute:', error);
    }
  }

  async function toggleSolo(trackId: number) {
    try {
      const track = mixerState?.channels.find(c => c.track_id === trackId);
      if (track) {
        await api.window.setChannelSolo(trackId, !track.soloed);
        mixerState = await api.window.getMixerState();
      }
    } catch (error) {
      console.error('Failed to toggle solo:', error);
    }
  }
</script>

<!-- Mixer channel UI -->
{#if mixerState}
  {#each mixerState.channels as channel}
    <div class="mixer-channel">
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={channel.volume}
        on:input={(e) => updateVolume(channel.track_id, parseFloat(e.currentTarget.value))}
      />
      <input
        type="range"
        min="-1"
        max="1"
        step="0.01"
        value={channel.pan}
        on:input={(e) => updatePan(channel.track_id, parseFloat(e.currentTarget.value))}
      />
      <button on:click={() => toggleMute(channel.track_id)}>
        {channel.muted ? 'M' : ''}
      </button>
      <button on:click={() => toggleSolo(channel.track_id)}>
        {channel.soloed ? 'S' : ''}
      </button>
    </div>
  {/each}
{/if}
```

### Step 7: Fix DAWWindow

**File:** `app/src/lib/windows/DAWWindow.svelte`

**Add track loading and playback updates:**
```svelte
<script lang="ts">
  import { api } from '$lib/api';
  import type { TrackInfo, PlaybackState } from '$lib/types';

  let tracks: TrackInfo[] = [];
  let playbackState: PlaybackState | null = null;
  let bpm = 120;
  let updateInterval: number;

  onMount(async () => {
    // Load initial tracks
    try {
      tracks = await api.window.getAllWindowTracks();
      bpm = await api.window.getBpm();
    } catch (error) {
      console.error('Failed to load tracks:', error);
    }

    // Start playback position updates
    updateInterval = setInterval(async () => {
      try {
        playbackState = await api.window.getPlaybackState();
      } catch (error) {
        console.error('Failed to get playback state:', error);
      }
    }, 100);

    return () => {
      if (updateInterval) clearInterval(updateInterval);
    };
  });

  async function addTrack() {
    try {
      const trackId = await api.window.addWindowTrack(`Track ${tracks.length + 1}`);
      tracks = await api.window.getAllWindowTracks();
    } catch (error) {
      console.error('Failed to add track:', error);
    }
  }

  async function removeTrack(trackId: number) {
    try {
      await api.window.removeWindowTrack(trackId);
      tracks = await api.window.getAllWindowTracks();
    } catch (error) {
      console.error('Failed to remove track:', error);
    }
  }

  async function handlePlay() {
    try {
      await api.window.playTransport();
    } catch (error) {
      console.error('Failed to play:', error);
    }
  }

  async function handleStop() {
    try {
      await api.window.stopTransport();
    } catch (error) {
      console.error('Failed to stop:', error);
    }
  }

  async function handlePause() {
    try {
      await api.window.pauseTransport();
    } catch (error) {
      console.error('Failed to pause:', error);
    }
  }
</script>

<!-- Transport controls -->
<div class="transport">
  <button on:click={handlePlay}>Play</button>
  <button on:click={handlePause}>Pause</button>
  <button on:click={handleStop}>Stop</button>
  <div class="position">
    {#if playbackState}
      Bar: {playbackState.bar} | Beat: {playbackState.beat} | Tick: {playbackState.tick}
    {/if}
  </div>
  <div class="bpm">BPM: {bpm}</div>
</div>

<!-- Track list -->
<div class="tracks">
  <button on:click={addTrack}>Add Track</button>
  {#each tracks as track}
    <div class="track">
      <span>{track.label}</span>
      <button on:click={() => removeTrack(track.id)}>Remove</button>
    </div>
  {/each}
</div>
```

---

## Part 6: Verification Checklist

After implementing fixes, verify:

### Backend Registration
- [ ] Run `cargo build` - should succeed with no errors
- [ ] Check `main.rs` - all database commands registered
- [ ] Check `main.rs` - all pipeline commands registered
- [ ] Check `main.rs` - `PipelineState` added to `.manage()`

### Frontend API
- [ ] TypeScript compiles - no type errors
- [ ] All API modules defined in `api.ts`
- [ ] All types defined in `types.ts`
- [ ] No "cannot find name" errors

### Component Functionality
- [ ] DatabaseWindow: Search works
- [ ] DatabaseWindow: Favorite toggle works
- [ ] DatabaseWindow: Statistics display appears
- [ ] PipelineWindow: Import button functional
- [ ] PipelineWindow: Progress bar updates
- [ ] PipelineWindow: Analysis button works
- [ ] MixerWindow: Volume sliders work
- [ ] MixerWindow: Mute/Solo buttons work
- [ ] DAWWindow: Transport controls work
- [ ] DAWWindow: Track list loads
- [ ] DAWWindow: Playback position updates

### End-to-End Tests
- [ ] Import a MIDI file → appears in database
- [ ] Search for file → appears in results
- [ ] Add to favorites → star appears
- [ ] Load file into DAW → appears in track list
- [ ] Adjust mixer controls → audio changes
- [ ] Play transport → playback position updates

---

## Part 7: Estimated Effort

| Task | Estimated Time | Priority |
|------|---------------|----------|
| Register database commands | 10 minutes | HIGH |
| Register pipeline commands | 15 minutes | HIGH |
| Add database API wrappers | 20 minutes | HIGH |
| Fix pipeline API wrappers | 15 minutes | HIGH |
| Add missing type definitions | 20 minutes | MEDIUM |
| Fix DatabaseWindow | 30 minutes | MEDIUM |
| Fix PipelineWindow | 45 minutes | MEDIUM |
| Fix MixerWindow | 20 minutes | LOW |
| Fix DAWWindow | 25 minutes | LOW |
| Testing and verification | 60 minutes | HIGH |
| **TOTAL** | **4 hours** | |

---

## Conclusion

**Root Cause:** Backend commands were implemented but never registered in `main.rs`, and frontend API wrappers were either missing or calling the wrong commands.

**Impact:** Multiple UI features completely non-functional despite having working backend code.

**Solution:** Follow the 7-step fix plan above to:
1. Register missing commands in backend
2. Add missing API wrappers in frontend
3. Add missing type definitions
4. Connect UI components to API
5. Test end-to-end functionality

**Estimated Total Time:** 4 hours to full functionality.

---

**Report Generated:** 2025-11-12
**Next Step:** Begin with Step 1 (Register backend commands in main.rs)
