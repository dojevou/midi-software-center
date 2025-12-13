# Schema Mismatch - Quick Fix Checklist

**Date:** 2025-11-11
**Status:** 28 CRITICAL COLUMNS MISSING FROM RUST CODE
**Estimated Time:** 4-5 hours

---

## Phase 1: Files Table Struct Updates (30 minutes)

### File struct - Add 11 fields
Location: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 18-55

```rust
#[derive(Debug, Clone, FromRow)]
pub struct File {
    // ... existing fields ...
    
    // Add from migration 002:
    pub parent_folder: Option<String>,
    
    // Add from migration 008:
    pub filename_bpm: Option<f32>,
    pub filename_key: Option<String>,
    pub filename_genres: Option<Vec<String>>,
    pub structure_tags: Option<Vec<String>>,
    pub metadata_source: Option<String>,
    
    // Add from migration 009:
    pub track_names: Option<Vec<String>>,
    pub copyright: Option<String>,
    pub instrument_names_text: Option<Vec<String>>,
    pub markers: Option<Vec<String>>,
    pub lyrics: Option<Vec<String>>,
}
```

### NewFile struct - Add 11 fields
Location: Same file, lines 57-74

```rust
#[derive(Debug, Clone)]
pub struct NewFile {
    // ... existing fields ...
    
    // Add same 11 fields as File struct
    pub parent_folder: Option<String>,
    pub filename_bpm: Option<f32>,
    pub filename_key: Option<String>,
    pub filename_genres: Option<Vec<String>>,
    pub structure_tags: Option<Vec<String>>,
    pub metadata_source: Option<String>,
    pub track_names: Option<Vec<String>>,
    pub copyright: Option<String>,
    pub instrument_names_text: Option<Vec<String>>,
    pub markers: Option<Vec<String>>,
    pub lyrics: Option<Vec<String>>,
}
```

- [ ] Add 11 fields to File struct
- [ ] Add 11 fields to NewFile struct
- [ ] Verify types match database schema
- [ ] Check Option<> usage is correct

---

## Phase 2: MusicalMetadata Struct Updates (15 minutes)

### MusicalMetadata struct - Add 6 fields
Location: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` lines 82-130

```rust
#[derive(Debug, Clone, FromRow)]
pub struct MusicalMetadata {
    // ... existing fields ...
    pub created_at: DateTime<Utc>,
    
    // Add from migration 010:
    pub chord_progression: Option<serde_json::Value>,  // JSONB
    pub chord_types: Option<Vec<String>>,
    pub has_seventh_chords: Option<bool>,
    pub has_extended_chords: Option<bool>,
    pub chord_change_rate: Option<BigDecimal>,
    pub chord_complexity_score: Option<BigDecimal>,
}
```

### NewMusicalMetadata struct - Add 6 fields
Location: Same file, lines 133-151

```rust
#[derive(Debug, Clone)]
pub struct NewMusicalMetadata {
    // ... existing fields ...
    pub is_percussive: Option<bool>,
    
    // Add from migration 010:
    pub chord_progression: Option<serde_json::Value>,
    pub chord_types: Option<Vec<String>>,
    pub has_seventh_chords: Option<bool>,
    pub has_extended_chords: Option<bool>,
    pub chord_change_rate: Option<BigDecimal>,
    pub chord_complexity_score: Option<BigDecimal>,
}
```

- [ ] Add 6 fields to MusicalMetadata struct
- [ ] Add 6 fields to NewMusicalMetadata struct
- [ ] Verify types match database schema
- [ ] Handle BigDecimal for numeric precision

---

## Phase 3: FileRepository Query Updates (45 minutes)

### FileRepository::insert() - Add 11 columns
Location: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/file_repository.rs` lines 11-30

**Current query has 14 columns, expand to 25 columns:**

```rust
let file_id = sqlx::query_scalar!(
    r#"
    INSERT INTO files (
        filename,
        filepath,
        original_filename,
        content_hash,
        file_size_bytes,
        format,
        num_tracks,
        ticks_per_quarter_note,
        duration_seconds,
        duration_ticks,
        manufacturer,
        collection_name,
        folder_tags,
        import_batch_id,
        // ADD THESE 11:
        parent_folder,
        filename_bpm,
        filename_key,
        filename_genres,
        structure_tags,
        metadata_source,
        track_names,
        copyright,
        instrument_names_text,
        markers,
        lyrics
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
    RETURNING id
    "#,
    // ... existing parameters ...
    new_file.parent_folder,
    new_file.filename_bpm,
    new_file.filename_key,
    new_file.filename_genres.as_deref(),
    new_file.structure_tags.as_deref(),
    new_file.metadata_source,
    new_file.track_names.as_deref(),
    new_file.copyright,
    new_file.instrument_names_text.as_deref(),
    new_file.markers.as_deref(),
    new_file.lyrics.as_deref(),
)
```

- [ ] Update INSERT query with 25 columns total
- [ ] Add 11 parameter values
- [ ] Update parameter count to $25
- [ ] Use .as_deref() for Vec<String> fields
- [ ] Test compilation with sqlx

### FileRepository::find_by_id() - Add 11 columns to SELECT
Location: Same file, lines 54-87

**Current query SELECTs 22 columns, expand to 33 columns:**

```rust
let file = sqlx::query_as!(
    File,
    r#"
    SELECT
        id,
        filename,
        filepath,
        original_filename,
        content_hash,
        file_size_bytes,
        format,
        num_tracks,
        ticks_per_quarter_note,
        duration_seconds,
        duration_ticks,
        is_multi_track,
        parent_file_id,
        track_number,
        total_tracks,
        manufacturer,
        collection_name,
        folder_tags,
        created_at as "created_at!",
        updated_at as "updated_at!",
        analyzed_at,
        import_batch_id,
        // ADD THESE 11:
        parent_folder,
        filename_bpm,
        filename_key,
        filename_genres,
        structure_tags,
        metadata_source,
        track_names,
        copyright,
        instrument_names_text,
        markers,
        lyrics
    FROM files WHERE id = $1
    "#,
    id
)
```

- [ ] Update SELECT query with 33 columns
- [ ] Verify column order matches struct definition
- [ ] Test compilation

### FileRepository::find_by_hash() - Add 11 columns to SELECT
Location: Same file, lines 111-144

Same 11 columns as find_by_id()

- [ ] Update SELECT query
- [ ] Same 11 new columns

### FileRepository::find_by_path() - Add 11 columns to SELECT
Location: Same file, lines 148+

Same 11 columns as find_by_id()

- [ ] Update SELECT query
- [ ] Same 11 new columns

### FileRepository - Other SELECT queries
Search for other `sqlx::query_as!(File, ...` patterns

- [ ] Check search() method
- [ ] Check list methods
- [ ] Check any other File selects
- [ ] Update all to include 11 new columns

---

## Phase 4: MetadataRepository Query Updates (30 minutes)

### MetadataRepository::insert() - Add 6 columns
Location: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/metadata_repository.rs` lines 11-71

**Current query has 16 columns, expand to 22 columns:**

```rust
pub async fn insert(pool: &PgPool, metadata: NewMusicalMetadata) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO musical_metadata (
            file_id,
            bpm,
            bpm_confidence,
            key_signature,
            key_confidence,
            time_signature_numerator,
            time_signature_denominator,
            total_notes,
            unique_pitches,
            pitch_range_min,
            pitch_range_max,
            avg_velocity,
            note_density,
            polyphony_max,
            polyphony_avg,
            is_percussive,
            // ADD THESE 6:
            chord_progression,
            chord_types,
            has_seventh_chords,
            has_extended_chords,
            chord_change_rate,
            chord_complexity_score
        ) VALUES (
            $1, $2, $3, $4::text::musical_key, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
            $17, $18, $19, $20, $21, $22
        )
        ON CONFLICT (file_id) DO UPDATE SET
            bpm = EXCLUDED.bpm,
            // ... existing updates ...
            is_percussive = EXCLUDED.is_percussive,
            // ADD THESE 6:
            chord_progression = EXCLUDED.chord_progression,
            chord_types = EXCLUDED.chord_types,
            has_seventh_chords = EXCLUDED.has_seventh_chords,
            has_extended_chords = EXCLUDED.has_extended_chords,
            chord_change_rate = EXCLUDED.chord_change_rate,
            chord_complexity_score = EXCLUDED.chord_complexity_score
        "#,
        // ... existing parameters ...
        metadata.chord_progression,
        metadata.chord_types.as_deref(),
        metadata.has_seventh_chords,
        metadata.has_extended_chords,
        metadata.chord_change_rate,
        metadata.chord_complexity_score,
    )
}
```

- [ ] Update INSERT with 6 chord columns
- [ ] Update ON CONFLICT clause with 6 columns
- [ ] Add 6 parameter values
- [ ] Verify column order and types

### MetadataRepository::find_by_file_id() - Add 6 columns to SELECT
Location: Same file, lines 78-119

**Current query SELECTs 24 columns, expand to 30 columns:**

```rust
let metadata = sqlx::query_as!(
    MusicalMetadata,
    r#"
    SELECT
        file_id,
        bpm,
        bpm_confidence,
        has_tempo_changes,
        tempo_changes,
        key_signature::text as key_signature,
        key_confidence,
        has_key_changes,
        key_changes,
        time_signature_numerator,
        time_signature_denominator,
        has_time_signature_changes,
        time_signature_changes,
        total_notes,
        unique_pitches,
        pitch_range_min,
        pitch_range_max,
        avg_velocity,
        note_density,
        polyphony_max,
        polyphony_avg,
        is_monophonic,
        is_polyphonic,
        is_percussive,
        has_chords,
        chord_complexity,
        has_melody,
        melodic_range,
        created_at as "created_at!",
        // ADD THESE 6:
        chord_progression,
        chord_types,
        has_seventh_chords,
        has_extended_chords,
        chord_change_rate,
        chord_complexity_score
    FROM musical_metadata WHERE file_id = $1
    "#,
    file_id
)
```

- [ ] Update SELECT with 6 chord columns
- [ ] Verify column order matches struct
- [ ] Test compilation

### MetadataRepository - Add chord update method (NEW)

```rust
pub async fn update_chords(
    pool: &PgPool,
    file_id: i64,
    chord_progression: Option<serde_json::Value>,
    chord_types: Option<Vec<String>>,
    has_seventh_chords: Option<bool>,
    has_extended_chords: Option<bool>,
    chord_change_rate: Option<sqlx::types::BigDecimal>,
    chord_complexity_score: Option<sqlx::types::BigDecimal>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE musical_metadata
        SET chord_progression = $1,
            chord_types = $2,
            has_seventh_chords = $3,
            has_extended_chords = $4,
            chord_change_rate = $5,
            chord_complexity_score = $6
        WHERE file_id = $7
        "#,
        chord_progression,
        chord_types.as_deref(),
        has_seventh_chords,
        has_extended_chords,
        chord_change_rate,
        chord_complexity_score,
        file_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

- [ ] Create new update_chords() method
- [ ] Add to MetadataRepository implementation
- [ ] Verify parameter order and types

---

## Phase 5: Build & Validation (20 minutes)

- [ ] Run `cargo build --workspace` in pipeline/src-tauri
- [ ] Check for compilation errors
- [ ] Run `sqlx prepare --database-url <url> -- pipeline/src-tauri/src`
- [ ] Verify all queries compile against actual database
- [ ] Run `cargo test --workspace` for unit tests
- [ ] Fix any remaining compile errors

---

## Phase 6: Tag System Implementation (2.5 hours) - OPTIONAL

**SKIP if tags not needed for MVP**

### Create Tag Models
File: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/models.rs` (or new tag_models.rs)

```rust
#[derive(Debug, Clone, FromRow)]
pub struct TagCategory {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
    pub category_id: Option<i32>,
    pub priority: Option<i32>,
    pub auto_detected: Option<bool>,
    pub confidence_score: Option<BigDecimal>,
    pub detection_method: Option<String>,
    pub parent_tag_id: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TagAlias {
    pub id: i32,
    pub tag_id: i32,
    pub alias: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AutoTaggingRule {
    pub id: i32,
    pub rule_name: String,
    pub rule_type: String,
    pub pattern: String,
    pub tags_to_add: Vec<i32>,
    pub confidence: Option<BigDecimal>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TagSuggestion {
    pub id: i32,
    pub file_id: i64,
    pub suggested_tag_id: i32,
    pub confidence: BigDecimal,
    pub source: String,
    pub is_accepted: Option<bool>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

- [ ] Create all 5 tag models
- [ ] Verify field types match schema
- [ ] Add FromRow derives

### Implement TagRepository Functions

File: `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/tag_repository.rs`

Key functions:
- `insert_category()`
- `get_tags_by_category()`
- `insert_tag()`
- `get_tag_suggestions()`
- `accept_suggestion()`
- `apply_auto_tagging_rules()`

- [ ] Implement TagRepository struct
- [ ] Implement at least 6 core functions
- [ ] Test with sample data

### Update SearchRepository
Add filters for tag-based searches

- [ ] Add tag filtering to search()
- [ ] Add confidence filtering
- [ ] Add detection_method filtering

---

## Phase 7: Testing & Deployment (1 hour)

### Unit Tests
- [ ] Test File struct with 11 new fields
- [ ] Test MusicalMetadata struct with 6 new fields
- [ ] Test FileRepository insert/select operations
- [ ] Test MetadataRepository insert/select operations
- [ ] Test with NULL values (all Option<> types)

### Integration Tests
- [ ] Import MIDI file with filename metadata
- [ ] Verify filename_bpm/key are extracted and persisted
- [ ] Verify text metadata (copyright, track names) is persisted
- [ ] Verify harmonic analysis data is persisted
- [ ] Verify search works with new columns

### Database Tests
- [ ] Verify indexes are created
- [ ] Check index usage (EXPLAIN ANALYZE)
- [ ] Verify constraints are working
- [ ] Check for data type mismatches

### Deployment Checklist
- [ ] All tests passing (cargo test --workspace)
- [ ] No compiler warnings
- [ ] sqlx prepare validates all queries
- [ ] Performance testing with 1000+ files
- [ ] Backup database before migration
- [ ] Test rollback procedure

---

## Summary Checklist

### Critical Path (MUST DO)
- [ ] Phase 1: File struct (11 fields)
- [ ] Phase 2: MusicalMetadata struct (6 fields)
- [ ] Phase 3: FileRepository queries (25 columns)
- [ ] Phase 4: MetadataRepository queries (22 columns)
- [ ] Phase 5: Build & validation
- [ ] Phase 7: Testing

### Optional (IF MVP NEEDS IT)
- [ ] Phase 6: Tag system implementation

### Time Tracking
- Phase 1: 30 min
- Phase 2: 15 min
- Phase 3: 45 min
- Phase 4: 30 min
- Phase 5: 20 min
- Phase 6: 150 min (optional)
- Phase 7: 60 min
- **TOTAL: 190 min (3.2 hours) REQUIRED + 2.5 hours OPTIONAL**

---

## Important Notes

1. **Type Safety:** Use `Option<T>` for all nullable columns
2. **Array Handling:** Use `.as_deref()` when passing Vec<String> to sqlx
3. **JSONB:** Use `serde_json::Value` for JSON data
4. **Numeric Precision:** Use `BigDecimal` for NUMERIC types
5. **Enums:** For the musical_key enum, cast as `::text::musical_key`
6. **Order Matters:** Column order in struct must match schema
7. **Test After Each Phase:** Don't wait until end to test

---

**Generated:** 2025-11-11
**Status:** Ready for implementation
**Confidence:** HIGH
