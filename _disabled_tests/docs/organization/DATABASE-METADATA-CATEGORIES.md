# Database Metadata Categories

**Complete list of all metadata we collect for each MIDI file**
**Date:** 2025-11-19

---

## 📊 METADATA OVERVIEW

We extract **67 metadata fields** across 2 main tables:
- **`files` table**: 35 fields (file info, paths, basic MIDI data)
- **`musical_metadata` table**: 32 fields (deep musical analysis)

---

## 1️⃣ FILES TABLE (35 fields)

### File Identity & Storage
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `id` | bigint | Unique file ID | Auto-generated |
| `filename` | text | Current filename | Import |
| `original_filename` | text | Original filename before any changes | Import |
| `filepath` | text | Full absolute path to file | Import |
| `content_hash` | bytea | BLAKE3 hash for deduplication | Calculated |
| `file_size_bytes` | bigint | File size in bytes | Import |

### MIDI Structure
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `format` | smallint | MIDI format (0, 1, or 2) | MIDI Parser |
| `num_tracks` | smallint | Number of tracks | MIDI Parser |
| `ticks_per_quarter_note` | integer | MIDI timing resolution | MIDI Parser |
| `duration_seconds` | numeric(10,3) | Total duration in seconds | Calculated |
| `duration_ticks` | bigint | Total duration in MIDI ticks | Calculated |

### Track Information
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `is_multi_track` | boolean | Is this a multi-track MIDI? | Analysis |
| `parent_file_id` | bigint | ID of parent (if this is a split track) | Split Process |
| `track_number` | smallint | Track number (for splits) | Split Process |
| `total_tracks` | smallint | Total tracks in parent file | Split Process |
| `track_names` | text[] | Array of track names from MIDI | MIDI Parser |

### Filename Metadata Extraction
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `filename_bpm` | real | BPM extracted from filename (30-300) | Filename Parser |
| `filename_key` | text | Musical key from filename | Filename Parser |
| `filename_genres` | text[] | Genre tags from filename | Filename Parser |
| `structure_tags` | text[] | Structure tags (verse, chorus, etc.) | Filename Parser |

### Organization & Classification
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `parent_folder` | text | Parent directory name | Path Analysis |
| `folder_tags` | text[] | Tags from folder hierarchy | Path Analysis |
| `collection_name` | text | Collection/pack name | Path Analysis |
| `manufacturer` | text | Manufacturer/brand | Path Analysis |
| `metadata_source` | text | Source of metadata (analyzed/filename/both) | System |

### Additional MIDI Metadata
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `instrument_names_text` | text[] | Instrument names from MIDI | MIDI Parser |
| `copyright` | text | Copyright text from MIDI | MIDI Parser |
| `markers` | text[] | MIDI markers | MIDI Parser |
| `lyrics` | text[] | Lyrics from MIDI | MIDI Parser |

### System Fields
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `created_at` | timestamptz | When file was imported | Auto |
| `updated_at` | timestamptz | Last update time | Auto |
| `analyzed_at` | timestamptz | When analysis completed | Analysis |
| `import_batch_id` | uuid | Import batch identifier | Import |
| `search_vector` | tsvector | Full-text search index | Auto-generated |

---

## 2️⃣ MUSICAL_METADATA TABLE (32 fields)

### Tempo & Timing
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `file_id` | bigint | Reference to files table | System |
| `bpm` | numeric | Detected BPM (tempo) | BPM Detector |
| `bpm_confidence` | real | Confidence score (0.0-1.0) | BPM Detector |
| `has_tempo_changes` | boolean | Does tempo vary? | Analysis |
| `tempo_changes` | jsonb | Array of tempo change points | Analysis |

### Key & Harmony
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `key_signature` | text | Detected musical key (C, Dm, F#, etc.) | Key Detector |
| `key_confidence` | real | Confidence score (0.0-1.0) | Key Detector |
| `has_key_changes` | boolean | Does key modulate? | Analysis |
| `key_changes` | jsonb | Array of key change points | Analysis |

### Time Signature
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `time_signature_numerator` | smallint | Top number (4 in 4/4) | MIDI Parser |
| `time_signature_denominator` | smallint | Bottom number (4 in 4/4) | MIDI Parser |
| `has_time_signature_changes` | boolean | Does time signature change? | Analysis |
| `time_signature_changes` | jsonb | Array of time sig changes | Analysis |

### Note Statistics
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `total_notes` | integer | Total number of notes | Note Analysis |
| `unique_pitches` | integer | Number of different pitches used | Note Analysis |
| `pitch_range_min` | smallint | Lowest MIDI note (0-127) | Note Analysis |
| `pitch_range_max` | smallint | Highest MIDI note (0-127) | Note Analysis |
| `melodic_range` | smallint | Pitch range (max - min) | Calculated |
| `avg_velocity` | real | Average note velocity | Note Analysis |
| `note_density` | real | Notes per second | Calculated |

### Polyphony & Texture
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `polyphony_max` | smallint | Max simultaneous notes | Polyphony Analysis |
| `polyphony_avg` | real | Average simultaneous notes | Polyphony Analysis |
| `is_monophonic` | boolean | Single-note melody | Analysis |
| `is_polyphonic` | boolean | Multi-note chords | Analysis |
| `is_percussive` | boolean | Drum/percussion file | Analysis |

### Harmonic Analysis
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `has_chords` | boolean | Contains chordal content | Chord Analyzer |
| `has_melody` | boolean | Contains melodic content | Analysis |
| `chord_progression` | jsonb | Detected chord progression | Chord Analyzer |
| `chord_types` | text[] | Types of chords (maj, min, 7th, etc.) | Chord Analyzer |
| `has_seventh_chords` | boolean | Contains 7th chords | Chord Analyzer |
| `has_extended_chords` | boolean | Contains 9th, 11th, 13th chords | Chord Analyzer |
| `chord_change_rate` | real | Chords per second | Chord Analyzer |
| `chord_complexity` | real | Complexity score | Chord Analyzer |
| `chord_complexity_score` | real | Alternative complexity metric | Chord Analyzer |

### System
| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `created_at` | timestamptz | When analysis was performed | Auto |

---

## 3️⃣ RELATED TABLES (Additional Metadata)

### FILE_TAGS (Many-to-Many)
- `file_id` - Reference to file
- `tag_id` - Reference to tag
- Tags include: genre, instrument, mood, style, pattern type

### FILE_INSTRUMENTS (Detailed Instrument Info)
- `file_id` - Reference to file
- `channel` - MIDI channel (0-15)
- `program_number` - GM program (0-127)
- `program_name` - Instrument name
- `instrument_family` - Piano, Bass, Drums, etc.
- `instrument_type` - Specific type
- `note_count` - Notes for this instrument
- `is_primary` - Primary instrument flag
- `avg_velocity` - Average velocity
- `pitch_range_low` - Lowest note
- `pitch_range_high` - Highest note

### FILE_CATEGORIES
- `file_id` - Reference to file
- `category_type` - Type of category (genre, style, etc.)
- `category_value` - Category value
- `confidence` - Confidence score

### HARMONIC_PATTERNS
- `file_id` - Reference to file
- `pattern_type` - Type of harmonic pattern
- `start_tick` - When pattern starts
- `end_tick` - When pattern ends
- `chord_sequence` - Sequence of chords
- `confidence` - Pattern detection confidence

### MELODIC_PATTERNS
- `file_id` - Reference to file
- `pattern_type` - Type of melodic pattern
- `start_tick` - When pattern starts
- `end_tick` - When pattern ends
- `note_sequence` - Sequence of notes
- `confidence` - Pattern detection confidence

### RHYTHM_PATTERNS
- `file_id` - Reference to file
- `pattern_type` - Groove, fill, intro, etc.
- `start_tick` - When pattern starts
- `end_tick` - When pattern ends
- `note_sequence` - Rhythmic pattern
- `confidence` - Pattern detection confidence

---

## 📈 METADATA EXTRACTION PIPELINE

### Phase 1: Import
**Extracts:** File identity, MIDI structure, filename metadata
- File path, size, hash
- MIDI format, tracks, timing
- BPM/key/genre from filename
- Track names, copyright, markers

### Phase 2: Sanitize (Optional)
**Modifies:** Filename, filepath
- Clean special characters
- Normalize extensions
- Replace spaces

### Phase 3: Split (Optional)
**Extracts:** Track relationships
- Parent file ID
- Track number
- Total tracks

### Phase 4: Analysis
**Extracts:** Deep musical metadata (all 32 musical_metadata fields)
- BPM detection (30-300 range)
- Key detection (24 keys)
- Chord analysis
- Note statistics
- Polyphony analysis
- Drum pattern detection (NEW)

### Phase 5: Tag Generation
**Extracts:** Genre, instrument, mood tags
- Auto-generated from analysis
- Based on folders/filenames
- 500+ possible tags

### Phase 6: Rename (Optional)
**Uses:** All collected metadata
- Generates MPC-compatible names
- Organizes into folder structure

---

## 🎯 METADATA USAGE

### Search & Discovery
- Full-text search (`search_vector`)
- Tag filtering (`file_tags`)
- Folder/collection browsing (`folder_tags`, `collection_name`)

### Organization
- Folder structure (`parent_folder`)
- Collection grouping (`collection_name`)
- Category classification (`file_categories`)

### Musical Compatibility
- Key matching (`key_signature`)
- BPM matching (`bpm`)
- Time signature (`time_signature_*`)
- Instrument type (`file_instruments`)

### Quality Assessment
- Confidence scores (`bpm_confidence`, `key_confidence`)
- Complexity metrics (`chord_complexity`)
- Note density (`note_density`)

### MPC Export
- Filename generation (BPM + key + tags)
- Folder organization (instrument + genre)
- Bar calculation (duration + BPM + time sig)

---

## 📊 STATISTICS

**Total Metadata Fields:** 67+
- Core file metadata: 35 fields
- Musical analysis: 32 fields
- Relationships: 6+ tables
- Tags: Unlimited (many-to-many)

**Storage per File:** ~2-5 KB metadata
**For 4.3M files:** ~10-20 GB metadata

**Analysis Time:**
- Import: <1ms per file
- Full analysis: 5-50ms per file (depends on complexity)

---

## ✅ VALIDATION

All metadata is:
- ✅ Validated on insert (CHECK constraints)
- ✅ Indexed for fast queries
- ✅ Normalized (separate tables for relationships)
- ✅ Backed up automatically
- ✅ Version controlled (migrations)

**Last Updated:** 2025-11-19
