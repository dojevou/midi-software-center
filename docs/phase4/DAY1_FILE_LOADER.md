# Phase 4, Day 1: File Loader Command

**Time Estimate:** 2-3 hours
**Prerequisite:** Phase 3 complete (automation, presets, projects working)

## Overview

This part implements the core integration between VIP3 browser and DAW sequencer. We'll create a convenience wrapper command `load_file_to_daw` that fetches a file by ID from the database, parses it, and adds it to the sequencer.

**What we're building:**
- `load_file_to_daw(file_id)` Tauri command
- File loader service in Rust
- Error handling for all edge cases
- Integration API in TypeScript

**Why this matters:**
- Central point for VIP3 → DAW file loading
- Consistent error handling across all load methods
- Reusable for double-click, drag-and-drop, and programmatic loading

---

## Step 1: File Loader Service

### File: `app/src-tauri/src/daw/integration/loader.rs`

```rust
use crate::core::midi::MidiParser;
use crate::db::repositories::FileRepository;
use crate::daw::sequencer::Track;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct FileLoaderService {
    db_pool: PgPool,
    parser: Arc<MidiParser>,
}

impl FileLoaderService {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            parser: Arc::new(MidiParser::new()),
        }
    }

    /// Load a MIDI file from database and create a Track
    pub async fn load_file_by_id(&self, file_id: i64) -> Result<Track, FileLoaderError> {
        // Step 1: Fetch file metadata from database
        let repo = FileRepository::new(self.db_pool.clone());
        let file_record = repo
            .get_by_id(file_id)
            .await
            .map_err(|e| FileLoaderError::DatabaseError(e.to_string()))?
            .ok_or_else(|| FileLoaderError::FileNotFound(file_id))?;

        // Step 2: Verify file exists on disk
        let file_path = PathBuf::from(&file_record.file_path);
        if !file_path.exists() {
            return Err(FileLoaderError::FileNotOnDisk(file_path));
        }

        // Step 3: Parse MIDI file
        let midi_data = self
            .parser
            .parse_file(&file_path)
            .map_err(|e| FileLoaderError::ParseError(e.to_string()))?;

        // Step 4: Create Track from parsed data
        let track = Track {
            id: 0, // Will be assigned by sequencer
            file_id: Some(file_id),
            name: file_record.original_filename.clone(),
            file_path: Some(file_path),
            midi_data,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            armed: false,
            position_ticks: 0,
        };

        Ok(track)
    }

    /// Load multiple files by IDs
    pub async fn load_files_by_ids(&self, file_ids: Vec<i64>) -> Vec<Result<Track, FileLoaderError>> {
        let mut results = Vec::new();

        for file_id in file_ids {
            results.push(self.load_file_by_id(file_id).await);
        }

        results
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileLoaderError {
    #[error("File not found in database: {0}")]
    FileNotFound(i64),

    #[error("File path does not exist on disk: {0}")]
    FileNotOnDisk(PathBuf),

    #[error("Failed to parse MIDI file: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Sequencer error: {0}")]
    SequencerError(String),
}
```

### File: `app/src-tauri/src/daw/integration/mod.rs`

```rust
pub mod loader;

pub use loader::{FileLoaderError, FileLoaderService};
```

**Update:** `app/src-tauri/src/daw/mod.rs`

```rust
pub mod automation;
pub mod integration;  // Add this
pub mod mixer;
pub mod presets;
pub mod project;
pub mod sequencer;
```

---

## Step 2: Tauri Command

### File: `app/src-tauri/src/commands/integration/file_loader.rs`

```rust
use crate::daw::integration::FileLoaderService;
use crate::daw::sequencer::SequencerEngine;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Load a MIDI file from VIP3 into the DAW sequencer
///
/// # Arguments
/// * `file_id` - Database ID of the file to load
///
/// # Returns
/// * `Ok(track_id)` - Track was added successfully, returns the new track ID
/// * `Err(message)` - Error occurred during loading
#[tauri::command]
pub async fn load_file_to_daw(
    file_id: i64,
    loader: State<'_, Arc<FileLoaderService>>,
    sequencer: State<'_, Arc<Mutex<SequencerEngine>>>,
) -> Result<u32, String> {
    // Load file and create track
    let track = loader
        .load_file_by_id(file_id)
        .await
        .map_err(|e| e.to_string())?;

    // Add track to sequencer
    let mut seq = sequencer.lock().await;
    let track_id = seq.add_track(track).map_err(|e| e.to_string())?;

    Ok(track_id)
}

/// Load multiple files into the DAW sequencer
///
/// # Arguments
/// * `file_ids` - Array of database IDs to load
///
/// # Returns
/// * `Vec<Result<u32, String>>` - Results for each file (track ID or error)
#[tauri::command]
pub async fn load_files_to_daw(
    file_ids: Vec<i64>,
    loader: State<'_, Arc<FileLoaderService>>,
    sequencer: State<'_, Arc<Mutex<SequencerEngine>>>,
) -> Result<Vec<Result<u32, String>>, String> {
    let mut results = Vec::new();

    for file_id in file_ids {
        match loader.load_file_by_id(file_id).await {
            Ok(track) => {
                let mut seq = sequencer.lock().await;
                match seq.add_track(track) {
                    Ok(track_id) => results.push(Ok(track_id)),
                    Err(e) => results.push(Err(e.to_string())),
                }
            }
            Err(e) => results.push(Err(e.to_string())),
        }
    }

    Ok(results)
}

/// Get information about a file without loading it
///
/// # Arguments
/// * `file_id` - Database ID of the file
///
/// # Returns
/// * File metadata (name, path, duration, etc.)
#[tauri::command]
pub async fn get_file_info_for_daw(
    file_id: i64,
    loader: State<'_, Arc<FileLoaderService>>,
) -> Result<FileInfo, String> {
    let track = loader
        .load_file_by_id(file_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(FileInfo {
        name: track.name,
        path: track.file_path.map(|p| p.to_string_lossy().to_string()),
        track_count: track.midi_data.tracks.len(),
        duration_ticks: track.midi_data.duration_ticks,
        tempo_bpm: track.midi_data.tempo_bpm,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: Option<String>,
    pub track_count: usize,
    pub duration_ticks: u64,
    pub tempo_bpm: f32,
}
```

### File: `app/src-tauri/src/commands/integration/mod.rs`

```rust
pub mod file_loader;

pub use file_loader::{get_file_info_for_daw, load_file_to_daw, load_files_to_daw};
```

**Update:** `app/src-tauri/src/commands/mod.rs`

```rust
pub mod daw;
pub mod integration;  // Add this
pub mod pipeline;
```

---

## Step 3: Register Commands and State

### Update: `app/src-tauri/src/lib.rs`

```rust
use crate::daw::integration::FileLoaderService;
use std::sync::Arc;

pub fn run() {
    // ... existing setup ...

    // Create database pool
    let db_pool = /* your existing pool creation */;

    // Create file loader service
    let file_loader = Arc::new(FileLoaderService::new(db_pool.clone()));

    tauri::Builder::default()
        .manage(file_loader.clone())  // Add this
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...

            // Integration commands
            commands::integration::load_file_to_daw,
            commands::integration::load_files_to_daw,
            commands::integration::get_file_info_for_daw,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Step 4: TypeScript Integration API

### File: `app/src/lib/api/integrationApi.ts`

```typescript
import { invoke } from '@tauri-apps/api/tauri';

export interface FileInfo {
  name: string;
  path: string | null;
  track_count: number;
  duration_ticks: number;
  tempo_bpm: number;
}

export class IntegrationApi {
  /**
   * Load a MIDI file from VIP3 into the DAW sequencer
   * @param fileId - Database ID of the file
   * @returns Track ID if successful
   */
  static async loadFileToDaw(fileId: number): Promise<number> {
    try {
      const trackId = await invoke<number>('load_file_to_daw', { fileId });
      return trackId;
    } catch (error) {
      throw new Error(`Failed to load file ${fileId}: ${error}`);
    }
  }

  /**
   * Load multiple files into the DAW
   * @param fileIds - Array of database IDs
   * @returns Array of results (track ID or error message)
   */
  static async loadFilesToDaw(fileIds: number[]): Promise<Array<{ ok: boolean; value: number | string }>> {
    try {
      const results = await invoke<Array<{ Ok?: number; Err?: string }>>('load_files_to_daw', { fileIds });

      return results.map((result) => {
        if ('Ok' in result) {
          return { ok: true, value: result.Ok! };
        } else {
          return { ok: false, value: result.Err! };
        }
      });
    } catch (error) {
      throw new Error(`Failed to load files: ${error}`);
    }
  }

  /**
   * Get file information without loading it
   * @param fileId - Database ID of the file
   * @returns File metadata
   */
  static async getFileInfo(fileId: number): Promise<FileInfo> {
    try {
      const info = await invoke<FileInfo>('get_file_info_for_daw', { fileId });
      return info;
    } catch (error) {
      throw new Error(`Failed to get file info: ${error}`);
    }
  }
}
```

**Export:** Update `app/src/lib/api/index.ts`

```typescript
export * from './automationApi';
export * from './integrationApi';  // Add this
export * from './mixerApi';
export * from './presetApi';
export * from './projectApi';
export * from './sequencerApi';
```

---

## Step 5: Error Handling and Logging

### Update: `app/src-tauri/src/daw/integration/loader.rs`

Add logging to help debug issues:

```rust
use tracing::{error, info, warn};

impl FileLoaderService {
    pub async fn load_file_by_id(&self, file_id: i64) -> Result<Track, FileLoaderError> {
        info!("Loading file {} from database", file_id);

        // Step 1: Fetch file metadata
        let repo = FileRepository::new(self.db_pool.clone());
        let file_record = match repo.get_by_id(file_id).await {
            Ok(Some(record)) => {
                info!("Found file record: {}", record.original_filename);
                record
            }
            Ok(None) => {
                error!("File {} not found in database", file_id);
                return Err(FileLoaderError::FileNotFound(file_id));
            }
            Err(e) => {
                error!("Database error fetching file {}: {}", file_id, e);
                return Err(FileLoaderError::DatabaseError(e.to_string()));
            }
        };

        // Step 2: Verify file exists
        let file_path = PathBuf::from(&file_record.file_path);
        if !file_path.exists() {
            error!("File path does not exist: {}", file_path.display());
            return Err(FileLoaderError::FileNotOnDisk(file_path));
        }
        info!("File exists on disk: {}", file_path.display());

        // Step 3: Parse MIDI
        info!("Parsing MIDI file...");
        let midi_data = match self.parser.parse_file(&file_path) {
            Ok(data) => {
                info!(
                    "Parsed successfully: {} tracks, {} ticks duration",
                    data.tracks.len(),
                    data.duration_ticks
                );
                data
            }
            Err(e) => {
                error!("Failed to parse MIDI file: {}", e);
                return Err(FileLoaderError::ParseError(e.to_string()));
            }
        };

        // Step 4: Create track
        let track = Track {
            id: 0,
            file_id: Some(file_id),
            name: file_record.original_filename.clone(),
            file_path: Some(file_path),
            midi_data,
            gain: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            armed: false,
            position_ticks: 0,
        };

        info!("Track created successfully for file {}", file_id);
        Ok(track)
    }
}
```

---

## Step 6: Unit Tests

### File: `app/src-tauri/tests/integration/file_loader_test.rs`

```rust
use midi_app::daw::integration::FileLoaderService;
use sqlx::PgPool;
use std::path::PathBuf;

#[tokio::test]
async fn test_load_file_by_id_success() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect");

    let loader = FileLoaderService::new(pool);

    // Assume file ID 1 exists in test database
    let result = loader.load_file_by_id(1).await;

    match result {
        Ok(track) => {
            assert!(track.name.len() > 0, "Track should have a name");
            assert!(track.file_id == Some(1), "Track should have correct file_id");
            println!("✓ Loaded track: {}", track.name);
        }
        Err(e) => {
            println!("✗ Failed to load: {}", e);
            panic!("Test failed");
        }
    }
}

#[tokio::test]
async fn test_load_nonexistent_file() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect");

    let loader = FileLoaderService::new(pool);

    // Use a very high ID that definitely doesn't exist
    let result = loader.load_file_by_id(999999999).await;

    assert!(result.is_err(), "Should fail for non-existent file");

    match result {
        Err(e) => println!("✓ Correctly rejected non-existent file: {}", e),
        Ok(_) => panic!("Should have failed"),
    }
}

#[tokio::test]
async fn test_load_multiple_files() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect");

    let loader = FileLoaderService::new(pool);

    // Load files 1, 2, 3
    let results = loader.load_files_by_ids(vec![1, 2, 3]).await;

    assert_eq!(results.len(), 3, "Should have 3 results");

    let success_count = results.iter().filter(|r| r.is_ok()).count();
    println!("✓ Successfully loaded {}/3 files", success_count);
}
```

**Run Tests:**
```bash
cargo test --test file_loader_test -- --nocapture
```

---

## Verification Checklist

### Manual Testing

1. **Test successful file load:**
```bash
# In Rust console or via Tauri DevTools
load_file_to_daw(1)  # Should return track_id
```

2. **Test error cases:**
```bash
# Non-existent file
load_file_to_daw(999999999)  # Should return error

# Invalid file ID
load_file_to_daw(-1)  # Should return error
```

3. **Test multiple files:**
```bash
load_files_to_daw([1, 2, 3])  # Should return array of results
```

4. **Test file info:**
```bash
get_file_info_for_daw(1)  # Should return file metadata
```

### Expected Output

**Successful load:**
```
INFO  Loading file 1 from database
INFO  Found file record: my_midi_file.mid
INFO  File exists on disk: /path/to/my_midi_file.mid
INFO  Parsing MIDI file...
INFO  Parsed successfully: 3 tracks, 3840 ticks duration
INFO  Track created successfully for file 1
```

**File not found:**
```
ERROR File 999999999 not found in database
```

**File not on disk:**
```
ERROR File path does not exist: /path/to/missing_file.mid
```

**Parse error:**
```
ERROR Failed to parse MIDI file: Invalid MIDI header
```

---

## Troubleshooting

### Issue: "File not found in database"

**Symptom:**
```
Error: File not found in database: 123
```

**Solution:**
1. Verify file exists: `SELECT * FROM files WHERE id = 123;`
2. Check database connection
3. Ensure correct file_id is passed

---

### Issue: "File path does not exist on disk"

**Symptom:**
```
Error: File path does not exist on disk: /path/to/file.mid
```

**Solution:**
1. Check `file_path` column in database is correct
2. Verify file hasn't been moved or deleted
3. Check file permissions
4. Re-import file if necessary

---

### Issue: "Failed to parse MIDI file"

**Symptom:**
```
Error: Failed to parse MIDI file: Invalid MIDI header
```

**Solution:**
1. Verify file is valid MIDI: `file /path/to/file.mid`
2. Check file isn't corrupted
3. Try opening in another MIDI application
4. Check parser logs for detailed error

---

### Issue: Track not appearing in sequencer

**Symptom:**
Command returns track_id but track doesn't appear

**Solution:**
1. Check sequencer state is initialized
2. Verify `add_track()` succeeded
3. Check frontend is polling sequencer state
4. Refresh sequencer UI component

---

## What's Next?

You've completed Day 1! The file loader command is now functional.

**Next Steps:**
1. Move to [Day 2: Double-Click Integration](./DAY2_DOUBLE_CLICK.md)
2. Integrate file loader with VIP3 UI
3. Add user-friendly notifications

**What you've built:**
- ✅ `load_file_to_daw` command
- ✅ Error handling for all edge cases
- ✅ File loader service
- ✅ TypeScript integration API
- ✅ Unit tests

**What's coming:**
- Double-click file cards to load
- Success/error notifications
- Auto-switch to DAW tab (optional)
