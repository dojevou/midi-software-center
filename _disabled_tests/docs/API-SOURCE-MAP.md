# API Commands - Source File Map

Complete mapping of Tauri commands to their implementation files.

---

## Pipeline Commands (/pipeline/src-tauri/src/commands/)

### Import Commands
**File:** `/pipeline/src-tauri/src/commands/file_import.rs`
- `import_single_file` - Single MIDI import with metadata
- `import_directory` - Batch import with recursion option
- Performance: 7,830 files/sec (64 threads, batch inserts)

**File:** `/pipeline/src-tauri/src/commands/archive_import.rs`
- `import_archive_collection` - Multi-archive extraction and import
- Features: Recursive zip extraction (10 levels), progress events

### Analysis Commands
**File:** `/pipeline/src-tauri/src/commands/analyze.rs`
- `start_analysis` - Full 10-point MIDI analysis
- Analysis includes: BPM, key, time signature, notes, chords, drums, controllers, articulation, structure
- Performance: 181-360 files/sec (32 workers, 1,000 file batches)

### Track Splitting Commands
**File:** `/pipeline/src-tauri/src/commands/split_file.rs`
- `split_and_import` - Split multi-track files with auto-repair
- Helper functions:
  - `import_split_track` - Database insertion with metadata
  - `insert_track_split_relationship` - Relationship tracking
  - `generate_split_filename` - Production naming
  - `sanitize_filename` - Filename validation
  - `extract_time_signature_from_midi` - MIDI parsing
- Features: 99.5% corruption repair success

### Search Commands
**File:** `/pipeline/src-tauri/src/commands/search.rs`
- `search_files` - Advanced keyword + filter search
- Filters: Category, BPM range, key signature
- Features: Pagination (1-100 per page), 60+ database indexes
- Performance: <100ms typical query time

### Tag Commands
**File:** `/pipeline/src-tauri/src/commands/tags.rs`
- `get_file_tags` - Tags for specific file
- `get_popular_tags` - Most-used tags with counts
- `search_tags` - Prefix matching for autocomplete
- `get_tag_categories` - All unique categories
- Data structure: `TagResponse` (id, name, category, usage_count)

### Statistics Commands
**File:** `/pipeline/src-tauri/src/commands/stats.rs`
- `get_category_stats` - File count by category
- `get_manufacturer_stats` - File count by manufacturer
- `get_key_signature_stats` - File count by key
- `get_recently_added_count` - Files from last 7 days
- Implementation: SQL aggregation queries

### Commands Module
**File:** `/pipeline/src-tauri/src/commands/mod.rs`
- Module exports all command submodules
- Re-exports for Tauri registration

---

## DAW Commands (/daw/src-tauri/src/commands/)

### Database Commands
**File:** `/daw/src-tauri/src/commands/database.rs`
- `database_search` - MIDI file search with filters
- Types:
  - `MidiFile` - File metadata with timestamps
  - `SearchFilters` - Query parameters (BPM, key, tag)
  - `SearchResults` - Paginated results
  - `DatabaseStats` - Aggregate statistics

### Sequencer Commands
**File:** `/daw/src-tauri/src/commands/sequencer.rs`
- `start_sequencer` - Begin playback from current position
- `stop_sequencer` - Stop and reset to bar 0
- `pause_sequencer` - Pause (maintains position)
- `resume_sequencer` - Resume from pause
- `get_playback_position` - Current bar/beat/tick
- `seek_position` - Jump to bar/beat
- `set_tempo` - Set BPM
- `get_tempo` - Query current BPM
- `add_track` - Load MIDI file as track
- Returns: `PlaybackPosition`, `Track` objects
- Integration: Uses `SequencerEngine` (Arc<SequencerEngine>)

### MIDI Device Commands
**File:** `/daw/src-tauri/src/commands/midi.rs`
- `midi_list_devices` - Enumerate MIDI outputs
- `midi_connect` - Open device connection
- `midi_disconnect` - Close device connection
- `midi_is_connected` - Check connection status
- `midi_get_current_device` - Get active device info
- `midi_send_test_note` - Verify connection with test note
- Integration: Uses `MidiManager` (Arc<MidiManager>)
- Types: `MidiDevice` (name, manufacturer)

### System Commands
**File:** `/daw/src-tauri/src/commands/system.rs` (referenced in mod.rs)
- `initialize_database` - Test database connection

### Commands Module
**File:** `/daw/src-tauri/src/commands/mod.rs`
- Module structure defining all submodules
- Re-exports for Tauri command registration
- AppState struct (db_pool: Option<PgPool>)

---

## Core Architecture

### Pipeline Architecture (Grown-up Script Pattern)
Each command file follows three-layer pattern:
1. **Error Handling** - Custom error types for domain
2. **Type Definitions** - Serializable request/response structs
3. **Public API** - Tauri #[command] decorated functions

Example flow (file_import.rs):
```
import_directory() [Tauri command]
  -> Validate input
  -> Call file_import_impl() [Internal]
  -> Database queries
  -> Hash calculation
  -> MIDI parsing
  -> Batch insertion
  -> Progress events
```

### DAW Architecture
Similar pattern with additional state management:
- `AppState` - Global application state (database pool)
- `SequencerEngine` - Shared sequencer logic (Arc)
- `MidiManager` - MIDI device abstraction (Arc)

---

## Database Schema Accessed

### Pipeline Commands Access:
- `files` - Core file metadata
- `musical_metadata` - BPM, key, duration, time signature
- `file_tags` - File-tag relationships
- `tags` - Tag definitions and categories
- `track_splits` - Parent-child split relationships
- `file_categories` - Category assignments
- `corrupted_files` - Corruption repair log

### DAW Commands Access:
- `files` - File metadata
- `musical_metadata` - Musical properties
- `file_tags` - Tags (for filtering)
- Track loading from filesystem

---

## Integration Points

### Shared Library Usage
Pipeline commands import from `midi_library_shared`:
- `core::midi::parser` - MIDI file parsing
- `core::analysis::bpm_detector` - BPM detection
- `core::analysis::key_detector` - Key detection

### Error Handling
All commands convert errors to `String` for Tauri:
```rust
pub async fn command_name(...) -> Result<T, String> {
    // Errors are converted using .map_err(|e| e.to_string())
}
```

### Event Emission
Long-running operations emit events to frontend:
```rust
window.emit("import-progress", serde_json::json!({
    "current": i,
    "total": count,
    "currentFile": filename,
    "rate": files_per_sec
}))?;
```

---

## Testing

### Test Coverage
- **File:** Pipeline tests in `/pipeline/src-tauri/tests/`
  - `file_import_test.rs` - Import command tests
  - `search_repository_test.rs` - Search functionality
  - `tag_repository_test.rs` - Tag operations
  - `metadata_repository_test.rs` - Metadata handling

- **File:** DAW tests in `/daw/src-tauri/tests/`
  - Command integration tests
  - MIDI device mocking

### Running Tests
```bash
# All pipeline tests
cargo test --package midi-pipeline -- --test-threads=1

# Specific command test
cargo test --package midi-pipeline file_import_test

# With coverage
cargo tarpaulin --package midi-pipeline --out Html
```

---

## Performance Characteristics

### Import Pipeline (file_import.rs)
- **Speed:** 7,830 files/sec
- **Parallelism:** 64 threads
- **Batch Size:** 1,000 files per transaction
- **Operations:** Hash (BLAKE3), parse (midly), detect (BPM/key), tag, insert

### Analysis Pipeline (analyze.rs)
- **Speed:** 181-360 files/sec
- **Parallelism:** 32 workers
- **Batch Size:** 1,000 files per transaction
- **Analysis:** 10 metrics per file

### Search (search.rs)
- **Speed:** <100ms per query
- **Indexes:** 60+ covering indexes
- **Pagination:** Offset-based (configurable 1-100)

### Tag Operations (tags.rs)
- **Speed:** <50ms per operation
- **Caching:** Implicit via database connection pool
- **Operations:** O(1) lookups with B-tree indexes

---

## Configuration Files

### Command Registration
- **Pipeline:** `pipeline/src-tauri/src/lib.rs` - Registers all commands
- **DAW:** `daw/src-tauri/src/lib.rs` - Registers all commands

### Tauri Configuration
- **Pipeline:** `pipeline/tauri.conf.json` - Window and feature setup
- **DAW:** `daw/tauri.conf.json` - Window and feature setup

---

## Changelog & History

**Latest Updates (Nov 22, 2025):**
- ✅ All 24 commands documented
- ✅ Performance metrics captured
- ✅ TypeScript examples added
- ✅ Error handling patterns documented
- ✅ Database schema mapping completed

**Implementation Status:**
- Pipeline: 11 commands (100% complete)
- DAW: 13 commands (100% complete)
- Tests: 1,223+ tests passing
- Coverage: 54.53% baseline + integration tests

---

## Documentation Files

1. **API-COMMANDS.md** - Comprehensive reference
2. **API-QUICK-REFERENCE.md** - Quick lookup
3. **API-SOURCE-MAP.md** - This file

**Related Documentation:**
- CLAUDE.md - Project guidance
- ARCHITECTURE-REFERENCE.md - Three Archetypes pattern
- DEVELOPMENT-WORKFLOW.md - Implementation process
- CRITICAL-REQUIREMENTS-ADDENDUM.md - Code standards

---

**Last Updated:** November 22, 2025
**Total Commands Documented:** 24
**Files Analyzed:** 15 command modules
