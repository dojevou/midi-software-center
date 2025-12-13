# MIDI Software Center - API Quick Reference

Concise command listing and usage patterns.

## Pipeline Commands (11 total)

### Import (3)
| Command | Purpose | Speed |
|---------|---------|-------|
| `import_single_file` | Import one MIDI file | 1 file/op |
| `import_directory` | Import directory recursively | 7,830 files/sec |
| `import_archive_collection` | Extract and import zip archives | Variable |

### Analysis (1)
| Command | Purpose | Speed |
|---------|---------|-------|
| `start_analysis` | Full 10-point MIDI analysis | 181-360 files/sec |

### Splitting (1)
| Command | Purpose | Speed |
|---------|---------|-------|
| `split_and_import` | Split multi-track files | Depends on tracks |

### Search (1)
| Command | Purpose | Notes |
|---------|---------|-------|
| `search_files` | Keyword + filter search | <100ms, paginated |

### Tags (4)
| Command | Purpose | Returns |
|---------|---------|---------|
| `get_file_tags` | Tags for file | Array<Tag> |
| `get_popular_tags` | Most-used tags | Array<Tag> |
| `search_tags` | Autocomplete search | Array<Tag> |
| `get_tag_categories` | All categories | Array<String> |

### Statistics (4)
| Command | Purpose | Returns |
|---------|---------|---------|
| `get_category_stats` | File count/category | Map<String, i64> |
| `get_manufacturer_stats` | File count/manufacturer | Map<String, i64> |
| `get_key_signature_stats` | File count/key | Map<String, i64> |
| `get_recently_added_count` | Last 7 days | i64 |

---

## DAW Commands (13 total)

### Database (1)
| Command | Purpose | Speed |
|---------|---------|-------|
| `database_search` | BPM/key/tag search | <100ms |

### Sequencer (8)
| Command | Args | Returns |
|---------|------|---------|
| `start_sequencer` | None | Result |
| `stop_sequencer` | None | Result |
| `pause_sequencer` | None | Result |
| `resume_sequencer` | None | Result |
| `get_playback_position` | None | { bar, beat, tick } |
| `seek_position` | bar, beat | Result |
| `set_tempo` | bpm | Result |
| `get_tempo` | None | f32 |

### MIDI Device (6)
| Command | Purpose | Returns |
|---------|---------|---------|
| `midi_list_devices` | List devices | Array<Device> |
| `midi_connect` | Connect by name | Result |
| `midi_disconnect` | Disconnect | Result |
| `midi_is_connected` | Status check | bool |
| `midi_get_current_device` | Current device | Option<Device> |
| `midi_send_test_note` | Verify connection | Result |

---

## TypeScript Usage

### Import & Analyze
```typescript
import { invoke } from '@tauri-apps/api/core';

const imported = await invoke('import_directory', {
  directoryPath: '/path',
  recursive: true,
  category: 'Drums'
});

const analyzed = await invoke('start_analysis');
```

### Search
```typescript
const results = await invoke('search_files', {
  query: 'kick',
  filters: { minBpm: 120, maxBpm: 130 },
  page: 1,
  pageSize: 50
});
```

### Sequencer
```typescript
await invoke('midi_connect', { deviceName: 'Akai Force' });
await invoke('set_tempo', { bpm: 120 });
await invoke('start_sequencer');
await invoke('midi_send_test_note', {
  channel: 0, note: 60, velocity: 100
});
```

### Progress Events
```typescript
import { listen } from '@tauri-apps/api/event';

listen('import-progress', (event) => {
  const { current, total, rate } = event.payload;
  console.log(`${current}/${total} @ ${rate.toFixed(0)} files/sec`);
});
```

---

## Response Structures

### FileMetadata
```json
{
  "id": i64,
  "filename": String,
  "filepath": String,
  "content_hash": String,
  "file_size_bytes": i64,
  "bpm": Option<f64>,
  "key_signature": Option<String>
}
```

### SearchResults
```json
{
  "items": [{ id, filename, filepath, bpm, key, duration, category }],
  "total_count": i64,
  "page": i32,
  "total_pages": i32
}
```

### TagResponse
```json
{
  "id": i32,
  "name": String,
  "category": Option<String>,
  "usage_count": i32
}
```

### PlaybackPosition
```json
{
  "bar": u32,
  "beat": u32,
  "tick": u32
}
```

### MidiDevice
```json
{
  "name": String,
  "manufacturer": Option<String>
}
```

---

## Error Handling

All commands return `Result<T, String>`:

```typescript
try {
  const result = await invoke('import_directory', {
    directoryPath: '/path'
  });
  console.log(result);
} catch (error) {
  console.error('Error:', error);
}
```

---

## Performance Summary

| Operation | Throughput |
|-----------|-----------|
| Import Files | 7,830/sec |
| Analyze Files | 181-360/sec |
| Search Query | <100ms |
| Tag Lookup | <50ms |
| MIDI Analysis | 10 metrics |

---

## File Locations

All command implementations:
- **Pipeline:** `/pipeline/src-tauri/src/commands/*.rs`
- **DAW:** `/daw/src-tauri/src/commands/*.rs`

---

**For detailed documentation, see:** `/docs/API-COMMANDS.md`
