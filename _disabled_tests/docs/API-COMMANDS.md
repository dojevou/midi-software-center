# MIDI Software Center - Tauri Commands API Reference

Complete documentation of all Tauri commands for Pipeline and DAW components.

---

## Pipeline Commands

### Import Operations

#### `import_single_file`
Imports a single MIDI file with auto-tagging and metadata detection.

**Parameters:**
- `file_path: String` - Path to MIDI file
- `category: Option<String>` - Optional category tag

**Returns:**
```json
{
  "id": i64,
  "filename": String,
  "original_filename": String,
  "filepath": String,
  "content_hash": String,
  "file_size_bytes": i64,
  "bpm": Option<f64>,
  "key_signature": Option<String>
}
```

**Performance:** ~1 file/operation (includes hash, MIDI parsing, tagging)

---

#### `import_directory`
Imports all MIDI files from directory (recursive or shallow) with parallel processing.

**Parameters:**
- `directory_path: String` - Directory containing MIDI files
- `recursive: bool` - Recursively search subdirectories
- `category: Option<String>` - Category for all files
- `_window: Window` - UI event emitter (internal)

**Returns:**
```json
{
  "total_files": usize,
  "imported": usize,
  "skipped": usize,
  "errors": Vec<String>,
  "duration_secs": f64,
  "rate": f64
}
```

**Performance:** 7,830 files/sec (45x faster than baseline)
- Emits `import-progress` events (throttled to 10 files)

---

#### `import_archive_collection`
Processes entire collections of nested zip archives, extracting and importing all MIDI files.

**Parameters:**
- `collection_path: String` - Directory containing zip archives

**Returns:**
```json
{
  "total_archives": usize,
  "total_files_imported": usize,
  "total_files_skipped": usize,
  "total_errors": usize,
  "duration_secs": f64,
  "archives_processed": [
    {
      "archive_name": String,
      "midi_files_found": usize,
      "files_imported": usize,
      "success": bool,
      "error_message": Option<String>
    }
  ]
}
```

**Features:**
- Recursive archive extraction (10 levels deep)
- Automatic MIDI file discovery
- Emits `archive-progress` events

---

### Analysis Operations

#### `start_analysis`
Analyzes all unanalyzed MIDI files (BPM, key, chords, articulation, structure).

**Parameters:**
- `state: State` - Application state with database
- `window: Window` - UI event emitter

**Returns:**
```json
{
  "total_files": usize,
  "analyzed": usize,
  "skipped": usize,
  "errors": Vec<String>,
  "duration_secs": f64,
  "rate": f64
}
```

**Analysis Performed:**
1. BPM Detection (30-300 range)
2. Key Signature Detection (24 keys)
3. Time Signature Extraction
4. Note/Pitch Analysis
5. Polyphony Detection
6. Chord Analysis (progressions, 7th/extended)
7. Drum Analysis (48 GM types)
8. Controller Analysis (CC messages)
9. Articulation Analysis (legato, staccato, velocity)
10. Structure Analysis (form detection)

**Performance:** 181-360 files/sec (6.8x faster than 2min baseline)
- Emits `analysis-progress` events (every 10 files)
- Processes 1,000 files per batch
- 32 concurrent workers

---

### Track Splitting Operations

#### `split_and_import`
Splits multi-track MIDI files into individual single-track files with auto-repair.

**Parameters:**
- `file_id: i64` - Database ID of parent file
- `output_dir: PathBuf` - Directory for split files
- `pool: &PgPool` - Database connection

**Returns:**
```json
{
  "split_file_ids": Vec<i64>,
  "tracks_split": usize,
  "output_dir": String
}
```

**Features:**
- Automatic MIDI corruption repair (99.5% success)
- Production layer filename generation
- Metadata preservation per track
- Relationship tracking in `track_splits` table

---

### Search Operations

#### `search_files`
Advanced search with filters and pagination.

**Parameters:**
- `query: String` - Search query (filename/path substring)
- `filters: SearchFilters` - Filter object:
  - `category: Option<String>`
  - `min_bpm: Option<f64>`
  - `max_bpm: Option<f64>`
  - `key_signature: Option<String>`
- `page: i32` - Page number (1-indexed)
- `page_size: i32` - Results per page (1-100)

**Returns:**
```json
{
  "items": [
    {
      "id": i64,
      "filename": String,
      "filepath": String,
      "bpm": Option<f64>,
      "key_signature": Option<String>,
      "duration_seconds": Option<f64>,
      "category": Option<String>
    }
  ],
  "total_count": i64,
  "page": i32,
  "page_size": i32,
  "total_pages": i32
}
```

**Query Examples:**
```typescript
// Search by name
await invoke('search_files', {
  query: 'vengeance',
  filters: {},
  page: 1,
  pageSize: 50
});

// BPM range search
await invoke('search_files', {
  query: '',
  filters: { minBpm: 118, maxBpm: 122, keySignature: 'C' },
  page: 1,
  pageSize: 50
});
```

---

### Tag Operations

#### `get_file_tags`
Gets all tags associated with a file.

**Parameters:**
- `file_id: i64` - Database file ID

**Returns:**
```json
[
  {
    "id": i32,
    "name": String,
    "category": Option<String>,
    "usage_count": i32
  }
]
```

---

#### `get_popular_tags`
Gets most-used tags for tag cloud display.

**Parameters:**
- `limit: Option<i32>` - Max tags to return (default: 50)

**Returns:** Array of tag objects (same structure as `get_file_tags`)

---

#### `search_tags`
Searches tags by prefix (for autocomplete).

**Parameters:**
- `query: String` - Search prefix
- `limit: Option<i32>` - Max results (default: 10)

**Returns:** Array of matching tags

---

#### `get_tag_categories`
Gets all unique tag categories.

**Parameters:** None

**Returns:**
```json
["drums", "bass", "synth", "genre", ...]
```

---

### Statistics Operations

#### `get_category_stats`
File count breakdown by category.

**Parameters:** None

**Returns:**
```json
{
  "drums": 150,
  "bass": 200,
  "synth": 100,
  ...
}
```

---

#### `get_manufacturer_stats`
File count breakdown by manufacturer (from metadata).

**Parameters:** None

**Returns:** Map of manufacturer names to counts

---

#### `get_key_signature_stats`
File count breakdown by key signature.

**Parameters:** None

**Returns:** Map of key signatures to counts

---

#### `get_recently_added_count`
Count of files added in last 7 days.

**Parameters:** None

**Returns:** `i64` - File count

---

---

## DAW Commands

### Database Search

#### `database_search`
Searches MIDI files with BPM, key, and tag filters.

**Parameters:**
- `filters: SearchFilters`:
  - `query: Option<String>` - Filename/tag search
  - `bpm_min: Option<f32>`
  - `bpm_max: Option<f32>`
  - `key: Option<String>`
  - `tag: Option<String>`
  - `limit: Option<i64>` (default: 50)
  - `offset: Option<i64>` (default: 0)

**Returns:**
```json
{
  "files": [
    {
      "id": i64,
      "file_path": String,
      "file_name": String,
      "bpm": f32,
      "key_signature": String,
      "tags": Vec<String>,
      "duration": f32,
      "track_count": i32,
      "file_size": i64,
      "created_at": DateTime<Utc>,
      "updated_at": DateTime<Utc>
    }
  ],
  "total_count": i64
}
```

---

### Sequencer Operations

#### `start_sequencer`
Starts sequencer playback from current position.

**Parameters:** None
**Returns:** `Result<(), String>`

---

#### `stop_sequencer`
Stops playback and resets position to bar 0.

**Parameters:** None
**Returns:** `Result<(), String>`

---

#### `pause_sequencer`
Pauses playback (maintains position).

**Parameters:** None
**Returns:** `Result<(), String>`

---

#### `resume_sequencer`
Resumes from paused state.

**Parameters:** None
**Returns:** `Result<(), String>`

---

#### `get_playback_position`
Gets current playback position.

**Parameters:** None

**Returns:**
```json
{
  "bar": u32,
  "beat": u32,
  "tick": u32
}
```

---

#### `seek_position`
Seeks to specific bar/beat position.

**Parameters:**
- `bar: u32` - Bar number (0-indexed)
- `beat: u32` - Beat within bar (0-indexed)

**Returns:** `Result<(), String>`

---

#### `set_tempo`
Sets global tempo (BPM).

**Parameters:**
- `bpm: f32` - Tempo in BPM

**Returns:** `Result<(), String>`

---

#### `get_tempo`
Gets current tempo.

**Parameters:** None
**Returns:** `f32`

---

#### `add_track`
Adds a MIDI file as a track in sequencer.

**Parameters:**
- `file_id: i32` - Database file ID
- `channel: u8` - MIDI channel (0-15)

**Returns:**
```json
{
  "id": i32,
  "name": String,
  "channel": u8,
  "volume": f32,
  "pan": f32,
  "mute": bool,
  "solo": bool,
  "is_armed": bool
}
```

---

### MIDI Device Operations

#### `midi_list_devices`
Lists all available MIDI output devices.

**Parameters:** None

**Returns:**
```json
[
  {
    "name": String,
    "manufacturer": Option<String>
  }
]
```

---

#### `midi_connect`
Connects to a MIDI device by name.

**Parameters:**
- `device_name: String` - Name from device list

**Returns:** `Result<(), String>`

---

#### `midi_disconnect`
Disconnects from current MIDI device.

**Parameters:** None
**Returns:** `Result<(), String>`

---

#### `midi_is_connected`
Checks if MIDI device is connected.

**Parameters:** None
**Returns:** `bool`

---

#### `midi_get_current_device`
Gets info about currently connected device.

**Parameters:** None

**Returns:**
```json
{
  "name": String,
  "manufacturer": Option<String>
}
```

---

#### `midi_send_test_note`
Sends test note to verify MIDI connection (500ms duration).

**Parameters:**
- `channel: u8` - MIDI channel (0-15)
- `note: u8` - Note number (0-127)
- `velocity: u8` - Velocity (1-127)

**Returns:** `Result<(), String>`

---

### System Operations

#### `initialize_database`
Tests database connection.

**Parameters:** None
**Returns:** `Result<(), String>`

---

---

## Common Patterns

### Error Handling
All commands return `Result<T, String>` with error descriptions:
```typescript
try {
  const result = await invoke('import_single_file', {
    filePath: '/path/to/file.mid'
  });
} catch (error) {
  console.error('Import failed:', error);
}
```

### Progress Events
Long-running operations emit progress events:
```typescript
// Listen to import progress
listen('import-progress', (event) => {
  const { current, total, currentFile, rate } = event.payload;
  console.log(`${current}/${total} at ${rate.toFixed(0)} files/sec`);
});

// Listen to analysis progress
listen('analysis-progress', (event) => {
  const { current, total, rate, etaSeconds } = event.payload;
});

// Listen to archive progress
listen('archive-progress', (event) => {
  const { current, total, archiveName } = event.payload;
});
```

### Frontend Usage Examples

**Import and analyze complete pipeline:**
```typescript
import { invoke } from '@tauri-apps/api/core';

// 1. Import files
const importResult = await invoke('import_directory', {
  directoryPath: '/home/user/midi-files',
  recursive: true,
  category: 'Drums'
});

console.log(`Imported ${importResult.imported} files`);

// 2. Analyze files
const analysisResult = await invoke('start_analysis');
console.log(`Analyzed ${analysisResult.analyzed} files`);

// 3. Search files
const searchResult = await invoke('search_files', {
  query: 'kick',
  filters: { minBpm: 120, maxBpm: 130 },
  page: 1,
  pageSize: 50
});

console.log(`Found ${searchResult.totalCount} results`);
```

**Sequencer and MIDI operations:**
```typescript
// Connect to MIDI device
await invoke('midi_connect', { deviceName: 'Akai Force' });

// Add track to sequencer
const track = await invoke('add_track', {
  fileId: 123,
  channel: 0
});

// Control playback
await invoke('set_tempo', { bpm: 120 });
await invoke('start_sequencer');

// Send test note
await invoke('midi_send_test_note', {
  channel: 0,
  note: 60,
  velocity: 100
});
```

---

## Performance Metrics

| Operation | Speed | Concurrency | Notes |
|-----------|-------|-------------|-------|
| Import | 7,830 files/sec | 64 threads | BLAKE3 hashing, batch inserts |
| Analysis | 181-360 files/sec | 32 workers | Full 10-point analysis |
| Archive Import | Variable | Sequential | Depends on extraction speed |
| Search | <100ms | Single | Indexed database queries |
| Tag Operations | <50ms | Single | Memory cached |

---

## Type Definitions

### SearchFilters (Pipeline)
```rust
pub struct SearchFilters {
    pub category: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key_signature: Option<String>,
}
```

### TagResponse
```rust
pub struct TagResponse {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}
```

### AnalyzedFile (Internal)
Comprehensive analysis result containing:
- Tempo/BPM with variation detection
- Key signature with confidence
- Time signature
- Duration in seconds and ticks
- Note statistics (count, unique pitches, range, velocity)
- Polyphony metrics (max, average)
- Chord analysis (progression, types, complexity)
- Drum pattern analysis
- Melody detection
- Controller data (CC analysis)
- Articulation metrics (legato, staccato, timing)
- Musical structure/form analysis
- Complexity score (0-100)

---

## Database Integration

All commands interact with PostgreSQL 16:
- **Tables:** 15+ tables (files, musical_metadata, tags, track_splits, etc.)
- **Indexes:** 60+ indexes for performance
- **Connection Pool:** Dynamic sizing (1-64 connections)
- **Batch Operations:** 1,000 files per transaction

---

## Notes

- All timestamps are UTC (PostgreSQL `TIMESTAMP WITH TIME ZONE`)
- BPM values are stored as `NUMERIC(6,2)` for precision
- File paths are absolute (full filesystem paths)
- MIDI parsing uses `midly` crate (zero-copy)
- Analysis uses multi-threaded processing with tokio
- All operations are async and non-blocking

---

**Last Updated:** November 22, 2025
**API Version:** 1.0
**Stack:** Rust + Tauri 2.7 + PostgreSQL 16
