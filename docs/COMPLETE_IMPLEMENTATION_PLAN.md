# Complete Implementation Plan - MIDI Software Center

**Generated:** 2025-12-15
**Based on:** `.github/copilot-instructions.md`
**Purpose:** Comprehensive checklist to implement EVERYTHING mentioned in the copilot instructions

---

## Table of Contents

1. [Core Infrastructure](#1-core-infrastructure)
2. [Database Layer](#2-database-layer)
3. [Pipeline System](#3-pipeline-system)
4. [DAW System](#4-daw-system)
5. [VIP3 Browser](#5-vip3-browser)
6. [MIDI Analysis Engine](#6-midi-analysis-engine)
7. [MIDI Hardware Integration](#7-midi-hardware-integration)
8. [Frontend Components](#8-frontend-components)
9. [API Layer](#9-api-layer)
10. [Testing Infrastructure](#10-testing-infrastructure)
11. [DevOps & Environment](#11-devops--environment)
12. [Missing Features (Future)](#12-missing-features-future)
13. [Verification & Quality Assurance](#13-verification--quality-assurance)

---

## 1. Core Infrastructure

### 1.1 Project Structure

- [ ] **Workspace setup**
  - [ ] `app/src-tauri/` - Main Rust backend
  - [ ] `app/src/` - Svelte frontend
  - [ ] `database/migrations/` - SQL migrations (001-019+)
  - [ ] `verification/` - Test suite
  - [ ] `scripts/` - CLI utilities and import tools

- [ ] **Build system**
  - [ ] `Makefile` with all commands:
    - [ ] `make setup` - Install dependencies
    - [ ] `make dev` - Start dev server
    - [ ] `make check` - Pre-commit validation
    - [ ] `make docker-up` - Start services
    - [ ] `make db-reset` - Reset database
    - [ ] `make db-migrate` - Run migrations
    - [ ] `make lint` - Clippy + format check
    - [ ] `make format` - Format code
    - [ ] `make test` - Run tests

- [ ] **Dependencies**
  - [ ] Rust toolchain (latest stable)
  - [ ] Node.js + npm
  - [ ] Docker + Docker Compose
  - [ ] `sqlx-cli` for migrations
  - [ ] `tarpaulin` for coverage
  - [ ] Platform-specific MIDI libraries (JACK/ALSA/CoreMIDI)

### 1.2 Core Rust Modules

- [ ] **`app/src-tauri/src/` structure**
  - [ ] `main.rs` - Tauri app entry point
  - [ ] `lib.rs` - Library exports
  - [ ] `core/` - Business logic
  - [ ] `db/` - Database layer
  - [ ] `commands/` - Tauri commands
  - [ ] `hardware/` - MIDI I/O
  - [ ] `sequencer/` - Playback engine
  - [ ] `midi_io/` - Device management
  - [ ] `windows/` - Window state management

### 1.3 Configuration

- [ ] **Environment variables**
  - [ ] `DATABASE_URL` - PostgreSQL connection
  - [ ] `MEILISEARCH_URL` - Search service
  - [ ] `RUST_LOG` - Logging configuration

- [ ] **Docker Compose services**
  - [ ] PostgreSQL 16 (port 5433)
  - [ ] Meilisearch (port 7700)
  - [ ] Redis (port 6379)

---

## 2. Database Layer

### 2.1 PostgreSQL Schema (15 Tables)

#### Core Tables
- [ ] **`files`** - MIDI file metadata
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `file_path` (text UNIQUE NOT NULL)
  - [ ] `content_hash` (text UNIQUE NOT NULL) - SHA256 for deduplication
  - [ ] `file_size` (bigint)
  - [ ] `format` (smallint) - MIDI format 0/1/2
  - [ ] `created_at`, `updated_at` (timestamptz)
  - [ ] Indexes: `idx_files_content_hash`, `idx_files_path`

- [ ] **`musical_metadata`** - Musical properties
  - [ ] `file_id` (bigint UNIQUE REFERENCES files)
  - [ ] `bpm` (real)
  - [ ] `key_signature` (text)
  - [ ] `time_signature` (text)
  - [ ] `duration_seconds` (real)
  - [ ] `duration_ticks` (bigint)
  - [ ] Indexes: `idx_musical_metadata_bpm`, `idx_musical_metadata_key`

- [ ] **`tags`** - Tag definitions (97 instruments)
  - [ ] `id` (serial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)
  - [ ] `category` (text) - instrument, genre, style, etc.
  - [ ] `description` (text)

- [ ] **`file_tags`** - Many-to-many file ↔ tags
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `tag_id` (int REFERENCES tags)
  - [ ] PRIMARY KEY (`file_id`, `tag_id`)
  - [ ] Indexes: `idx_file_tags_file_id`, `idx_file_tags_tag_id`

#### Organization Tables
- [ ] **`folders`** - Folder hierarchy
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `name` (text NOT NULL)
  - [ ] `path` (text UNIQUE NOT NULL)
  - [ ] `parent_id` (bigint REFERENCES folders)
  - [ ] Index: `idx_folders_parent_id`

- [ ] **`saved_searches`** - Named VIP3 search templates
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)
  - [ ] `filters` (jsonb NOT NULL) - SearchFilters serialized
  - [ ] `use_count` (int DEFAULT 0)
  - [ ] `last_used` (timestamptz)
  - [ ] Index: `idx_saved_searches_last_used`

- [ ] **`collections`** - Ordered file collections
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)
  - [ ] `description` (text)
  - [ ] `created_at` (timestamptz)

- [ ] **`collection_files`** - Files in collections
  - [ ] `collection_id` (bigint REFERENCES collections)
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `position` (int NOT NULL) - Order in collection
  - [ ] PRIMARY KEY (`collection_id`, `file_id`)

#### MIDI Structure Tables
- [ ] **`midi_tracks`** - Track metadata
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `track_number` (int)
  - [ ] `track_name` (text)
  - [ ] `instrument` (text)
  - [ ] `channel` (smallint)
  - [ ] `note_count` (int)
  - [ ] Index: `idx_midi_tracks_file_id`

- [ ] **`midi_events`** - Individual MIDI events
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `track_id` (bigint REFERENCES midi_tracks)
  - [ ] `tick` (bigint)
  - [ ] `event_type` (text) - NoteOn, NoteOff, ControlChange, etc.
  - [ ] `channel` (smallint)
  - [ ] `data` (jsonb) - Event-specific data
  - [ ] Index: `idx_midi_events_track_id`

- [ ] **`midi_clips`** - Clip/section markers
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `name` (text)
  - [ ] `start_tick` (bigint)
  - [ ] `end_tick` (bigint)
  - [ ] `track_id` (bigint REFERENCES midi_tracks)

- [ ] **`track_splits`** - Multi-track separation metadata
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `original_file_id` (bigint REFERENCES files)
  - [ ] `split_file_id` (bigint REFERENCES files)
  - [ ] `track_number` (int)
  - [ ] `split_timestamp` (timestamptz)

#### Analysis Tables
- [ ] **`analysis_results`** - JSON analysis output
  - [ ] `file_id` (bigint UNIQUE REFERENCES files)
  - [ ] `analysis_type` (text) - bpm, key, drum, chord, structure
  - [ ] `results` (jsonb NOT NULL)
  - [ ] `analyzed_at` (timestamptz)
  - [ ] Index: `idx_analysis_results_type`

- [ ] **`drum_patterns`** - Drum-specific analysis
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `pattern_data` (jsonb)
  - [ ] `kick_hits` (int)
  - [ ] `snare_hits` (int)
  - [ ] `hihat_hits` (int)
  - [ ] `has_drums` (boolean)

- [ ] **`chords`** - Chord progressions
  - [ ] `id` (bigserial PRIMARY KEY)
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `tick_position` (bigint)
  - [ ] `chord_name` (text)
  - [ ] `root_note` (text)
  - [ ] `chord_type` (text) - major, minor, diminished, etc.
  - [ ] `confidence` (real)

#### VIP3 Category Tables
- [ ] **`timbres`** - Timbre category (sound character)
  - [ ] `id` (serial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)
  - [ ] `description` (text)

- [ ] **`file_timbres`** - Many-to-many file ↔ timbres
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `timbre_id` (int REFERENCES timbres)
  - [ ] PRIMARY KEY (`file_id`, `timbre_id`)
  - [ ] Partial index for filter performance

- [ ] **`styles`** - Musical style category
  - [ ] `id` (serial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)

- [ ] **`file_styles`** - Many-to-many file ↔ styles
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `style_id` (int REFERENCES styles)
  - [ ] PRIMARY KEY (`file_id`, `style_id`)

- [ ] **`articulations`** - Playing technique category
  - [ ] `id` (serial PRIMARY KEY)
  - [ ] `name` (text UNIQUE NOT NULL)

- [ ] **`file_articulations`** - Many-to-many file ↔ articulations
  - [ ] `file_id` (bigint REFERENCES files)
  - [ ] `articulation_id` (int REFERENCES articulations)
  - [ ] PRIMARY KEY (`file_id`, `articulation_id`)

### 2.2 Database Functions

- [ ] **PostgreSQL functions**
  - [ ] `get_files_by_instrument(instrument_name text)` - Returns files with specific instrument
  - [ ] `get_files_by_bpm_range(min_bpm real, max_bpm real)` - BPM range query
  - [ ] `get_files_by_instruments(instrument_names text[])` - Multi-instrument query
  - [ ] Additional helper functions for complex queries

### 2.3 Indexes (60+ total)

- [ ] **Performance indexes**
  - [ ] B-tree indexes on foreign keys
  - [ ] Partial indexes on category junction tables
  - [ ] GiST indexes for full-text search (if using PostgreSQL search)
  - [ ] Hash indexes on content_hash for deduplication
  - [ ] Composite indexes for common query patterns

### 2.4 Migrations

- [ ] **Migration files** (`database/migrations/`)
  - [ ] `001_initial_schema.sql` - Core tables
  - [ ] `002-011_*.sql` - Additional features
  - [ ] `012-018_*.sql` - VIP3 categories, collections, etc.
  - [ ] `019_*.sql` - Latest schema updates
  - [ ] All migrations idempotent and reversible
  - [ ] **Never edit existing migrations - always create new**

### 2.5 Repository Pattern Implementation

- [ ] **`app/src-tauri/src/db/repositories/`**

#### FileRepository
- [ ] `insert(&self, file: CreateMidiFile) -> DbResult<i64>`
  - [ ] Check `content_hash` for deduplication
  - [ ] Return existing file ID if duplicate found
  - [ ] Insert new file if unique
- [ ] `search(&self, query: &str, filters: SearchFilters) -> DbResult<Vec<MidiFile>>`
  - [ ] Parameterized queries (SQL injection prevention)
  - [ ] Support pagination
- [ ] `get_by_id(&self, id: i64) -> DbResult<Option<MidiFile>>`
- [ ] `get_by_hash(&self, hash: &str) -> DbResult<Option<MidiFile>>`
- [ ] `update(&self, id: i64, updates: UpdateMidiFile) -> DbResult<()>`
- [ ] `delete(&self, id: i64) -> DbResult<()>`
- [ ] `batch_insert(&self, files: Vec<CreateMidiFile>) -> DbResult<Vec<i64>>`
  - [ ] Use PostgreSQL COPY for high performance
  - [ ] Handle deduplication in batch

#### TagRepository
- [ ] `create_tag(&self, name: &str, category: &str) -> DbResult<i64>`
- [ ] `get_all_tags(&self) -> DbResult<Vec<Tag>>`
- [ ] `add_tag_to_file(&self, file_id: i64, tag_id: i32) -> DbResult<()>`
- [ ] `remove_tag_from_file(&self, file_id: i64, tag_id: i32) -> DbResult<()>`
- [ ] `get_tags_for_file(&self, file_id: i64) -> DbResult<Vec<Tag>>`
- [ ] `bulk_tag_files(&self, file_ids: Vec<i64>, tag_id: i32) -> DbResult<()>`

#### FolderRepository
- [ ] `create_folder(&self, name: &str, path: &str, parent_id: Option<i64>) -> DbResult<i64>`
- [ ] `get_folder_hierarchy(&self) -> DbResult<Vec<FolderNode>>`
- [ ] `move_folder(&self, folder_id: i64, new_parent_id: i64) -> DbResult<()>`
- [ ] `delete_folder(&self, folder_id: i64, recursive: bool) -> DbResult<()>`

#### SearchRepository
- [ ] `save_search(&self, name: &str, filters: SearchFilters) -> DbResult<i64>`
- [ ] `load_saved_search(&self, id: i64) -> DbResult<SavedSearch>`
- [ ] `update_search_usage(&self, id: i64) -> DbResult<()>` - Increment use_count, update last_used
- [ ] `get_saved_searches(&self) -> DbResult<Vec<SavedSearch>>`
- [ ] `delete_saved_search(&self, id: i64) -> DbResult<()>`

#### CollectionRepository
- [ ] `create_collection(&self, name: &str, description: &str) -> DbResult<i64>`
- [ ] `add_file_to_collection(&self, collection_id: i64, file_id: i64, position: i32) -> DbResult<()>`
- [ ] `remove_file_from_collection(&self, collection_id: i64, file_id: i64) -> DbResult<()>`
- [ ] `reorder_collection(&self, collection_id: i64, file_id: i64, new_position: i32) -> DbResult<()>`
- [ ] `get_collection_files(&self, collection_id: i64) -> DbResult<Vec<MidiFile>>`
- [ ] `delete_collection(&self, collection_id: i64) -> DbResult<()>`

#### VIP3Repository
- [ ] `add_timbre_to_file(&self, file_id: i64, timbre_id: i32) -> DbResult<()>`
- [ ] `remove_timbre_from_file(&self, file_id: i64, timbre_id: i32) -> DbResult<()>`
- [ ] `add_style_to_file(&self, file_id: i64, style_id: i32) -> DbResult<()>`
- [ ] `remove_style_from_file(&self, file_id: i64, style_id: i32) -> DbResult<()>`
- [ ] `add_articulation_to_file(&self, file_id: i64, articulation_id: i32) -> DbResult<()>`
- [ ] `remove_articulation_from_file(&self, file_id: i64, articulation_id: i32) -> DbResult<()>`
- [ ] `get_filter_counts(&self, current_filters: SearchFilters) -> DbResult<FilterCounts>`
  - [ ] Real-time count updates for each filter value
  - [ ] Uses COUNT DISTINCT on indexed columns
  - [ ] Target: <50ms response time

#### AnalysisRepository
- [ ] `save_analysis_result(&self, file_id: i64, analysis_type: &str, results: serde_json::Value) -> DbResult<()>`
- [ ] `get_analysis_result(&self, file_id: i64, analysis_type: &str) -> DbResult<Option<serde_json::Value>>`
- [ ] `save_drum_pattern(&self, file_id: i64, pattern: DrumPattern) -> DbResult<()>`
- [ ] `save_chords(&self, file_id: i64, chords: Vec<Chord>) -> DbResult<()>`

### 2.6 Database Models

- [ ] **`app/src-tauri/src/db/models/`**
  - [ ] `file.rs` - `MidiFile`, `CreateMidiFile`, `UpdateMidiFile`
  - [ ] `musical_metadata.rs` - `MusicalMetadata`
  - [ ] `tag.rs` - `Tag`, `CreateTag`
  - [ ] `folder.rs` - `Folder`, `FolderNode` (with hierarchy)
  - [ ] `saved_search.rs` - `SavedSearch`
  - [ ] `collection.rs` - `Collection`, `CollectionFile`
  - [ ] `midi_track.rs` - `MidiTrack`, `MidiEvent`
  - [ ] `analysis.rs` - `AnalysisResult`, `DrumPattern`, `Chord`
  - [ ] `vip3.rs` - `Timbre`, `Style`, `Articulation`, `FilterCounts`

### 2.7 Error Handling

- [ ] **Custom error types** (`app/src-tauri/src/db/error.rs`)
  - [ ] `DbError` enum with `thiserror`
    - [ ] `QueryFailed(String)`
    - [ ] `FileNotFound { path: String }`
    - [ ] `DuplicateEntry { hash: String }`
    - [ ] `ConnectionError(sqlx::Error)`
    - [ ] `MigrationError(String)`
  - [ ] `type DbResult<T> = Result<T, DbError>`
  - [ ] Convert to String in Tauri commands: `.map_err(|e| e.to_string())`

---

## 3. Pipeline System

### 3.1 Import Phase

- [ ] **File discovery**
  - [ ] Recursive directory scanning
  - [ ] MIDI file detection (.mid, .midi)
  - [ ] Symlink handling (optional: follow or skip)
  - [ ] Hidden file filtering

- [ ] **Deduplication**
  - [ ] SHA256 content hashing
  - [ ] Hash comparison before insert
  - [ ] Return existing file ID on duplicate
  - [ ] Performance: 7,830 files/sec target

- [ ] **MIDI parsing**
  - [ ] Use `midly` crate for parsing
  - [ ] Extract format (0/1/2)
  - [ ] Extract track count
  - [ ] Handle corrupted files gracefully
  - [ ] Auto-repair common issues

- [ ] **Initial tagging**
  - [ ] Extract track names for instrument detection
  - [ ] Apply auto-tagger (97 instrument categories)
  - [ ] Store in `file_tags` junction table

- [ ] **Database insertion**
  - [ ] Batch insert using COPY
  - [ ] Transaction per batch (not per file)
  - [ ] Progress events: `emit('import_progress', { processed, total, rate })`

- [ ] **Statistics tracking**
  - [ ] `ImportStats` struct:
    - [ ] `total_files: usize`
    - [ ] `successful: usize`
    - [ ] `failed: usize`
    - [ ] `duplicates: usize`
    - [ ] `errors: Vec<(String, String)>` - (path, error)
    - [ ] `elapsed_seconds: f64`
    - [ ] `files_per_second: f64`

### 3.2 Sanitize Phase

- [ ] **Filename cleaning**
  - [ ] Remove special characters
  - [ ] Normalize spaces
  - [ ] Handle encoding issues (UTF-8 conversion)
  - [ ] Preserve original path in database

- [ ] **Path validation**
  - [ ] Check for illegal characters
  - [ ] Ensure path uniqueness after sanitization
  - [ ] Update database with sanitized paths

### 3.3 Split Phase (Multi-Track Separation)

- [ ] **Track extraction**
  - [ ] Parse MIDI format 1 files
  - [ ] Extract individual tracks
  - [ ] Create separate MIDI files (one per track)
  - [ ] Store split metadata in `track_splits` table

- [ ] **Auto-repair**
  - [ ] Fix timing issues (missing tempo events)
  - [ ] Add missing track names
  - [ ] Normalize tick resolution
  - [ ] Handle format 0 → format 1 conversion

- [ ] **Batch operations**
  - [ ] `split_file(file_id: i64) -> Result<Vec<i64>, String>`
  - [ ] `split_file_batch(file_ids: Vec<i64>) -> Result<SplitStats, String>`
  - [ ] Progress tracking for large batches

### 3.4 Analyze Phase

- [ ] **BPM detection** (`app/src-tauri/src/core/analysis/bpm_detector.rs`)
  - [ ] Onset detection from MIDI events
  - [ ] Inter-onset interval (IOI) analysis
  - [ ] Autocorrelation for tempo estimation
  - [ ] Handle tempo changes (use average or first tempo)
  - [ ] Performance: 181-360 files/sec target

- [ ] **Key detection** (`app/src-tauri/src/core/analysis/key_detector.rs`)
  - [ ] Pitch class histogram
  - [ ] Krumhansl-Schmuckler key-finding algorithm
  - [ ] Major/minor key classification
  - [ ] Confidence scoring

- [ ] **Drum analysis** (`app/src-tauri/src/core/analysis/drum_analyzer.rs`)
  - [ ] Detect drum channel (usually channel 10)
  - [ ] Count hits per drum type (kick, snare, hihat, etc.)
  - [ ] Identify drum patterns
  - [ ] Store in `drum_patterns` table

- [ ] **Chord detection**
  - [ ] Note aggregation in time windows
  - [ ] Chord identification (major, minor, diminished, augmented, 7th, etc.)
  - [ ] Root note extraction
  - [ ] Confidence scoring
  - [ ] Store in `chords` table

- [ ] **Structure analysis**
  - [ ] Identify sections (intro, verse, chorus, etc.)
  - [ ] Detect repetitions
  - [ ] Store in `analysis_results` as JSON

### 3.5 Organize Phase

- [ ] **Instrument tagging** (`app/src-tauri/src/core/analysis/auto_tagger.rs`)
  - [ ] 97 instrument categories
  - [ ] Track name parsing (regex patterns)
  - [ ] MIDI program change detection
  - [ ] Bulk tagging via `bulk_tag_files()`
  - [ ] Script: `./scripts/organize-database.sh`

- [ ] **Folder organization**
  - [ ] Auto-create folders based on tags
  - [ ] Move files to organized structure
  - [ ] Preserve original paths

### 3.6 Rename Phase

- [ ] **Metadata-based renaming**
  - [ ] Pattern: `{instrument}_{bpm}_{key}_{original_name}.mid`
  - [ ] Safe renaming (check for conflicts)
  - [ ] Update database paths

### 3.7 Pipeline Commands

- [ ] **Tauri commands** (`app/src-tauri/src/commands/pipeline/`)
  - [ ] `import_files(paths: Vec<String>) -> Result<ImportStats, String>`
  - [ ] `split_file(file_id: i64) -> Result<Vec<i64>, String>`
  - [ ] `split_file_batch(file_ids: Vec<i64>) -> Result<SplitStats, String>`
  - [ ] `analyze_file(file_id: i64) -> Result<AnalysisResult, String>`
  - [ ] `analyze_batch(file_ids: Vec<i64>) -> Result<AnalysisStats, String>`
  - [ ] `bulk_retag_vip3(tag_mappings: Vec<TagMapping>) -> Result<RetagStats, String>`

---

## 4. DAW System

### 4.1 Sequencer Engine

- [ ] **Core sequencer** (`app/src-tauri/src/sequencer/`)
  - [ ] `SequencerEngine` struct with `Arc<Mutex<State>>`
  - [ ] Lock-free ringbuffers for audio thread communication
  - [ ] Playback state machine (stopped, playing, paused)
  - [ ] Real-time clock (bar/beat/tick tracking)
  - [ ] Tempo control (BPM)
  - [ ] Time signature support

- [ ] **Track management**
  - [ ] `Track` struct with MIDI events
  - [ ] Add/remove tracks dynamically
  - [ ] Per-track mute/solo
  - [ ] Track routing (assign to MIDI outputs)

- [ ] **Event scheduling**
  - [ ] Sample-accurate event timing
  - [ ] Event queue sorted by tick position
  - [ ] Look-ahead buffer for smooth playback
  - [ ] MIDI event types: NoteOn, NoteOff, CC, PitchBend, etc.

### 4.2 Sequencer Commands

- [ ] **Playback control** (`app/src-tauri/src/commands/daw/sequencer.rs`)
  - [ ] `start_sequencer(engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
    - [ ] Fail if already playing
    - [ ] Initialize playback from current position
  - [ ] `pause_sequencer() -> Result<(), String>`
    - [ ] Pause without resetting position
  - [ ] `resume_sequencer() -> Result<(), String>`
    - [ ] Resume from paused position
  - [ ] `stop_sequencer() -> Result<(), String>`
    - [ ] Stop and reset to start
    - [ ] Send all-notes-off to prevent stuck notes

- [ ] **Position control**
  - [ ] `seek_position(bar: u32, beat: u32) -> Result<(), String>`
  - [ ] `get_playback_position() -> Result<Position, String>`
    - [ ] Return: `{ bar, beat, tick, seconds }`

- [ ] **Tempo control**
  - [ ] `set_tempo(bpm: f64) -> Result<(), String>`
  - [ ] `get_tempo() -> Result<f64, String>`

- [ ] **Track management**
  - [ ] `add_track(track: Track) -> Result<u32, String>` - Returns track ID
  - [ ] `remove_track(track_id: u32) -> Result<(), String>`
  - [ ] `get_tracks() -> Result<Vec<Track>, String>`
  - [ ] `update_track(track_id: u32, updates: TrackUpdates) -> Result<(), String>`

### 4.3 MIDI Hardware Integration

- [ ] **Device management** (`app/src-tauri/src/midi_io/`)
  - [ ] `MidiDeviceManager` - Enumerate and manage devices
  - [ ] Device connect/disconnect events
  - [ ] Hot-plug support (detect new devices)

- [ ] **MIDI I/O commands** (`app/src-tauri/src/commands/daw/midi.rs`)
  - [ ] `midi_list_devices() -> Result<Vec<MidiDevice>, String>`
    - [ ] Return: `{ name, id, is_input, is_output }`
  - [ ] `midi_connect(device_name: String) -> Result<(), String>`
  - [ ] `midi_disconnect() -> Result<(), String>`
  - [ ] `midi_is_connected() -> Result<bool, String>`
  - [ ] `midi_get_current_device() -> Result<Option<MidiDevice>, String>`
  - [ ] `midi_send_test_note(channel: u8, note: u8, velocity: u8) -> Result<(), String>`

### 4.4 Mixer System

- [ ] **Mixer core** (`app/src-tauri/src/sequencer/mixer.rs`)
  - [ ] Per-track channel strips
  - [ ] Gain (volume) control: -∞ to +12 dB
  - [ ] Pan control: -100% (left) to +100% (right)
  - [ ] Mute/solo state
  - [ ] VU metering (peak/RMS)
  - [ ] Master channel (all tracks mix down)

- [ ] **Effect chain**
  - [ ] Per-track effect slots (8 slots per track)
  - [ ] Effect types: EQ, Compressor, Reverb, Delay, etc.
  - [ ] Bypass individual effects
  - [ ] Wet/dry mix control

- [ ] **Routing**
  - [ ] Send/return buses
  - [ ] Sidechain routing
  - [ ] MIDI output assignment per track

- [ ] **Mixer commands** (`app/src-tauri/src/commands/daw/mixer.rs`)
  - [ ] **Channel control:**
    - [ ] `set_channel_gain(track_id: u32, gain_db: f32) -> Result<(), String>`
    - [ ] `get_channel_gain(track_id: u32) -> Result<f32, String>`
    - [ ] `set_channel_pan(track_id: u32, pan: f32) -> Result<(), String>` - -1.0 to +1.0
    - [ ] `get_channel_pan(track_id: u32) -> Result<f32, String>`
    - [ ] `set_channel_mute(track_id: u32, muted: bool) -> Result<(), String>`
    - [ ] `get_channel_mute(track_id: u32) -> Result<bool, String>`
    - [ ] `set_channel_solo(track_id: u32, solo: bool) -> Result<(), String>`
    - [ ] `get_channel_solo(track_id: u32) -> Result<bool, String>`

  - [ ] **VU metering:**
    - [ ] `get_channel_meter(track_id: u32) -> Result<MeterLevel, String>`
      - [ ] Return: `{ peak_db, rms_db, clip_detected }`
    - [ ] `get_all_meters() -> Result<Vec<MeterLevel>, String>`

  - [ ] **Master channel:**
    - [ ] `set_master_volume(gain_db: f32) -> Result<(), String>`
    - [ ] `get_master_volume() -> Result<f32, String>`
    - [ ] `get_master_meter() -> Result<MeterLevel, String>`

  - [ ] **Effect chains:**
    - [ ] `add_effect(track_id: u32, slot: u8, effect_type: String, params: serde_json::Value) -> Result<(), String>`
    - [ ] `remove_effect(track_id: u32, slot: u8) -> Result<(), String>`
    - [ ] `bypass_effect(track_id: u32, slot: u8, bypassed: bool) -> Result<(), String>`
    - [ ] `set_effect_param(track_id: u32, slot: u8, param: String, value: f32) -> Result<(), String>`
    - [ ] `get_effect_chain(track_id: u32) -> Result<Vec<Effect>, String>`

  - [ ] **Routing:**
    - [ ] `set_channel_output(track_id: u32, output_device: String) -> Result<(), String>`
    - [ ] `get_channel_output(track_id: u32) -> Result<String, String>`
    - [ ] `create_send_bus(name: String) -> Result<u32, String>`
    - [ ] `route_to_send(track_id: u32, send_id: u32, amount: f32) -> Result<(), String>`

### 4.5 Automation System

- [ ] **Automation core** (`app/src-tauri/src/sequencer/automation.rs`)
  - [ ] Parameter automation lanes
  - [ ] Automation recording (latch, touch, write modes)
  - [ ] Automation playback
  - [ ] Automation point editing (add/remove/move)
  - [ ] Curve types: linear, exponential, bezier

- [ ] **Automation commands** (`app/src-tauri/src/commands/daw/automation.rs`)
  - [ ] `record_automation(track_id: u32, param: String) -> Result<(), String>`
  - [ ] `play_automation(track_id: u32, param: String) -> Result<(), String>`
  - [ ] `clear_automation(track_id: u32, param: String) -> Result<(), String>`
  - [ ] `add_automation_point(track_id: u32, param: String, tick: u64, value: f32) -> Result<(), String>`
  - [ ] `remove_automation_point(track_id: u32, param: String, tick: u64) -> Result<(), String>`
  - [ ] `get_automation_lane(track_id: u32, param: String) -> Result<Vec<AutomationPoint>, String>`

### 4.6 Preset System

- [ ] **Preset storage**
  - [ ] Database table: `presets` (id, name, preset_type, data JSONB)
  - [ ] Local JSON files: `~/.midi-software-center/presets/`
  - [ ] Preset types: track, mixer, effect, project

- [ ] **Preset commands** (`app/src-tauri/src/commands/daw/presets.rs`)
  - [ ] `save_preset(name: String, preset_type: String, data: serde_json::Value) -> Result<i64, String>`
  - [ ] `load_preset(id: i64) -> Result<Preset, String>`
  - [ ] `delete_preset(id: i64) -> Result<(), String>`
  - [ ] `get_presets(preset_type: Option<String>) -> Result<Vec<Preset>, String>`
  - [ ] `update_preset(id: i64, updates: PresetUpdates) -> Result<(), String>`

### 4.7 Project Management

- [ ] **Project structure**
  - [ ] Database table: `projects` (id, name, file_path, created_at, updated_at)
  - [ ] Project file format: JSON with all session state
  - [ ] Include: tracks, mixer state, automation, tempo, markers

- [ ] **Project commands** (`app/src-tauri/src/commands/daw/project.rs`)
  - [ ] `create_project(name: String) -> Result<i64, String>`
  - [ ] `open_project(project_id: i64) -> Result<Project, String>`
    - [ ] Load all tracks into sequencer
    - [ ] Restore mixer state
    - [ ] Load automation
  - [ ] `save_project(project_id: i64) -> Result<(), String>`
    - [ ] Serialize current session state
    - [ ] Save to database and JSON file
  - [ ] `save_project_as(name: String) -> Result<i64, String>`
  - [ ] `close_project() -> Result<(), String>`
    - [ ] Clear sequencer state
    - [ ] Prompt to save if modified
  - [ ] `get_recent_projects(limit: u32) -> Result<Vec<Project>, String>`
  - [ ] `delete_project(project_id: i64) -> Result<(), String>`

### 4.8 MIDI File Loading

- [ ] **MIDI parser integration**
  - [ ] `daw_load_midi_file(file_path: String) -> Result<Track, String>`
    - [ ] Parse MIDI file using `midly`
    - [ ] Convert to internal Track format
    - [ ] Return Track ready for sequencer

- [ ] **Convenience wrapper** (suggested feature)
  - [ ] `load_file_to_daw(file_id: i64) -> Result<u32, String>`
    - [ ] Fetch file path from database
    - [ ] Call `daw_load_midi_file()`
    - [ ] Call `add_track()` automatically
    - [ ] Return track ID

### 4.9 MIDI Repair & Utilities

- [ ] **Repair commands** (`app/src-tauri/src/commands/daw/repair.rs`)
  - [ ] `repair_midi_file(file_path: String) -> Result<RepairReport, String>`
    - [ ] Fix corrupted headers
    - [ ] Add missing end-of-track events
    - [ ] Fix timing issues
    - [ ] Return repair report with issues found
  - [ ] `analyze_midi(file_path: String) -> Result<MidiAnalysis, String>`
    - [ ] Full musical analysis (BPM, key, drums, chords)
    - [ ] Return comprehensive analysis JSON
  - [ ] `export_midi(file_path: String, format: MidiFormat) -> Result<String, String>`
    - [ ] Export with format selection (0/1/2)
    - [ ] Return path to exported file

---

## 5. VIP3 Browser

### 5.1 Search System

- [ ] **Dynamic filter building** (`app/src-tauri/src/commands/pipeline/vip3/search.rs`)
  - [ ] `search_files_vip3(filters: Vip3Filters) -> Result<SearchResults, String>`
  - [ ] Build parameterized WHERE clauses dynamically
  - [ ] Support multiple simultaneous filters (up to 8)
  - [ ] Pagination (max 500 items/page)
  - [ ] Performance target: <100ms for multi-filter queries

- [ ] **Filter types**
  - [ ] `folder_ids: Option<Vec<i64>>`
  - [ ] `instrument_ids: Option<Vec<i64>>`
  - [ ] `timbre_ids: Option<Vec<i32>>`
  - [ ] `style_ids: Option<Vec<i32>>`
  - [ ] `articulation_ids: Option<Vec<i32>>`
  - [ ] `bpm_range_ids: Option<Vec<i32>>` - Predefined BPM ranges
  - [ ] `key_ids: Option<Vec<i32>>` - Key signatures
  - [ ] `channel: Option<u8>` - MIDI channel filter
  - [ ] `search_query: Option<String>` - Full-text search
  - [ ] `favorites_only: Option<bool>`
  - [ ] `tag_ids: Option<Vec<i32>>`
  - [ ] `min_rating: Option<i32>` - 1-5 star rating
  - [ ] `limit: Option<i32>` - Default 100, max 500
  - [ ] `offset: Option<i32>` - For pagination

- [ ] **SQL pattern**
  - [ ] Use PostgreSQL `ANY($::type[])` for multi-select filters
  - [ ] Build dynamic WHERE: `1=1 AND f.folder_id = ANY($1) AND EXISTS(SELECT...timbres...) AND...`
  - [ ] All queries parameterized (SQL injection prevention)

### 5.2 Filter Count System

- [ ] **Real-time counts** (`app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs`)
  - [ ] `get_vip3_filter_counts(current_filters: Vip3Filters) -> Result<FilterCounts, String>`
  - [ ] Return counts for each filter value (how many files match)
  - [ ] Performance: <50ms using partial indexes
  - [ ] Update on every filter change

- [ ] **FilterCounts structure**
  - [ ] `folder_counts: HashMap<i64, usize>`
  - [ ] `instrument_counts: HashMap<i64, usize>`
  - [ ] `timbre_counts: HashMap<i32, usize>`
  - [ ] `style_counts: HashMap<i32, usize>`
  - [ ] `articulation_counts: HashMap<i32, usize>`
  - [ ] `bpm_range_counts: HashMap<i32, usize>`
  - [ ] `key_counts: HashMap<i32, usize>`
  - [ ] `total_matches: usize`

### 5.3 Favorites System

- [ ] **Database table**
  - [ ] `favorites` (user_id, file_id, created_at)
  - [ ] Index on `user_id` for fast lookups

- [ ] **Favorites commands** (`app/src-tauri/src/commands/pipeline/vip3/favorites.rs`)
  - [ ] `toggle_favorite(file_id: i64) -> Result<bool, String>` - Returns new state
  - [ ] `get_favorites(limit: Option<i32>, offset: Option<i32>) -> Result<Vec<MidiFile>, String>`
  - [ ] `remove_favorite(file_id: i64) -> Result<(), String>`
  - [ ] `is_favorite(file_id: i64) -> Result<bool, String>`

### 5.4 Saved Searches

- [ ] **Saved search commands** (`app/src-tauri/src/commands/pipeline/vip3/saved_searches.rs`)
  - [ ] `save_search(name: String, filters: Vip3Filters) -> Result<i64, String>`
  - [ ] `load_saved_search(id: i64) -> Result<SavedSearch, String>`
    - [ ] Increment `use_count`
    - [ ] Update `last_used` timestamp
  - [ ] `get_saved_searches() -> Result<Vec<SavedSearch>, String>`
    - [ ] Order by `last_used` DESC (most recent first)
  - [ ] `delete_saved_search(id: i64) -> Result<(), String>`
  - [ ] `update_saved_search(id: i64, name: String, filters: Vip3Filters) -> Result<(), String>`

### 5.5 Collections

- [ ] **Collection commands** (`app/src-tauri/src/commands/pipeline/vip3/collections.rs`)
  - [ ] `create_collection(name: String, description: String) -> Result<i64, String>`
  - [ ] `add_file_to_collection(collection_id: i64, file_id: i64, position: i32) -> Result<(), String>`
  - [ ] `remove_file_from_collection(collection_id: i64, file_id: i64) -> Result<(), String>`
  - [ ] `reorder_collection(collection_id: i64, file_id: i64, new_position: i32) -> Result<(), String>`
  - [ ] `get_collection_files(collection_id: i64) -> Result<Vec<MidiFile>, String>` - Ordered by position
  - [ ] `get_collections() -> Result<Vec<Collection>, String>`
  - [ ] `delete_collection(collection_id: i64) -> Result<(), String>`

### 5.6 Category Management

- [ ] **Timbre commands** (`app/src-tauri/src/commands/pipeline/vip3/categories.rs`)
  - [ ] `add_timbre_to_file(file_id: i64, timbre_id: i32) -> Result<(), String>`
  - [ ] `remove_timbre_from_file(file_id: i64, timbre_id: i32) -> Result<(), String>`
  - [ ] `get_all_timbres() -> Result<Vec<Timbre>, String>`
  - [ ] `create_timbre(name: String, description: String) -> Result<i32, String>`

- [ ] **Style commands**
  - [ ] `add_style_to_file(file_id: i64, style_id: i32) -> Result<(), String>`
  - [ ] `remove_style_from_file(file_id: i64, style_id: i32) -> Result<(), String>`
  - [ ] `get_all_styles() -> Result<Vec<Style>, String>`
  - [ ] `create_style(name: String) -> Result<i32, String>`

- [ ] **Articulation commands**
  - [ ] `add_articulation_to_file(file_id: i64, articulation_id: i32) -> Result<(), String>`
  - [ ] `remove_articulation_from_file(file_id: i64, articulation_id: i32) -> Result<(), String>`
  - [ ] `get_all_articulations() -> Result<Vec<Articulation>, String>`
  - [ ] `create_articulation(name: String) -> Result<i32, String>`

---

## 6. MIDI Analysis Engine

### 6.1 Core Parser

- [ ] **`app/src-tauri/src/core/midi/analysis_parser.rs`**
  - [ ] Parse MIDI files using `midly` crate
  - [ ] Extract tracks, events, metadata
  - [ ] Handle all MIDI formats (0/1/2)
  - [ ] Pure function (no I/O): `pub fn parse_midi(data: &[u8]) -> Result<MidiData>`
  - [ ] Return strongly-typed MIDI structures

### 6.2 BPM Detector

- [ ] **`app/src-tauri/src/core/analysis/bpm_detector.rs`**
  - [ ] Extract tempo from MIDI meta events (SetTempo)
  - [ ] If no tempo events, use onset detection:
    - [ ] Collect all NoteOn events
    - [ ] Calculate inter-onset intervals (IOI)
    - [ ] Use autocorrelation to find period
    - [ ] Convert period to BPM
  - [ ] Handle tempo changes (average, first, or return all)
  - [ ] Pure function: `pub fn detect_bpm(midi_data: &MidiData) -> Result<f64>`
  - [ ] Performance: 181-360 files/sec

### 6.3 Key Detector

- [ ] **`app/src-tauri/src/core/analysis/key_detector.rs`**
  - [ ] Check MIDI KeySignature events first
  - [ ] If no key signature, use analysis:
    - [ ] Build pitch class histogram (12 bins, C through B)
    - [ ] Apply Krumhansl-Schmuckler algorithm
    - [ ] Test against major/minor templates
    - [ ] Return best match with confidence
  - [ ] Pure function: `pub fn detect_key(midi_data: &MidiData) -> Result<KeySignature>`
  - [ ] Return format: "C major", "A minor", etc.

### 6.4 Drum Analyzer

- [ ] **`app/src-tauri/src/core/analysis/drum_analyzer.rs`**
  - [ ] Detect drum channel (usually channel 10, but check notes)
  - [ ] Map MIDI note numbers to drum types:
    - [ ] 35-36: Kick
    - [ ] 38, 40: Snare
    - [ ] 42, 44, 46: Hi-hat
    - [ ] 49, 51, 53, 55, 57, 59: Cymbals
    - [ ] 41, 43, 45, 47, 48, 50: Toms
  - [ ] Count hits per drum type
  - [ ] Identify patterns (e.g., four-on-the-floor, breakbeat)
  - [ ] Pure function: `pub fn analyze_drums(midi_data: &MidiData) -> Result<DrumPattern>`
  - [ ] Return: `{ kick_hits, snare_hits, hihat_hits, has_drums, pattern_type }`

### 6.5 Chord Detector

- [ ] **`app/src-tauri/src/core/analysis/chord_detector.rs`**
  - [ ] Divide MIDI into time windows (e.g., 1 beat)
  - [ ] Aggregate notes in each window
  - [ ] Identify chord types:
    - [ ] Major: root, M3, P5
    - [ ] Minor: root, m3, P5
    - [ ] Diminished: root, m3, d5
    - [ ] Augmented: root, M3, A5
    - [ ] Dominant 7th: root, M3, P5, m7
    - [ ] Major 7th: root, M3, P5, M7
    - [ ] Minor 7th: root, m3, P5, m7
  - [ ] Score confidence based on note alignment
  - [ ] Pure function: `pub fn detect_chords(midi_data: &MidiData) -> Result<Vec<Chord>>`
  - [ ] Return: `{ tick, chord_name, root, chord_type, confidence }`

### 6.6 Auto Tagger

- [ ] **`app/src-tauri/src/core/analysis/auto_tagger.rs`**
  - [ ] 97 instrument categories (see tag list)
  - [ ] Track name regex patterns:
    - [ ] "piano" → Piano tag
    - [ ] "bass" → Bass tag
    - [ ] "drums", "perc" → Drums tag
    - [ ] "guitar", "gtr" → Guitar tag
    - [ ] etc.
  - [ ] MIDI program change detection (General MIDI mapping)
  - [ ] Return applicable tag IDs
  - [ ] Pure function: `pub fn auto_tag(midi_data: &MidiData) -> Result<Vec<i32>>`

### 6.7 Structure Analyzer (Optional)

- [ ] **`app/src-tauri/src/core/analysis/structure_analyzer.rs`**
  - [ ] Detect sections: intro, verse, chorus, bridge, outro
  - [ ] Use repetition detection (self-similarity matrix)
  - [ ] Identify section boundaries
  - [ ] Return JSON with section markers

---

## 7. MIDI Hardware Integration

### 7.1 Backend Selection System

- [ ] **`app/src-tauri/src/hardware/midi_backend.rs`**
  - [ ] Auto-detect platform (Linux/macOS/Windows)
  - [ ] Try backends in priority order:
    1. [ ] JACK (Linux/macOS) - ~3ms latency
    2. [ ] ALSA Raw (Linux) - ~5ms latency
    3. [ ] CoreMIDI (macOS) - ~5ms latency
    4. [ ] midir (all platforms) - ~10-15ms latency
  - [ ] Fall back if preferred backend unavailable
  - [ ] Return selected backend info

### 7.2 JACK Backend

- [ ] **`app/src-tauri/src/hardware/backends/jack.rs`**
  - [ ] Requires `jack` crate
  - [ ] Cargo feature: `jack`
  - [ ] Connect to JACK server
  - [ ] Register MIDI input/output ports
  - [ ] Lock-free ringbuffer for real-time thread communication
  - [ ] Process callback for MIDI events
  - [ ] Target latency: ~3ms

### 7.3 ALSA Backend

- [ ] **`app/src-tauri/src/hardware/backends/alsa.rs`**
  - [ ] Requires `alsa` crate
  - [ ] Cargo feature: `alsa`
  - [ ] Direct kernel access (no sound server)
  - [ ] Enumerate ALSA MIDI devices
  - [ ] Send/receive raw MIDI messages
  - [ ] Target latency: ~5ms

### 7.4 CoreMIDI Backend

- [ ] **`app/src-tauri/src/hardware/backends/coremidi.rs`**
  - [ ] Requires `coremidi` crate
  - [ ] macOS only (automatic on macOS)
  - [ ] Connect to CoreMIDI framework
  - [ ] Enumerate MIDI devices
  - [ ] Virtual MIDI ports (optional)
  - [ ] Target latency: ~5ms

### 7.5 midir Backend (Fallback)

- [ ] **`app/src-tauri/src/hardware/backends/midir.rs`**
  - [ ] Requires `midir` crate (always included)
  - [ ] Cross-platform guaranteed fallback
  - [ ] Enumerate MIDI devices
  - [ ] Send/receive MIDI messages
  - [ ] Higher latency but always works: ~10-15ms

### 7.6 Device Manager

- [ ] **`app/src-tauri/src/midi_io/device_manager.rs`**
  - [ ] `MidiDeviceManager` struct
  - [ ] Enumerate all available devices (input + output)
  - [ ] Connect to selected device
  - [ ] Disconnect gracefully
  - [ ] Hot-plug detection (device added/removed)
  - [ ] Send events to frontend on device changes

---

## 8. Frontend Components

### 8.1 Svelte App Structure

- [ ] **`app/src/` directory structure**
  - [ ] `App.svelte` - Root component
  - [ ] `lib/components/` - Reusable components
  - [ ] `lib/stores/` - Svelte stores (state management)
  - [ ] `lib/api/` - TypeScript API wrappers
  - [ ] `lib/types/` - TypeScript type definitions

### 8.2 Pipeline UI Components

- [ ] **`lib/components/Pipeline/`**
  - [ ] `ImportDialog.svelte` - File/folder selection for import
  - [ ] `ImportProgress.svelte` - Real-time import progress (files/sec, ETA)
  - [ ] `FileList.svelte` - Display imported files with metadata
  - [ ] `AnalysisPanel.svelte` - Show BPM, key, drums, chords
  - [ ] `SplitControls.svelte` - Multi-track split UI
  - [ ] `TagManager.svelte` - Bulk tagging interface

### 8.3 DAW UI Components

- [ ] **`lib/components/DAW/`**
  - [ ] `Sequencer.svelte` - Main sequencer view
    - [ ] Timeline with bar/beat grid
    - [ ] Track lanes with MIDI events
    - [ ] Playhead indicator
    - [ ] Zoom controls (horizontal/vertical)
  - [ ] `TransportControls.svelte` - Play/pause/stop/record buttons
  - [ ] `TempoControl.svelte` - BPM input
  - [ ] `TrackList.svelte` - List of tracks with controls
  - [ ] `TrackHeader.svelte` - Track name, mute, solo, record arm
  - [ ] `MixerPanel.svelte` - Mixer view
    - [ ] Channel strips (gain, pan, mute, solo)
    - [ ] VU meters (real-time level monitoring)
    - [ ] Master fader
  - [ ] `EffectRack.svelte` - Effect chain editor
  - [ ] `AutomationLane.svelte` - Automation curve editor
  - [ ] `MidiDeviceSelector.svelte` - MIDI device dropdown
  - [ ] `ProjectManager.svelte` - Open/save/close project UI

### 8.4 VIP3 Browser Components

- [ ] **`lib/components/VIP3/`**
  - [ ] `VIP3Browser.svelte` - Main browser view
    - [ ] 8 filter columns (folder, instrument, timbre, style, articulation, BPM, key, channel)
    - [ ] Results grid with file previews
    - [ ] Pagination controls
  - [ ] `VIP3Column.svelte` - Reusable filter column
    - [ ] Checkbox groups for multi-select
    - [ ] Result counts next to each option
    - [ ] "Select All" / "Clear" buttons
  - [ ] `VIP3Results.svelte` - Search results grid
    - [ ] File cards with metadata (BPM, key, tags)
    - [ ] Double-click to load in DAW
    - [ ] Drag-and-drop (future feature)
    - [ ] Favorite toggle
  - [ ] `VIP3Favorites.svelte` - Favorites tab
  - [ ] `VIP3SavedSearches.svelte` - Saved searches tab
  - [ ] `VIP3Collections.svelte` - Collections manager

### 8.5 Svelte Stores

- [ ] **`lib/stores/vip3Store.ts`**
  - [ ] `currentFilters: Writable<Vip3Filters>`
  - [ ] `searchResults: Writable<MidiFile[]>`
  - [ ] `filterCounts: Writable<FilterCounts>`
  - [ ] `pagination: Writable<{ limit, offset, total }>`
  - [ ] `categoryData: Writable<{ timbres, styles, articulations }>`
  - [ ] Actions: `updateFilter()`, `clearFilters()`, `loadSearch()`

- [ ] **`lib/stores/dawStore.ts`**
  - [ ] `tracks: Writable<Track[]>`
  - [ ] `playbackState: Writable<'stopped' | 'playing' | 'paused'>`
  - [ ] `playbackPosition: Writable<Position>`
  - [ ] `tempo: Writable<number>`
  - [ ] `selectedTrack: Writable<number | null>`
  - [ ] `midiDevice: Writable<MidiDevice | null>`

- [ ] **`lib/stores/pipelineStore.ts`**
  - [ ] `importProgress: Writable<ImportProgress>`
  - [ ] `files: Writable<MidiFile[]>`
  - [ ] `selectedFiles: Writable<Set<number>>`

### 8.6 TypeScript API Layer

- [ ] **`lib/api.ts` - Core API wrapper**
  - [ ] `invoke<T>(command: string, args?: any): Promise<T>`
  - [ ] Handle Tauri 2.x timing (`__TAURI_INTERNALS__` availability)
  - [ ] Log command invocations for debugging
  - [ ] Type-safe generic wrapper

- [ ] **`lib/api/pipelineApi.ts`**
  - [ ] `importFiles(paths: string[]): Promise<ImportStats>`
  - [ ] `splitFile(fileId: number): Promise<number[]>`
  - [ ] `analyzeBatch(fileIds: number[]): Promise<AnalysisStats>`
  - [ ] `bulkRetag(mappings: TagMapping[]): Promise<RetagStats>`

- [ ] **`lib/api/dawApi.ts`**
  - [ ] `startSequencer(): Promise<void>`
  - [ ] `pauseSequencer(): Promise<void>`
  - [ ] `setTempo(bpm: number): Promise<void>`
  - [ ] `addTrack(track: Track): Promise<number>`
  - [ ] `midiListDevices(): Promise<MidiDevice[]>`
  - [ ] `midiConnect(deviceName: string): Promise<void>`

- [ ] **`lib/api/vip3BrowserApi.ts`**
  - [ ] `searchFilesVip3(filters: Vip3Filters): Promise<SearchResults>`
  - [ ] `getVip3FilterCounts(filters: Vip3Filters): Promise<FilterCounts>`
  - [ ] `toggleFavorite(fileId: number): Promise<boolean>`
  - [ ] `saveSearch(name: string, filters: Vip3Filters): Promise<number>`
  - [ ] `createCollection(name: string, description: string): Promise<number>`

### 8.7 TypeScript Type Definitions

- [ ] **`lib/types/index.ts`**
  - [ ] `interface MidiFile { id, file_path, content_hash, bpm, key_signature, ... }`
  - [ ] `interface Vip3Filters { folder_ids?, instrument_ids?, timbre_ids?, ... }`
  - [ ] `interface SearchResults { files: MidiFile[], total: number }`
  - [ ] `interface FilterCounts { folder_counts, instrument_counts, ... }`
  - [ ] `interface Track { id, name, events, channel, ... }`
  - [ ] `interface Position { bar, beat, tick, seconds }`
  - [ ] `interface MidiDevice { name, id, is_input, is_output }`
  - [ ] `interface ImportStats { total_files, successful, failed, ... }`
  - [ ] All types match Rust structs exactly (use `serde` serialization)

---

## 9. API Layer

### 9.1 Tauri Command Registration

- [ ] **`app/src-tauri/src/main.rs`**
  - [ ] Register all Tauri commands in `tauri::Builder`
  - [ ] Initialize global state: `AppState`, `Arc<SequencerEngine>`
  - [ ] Setup event listeners
  - [ ] Configure window settings

### 9.2 AppState

- [ ] **`app/src-tauri/src/state.rs`**
  - [ ] `AppState` struct with:
    - [ ] `db_pool: PgPool` - Database connection pool
    - [ ] `sequencer: Arc<SequencerEngine>` - Sequencer state
    - [ ] `midi_device_manager: Arc<Mutex<MidiDeviceManager>>`
    - [ ] `config: Arc<Config>` - App configuration
  - [ ] Thread-safe with `Arc<Mutex<>>` where needed

### 9.3 Event System

- [ ] **Tauri events** (backend → frontend)
  - [ ] `import_progress` - Import progress updates
  - [ ] `analysis_progress` - Analysis progress updates
  - [ ] `playback_position` - Playback position (bar/beat/tick)
  - [ ] `midi_device_added` - Hot-plug device detection
  - [ ] `midi_device_removed` - Device disconnection
  - [ ] `error` - Error notifications

---

## 10. Testing Infrastructure

### 10.1 Test Organization

- [ ] **Test files** (co-located with source)
  - [ ] `app/src-tauri/src/db/repositories/file_repository.rs` has `#[cfg(test)] mod tests { ... }`
  - [ ] All modules have test modules
  - [ ] Integration tests in `app/src-tauri/tests/`

### 10.2 Database Tests

- [ ] **Test database setup**
  - [ ] Use `DATABASE_URL` env var for test DB
  - [ ] Run migrations before tests: `sqlx migrate run`
  - [ ] Clean database between tests (truncate tables or use transactions)
  - [ ] **CRITICAL:** Run with `--test-threads=1` to avoid race conditions

- [ ] **Test coverage**
  - [ ] All repository methods tested
  - [ ] Deduplication tested (insert same file twice)
  - [ ] Search filters tested (all combinations)
  - [ ] Pagination tested
  - [ ] Error cases tested (invalid IDs, constraint violations)

### 10.3 MIDI Analysis Tests

- [ ] **Test fixtures**
  - [ ] Sample MIDI files in `scripts/test-midi-files/`
  - [ ] Cover all formats (0/1/2)
  - [ ] Include edge cases (corrupted files, empty tracks, etc.)

- [ ] **Analysis tests**
  - [ ] BPM detection accuracy (known BPM files)
  - [ ] Key detection accuracy (known key files)
  - [ ] Drum detection (files with/without drums)
  - [ ] Chord detection (simple chord progressions)
  - [ ] Performance benchmarks (files/sec)

### 10.4 Sequencer Tests

- [ ] **Playback tests**
  - [ ] Start/stop/pause state transitions
  - [ ] Position tracking (bar/beat/tick)
  - [ ] Tempo changes
  - [ ] Track add/remove during playback

- [ ] **Mock MIDI devices**
  - [ ] Use `#[cfg(test)]` to avoid real hardware
  - [ ] Mock device manager for testing

### 10.5 Integration Tests

- [ ] **End-to-end workflows**
  - [ ] Import → analyze → search → load to DAW
  - [ ] Create project → add tracks → save → reload
  - [ ] VIP3 search with multiple filters → add to collection

### 10.6 Coverage Tools

- [ ] **Tarpaulin** for code coverage
  - [ ] `cargo tarpaulin --workspace --out Html`
  - [ ] Target: >80% coverage
  - [ ] Exclude test code from coverage

### 10.7 Verification Suite

- [ ] **`verification/src/main.rs`**
  - [ ] Health checks for all subsystems
  - [ ] Schema validation (15 tables, 60+ indexes)
  - [ ] Performance benchmarks
  - [ ] Smoke tests for critical paths
  - [ ] Run as part of CI/CD

---

## 11. DevOps & Environment

### 11.1 Docker Compose

- [ ] **`docker-compose.yml`**
  - [ ] PostgreSQL 16 service:
    - [ ] Port: 5433:5432 (non-standard to avoid conflicts)
    - [ ] Volume: `postgres_data`
    - [ ] Environment: `POSTGRES_USER=midiuser`, `POSTGRES_PASSWORD=145278963`, `POSTGRES_DB=midi_library`
    - [ ] Extensions: `pgvector`, `pg_trgm`
  - [ ] Meilisearch service:
    - [ ] Port: 7700
    - [ ] Volume: `meilisearch_data`
  - [ ] Redis service (optional, for caching):
    - [ ] Port: 6379
    - [ ] Volume: `redis_data`

- [ ] **Docker commands**
  - [ ] `make docker-up` → `docker-compose up -d`
  - [ ] `make docker-down` → `docker-compose down`
  - [ ] `make docker-logs` → `docker-compose logs -f`
  - [ ] `make docker-reset` → `docker-compose down -v` (destroys data)

### 11.2 Environment Variables

- [ ] **`.env` file** (not committed to git)
  - [ ] `DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library`
  - [ ] `MEILISEARCH_URL=http://localhost:7700`
  - [ ] `RUST_LOG=info,midi_app=debug`
  - [ ] `MIDI_LIBRARY_PATH=/path/to/midi-library/`

- [ ] **Environment setup script**
  - [ ] `scripts/setup-env.sh` - Generate `.env` from template

### 11.3 Database Migrations

- [ ] **sqlx-cli**
  - [ ] Install: `cargo install sqlx-cli --no-default-features --features postgres`
  - [ ] Create migration: `sqlx migrate add <name>`
  - [ ] Run migrations: `sqlx migrate run`
  - [ ] Revert migration: `sqlx migrate revert`

- [ ] **Migration workflow**
  - [ ] All migrations in `database/migrations/`
  - [ ] Sequential numbering: `001_*.sql`, `002_*.sql`, etc.
  - [ ] **Never edit existing migrations**
  - [ ] Always test migrations on clean database

### 11.4 CI/CD Pipeline (Optional)

- [ ] **GitHub Actions** (`.github/workflows/ci.yml`)
  - [ ] Run on push to main, PRs
  - [ ] Steps:
    - [ ] Checkout code
    - [ ] Setup Rust toolchain
    - [ ] Start PostgreSQL service
    - [ ] Run migrations
    - [ ] Run tests with `--test-threads=1`
    - [ ] Run clippy (linter)
    - [ ] Run format check
    - [ ] Upload coverage report
  - [ ] Cache dependencies for speed

### 11.5 Deployment

- [ ] **Release builds**
  - [ ] `cargo build --release -p midi-software-center`
  - [ ] Binary location: `target/release/midi-software-center`
  - [ ] Bundle with Tauri: `cd app && npm run tauri build`

- [ ] **Platform-specific builds**
  - [ ] Linux: AppImage, .deb, .rpm
  - [ ] macOS: .dmg, .app bundle
  - [ ] Windows: .msi installer

---

## 12. Missing Features (Future)

These features are mentioned in the copilot instructions but **not yet implemented**:

### 12.1 Lua Scripting Runtime

- [ ] **Status:** Dependency ready (`mlua` with vendored Lua 5.4), runtime not implemented
- [ ] **Implementation plan:**
  - [ ] Create `app/src-tauri/src/scripting/` module
  - [ ] `lua_runtime.rs` - Initialize Lua VM
  - [ ] Expose Rust functions to Lua:
    - [ ] File operations (search, tag, analyze)
    - [ ] DAW control (add track, set tempo, etc.)
    - [ ] Automation tasks
  - [ ] Script loading: `~/.midi-software-center/scripts/`
  - [ ] Script execution commands:
    - [ ] `run_lua_script(script_path: String) -> Result<String, String>`
    - [ ] `eval_lua(code: String) -> Result<String, String>`
  - [ ] Example scripts:
    - [ ] Auto-organize by BPM
    - [ ] Batch file renaming
    - [ ] Custom import workflows

### 12.2 Meilisearch Integration

- [ ] **Status:** Configured in Docker Compose, not integrated in code
- [ ] **Implementation plan:**
  - [ ] Create `app/src-tauri/src/search/meilisearch.rs`
  - [ ] Initialize Meilisearch client
  - [ ] Index MIDI files on import:
    - [ ] Document structure: `{ id, file_path, tags, bpm, key, ... }`
    - [ ] Update index on file updates
  - [ ] Search commands:
    - [ ] `search_meilisearch(query: String, filters: SearchFilters) -> Result<Vec<MidiFile>, String>`
    - [ ] Faceted search (filter by BPM range, key, tags)
  - [ ] Replace PostgreSQL ILIKE search with Meilisearch for full-text
  - [ ] Keep PostgreSQL for structured queries

### 12.3 VIP3 ↔ DAW Drag-and-Drop

- [ ] **Status:** Planned but not implemented
- [ ] **Implementation plan:**
  - [ ] Frontend: `VIP3Results.svelte`
    - [ ] Make file cards draggable (HTML5 drag API)
    - [ ] Drag data: `{ file_id, file_path }`
  - [ ] Frontend: `Sequencer.svelte`
    - [ ] Accept drop events
    - [ ] Call `load_file_to_daw(file_id)` on drop
  - [ ] Backend: Convenience wrapper (see below)

### 12.4 Convenience Wrapper: load_file_to_daw

- [ ] **Status:** Suggested in copilot instructions, not implemented
- [ ] **Implementation plan:**
  - [ ] Create `app/src-tauri/src/commands/daw/file_loader.rs`
  - [ ] `load_file_to_daw(file_id: i64) -> Result<u32, String>`
    - [ ] Fetch file path from database: `FileRepository::get_by_id()`
    - [ ] Parse MIDI: `daw_load_midi_file(file_path)`
    - [ ] Add to sequencer: `add_track(track)`
    - [ ] Return track ID
  - [ ] Simplifies frontend code (single command instead of 3-step process)

---

## 13. Verification & Quality Assurance

### 13.1 Feature Completeness Checklist

For each implemented feature, verify:

- [ ] **Database table exists** (check migrations)
- [ ] **Repository methods implemented** (CRUD operations)
- [ ] **Tauri commands registered** (in `main.rs`)
- [ ] **Frontend API wrapper exists** (in `lib/api/`)
- [ ] **UI component exists** (in `lib/components/`)
- [ ] **Tests written** (unit + integration)
- [ ] **Documentation updated** (copilot-instructions.md, ARCHITECTURE_REFERENCE.md)

### 13.2 Performance Benchmarks

Verify performance targets:

- [ ] **Import:** 7,830 files/sec (use `./scripts/run-pipeline-ultra-fast.sh`)
- [ ] **Analysis:** 181-360 files/sec (BPM/key detection)
- [ ] **Deduplication:** 73.4% duplicate rate (use real-world dataset)
- [ ] **Simple queries:** <10ms (e.g., `get_file_by_id`)
- [ ] **Complex queries:** <100ms (e.g., VIP3 multi-filter search)
- [ ] **Filter counts:** <50ms (VIP3 real-time count updates)

### 13.3 Verification Suite

Run `verification/src/main.rs` to check:

- [ ] All 15 database tables exist
- [ ] All 60+ indexes exist
- [ ] Sample queries execute successfully
- [ ] Import workflow (end-to-end)
- [ ] Analysis workflow (end-to-end)
- [ ] DAW workflow (load file, play, stop)

### 13.4 Manual Testing Checklist

#### Pipeline
- [ ] Import folder with 1000+ MIDI files
- [ ] Verify deduplication (import same files twice)
- [ ] Run analysis batch (BPM, key, drums)
- [ ] Split multi-track files
- [ ] Bulk retag files

#### DAW
- [ ] Start sequencer with MIDI file loaded
- [ ] Pause/resume playback
- [ ] Change tempo during playback
- [ ] Connect to MIDI device
- [ ] Send test note to device
- [ ] Adjust mixer (gain, pan, mute, solo)
- [ ] Add automation to track
- [ ] Save/load project

#### VIP3 Browser
- [ ] Apply 8 simultaneous filters
- [ ] Verify filter counts update in real-time
- [ ] Toggle favorite
- [ ] Save search
- [ ] Load saved search
- [ ] Create collection
- [ ] Add files to collection
- [ ] Reorder collection

### 13.5 Code Quality

- [ ] **Clippy:** No warnings (`make lint`)
- [ ] **Format:** All code formatted (`make format`)
- [ ] **Coverage:** >80% test coverage (`cargo tarpaulin`)
- [ ] **Documentation:** All public functions documented
- [ ] **Error handling:** No unwrap() in production code (use `?` and `Result`)

---

## Implementation Order (Suggested)

1. **Phase 1: Foundation** (1-2 weeks)
   - Database schema (15 tables, migrations)
   - Repository pattern (all CRUD operations)
   - Core Rust modules structure
   - Error handling framework

2. **Phase 2: MIDI Analysis** (1 week)
   - MIDI parser (`analysis_parser.rs`)
   - BPM detector
   - Key detector
   - Drum analyzer
   - Auto-tagger

3. **Phase 3: Pipeline System** (1 week)
   - Import phase (file discovery, hashing, deduplication)
   - Split phase (multi-track separation)
   - Analyze phase (batch analysis)
   - Tauri commands for pipeline
   - Frontend UI for import/analysis

4. **Phase 4: VIP3 Browser** (1-2 weeks)
   - Dynamic search with filters
   - Filter count system
   - Favorites, saved searches, collections
   - VIP3 category tables (timbres, styles, articulations)
   - Frontend VIP3Browser components

5. **Phase 5: DAW Sequencer** (2 weeks)
   - Sequencer engine (playback, tempo, tracks)
   - MIDI hardware backends (JACK, ALSA, CoreMIDI, midir)
   - Sequencer commands (start, pause, seek, etc.)
   - MIDI I/O commands
   - Frontend DAW UI (transport, track list)

6. **Phase 6: DAW Mixer & Effects** (1 week)
   - Mixer system (gain, pan, mute, solo, VU meters)
   - Effect chain management
   - Mixer commands (30+ commands)
   - Frontend mixer UI

7. **Phase 7: DAW Advanced** (1 week)
   - Automation system
   - Preset system
   - Project management
   - MIDI repair utilities

8. **Phase 8: Integration** (1 week)
   - VIP3 ↔ DAW integration (double-click to load)
   - Drag-and-drop (future feature)
   - Convenience wrapper (`load_file_to_daw`)

9. **Phase 9: Testing & Verification** (1 week)
   - Comprehensive test suite (1,999 tests target)
   - Performance benchmarking
   - Verification suite
   - Code coverage >80%

10. **Phase 10: Future Features** (optional)
    - Lua scripting runtime
    - Meilisearch integration
    - Additional UI polish

**Total estimated time:** 10-13 weeks (for experienced Rust/Tauri developers)

---

## Success Criteria

Implementation is complete when:

- [ ] All 15 database tables exist with 60+ indexes
- [ ] All repository methods implemented and tested
- [ ] All 85+ Tauri commands implemented (60+ DAW, 25+ Pipeline/VIP3)
- [ ] Frontend has all UI components (Pipeline, DAW, VIP3)
- [ ] Performance targets met (7,830 import/sec, 181-360 analysis/sec, <100ms queries)
- [ ] 1,999+ tests passing with >80% coverage
- [ ] Verification suite passes all checks
- [ ] Documentation complete and accurate
- [ ] Production-ready builds for Linux/macOS/Windows

---

**This is a comprehensive, production-ready implementation plan.** Follow this checklist step-by-step to build the complete MIDI Software Center as described in the copilot instructions.
