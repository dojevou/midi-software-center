## API Reference

> **Status:** Draft - To be completed when Streams A-F finish
> **Last Updated:** 2025-12-17

This document provides a complete reference for all Tauri commands exposed to the frontend.

### Table of Contents

1. [VIP3 Browser Commands](#vip3-browser-commands)
2. [Mixer Commands](#mixer-commands)
3. [Automation Commands](#automation-commands)
4. [Project Commands](#project-commands)
5. [Collection Commands](#collection-commands)
6. [Sequencer Commands](#sequencer-commands)
7. [Type Definitions](#type-definitions)
8. [Error Handling](#error-handling)

---

## VIP3 Browser Commands

### `vip3_search`

Search for MIDI files with filters.

**Signature:**
```typescript
async function vip3_search(
  filters: VIP3Filters,
  offset: number,
  limit: number
): Promise<VIP3SearchResults>
```

**Parameters:**
- `filters`: Filter criteria (see [VIP3Filters](#vip3filters))
- `offset`: Pagination offset (0-based)
- `limit`: Maximum results to return (1-1000)

**Returns:** [VIP3SearchResults](#vip3searchresults)

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/core';

const results = await invoke('vip3_search', {
  filters: {
    instruments: ['piano', 'drums'],
    bpm_min: 120,
    bpm_max: 140,
    key: 'C',
  },
  offset: 0,
  limit: 50,
});

console.log(`Found ${results.total_count} files`);
results.files.forEach(file => {
  console.log(`${file.filename} - ${file.bpm} BPM`);
});
```

**Performance:** Typical response < 100ms

---

### `get_vip3_filter_counts`

> **Stream A Required**

Get counts for each filter category based on active filters.

**Signature:**
```typescript
async function get_vip3_filter_counts(
  active_filters: VIP3Filters
): Promise<VIP3FilterCounts>
```

**Parameters:**
- `active_filters`: Currently active filters

**Returns:** [VIP3FilterCounts](#vip3filtercounts)

**Example:**
```typescript
const counts = await invoke('get_vip3_filter_counts', {
  active_filters: {
    instruments: ['piano'],
  },
});

console.log('Folder counts:', counts.folders);
// { 1: 45, 2: 23, 3: 0 }  (folder_id: file_count)

console.log('BPM range counts:', counts.bpm_ranges);
// { '<100': 12, '100-120': 23, '120-140': 45, ... }
```

**Performance:** Target < 50ms, uses caching (5s TTL)

---

### `get_file_by_id`

Get detailed metadata for a single file.

**Signature:**
```typescript
async function get_file_by_id(file_id: number): Promise<FileMetadata>
```

**Parameters:**
- `file_id`: Database ID of the file

**Returns:** [FileMetadata](#filemetadata)

**Example:**
```typescript
const file = await invoke('get_file_by_id', { file_id: 12345 });
console.log(file.filename, file.bpm, file.key);
```

---

## Mixer Commands

> **Stream B Required**

### `mixer_set_gain`

Set track gain in decibels.

**Signature:**
```typescript
async function mixer_set_gain(
  track_id: number,
  gain_db: number
): Promise<void>
```

**Parameters:**
- `track_id`: Track identifier (1-128)
- `gain_db`: Gain value in dB (-60.0 to +12.0)

**Example:**
```typescript
await invoke('mixer_set_gain', { track_id: 1, gain_db: -6.0 });
```

**Performance:** < 10ms typical

---

### `mixer_set_pan`

Set track pan position.

**Signature:**
```typescript
async function mixer_set_pan(
  track_id: number,
  pan: number
): Promise<void>
```

**Parameters:**
- `track_id`: Track identifier
- `pan`: Pan position (-1.0 = hard left, 0.0 = center, +1.0 = hard right)

**Example:**
```typescript
await invoke('mixer_set_pan', { track_id: 1, pan: 0.5 }); // 50% right
```

---

### `mixer_toggle_mute`

Toggle track mute state.

**Signature:**
```typescript
async function mixer_toggle_mute(track_id: number): Promise<boolean>
```

**Parameters:**
- `track_id`: Track identifier

**Returns:** New mute state (`true` = muted)

**Example:**
```typescript
const is_muted = await invoke('mixer_toggle_mute', { track_id: 1 });
console.log(`Track ${is_muted ? 'muted' : 'unmuted'}`);
```

---

### `mixer_toggle_solo`

Toggle track solo state.

**Signature:**
```typescript
async function mixer_toggle_solo(track_id: number): Promise<boolean>
```

**Parameters:**
- `track_id`: Track identifier

**Returns:** New solo state (`true` = soloed)

**Notes:**
- By default, soloing a track unsolo all others (exclusive solo)
- Additive solo should be handled via `mixer_set_solo_additive`

---

### `mixer_add_effect`

Add effect to track's effect chain.

**Signature:**
```typescript
async function mixer_add_effect(
  track_id: number,
  effect_type: string,
  params: EffectParams
): Promise<number>
```

**Parameters:**
- `track_id`: Track identifier
- `effect_type`: Effect type name ("eq", "compressor", "reverb", "delay")
- `params`: Effect-specific parameters

**Returns:** Effect ID (for later reference)

**Example:**
```typescript
const effect_id = await invoke('mixer_add_effect', {
  track_id: 1,
  effect_type: 'reverb',
  params: {
    room_size: 0.7,
    damping: 0.5,
    wet_level: 0.3,
  },
});
```

---

### `mixer_get_meters`

Get current VU meter levels for all tracks.

**Signature:**
```typescript
async function mixer_get_meters(): Promise<MeterData[]>
```

**Returns:** Array of [MeterData](#meterdata)

**Example:**
```typescript
const meters = await invoke('mixer_get_meters');
meters.forEach(meter => {
  console.log(`Track ${meter.track_id}: Peak ${meter.peak_left}dB`);
});
```

**Performance:** < 1ms typical (optimized for 60 Hz polling)

---

### TODO: Additional Mixer Commands

- `mixer_remove_effect`
- `mixer_update_effect`
- `mixer_reorder_effects`
- `mixer_set_send`
- `mixer_create_bus`
- `mixer_route_track`
- `mixer_save_preset`
- `mixer_load_preset`
- (See Stream B for full list of 30 commands)

---

## Automation Commands

> **Stream D Required**

### `automation_set_mode`

Set automation mode for a parameter.

**Signature:**
```typescript
async function automation_set_mode(
  track_id: number,
  parameter: AutomationParameter,
  mode: AutomationMode
): Promise<void>
```

**Parameters:**
- `track_id`: Track identifier
- `parameter`: Parameter to automate (see [AutomationParameter](#automationparameter))
- `mode`: Recording mode (see [AutomationMode](#automationmode))

**Example:**
```typescript
await invoke('automation_set_mode', {
  track_id: 1,
  parameter: { type: 'Gain' },
  mode: 'Write',
});
```

---

### `automation_add_point`

Add automation point.

**Signature:**
```typescript
async function automation_add_point(
  track_id: number,
  parameter: AutomationParameter,
  time: number,
  value: number
): Promise<void>
```

**Parameters:**
- `track_id`: Track identifier
- `parameter`: Parameter being automated
- `time`: Time in beats (0.0+)
- `value`: Normalized value (0.0 to 1.0)

**Example:**
```typescript
await invoke('automation_add_point', {
  track_id: 1,
  parameter: { type: 'Gain' },
  time: 4.0,      // Beat 4
  value: 0.75,    // 75% of range
});
```

---

### `automation_get_lane`

Get all automation data for a parameter.

**Signature:**
```typescript
async function automation_get_lane(
  track_id: number,
  parameter: AutomationParameter
): Promise<AutomationLane>
```

**Returns:** [AutomationLane](#automationlane)

---

### TODO: Additional Automation Commands

- `automation_delete_point`
- `automation_move_point`
- `automation_clear_lane`
- `automation_playback_at_time`

---

## Project Commands

> **Stream E Required**

### `project_create`

Create a new project.

**Signature:**
```typescript
async function project_create(
  name: string,
  bpm: number
): Promise<number>
```

**Parameters:**
- `name`: Project name
- `bpm`: Tempo in beats per minute

**Returns:** Project ID

**Example:**
```typescript
const project_id = await invoke('project_create', {
  name: 'My Beat',
  bpm: 128.0,
});
```

---

### `project_save`

Save project state to database.

**Signature:**
```typescript
async function project_save(project_id: number): Promise<void>
```

**Example:**
```typescript
await invoke('project_save', { project_id: 1 });
```

---

### `project_load`

Load project state.

**Signature:**
```typescript
async function project_load(project_id: number): Promise<Project>
```

**Returns:** [Project](#project)

---

### `project_export`

Export project to MIDI file.

**Signature:**
```typescript
async function project_export(
  project_id: number,
  path: string
): Promise<void>
```

**Parameters:**
- `project_id`: Project to export
- `path`: Output file path

**Example:**
```typescript
await invoke('project_export', {
  project_id: 1,
  path: '/home/user/my_beat.mid',
});
```

---

## Collection Commands

> **Stream C Required**

### `vip3_create_collection`

Create a new collection.

**Signature:**
```typescript
async function vip3_create_collection(
  name: string,
  description: string | null
): Promise<number>
```

**Returns:** Collection ID

**Example:**
```typescript
const collection_id = await invoke('vip3_create_collection', {
  name: 'Jazz Essentials',
  description: 'My favorite jazz MIDI files',
});
```

---

### `vip3_add_to_collection`

Add files to a collection.

**Signature:**
```typescript
async function vip3_add_to_collection(
  collection_id: number,
  file_ids: number[]
): Promise<void>
```

**Example:**
```typescript
await invoke('vip3_add_to_collection', {
  collection_id: 1,
  file_ids: [123, 456, 789],
});
```

---

### `vip3_save_search`

Save current filter state as a named search.

**Signature:**
```typescript
async function vip3_save_search(
  name: string,
  filters: VIP3Filters
): Promise<number>
```

**Returns:** Saved search ID

---

## Type Definitions

### VIP3Filters

```typescript
interface VIP3Filters {
  // Folder filtering
  folder_ids?: number[];

  // Instrument filtering
  instruments?: string[];  // Tag names

  // BPM filtering
  bpm_min?: number;
  bpm_max?: number;

  // Key filtering
  key?: string;  // 'C', 'C#', 'D', etc.
  key_mode?: 'major' | 'minor' | null;

  // Time signature
  time_signature?: string;  // '4/4', '3/4', etc.

  // Track count
  track_count_min?: number;
  track_count_max?: number;

  // Additional tags
  timbres?: string[];       // 'bright', 'dark', 'warm', etc.
  styles?: string[];        // 'jazz', 'classical', etc.
  articulations?: string[]; // 'staccato', 'legato', etc.

  // Full-text search (future)
  search_query?: string;
}
```

### VIP3FilterCounts

```typescript
interface VIP3FilterCounts {
  // folder_id -> count
  folders: Record<number, number>;

  // instrument_name -> count
  instruments: Record<string, number>;

  // BPM range -> count
  bpm_ranges: {
    '<100': number;
    '100-120': number;
    '120-140': number;
    '140-160': number;
    '160+': number;
  };

  // key -> count
  keys: Record<string, number>;

  // time_signature -> count
  time_signatures: Record<string, number>;

  // track_count_range -> count
  track_counts: {
    '1': number;
    '2-5': number;
    '6-10': number;
    '10+': number;
  };
}
```

### VIP3SearchResults

```typescript
interface VIP3SearchResults {
  files: FileMetadata[];
  total_count: number;
  offset: number;
  limit: number;
}
```

### FileMetadata

```typescript
interface FileMetadata {
  id: number;
  filename: string;
  file_path: string;
  file_size: number;

  // Musical metadata
  bpm: number | null;
  key: string | null;
  time_signature: string | null;
  duration_seconds: number | null;
  track_count: number | null;

  // Tags
  instrument_tags: string[];
  timbre_tags: string[];
  style_tags: string[];

  // Timestamps
  created_at: string;
  updated_at: string;
}
```

### MeterData

```typescript
interface MeterData {
  track_id: number;
  peak_left: number;   // dB
  peak_right: number;  // dB
  rms_left: number;    // dB
  rms_right: number;   // dB
}
```

### AutomationParameter

```typescript
type AutomationParameter =
  | { type: 'Gain' }
  | { type: 'Pan' }
  | { type: 'Send', send_id: number }
  | { type: 'EffectParam', effect_id: number, param_id: number };
```

### AutomationMode

```typescript
type AutomationMode = 'Off' | 'Read' | 'Write' | 'Latch' | 'Touch';
```

### AutomationLane

```typescript
interface AutomationLane {
  parameter: AutomationParameter;
  points: AutomationPoint[];
  mode: AutomationMode;
}

interface AutomationPoint {
  time: number;      // beats
  value: number;     // normalized 0.0-1.0
  curve: CurveType;
}

type CurveType = 'Linear' | 'Stepped' | 'Bezier' | 'Exponential';
```

### Project

```typescript
interface Project {
  id: number;
  name: string;
  bpm: number;
  time_signature: TimeSignature;
  tracks: TrackState[];
  mixer_state: MixerState;
  automation: Record<number, AutomationLane[]>;
  created_at: string;
  updated_at: string;
}

interface TimeSignature {
  numerator: number;
  denominator: number;
}
```

---

## Error Handling

All commands return `Promise<T>` and may reject with errors.

### Error Format

```typescript
interface TauriError {
  message: string;
  code?: string;
}
```

### Common Error Codes

- `NOT_FOUND`: Resource doesn't exist
- `INVALID_INPUT`: Invalid parameters
- `DATABASE_ERROR`: Database operation failed
- `IO_ERROR`: File system error
- `INTERNAL_ERROR`: Unexpected error

### Example Error Handling

```typescript
try {
  const file = await invoke('get_file_by_id', { file_id: 999999 });
} catch (error) {
  if (error.code === 'NOT_FOUND') {
    console.error('File not found');
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

---

## Performance Notes

### Response Time Targets

| Command | Target | Notes |
|---------|--------|-------|
| `vip3_search` | < 100ms | Depends on result count |
| `get_vip3_filter_counts` | < 50ms | Cached (5s TTL) |
| `mixer_set_gain` | < 10ms | Real-time critical |
| `mixer_get_meters` | < 1ms | Polled at 60 Hz |
| `automation_add_point` | < 10ms | During recording |

### Optimization Tips

1. **Batch operations**: Use bulk commands when available
2. **Debounce filter changes**: Wait 100-200ms before calling `get_vip3_filter_counts`
3. **Pagination**: Use reasonable `limit` values (50-100)
4. **Meter polling**: Request meters only when mixer UI is visible

---

**TODO: Complete remaining sections when all streams finish**
