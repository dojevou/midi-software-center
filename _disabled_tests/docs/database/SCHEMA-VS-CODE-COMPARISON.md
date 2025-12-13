# Schema vs Code Comparison - Visual Guide

## Overview
This document shows side-by-side comparisons of what exists in the database schema vs what the Rust code supports.

---

## FILES TABLE - Comparison

### Migration 001 Status: COMPLETE
```
Database Schema                          | Rust Models
=========================================|=========================================
id BIGSERIAL PRIMARY KEY                | pub id: i64 ✅
filename TEXT NOT NULL                  | pub filename: String ✅
filepath TEXT NOT NULL UNIQUE           | pub filepath: String ✅
original_filename TEXT NOT NULL         | pub original_filename: String ✅
content_hash BYTEA NOT NULL             | pub content_hash: Vec<u8> ✅
file_size_bytes BIGINT NOT NULL         | pub file_size_bytes: i64 ✅
format SMALLINT                         | pub format: Option<i16> ✅
num_tracks SMALLINT NOT NULL            | pub num_tracks: i16 ✅
ticks_per_quarter_note INTEGER          | pub ticks_per_quarter_note: Option<i32> ✅
duration_seconds NUMERIC(10,3)          | pub duration_seconds: Option<BigDecimal> ✅
duration_ticks BIGINT                   | pub duration_ticks: Option<i64> ✅
is_multi_track BOOLEAN                  | pub is_multi_track: Option<bool> ✅
parent_file_id BIGINT                   | pub parent_file_id: Option<i64> ✅
track_number SMALLINT                   | pub track_number: Option<i16> ✅
total_tracks SMALLINT                   | pub total_tracks: Option<i16> ✅
manufacturer TEXT                       | pub manufacturer: Option<String> ✅
collection_name TEXT                    | pub collection_name: Option<String> ✅
folder_tags TEXT[]                      | pub folder_tags: Option<Vec<String>> ✅
search_vector tsvector                  | (auto-maintained by trigger) ✅
created_at TIMESTAMPTZ                  | pub created_at: DateTime<Utc> ✅
updated_at TIMESTAMPTZ                  | pub updated_at: DateTime<Utc> ✅
analyzed_at TIMESTAMPTZ                 | pub analyzed_at: Option<DateTime<Utc>> ✅
import_batch_id UUID                    | pub import_batch_id: Option<Uuid> ✅
```

### Migration 002 Status: MISSING
```
Database Schema                          | Rust Models
=========================================|=========================================
parent_folder TEXT                      | NOT IN CODE ❌
CREATE INDEX idx_files_parent_folder    | NO SUPPORT ❌
```
**File Location:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 18-55

---

### Migration 008 Status: MISSING
```
Database Schema                          | Rust Models
=========================================|=========================================
filename_bpm REAL CHECK (30-300)        | NOT IN CODE ❌
filename_key TEXT CHECK (1-3 chars)     | NOT IN CODE ❌
filename_genres TEXT[]                  | NOT IN CODE ❌
structure_tags TEXT[]                   | NOT IN CODE ❌
metadata_source TEXT (enum)             | NOT IN CODE ❌
CREATE INDEX idx_files_filename_bpm     | NO SUPPORT ❌
CREATE INDEX idx_files_filename_key     | NO SUPPORT ❌
CREATE INDEX idx_files_filename_genres  | NO SUPPORT ❌
CREATE INDEX idx_files_structure_tags   | NO SUPPORT ❌
CREATE INDEX idx_files_metadata_source  | NO SUPPORT ❌
```
**File Location:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 18-55
**Files Affected:**
- `NewFile` struct missing 5 fields
- `FileRepository::insert()` missing 5 columns
- `FileRepository::find_*()` queries missing 5 columns

---

### Migration 009 Status: MISSING
```
Database Schema                          | Rust Models
=========================================|=========================================
track_names TEXT[] DEFAULT '{}'         | NOT IN CODE ❌
copyright TEXT                          | NOT IN CODE ❌
instrument_names_text TEXT[]            | NOT IN CODE ❌
markers TEXT[] DEFAULT '{}'             | NOT IN CODE ❌
lyrics TEXT[] DEFAULT '{}'              | NOT IN CODE ❌
CREATE INDEX idx_files_copyright        | NO SUPPORT ❌
CREATE INDEX idx_files_track_names      | NO SUPPORT ❌
CREATE INDEX idx_files_instrument_names | NO SUPPORT ❌
CREATE INDEX idx_files_markers          | NO SUPPORT ❌
CREATE INDEX idx_files_copyright_trgm   | NO SUPPORT ❌
```
**File Location:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 18-55
**Files Affected:**
- `NewFile` struct missing 5 fields
- `FileRepository::insert()` missing 5 columns
- `FileRepository::find_*()` queries missing 5 columns
- SearchRepository missing text metadata filters

---

## MUSICAL_METADATA TABLE - Comparison

### Migration 001 Status: COMPLETE
```
Database Schema                          | Rust Models
=========================================|=========================================
file_id BIGINT PRIMARY KEY              | pub file_id: i64 ✅
bpm NUMERIC(6,2)                        | pub bpm: Option<BigDecimal> ✅
bpm_confidence REAL                     | pub bpm_confidence: Option<f32> ✅
has_tempo_changes BOOLEAN               | pub has_tempo_changes: Option<bool> ✅
tempo_changes JSONB                     | pub tempo_changes: Option<serde_json::Value> ✅
key_signature musical_key               | pub key_signature: Option<String> ✅
key_confidence REAL                     | pub key_confidence: Option<f32> ✅
has_key_changes BOOLEAN                 | pub has_key_changes: Option<bool> ✅
key_changes JSONB                       | pub key_changes: Option<serde_json::Value> ✅
time_signature_numerator SMALLINT       | pub time_signature_numerator: Option<i16> ✅
time_signature_denominator SMALLINT     | pub time_signature_denominator: Option<i16> ✅
has_time_signature_changes BOOLEAN      | pub has_time_signature_changes: Option<bool> ✅
time_signature_changes JSONB            | pub time_signature_changes: Option<serde_json::Value> ✅
total_notes INTEGER                     | pub total_notes: i32 ✅
unique_pitches INTEGER                  | pub unique_pitches: Option<i32> ✅
pitch_range_min SMALLINT                | pub pitch_range_min: Option<i16> ✅
pitch_range_max SMALLINT                | pub pitch_range_max: Option<i16> ✅
avg_velocity NUMERIC(5,2)               | pub avg_velocity: Option<BigDecimal> ✅
note_density NUMERIC(8,3)               | pub note_density: Option<BigDecimal> ✅
polyphony_max SMALLINT                  | pub polyphony_max: Option<i16> ✅
polyphony_avg NUMERIC(5,2)              | pub polyphony_avg: Option<BigDecimal> ✅
is_monophonic BOOLEAN                   | pub is_monophonic: Option<bool> ✅
is_polyphonic BOOLEAN                   | pub is_polyphonic: Option<bool> ✅
is_percussive BOOLEAN                   | pub is_percussive: Option<bool> ✅
has_chords BOOLEAN                      | pub has_chords: Option<bool> ✅
chord_complexity REAL                   | pub chord_complexity: Option<f32> ✅
has_melody BOOLEAN                      | pub has_melody: Option<bool> ✅
melodic_range SMALLINT                  | pub melodic_range: Option<i16> ✅
created_at TIMESTAMPTZ                  | pub created_at: DateTime<Utc> ✅
```

### Migration 010 Status: MISSING
```
Database Schema                          | Rust Models
=========================================|=========================================
chord_progression JSONB                 | NOT IN CODE ❌
chord_types TEXT[]                      | NOT IN CODE ❌
has_seventh_chords BOOLEAN              | NOT IN CODE ❌
has_extended_chords BOOLEAN             | NOT IN CODE ❌
chord_change_rate NUMERIC(5,2)          | NOT IN CODE ❌
chord_complexity_score NUMERIC(4,3)     | NOT IN CODE ❌
CREATE INDEX idx_has_seventh_chords     | NO SUPPORT ❌
CREATE INDEX idx_has_extended_chords    | NO SUPPORT ❌
CREATE INDEX idx_chord_progression      | NO SUPPORT ❌
CREATE INDEX idx_chord_types            | NO SUPPORT ❌
CREATE INDEX idx_chord_complexity       | NO SUPPORT ❌
```
**File Location:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 82-130
**Files Affected:**
- `MusicalMetadata` struct missing 6 fields
- `NewMusicalMetadata` struct missing 6 fields
- `MetadataRepository::insert()` missing 6 columns
- `MetadataRepository::find_by_file_id()` missing 6 columns

---

## TAGS TABLE - Comparison

### Migration 001 Status: PARTIAL
```
Database Schema                          | Rust Models
=========================================|=========================================
id SERIAL PRIMARY KEY                   | (basic support)
name TEXT NOT NULL UNIQUE               | (basic support)
category TEXT                           | (basic support)
usage_count INTEGER                     | (basic support)
created_at TIMESTAMPTZ                  | (basic support)
```

### Migration 007 - NEW COLUMNS Status: MISSING
```
Database Schema                          | Rust Models
=========================================|=========================================
category_id INTEGER FK                  | NOT IN CODE ❌
priority INTEGER DEFAULT 50             | NOT IN CODE ❌
auto_detected BOOLEAN DEFAULT FALSE     | NOT IN CODE ❌
confidence_score DECIMAL(3,2)           | NOT IN CODE ❌
detection_method VARCHAR(50)            | NOT IN CODE ❌
parent_tag_id INTEGER FK                | NOT IN CODE ❌
is_active BOOLEAN DEFAULT TRUE          | NOT IN CODE ❌
```

### Migration 007 - NEW TABLES Status: MISSING
```
Database Table                          | Rust Models
=========================================|=========================================
CREATE TABLE tag_categories             | NO MODEL ❌
CREATE TABLE tag_aliases                | NO MODEL ❌
CREATE TABLE auto_tagging_rules         | NO MODEL ❌
CREATE TABLE tag_suggestions            | NO MODEL ❌
```
**Missing Files:**
- No TagCategory model
- No TagAlias model
- No AutoTaggingRule model
- No TagSuggestion model
- Minimal/incomplete TagRepository

---

## REPOSITORY QUERY IMPACT

### FileRepository::insert() - MISSING DATA
```sql
-- CURRENT (INCOMPLETE)
INSERT INTO files (
    filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    manufacturer, collection_name, folder_tags, import_batch_id
) VALUES (...)

-- MISSING COLUMNS (will be NULL)
-- parent_folder                    [Migration 002]
-- filename_bpm, filename_key, ...  [Migration 008]
-- track_names, copyright, ...      [Migration 009]
```
**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/file_repository.rs` lines 11-50

### FileRepository::find_by_id() - INCOMPLETE READS
```sql
-- CURRENT (MISSING COLUMNS)
SELECT id, filename, filepath, ..., import_batch_id
FROM files WHERE id = $1

-- NOT SELECTED (returns NULL/unknown)
-- parent_folder
-- filename_bpm, filename_key, filename_genres, structure_tags, metadata_source
-- track_names, copyright, instrument_names_text, markers, lyrics
```
**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/file_repository.rs` lines 54-88

### MetadataRepository::insert() - MISSING CHORD DATA
```sql
-- CURRENT (INCOMPLETE)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, ..., is_percussive
) VALUES (...)

-- MISSING COLUMNS (will be NULL)
-- chord_progression, chord_types
-- has_seventh_chords, has_extended_chords
-- chord_change_rate, chord_complexity_score
```
**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/metadata_repository.rs` lines 11-71

### MetadataRepository::find_by_file_id() - INCOMPLETE READS
```sql
-- CURRENT (MISSING COLUMNS)
SELECT file_id, bpm, ..., melodic_range, created_at
FROM musical_metadata WHERE file_id = $1

-- NOT SELECTED (returns NULL/unknown)
-- chord_progression, chord_types
-- has_seventh_chords, has_extended_chords
-- chord_change_rate, chord_complexity_score
```
**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/metadata_repository.rs` lines 78-119

---

## Summary Statistics

| Category | Total | Supported | Missing | % Complete |
|----------|-------|-----------|---------|------------|
| Files table columns | 33 | 22 | 11 | 67% |
| Musical_metadata columns | 30 | 24 | 6 | 80% |
| Tags table columns | 12 | 5 | 7 | 42% |
| Tag-related tables | 4 | 0 | 4 | 0% |
| **TOTAL** | **79** | **51** | **28** | **65%** |

---

## Data Flow Impact

### IMPORT WORKFLOW (Migration 008 - BROKEN)
```
Filename Analysis
    ↓
Extract: filename_bpm, filename_key, filename_genres, structure_tags, metadata_source
    ↓
INSERT INTO files ❌ MISSING FIELDS (silently ignored/NULL)
    ↓
Data LOST - Cannot retrieve these values
```

### TEXT METADATA WORKFLOW (Migration 009 - BROKEN)
```
MIDI Parse
    ↓
Extract: track_names, copyright, instrument_names_text, markers, lyrics
    ↓
INSERT INTO files ❌ MISSING FIELDS (silently ignored/NULL)
    ↓
Data LOST - Cannot retrieve these values
```

### HARMONIC ANALYSIS WORKFLOW (Migration 010 - BROKEN)
```
Analysis Engine
    ↓
Compute: chord_progression, chord_types, chord_complexity_score, etc.
    ↓
INSERT INTO musical_metadata ❌ MISSING FIELDS (silently ignored/NULL)
    ↓
Data LOST - Features non-functional
```

### TAG SYSTEM WORKFLOW (Migration 007 - BROKEN)
```
Auto-Tagging Rules (NOT IN CODE)
    ↓
Tag Suggestions (NO MODEL)
    ↓
Tag Categories (NO MODEL)
    ↓
Cannot apply priority/confidence/detection_method (NOT IN STRUCT)
    ↓
Feature DISABLED - System degraded
```

---

## Fix Complexity Assessment

| Item | Complexity | Effort | Risk |
|------|-----------|--------|------|
| Add 11 fields to File struct | LOW | 15 min | LOW |
| Update FileRepository INSERT | LOW | 20 min | LOW |
| Update FileRepository SELECTs (4 places) | LOW | 30 min | LOW |
| Add 6 fields to MusicalMetadata struct | LOW | 10 min | LOW |
| Update MetadataRepository INSERT | LOW | 15 min | LOW |
| Update MetadataRepository SELECT | LOW | 10 min | LOW |
| Implement Tag models (5 structs) | MEDIUM | 60 min | MEDIUM |
| Implement TagRepository functions | MEDIUM | 120 min | MEDIUM |
| Test & validation | LOW-MEDIUM | 45 min | LOW |
| **TOTAL** | **MEDIUM** | **~4.5 hours** | **MEDIUM** |

---

## Critical Path

1. **Add missing struct fields** (35 min) - UNBLOCKS other work
2. **Update repository queries** (75 min) - REQUIRED for functionality
3. **Test basic operations** (20 min) - VERIFY data persistence
4. **Implement Tag system** (180 min) - ENABLES auto-tagging
5. **Performance & validation** (30 min) - ENSURE production readiness

**Critical path to production:** 4-5 hours of focused development

