# MIDI Software Center - API Documentation Index

Complete guide to all Tauri command documentation and usage patterns.

---

## Documentation Files

### 1. **API-QUICK-REFERENCE.md** (START HERE)
Quick lookup for developers who need to call commands.
- Command summary tables
- TypeScript code examples
- Response structure examples
- Performance metrics
- Best for: "How do I use X command?"

### 2. **API-COMMANDS.md** (COMPREHENSIVE REFERENCE)
Complete documentation of all 24 commands with full details.
- Command signatures and parameters
- Parameter descriptions
- Return types with JSON examples
- Error handling
- Frontend usage examples
- Performance notes
- Database integration
- Best for: "I need all the details about command X"

### 3. **API-SOURCE-MAP.md** (IMPLEMENTATION DETAILS)
Maps commands to source files and implementation details.
- File locations for each command
- Function signatures
- Helper function documentation
- Database schema accessed
- Integration points
- Testing information
- Best for: "Where is command X implemented?"

---

## Command Organization

### Pipeline Commands (11 total)

**Import Operations** - Load MIDI files from disk
- `import_single_file` - One file
- `import_directory` - Batch with recursion
- `import_archive_collection` - Archives with extraction

**Analysis** - Musical analysis
- `start_analysis` - 10-point analysis (BPM, key, chords, etc.)

**Splitting** - Track separation
- `split_and_import` - Multi-track to single-track conversion

**Search** - Query and discovery
- `search_files` - Keyword and filter search

**Tags** - Categorization
- `get_file_tags` - Get file tags
- `get_popular_tags` - Popular tags
- `search_tags` - Tag autocomplete
- `get_tag_categories` - All categories

**Statistics** - Analytics
- `get_category_stats` - Count by category
- `get_manufacturer_stats` - Count by manufacturer
- `get_key_signature_stats` - Count by key
- `get_recently_added_count` - Recent files

### DAW Commands (13 total)

**Database** - File discovery
- `database_search` - Search with filters

**Sequencer** - Playback control
- `start_sequencer`, `stop_sequencer`, `pause_sequencer`, `resume_sequencer`
- `get_playback_position`, `seek_position`
- `set_tempo`, `get_tempo`

**MIDI Device** - Hardware integration
- `midi_list_devices` - Available devices
- `midi_connect`, `midi_disconnect` - Connection control
- `midi_is_connected` - Status
- `midi_get_current_device` - Current device info
- `midi_send_test_note` - Verify connection

**System** - Infrastructure
- `initialize_database` - Database health check

---

## Common Workflows

### 1. Import and Analyze Collection

```typescript
import { invoke, listen } from '@tauri-apps/api/core';

// Listen to progress
listen('import-progress', (event) => {
  console.log(`${event.payload.current}/${event.payload.total}`);
});

// Import files
const importResult = await invoke('import_directory', {
  directoryPath: '/path/to/files',
  recursive: true,
  category: 'Drums'
});
console.log(`Imported ${importResult.imported} files`);

// Analyze
const analysisResult = await invoke('start_analysis');
console.log(`Analyzed ${analysisResult.analyzed} files`);
```

**See:** API-COMMANDS.md → "Import Operations" and "Analysis Operations"

### 2. Search and Filter

```typescript
// Text search with BPM filter
const results = await invoke('search_files', {
  query: 'kick',
  filters: {
    minBpm: 120,
    maxBpm: 130,
    keySignature: 'C'
  },
  page: 1,
  pageSize: 50
});

console.log(`Found ${results.totalCount} results`);
```

**See:** API-COMMANDS.md → "Search Operations"

### 3. MIDI Hardware Setup

```typescript
// List devices
const devices = await invoke('midi_list_devices');
console.log('Available MIDI devices:', devices);

// Connect
await invoke('midi_connect', { deviceName: 'Akai Force' });

// Verify
await invoke('midi_send_test_note', {
  channel: 0,
  note: 60,
  velocity: 100
});

console.log('MIDI connected and verified');
```

**See:** API-COMMANDS.md → "MIDI Device Operations"

### 4. Playback Control

```typescript
// Set tempo
await invoke('set_tempo', { bpm: 120 });

// Load track
const track = await invoke('add_track', {
  fileId: 123,
  channel: 0
});

// Control playback
await invoke('start_sequencer');
await invoke('seek_position', { bar: 4, beat: 0 });
await invoke('pause_sequencer');
```

**See:** API-COMMANDS.md → "Sequencer Operations"

### 5. Statistics and Insights

```typescript
// Get breakdowns
const categoryStats = await invoke('get_category_stats');
const keyStats = await invoke('get_key_signature_stats');
const recentCount = await invoke('get_recently_added_count');

console.log('Category distribution:', categoryStats);
console.log('Files by key:', keyStats);
console.log('Recently added:', recentCount);
```

**See:** API-COMMANDS.md → "Statistics Operations"

---

## Performance Quick Reference

| Operation | Speed | Notes |
|-----------|-------|-------|
| Import | 7,830 files/sec | Parallel with hashing |
| Analysis | 181-360 files/sec | 10-point analysis |
| Search | <100ms | Indexed queries |
| Tag Lookup | <50ms | Database cached |
| MIDI Connect | <500ms | Hardware dependent |
| Sequencer Start | <100ms | In-memory |

---

## Error Handling Pattern

All commands return `Result<T, String>`:

```typescript
try {
  const result = await invoke('command_name', { /* params */ });
  console.log('Success:', result);
} catch (error) {
  console.error('Error:', error);
  // Handle error - typically string message
}
```

Common errors:
- "Database pool not initialized" - Connection issue
- "File not found: ..." - Missing file
- "Failed to parse MIDI: ..." - Corrupt file
- "Search error: ..." - Query issue

**See:** API-COMMANDS.md → "Common Patterns"

---

## Type Definitions

### Request Types
- `SearchFilters` - Query parameters (category, BPM range, key)
- `SearchFilters` (DAW) - Includes tag filtering
- `PlaybackPosition` - bar, beat, tick
- `MidiDevice` - name, manufacturer

### Response Types
- `ImportResult` - total_files, imported, skipped, errors, duration_secs, rate
- `AnalysisResult` - analyzed, skipped, errors, duration_secs, rate
- `SearchResults` - items, total_count, page, total_pages
- `TagResponse` - id, name, category, usage_count
- `Track` - id, name, channel, volume, pan, mute, solo, is_armed

**See:** API-COMMANDS.md → "Type Definitions"

---

## Database Integration

All commands interact with PostgreSQL 16:

**Key Tables:**
- `files` - 1.72M+ MIDI files
- `musical_metadata` - BPM, key, duration, time signature
- `file_tags` - Tag relationships (many-to-many)
- `tags` - Tag definitions (1,640+ tags)
- `track_splits` - Parent-child split relationships

**Performance:**
- 60+ indexes for fast queries
- Connection pool: 1-64 dynamic connections
- Batch operations: 1,000 files per transaction

**See:** API-SOURCE-MAP.md → "Database Schema Accessed"

---

## Frontend Integration

### Tauri IPC
Commands are called using `invoke()` from `@tauri-apps/api/core`:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('command_name', { param1, param2 });
```

### Event Listeners
Long-running operations emit progress events:

```typescript
import { listen } from '@tauri-apps/api/event';

listen('import-progress', (event) => {
  // Handle progress update
});

listen('analysis-progress', (event) => {
  // Handle analysis update
});
```

### Error Handling
Errors are thrown as strings (Tauri limitation):

```typescript
try {
  await invoke('command');
} catch (error: unknown) {
  const message = typeof error === 'string' ? error : String(error);
  console.error(message);
}
```

---

## Stack Information

**Technologies:**
- **Backend:** Rust 1.70+ with Tauri 2.7
- **Database:** PostgreSQL 16 + Meilisearch 1.5
- **Frontend:** TypeScript, Svelte, Vite
- **IPC:** Tauri commands (JSON serialization)
- **Async:** tokio runtime

**Key Crates:**
- `midly` - MIDI parsing (zero-copy)
- `sqlx` - Database with compile-time verification
- `tokio` - Async runtime
- `serde` - Serialization
- `parking_lot` - Fast locking primitives

---

## Getting Help

### For Quick Reference
→ See **API-QUICK-REFERENCE.md**

### For Command Details
→ See **API-COMMANDS.md**

### For Implementation Details
→ See **API-SOURCE-MAP.md**

### For Architecture
→ See **ARCHITECTURE-REFERENCE.md** in docs/

### For Development
→ See **DEVELOPMENT-WORKFLOW.md** in docs/

### For Code Standards
→ See **CRITICAL-REQUIREMENTS-ADDENDUM.md** in docs/

---

## Updates and Maintenance

**Last Updated:** November 22, 2025

**Maintenance Notes:**
- New commands should follow Grown-up Script pattern
- Add tests before implementation
- Update all three documentation files
- Include TypeScript examples
- Document performance characteristics

**Related Issues:**
- None currently blocking
- All 24 commands fully documented
- 1,223+ tests passing
- 0 production errors

---

## Quick Links

- **Quick Start:** API-QUICK-REFERENCE.md
- **Full Details:** API-COMMANDS.md
- **Source Code:** API-SOURCE-MAP.md
- **Architecture:** docs/ARCHITECTURE-REFERENCE.md
- **Testing:** docs/TEST-COVERAGE-PLAN.md
- **Project Guidance:** CLAUDE.md

---

**Total Commands Documented:** 24
**Documentation Files:** 4
**Example Code Snippets:** 15+
**TypeScript Examples:** 20+

Ready for production use. All commands tested and documented.
