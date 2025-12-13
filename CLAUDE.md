# CLAUDE.md - MIDI Software Center

## Tool Usage Rules

**IMPORTANT - Prevent Context Bloat:**
- Use the built-in `Read` tool, NOT `mcp__filesystem__read_multiple_files`
- Read files one at a time with specific line ranges when possible
- Avoid reading more than 3 files in a single operation
- Use `Grep` to find specific content instead of reading entire files

## Status: PRODUCTION READY (Phase 17 Complete)

**Scale:** 2.15M files, 7.9M tags, 1,861 tag types, 15 tables, 60+ indexes, 1,051 tests passing

## Quick Start

```bash
make setup && make docker-up    # Setup
make dev-both                   # Dev servers (:5173 Pipeline, :5174 DAW)
make format test                # Format & test
./scripts/run-pipeline-ultra-fast.sh    # Import files
./scripts/organize-database.sh          # Apply 97 instrument tags
```

## Architecture

**Components:** Pipeline (batch import/analysis), DAW (real-time MIDI), Shared (parser/DB), Database (PostgreSQL/Meilisearch)

**Stack:** Rust backend + Tauri + Svelte/TypeScript frontend + PostgreSQL 16 + pgvector + Meilisearch

**Workspace:** `pipeline/src-tauri`, `daw/src-tauri`, `shared/rust`, `database/migrations/`

## Pipeline Phases

| Phase | Purpose | Speed |
|-------|---------|-------|
| 1. Import | Ingest + hash + parse + tag + index | 7,830/sec |
| 2. Sanitize | Clean filenames | Instant |
| 3. Split | Multi-track separation + auto-repair | Batch |
| 4. Analyze | BPM, key, drums, chords, structure | 181-360/sec |
| 5. Rename | Metadata-based filenames | Batch |

**Order:** Import -> Sanitize -> Split -> Analyze -> Rename

## Key Files

| Purpose | Location |
|---------|----------|
| MIDI Parser | `shared/rust/src/core/midi/parser.rs` |
| BPM Detector | `shared/rust/src/core/analysis/bpm_detector.rs` |
| Key Detector | `pipeline/src-tauri/src/core/analysis/key_detector.rs` |
| Auto-tagger | `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` |
| Drum Analyzer | `pipeline/src-tauri/src/core/analysis/drum_analyzer.rs` |
| File Repository | `pipeline/src-tauri/src/db/repositories/file_repository.rs` |
| Migrations | `database/migrations/001-011*.sql` |

## Database Schema (15 tables)

- `files` - Core metadata (hash, size, path)
- `musical_metadata` - BPM, key, duration, time sig
- `tags` / `file_tags` - Many-to-many tagging (97 instruments)
- `midi_tracks` / `midi_events` - Track/event data
- `analysis_results` - Enhanced analysis JSON
- `drum_patterns` - Drum-specific analysis
- `chords` - Chord progressions

**Key Queries:**
```sql
SELECT * FROM get_files_by_instrument('ride');
SELECT * FROM get_files_by_bpm_range(118, 122);
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);
```

## Development

```bash
cargo test --workspace --lib -- --test-threads=1  # Run tests (shared DB state)
cargo tarpaulin --workspace --out Html            # Coverage report
make check                                        # Pre-commit validation
```

**Rules:**
- Never edit migrations - always create new
- Use `--test-threads=1` for DB tests
- LUDICROUS mode is import-only (unsafe: fsync=off)
- Backup before `make db-reset` or `docker-compose down -v`

## Performance Achieved

| Metric | Value | Industry |
|--------|-------|----------|
| Import | 7,830/sec | 10-60/sec |
| Analysis | 181-360/sec | 10-50/sec |
| Query (simple) | <10ms | - |
| Query (complex) | <100ms | - |
| Deduplication | 73.4% (4.74M dupes removed) | - |

## Component Separation

- **Shared:** MIDI parsing, BPM/key, DB models (NO UI, NO commands)
- **Pipeline:** Batch ops, archives (NO playback, NO MIDI I/O)
- **DAW:** Real-time, MIDI hardware (NO batch import, NO archives)

## DB Connection

```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library"
```
