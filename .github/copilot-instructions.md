# Copilot Instructions - MIDI Software Center

**Last Updated:** December 2025 | **Status:** Production-Ready (Unified Architecture with VIP3 Browser)

## Quick Reference

- **Stack:** Rust (Tokio) + Tauri 2.0 + Svelte/TypeScript + PostgreSQL 16 + pgvector + Meilisearch
- **Performance:** 7,830 import/sec, 181-360 analysis/sec, <10ms queries
- **Scale:** 2.15M files, 7.9M tags, 15 database tables, 60+ indexes
- **Architecture:** Unified app with three major subsystems: Pipeline (batch), DAW (real-time), VIP3 (search/browse)
- **Key UIs:** Pipeline importer + DAW sequencer + VIP3 browser (multi-filter search, favorites, saved searches)

## Architecture Overview

### Unified Application Design

This project combines **Pipeline** (batch MIDI import/analysis) and **DAW** (real-time sequencing) in a single Tauri application. Key implication: all code must support both batch and real-time contexts.

```
Frontend (Svelte/TypeScript) ← IPC/Tauri → Backend (Rust)
   ↓ invoke('command', args)                      ↓ #[tauri::command]
   Pipeline/DAW UI                         Core + Database Layer
```

**Key Workspace Structure:**
- `app/src-tauri/src/` - Unified backend (all functionality)
  - `core/` - MIDI analysis (BPM, key, drums, analysis_parser)
  - `db/` - PostgreSQL repositories + models (15 tables: files, musical_metadata, tags, folders, midi_clips, track_splits, etc.)
  - `commands/` - Tauri IPC handlers (pipeline/ + daw/)
    - `daw/` - 60+ commands: sequencer control, mixer, MIDI I/O, effects, automation, presets, projects
  - `hardware/` - MIDI I/O backends (JACK/ALSA/CoreMIDI/midir auto-detection)
  - `sequencer/` - Real-time sequencing engine (lock-free ringbuffers, playback control)
  - `midi_io/` - Hardware MIDI device management
- `app/src/` - Svelte frontend + TypeScript API wrapper
- `database/migrations/` - PostgreSQL schema (001-019*.sql, 15 tables)
- `verification/` - Test suite (health checks, schema validation)
- `scripts/` - Import tools, CLI utilities, test fixtures

## Critical Developer Workflows

### Build & Development

```bash
make setup                    # First run: setup all dependencies
make dev                      # Start dev server (:5173) - FAST for iteration
make check                    # Format, lint, test (pre-commit)
make docker-up                # Start PostgreSQL 16 + Meilisearch containers
make db-reset                 # DESTRUCTIVE: Wipe database, rerun migrations
```

**Key:** `make dev` rebuilds incrementally; for full rebuild use `cargo build -p midi-software-center`.

### Testing & Validation

```bash
cargo test --workspace --lib -- --test-threads=1    # All tests sequential
cargo test -p midi-software-center --lib             # App tests only
cargo tarpaulin --workspace --out Html               # Coverage report
make lint                                             # Clippy + format check
```

**Critical:** Tests run with `--test-threads=1` because database state is shared. Never remove this flag.

### Database Operations

```bash
make db-migrate          # Apply pending migrations (idempotent)
make db-reset           # Reset to clean state (BACKUP FIRST!)
./scripts/run-pipeline-ultra-fast.sh   # Fast import 7,830 files/sec
./scripts/organize-database.sh          # Apply 97 instrument tags
```

**Pattern:** Migrations go in `database/migrations/` with sequential numbers. **Never edit migrations** – always create new ones.

## Project-Specific Patterns

### 1. Tauri Command Pattern (Frontend-Backend Communication)

**Backend** (`app/src-tauri/src/commands/pipeline/` or `commands/daw/`):
```rust
#[tauri::command]
async fn import_files(paths: Vec<String>, state: tauri::State<'_, AppState>) -> Result<ImportStats, String> {
    // Commands: async, return JSON-serializable types
    // Errors become frontend Error objects
}
```

**Frontend** (`app/src/lib/api.ts`):
```typescript
// Safe wrapper handles Tauri 2.x timing quirks
const stats = await invoke<ImportStats>('import_files', { paths });
// Always use type parameter for type safety
```

**Key:** Commands are async; errors propagate as `String` to frontend. Use `.map_err(|e| e.to_string())` for error conversion.

### 2. Database Repository Pattern

All database access uses **repository pattern** (`app/src-tauri/src/db/repositories/`):

```rust
pub struct FileRepository { pool: PgPool }

impl FileRepository {
    pub async fn insert(&self, file: CreateMidiFile) -> DbResult<i64> {
        // Deduplication by content_hash is built-in
        // Returns duplicate ID if file already exists
    }
    pub async fn search(&self, query: &str, filters: SearchFilters) -> DbResult<Vec<MidiFile>> {
        // Always use parameterized queries (sqlx prevents SQL injection)
    }
}
```

**Create repositories** in `db/repositories/` following existing patterns. **Never write raw SQL in commands** – use repositories.

### 3. MIDI Analysis Pipeline

Located in `app/src-tauri/src/core/`:

- `midi/analysis_parser.rs` - Core MIDI file parsing (midly library)
- `analysis/bpm_detector.rs` - Tempo detection
- `analysis/key_detector.rs` - Key signature detection
- `analysis/drum_analyzer.rs` - Drum pattern recognition
- `analysis/auto_tagger.rs` - Auto-tag by instrument

**Pattern:** Analysis functions are **pure** (no I/O) → highly testable. Example:
```rust
pub fn analyze_bpm(midi_data: &[u8]) -> Result<f64> {
    // Pure analysis, zero I/O, no database calls
}
```

### 4. Performance Considerations

- **Batch Operations:** Use `sqlx::query!` with `COPY` for inserts (see `file_repository.rs`)
- **Deduplication:** Check `content_hash` before insert (automatic in `FileRepository`)
- **LUDICROUS Mode:** Import-only, uses `fsync=off` for speed. Only for offline environments.
- **Indexing:** 60+ indexes optimized for search patterns (see `ARCHITECTURE_REFERENCE.md`)

### 5. Error Handling

Use `thiserror` for custom errors:
```rust
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    QueryFailed(String),
    #[error("File not found: {path}")]
    FileNotFound { path: String },
}

pub type DbResult<T> = Result<T, DbError>;
```

Convert to frontend-friendly strings in commands: `.map_err(|e| e.to_string())`.

### 6. Frontend API Wrapper Pattern

See `app/src/lib/api.ts` – centralized invoke wrapper that:
- Handles Tauri 2.x timing (`__TAURI_INTERNALS__` availability)
- Provides type-safe generic `invoke<T>(command, args)`
- Logs command invocations for debugging

Use this wrapper for all backend communication.

## Three Major Subsystems

### 1. Pipeline (Batch Import & Analysis)
**Location:** `app/src-tauri/src/commands/pipeline/`

Handles high-volume MIDI import, deduplication, analysis, and organization.

**Key Commands:**
- `search_files_vip3()` - Multi-filter database search (main browser query)
- `split_file()` / `split_file_batch()` - Separate multi-track files into individual tracks
- `bulk_retag_vip3()` - High-performance category assignment (97 instrument tags)

**Pattern:** All commands return detailed stats. Long-running operations emit progress events via `emit('import_progress', ...)`.

### 2. DAW (Real-Time Sequencer & MIDI I/O)
**Location:** `app/src-tauri/src/commands/daw/`

Real-time MIDI playback, sequencing, hardware I/O, mixer (60+ commands), effects, automation, presets, project management.

**Key Subsystems:**
- `sequencer/` - Playback engine (`SequencerEngine` with lock-free ringbuffers). Commands: `start_sequencer()`, `pause_sequencer()`, `resume_sequencer()`, `stop_sequencer()`, `seek_position()`, `get_playback_position()`, `add_track()`, `remove_track()`, `get_tracks()`
- `midi.rs` - Hardware MIDI device control (auto-detects JACK/ALSA/CoreMIDI/midir). Commands: `midi_list_devices()`, `midi_connect()`, `midi_disconnect()`, `midi_is_connected()`, `midi_get_current_device()`, `midi_send_test_note()`
- `mixer.rs` - Track mixing, gain, pan, mute, VU metering, master channel. 60+ commands for channel control, effect chains, routing
- `automation.rs` - Parameter automation recording/playback
- `presets.rs` - Preset storage (shared DB + local JSON)
- `project.rs` - Project/session management
- `repair.rs` - MIDI file repair utilities
- `effect.rs` - Effect chain management

**State Management:** Global `Arc<SequencerEngine>` passed to all commands. Ensures thread-safe playback control.

**Critical Pattern:** DAW commands are real-time, **all-or-nothing**. They fail immediately on error (no retries). Example:
```rust
#[tauri::command]
pub async fn start_sequencer(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    engine.start().await  // Fails if already playing
}
```

### 3. VIP3 Browser (Multi-Filter Search & Organization)
**Location:** `app/src-tauri/src/commands/pipeline/vip3/`

Akai VIP3-style category filtering with dynamic WHERE clause building. **Performance:** <100ms for multi-filter queries (up to 2.15M files with 8 simultaneous filters).

**Managed Categories:** Stored in dedicated tables (`timbres`, `styles`, `articulations` - see migrations). User-editable via `add_timbre_to_file()` / `remove_timbre_from_file()` commands.

**Key Commands:**
- `search_files_vip3()` - Main search with dynamic filter building. Supports pagination (up to 500 items/page)
- `get_vip3_filter_counts()` - Real-time count updates (how many files match each filter value). Uses COUNT DISTINCT on indexed columns; stays fast (<50ms) due to partial indexes on category junction tables
- `toggle_favorite()` / `get_favorites()` - Quick-access favorites
- `save_search()` / `load_saved_search()` - Named search templates with use tracking. Saves `SearchFilters` as JSON; tracks `use_count` and `last_used` timestamp for analytics
- `create_collection()` / `add_file_to_collection()` - Ordered file collections
- `add_timbre_to_file()` / `remove_timbre_from_file()` - Dynamic category management

**Search Pattern:** Builds dynamic SQL with parameterized queries:
```rust
// Each filter adds a condition:
// conditions.push(format!("f.folder_id = ANY(${}::bigint[])", param_idx));
// Full WHERE: "1=1 AND f.folder_id = ANY($1) AND EXISTS (SELECT...timbres...) AND..."
```

**Frontend Request Format (TypeScript):**
```typescript
// What VIP3Browser.svelte sends to backend
interface Vip3Filters {
  folder_ids?: bigint[];
  instrument_ids?: bigint[];
  timbre_ids?: number[];
  style_ids?: number[];
  articulation_ids?: number[];
  bpm_range_ids?: number[];
  key_ids?: number[];
  channel?: number;
  search_query?: string;
  favorites_only?: boolean;
  tag_ids?: number[];
  min_rating?: number;  // 1-5 rating filter
  limit?: number;  // Max 500 items/page
  offset?: number;
}
```

**Note:** Uses PostgreSQL `ANY[$::type[]]` operator for multi-select filters. Porting to other SQL databases requires dynamic `IN (?, ?, ...)` parameter expansion.

**Frontend (Svelte):** 
- `VIP3Browser.svelte` - Main UI with 8 filter columns (folder, instrument, timbre, style, articulation, BPM, key, channel)
- `VIP3Column.svelte` - Reusable filter column with checkbox groups and result counts
- `vip3Store.ts` - Centralized state: current filters, search results, pagination, category data
- `vip3BrowserApi.ts` - Type-safe API wrapper with logging

## DAW Command Reference (60+ Commands)

**Sequencer Control:**
- `start_sequencer()`, `pause_sequencer()`, `resume_sequencer()`, `stop_sequencer()` - Playback control
- `seek_position(bar, beat)` - Seek to specific location
- `get_playback_position()` - Current playback state
- `set_tempo(bpm)`, `get_tempo()` - Global tempo control
- `add_track(track)`, `remove_track(track_id)`, `get_tracks()` - Track management

**MIDI Hardware:**
- `midi_list_devices()` - Enumerate connected devices
- `midi_connect(device_name)`, `midi_disconnect()` - Device connection
- `midi_is_connected()` - Connection status
- `midi_get_current_device()` - Active device info
- `midi_send_test_note(channel, note, velocity)` - Verify connection

**Mixer (30+ commands):**
- Channel control: `set_channel_gain()`, `set_channel_pan()`, `set_channel_mute()`, `set_channel_solo()`
- VU metering: `get_channel_meter()` - Real-time level monitoring
- Master channel: `set_master_volume()`, `get_master_volume()`
- Effect chains: `add_effect()`, `remove_effect()`, `bypass_effect()`
- Routing: `set_channel_output()` - Configure output routing

**Automation & Presets:**
- `record_automation()`, `play_automation()`, `clear_automation()` - Parameter automation
- `save_preset()`, `load_preset()`, `delete_preset()` - Preset management
- `get_presets()` - List available presets

**Project Management:**
- `create_project()`, `open_project()`, `save_project()`, `close_project()`
- `get_recent_projects()` - Recently opened projects
- `delete_project(project_id)` - Remove project

**MIDI Repair & Utilities:**
- `repair_midi_file(file_path)` - Auto-fix corrupted MIDI files
- `analyze_midi(file_path)` - Musical analysis (BPM, key, drums, chords)
- `export_midi(file_path, format)` - Export with format selection

**Note:** See `app/src-tauri/src/commands/daw/` for complete command list. Most return `Result<T, String>` following the fail-fast pattern.

## Integration Points

### Cross-Component Communication

1. **Pipeline ↔ Database:** Results persist in database; cached for later access
2. **DAW ↔ Sequencer:** Global `Arc<SequencerEngine>` state passed to all playback commands
3. **DAW ↔ Hardware:** `hardware/midi_backend.rs` manages platform-specific I/O with auto-detection (JACK→ALSA→CoreMIDI→midir priority)
4. **VIP3 ↔ Database:** Dynamic filter queries use parameterized `sqlx` with ANY[] operators for multi-select filters
5. **Frontend ↔ VIP3 Store:** Svelte stores (`vip3Store.ts`) maintain filter selections, pagination, category data across component tree
6. **Lua Scripting:** Dependency ready (`mlua` with vendored Lua 5.4) but runtime not implemented yet – future automation feature
7. **Search:** Currently uses pure PostgreSQL (ILIKE, ANY[] filters); Meilisearch configured but not integrated yet
8. **VIP3 ↔ DAW:** Double-click file in `VIP3Browser.svelte` calls `daw_load_midi_file(file_path)` which parses MIDI and returns Track, then calls `add_track()` to insert into sequencer. Drag-and-drop planned but not yet implemented
   
   **Convenience Wrapper Suggestion:** Create `load_file_to_daw(file_id)` command that fetches file path from database, then calls `daw_load_midi_file()` + `add_track()` atomically. Simplifies frontend code.

### Database Schema Essentials

**Core tables** (see `database/migrations/001_initial_schema.sql` and subsequent migrations):
- `files` - MIDI file metadata (hash, path, duration, format)
- `musical_metadata` - BPM, key, time signature, duration_ticks
- `tags` / `file_tags` - 97 instrument categories (many-to-many)
- `folders` - Folder organization and hierarchy
- `midi_tracks` / `midi_events` - Track structure and note events
- `midi_clips` - Clip/section markers within files
- `track_splits` - Split track metadata (from multi-track separation)
- `analysis_results` - JSON analysis output
- `drum_patterns`, `chords` - Specialized analysis
- `saved_searches` - Named VIP3 search templates with use tracking
- `collections` - Ordered file collections
- `timbres`, `styles`, `articulations` - VIP3 filter categories (see migration 019)

**Key queries available** (pre-optimized):
```sql
SELECT * FROM get_files_by_instrument('ride');
SELECT * FROM get_files_by_bpm_range(118, 122);
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);
```

## File Organization Guide

- **New feature command?** → `app/src-tauri/src/commands/{pipeline|daw}/your_command.rs`
- **New database type?** → `app/src-tauri/src/db/models/your_model.rs`
- **New MIDI analysis?** → `app/src-tauri/src/core/analysis/your_analyzer.rs`
- **New frontend component?** → `app/src/lib/components/YourComponent.svelte`
- **New API module?** → `app/src/lib/api/your_api.ts`

## Common Pitfalls to Avoid

1. **Database:** Don't skip `--test-threads=1` in tests; don't edit migrations
2. **Async:** All Tauri commands must be `async` (even if they don't await)
3. **Errors:** Always convert to `String` before returning from commands
4. **Frontend:** Always use `invoke<T>()` wrapper, never raw `tauriInvoke`
5. **MIDI:** Parsing uses `midly` crate; output is strongly-typed MIDI structures
6. **Performance:** For >100K files, use batch operations with COPY

## Environment Setup Details

### Required Services & Versions

```bash
# PostgreSQL 16+ (required for pgvector)
# Docker Compose handles this automatically
make docker-up   # Starts PostgreSQL:5433, Meilisearch:7700, Redis:6379
```

**Important Port Mappings:**
- PostgreSQL: `5433` (host) → `5432` (container) – **NON-STANDARD PORT** avoids conflicts
- Meilisearch: `7700` (both)
- Svelte Dev Server: `5173`

**Environment Variables** (auto-configured in `docker-compose.yml`):
```bash
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
MEILISEARCH_URL=http://localhost:7700
RUST_LOG=info,midi_app=debug
```

**Platform-Specific Setup:**
- **Linux:** ALSA/JACK for MIDI I/O (optional, auto-detected)
- **macOS:** CoreMIDI (built-in, no setup needed)
- **Windows:** midir fallback (cross-platform, higher latency ~15ms)

**Test Data:** Sample MIDI files expected at `./midi-library/` (create manually or mount external library).

### Common Setup Gotchas

1. **PostgreSQL 16 Required:** Older versions lack `pgvector` extension. Use Docker image.
2. **Port 5433 vs 5432:** Tests connect to `:5433` by default; adjust if different.
3. **Missing `cargo install`:** Run `make setup` to install `sqlx-cli`, `tarpaulin`, etc.
4. **Docker volumes:** `docker-compose down -v` destroys data; backup first.
5. **Frontend deps:** `cd app && npm install` required before `make dev`.

## MIDI Hardware Backend Selection

**Auto-Selected at Runtime** (priority: latency, platform availability):

1. **JACK** (Linux/macOS) – ~3ms, lock-free ringbuffers – best for pro audio
2. **ALSA Raw** (Linux) – ~5ms, direct kernel access – when JACK unavailable
3. **CoreMIDI** (macOS) – ~5ms, native framework – automatic on macOS
4. **midir** (all platforms) – ~10-15ms, cross-platform fallback – guaranteed to work

**Implementation:** `app/src-tauri/src/hardware/midi_backend.rs` detects platform, tries backends in order.

**Cargo Features** (optional backends):
```toml
# Default: midir only (portable)
# Enable JACK/ALSA:
cargo build --features jack,alsa
```

**When Adding MIDI I/O Features:**
- Test on **both** low-latency (JACK) and fallback (midir) backends
- Mock hardware in tests: `#[cfg(test)]` to avoid real device dependencies
- Handle buffer sizes: JACK uses fixed-size frames, midir is variable

## DAW vs Pipeline Command Behavioral Differences

### Pipeline Commands (Batch, Durable, Retry-Safe)

**Location:** `app/src-tauri/src/commands/pipeline/`

**Characteristics:**
- **Tolerates partial failures:** Returns `ImportStats { successful: 100, failed: 5, ... }`
- **Long-running:** Progress updates via events (`emit('import_progress', ...)`)
- **Database transactions:** Auto-commit per batch (not all-or-nothing)
- **Example:**
  ```rust
  #[tauri::command]
  async fn import_files(paths: Vec<String>, state: State<'_, AppState>)
      -> Result<ImportStats, String> {
      // Batch import: some files may fail, return stats
  }
  ```

### DAW Commands (Real-Time, Stateful, Immediate)

**Location:** `app/src-tauri/src/commands/daw/`

**Characteristics:**
- **Immediate feedback:** Returns `Ok(())` or `Err(String)` – all-or-nothing
- **State changes:** Modifies sequencer state (playback position, tempo, tracks)
- **No retries:** Failures require user action
- **Example:**
  ```rust
  #[tauri::command]
  async fn start_sequencer(engine: State<'_, Arc<SequencerEngine>>)
      -> Result<(), String> {
      engine.start().await  // Fails if already playing
  }
  ```

**Key Distinction:**
- **Pipeline:** "Process 1M files, tell me results" → returns partial success stats
- **DAW:** "Start playback now" → either succeeds or fails immediately

**Error Handling Pattern:**
```rust
// Pipeline: continue on error, collect stats
for file in files {
    match process(file) {
        Ok(_) => stats.successful += 1,
        Err(e) => stats.failed.push((file, e))  // Keep going
    }
}

// DAW: fail-fast on error
engine.start().await.map_err(|e| format!("Cannot start: {}", e))?;
```

## References

- **Full Architecture:** `ARCHITECTURE_REFERENCE.md` (database schema, models, patterns)
- **CLAUDE.md:** Quick stats, phase overview, performance metrics
- **README.md:** Feature overview, quick start
- **Migrations:** `database/migrations/` (schema evolution)
- **Verification Suite:** `verification/src/main.rs` (validation patterns)
