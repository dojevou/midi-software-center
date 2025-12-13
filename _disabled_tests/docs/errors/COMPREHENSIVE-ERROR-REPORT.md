# Comprehensive Error Report for DeepSeek Review

**Generated:** 2025-11-29
**Project:** MIDI Software Center
**Total Errors:** 58 (49 Rust + 9 TypeScript/Svelte)
**Total Warnings:** 355+

---

## TABLE OF CONTENTS

1. [TypeScript/Svelte Errors (9)](#1-typescriptsvelte-errors)
2. [Rust Test Compilation Errors (49)](#2-rust-test-compilation-errors)
3. [Rust Warnings Summary](#3-rust-warnings-summary)
4. [A11y Warnings](#4-a11y-warnings)

---

## 1. TYPESCRIPT/SVELTE ERRORS

### Error Count: 9 errors

---

### FILE: app/src/lib/types.ts

```typescript
/**
 * Complete file details with all metadata
 * Backend: daw/src-tauri/src/models/midi_file.rs
 */
export interface FileDetails {
  id: number;                          // i64
  filename: string;                    // filename (from backend)
  filepath: string;                    // filepath (from backend)
  file_size_bytes: number;             // file_size_bytes (from backend)
  bpm?: number;                        // Option<f64>
  key_signature?: string;              // key_signature option
  time_signature?: string;             // "4/4" format
  duration_seconds?: number;           // NUMERIC(10,3)
  total_notes?: number;                // INTEGER
  primary_category?: string;           // primary_category optional
  parent_folder?: string;              // Option<String>
  created_at: string;                  // ISO 8601 timestamp
  is_favorite?: boolean;               // bool - made optional for frontend flexibility
  tags: string[];                      // Vec<String> (folder_tags from backend)
  manufacturer?: string;               // Option<String>
  collection_name?: string;            // collection_name optional
  track_count: number;                 // num_tracks (i16)
  has_notes: boolean;                  // bool
  has_drums?: boolean;                 // Option<bool>
  content_hash: string;                // content_hash from backend
}

export interface SearchFilters {
  search_text?: string;
  min_bpm?: number;
  max_bpm?: number;
  key_signature?: string;
  time_signature?: string;
  category?: string;
  min_notes?: number;
  max_notes?: number;
  min_duration?: number;
  max_duration?: number;
  instruments?: string[];
  sort_by?: string;
  sort_desc?: boolean;
  limit?: number;
  offset?: number;
}

// ... more types ...

export interface PlaybackPosition {
  bar: number;      // <-- PROBLEM: Uses 'bar' but code uses 'current_bar'
  beat: number;     // <-- PROBLEM: Uses 'beat' but code uses 'current_beat'
  tick: number;     // <-- PROBLEM: Uses 'tick' but code uses 'current_tick'
}

// ... more types ...

export interface ArchiveProgress {
  current: number;
  total: number;
  current_file: string;        // <-- PROBLEM: Missing 'current_archive' property
  bytes_processed: number;
  total_bytes: number;
}

export interface ArchiveError {
  file_path: string;           // <-- PROBLEM: Uses 'file_path' but code uses 'archivePath'
  error_message: string;       // <-- PROBLEM: Uses 'error_message' but code uses 'error'
}

export interface FileParams {
  file_path: string;
  file_name: string;
  bpm?: number;
  key_signature?: string;
  tags?: string[];
  duration?: number;
  track_count?: number;
  file_size?: number;
}
// <-- PROBLEM: FileParams interface missing index signature for Record<string, unknown>
```

---

### FILE: app/src/lib/stores/playbackStore.ts

```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { PlaybackPosition } from '$lib/types';

// ============================================================================
// PLAYBACK STATE
// ============================================================================

export interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  tempo: number;
  timeSignature: [number, number];
  keySignature: string;
  position: PlaybackPosition;
  loopEnabled: boolean;
  loopStart: number;
  loopEnd: number;
  metronomeEnabled: boolean;
  metronomeVolume: number;
}

const initialState: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  tempo: 120,
  timeSignature: [4, 4],
  keySignature: 'C',
  position: {
    current_tick: 0,    // <-- ERROR [Line 31]: 'current_tick' does not exist in type 'PlaybackPosition'
    current_bar: 0,     // <-- ERROR [Line 32]: 'current_bar' does not exist in type 'PlaybackPosition'
    current_beat: 0,    // <-- ERROR [Line 33]: 'current_beat' does not exist in type 'PlaybackPosition'
  },
  loopEnabled: false,
  loopStart: 0,
  loopEnd: 0,
  metronomeEnabled: false,
  metronomeVolume: 0.7,
};

export const playbackStore = writable<PlaybackState>(initialState);

// ... more code ...

    // Listen for playback stopped
    const unlistenStop = await listen('daw::playback-stopped', () => {
      playbackStore.update(state => ({
        ...state,
        isPlaying: false,
        isPaused: false,
        position: { current_tick: 0, current_bar: 0, current_beat: 0 }  // <-- ERROR [Line 67]
      }));
    });

// ... more code ...

      playbackStore.update(s => ({
        ...s,
        position: {
          current_tick,    // <-- ERROR [Line 88]: 'current_tick' does not exist
          current_bar,     // <-- ERROR [Line 89]: 'current_bar' does not exist
          current_beat,    // <-- ERROR [Line 90]: 'current_beat' does not exist
        }
      }));

// ... more code ...

export const formattedPosition = derived(
  playbackStore,
  ($playback) => {
    const { current_bar, current_beat } = $playback.position;  // <-- ERROR [Line 234]: Properties don't exist
    return `${current_bar + 1}:${current_beat + 1}`;
  }
);
```

### LINT OUTPUT for playbackStore.ts:

```
/home/dojevou/projects/midi-software-center/app/src/lib/stores/playbackStore.ts:31:5
Error: Object literal may only specify known properties, and 'current_tick' does not exist in type 'PlaybackPosition'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/playbackStore.ts:67:21
Error: Object literal may only specify known properties, and 'current_tick' does not exist in type 'PlaybackPosition'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/playbackStore.ts:88:11
Error: Object literal may only specify known properties, and 'current_tick' does not exist in type 'PlaybackPosition'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/playbackStore.ts:234:13
Error: Property 'current_bar' does not exist on type 'PlaybackPosition'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/playbackStore.ts:234:26
Error: Property 'current_beat' does not exist on type 'PlaybackPosition'.
```

**ROOT CAUSE:** The `PlaybackPosition` interface in `types.ts` defines `{ bar, beat, tick }` but the code uses `{ current_bar, current_beat, current_tick }`.

**FIX:** Either update the interface to use `current_*` prefixes OR update the code to use the shorter names.

---

### FILE: app/src/lib/api.ts

```typescript
    /**
     * Add file to database
     * Backend: daw/src-tauri/src/commands/database.rs:166
     */
    addFile: async (params: FileParams): Promise<number> => {
      try {
        return await invoke('database_add_file', params);  // <-- ERROR [Line 617]
      } catch (error) {
        console.error('Add file failed:', error);
        throw error;
      }
    },
```

### LINT OUTPUT for api.ts:

```
/home/dojevou/projects/midi-software-center/app/src/lib/api.ts:617:50
Error: Argument of type 'FileParams' is not assignable to parameter of type 'InvokeArgs | undefined'.
  Type 'FileParams' is not assignable to type 'Record<string, unknown>'.
    Index signature for type 'string' is missing in type 'FileParams'.
```

**ROOT CAUSE:** Tauri's `invoke()` expects `Record<string, unknown>` but `FileParams` is a typed interface without an index signature.

**FIX:** Add index signature to FileParams: `[key: string]: unknown;` OR cast params: `params as Record<string, unknown>`

---

### FILE: app/src/lib/stores/archiveStore.ts

```typescript
import { writable, type Writable } from 'svelte/store';
import type { ArchiveProgress, ArchiveError } from '../types';

export interface ArchiveState {
  isExtracting: boolean;
  progress: number;
  currentArchive: string;
  extracted: number;
  totalFiles: number;
  errors: string[];
  extractedPaths: string[];
}

// ... more code ...

  updateProgress: (progress: ArchiveProgress) => {
    update((state: ArchiveState) => ({
      ...state,
      progress: (progress.current / progress.total) * 100,
      extracted: progress.current,
      totalFiles: progress.total,
      currentArchive: progress.current_archive  // <-- ERROR [Line 46]: Property 'current_archive' does not exist
    }));
  },

  addError: (error: ArchiveError) => {
    update((state: ArchiveState) => ({
      ...state,
      errors: [...state.errors, `${error.archivePath}: ${error.error}`]  // <-- ERROR [Line 53]: Properties don't exist
    }));
  },
```

### LINT OUTPUT for archiveStore.ts:

```
/home/dojevou/projects/midi-software-center/app/src/lib/stores/archiveStore.ts:46:32
Error: Property 'current_archive' does not exist on type 'ArchiveProgress'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/archiveStore.ts:53:42
Error: Property 'archivePath' does not exist on type 'ArchiveError'.

/home/dojevou/projects/midi-software-center/app/src/lib/stores/archiveStore.ts:53:64
Error: Property 'error' does not exist on type 'ArchiveError'.
```

**ROOT CAUSE:** `ArchiveProgress` defines `current_file` but code uses `current_archive`. `ArchiveError` defines `file_path`/`error_message` but code uses `archivePath`/`error`.

**FIX:** Update the interface to match actual usage OR update code to match interface.

---

## 2. RUST TEST COMPILATION ERRORS

### Error Count: 49 errors

### Error Categories:
- `E0308` mismatched types: 30 errors
- `E0308` arguments incorrect: 6 errors
- `E0277` trait bound not satisfied: 3 errors
- `E0609` no field on type: 3 errors
- `E0063` missing field in initializer: 1 error

---

### FILE: pipeline/src-tauri/tests/commands/tags_error_test.rs

```rust
/// Tags Command Error Path Tests
/// Tests error handling, edge cases, and boundary conditions for tag functionality

use midi_pipeline::commands::tags::{
    get_file_tags_impl, add_tags_to_file_impl,
    search_tags_impl, get_popular_tags_impl
};
use midi_pipeline::{AppState, database::Database};

#[tokio::test]
async fn test_get_tags_nonexistent_file() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = get_file_tags_impl(999999999, &state).await;

    // Should handle gracefully (empty result or error)
    if let Ok(tags) = result {
        assert!(tags.is_empty(), "Nonexistent file should have no tags");
    }
    // Or it could error - both are acceptable
}

// ... more tests ...

#[tokio::test]
async fn test_search_tags_empty_query() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_tags_impl("".to_string(), 10, &state).await;  // <-- ERROR [Line 170]
    //                                            ^^ expected Option<i32>, found integer

    // Empty query should return results (all tags) or empty
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_tags_no_matches() {
    // ... setup ...

    let result = search_tags_impl("NONEXISTENT_TAG_SEARCH_XYZ".to_string(), 10, &state).await;  // <-- ERROR [Line 184]
    //                                                                       ^^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_search_tags_special_characters() {
    // ... setup ...

    for query in special_queries {
        let result = search_tags_impl(query.to_string(), 10, &state).await;  // <-- ERROR [Line 202]
        //                                              ^^ expected Option<i32>, found integer
        assert!(result.is_ok(), "Failed on special character: {}", query);
    }
}

#[tokio::test]
async fn test_search_tags_negative_limit() {
    // ... setup ...

    let result = search_tags_impl("test".to_string(), -10, &state).await;  // <-- ERROR [Line 215]
    //                                                ^^^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_search_tags_zero_limit() {
    // ... setup ...

    let result = search_tags_impl("test".to_string(), 0, &state).await;  // <-- ERROR [Line 229]
    //                                                ^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_search_tags_excessive_limit() {
    // ... setup ...

    let result = search_tags_impl("test".to_string(), 100000, &state).await;  // <-- ERROR [Line 245]
    //                                                ^^^^^^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_get_popular_tags_negative_limit() {
    // ... setup ...

    let result = get_popular_tags_impl(-10, &state).await;  // <-- ERROR [Line 261]
    //                                 ^^^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_get_popular_tags_zero_limit() {
    // ... setup ...

    let result = get_popular_tags_impl(0, &state).await;  // <-- ERROR [Line 275]
    //                                 ^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_get_popular_tags_empty_database() {
    // ... setup ...

    let result = get_popular_tags_impl(10, &state).await;  // <-- ERROR [Line 290]
    //                                 ^^ expected Option<i32>, found integer

    // ... assertions ...
}

#[tokio::test]
async fn test_tags_sql_injection_prevention() {
    // ... setup ...

    // ... test code ...

    let verify = search_tags_impl("test".to_string(), 10, &state).await;  // <-- ERROR [Line 317]
    //                                                ^^ expected Option<i32>, found integer
    assert!(verify.is_ok(), "Database should still be intact");
}
```

### LINT OUTPUT for tags_error_test.rs:

```
error[E0308]: mismatched types
   --> pipeline/src-tauri/tests/commands/tags_error_test.rs:170:51
    |
170 |     let result = search_tags_impl("".to_string(), 10, &state).await;
    |                  ----------------                 ^^ expected `Option<i32>`, found integer
    |                  |
    |                  arguments to this function are incorrect
    |
    = note: expected enum `std::option::Option<i32>`
               found type `{integer}`
note: function defined here
   --> /home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/commands/tags.rs:115:14
    |
115 | pub async fn search_tags_impl(
    |              ^^^^^^^^^^^^^^^^
help: try wrapping the expression in `Some`
    |
170 |     let result = search_tags_impl("".to_string(), Some(10), &state).await;
    |                                                   +++++  +

error[E0308]: mismatched types
   --> pipeline/src-tauri/tests/commands/tags_error_test.rs:184:77
... (similar errors for lines 202, 215, 229, 245, 261, 275, 290, 317)
```

**ROOT CAUSE:** The function signature changed from `limit: i32` to `limit: Option<i32>` but tests still pass bare integers.

**FIX:** Wrap all limit arguments in `Some()`:
- `search_tags_impl("".to_string(), 10, &state)` → `search_tags_impl("".to_string(), Some(10), &state)`
- `get_popular_tags_impl(10, &state)` → `get_popular_tags_impl(Some(10), &state)`

---

### FILE: pipeline/src-tauri/tests/commands/search_error_test.rs

```rust
/// Search Command Error Path Tests
/// Tests error handling, edge cases, and boundary conditions for search functionality

use midi_pipeline::commands::search::{search_files_impl, SearchFilters, SearchResults};
use midi_pipeline::{AppState, database::Database};

#[tokio::test]
async fn test_search_empty_query() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    let database = Database::new(&db_url).await.expect("Failed to connect to database");
    let state = AppState { database };

    let result = search_files_impl(
        "".to_string(),
        SearchFilters {
            category: None,
            min_bpm: None,
            max_bpm: None,
            key_signature: None,  // <-- This field exists in struct
        },  // <-- ERROR [Line 274 area]: missing field `key_signature` - struct might require more fields
        1,
        20,
        &state,
    ).await;

    // ... assertions ...
}

// Similar pattern in all other tests using SearchFilters
```

### LINT OUTPUT for search_error_test.rs:

```
error[E0063]: missing field `key_signature` in initializer of `midi_pipeline::commands::search::SearchFilters`
   --> pipeline/src-tauri/tests/commands/search_error_test.rs:274:9
    |
274 |         SearchFilters {
    |         ^^^^^^^^^^^^^ missing `key_signature`
```

**ROOT CAUSE:** The `SearchFilters` struct has a required `key_signature` field but tests are missing it (or the struct definition changed).

**FIX:** Add `key_signature: None` to all `SearchFilters` initializers OR make the field optional in the struct.

---

### FILE: pipeline/src-tauri/src/commands/search.rs (Reference - The actual struct definition)

```rust
/// Search filters from frontend
#[derive(Debug, Clone, Deserialize)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key_signature: Option<String>,  // <-- This field exists
}
```

---

### FILE: pipeline/src-tauri/src/commands/tags.rs (Reference - The actual function signature)

```rust
/// Search tags by name prefix (implementation for tests and reuse)
pub async fn search_tags_impl(
    query: String,
    limit: Option<i32>,  // <-- NOTE: Option<i32> not i32
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let repo = TagRepository::new(pool);

    let limit = limit.unwrap_or(10);  // <-- Defaults to 10 if None

    // ... implementation ...
}
```

---

### FILE: pipeline/src-tauri/tests/search_repository_test.rs

```rust
// Multiple errors in this file related to type mismatches in helper functions

#[tokio::test]
async fn test_search_with_min_bpm_greater_than_max_bpm() {
    // Description: Min BPM > Max BPM should fail or return empty
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    // Create test data
    let file_id = create_test_file(&pool, "test.mid").await;
    insert_metadata(&pool, file_id, Some("120.0"), None, None).await;  // <-- POTENTIAL TYPE MISMATCH

    // Query with min > max (logical error)
    let query = SearchQueryBuilder::new().min_bpm(150.0).max_bpm(100.0).build();

    let results = SearchRepository::search(&pool, query, 100, 0)
        .await
        .expect("Query should not error");
    assert!(
        results.is_empty(),
        "Query with min > max should return empty results"
    );

    cleanup_database(&pool).await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_search_with_negative_bpm_filter() {
    // ... similar pattern ...
    let file_id = create_test_file(&pool, "test.mid").await;
    insert_metadata(&pool, file_id, Some("120.0"), None, None).await;  // <-- Line 1352

    // ... rest of test ...
}

#[tokio::test]
async fn test_search_with_invalid_key_filter() {
    // ... similar pattern ...
    let file_id = create_test_file(&pool, "test.mid").await;
    insert_metadata(&pool, file_id, None, Some("C".to_string()), None).await;  // <-- Line 1373

    // ... rest of test ...
}
```

### LINT OUTPUT for search_repository_test.rs:

```
error[E0308]: mismatched types
    --> pipeline/src-tauri/tests/search_repository_test.rs:1329:42
    |
1329|     insert_metadata(&pool, file_id, Some("120.0"), None, None).await;
    |                                     ^^^^^^^^^^^^^ expected different type

error[E0308]: mismatched types
    --> pipeline/src-tauri/tests/search_repository_test.rs:1352:42
... (similar errors for lines 1352, 1373, 1420, 1505, 1667)
```

**ROOT CAUSE:** The `insert_metadata` helper function signature changed but tests still pass old argument types.

**FIX:** Update helper function calls to match the current signature.

---

### FILE: daw/src-tauri/tests/common/mod.rs

```rust
pub mod assertions;
pub mod builders;
/// Common test infrastructure for DAW command tests
/// Provides database mocks, MIDI device mocks, fixtures, builders, and assertions
pub mod database;
pub mod fixtures;
pub mod mocks;

pub use assertions::*;
pub use builders::{MidiFileBuilder, SequencerStateBuilder, TrackBuilder};
pub use database::TestDatabase;
pub use fixtures::{FileFixtures, TestFixtures};
pub use mocks::{EmittedEvent, MockAppHandle, MockMidiDevice, MockWindow};  // <-- ERROR: MockMidiDevice not found

// Re-export commonly used items
pub use sqlx::PgPool;
pub use std::sync::Arc;
pub use tokio::sync::Mutex;
```

### LINT OUTPUT for daw tests common/mod.rs:

```
error[E0432]: unresolved import `mocks::MockMidiDevice`
  --> daw/src-tauri/tests/common/mod.rs:13:45
   |
13 | pub use mocks::{EmittedEvent, MockAppHandle, MockMidiDevice, MockWindow};
   |                                              ^^^^^^^^^^^^^^ no `MockMidiDevice` in `mocks`
```

**ROOT CAUSE:** `MockMidiDevice` is referenced but doesn't exist in the mocks module.

**FIX:** Either add `MockMidiDevice` to mocks.rs OR remove the import.

---

### FILE: daw/src-tauri/tests/common/builders.rs

```rust
/// Test data builders for fluent test construction
use midi_software_center_daw::models::{MidiEvent, MidiEventType};  // <-- POTENTIAL IMPORT ERROR

/// Builder for MIDI files in database
pub struct MidiFileBuilder {
    filepath: String,
    filename: String,
    file_size_bytes: i64,
    num_tracks: i16,
    bpm: Option<f64>,
    key_signature: Option<String>,
}

impl MidiFileBuilder {
    pub fn new(filename: &str) -> Self {
        Self {
            filepath: format!("/test/{}", filename),
            filename: filename.to_string(),
            file_size_bytes: 1024,
            num_tracks: 1,
            bpm: Some(120.0),
            key_signature: Some("C_MAJOR".to_string()),
        }
    }

    // ... builder methods ...

    pub async fn insert(self, pool: &sqlx::PgPool) -> i64 {
        // Generate a simple hash based on timestamp and thread ID
        let hash_str = generate_test_hash();

        let result = sqlx::query!(
            "INSERT INTO files (filepath, filename, content_hash, file_size_bytes, num_tracks)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            self.filepath,
            self.filename,
            hash_str.as_bytes(),
            self.file_size_bytes,
            self.num_tracks
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert test file");

        let file_id = result.id;

        // ... insert metadata ...

        file_id
    }
}

// ... more builders ...
```

### LINT OUTPUT for daw builders.rs:

```
error[E0432]: unresolved import `midi_software_center_daw::models`
 --> daw/src-tauri/tests/common/builders.rs:2:5
  |
2 | use midi_software_center_daw::models::{MidiEvent, MidiEventType};
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `models` in the root of crate `midi_software_center_daw`
```

**ROOT CAUSE:** The models module path has changed or doesn't exist in the DAW crate.

**FIX:** Update import path to correct module location.

---

### FILE: daw/src-tauri/tests/common/assertions.rs

```rust
use sqlx::types::BigDecimal;
/// Custom assertions for DAW command tests
use sqlx::PgPool;
use std::str::FromStr;

/// Assert file exists in database
pub async fn assert_file_exists(pool: &PgPool, file_id: i64) {
    let result = sqlx::query!("SELECT id FROM files WHERE id = $1", file_id)
        .fetch_optional(pool)
        .await
        .expect("Database query failed");

    assert!(
        result.is_some(),
        "Expected file {} to exist in database",
        file_id
    );
}

/// Assert file has specific metadata
pub async fn assert_file_has_metadata(pool: &PgPool, file_id: i64, expected_bpm: Option<f64>) {
    let result = sqlx::query!(
        "SELECT bpm FROM musical_metadata WHERE file_id = $1",
        file_id
    )
    .fetch_optional(pool)
    .await
    .expect("Database query failed");

    if let Some(expected) = expected_bpm {
        assert!(result.is_some(), "Expected metadata for file {}", file_id);
        let actual = result.unwrap().bpm;
        assert!(actual.is_some(), "Expected BPM for file {}", file_id);

        // Convert BigDecimal to f64 for comparison
        let actual_val = actual.unwrap();
        let expected_bd = BigDecimal::from_str(&expected.to_string())
            .unwrap_or_else(|_| BigDecimal::from_str("0").unwrap());
        let diff = if actual_val > expected_bd {
            (&actual_val - &expected_bd).to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            (&expected_bd - &actual_val).to_string().parse::<f64>().unwrap_or(0.0)
        };

        assert!(diff < 0.01, "Expected BPM {}, got {}", expected, actual_val);
    }
}

// ... more assertions ...

/// Assert result is error with specific message
pub fn assert_error_contains(result: Result<(), String>, expected_msg: &str) {
    assert!(result.is_err(), "Expected error, got Ok");
    let error = result.unwrap_err();
    assert!(
        error.contains(expected_msg),
        "Error '{}' does not contain '{}'",
        error,
        expected_msg
    );
}

// ... more assertions ...
```

---

## 3. RUST WARNINGS SUMMARY

### Total Warnings: 351

### Warning Categories:

| Warning Type | Count | Example |
|-------------|-------|---------|
| `unused_variables` | ~50 | `let drum_analysis = ...` |
| `unused_imports` | ~80 | `use Row` not used |
| `dead_code` | ~100 | Structs/functions never used |
| `unused_fields` | ~40 | `config` field never read |
| `unused_must_use` | ~20 | Result not handled |
| Other | ~60 | Various minor warnings |

### Key Unused Code (Dead Code Warnings):

```rust
// pipeline/src-tauri/src/commands/analyze.rs
warning: struct `ControllerStats` is never constructed
    --> pipeline/src-tauri/src/commands/analyze.rs:1180:8
     |
1180 | struct ControllerStats {
     |        ^^^^^^^^^^^^^^^

warning: struct `ArticulationAnalysis` is never constructed
    --> pipeline/src-tauri/src/commands/analyze.rs:1289:8
     |
1289 | struct ArticulationAnalysis {
     |        ^^^^^^^^^^^^^^^^^^^^

// pipeline/src-tauri/src/core/pipeline/workers/*.rs
warning: field `config` is never read
  --> pipeline/src-tauri/src/core/pipeline/workers/import.rs:25:5
   |
24 | pub struct ImportWorker {
   |            ------------ field in this struct
25 |     config: ImportWorkerConfig,
   |     ^^^^^^

// (Similar warnings for SanitizeWorker, SplitWorker, AnalyzeWorker, RenameWorker, ExportWorker)

// pipeline/src-tauri/src/bin/find_duplicates.rs
warning: field `hash` is never read
  --> pipeline/src-tauri/src/bin/find_duplicates.rs:17:5

warning: function `format_number` is never used
   --> pipeline/src-tauri/src/bin/find_duplicates.rs:176:4
```

---

## 4. A11Y WARNINGS

### Total A11y Warnings: 10 (MenuBar.svelte)

### FILE: app/src/lib/components/MenuBar.svelte

```svelte
<!-- Line 390-440: Preferences Dialog -->
{#if showPreferencesDialog}
  <!-- WARNING: Click handler on div without keyboard handler -->
  <!-- WARNING: Div with click handler needs ARIA role -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]" on:click={closePreferences}>
    <div class="dark:bg-window p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4" on:click|stopPropagation>
      <h2 class="text-xl font-bold mb-4">Preferences</h2>

      <div class="space-y-4">
        <!-- Theme Selection -->
        <div>
          <!-- WARNING: Label not associated with control -->
          <label class="block text-sm font-medium mb-2">Theme</label>
          <select class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full">
            <option value="dark" selected>Dark</option>
            <option value="light">Light</option>
          </select>
        </div>

        <!-- Audio Settings -->
        <div>
          <!-- WARNING: Label not associated with control -->
          <label class="block text-sm font-medium mb-2">Audio Buffer Size</label>
          <select class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full">
            <option value="256">256 samples</option>
            <option value="512" selected>512 samples</option>
            <option value="1024">1024 samples</option>
            <option value="2048">2048 samples</option>
          </select>
        </div>

        <!-- Database Settings -->
        <div>
          <!-- WARNING: Label not associated with control -->
          <label class="block text-sm font-medium mb-2">Database Connection</label>
          <input
            type="text"
            class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full"
            value="postgresql://localhost:5432/midi_library"
            readonly
          />
        </div>
      </div>

      <!-- buttons -->
    </div>
  </div>
{/if}

<!-- Line 446-520: Keyboard Shortcuts Dialog -->
{#if showKeyboardShortcuts}
  <!-- Similar warnings as above -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]" on:click={closeKeyboardShortcuts}>
    <!-- ... content ... -->
  </div>
{/if}
```

### LINT OUTPUT for MenuBar.svelte:

```
/home/dojevou/projects/midi-software-center/app/src/lib/components/MenuBar.svelte:396:11
Warn: A11y: A form label must be associated with a control. (svelte)

/home/dojevou/projects/midi-software-center/app/src/lib/components/MenuBar.svelte:405:11
Warn: A11y: A form label must be associated with a control. (svelte)

/home/dojevou/projects/midi-software-center/app/src/lib/components/MenuBar.svelte:416:11
Warn: A11y: A form label must be associated with a control. (svelte)

/home/dojevou/projects/midi-software-center/app/src/lib/components/MenuBar.svelte:390:5
Warn: A11y: visible, non-interactive elements with an on:click event must be accompanied by a keyboard event handler.

/home/dojevou/projects/midi-software-center/app/src/lib/components/MenuBar.svelte:390:5
Warn: A11y: <div> with click handler must have an ARIA role (svelte)

... (similar warnings for keyboard shortcuts dialog at lines 446-447)
```

**FIX for labels:**
```svelte
<!-- Before -->
<label class="block text-sm font-medium mb-2">Theme</label>
<select class="...">

<!-- After -->
<label for="theme-select" class="block text-sm font-medium mb-2">Theme</label>
<select id="theme-select" class="...">
```

**FIX for click handlers on divs:**
```svelte
<!-- Before -->
<div class="..." on:click={closePreferences}>

<!-- After -->
<div
  class="..."
  role="button"
  tabindex="0"
  on:click={closePreferences}
  on:keydown={(e) => e.key === 'Escape' && closePreferences()}
>
```

---

## SUMMARY

| Category | Errors | Warnings |
|----------|--------|----------|
| TypeScript/Svelte | 9 | 10 (A11y) |
| Rust Production | 0 | 22 |
| Rust Tests | 49 | 329 |
| **TOTAL** | **58** | **361** |

### Priority Fix Order:

1. **TypeScript types.ts** (5 min) - Fix interface definitions to match usage
2. **TypeScript api.ts** (2 min) - Add index signature or cast
3. **TypeScript archiveStore.ts** (3 min) - Fix property names
4. **Rust tags_error_test.rs** (10 min) - Wrap limits in `Some()`
5. **Rust search_error_test.rs** (5 min) - Add missing field
6. **Rust search_repository_test.rs** (30 min) - Fix helper function calls
7. **Rust daw/tests/common** (15 min) - Fix imports and missing types
8. **A11y warnings** (15 min) - Add IDs, roles, keyboard handlers

**Estimated Total Fix Time:** 1.5-2 hours
