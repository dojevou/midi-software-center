# MIDI Software Center - AI Agent Instructions

## Project Overview

**MIDI Software Center** is a high-performance Rust+Tauri desktop application managing 2.15M+ MIDI files with batch processing (pipeline) and real-time DAW capabilities. Key metrics: 7,830 import/sec, 181-360 analysis/sec, <10ms query time.

## Architecture: Unified Application

The codebase combines two operational modes in a single Rust binary:

- **Pipeline** (Batch): File import → sanitization → splitting → analysis → rename (ordered phases)
- **DAW** (Real-time): Sequencer, MIDI I/O, piano roll, automation, mixer

Both share: PostgreSQL backend, core analysis (BPM, key, drums), database repositories, Tauri commands.

**Workspace Structure:**
```
app/src-tauri/         # Main unified app (Pipeline + DAW)
├── src/core/          # Shared analysis, MIDI parsing, pipeline orchestration
├── src/db/            # Database models & repositories
├── src/commands/      # Tauri commands (pipeline/ + daw/ submodules)
└── src/{daw,hardware,midi,sequencer,scripting}/  # DAW modules
scripts/import-tool/   # CLI import utilities
scripts/test-midi-files/  # MIDI testing tool
verification/          # Test suite
database/migrations/   # PostgreSQL (15 tables, 26 migrations)
```

## Core Modules & Responsibilities

| Module | Purpose | Key Files |
|--------|---------|-----------|
| `core::pipeline` | Lock-free pipelined worker architecture (import→analyze→split→rename) | `orchestrator.rs`, `worker_pool.rs`, `queues.rs` |
| `core::analysis` | Musical analysis (BPM, key, drums, chords) | `bpm_detector.rs`, `key_detector.rs`, `drum_analyzer.rs` |
| `core::midi` | MIDI parsing & serialization (separate parsers: analysis vs playback) | `analysis_parser.rs`, `playback_parser.rs` |
| `core::naming` | Smart file renaming (metadata-based) | `naming_engine.rs` |
| `core::hash` | Deduplication via BLAKE3 | `hash_utils.rs` |
| `db::repositories` | Data access layer (file_repo, tag_repo, metadata_repo, etc.) | `file_repository.rs`, `tag_repository.rs` |
| `commands::pipeline` | Tauri handlers for batch operations | `import.rs`, `analyze.rs`, `organize.rs` |
| `commands::daw` | Tauri handlers for playback, sequencer, I/O | `playback.rs`, `sequencer.rs` |

## DAW Components

### Sequencer Engine
- **Event Scheduling**: Priority queue (min-heap) of `ScheduledEvent` objects using tick-based timing
- **Timing Resolution**: Configurable `ticks_per_quarter` (default 480 PPQN = MIDI standard)
- **Playback States**: `Stopped`, `Playing`, `Paused` controlled via transport UI
- **Multi-track**: `TrackManager` holds parallel tracks; `EventScheduler` fires events across all tracks
- **Tempo Sync**: BPM stored in `SequencerEngine`, converted to milliseconds for real-time playback

Key files: [app/src-tauri/src/sequencer/engine.rs](app/src-tauri/src/sequencer/engine.rs#L27), [app/src-tauri/src/sequencer/scheduler.rs](app/src-tauri/src/sequencer/scheduler.rs#L36)

### Hardware MIDI Integration
- **Device Discovery**: `MidiDevice` struct in `device_manager.rs` with port enumeration (inputs/outputs)
- **ALSA Integration** (Linux): Underlying `midir` crate handles device detection
- **MIDI Clock Sync**: Dedicated `midi_clock.rs` module for sync/slave mode control
- **Device Mapping**: `DeviceMapping` struct for CC/note mapping and channel routing
- **Port Management**: `MidiRouter` coordinates input/output port connections

Key files: [app/src-tauri/src/hardware/device_manager.rs](app/src-tauri/src/hardware/device_manager.rs#L19), [app/src-tauri/src/hardware/midi_router.rs](app/src-tauri/src/hardware/midi_router.rs)

### Lua Scripting Runtime
- **Sandboxing**: MLua runtime with restricted API bindings (no filesystem access by default)
- **Script Types**: `MidiProcessor` (real-time transforms), `Generator` (arpeggiator), `Automation`, `Action`
- **API Bindings**: Scripts interact via `ScriptAction` enum (SendMidi, SetTempo, SetParameter, Log, etc.)
- **Use Cases**: Real-time MIDI processing, parameter automation, event generation, custom analysis
- **Storage**: Scripts stored in user directory with metadata (`ScriptInfo` struct)

Key files: [app/src-tauri/src/scripting/lua_runtime.rs](app/src-tauri/src/scripting/lua_runtime.rs#L86)

### Automation System
- **Automation Points**: Ordered by time with normalized values (0.0-1.0)
- **Curve Types**: `Linear`, `Bezier`, `Exponential`, `Step`, `SCurve` interpolation
- **Recording Modes**: `Read` (playback), `Write` (overwrite), `Touch` (hybrid)
- **Parameter Binding**: Generic path-based parameter targets (volume, pan, CC, custom)
- **Serialization**: Full automation lanes persisted with project data

Key files: [app/src-tauri/src/automation.rs](app/src-tauri/src/automation.rs#L15)

### VIP3 Browser (File Search & Organization)
- **Dynamic Filtering**: `Vip3Filters` struct with multi-faceted filters (BPM, key, instruments, tags)
- **Saved Searches**: Persisted filter presets via `Vip3Repository`
- **Collections**: Grouped file sets with custom organization
- **Filter Counts**: Real-time counts of matching files per filter category
- **Frontend State**: `vip3Store.ts` manages active filters and search results

Key files: [app/src-tauri/src/commands/pipeline/vip3/](app/src-tauri/src/commands/pipeline/vip3/), [app/src/lib/stores/vip3Store.ts](app/src/lib/stores/vip3Store.ts)

### Frontend State Management (Svelte Stores)
- **Pattern**: `writable` and `derived` stores from `svelte/store` for reactive state
- **Key Stores** (all implemented in [app/src/lib/stores/](app/src/lib/stores/)):
  - `uiStore.ts` - Active windows, panel visibility, layout
  - `vip3Store.ts` - File browser filters and search state
  - `playbackStore.ts` - Current position, tempo, playback state
  - `automationStore.ts` - Active automation lanes
  - `midiDeviceStore.ts` - Connected devices and routing
  - `errorStore.ts` - Error messages and alerts
- **Tauri Sync**: Stores emit commands via `invoke()` when state changes; listen to `listen()` for backend updates

Example: [app/src/lib/stores/uiStore.ts](app/src/lib/stores/uiStore.ts#L265)

### Window Management
- **Multi-window Architecture**: Main window + secondary editors (mixer, export, notation)
- **Window Types**: Defined in `windows/` module with per-window state
- **Base Component**: `WindowBase.svelte` wrapper for consistent chrome
- **State Sync**: Window state (position, size, docking) managed via `uiStore`

## Critical Patterns & Conventions

### 1. Pipeline Architecture
- **Lock-free pipelined design**: All 6 phases run simultaneously via MPMC channels
- **Worker pool model**: Fixed thread count, FIFO job dispatch
- **Order:** Import → Sanitize → Split → Analyze → Rename (immutable config)
- **Never change migration order** - Always create new migrations (`database/migrations/NNN_*.sql`)

Example: [app/src-tauri/src/core/pipeline/orchestrator.rs](app/src-tauri/src/core/pipeline/orchestrator.rs)

### 2. Database Patterns
- **Repository pattern** for all DB access (no raw queries in business logic)
- **SQLx compile-time checking** (`sqlx::query!()` validates against live DB schema)
- **Batch operations** for performance (insert 1000s at once)
- **Connection pooling** via SQLx pool in database service

Key tables: `files`, `musical_metadata`, `file_tags`, `tags`, `midi_tracks`, `analysis_results`, `drum_patterns`

Example query functions:
```sql
SELECT * FROM get_files_by_instrument('piano');
SELECT * FROM get_files_by_bpm_range(118, 122);
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);
```

### 3. MIDI Parsing & Analysis
- **Two separate MIDI parsers**: `analysis_parser` (extracts BPM, key, notes) vs `playback_parser` (full event-based for real-time)
- **BPM detection** uses three methods: tempo events, onset analysis, hybrid (fallback)
- **Key detection** via chromagram + FFT
- **No mutable MIDI data** - all modifications create new structures

Example: [app/src-tauri/src/core/analysis/bpm_detector.rs](app/src-tauri/src/core/analysis/bpm_detector.rs#L110)

### 4. Tauri Command Structure
- All commands are `async fn` with `#[tauri::command]` macro
- Commands organized by feature: `commands/pipeline/*` vs `commands/daw/*`
- Return types: `Result<T, String>` (errors serialized to JSON)
- No blocking I/O - use `tokio` for async tasks

Pattern:
```rust
#[tauri::command]
pub async fn import_files(paths: Vec<String>, #[state] db: State<'_, Database>) -> Result<ImportResult, String> {
    // async pipeline orchestration
}
```

### 5. Error Handling
- Use `anyhow::Result<T>` for internal logic (rich error chains)
- Convert to `Result<T, String>` for Tauri command boundaries
- No unwrap() in production code (use `?` operator or `.context()`)

Example: [CLAUDE.md](CLAUDE.md) rule: "Never edit migrations"

### 6. Testing Approach
- **Unit tests** in `tests/` subdirectories (same modules)
- **Pipeline tests** run phases sequentially with mock data
- **Database tests** use test database (fixture schema)
- Run with `cargo test --workspace --lib -- --test-threads=1` (sequential for DB isolation)

## Development Workflows

### Start Development
```bash
make setup               # Install Rust, Node, Docker
make docker-up          # Start PostgreSQL (port 5433)
make db-migrate         # Create schema
make dev                # Tauri + Vite dev server (:5173)
```

### Testing & Quality
```bash
cargo test --workspace --lib -- --test-threads=1  # Run all tests
cargo fmt --all                                    # Format code
cargo clippy --workspace -- -D warnings            # Lint
make check                                         # Format + lint + test
```

### Pipeline Workflows
```bash
./scripts/run-pipeline-ultra-fast.sh /path/to/files  # Import (7,830/sec)
./scripts/organize-database.sh                       # Apply 97 instrument tags
cargo test --package verification                    # Verify output
```

### Performance Profiling
```bash
make pgo-build          # Build with PGO (10-20% faster, 15-25 min)
make bench              # Run benchmarks
cargo tarpaulin --workspace --out Html  # Coverage report
```

### Database Operations
```bash
make db-migrate         # Apply pending migrations (never edit existing)
make db-reset          # Reset schema (backs up first)
make db-shell          # Open psql shell
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "SELECT COUNT(*) FROM files;"
```

## Project-Specific Conventions

1. **Never use `fsync=off`** except in LUDICROUS import mode (unsafe)
2. **Always backup before `db-reset`** or `docker-compose down -v`
3. **Migrations are immutable** - create new migrations, never edit existing
4. **No direct SQL in Rust** - use repositories or SQLx compile-time queries
5. **BPM detection** uses 120 as fallback (musical default)
6. **File deduplication** happens at import (BLAKE3 hash comparison)
7. **Async everything** - Tokio runtime required, no blocking syscalls
8. **Frontend is Svelte 5 + TypeScript** - type-safe event handling between frontend/backend

## Common Tasks

### Add a New Tauri Command

1. **Create command file** in `commands/{pipeline,daw}/` with your feature name
2. **Use async + Result<T, String> signature**:
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String, #[state] db: State<'_, Database>) -> Result<MyResponse, String> {
       // Convert internal anyhow::Result to String error
       db.some_operation(param).await.map_err(|e| e.to_string())
   }
   ```
3. **Register in mod.rs** - add `mod my_module; pub use my_module::*;`
4. **Wire in main.rs** - invoke command builder: `.invoke_handler(tauri::generate_handler![my_command])`
5. **Call from frontend** via `invoke('my_command', { param })` in TypeScript

### Create a Database Migration

1. **Create file** `database/migrations/NNN_feature_name.sql` (increment NNN sequentially)
2. **Write migration** (SQLx migrations are one-way - rollback is manual):
   ```sql
   -- Migration UP only (SQLx doesn't support automatic DOWN)
   CREATE TABLE my_table (
       id BIGSERIAL PRIMARY KEY,
       name TEXT NOT NULL,
       created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
   );
   CREATE INDEX idx_my_table_name ON my_table(name);
   
   -- For manual rollback reference (if needed):
   -- DROP TABLE my_table;
   ```
3. **Validate schema** - run `make db-migrate` and test with `cargo test --workspace --lib`
4. **Never edit existing migrations** - create a new one to modify schema
5. **SQLx compile-time checks** - queries using `sqlx::query!()` will validate against live schema
6. **Rollback strategy** - For production, create a new migration with corrective DDL or use `make db-reset` (dev only)

### Add a New Analysis Algorithm

1. **Create module** in `core/analysis/` (e.g., `chord_detector.rs`)
2. **Use MIDI parser input**:
   ```rust
   use crate::core::midi::analysis_parser::MidiFile;
   
   pub fn detect_chords(midi_file: &MidiFile) -> Vec<ChordInfo> {
       // Pure function - no mutations
       // Access midi_file.tracks[].notes
   }
   ```
3. **Add tests** in `tests/` subdirectory with fixture MIDI files
4. **Integrate into pipeline** - call from `core/pipeline/workers.rs` AnalyzeWorker
5. **Store results** via `AnalysisRepository` - insert JSON into `analysis_results` table

### Debug MIDI Parsing Issues

1. **Check parser used**: `analysis_parser` for import, `playback_parser` for DAW playback
2. **Enable logging**: Set `RUST_LOG=midi_software_center=debug` environment variable
3. **Add tracing**: Use `tracing::debug!("event: {:?}", event)` in parser
4. **Validate MIDI file**: Use test utility `cargo run --bin test-midi-files -- /path/to/file.mid`
5. **Compare parsers**: If one works but not the other, check differences in event filtering
6. **Run test MIDI**: Use fixture files in `tests/fixtures/midi/` to reproduce issues reproducibly

## Troubleshooting

### Database Connection Issues

**Symptom**: "connection refused" or "connection timeout"

**Solution**:
1. Verify PostgreSQL is running: `docker ps | grep postgres`
2. Check connection string in `.env`: `postgresql://midiuser:145278963@localhost:5433/midi_library`
3. Ensure database exists: `make db-shell` then `\l` to list databases
4. Restart database: `make docker-down && make docker-up && make db-migrate`
5. Check port (5433 for dev, not 5432): `lsof -i :5433`

### SQLx Compile-Time Check Failures

**Symptom**: `cargo check` fails with "column not found" or "table does not exist"

**Solution**:
1. Verify schema is migrated: `make db-migrate && make db-shell`
2. Check `.sqlx/` cache is up-to-date: `rm -rf .sqlx && cargo build`
3. Ensure test database exists (migrations create it automatically)
4. Verify query matches actual schema: `SELECT * FROM information_schema.columns WHERE table_name='files'`
5. Use prepared statements instead of string concatenation: `sqlx::query!()` not `query()`

### Common Tauri IPC Errors

**Symptom**: "command not found" or JSON serialization errors

**Solution**:
1. Verify command is registered: grep for `invoke_handler` in `main.rs`
2. Check return type implements `Serialize`: use `serde::{Serialize, Deserialize}`
3. Ensure Tauri command name matches frontend call: `invoke('snake_case_name')`
4. Check error is `String` type (Tauri requires this): `Result<T, String>`
5. Log frontend error: wrap invoke in `.catch(err => console.error(err))`

### Pipeline Phase Failures

**Symptom**: Phase hangs, workers crash, or files stuck in queue

**Solution**:
1. Check worker thread count: `orchestrator.rs` sets `num_workers` per phase
2. Enable phase logging: `RUST_LOG=midi_software_center::core::pipeline=trace`
3. Verify MIDI file validity: corrupt files can hang BPM detection
4. Check disk space: split/analysis phases create temporary files
5. Monitor queue sizes: `queues.rs` exposes queue depths for debugging
6. Reset pipeline state: `make db-reset` clears in-progress markers

## File References for Examples

- **Pipeline orchestration**: [app/src-tauri/src/core/pipeline/orchestrator.rs](app/src-tauri/src/core/pipeline/orchestrator.rs)
- **Database repositories**: [app/src-tauri/src/db/repositories/file_repository.rs](app/src-tauri/src/db/repositories/file_repository.rs)
- **MIDI analysis**: [app/src-tauri/src/core/analysis/bpm_detector.rs](app/src-tauri/src/core/analysis/bpm_detector.rs)
- **Tauri commands**: [app/src-tauri/src/commands/mod.rs](app/src-tauri/src/commands/mod.rs)
- **Database schema**: [database/migrations/](database/migrations/)
- **Configuration & tooling**: [Makefile](Makefile), [Cargo.toml](Cargo.toml)
- **Project context**: [CLAUDE.md](CLAUDE.md) (performance targets & commands)

## When in Doubt

1. **Check CLAUDE.md** for quick reference on architecture & commands
2. **Look at migrations** for database schema understanding
3. **Trace Tauri commands** to understand frontend-backend flow
4. **Run tests** to verify assumptions about module behavior
5. **Use `make check`** before committing (format + lint + test)

---

*Last updated: December 2025 | 2.15M files, 1,861 tag types, 7.9M+ tags, 99.5% auto-repair success*
