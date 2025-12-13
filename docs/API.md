# MIDI Software Center API Reference

This document describes the Tauri IPC commands and repository methods available in the MIDI Software Center.

## Table of Contents

- [Tauri Commands](#tauri-commands)
  - [Pipeline Commands](#pipeline-commands)
  - [DAW Commands](#daw-commands)
- [Repository Methods](#repository-methods)
- [Data Types](#data-types)

---

## Tauri Commands

All commands are invoked via Tauri's `invoke()` function from the frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('command_name', { param1: value1 });
```

### Pipeline Commands

#### File Operations

##### `test_db_connection`
Test database connectivity.

```typescript
const connected: boolean = await invoke('test_db_connection');
```

**Returns:** `boolean` - True if database is connected

---

##### `get_file_count`
Get total number of files in the database.

```typescript
const count: number = await invoke('get_file_count');
```

**Returns:** `number` - Total file count

---

##### `get_file_details`
Get detailed information about a specific file.

```typescript
const file: MidiFile = await invoke('get_file_details', { fileId: 12345 });
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `fileId` | `number` | File ID to retrieve |

**Returns:** `MidiFile` - Complete file details including metadata

---

##### `get_file`
Alias for `get_file_details`.

```typescript
const file: MidiFile = await invoke('get_file', { fileId: 12345 });
```

---

##### `list_files`
List files with pagination.

```typescript
const files: MidiFile[] = await invoke('list_files', {
  limit: 50,
  offset: 0
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `limit` | `number?` | Maximum files to return (default: 50) |
| `offset` | `number?` | Number of files to skip (default: 0) |

**Returns:** `MidiFile[]` - Array of file objects

---

##### `get_files_by_category`
Get files filtered by category.

```typescript
const files: MidiFile[] = await invoke('get_files_by_category', {
  category: 'DRUMS',
  limit: 100,
  offset: 0
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `category` | `string` | Category name (DRUMS, BASS, KEYS, etc.) |
| `limit` | `number?` | Maximum files to return |
| `offset` | `number?` | Number of files to skip |

**Returns:** `MidiFile[]` - Filtered file array

---

##### `get_recent_files`
Get recently added files.

```typescript
const files: MidiFile[] = await invoke('get_recent_files', {
  limit: 20
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `limit` | `number?` | Maximum files to return (default: 20) |

**Returns:** `MidiFile[]` - Recently added files

---

##### `delete_file`
Delete a file from the database.

```typescript
await invoke('delete_file', { fileId: 12345 });
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `fileId` | `number` | File ID to delete |

---

#### Import Operations

##### `import_single_file`
Import a single MIDI file.

```typescript
const result = await invoke('import_single_file', {
  path: '/path/to/file.mid'
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Absolute path to MIDI file |

**Returns:** Import result with file ID

---

##### `import_directory`
Import all MIDI files from a directory.

```typescript
const result = await invoke('import_directory', {
  path: '/path/to/midi/folder',
  recursive: true
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `path` | `string` | Directory path |
| `recursive` | `boolean?` | Include subdirectories (default: true) |

**Returns:** Import statistics (files processed, duplicates, errors)

---

#### Search Operations

##### `search_files`
Search files with multiple criteria.

```typescript
const results: SearchResult[] = await invoke('search_files', {
  query: 'drum loop',
  bpmMin: 118,
  bpmMax: 122,
  key: 'C',
  category: 'DRUMS',
  limit: 50,
  offset: 0
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `query` | `string?` | Text search query |
| `bpmMin` | `number?` | Minimum BPM |
| `bpmMax` | `number?` | Maximum BPM |
| `key` | `string?` | Musical key (C, Cm, G, etc.) |
| `category` | `string?` | Category filter |
| `limit` | `number?` | Maximum results |
| `offset` | `number?` | Pagination offset |

**Returns:** `SearchResult[]` - Matching files with relevance scores

---

#### Analysis Operations

##### `analyze_file`
Run musical analysis on a file.

```typescript
const analysis = await invoke('analyze_file', { fileId: 12345 });
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `fileId` | `number` | File ID to analyze |

**Returns:** Analysis results (BPM, key, chords, etc.)

---

##### `analyze_batch`
Analyze multiple files.

```typescript
const results = await invoke('analyze_batch', {
  fileIds: [1, 2, 3, 4, 5]
});
```

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `fileIds` | `number[]` | Array of file IDs |

**Returns:** Batch analysis results

---

#### Statistics

##### `get_category_stats`
Get file count by category.

```typescript
const stats: Record<string, number> = await invoke('get_category_stats');
// { "DRUMS": 50000, "BASS": 12000, "KEYS": 8000, ... }
```

**Returns:** `Record<string, number>` - Category name to count mapping

---

#### Progress Tracking

##### `get_import_progress`
Get current import progress.

```typescript
const progress: ImportProgress = await invoke('get_import_progress');
```

**Returns:** `ImportProgress` - Current import state

---

### DAW Commands

#### Piano Roll

##### `get_notes`
Get notes for a track.

```typescript
const notes: Note[] = await invoke('get_notes', { trackId: 1 });
```

---

##### `add_note`
Add a note to a track.

```typescript
await invoke('add_note', {
  trackId: 1,
  pitch: 60,
  velocity: 100,
  startTick: 0,
  duration: 480
});
```

---

#### MIDI Hardware

##### `list_midi_devices`
List connected MIDI devices.

```typescript
const devices: MidiDevice[] = await invoke('list_midi_devices');
```

---

##### `connect_midi_device`
Connect to a MIDI device.

```typescript
await invoke('connect_midi_device', { deviceId: 'device-uuid' });
```

---

#### Undo/Redo

##### `undo`
Undo the last action.

```typescript
await invoke('undo');
```

---

##### `redo`
Redo the last undone action.

```typescript
await invoke('redo');
```

---

## Repository Methods

For Rust code, the repository pattern provides database access:

### FileRepository

```rust
use midi_pipeline::db::repositories::FileRepository;

// Insert a file
let file_id = file_repo.insert(&file).await?;

// Find by hash (deduplication)
let existing = file_repo.find_by_hash(&blake3_hash).await?;

// Search files
let results = file_repo.search("drum loop", 50, 0).await?;

// Get by ID
let file = file_repo.get_by_id(12345).await?;

// Delete
file_repo.delete(12345).await?;
```

### MetadataRepository

```rust
use midi_pipeline::db::repositories::MetadataRepository;

// Get metadata
let metadata = metadata_repo.get_by_file_id(file_id).await?;

// Update BPM
metadata_repo.update_bpm(file_id, 120.0).await?;

// Update key
metadata_repo.update_key(file_id, "Cm").await?;

// Batch insert
metadata_repo.insert_batch(&metadata_list).await?;
```

### TagRepository

```rust
use midi_pipeline::db::repositories::TagRepository;

// Add tags to file
tag_repo.add_tags(file_id, &["drums", "loop", "120bpm"]).await?;

// Get files by tag
let files = tag_repo.get_files_by_tag("drums").await?;

// Get tags for file
let tags = tag_repo.get_tags_for_file(file_id).await?;

// Create tag
tag_repo.create_tag("new-tag", "INSTRUMENT").await?;
```

### SearchRepository

```rust
use midi_pipeline::db::repositories::SearchRepository;

// Full-text search
let results = search_repo.search("jazz piano", 50, 0).await?;

// Advanced search
let results = search_repo.advanced_search(SearchParams {
    query: Some("loop"),
    bpm_min: Some(118.0),
    bpm_max: Some(122.0),
    key: Some("C"),
    category: Some("DRUMS"),
    limit: 50,
    offset: 0,
}).await?;
```

---

## Data Types

### MidiFile

```typescript
interface MidiFile {
  id: number;
  filepath: string;
  filename: string;
  hash: string;
  size: number;
  duration?: number;
  bpm?: number;
  key_signature?: string;
  time_signature?: string;
  track_count?: number;
  category?: string;
  tags?: string[];
  created_at: string;
  updated_at: string;
}
```

### SearchResult

```typescript
interface SearchResult {
  file: MidiFile;
  score: number;
  highlights?: string[];
}
```

### ImportProgress

```typescript
interface ImportProgress {
  total_files: number;
  processed: number;
  successful: number;
  failed: number;
  duplicates: number;
  current_file?: string;
  status: 'idle' | 'running' | 'completed' | 'error';
  error_message?: string;
}
```

### Note

```typescript
interface Note {
  id: number;
  pitch: number;        // MIDI pitch (0-127)
  velocity: number;     // MIDI velocity (0-127)
  start_tick: number;   // Start position in ticks
  duration: number;     // Duration in ticks
  channel: number;      // MIDI channel (0-15)
}
```

### MidiDevice

```typescript
interface MidiDevice {
  id: string;
  name: string;
  device_type: 'input' | 'output' | 'both';
  connected: boolean;
  port_number: number;
}
```

---

## Error Handling

All commands return errors as strings. Use try/catch for error handling:

```typescript
try {
  const file = await invoke('get_file_details', { fileId: 99999 });
} catch (error) {
  console.error('Failed to get file:', error);
  // error is a string describing the failure
}
```

Common error types:
- `"File not found"` - Requested file ID doesn't exist
- `"Database error: ..."` - Database query failed
- `"Invalid parameter: ..."` - Invalid input parameter
- `"Permission denied"` - Insufficient permissions

---

## Performance Notes

1. **Batch Operations**: Use batch methods for multiple files to reduce database round-trips
2. **Pagination**: Always use `limit` and `offset` for large result sets
3. **Search Indexing**: Full-text search uses Meilisearch for fast queries
4. **Connection Pooling**: Database connections are pooled (max 34 connections)

---

## Version History

- **1.0.0** - Initial API release
