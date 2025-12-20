# Copilot Instructions - MIDI Software Center

**Last Updated:** December 2025 | **Status:** Production-Ready (Unified Architecture)

## Quick Reference

- **Stack:** Rust (Tokio) + Tauri 2.0 + Svelte/TypeScript + PostgreSQL 16 + pgvector + Meilisearch
- **Performance:** 7,830 import/sec, 181-360 analysis/sec, <10ms queries
- **Scale:** 2.15M files, 7.9M tags, 15 database tables, 60+ indexes
- **Architecture:** Single unified app (Pipeline batch processing + DAW real-time) in `app/src-tauri`

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
  - `db/` - PostgreSQL repositories + models
  - `commands/` - Tauri IPC handlers (pipeline/ + daw/)
  - `hardware/` - MIDI I/O backends
  - `sequencer/` - Real-time sequencing
  - `scripting/` - Lua runtime (planned, not yet implemented)
- `app/src/` - Svelte frontend + TypeScript API wrapper
- `database/migrations/` - PostgreSQL schema (001-011*.sql)
- `verification/` - Test suite
- `scripts/` - Import tools, utilities

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

## Integration Points

### Cross-Component Communication

1. **Pipeline → DAW:** Results persist in database; DAW reads via repositories
2. **DAW → Hardware:** `hardware/midi_backend.rs` manages platform-specific I/O (JACK, ALSA, CoreMIDI, midir)
3. **Lua Scripting (Future):** Dependency ready (`mlua` 0.9 with vendored Lua 5.4); intended for user macros/instrument control automation. Not implemented yet.
4. **Search:** Currently uses pure PostgreSQL (ILIKE queries, 60+ indexes, <10ms); Meilisearch configured but not integrated yet

### Database Schema Essentials

**Core tables** (see `database/migrations/001_initial_schema.sql`):
- `files` - MIDI file metadata (hash, path, duration, format)
- `musical_metadata` - BPM, key, time signature, duration_ticks
- `tags` / `file_tags` - 97 instrument categories (many-to-many)
- `midi_tracks` / `midi_events` - Track structure
- `analysis_results` - JSON analysis output
- `drum_patterns`, `chords` - Specialized analysis

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

**PostgreSQL 16+ REQUIRED** (older versions lack `pgvector` extension):
```bash
make docker-up   # Starts PostgreSQL:5433, Meilisearch:7700 (docker-compose enforces version)
```

**Environment Variables** (auto-configured in `docker-compose.yml`):
```bash
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
MEILISEARCH_URL=http://localhost:7700
RUST_LOG=info,midi_app=debug
```

**Important Port Mappings:**
- PostgreSQL: `5433` (host) → `5432` (container) – **NON-STANDARD PORT** avoids conflicts
- Meilisearch: `7700` (both)
- Svelte Dev Server: `5173`

**Platform-Specific MIDI Backend Setup:**
- **Linux:** ALSA/JACK for MIDI I/O (optional, auto-detected). Enable with `cargo build --features jack,alsa`
- **macOS:** CoreMIDI (built-in, no setup needed)
- **Windows:** midir fallback (cross-platform, ~10-15ms latency)

### Common Setup Gotchas

1. **PostgreSQL 16 Required:** Migrations fail silently if version <16. Use Docker.
2. **Port 5433 vs 5432:** Tests default to `:5433`. Update `DATABASE_URL` if you change it.
3. **Missing `cargo install` tools:** Run `make setup` to install `sqlx-cli`, `tarpaulin`, etc.
4. **Docker data loss:** `docker-compose down -v` destroys all data. Backup first.
5. **Frontend deps missing:** `cd app && npm install` required before `make dev`.
6. **Test data:** Create `./midi-library/` manually and add sample MIDI files for import tests.

## MIDI Hardware Backend Selection

**Auto-Selected at Runtime** (priority: latency, platform availability):

```rust
// From app/src-tauri/src/hardware/midi_backend.rs
// Selection priority (best-to-fallback):
1. JACK      (Linux/macOS) – ~3ms latency, lock-free ringbuffers
2. ALSA Raw  (Linux)       – ~5ms latency, direct kernel access
3. CoreMIDI  (macOS)       – ~5ms latency, native framework
4. midir     (all)         – ~10-15ms latency, guaranteed to work
```

**When Adding MIDI I/O Features:**
- Test on **both** low-latency (JACK/ALSA) and fallback (midir) backends
- Mock hardware in tests: `#[cfg(test)]` to avoid real device dependencies
- Handle buffer sizes: JACK uses fixed-size frames; midir is variable
- Timestamp with microsecond precision (`timestamp_us` field)

**Cargo Features for Backends:**
```toml
# Default: midir only (portable)
# Enable low-latency:
cargo build --features jack,alsa
```

## DAW vs Pipeline Command Behavioral Differences

### Pipeline Commands: Batch, Durable, Resilient

**Location:** `app/src-tauri/src/commands/pipeline/`

**Characteristics:**
- **Tolerates partial failures:** Returns `ImportStats { successful: 100, failed: 5, errors: [...] }`
- **Long-running:** Progress updates via `emit('import_progress', stats)`
- **Per-item commits:** Each file writes independently (safe to retry)
- **Idempotent:** Safe to re-run (deduplication by `content_hash`)

**Error Handling Pattern:**
```rust
#[tauri::command]
async fn import_files(paths: Vec<String>, state: State<'_, AppState>)
    -> Result<ImportStats, String> {
    let mut stats = ImportStats::default();
    for path in paths {
        match import_single_file(&path, &state).await {
            Ok(id) => stats.successful += 1,
            Err(e) => {
                stats.failed += 1;
                stats.errors.push(e);
                // KEEP GOING - don't fail entire batch
            }
        }
    }
    Ok(stats)  // Returns partial results
}
```

### DAW Commands: Real-Time, Stateful, All-or-Nothing

**Location:** `app/src-tauri/src/commands/daw/`

**Characteristics:**
- **Immediate feedback:** Returns `Ok(())` or `Err(String)` – all-or-nothing
- **State changes:** Modifies sequencer state (playback position, tempo, tracks)
- **Fail-fast:** Stop immediately on error, don't continue
- **Not idempotent:** Failures require user intervention

**Error Handling Pattern:**
```rust
#[tauri::command]
async fn start_sequencer(engine: State<'_, Arc<SequencerEngine>>)
    -> Result<(), String> {
    // Validate state BEFORE any changes
    if engine.is_playing().await {
        return Err("Sequencer already playing".to_string());
    }
    // All-or-nothing: either succeeds or fails
    engine.start().await.map_err(|e| e.to_string())?;
    Ok(())
}
```

| Aspect | Pipeline | DAW |
|--------|----------|-----|
| **Purpose** | Batch data processing | Real-time state control |
| **Failure Model** | Partial failure OK, continue | All-or-nothing, fail-fast |
| **Return Type** | `Result<Stats, String>` | `Result<(), String>` |
| **Latency** | Seconds to hours | <1ms |
| **Example** | Import 1M files (200 fail) | Trigger MIDI note now |

## Lua Scripting Integration (Future Feature)

**Status:** Dependency ready (`mlua` 0.9), runtime not implemented yet.

**Intended Use Cases:**
1. User macros – Record/playback MIDI sequences with transformations
2. Instrument control – CC automation, note pattern generation
3. Batch transformations – Auto-transpose based on detected key

**Implementation Plan (When Needed):**
- Runtime: `app/src-tauri/src/scripting/lua_runtime.rs` (create when ready)
- Bindings: Expose `analyze_bpm`, `analyze_key`, `transpose` functions
- Safety: Sandbox file system access to `./midi-library/` only (no `io`, `os` modules)
- Use `mlua::async` feature for async operations

## Search & Indexing

**Current Status:** Pure PostgreSQL queries (ILIKE with 60+ indexes, <10ms).

**Meilisearch:** Configured but not actively used (future full-text search cache).

**When to Use:**
| Scenario | Solution |
|----------|----------|
| Exact filename match, BPM/key filters | PostgreSQL (current) |
| Full-text search with typo tolerance | Meilisearch (planned) |
| Faceted search UI (filter by genre, artist) | Meilisearch (planned) |
| Similarity search (find similar melodies) | pgvector (planned) |

**Current Implementation** (from `commands/pipeline/search.rs`):
```rust
// Uses ILIKE on filename/filepath, indexed BPM/key ranges
SELECT f.id, f.filename, mm.bpm, mm.key_signature, ...
FROM files f
LEFT JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.filename ILIKE $1 AND mm.bpm BETWEEN $2 AND $3
```

## References

- **Full Architecture:** `ARCHITECTURE_REFERENCE.md` (database schema, models, patterns)
- **CLAUDE.md:** Quick stats, phase overview, performance metrics
- **README.md:** Feature overview, quick start
- **Migrations:** `database/migrations/` (schema evolution)
- **Verification Suite:** `verification/src/main.rs` (validation patterns)
- **Detailed Q&A:** See codebase analysis document for clarifications on hardware, Lua, search, setup, and command patterns

Feedback Requested
This updated version now includes:

✅ Lua clarifications – Status (not implemented), use cases, and implementation plan
✅ Hardware MIDI backend – Selection algorithm, latency tradeoffs, testing guidance
✅ Search strategy – PostgreSQL (current) vs Meilisearch (future) decision table
✅ Setup gotchas – Port 5433, PostgreSQL 16 enforcement, test data location
✅ DAW vs Pipeline – Side-by-side behavioral comparison with code examples

Questions:

Level of detail: Are the hardware backend details sufficient for developers adding MIDI features, or should I expand with example code?
Lua section placement: Should it be higher (core feature) or stay in Integration Points?
Search table: Helpful decision tool, or too prescriptive for a future feature?
Anything still unclear? Let me know which sections need iteration.
