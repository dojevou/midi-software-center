# MIDI Metadata Extraction System

**Database**: PostgreSQL 16 at localhost:5433/midi_library
**Files Found**: 5,817,980 MIDI files on system
**Date**: 2025-11-11

---

## Overview

The system extracts metadata from **THREE sources**:

1. **Filename Analysis** - Extracted from file/folder names
2. **MIDI Content Analysis** - Extracted from MIDI file data
3. **File System** - Extracted from file paths and properties

---

## Database Structure

### Table 1: `files` (Main File Records)

**Purpose**: Store all MIDI files with basic info + filename metadata

**Columns** (28 total):

#### Basic File Info (9 columns)
```sql
id                      BIGINT PRIMARY KEY
filename                TEXT NOT NULL              -- e.g., "120bpm_Cm_house_loop.mid"
filepath                TEXT NOT NULL UNIQUE       -- Full path
original_filename       TEXT NOT NULL              -- Original name before processing
content_hash            BYTEA NOT NULL             -- BLAKE3 hash for deduplication
file_size_bytes         BIGINT NOT NULL            -- File size
created_at              TIMESTAMPTZ                -- Import timestamp
updated_at              TIMESTAMPTZ                -- Last update
analyzed_at             TIMESTAMPTZ                -- Analysis timestamp
```

#### MIDI Format Info (4 columns)
```sql
format                  SMALLINT                   -- 0, 1, or 2
num_tracks              SMALLINT DEFAULT 1         -- Number of tracks
ticks_per_quarter_note  INTEGER                    -- MIDI timing resolution
duration_seconds        NUMERIC(10,3)              -- Song duration
duration_ticks          BIGINT                     -- Duration in MIDI ticks
```

#### Multi-Track Handling (4 columns)
```sql
is_multi_track          BOOLEAN DEFAULT FALSE      -- Split into tracks?
parent_file_id          BIGINT REFERENCES files    -- Parent if split
track_number            SMALLINT                   -- Track 1, 2, 3...
total_tracks            SMALLINT                   -- Total tracks
```

#### Folder/Path Metadata (3 columns)
```sql
manufacturer            TEXT                       -- e.g., "Vengeance", "Splice"
collection_name         TEXT                       -- e.g., "Essential House Vol 2"
folder_tags             TEXT[]                     -- Array from folder structure
```

#### **NEW: Filename-Extracted Metadata (5 columns)**
```sql
filename_bpm            REAL                       -- BPM from filename (30-300)
filename_key            TEXT                       -- Key from filename ("Cm", "Am")
filename_genres         TEXT[]                     -- ["house", "techno"]
structure_tags          TEXT[]                     -- ["loop", "fill", "intro"]
metadata_source         TEXT                       -- 'analyzed', 'filename', 'both', 'validated'
```

#### Other (3 columns)
```sql
search_vector           TSVECTOR                   -- Full-text search index
import_batch_id         UUID                       -- Batch import tracking
```

---

### Table 2: `musical_metadata` (Rich MIDI Analysis)

**Purpose**: Store detailed musical analysis from MIDI content
**Relationship**: 1-to-1 with `files` table (file_id foreign key)

**Columns** (29 total):

#### Tempo Analysis (4 fields)
```sql
bpm                     NUMERIC(6,2)               -- Analyzed BPM (20-300)
bpm_confidence          REAL                       -- Confidence score 0-1
has_tempo_changes       BOOLEAN                    -- Multiple tempos?
tempo_changes           JSONB                      -- [{tick, bpm}, ...]
```

#### Key Signature (4 fields)
```sql
key_signature           musical_key ENUM           -- 35 possible keys
key_confidence          REAL                       -- Confidence score
has_key_changes         BOOLEAN                    -- Modulations?
key_changes             JSONB                      -- [{tick, key}, ...]
```

#### Time Signature (4 fields)
```sql
time_signature_numerator    SMALLINT DEFAULT 4     -- Top number (4 in 4/4)
time_signature_denominator  SMALLINT DEFAULT 4     -- Bottom number (4 in 4/4)
has_time_signature_changes  BOOLEAN                -- Changes?
time_signature_changes      JSONB                  -- [{tick, num, denom}, ...]
```

#### Note Analysis (7 fields)
```sql
total_notes             INTEGER NOT NULL           -- Total note count
unique_pitches          INTEGER                    -- Unique MIDI notes used
pitch_range_min         SMALLINT                   -- Lowest note (0-127)
pitch_range_max         SMALLINT                   -- Highest note (0-127)
avg_velocity            NUMERIC(5,2)               -- Average note velocity
note_density            NUMERIC(8,3)               -- Notes per second
```

#### Polyphony Analysis (5 fields)
```sql
polyphony_max           SMALLINT                   -- Max simultaneous notes
polyphony_avg           NUMERIC(5,2)               -- Average polyphony
is_monophonic           BOOLEAN DEFAULT FALSE      -- Single-note melody?
is_polyphonic           BOOLEAN DEFAULT TRUE       -- Multiple notes?
```

#### Musical Features (5 fields)
```sql
is_percussive           BOOLEAN DEFAULT FALSE      -- Drums/percussion?
has_chords              BOOLEAN DEFAULT FALSE      -- Contains chords?
chord_complexity        REAL                       -- Chord complexity score
has_melody              BOOLEAN DEFAULT FALSE      -- Has melodic line?
melodic_range           SMALLINT                   -- Melody pitch range
```

---

## Extraction Process

### 1. Filename Extraction

**Pattern Matching Examples**:

```bash
# BPM Extraction
"120_bpm_house.mid"        → filename_bpm = 120
"techno_128bpm.mid"        → filename_bpm = 128
"trap_140.mid"             → filename_bpm = 140

# Key Extraction
"Cm_bass_loop.mid"         → filename_key = "Cm"
"melody_in_Am.mid"         → filename_key = "Am"
"F#_lead.mid"              → filename_key = "F#"

# Genre Extraction
"house_loop_120.mid"       → filename_genres = ["house"]
"techno_dnb_mix.mid"       → filename_genres = ["techno", "dnb"]

# Structure Extraction
"intro_buildup.mid"        → structure_tags = ["intro", "buildup"]
"verse_loop.mid"           → structure_tags = ["verse", "loop"]
"drum_fill_01.mid"         → structure_tags = ["fill"]

# Track Number
"01_kick.mid"              → track_number = 1
"05_bass.mid"              → track_number = 5
```

**Extracted to**: `files` table columns

---

### 2. MIDI Content Analysis

**Analyzed from MIDI Messages**:

```rust
// Tempo Detection
- Reads "set_tempo" meta messages
- Calculates BPM: 60,000,000 / microseconds_per_quarter
- Detects tempo changes throughout file

// Key Detection
- Krumhansl-Schmuckler algorithm
- Analyzes note distribution
- Returns key + confidence score

// Time Signature
- Reads "time_signature" meta messages
- Extracts numerator/denominator
- Tracks changes

// Note Statistics
- Counts all note_on events
- Tracks pitch range (0-127)
- Calculates velocity average
- Measures note density (notes/second)

// Polyphony Analysis
- Tracks simultaneous notes at each tick
- Determines if monophonic vs polyphonic
- Calculates average/max polyphony

// Musical Features
- Chord detection: 3+ notes within 50ms
- Melody detection: single-note passages
- Percussion detection: channel 10 (GM standard)
```

**Extracted to**: `musical_metadata` table

---

### 3. Folder/Path Analysis

**Directory Structure Example**:

```
/Vengeance/Essential_House_Vol_2/Drums/Kicks/
  → manufacturer = "Vengeance"
  → collection_name = "Essential House Vol 2"
  → folder_tags = ["drums", "kicks"]
```

**Extracted to**: `files` table (manufacturer, collection_name, folder_tags)

---

## Import Performance

**Rust Pipeline Performance Targets**:

```
Files         | Target Time    | Target Rate
--------------+----------------+-------------
1,000 files   | < 2 seconds    | 500 files/sec
10,000 files  | ~25 seconds    | 400 files/sec
100,000 files | ~4 minutes     | 400 files/sec
1,000,000     | ~40 minutes    | 400 files/sec
5,817,980     | ~4 hours       | 400 files/sec
```

**Features**:
- Parallel processing with dynamic concurrency
- BLAKE3 hashing (7x faster than SHA-256)
- Batch database inserts (500 files/batch)
- Progress events to UI
- Automatic deduplication

---

## Usage Examples

### Query 1: Find house music loops in C minor at 120-130 BPM

```sql
SELECT f.id, f.filename, mm.bpm, mm.key_signature
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.bpm BETWEEN 120 AND 130
  AND mm.key_signature = 'Cm'
  AND 'house' = ANY(f.filename_genres)
  AND 'loop' = ANY(f.structure_tags);
```

### Query 2: Files with both analyzed AND filename metadata

```sql
SELECT f.id, f.filename,
       mm.bpm AS analyzed_bpm,
       f.filename_bpm,
       mm.key_signature AS analyzed_key,
       f.filename_key
FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE f.metadata_source IN ('both', 'validated');
```

### Query 3: Find drum loops with kicks and snares

```sql
SELECT f.* FROM files f
JOIN musical_metadata mm ON f.id = mm.file_id
WHERE mm.is_percussive = TRUE
  AND 'loop' = ANY(f.structure_tags)
  AND f.filename ILIKE '%kick%'
  AND f.filename ILIKE '%snare%';
```

---

## Metadata Source Tracking

The `metadata_source` column tracks where metadata comes from:

- **'analyzed'**: Only MIDI content analysis (no filename data)
- **'filename'**: Only filename extraction (no MIDI analysis)
- **'both'**: Both sources available
- **'validated'**: Both sources agree (±5 BPM tolerance for BPM, exact match for key)
- **'none'**: No metadata extracted

---

## Complete Extraction Command

```bash
# Import with full extraction
cd /home/dojevou/projects/midi-software-center

# Start pipeline UI
cd pipeline && pnpm dev

# Or use CLI import
cargo run --package midi-pipeline --release -- \
  import \
  --directory /path/to/midi/files \
  --batch-size 500 \
  --concurrency auto
```

---

## Summary Statistics

**Database Capacity**: Optimized for 3,000,000+ files
**Current System**: 5,817,980 MIDI files found
**Extraction Points**:
- 28 columns in `files` table
- 29 columns in `musical_metadata` table
- **Total: 57 data points per MIDI file**

**Indexes**: 60+ optimized indexes for fast querying
**Full-text Search**: tsvector GIN index on filename + metadata
**Deduplication**: BLAKE3 content hashing
