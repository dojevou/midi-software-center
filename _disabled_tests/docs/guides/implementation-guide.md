# MIDI Software Center Implementation Guide

This guide provides a hierarchical breakdown of the implementation process for rebuilding the MIDI Software Center. It starts with high-level steps, expands them into detailed steps, and further breaks them down into microsteps. The structure follows the approved plan, focusing on backend Tauri commands, frontend Svelte components, audio integration, database/Meilisearch setup, pipeline workflows, testing, and deployment.

## High-Level Steps

1. **Set up Project Foundation**
   - Prepare the development environment, dependencies, and initial structure.

2. **Implement Backend Tauri Commands**
   - Build Rust commands for DAW, Mixer, Database, and Pipeline functionality.

3. **Integrate Frontend Svelte Components**
   - Create and connect Svelte components to backend via Tauri invokes and events.

4. **Add Audio Engine with Tone.js and Web Audio API**
   - Implement playback, mixing, and real-time audio processing in the browser.

5. **Configure Database and Meilisearch**
   - Set up PostgreSQL schema and Meilisearch for metadata storage and search.

6. **Develop Pipeline Batch Processing**
   - Enable import, analysis, and archiving with progress tracking.

7. **Implement Integration and Testing**
   - Ensure end-to-end data flow and add unit/integration tests.

8. **Finalize Deployment and Verification**
   - Package the app, document setup, and verify full functionality.

## Detailed Steps

### 1. Set up Project Foundation
   - Install and configure tools, update dependencies, and initialize core files.

### 2. Implement Backend Tauri Commands
   - Define state structures, create command modules for each component, register handlers in main.rs, and add error handling/logging.

### 3. Integrate Frontend Svelte Components
   - Update stores for real data, enhance components with invokes/events, and style with Tailwind per GUI-LAYOUT-ASCII.md.

### 4. Add Audio Engine with Tone.js and Web Audio API
   - Install libraries, set up Tone.Transport for DAW, create Web Audio nodes for Mixer, and sync with backend events.

### 5. Configure Database and Meilisearch
   - Define schema migrations, set up connection pools, integrate Meilisearch client, and index sample data.

### 6. Develop Pipeline Batch Processing
   - Implement async tasks in Rust for file handling, add progress emission, and connect to frontend tabs.

### 7. Implement Integration and Testing
   - Write Rust unit tests, Svelte Vitest tests, and Playwright E2E scripts; test full flow from import to playback.

### 8. Finalize Deployment and Verification
   - Build for platforms, create setup scripts, and run verification checklist.

## Microsteps Breakdown

### 1. Set up Project Foundation
   #### 1.1 Install Dependencies
      - 1.1.1 Run `pnpm install` in app/ to add Svelte, Tone.js, Meilisearch JS client.
      - 1.1.2 Update Cargo.toml in daw/src-tauri/ to include midly, sqlx, meilisearch-rust, sysinfo.
      - 1.1.3 Add .env.example with DATABASE_URL and MEILISEARCH_URL.
   #### 1.2 Configure Environment
      - 1.2.1 Set up PostgreSQL DB with initial schema via sqlx migrate.
      - 1.2.2 Start Meilisearch instance (docker run getmeili/meilisearch).
      - 1.2.3 Update tauri.conf.json for window sizes per layout (1280x800).
   #### 1.3 Initialize Core Files
      - 1.3.1 Create commands/mod.rs with state structs (AppState, DawState, etc.).
      - 1.3.2 Update main.rs to manage states and register all handlers.
      - 1.3.3 Add logging with tracing in main.rs.

### 2. Implement Backend Tauri Commands
   #### 2.1 DAW Commands (commands/daw.rs)
      - 2.1.1 Define structs: Track, Clip, TransportState, PlaybackInfo.
      - 2.1.2 Implement transport: daw_play (start thread, emit event), daw_pause/stop/record.
      - 2.1.3 Add track mgmt: daw_add_track/remove/update, daw_set_mute/solo.
      - 2.1.4 MIDI loading: daw_load_midi_file (parse with midly, extract notes).
      - 2.1.5 Playback loop: async task for position updates, metronome clicks.
   #### 2.2 Mixer Commands (commands/mixer.rs)
      - 2.2.1 Define structs: MixerChannel, MasterChannel, MeterData.
      - 2.2.2 Channel controls: mixer_set_volume/pan/mute/solo, add/remove channel.
      - 2.2.3 Master controls: mixer_set_master_volume/limiter/compressor.
      - 2.2.4 Effects: mixer_add/remove_effect, set_enabled/wet_dry.
      - 2.2.5 Meter loop: async task for VU/peak simulation, emit updates.
   #### 2.3 Database Commands (commands/database.rs)
      - 2.3.1 Define structs: MidiFile, SearchFilters, SearchResults, DatabaseStats.
      - 2.3.2 Search: database_search (filter by query/tags/BPM/key, paginate).
      - 2.3.3 File mgmt: database_add/remove_file, update_metadata, get_recent.
      - 2.3.4 Stats: database_get_stats (counts, averages, common items).
      - 2.3.5 Index: Update search_index HashMap for tags/instruments.
   #### 2.4 Pipeline Commands (commands/pipeline.rs)
      - 2.4.1 Define structs: PipelineProgress, ImportStats, AnalysisResults.
      - 2.4.2 Import: pipeline_import_files (process files, emit progress, handle errors).
      - 2.4.3 Analyze: pipeline_analyze_files (simulate BPM/key detection).
      - 2.4.4 Archive: pipeline_archive_files (mock zip creation).
      - 2.4.5 Controls: get_progress, cancel, clean_temp_files.
   #### 2.5 System/Status/Menu Commands
      - 2.5.1 System: get_system_info (sysinfo for CPU/mem), get_audio/midi_devices (mock).
      - 2.5.2 Settings: get/update_settings (persist to file).
      - 2.5.3 Status: status_get_info, update_position/bpm, monitoring loop.
      - 2.5.4 Menu: Handlers for all menu items (print/log actions).
      - 2.5.5 Add error handling: Result<String> for all commands, tracing logs.

### 3. Integrate Frontend Svelte Components
   #### 3.1 Update Stores (lib/stores/)
      - 3.1.1 playbackStore.ts: Add real invokes (api.daw_play), subscribe to events.
      - 3.1.2 databaseStore.ts: Use api.database_search, handle pagination/filters.
      - 3.1.3 projectStore.ts: Sync tracks with api.daw_get_tracks/add.
      - 3.1.4 Add pipelineStore.ts for progress events.
   #### 3.2 Enhance Components (lib/windows/)
      - 3.2.1 DAWWindow.svelte: Bind transport buttons to actions, display real position/BPM.
      - 3.2.2 MixerWindow.svelte: Sliders invoke set_volume/pan, show VU from events.
      - 3.2.3 DatabaseWindow.svelte: Search input triggers api.database_search, filters dropdowns.
      - 3.2.4 PipelineWindow.svelte: Tabs invoke import/analyze, progress bar from events.
   #### 3.3 Add Real-Time Events
      - 3.3.1 Listen in App.svelte: tauri.event.listen for daw::position-updated, mixer::meter-update.
      - 3.3.2 Update UI: Reactive stores for live position, meters, progress.
      - 3.3.3 Style: Match GUI-LAYOUT-ASCII.md with Tailwind classes (dark theme, colors).

### 4. Add Audio Engine with Tone.js and Web Audio API
   #### 4.1 Install and Setup
      - 4.1.1 pnpm add tone @types/tone.
      - 4.1.2 Create audioContext in App.svelte, init Tone.start().
   #### 4.2 DAW Audio (DAWWindow.svelte)
      - 4.2.1 Transport: Tone.Transport.start/stop on play/pause, schedule events from loaded MIDI.
      - 4.2.2 Tracks: Create Tone.Player per clip, connect to master.
      - 4.2.3 Sync: Map backend position to Tone.Transport.position, handle loop/metronome.
   #### 4.3 Mixer Audio (MixerWindow.svelte)
      - 4.3.1 Nodes: GainNode for volume, StereoPannerNode for pan per track.
      - 4.3.2 VU Meters: AnalyserNode.getByteFrequencyData for levels, update every 50ms.
      - 4.3.3 Routing: Connect track nodes to master GainNode, handle mute/solo.
   #### 4.4 Integration
      - 4.4.1 Backend bridge: Commands to serialize MIDI events to JSON for Tone.
      - 4.4.2 Events: Emit audio-ready from frontend to backend for sync.

### 5. Configure Database and Meilisearch
   #### 5.1 PostgreSQL Schema
      - 5.1.1 Create migration: files table (id, path, name, bpm, key, tags JSONB).
      - 5.1.2 Add indexes: GIN on tags, B-tree on bpm/key.
      - 5.1.3 Update commands: Use sqlx for queries in database.rs.
   #### 5.2 Meilisearch Setup
      - 5.2.1 Create index: meilisearch.create_index("midi_files").
      - 5.2.2 Schema: Add searchableAttributes (name, tags), filterable (bpm, key).
      - 5.2.3 Sync: On import, add to Meilisearch via Rust client.
   #### 5.3 Frontend Client
      - 5.3.1 pnpm add meilisearch.
      - 5.3.2 In databaseStore: MeiliSearch client, search with filters.
      - 5.3.3 Test: Index sample MIDI files, query with BPM range.

### 6. Develop Pipeline Batch Processing
   #### 6.1 Rust Async Tasks
      - 6.1.1 Import: Use tokio::fs::read_dir, parse MIDI with midly, insert DB/Meili.
      - 6.1.2 Analyze: Detect BPM/key (use musicg or similar lib), update metadata.
      - 6.1.3 Archive: Use zip crate to create archives.
   #### 6.2 Progress Tracking
      - 6.2.1 Channels: Use tokio::sync::mpsc for progress updates.
      - 6.2.2 Emit: window.emit in loop, include current/total/ETA.
      - 6.2.3 Error handling: Collect errors, emit on failure.
   #### 6.3 Frontend Tabs
      - 6.3.1 PipelineWindow: Tabs for import/analyze/archive, drag-drop with tauri dialog.
      - 6.3.2 Progress: Bind to store, show bar/stats/ETA.
      - 6.3.3 Validation: Check file types/sizes before processing.

### 7. Implement Integration and Testing
   #### 7.1 Unit Tests (Rust)
      - 7.1.1 Cargo test for commands: Mock states, assert invocations/emits.
      - 7.1.2 Test MIDI parsing: Load sample.mid, verify notes/BPM.
      - 7.1.3 Search tests: Add files, query, assert results.
   #### 7.2 Frontend Tests (Vitest)
      - 7.2.1 Test stores: Mock api, assert state updates.
      - 7.2.2 Component tests: Render DAW, simulate play, check events.
      - 7.2.3 Audio tests: Mock Tone, verify node connections.
   #### 7.3 E2E Tests (Playwright)
      - 7.3.1 tauri dev, test import → search → load → play.
      - 7.3.2 Verify UI: Click play, assert position changes.
      - 7.3.3 Audio flow: Check console for no errors, mock playback.

### 8. Finalize Deployment and Verification
   #### 8.1 Build and Package
      - 8.1.1 tauri build for win/mac/linux.
      - 8.1.2 Include assets: Sample MIDI, DB schema script.
   #### 8.2 Setup Scripts
      - 8.2.1 setup.sh: Install deps, init DB/Meili, index samples.
      - 8.2.2 .env setup: DATABASE_URL, MEILISEARCH_URL.
   #### 8.3 Verification Checklist
      - 8.3.1 Run app, import MIDI, search/filter, load to DAW, play/mix.
      - 8.3.2 Check events: Position updates, VU meters, progress bars.
      - 8.3.3 Test errors: Invalid file, DB disconnect, recover gracefully.
      - 8.3.4 Performance: Load 100 files, measure search time <500ms.