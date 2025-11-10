# MIDI Metadata Extraction Enhancement Plan

**Date:** 2025-11-09
**Status:** Proposed
**Priority:** High

## Problem Statement

Currently, MIDI metadata extraction happens in **TWO separate phases**:
1. **Import Phase** (file_import.rs): Extracts only basic file info (filename, path, size)
2. **Analysis Phase** (analyze.rs): Extracts musical metadata (BPM, key, notes)

**Issues:**
- Metadata unavailable until analysis runs (could be hours/days later)
- Text metadata (track names, copyright) never extracted
- Tempo/key/time signature changes detected but not stored
- User sees empty metadata immediately after import

## Proposed Solution

Create a **unified metadata extractor** that runs during import to extract:

### Phase 1: Immediate Extraction (During Import)
Extract from MIDI file content (no analysis required):

1. **Header Metadata**
   - Format type (0, 1, 2)
   - Number of tracks
   - Ticks per quarter note
   - Total duration (seconds + ticks)

2. **Text Metadata** (from MIDI Text events)
   - Track names
   - Instrument names
   - Copyright notices
   - Lyrics
   - Markers
   - Cue points

3. **Event Metadata**
   - Time signature (first + changes)
   - Key signature (first + changes)
   - Tempo changes (all with timestamps)
   - Program changes (instrument list)

4. **Simple Statistics**
   - Note count
   - Channel usage (which channels used)
   - Pitch range (min/max note)
   - Has pitch bend / CC messages

### Phase 2: Analysis Extraction (Later)
Keep existing analysis.rs for complex processing:
- BPM detection (with confidence)
- Key detection (with confidence)
- Complexity scoring
- Advanced note analysis

## Implementation Plan

### Step 1: Create Metadata Extractor Module
**File:** `shared/rust/src/core/midi/metadata_extractor.rs`

```rust
pub struct MidiMetadata {
    // Header
    pub format: u16,
    pub num_tracks: u16,
    pub ticks_per_quarter_note: u16,

    // Duration
    pub duration_seconds: Option<f64>,
    pub duration_ticks: i32,

    // Text metadata
    pub track_names: Vec<String>,
    pub instrument_names: Vec<String>,
    pub copyright: Option<String>,
    pub lyrics: Vec<String>,
    pub markers: Vec<String>,
    pub cue_points: Vec<String>,

    // Time signature
    pub time_signature: (u8, u8),
    pub time_signature_changes: Vec<TimeSignatureChange>,

    // Key signature
    pub key_signature: Option<KeySignature>,
    pub key_changes: Vec<KeyChange>,

    // Tempo
    pub initial_tempo_bpm: Option<f64>,
    pub tempo_changes: Vec<TempoChange>,

    // Statistics
    pub note_count: i32,
    pub channels_used: Vec<u8>,
    pub pitch_range: (u8, u8),
    pub has_pitch_bend: bool,
    pub has_cc_messages: bool,
}

pub fn extract_metadata(midi_file: &MidiFile) -> MidiMetadata { ... }
```

### Step 2: Update Database Schema
**File:** `database/migrations/009_import_metadata.sql`

Add columns to `files` table:
- `track_names TEXT[]`
- `copyright TEXT`
- `initial_tempo_bpm REAL`
- `initial_time_signature TEXT`
- `initial_key_signature TEXT`

Add columns to `musical_metadata` table:
- Update `tempo_changes JSONB`
- Update `key_changes JSONB`
- Update `time_signature_changes JSONB`

### Step 3: Integrate with file_import.rs
Update import logic to extract metadata immediately:

```rust
// Phase 5: Extract metadata (NEW)
let metadata = extract_metadata(&midi_file);

// Phase 6: Insert into database with metadata
repo.insert_with_metadata(&file_record, &metadata).await?;
```

### Step 4: Update Frontend
Add metadata display in DatabaseWindow.svelte:
- Show track names
- Show copyright
- Show initial tempo/key/time signature
- Show channel usage

## Benefits

1. **Immediate Metadata Availability**: Users see metadata right after import
2. **Complete Text Extraction**: Track names, copyright always available
3. **Change Detection**: All tempo/key/time signature changes stored
4. **Better Search**: Can search by track names, copyright, etc.
5. **Separation of Concerns**: Simple extraction vs complex analysis

## Performance Impact

- **Minimal**: Metadata extraction is fast (< 1ms per file)
- **No Extra I/O**: File already loaded for import
- **Parallel Safe**: Each file extracted independently

## Timeline

- ✅ Step 1: Design (this document) - 30 min
- ⏳ Step 2: Implement metadata_extractor.rs - 2 hours
- ⏳ Step 3: Create database migration - 30 min
- ⏳ Step 4: Integrate with file_import.rs - 1 hour
- ⏳ Step 5: Add tests - 1 hour
- ⏳ Step 6: Update frontend display - 1 hour

**Total: ~6 hours of work**

## Next Steps

1. Get approval on approach
2. Create metadata_extractor.rs module
3. Write comprehensive tests
4. Integrate with file_import.rs
5. Test with production MIDI files
