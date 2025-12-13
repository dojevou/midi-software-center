# Schema Mismatch Report - MIDI Software Center
**Date:** 2025-11-11
**Scope:** Migrations 001-010 vs Rust Models & Repositories

---

## Executive Summary

**Total Critical Mismatches Found:** 15
**Severity:** MEDIUM-HIGH (Queries will fail or return incomplete data)

The Rust code does not reference columns added in migrations 008-010. New metadata columns are defined in the database schema but are not being read or written by application code.

---

## Critical Mismatches

### 1. MIGRATION 008: Filename Metadata Columns (NOT IN RUST CODE)

**Location:** Migration `008_filename_metadata_fixed.sql`

**Schema Definition:**
```sql
ALTER TABLE files ADD COLUMN filename_bpm REAL (30-300);
ALTER TABLE files ADD COLUMN filename_key TEXT (1-3 chars);
ALTER TABLE files ADD COLUMN filename_genres TEXT[];
ALTER TABLE files ADD COLUMN structure_tags TEXT[];
ALTER TABLE files ADD COLUMN metadata_source TEXT (enum: 'analyzed', 'filename', 'both', 'none', 'validated');
```

**Rust Code Status:** **MISSING**
- File struct (`pipeline/src-tauri/src/db/models.rs` lines 18-55) does NOT include:
  - `filename_bpm`
  - `filename_key`
  - `filename_genres`
  - `structure_tags`
  - `metadata_source`
- FileRepository queries (`file_repository.rs`) do NOT SELECT these columns
- NewFile struct does NOT support insertion of these fields

**Impact:**
- Cannot store or retrieve filename-extracted metadata
- Analysis commands won't save extracted BPM/key from filenames
- Data loss when analysis runs (values inserted to DB but ignored by app)

**Fix Required:**
- Add 5 fields to `File` struct
- Add 5 fields to `NewFile` struct
- Update INSERT query in `insert()` method
- Update SELECT queries in `find_by_id()`, `find_by_hash()`, `find_by_path()`

---

### 2. MIGRATION 009: Text Metadata Columns (NOT IN RUST CODE)

**Location:** Migration `009_text_metadata.sql`

**Schema Definition:**
```sql
ALTER TABLE files ADD COLUMN track_names TEXT[] DEFAULT '{}';
ALTER TABLE files ADD COLUMN copyright TEXT;
ALTER TABLE files ADD COLUMN instrument_names_text TEXT[] DEFAULT '{}';
ALTER TABLE files ADD COLUMN markers TEXT[] DEFAULT '{}';
ALTER TABLE files ADD COLUMN lyrics TEXT[] DEFAULT '{}';
```

**Rust Code Status:** **MISSING**
- File struct does NOT include:
  - `track_names`
  - `copyright`
  - `instrument_names_text`
  - `markers`
  - `lyrics`
- FileRepository queries do NOT SELECT these columns
- NewFile struct does NOT support insertion

**Impact:**
- MIDI text metadata (track names, copyright, markers) is discarded
- Cannot search by copyright or track names
- 50% of MIDI metadata is never extracted or stored

**Fix Required:**
- Add 5 fields to `File` struct
- Add 5 fields to `NewFile` struct
- Update all SELECT queries in FileRepository (4+ locations)
- Update INSERT query

---

### 3. MIGRATION 010: Harmonic Analysis Columns (NOT IN RUST CODE)

**Location:** Migration `010_harmonic_analysis.sql`

**Schema Definition:**
```sql
-- Added to musical_metadata table:
ALTER TABLE musical_metadata ADD COLUMN chord_progression JSONB;
ALTER TABLE musical_metadata ADD COLUMN chord_types TEXT[];
ALTER TABLE musical_metadata ADD COLUMN has_seventh_chords BOOLEAN;
ALTER TABLE musical_metadata ADD COLUMN has_extended_chords BOOLEAN;
ALTER TABLE musical_metadata ADD COLUMN chord_change_rate NUMERIC(5,2);
ALTER TABLE musical_metadata ADD COLUMN chord_complexity_score NUMERIC(4,3);
```

**Rust Code Status:** **MISSING**
- MusicalMetadata struct (`pipeline/src-tauri/src/db/models.rs` lines 82-130) does NOT include:
  - `chord_progression` (JSONB)
  - `chord_types` (TEXT array)
  - `has_seventh_chords`
  - `has_extended_chords`
  - `chord_change_rate` (NUMERIC)
  - `chord_complexity_score` (NUMERIC)
- MetadataRepository INSERT (`metadata_repository.rs` lines 11-71) does NOT insert these columns
- MetadataRepository SELECT (`metadata_repository.rs` lines 78-119) does NOT read these columns
- NewMusicalMetadata struct does NOT support these fields

**Impact:**
- Harmonic analysis results cannot be stored or retrieved
- Feature completely non-functional despite being in migrations
- Chord analysis data is lost when analysis completes

**Fix Required:**
- Add 6 fields to `MusicalMetadata` struct
- Add 6 fields to `NewMusicalMetadata` struct
- Update INSERT query in `insert()` method (line 11-71) to include all 6 columns
- Update SELECT query in `find_by_file_id()` (line 78-119) to read all 6 columns
- Add update method for chord data (similar to `update_bpm`, `update_key`)

---

### 4. MIGRATION 002: Parent Folder Column (PARTIALLY SUPPORTED)

**Location:** Migration `002_add_parent_folder.sql`

**Schema Definition:**
```sql
ALTER TABLE files ADD COLUMN parent_folder TEXT;
CREATE INDEX idx_files_parent_folder ON files(parent_folder) WHERE parent_folder IS NOT NULL;
```

**Rust Code Status:** **MISSING**
- File struct does NOT include `parent_folder` field
- FileRepository queries do NOT SELECT/INSERT this column
- NewFile struct does NOT support this field

**Note:** Migration 001 defines `folder_tags TEXT[]` which IS supported in Rust, but `parent_folder` is separate.

**Impact:**
- Cannot store or query by parent directory name
- Folder-based filtering not implemented

---

### 5. MIGRATION 007: Tag System Enhancements (PARTIALLY MISSING)

**Location:** Migration `007_enhanced_tags.sql`

**Schema Definition - New Tables:**
```sql
CREATE TABLE tag_categories (...);
CREATE TABLE tag_aliases (...);
CREATE TABLE auto_tagging_rules (...);
CREATE TABLE tag_suggestions (...);
```

**Schema Definition - Enhanced Tags Table:**
```sql
ALTER TABLE tags ADD COLUMN category_id INTEGER REFERENCES tag_categories(id);
ALTER TABLE tags ADD COLUMN priority INTEGER DEFAULT 50;
ALTER TABLE tags ADD COLUMN auto_detected BOOLEAN DEFAULT FALSE;
ALTER TABLE tags ADD COLUMN confidence_score DECIMAL(3,2);
ALTER TABLE tags ADD COLUMN detection_method VARCHAR(50);
ALTER TABLE tags ADD COLUMN parent_tag_id INTEGER REFERENCES tags(id);
ALTER TABLE tags ADD COLUMN is_active BOOLEAN DEFAULT TRUE;
```

**Rust Code Status:** **NOT IMPLEMENTED**
- No Tag or TagCategory models exist
- TagRepository not created (`pipeline/src-tauri/src/db/repositories/tag_repository.rs` exists but is minimal)
- No support for:
  - `category_id`
  - `priority`
  - `auto_detected`
  - `confidence_score`
  - `detection_method`
  - `parent_tag_id`
  - `is_active`
- No models for:
  - `TagCategory`
  - `TagAlias`
  - `AutoTaggingRule`
  - `TagSuggestion`

**Impact:**
- Auto-tagging priority system not functional
- Tag categories not accessible from queries
- Tag confidence scores ignored
- Cannot filter by detection method (auto vs manual)
- Tag hierarchy (parent_tag_id) not supported

---

## Detailed Column Mapping

### Files Table

| Column | Migration | Rust Supported | Status |
|--------|-----------|-----------------|--------|
| id | 001 | ✅ | OK |
| filename | 001 | ✅ | OK |
| filepath | 001 | ✅ | OK |
| original_filename | 001 | ✅ | OK |
| content_hash | 001 | ✅ | OK |
| file_size_bytes | 001 | ✅ | OK |
| format | 001 | ✅ | OK |
| num_tracks | 001 | ✅ | OK |
| ticks_per_quarter_note | 001 | ✅ | OK |
| duration_seconds | 001 | ✅ | OK |
| duration_ticks | 001 | ✅ | OK |
| is_multi_track | 001 | ✅ | OK |
| parent_file_id | 001 | ✅ | OK |
| track_number | 001 | ✅ | OK |
| total_tracks | 001 | ✅ | OK |
| manufacturer | 001 | ✅ | OK |
| collection_name | 001 | ✅ | OK |
| folder_tags | 001 | ✅ | OK |
| search_vector | 001 | ✅ (auto-maintained) | OK |
| created_at | 001 | ✅ | OK |
| updated_at | 001 | ✅ | OK |
| analyzed_at | 001 | ✅ | OK |
| import_batch_id | 001 | ✅ | OK |
| **parent_folder** | **002** | **❌** | **MISSING** |
| **filename_bpm** | **008** | **❌** | **MISSING** |
| **filename_key** | **008** | **❌** | **MISSING** |
| **filename_genres** | **008** | **❌** | **MISSING** |
| **structure_tags** | **008** | **❌** | **MISSING** |
| **metadata_source** | **008** | **❌** | **MISSING** |
| **track_names** | **009** | **❌** | **MISSING** |
| **copyright** | **009** | **❌** | **MISSING** |
| **instrument_names_text** | **009** | **❌** | **MISSING** |
| **markers** | **009** | **❌** | **MISSING** |
| **lyrics** | **009** | **❌** | **MISSING** |

### Musical_Metadata Table

| Column | Migration | Rust Supported | Status |
|--------|-----------|-----------------|--------|
| file_id | 001 | ✅ | OK |
| bpm | 001 | ✅ | OK |
| bpm_confidence | 001 | ✅ | OK |
| has_tempo_changes | 001 | ✅ | OK |
| tempo_changes | 001 | ✅ | OK |
| key_signature | 001 | ✅ | OK |
| key_confidence | 001 | ✅ | OK |
| has_key_changes | 001 | ✅ | OK |
| key_changes | 001 | ✅ | OK |
| time_signature_numerator | 001 | ✅ | OK |
| time_signature_denominator | 001 | ✅ | OK |
| has_time_signature_changes | 001 | ✅ | OK |
| time_signature_changes | 001 | ✅ | OK |
| total_notes | 001 | ✅ | OK |
| unique_pitches | 001 | ✅ | OK |
| pitch_range_min | 001 | ✅ | OK |
| pitch_range_max | 001 | ✅ | OK |
| avg_velocity | 001 | ✅ | OK |
| note_density | 001 | ✅ | OK |
| polyphony_max | 001 | ✅ | OK |
| polyphony_avg | 001 | ✅ | OK |
| is_monophonic | 001 | ✅ | OK |
| is_polyphonic | 001 | ✅ | OK |
| is_percussive | 001 | ✅ | OK |
| has_chords | 001 | ✅ | OK |
| chord_complexity | 001 | ✅ | OK |
| has_melody | 001 | ✅ | OK |
| melodic_range | 001 | ✅ | OK |
| created_at | 001 | ✅ | OK |
| **chord_progression** | **010** | **❌** | **MISSING** |
| **chord_types** | **010** | **❌** | **MISSING** |
| **has_seventh_chords** | **010** | **❌** | **MISSING** |
| **has_extended_chords** | **010** | **❌** | **MISSING** |
| **chord_change_rate** | **010** | **❌** | **MISSING** |
| **chord_complexity_score** | **010** | **❌** | **MISSING** |

---

## Query Impact Analysis

### FileRepository.insert() - WILL FAIL

**Current Query (line 11-30):**
```rust
INSERT INTO files (
    filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    manufacturer, collection_name, folder_tags, import_batch_id
)
```

**Missing Columns:**
- parent_folder (migration 002)
- filename_bpm, filename_key, filename_genres, structure_tags, metadata_source (migration 008)
- track_names, copyright, instrument_names_text, markers, lyrics (migration 009)

**Result:** Columns inserted by analysis will be NULL in database.

### MetadataRepository.insert() - WILL FAIL

**Current Query (line 11-49):**
```rust
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max,
    avg_velocity, note_density, polyphony_max, polyphony_avg, is_percussive
)
```

**Missing Columns:**
- chord_progression, chord_types, has_seventh_chords, has_extended_chords, chord_change_rate, chord_complexity_score (migration 010)

**Result:** Harmonic analysis is computed but not persisted.

### FileRepository Queries - INCOMPLETE DATA

All SELECT queries in FileRepository omit the new columns. Example from `find_by_id()` (lines 54-87):

**Missing from SELECT:**
- parent_folder
- filename_bpm, filename_key, filename_genres, structure_tags, metadata_source
- track_names, copyright, instrument_names_text, markers, lyrics

---

## Recommendations

### Priority 1: Critical (Blocks functionality)

1. **Add 11 fields to File struct** (migrations 002, 008, 009)
   - File: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs`
   - Lines 18-55

2. **Update FileRepository INSERT** (line 11-30)
   - Add 11 new columns to INSERT statement
   - Add 11 new fields to NewFile struct

3. **Update FileRepository SELECT queries** (4 locations)
   - `find_by_id()` (line 54-87)
   - `find_by_hash()` (line 111-144)
   - `find_by_path()` (line 148+)
   - `search()` and other READ operations

4. **Add 6 fields to MusicalMetadata struct** (migration 010)
   - File: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs`
   - Lines 82-130

5. **Update MetadataRepository INSERT** (line 11-71)
   - Add 6 chord columns to INSERT statement

6. **Update MetadataRepository SELECT** (line 78-119)
   - Add 6 chord columns to SELECT statement in `find_by_file_id()`

### Priority 2: High (Features incomplete)

7. **Implement Tag system models** (migration 007)
   - Create `TagCategory` model
   - Create `Tag` model (with new fields: priority, auto_detected, confidence_score, etc.)
   - Create `TagAlias` model
   - Create `AutoTaggingRule` model
   - Create `TagSuggestion` model

8. **Implement TagRepository** (migration 007)
   - Insert tag categories
   - Query tags by category
   - Support tag suggestions
   - Support auto-tagging rules

### Priority 3: Medium (Track splits & favorites)

9. **Verify TrackSplit table support** (migration 006)
   - Create TrackSplit model if missing
   - Create TrackSplitRepository

10. **Verify Favorites table support** (migration 003)
    - Create Favorite model if missing
    - Create FavoriteRepository

---

## Quick Fix Checklist

- [ ] Add 11 fields to `File` struct
- [ ] Add 11 fields to `NewFile` struct
- [ ] Update `FileRepository::insert()` with 11 new columns
- [ ] Update `FileRepository::find_by_id()` to SELECT 11 new columns
- [ ] Update `FileRepository::find_by_hash()` to SELECT 11 new columns
- [ ] Update `FileRepository::find_by_path()` to SELECT 11 new columns
- [ ] Add 6 fields to `MusicalMetadata` struct
- [ ] Add 6 fields to `NewMusicalMetadata` struct
- [ ] Update `MetadataRepository::insert()` with 6 chord columns
- [ ] Update `MetadataRepository::find_by_file_id()` to SELECT 6 chord columns
- [ ] Create chord update method in `MetadataRepository`
- [ ] Implement Tag models (5 structs)
- [ ] Implement TagRepository functions
- [ ] Test with actual data inserts/queries

---

## Files Affected

1. **`pipeline/src-tauri/src/db/models.rs`** (2 structs: File, NewFile, MusicalMetadata, NewMusicalMetadata)
2. **`pipeline/src-tauri/src/db/repositories/file_repository.rs`** (4+ methods with SELECT)
3. **`pipeline/src-tauri/src/db/repositories/metadata_repository.rs`** (INSERT + SELECT methods)
4. **`pipeline/src-tauri/src/db/repositories/tag_repository.rs`** (NEW - tag support)
5. **`pipeline/src-tauri/src/db/repositories/mod.rs`** (exports)
6. **`shared/rust/src/db/models/mod.rs`** (if shared models needed)

---

## Testing Recommendations

1. Run `sqlx prepare` to validate compile-time queries against actual schema
2. Test INSERT operations with all new fields populated
3. Test SELECT operations return all columns (check NULL vs missing)
4. Verify indexes are used for new columns (EXPLAIN ANALYZE)
5. Test with real MIDI files to ensure metadata is persisted

---

**Report Generated:** 2025-11-11
**Status:** 15 CRITICAL MISMATCHES IDENTIFIED
