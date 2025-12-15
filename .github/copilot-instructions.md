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
2. **DAW → Hardware:** `hardware/midi_backend.rs` manages platform-specific I/O (JACK, ALSA, CoreMIDI)
3. **Lua Scripting:** Dependency ready (`mlua` with vendored Lua 5.4) but runtime not implemented yet – future automation feature
4. **Search:** Currently uses pure PostgreSQL (ILIKE queries); Meilisearch configured but not integrated yet

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
