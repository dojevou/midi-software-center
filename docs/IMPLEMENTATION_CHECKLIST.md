# Implementation Checklist - MIDI Software Center

**Generated from:** `.github/copilot-instructions.md`
**Purpose:** Comprehensive task list for implementing/verifying all documented patterns and features

---

## Phase 1: Foundation & Environment Setup

### 1.1 Docker Services & Infrastructure
- [ ] Configure PostgreSQL 16+ container on port 5433
- [ ] Configure Meilisearch container on port 7700
- [ ] Configure Redis container on port 6379
- [ ] Set up Docker Compose volume management with backup strategy
- [ ] Create `.env` file with `DATABASE_URL`, `MEILISEARCH_URL`, `RUST_LOG`
- [ ] Verify pgvector extension is available in PostgreSQL 16

### 1.2 Project Workspace Structure
- [ ] Create `app/src-tauri/` as main unified application directory
- [ ] Create `app/src-tauri/src/core/` for MIDI analysis modules
- [ ] Create `app/src-tauri/src/db/` for database layer
- [ ] Create `app/src-tauri/src/commands/` for Tauri IPC handlers
- [ ] Create `app/src-tauri/src/commands/pipeline/` for batch commands
- [ ] Create `app/src-tauri/src/commands/daw/` for real-time commands
- [ ] Create `app/src-tauri/src/hardware/` for MIDI I/O backends
- [ ] Create `app/src-tauri/src/sequencer/` for real-time sequencing
- [ ] Create `app/src/` for Svelte frontend
- [ ] Create `database/migrations/` for schema migrations
- [ ] Create `verification/` for test suite
- [ ] Create `scripts/` for import tools and utilities

### 1.3 Build System & Makefile
- [ ] Implement `make setup` - Install sqlx-cli, tarpaulin, dependencies
- [ ] Implement `make dev` - Start Svelte dev server on :5173
- [ ] Implement `make check` - Format, lint, test (pre-commit validation)
- [ ] Implement `make docker-up` - Start all Docker services
- [ ] Implement `make docker-down` - Stop Docker services
- [ ] Implement `make db-migrate` - Apply pending migrations (idempotent)
- [ ] Implement `make db-reset` - Wipe database and rerun migrations (with warning)
- [ ] Implement `make lint` - Run clippy + format check
- [ ] Implement `make format` - Format all code
- [ ] Implement `make test` - Run all tests with `--test-threads=1`

### 1.4 Cargo Configuration
- [ ] Configure workspace members in root `Cargo.toml`
- [ ] Add `midi-software-center` as main app package
- [ ] Add feature flags: `jack`, `alsa`, `default = ["midir"]`
- [ ] Configure dependencies:
  - `tauri = "2.0"` with required features
  - `sqlx` with PostgreSQL, runtime, macros
  - `tokio` with full features
  - `midly` for MIDI parsing
  - `serde`, `serde_json` for serialization
  - `thiserror` for error handling
  - `mlua` with vendored Lua 5.4 (future scripting)
- [ ] Configure dev dependencies: `tarpaulin`, test utilities

---

## Phase 2: Database Layer Implementation

### 2.1 Migration Files (Sequential, Never Edit)
- [ ] Create `001_initial_schema.sql` - Core tables
  - [ ] `files` table (id, content_hash, path, size, format, created_at)
  - [ ] `musical_metadata` table (file_id FK, bpm, key, time_sig, duration_ticks)
  - [ ] `tags` table (id, name, category)
  - [ ] `file_tags` junction table (file_id, tag_id)
  - [ ] `midi_tracks` table (id, file_id, track_number, name, channel)
  - [ ] `midi_events` table (id, track_id, tick, event_type, data)
  - [ ] `analysis_results` table (file_id, analysis_json JSONB)
  - [ ] `drum_patterns` table (file_id, pattern_data JSONB)
  - [ ] `chords` table (file_id, chord_progression JSONB)
- [ ] Create `002_indexes.sql` - 60+ performance indexes
  - [ ] Index on `files.content_hash` (UNIQUE for deduplication)
  - [ ] Index on `musical_metadata.bpm` (range queries)
  - [ ] Index on `musical_metadata.key` (filtering)
  - [ ] Index on `file_tags.tag_id` and `file_tags.file_id`
  - [ ] GIN index on `analysis_results.analysis_json`
  - [ ] Index on `midi_events.tick` (sorted access)
- [ ] Create `003_functions.sql` - Optimized query functions
  - [ ] `get_files_by_instrument(text)` function
  - [ ] `get_files_by_bpm_range(float, float)` function
  - [ ] `get_files_by_instruments(text[])` function
  - [ ] `get_files_by_key(text)` function
- [ ] Create `004_pgvector.sql` - Vector similarity search
  - [ ] Enable pgvector extension
  - [ ] Add embedding columns to appropriate tables
  - [ ] Create vector similarity indexes
- [ ] Create migration runner script that enforces sequential order

### 2.2 Database Models (`app/src-tauri/src/db/models/`)
- [ ] Create `mod.rs` - Export all models
- [ ] Create `file.rs`:
  - [ ] `MidiFile` struct (maps to `files` table)
  - [ ] `CreateMidiFile` struct (for inserts)
  - [ ] Derive: `Debug, Clone, Serialize, Deserialize, sqlx::FromRow`
- [ ] Create `musical_metadata.rs`:
  - [ ] `MusicalMetadata` struct (bpm, key, time_signature, duration)
  - [ ] Optional fields for nullable columns
- [ ] Create `tag.rs`:
  - [ ] `Tag` struct (id, name, category)
  - [ ] `FileTag` junction struct
- [ ] Create `midi_track.rs`:
  - [ ] `MidiTrack` struct
  - [ ] `MidiEvent` struct with event_type enum
- [ ] Create `analysis.rs`:
  - [ ] `AnalysisResult` with JSONB field
  - [ ] `DrumPattern` struct
  - [ ] `ChordProgression` struct

### 2.3 Database Repositories (`app/src-tauri/src/db/repositories/`)
- [ ] Create `mod.rs` - Export all repositories
- [ ] Create `file_repository.rs`:
  - [ ] `FileRepository` struct with `PgPool`
  - [ ] `insert(&self, file: CreateMidiFile) -> DbResult<i64>` (with deduplication)
  - [ ] `get_by_id(&self, id: i64) -> DbResult<Option<MidiFile>>`
  - [ ] `get_by_hash(&self, hash: &str) -> DbResult<Option<MidiFile>>`
  - [ ] `search(&self, query: &str, filters: SearchFilters) -> DbResult<Vec<MidiFile>>`
  - [ ] `batch_insert(&self, files: Vec<CreateMidiFile>) -> DbResult<Vec<i64>>` (use COPY)
  - [ ] `delete(&self, id: i64) -> DbResult<()>`
- [ ] Create `tag_repository.rs`:
  - [ ] `TagRepository` struct
  - [ ] `get_or_create(&self, name: &str, category: &str) -> DbResult<i64>`
  - [ ] `add_tag_to_file(&self, file_id: i64, tag_id: i64) -> DbResult<()>`
  - [ ] `get_tags_for_file(&self, file_id: i64) -> DbResult<Vec<Tag>>`
  - [ ] `get_files_by_tag(&self, tag_name: &str) -> DbResult<Vec<i64>>`
- [ ] Create `metadata_repository.rs`:
  - [ ] Functions for CRUD on `musical_metadata`
- [ ] Create `track_repository.rs`:
  - [ ] Functions for CRUD on `midi_tracks` and `midi_events`
- [ ] Create error handling:
  - [ ] `DbError` enum with `thiserror`
  - [ ] `DbResult<T>` type alias
  - [ ] Error variants: `QueryFailed`, `NotFound`, `DuplicateHash`

### 2.4 Connection Pool Management
- [ ] Create `app/src-tauri/src/db/pool.rs`:
  - [ ] `create_pool(database_url: &str) -> Result<PgPool>`
  - [ ] Connection pooling configuration (min/max connections)
  - [ ] Handle `--test-threads=1` requirement in tests
- [ ] Create connection health check function
- [ ] Implement graceful shutdown for connection pool

---

## Phase 3: MIDI Analysis Core

### 3.1 MIDI Parser (`app/src-tauri/src/core/midi/`)
- [ ] Create `analysis_parser.rs`:
  - [ ] `parse_midi_file(data: &[u8]) -> Result<ParsedMidi>`
  - [ ] Use `midly` crate for parsing
  - [ ] Extract tracks, events, tempo changes, time signatures
  - [ ] Calculate total duration in ticks and seconds
  - [ ] Pure function (no I/O, no database calls)
  - [ ] Comprehensive unit tests with sample MIDI files

### 3.2 BPM Detection (`app/src-tauri/src/core/analysis/`)
- [ ] Create `bpm_detector.rs`:
  - [ ] `analyze_bpm(midi_data: &[u8]) -> Result<f64>`
  - [ ] Detect tempo changes across file
  - [ ] Return weighted average BPM or dominant tempo
  - [ ] Handle variable tempo MIDI files
  - [ ] Pure function with unit tests
  - [ ] Performance target: 181-360 files/sec

### 3.3 Key Detection
- [ ] Create `key_detector.rs`:
  - [ ] `analyze_key(midi_data: &[u8]) -> Result<String>`
  - [ ] Implement Krumhansl-Schmuckler algorithm
  - [ ] Return key signature (e.g., "C major", "A minor")
  - [ ] Handle modal ambiguity
  - [ ] Pure function with unit tests

### 3.4 Drum Pattern Analysis
- [ ] Create `drum_analyzer.rs`:
  - [ ] `analyze_drums(midi_data: &[u8]) -> Result<DrumPattern>`
  - [ ] Detect drum tracks (channel 10 / GM drum map)
  - [ ] Extract kick, snare, hi-hat patterns
  - [ ] Identify rhythmic patterns (4/4, swing, etc.)
  - [ ] Return structured pattern data
  - [ ] Pure function with unit tests

### 3.5 Auto-Tagger
- [ ] Create `auto_tagger.rs`:
  - [ ] `auto_tag(midi_data: &[u8]) -> Result<Vec<String>>`
  - [ ] Implement 97 instrument detection rules
  - [ ] Map MIDI program changes to instrument categories
  - [ ] Detect genre markers (jazz, classical, rock, etc.)
  - [ ] Return list of applicable tags
  - [ ] Pure function with unit tests

### 3.6 Chord Detection
- [ ] Create `chord_detector.rs`:
  - [ ] `analyze_chords(midi_data: &[u8]) -> Result<Vec<Chord>>`
  - [ ] Detect chord progressions from note events
  - [ ] Return chord sequence with timing
  - [ ] Handle polyphonic analysis
  - [ ] Pure function with unit tests

### 3.7 Performance Utilities
- [ ] Create `performance.rs`:
  - [ ] Batch processing helpers
  - [ ] Progress tracking utilities
  - [ ] Memory-efficient streaming for large files

---

## Phase 4: Tauri Commands (Backend IPC)

### 4.1 Pipeline Commands (`app/src-tauri/src/commands/pipeline/`)
- [ ] Create `mod.rs` - Export all pipeline commands
- [ ] Create `import.rs`:
  - [ ] `import_files(paths: Vec<String>, state: State<AppState>) -> Result<ImportStats, String>`
  - [ ] Batch process files (7,830/sec target)
  - [ ] Calculate content hash (SHA-256)
  - [ ] Parse MIDI with `analysis_parser`
  - [ ] Insert into database via `FileRepository`
  - [ ] Handle deduplication (return existing ID if duplicate hash)
  - [ ] Emit progress events: `emit('import_progress', { current, total })`
  - [ ] Return `ImportStats { successful, failed, duplicates, errors }`
  - [ ] Tolerates partial failures (continue on error)
- [ ] Create `analyze.rs`:
  - [ ] `analyze_files(file_ids: Vec<i64>, state: State<AppState>) -> Result<AnalysisStats, String>`
  - [ ] Run BPM, key, drum, chord analysis
  - [ ] Update `musical_metadata`, `analysis_results`, `drum_patterns`, `chords` tables
  - [ ] Batch operations for performance (181-360/sec target)
  - [ ] Emit progress events
  - [ ] Return stats with per-analyzer results
- [ ] Create `tag.rs`:
  - [ ] `auto_tag_files(file_ids: Vec<i64>, state: State<AppState>) -> Result<TagStats, String>`
  - [ ] Run auto-tagger on files
  - [ ] Bulk insert tags via `TagRepository`
  - [ ] Apply 97 instrument categories
  - [ ] Return tagging statistics
- [ ] Create `search.rs`:
  - [ ] `search_files(query: SearchQuery, state: State<AppState>) -> Result<Vec<MidiFile>, String>`
  - [ ] Support filters: BPM range, key, instruments, tags
  - [ ] Use `FileRepository::search()`
  - [ ] Return paginated results
- [ ] Create `split.rs`:
  - [ ] `split_multi_track(file_id: i64, state: State<AppState>) -> Result<Vec<i64>, String>`
  - [ ] Split multi-track MIDI into individual files
  - [ ] Auto-repair malformed MIDI during split
  - [ ] Insert new files with proper relationships
- [ ] Create `sanitize.rs`:
  - [ ] `sanitize_filenames(file_ids: Vec<i64>, state: State<AppState>) -> Result<SanitizeStats, String>`
  - [ ] Clean filenames (remove special chars, normalize)
  - [ ] Update database paths
- [ ] Create `rename.rs`:
  - [ ] `rename_by_metadata(file_ids: Vec<i64>, template: String, state: State<AppState>) -> Result<RenameStats, String>`
  - [ ] Rename files using metadata template
  - [ ] Template vars: `{bpm}`, `{key}`, `{instruments}`, etc.

### 4.2 DAW Commands (`app/src-tauri/src/commands/daw/`)
- [ ] Create `mod.rs` - Export all DAW commands
- [ ] Create `sequencer.rs`:
  - [ ] `start_sequencer(engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `stop_sequencer(engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `pause_sequencer(engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `set_tempo(bpm: f64, engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `seek(position: u64, engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] Fail-fast error handling (all-or-nothing)
  - [ ] Immediate feedback (no progress events)
- [ ] Create `tracks.rs`:
  - [ ] `add_track(file_id: i64, engine: State<Arc<SequencerEngine>>) -> Result<usize, String>`
  - [ ] `remove_track(track_idx: usize, engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `mute_track(track_idx: usize, muted: bool, engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
  - [ ] `solo_track(track_idx: usize, engine: State<Arc<SequencerEngine>>) -> Result<(), String>`
- [ ] Create `hardware.rs`:
  - [ ] `list_midi_devices(backend: State<Arc<MidiBackend>>) -> Result<Vec<MidiDevice>, String>`
  - [ ] `connect_device(device_id: String, backend: State<Arc<MidiBackend>>) -> Result<(), String>`
  - [ ] `disconnect_device(device_id: String, backend: State<Arc<MidiBackend>>) -> Result<(), String>`

### 4.3 Command Registration
- [ ] Register all commands in `app/src-tauri/src/main.rs`:
  ```rust
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![
          // Pipeline commands
          import_files, analyze_files, auto_tag_files, search_files,
          split_multi_track, sanitize_filenames, rename_by_metadata,
          // DAW commands
          start_sequencer, stop_sequencer, pause_sequencer, set_tempo, seek,
          add_track, remove_track, mute_track, solo_track,
          list_midi_devices, connect_device, disconnect_device,
      ])
  ```

---

## Phase 5: MIDI Hardware Backend

### 5.1 Backend Trait & Selection (`app/src-tauri/src/hardware/`)
- [ ] Create `midi_backend.rs`:
  - [ ] `MidiBackend` trait with:
    - [ ] `fn send(&self, message: &[u8]) -> Result<()>`
    - [ ] `fn receive(&self) -> Result<Option<Vec<u8>>>`
    - [ ] `fn list_devices(&self) -> Result<Vec<MidiDevice>>`
    - [ ] `fn connect(&mut self, device_id: &str) -> Result<()>`
    - [ ] `fn latency_ms(&self) -> f64`
  - [ ] `select_backend() -> Box<dyn MidiBackend>` (auto-detection)
  - [ ] Priority order: JACK → ALSA → CoreMIDI → midir

### 5.2 Platform-Specific Implementations
- [ ] Create `jack_backend.rs` (Linux/macOS, ~3ms latency):
  - [ ] Implement `MidiBackend` trait
  - [ ] Lock-free ringbuffers for real-time safety
  - [ ] Conditional compilation: `#[cfg(feature = "jack")]`
- [ ] Create `alsa_backend.rs` (Linux, ~5ms latency):
  - [ ] Implement `MidiBackend` trait
  - [ ] Direct kernel access via ALSA sequencer API
  - [ ] Conditional compilation: `#[cfg(feature = "alsa")]`
- [ ] Create `coremidi_backend.rs` (macOS, ~5ms latency):
  - [ ] Implement `MidiBackend` trait
  - [ ] Use CoreMIDI framework bindings
  - [ ] Conditional compilation: `#[cfg(target_os = "macos")]`
- [ ] Create `midir_backend.rs` (all platforms, ~10-15ms latency):
  - [ ] Implement `MidiBackend` trait
  - [ ] Cross-platform fallback (always available)
  - [ ] Default feature

### 5.3 Hardware Testing Strategy
- [ ] Create mock backend for tests: `#[cfg(test)]`
- [ ] Test variable buffer sizes (JACK fixed, midir variable)
- [ ] Integration tests with real devices (manual, not CI)

---

## Phase 6: Frontend Implementation

### 6.1 API Wrapper (`app/src/lib/api.ts`)
- [ ] Create type-safe `invoke<T>()` wrapper:
  ```typescript
  export async function invoke<T>(command: string, args?: object): Promise<T> {
    // Wait for Tauri 2.x __TAURI_INTERNALS__ availability
    // Log invocations for debugging
    // Call Tauri invoke with type safety
    // Handle errors
  }
  ```
- [ ] Create TypeScript interfaces matching Rust types:
  - [ ] `ImportStats`, `AnalysisStats`, `TagStats`, etc.
  - [ ] `MidiFile`, `MusicalMetadata`, `Tag`, etc.
  - [ ] `SearchQuery`, `SearchFilters`
- [ ] Create event listener helpers for progress events

### 6.2 Pipeline UI Components (`app/src/lib/components/pipeline/`)
- [ ] Create `ImportView.svelte`:
  - [ ] File/folder picker
  - [ ] Import button → calls `invoke<ImportStats>('import_files', { paths })`
  - [ ] Progress bar (subscribes to `import_progress` event)
  - [ ] Stats display (successful, failed, duplicates)
- [ ] Create `AnalyzeView.svelte`:
  - [ ] File selection (from database)
  - [ ] Analyze button → calls `invoke<AnalysisStats>('analyze_files', { file_ids })`
  - [ ] Progress bar and stats
- [ ] Create `SearchView.svelte`:
  - [ ] Search input with filters (BPM range, key, instruments)
  - [ ] Results table (file name, BPM, key, tags)
  - [ ] Pagination controls
- [ ] Create `TagView.svelte`:
  - [ ] Auto-tag button
  - [ ] Tag browser/editor

### 6.3 DAW UI Components (`app/src/lib/components/daw/`)
- [ ] Create `Sequencer.svelte`:
  - [ ] Transport controls (play, stop, pause)
  - [ ] Tempo slider
  - [ ] Position/timeline display
  - [ ] Connects to DAW commands
- [ ] Create `TrackList.svelte`:
  - [ ] Track rows with mute/solo buttons
  - [ ] Add/remove track buttons
  - [ ] Volume/pan controls (future)
- [ ] Create `MidiDevices.svelte`:
  - [ ] Device list (from `list_midi_devices`)
  - [ ] Connect/disconnect buttons

### 6.4 Shared Components (`app/src/lib/components/shared/`)
- [ ] Create `FileCard.svelte` - Display MIDI file metadata
- [ ] Create `ProgressBar.svelte` - Reusable progress indicator
- [ ] Create `ErrorDisplay.svelte` - Show errors from commands

### 6.5 Routing & Pages
- [ ] Create `app/src/routes/pipeline/+page.svelte` - Pipeline UI
- [ ] Create `app/src/routes/daw/+page.svelte` - DAW UI
- [ ] Create `app/src/routes/+layout.svelte` - App layout with navigation

---

## Phase 7: Scripts & Utilities

### 7.1 Import Scripts (`scripts/`)
- [ ] Create `run-pipeline-ultra-fast.sh`:
  - [ ] Enable LUDICROUS mode (fsync=off, synchronous_commit=off)
  - [ ] Batch import MIDI files at 7,830/sec
  - [ ] Print final stats
  - [ ] **WARNING:** LUDICROUS mode is unsafe, offline only
- [ ] Create `organize-database.sh`:
  - [ ] Run auto-tagger on all files
  - [ ] Apply 97 instrument categories
  - [ ] Generate summary statistics

### 7.2 Database Scripts
- [ ] Create `scripts/backup-database.sh`:
  - [ ] pg_dump with custom format
  - [ ] Timestamp backups
- [ ] Create `scripts/restore-database.sh`:
  - [ ] pg_restore from backup
- [ ] Create `scripts/vacuum-analyze.sh`:
  - [ ] VACUUM ANALYZE all tables
  - [ ] Refresh statistics

### 7.3 CLI Tools (`scripts/import-tool/`, `scripts/test-midi-files/`)
- [ ] Create standalone CLI import tool
- [ ] Create MIDI file testing utility
- [ ] Share core analysis logic with main app

---

## Phase 8: Testing & Validation

### 8.1 Unit Tests
- [ ] Test all pure analysis functions:
  - [ ] `bpm_detector::analyze_bpm()` with sample files
  - [ ] `key_detector::analyze_key()` with known keys
  - [ ] `drum_analyzer::analyze_drums()` with drum patterns
  - [ ] `auto_tagger::auto_tag()` with instrument files
- [ ] Test all repository functions:
  - [ ] Deduplication logic in `FileRepository::insert()`
  - [ ] Batch insert performance
  - [ ] Search with various filters
- [ ] Test models:
  - [ ] Serialization/deserialization
  - [ ] Database mapping with sqlx

### 8.2 Integration Tests
- [ ] Create `verification/src/main.rs` - Validation suite
- [ ] Test full pipeline workflow:
  - [ ] Import 1000 files → Analyze → Tag → Search
  - [ ] Verify stats match expectations
- [ ] Test DAW workflow:
  - [ ] Load file → Start sequencer → Stop → Verify state
- [ ] Test database constraints:
  - [ ] Unique hash constraint
  - [ ] Foreign key integrity
  - [ ] Transaction rollback behavior

### 8.3 Performance Tests
- [ ] Benchmark import speed (target: 7,830/sec)
- [ ] Benchmark analysis speed (target: 181-360/sec)
- [ ] Benchmark query latency (target: <10ms simple, <100ms complex)
- [ ] Benchmark deduplication (verify 73.4% detection on test set)

### 8.4 Test Configuration
- [ ] Configure `cargo test --workspace --lib --test-threads=1`
- [ ] Set up test database (separate from dev/prod)
- [ ] Create test fixtures (sample MIDI files)
- [ ] Configure `cargo tarpaulin` for coverage reports (HTML output)

---

## Phase 9: Performance Optimization

### 9.1 Database Optimizations
- [ ] Implement COPY for batch inserts (not INSERT)
- [ ] Create materialized views for common queries
- [ ] Set up connection pooling (min: 5, max: 20 connections)
- [ ] Configure PostgreSQL parameters:
  - [ ] `shared_buffers = 256MB`
  - [ ] `effective_cache_size = 1GB`
  - [ ] `work_mem = 16MB`
- [ ] Implement query result caching (Redis)

### 9.2 LUDICROUS Mode (Import Only)
- [ ] Add `--ludicrous` flag to import command
- [ ] Set PostgreSQL: `fsync=off`, `synchronous_commit=off`, `full_page_writes=off`
- [ ] Display **UNSAFE** warning in UI
- [ ] Disable for production/online systems
- [ ] Auto-restore safe settings after import

### 9.3 Indexing Strategy
- [ ] Verify all 60+ indexes are created
- [ ] EXPLAIN ANALYZE on common queries
- [ ] Identify missing indexes from slow query log
- [ ] Balance index count vs insert performance

### 9.4 Frontend Performance
- [ ] Implement virtual scrolling for large file lists
- [ ] Debounce search input
- [ ] Lazy load components
- [ ] Optimize Svelte reactivity (avoid unnecessary re-renders)

---

## Phase 10: Error Handling & Logging

### 10.1 Backend Error Handling
- [ ] Define `DbError` enum with `thiserror`:
  - [ ] `QueryFailed(String)`
  - [ ] `NotFound { path: String }`
  - [ ] `DuplicateHash { hash: String }`
  - [ ] `ConnectionError(String)`
  - [ ] `MigrationError(String)`
- [ ] Convert all errors to `String` in Tauri commands: `.map_err(|e| e.to_string())`
- [ ] Implement context-aware error messages
- [ ] Log all errors with `tracing` crate

### 10.2 Frontend Error Handling
- [ ] Display errors in UI (ErrorDisplay component)
- [ ] Implement retry logic for transient failures
- [ ] Show user-friendly error messages
- [ ] Log errors to console with context

### 10.3 Logging Configuration
- [ ] Set up `tracing` subscriber with env filter
- [ ] Configure `RUST_LOG=info,midi_app=debug`
- [ ] Implement structured logging (JSON format)
- [ ] Rotate log files (daily, max 100MB)
- [ ] Separate logs: `app.log`, `database.log`, `midi.log`

---

## Phase 11: Documentation & Onboarding

### 11.1 Code Documentation
- [ ] Add rustdoc comments to all public APIs
- [ ] Document Tauri command parameters and return types
- [ ] Document database schema in migration files
- [ ] Create module-level documentation (//!)

### 11.2 User Documentation
- [ ] Update `README.md` with:
  - [ ] Feature overview
  - [ ] Quick start guide
  - [ ] Installation instructions
- [ ] Create `ARCHITECTURE_REFERENCE.md`:
  - [ ] Database schema diagram
  - [ ] Module dependency graph
  - [ ] Performance benchmarks
- [ ] Create `CONTRIBUTING.md`:
  - [ ] Development workflow
  - [ ] Testing requirements
  - [ ] Code style guide

### 11.3 Migration Guides
- [ ] Document migration creation process
- [ ] Create rollback procedures
- [ ] Document LUDICROUS mode risks

---

## Phase 12: Future Features (Documented but Not Implemented)

### 12.1 Lua Scripting Runtime
- [ ] Implement Lua 5.4 runtime with `mlua`
- [ ] Expose MIDI API to Lua scripts
- [ ] Create script editor in UI
- [ ] Document scripting API

### 12.2 Meilisearch Integration
- [ ] Index MIDI files in Meilisearch
- [ ] Implement fuzzy search
- [ ] Replace ILIKE queries with Meilisearch
- [ ] Sync database changes to search index

### 12.3 VIP3 Browser Window
- [ ] Implement file browser with filters
- [ ] Multi-column layout
- [ ] Drag-and-drop to DAW

### 12.4 Advanced DAW Features
- [ ] Volume/pan controls per track
- [ ] Effect processing
- [ ] MIDI recording
- [ ] Piano roll editor

---

## Phase 13: Deployment & Operations

### 13.1 Build Optimization
- [ ] Configure release profile: `lto = true`, `codegen-units = 1`
- [ ] Strip debug symbols
- [ ] Optimize binary size with `strip = true`
- [ ] Create platform-specific installers (Tauri bundler)

### 13.2 Database Backup Strategy
- [ ] Implement automated daily backups (pg_dump)
- [ ] Test restoration procedures
- [ ] Document disaster recovery plan
- [ ] Set up point-in-time recovery (PITR)

### 13.3 Monitoring & Observability
- [ ] Implement health check endpoint
- [ ] Monitor database connection pool
- [ ] Track import/analysis throughput
- [ ] Alert on error rate spikes

### 13.4 Security Hardening
- [ ] Parameterized queries only (prevent SQL injection)
- [ ] Validate all user inputs
- [ ] Implement rate limiting on commands
- [ ] Secure PostgreSQL connection (SSL/TLS)
- [ ] Review file path handling (prevent path traversal)

---

## Phase 14: Verification Checklist

### 14.1 Common Pitfalls (Verify Not Present)
- [ ] ✅ All tests use `--test-threads=1`
- [ ] ✅ No edited migrations (only new files)
- [ ] ✅ All Tauri commands are `async`
- [ ] ✅ All command errors converted to `String`
- [ ] ✅ Frontend uses `invoke<T>()` wrapper (never raw `tauriInvoke`)
- [ ] ✅ MIDI parsing uses `midly` crate
- [ ] ✅ Batch operations (>100K files) use COPY
- [ ] ✅ No raw SQL in commands (only in repositories)

### 14.2 Pattern Compliance
- [ ] ✅ Pipeline commands tolerate partial failures (return stats)
- [ ] ✅ DAW commands fail-fast (all-or-nothing)
- [ ] ✅ All analysis functions are pure (no I/O)
- [ ] ✅ Repositories follow established patterns
- [ ] ✅ Models derive required traits (FromRow, Serialize, Deserialize)

### 14.3 Performance Targets
- [ ] ✅ Import: 7,830 files/sec
- [ ] ✅ Analysis: 181-360 files/sec
- [ ] ✅ Simple queries: <10ms
- [ ] ✅ Complex queries: <100ms
- [ ] ✅ Deduplication: 73.4% detection rate

---

## Appendix: Quick Command Reference

```bash
# Setup & Development
make setup              # First-time setup
make dev                # Start dev server (:5173)
make docker-up          # Start PostgreSQL, Meilisearch, Redis

# Testing
cargo test --workspace --lib --test-threads=1
cargo tarpaulin --workspace --out Html

# Database
make db-migrate         # Apply migrations
make db-reset          # Reset database (DESTRUCTIVE)
psql "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Pipeline
./scripts/run-pipeline-ultra-fast.sh    # Import files
./scripts/organize-database.sh          # Apply tags

# Quality
make check             # Pre-commit validation
make lint              # Clippy + format check
make format            # Format code
```

---

## Implementation Priority

**Critical Path (MVP):**
1. Phase 1-2: Foundation + Database (weeks 1-2)
2. Phase 3: MIDI Analysis Core (week 3)
3. Phase 4.1: Pipeline Commands (week 4)
4. Phase 6.1-6.2: Frontend API + Pipeline UI (week 5)
5. Phase 8: Testing & Validation (week 6)

**Enhanced Features:**
6. Phase 4.2: DAW Commands (week 7)
7. Phase 5: MIDI Hardware Backend (week 8)
8. Phase 6.3: DAW UI (week 9)

**Polish & Performance:**
9. Phase 9: Performance Optimization
10. Phase 10-11: Error Handling + Documentation

**Future Roadmap:**
11. Phase 12: Lua Scripting, Meilisearch, Advanced DAW

---

**Total Estimated Tasks:** 300+
**Complexity:** High (real-time + batch, Rust + TS, database + MIDI)
**Scale Target:** 2.15M files, 7.9M tags, 1,861 tag types
