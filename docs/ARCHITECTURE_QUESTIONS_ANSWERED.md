# Architecture Questions - Complete Answers

Generated: 2025-12-01
Purpose: Direct answers to all architecture questions for code modifications

---

## Backend - Unified App (app/src-tauri/src/)

### 12-13. `app/src-tauri/src/main.rs`

#### Q1: Current AppState structure

```rust
/// Combined application state
#[allow(dead_code)]
struct AppState {
    pipeline: PipelineState,
    db_pool: Option<sqlx::PgPool>,
}
```

**Note:** `PipelineState` is imported from `midi_pipeline::AppState as PipelineState`

#### Q2: How commands are currently registered (builder pattern)

```rust
tauri::Builder::default()
    .manage(state)
    .manage(daw_app_state)
    .manage(midi_manager)
    .manage(sequencer_engine)
    .manage(daw_state)
    .manage(daw_window_state)
    .manage(automation_state)
    .manage(mixer_state)
    .invoke_handler(tauri::generate_handler![
        // 100+ commands listed here
        shutdown_application,
        midi_pipeline::commands::files::test_db_connection,
        // ... etc
    ])
    .setup(|_app| {
        info!("✅ Application setup complete");
        Ok(())
    })
    .run(tauri::generate_context!())?;
```

#### Q3: Existing states already registered

| State Variable | Type | Source |
|----------------|------|--------|
| `state` | `AppState` | Local struct (pipeline + db_pool) |
| `daw_app_state` | `DawAppState` | `midi_software_center_daw::commands::AppState` |
| `midi_manager` | `Arc<MidiManager>` | `midi_software_center_daw::midi::MidiManager` |
| `sequencer_engine` | `Arc<SequencerEngine>` | `midi_software_center_daw::sequencer::SequencerEngine` |
| `daw_state` | `DawState` | `midi_software_center_daw::commands::DawState::default()` |
| `daw_window_state` | `DAWState` | `midi_software_center_daw::commands::DAWState::default()` |
| `automation_state` | `AutomationState` | `midi_software_center_daw::commands::AutomationState::new()` |
| `mixer_state` | `MixerState` | `midi_software_center_daw::commands::mixer::MixerState::default()` |

#### Q4: Current command invocations

**App Lifecycle:**
- `shutdown_application` (local)

**Pipeline - Files:**
- `midi_pipeline::commands::files::test_db_connection`
- `midi_pipeline::commands::files::get_file_count`
- `midi_pipeline::commands::files::get_file_details`
- `midi_pipeline::commands::files::get_file`
- `midi_pipeline::commands::files::list_files`
- `midi_pipeline::commands::files::get_files_by_category`
- `midi_pipeline::commands::files::get_recent_files`
- `midi_pipeline::commands::files::delete_file`

**Pipeline - Import:**
- `midi_pipeline::commands::file_import::import_single_file`
- `midi_pipeline::commands::file_import::import_directory`
- `midi_pipeline::commands::archive_import::import_archive_collection`

**Pipeline - Search:**
- `midi_pipeline::commands::search::get_all_tags`
- `midi_pipeline::commands::search::get_files_by_tag`
- `midi_pipeline::commands::search::get_bpm_range`
- `midi_pipeline::commands::search::get_all_keys`

**Pipeline - Analysis:**
- `midi_pipeline::commands::analyze::start_analysis`

**Pipeline - Stats:**
- `midi_pipeline::commands::stats::get_category_stats`
- `midi_pipeline::commands::stats::get_manufacturer_stats`
- `midi_pipeline::commands::stats::get_key_signature_stats`
- `midi_pipeline::commands::stats::get_recently_added_count`
- `midi_pipeline::commands::stats::get_duplicate_count`
- `midi_pipeline::commands::stats::get_database_size`
- `midi_pipeline::commands::stats::check_database_health`

**Pipeline - Tags:**
- `midi_pipeline::commands::tags::get_file_tags`
- `midi_pipeline::commands::tags::get_popular_tags`
- `midi_pipeline::commands::tags::search_tags`
- `midi_pipeline::commands::tags::get_tag_categories`
- `midi_pipeline::commands::tags::get_tags_by_category`
- `midi_pipeline::commands::tags::update_file_tags`
- `midi_pipeline::commands::tags::add_tags_to_file`
- `midi_pipeline::commands::tags::remove_tag_from_file`
- `midi_pipeline::commands::tags::get_files_by_tags`
- `midi_pipeline::commands::tags::get_tag_stats`

**Pipeline - Progress:**
- `midi_pipeline::commands::progress::start_progress_tracking`
- `midi_pipeline::commands::progress::update_progress`
- `midi_pipeline::commands::progress::increment_error_count`
- `midi_pipeline::commands::progress::increment_duplicate_count`
- `midi_pipeline::commands::progress::complete_progress`
- `midi_pipeline::commands::progress::get_current_progress`
- `midi_pipeline::commands::progress::reset_progress`

**Pipeline - System:**
- `midi_pipeline::commands::system::get_system_info`

**DAW - Database:**
- `midi_software_center_daw::commands::initialize_database`

**DAW - MIDI:**
- `midi_software_center_daw::commands::midi_list_devices`
- `midi_software_center_daw::commands::midi_connect`
- `midi_software_center_daw::commands::midi_disconnect`
- `midi_software_center_daw::commands::midi_is_connected`
- `midi_software_center_daw::commands::midi_get_current_device`
- `midi_software_center_daw::commands::midi_send_test_note`

**DAW - Sequencer:**
- `midi_software_center_daw::commands::start_sequencer`
- `midi_software_center_daw::commands::stop_sequencer`
- `midi_software_center_daw::commands::pause_sequencer`
- `midi_software_center_daw::commands::resume_sequencer`
- `midi_software_center_daw::commands::get_playback_position`
- `midi_software_center_daw::commands::seek_position`
- `midi_software_center_daw::commands::set_tempo`
- `midi_software_center_daw::commands::get_tempo`
- `midi_software_center_daw::commands::add_track`
- `midi_software_center_daw::commands::remove_track`
- `midi_software_center_daw::commands::update_track`
- `midi_software_center_daw::commands::get_tracks`
- `midi_software_center_daw::commands::load_sequencer_tracks`
- `midi_software_center_daw::commands::is_sequencer_playing`

**DAW - Search:**
- `midi_software_center_daw::commands::search_files`
- `midi_software_center_daw::commands::get_file_details`
- `midi_software_center_daw::commands::get_search_suggestions`

**DAW - Analysis:**
- `midi_software_center_daw::commands::find_compatible_files`
- `midi_software_center_daw::commands::add_favorite`
- `midi_software_center_daw::commands::remove_favorite`
- `midi_software_center_daw::commands::is_favorite`
- `midi_software_center_daw::commands::get_favorites`
- `midi_software_center_daw::commands::get_usage_stats`

**DAW - Project:**
- `midi_software_center_daw::commands::load_multiple_tracks`
- `midi_software_center_daw::commands::clear_all_tracks`
- `midi_software_center_daw::commands::get_track_details`

**DAW - Export:**
- `midi_software_center_daw::commands::export_project_midi`

**DAW - Window/Transport:**
- `midi_software_center_daw::commands::play_transport`
- `midi_software_center_daw::commands::stop_transport`
- `midi_software_center_daw::commands::pause_transport`
- `midi_software_center_daw::commands::set_playback_position`
- `midi_software_center_daw::commands::get_playback_state`
- `midi_software_center_daw::commands::set_bpm`
- `midi_software_center_daw::commands::get_bpm`
- `midi_software_center_daw::commands::set_time_signature`
- `midi_software_center_daw::commands::get_time_signature`
- `midi_software_center_daw::commands::set_key_signature`
- `midi_software_center_daw::commands::get_key_signature`

**DAW - Window/Tracks:**
- `midi_software_center_daw::commands::add_window_track`
- `midi_software_center_daw::commands::remove_window_track`
- `midi_software_center_daw::commands::get_all_window_tracks`
- `midi_software_center_daw::commands::set_track_visible`
- `midi_software_center_daw::commands::set_track_muted`
- `midi_software_center_daw::commands::set_track_soloed`
- `midi_software_center_daw::commands::get_track_info`
- `midi_software_center_daw::commands::update_track_label`

**DAW - Window/Mixer:**
- `midi_software_center_daw::commands::get_mixer_state`
- `midi_software_center_daw::commands::set_channel_volume`
- `midi_software_center_daw::commands::set_channel_pan`
- `midi_software_center_daw::commands::set_channel_mute`
- `midi_software_center_daw::commands::set_channel_solo`
- `midi_software_center_daw::commands::get_daw_state`
- `midi_software_center_daw::commands::reset_daw_state`

**DAW - Mixer Module:**
- `midi_software_center_daw::commands::mixer::mixer_get_channels`
- `midi_software_center_daw::commands::mixer::mixer_get_master`
- `midi_software_center_daw::commands::mixer::mixer_add_channel`
- `midi_software_center_daw::commands::mixer::mixer_remove_channel`
- `midi_software_center_daw::commands::mixer::mixer_set_volume`
- `midi_software_center_daw::commands::mixer::mixer_set_pan`
- `midi_software_center_daw::commands::mixer::mixer_set_mute`
- `midi_software_center_daw::commands::mixer::mixer_set_solo`
- `midi_software_center_daw::commands::mixer::mixer_set_master_volume`
- `midi_software_center_daw::commands::mixer::mixer_set_master_limiter`
- `midi_software_center_daw::commands::mixer::mixer_set_master_compressor`
- `midi_software_center_daw::commands::mixer::mixer_add_effect`
- `midi_software_center_daw::commands::mixer::mixer_remove_effect`
- `midi_software_center_daw::commands::mixer::mixer_set_effect_enabled`

**DAW - Automation:**
- `midi_software_center_daw::commands::create_automation_lane`
- `midi_software_center_daw::commands::delete_automation_lane`
- `midi_software_center_daw::commands::add_automation_point`
- `midi_software_center_daw::commands::remove_automation_point`
- `midi_software_center_daw::commands::move_automation_point`
- `midi_software_center_daw::commands::set_automation_curve_type`
- `midi_software_center_daw::commands::get_automation_lane`
- `midi_software_center_daw::commands::get_track_automation`
- `midi_software_center_daw::commands::get_automation_value`
- `midi_software_center_daw::commands::clear_track_automation`
- `midi_software_center_daw::commands::clear_all_automation`

---

## Backend - Pipeline (pipeline/src-tauri/src/)

### 14. `pipeline/src-tauri/src/commands/split_file.rs`

#### Q1: Current split_and_import function signature

```rust
pub async fn split_and_import(
    file_id: i64,
    output_dir: PathBuf,
    pool: &sqlx::PgPool,
) -> Result<SplitResult, SplitCommandError>
```

#### Q2: Current function body (summary)

```rust
pub async fn split_and_import(
    file_id: i64,
    output_dir: PathBuf,
    pool: &sqlx::PgPool,
) -> Result<SplitResult, SplitCommandError> {
    let start = Instant::now();

    // 1. Get source file from database
    let source_file = get_file_by_id(pool, file_id)
        .await?
        .ok_or(SplitCommandError::FileNotFound(file_id))?;

    // 2. Read and parse MIDI file
    let midi_bytes = std::fs::read(&source_file.filepath)?;
    let smf = midly::Smf::parse(&midi_bytes)?;

    // 3. Create output directory
    std::fs::create_dir_all(&output_dir)?;

    // 4. Split tracks
    let mut tracks_created = Vec::new();
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        // Determine channel from first note event
        let channel = find_track_channel(track);

        // Create individual track file
        let track_filename = format!(
            "{}_track{:02}_{}.mid",
            source_file.filename.trim_end_matches(".mid"),
            track_idx,
            channel.map_or("multi".to_string(), |c| format!("ch{}", c))
        );
        let track_path = output_dir.join(&track_filename);

        // Write track to file
        write_single_track(&smf, track_idx, &track_path)?;

        // Import to database
        let imported_id = import_split_track(pool, &track_path, file_id, track_idx as u16).await?;

        tracks_created.push(SplitTrackInfo {
            track_number: track_idx as u16,
            channel,
            file_path: track_path,
            file_id: Some(imported_id),
            event_count: track.len(),
            note_count: count_notes(track),
        });
    }

    Ok(SplitResult {
        source_file_id: file_id,
        tracks_created,
        total_tracks: smf.tracks.len(),
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
```

#### Q3: Any existing #[tauri::command] attributes

```rust
#[tauri::command]
pub async fn split_file(
    file_id: i64,
    output_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<SplitResult, String> {
    let pool = state.database.pool();
    let output_path = PathBuf::from(output_dir);

    split_and_import(file_id, output_path, pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn split_file_batch(
    file_ids: Vec<i64>,
    output_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SplitResult>, String> {
    // Batch splitting implementation
}
```

#### Q4: Error handling pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SplitCommandError {
    #[error("File not found: {0}")]
    FileNotFound(i64),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("MIDI parse error: {0}")]
    MidiParse(#[from] midly::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No tracks in file")]
    NoTracks,
}
```

---

### 15. `pipeline/src-tauri/src/commands/mod.rs`

#### Q1: Current exports

```rust
pub mod analyze;
pub mod archive_import;
pub mod file_import;
pub mod progress;
pub mod search;
pub mod split_file;
pub mod stats;
pub mod system;
pub mod tags;
```

#### Q2: Current module declarations

Same as exports - each `pub mod` declares the module.

#### Q3: Any existing pub use statements

```rust
pub use analyze::*;
pub use archive_import::*;
pub use file_import::*;
pub use progress::*;
pub use search::*;
pub use split_file::*;
pub use stats::*;
pub use system::*;
pub use tags::*;
```

---

### 16. `pipeline/src-tauri/src/core/analysis/arena_midi.rs`

#### Q1: Current code structure

```rust
use bumpalo::Bump;
use std::marker::PhantomData;

/// Arena-allocated MIDI file for zero-copy parsing
pub struct ArenaMidiFile<'arena> {
    pub format: u16,
    pub num_tracks: u16,
    pub ticks_per_quarter_note: u16,
    tracks: Vec<ArenaTrack<'arena>>,
    _marker: PhantomData<&'arena ()>,
}

pub struct ArenaTrack<'arena> {
    pub track_number: u16,
    pub events: &'arena [ArenaEvent<'arena>],
    pub name: Option<&'arena str>,
    pub channel: Option<u8>,
}

pub struct ArenaEvent<'arena> {
    pub delta_time: u32,
    pub event_type: ArenaEventType<'arena>,
}

pub enum ArenaEventType<'arena> {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },
    Controller { channel: u8, controller: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchBend { channel: u8, value: i16 },
    Meta(ArenaMetaEvent<'arena>),
    SysEx(&'arena [u8]),
}
```

#### Q2: Specific lifetime errors mentioned

The file uses `'arena` lifetime parameter tied to `bumpalo::Bump`. Potential issues:

1. **Lifetime covariance:** `PhantomData<&'arena ()>` ensures proper variance
2. **Event slice lifetime:** `&'arena [ArenaEvent<'arena>]` requires arena to outlive all references
3. **String lifetime:** Track names use `&'arena str` allocated in arena

**No explicit lifetime errors in current code** - the design is sound.

#### Q3: Any #[allow(unused)] or #[cfg(feature = "disabled")]

```rust
#[allow(dead_code)]  // On some helper functions
#[cfg(feature = "arena-parsing")]  // Feature-gated module
```

The arena parsing is **feature-gated** and not enabled by default in Cargo.toml.

#### Q4: Related structs and their lifetimes

```rust
// All use 'arena lifetime tied to Bump allocator
ArenaMidiFile<'arena>
ArenaTrack<'arena>
ArenaEvent<'arena>
ArenaEventType<'arena>
ArenaMetaEvent<'arena>

// Usage pattern:
impl<'arena> ArenaMidiFile<'arena> {
    pub fn parse(arena: &'arena Bump, bytes: &[u8]) -> Result<Self, ParseError> {
        // All allocations go into arena
        let tracks = arena.alloc_slice_fill_iter(parsed_tracks);
        // ...
    }
}
```

---

### 17. `pipeline/src-tauri/src/core/analysis/simd_bpm.rs`

#### Q1: Current disabled code

The SIMD functions have fallback implementations:

```rust
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
unsafe fn calculate_intervals_avx2(ticks: &[u64]) -> Vec<u64> {
    // AVX2 SIMD implementation
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
fn calculate_intervals_avx2(ticks: &[u64]) -> Vec<u64> {
    // Scalar fallback
    ticks.windows(2).map(|w| w[1] - w[0]).collect()
}
```

#### Q2: Any SIMD feature flags

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Runtime feature detection
fn has_avx2() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}
```

#### Q3: Current BPM detection implementation

```rust
pub fn detect_bpm_from_onsets(midi_file: &MidiFile) -> Option<OnsetBpmResult> {
    // 1. Extract note onsets
    let onsets = extract_onsets(midi_file);
    if onsets.len() < 4 {
        return None;
    }

    // 2. Calculate inter-onset intervals
    let intervals = if has_avx2() {
        unsafe { calculate_intervals_avx2(&onsets) }
    } else {
        calculate_intervals_scalar(&onsets)
    };

    // 3. Build tempo histogram
    let histogram = build_tempo_histogram(
        &intervals,
        midi_file.ticks_per_quarter_note,
    );

    // 4. Find dominant tempo
    let (bpm, confidence) = find_peak(&histogram)?;

    Some(OnsetBpmResult {
        bpm,
        confidence,
        onset_count: onsets.len(),
        tempo_histogram: histogram,
    })
}
```

#### Q4: How it integrates with the pipeline

```rust
// In pipeline/src-tauri/src/core/analysis/mod.rs
pub mod bpm_detector;
pub mod simd_bpm;  // Optional SIMD version

// In bpm_detector.rs
use super::simd_bpm;

pub fn detect_bpm(midi_file: &MidiFile) -> Option<f64> {
    // Try SIMD version first
    if let Some(result) = simd_bpm::detect_bpm_from_onsets(midi_file) {
        if result.confidence > 0.7 {
            return Some(result.bpm);
        }
    }

    // Fall back to standard detection
    detect_bpm_standard(midi_file)
}
```

---

## Backend - DAW (daw/src-tauri/src/)

### 18. `daw/src-tauri/src/commands/export.rs`

#### Q1: Current dummy export function

```rust
#[tauri::command]
pub async fn export_project_midi(output_path: String) -> Result<(), String> {
    let path = PathBuf::from(output_path);

    // TODO: Get actual tracks from sequencer state
    // Currently creates demo data for testing
    let events = create_demo_events();

    crate::midi::writer::write_midi_file(&path, &events, 480)
        .map_err(|e| e.to_string())
}

fn create_demo_events() -> Vec<MidiEvent> {
    vec![
        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 0,
            channel: 0,
            note: Some(60),
            velocity: Some(80),
            controller: None,
            value: None,
            program: None,
        },
        MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: 480,
            channel: 0,
            note: Some(60),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        },
        // ... more demo events
    ]
}
```

#### Q2: Track data structure definition

```rust
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,
    pub channel: u8,
    pub note: Option<u8>,
    pub velocity: Option<u8>,
    pub controller: Option<u8>,
    pub value: Option<u8>,
    pub program: Option<u8>,
}

pub enum MidiEventType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Meta,
}
```

#### Q3: Expected MIDI export format

Standard MIDI File (SMF) Format 1:
- Header chunk with format, track count, ticks per quarter
- One track per channel
- Delta-time encoded events
- End-of-track meta events

#### Q4: Any existing MIDI library usage

```rust
// Uses custom writer module
use crate::midi::writer;

// writer.rs uses midly for low-level MIDI writing
use midly::{Smf, Header, Track, TrackEvent, MidiMessage};
```

---

### 19. `daw/src-tauri/src/sequencer/engine.rs`

#### Q1: Current stop() function implementation

```rust
pub async fn stop(&self) -> Result<(), SequencerError> {
    // 1. Update playback state
    {
        let mut state = self.state.write().await;
        *state = PlaybackState::Stopped;
    }

    // 2. Abort playback thread if running
    {
        let mut handle = self.playback_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }
    }

    // 3. Reset position to zero
    {
        let mut tick = self.current_tick.write().await;
        *tick = 0;
    }

    // 4. Send all notes off (MIDI panic)
    self.midi_manager.send_all_notes_off().await?;

    Ok(())
}
```

#### Q2: Position tracking variables

```rust
pub struct SequencerEngine {
    // ...
    state: Arc<RwLock<PlaybackState>>,
    current_tick: Arc<RwLock<u64>>,
    bpm: Arc<RwLock<f32>>,
    ticks_per_quarter: u16,  // Usually 480
    // ...
}

pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}
```

#### Q3: Playback state management

```rust
impl SequencerEngine {
    pub async fn play(&self) -> Result<(), SequencerError> {
        let mut state = self.state.write().await;
        match *state {
            PlaybackState::Stopped | PlaybackState::Paused => {
                *state = PlaybackState::Playing;
                self.start_playback_thread().await?;
            }
            PlaybackState::Playing => {
                // Already playing, no-op
            }
        }
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), SequencerError> {
        let mut state = self.state.write().await;
        if *state == PlaybackState::Playing {
            *state = PlaybackState::Paused;
            // Position preserved, thread paused
        }
        Ok(())
    }

    pub async fn is_playing(&self) -> bool {
        *self.state.read().await == PlaybackState::Playing
    }
}
```

#### Q4: Any existing reset logic

```rust
pub async fn reset(&self) -> Result<(), SequencerError> {
    self.stop().await?;

    // Clear all tracks
    self.track_manager.clear().await;

    // Reset scheduler
    self.scheduler.clear().await;

    // Reset tempo to default
    {
        let mut bpm = self.bpm.write().await;
        *bpm = 120.0;
    }

    Ok(())
}
```

---

### 20. `daw/src-tauri/src/commands/window.rs`

#### Q1: Current undo/redo command signatures

**ANSWER: Undo/redo commands are NOT in window.rs**

They are in `daw/src-tauri/src/undo_redo/commands.rs`:

```rust
#[tauri::command]
pub async fn undo(
    state: tauri::State<'_, UndoRedoState>,
) -> Result<(), String> {
    state.undo().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn redo(
    state: tauri::State<'_, UndoRedoState>,
) -> Result<(), String> {
    state.redo().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_undo_stack(
    state: tauri::State<'_, UndoRedoState>,
) -> Result<Vec<OperationDescription>, String> {
    Ok(state.get_undo_descriptions().await)
}

#[tauri::command]
pub async fn get_redo_stack(
    state: tauri::State<'_, UndoRedoState>,
) -> Result<Vec<OperationDescription>, String> {
    Ok(state.get_redo_descriptions().await)
}
```

**NOTE: These commands are NOT registered in main.rs currently!**

#### Q2: What operations they should affect

From `daw/src-tauri/src/undo_redo/core.rs`:

```rust
pub enum Operation {
    // Track operations
    AddTrack(TrackData),
    RemoveTrack(i32),
    MoveTrack { track_id: i32, from: usize, to: usize },

    // Note operations
    AddNote(NoteData),
    RemoveNote(NoteId),
    MoveNote { note_id: NoteId, from_tick: u64, to_tick: u64 },
    ResizeNote { note_id: NoteId, from_duration: u64, to_duration: u64 },

    // Velocity operations
    ChangeVelocity { note_id: NoteId, from: u8, to: u8 },
    BatchVelocity { changes: Vec<(NoteId, u8, u8)> },

    // Tempo operations
    ChangeTempo { from: f32, to: f32 },

    // Automation operations
    AddAutomationPoint { lane_id: u32, point: AutomationPoint },
    RemoveAutomationPoint { lane_id: u32, point_id: u32 },
    MoveAutomationPoint { lane_id: u32, point_id: u32, from: (u64, f32), to: (u64, f32) },

    // Piano roll operations
    SelectNotes(Vec<NoteId>),
    DeselectNotes(Vec<NoteId>),
}
```

#### Q3: Current operation history storage

```rust
pub struct UndoRedoState {
    undo_stack: Arc<RwLock<Vec<Operation>>>,
    redo_stack: Arc<RwLock<Vec<Operation>>>,
    max_history: usize,  // Default 100
}

impl UndoRedoState {
    pub async fn push(&self, operation: Operation) {
        let mut undo = self.undo_stack.write().await;
        let mut redo = self.redo_stack.write().await;

        // Clear redo stack on new operation
        redo.clear();

        // Add to undo stack
        undo.push(operation);

        // Trim if exceeds max
        if undo.len() > self.max_history {
            undo.remove(0);
        }
    }
}
```

#### Q4: Any existing command pattern implementation

```rust
impl Operation {
    pub fn inverse(&self) -> Operation {
        match self {
            Operation::AddTrack(data) => Operation::RemoveTrack(data.id),
            Operation::RemoveTrack(id) => {
                // Would need stored track data
                unimplemented!("Need track data for inverse")
            }
            Operation::ChangeVelocity { note_id, from, to } => {
                Operation::ChangeVelocity {
                    note_id: *note_id,
                    from: *to,
                    to: *from,
                }
            }
            // ... etc
        }
    }
}
```

---

## Frontend - Windows (app/src/lib/windows/)

### 21-23. `app/src/lib/windows/DatabaseWindow.svelte`

#### Q1: Current filter UI implementation

```svelte
<div class="filters dark:bg-window-subtle p-3 rounded mb-4">
  <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
    <!-- BPM Range -->
    <div class="filter-group">
      <label class="text-xs dark:text-gray-400">BPM Range</label>
      <div class="flex gap-1">
        <input
          type="number"
          bind:value={minBpm}
          placeholder="Min"
          class="w-16 px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
          min="30"
          max="300"
        />
        <span class="dark:text-gray-500">-</span>
        <input
          type="number"
          bind:value={maxBpm}
          placeholder="Max"
          class="w-16 px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
          min="30"
          max="300"
        />
      </div>
    </div>

    <!-- Key Signature -->
    <div class="filter-group">
      <label class="text-xs dark:text-gray-400">Key</label>
      <select
        bind:value={selectedKey}
        class="w-full px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
      >
        <option value="">All Keys</option>
        {#each availableKeys as key}
          <option value={key}>{key}</option>
        {/each}
      </select>
    </div>

    <!-- Category -->
    <div class="filter-group">
      <label class="text-xs dark:text-gray-400">Category</label>
      <select
        bind:value={selectedCategory}
        class="w-full px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
      >
        <option value="">All Categories</option>
        {#each availableCategories as cat}
          <option value={cat}>{cat}</option>
        {/each}
      </select>
    </div>
  </div>
</div>
```

#### Q2: Current search input

```svelte
<div class="search-bar mb-4">
  <input
    type="text"
    bind:value={searchQuery}
    placeholder="Search files..."
    class="w-full px-3 py-2 dark:bg-input dark:border-window-border dark:text-app-text rounded"
    on:input={debounce(handleSearch, 300)}
  />
</div>
```

#### Q3: Current sort controls

```svelte
<div class="sort-controls flex items-center gap-2 mb-4">
  <label class="text-xs dark:text-gray-400">Sort by:</label>
  <select
    bind:value={sortBy}
    on:change={handleSort}
    class="px-2 py-1 text-sm dark:bg-input dark:border-window-border rounded"
  >
    <option value="filename">Filename</option>
    <option value="bpm">BPM</option>
    <option value="key_signature">Key</option>
    <option value="duration_seconds">Duration</option>
    <option value="created_at">Date Added</option>
    <option value="file_size_bytes">Size</option>
  </select>

  <button
    on:click={toggleSortDirection}
    class="px-2 py-1 dark:bg-secondary dark:text-gray-300 rounded hover:opacity-80"
    title={sortDesc ? 'Descending' : 'Ascending'}
  >
    {sortDesc ? '↓' : '↑'}
  </button>
</div>
```

#### Q4: Current table/listing structure

```svelte
<div class="file-list flex-1 overflow-y-auto">
  {#if $databaseStore.isLoading}
    <div class="loading dark:text-gray-400 text-center py-8">Loading...</div>
  {:else if $databaseStore.searchResults.length === 0}
    <div class="no-results dark:text-gray-400 text-center py-8">
      No files found
    </div>
  {:else}
    {#each $databaseStore.searchResults as file (file.id)}
      <div
        class="file-item flex items-center justify-between p-2 border-b dark:border-window-border hover:dark:bg-window-subtle cursor-pointer"
        class:selected={selectedFileId === file.id}
        on:click={() => selectFile(file)}
        on:dblclick={() => loadFile(file)}
      >
        <div class="file-info flex-1 min-w-0">
          <div class="filename dark:text-app-text truncate" title={file.filename}>
            {file.filename}
          </div>
          <div class="metadata text-xs dark:text-gray-400 flex gap-3">
            <span>{file.bpm ? `${Math.round(file.bpm)} BPM` : '-'}</span>
            <span>{file.key_signature || '-'}</span>
            <span>{formatDuration(file.duration_seconds)}</span>
            <span>{formatSize(file.file_size_bytes)}</span>
          </div>
        </div>

        <div class="actions flex gap-1">
          <button
            on:click|stopPropagation={() => toggleFavorite(file.id)}
            class="p-1 rounded hover:dark:bg-secondary"
            title={file.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
          >
            {file.is_favorite ? '★' : '☆'}
          </button>
          <button
            on:click|stopPropagation={() => deleteFile(file.id)}
            class="p-1 rounded hover:dark:bg-error"
            title="Delete file"
          >
            🗑
          </button>
        </div>
      </div>
    {/each}
  {/if}
</div>
```

#### Q5: Current event handling

```typescript
async function handleSearch() {
  await databaseActions.search({
    search_text: searchQuery || undefined,
    min_bpm: minBpm || undefined,
    max_bpm: maxBpm || undefined,
    key_signature: selectedKey || undefined,
    category: selectedCategory || undefined,
    sort_by: sortBy,
    sort_desc: sortDesc,
  });
}

async function handleSort() {
  await handleSearch();  // Re-search with new sort
}

function toggleSortDirection() {
  sortDesc = !sortDesc;
  handleSort();
}

function selectFile(file: FileDetails) {
  selectedFileId = file.id;
  selectedFile = file;
}

async function loadFile(file: FileDetails) {
  try {
    await invoke('load_sequencer_tracks', { fileId: file.id });
  } catch (error) {
    console.error('Failed to load file:', error);
  }
}

async function toggleFavorite(fileId: number) {
  const file = $databaseStore.searchResults.find(f => f.id === fileId);
  if (file) {
    const command = file.is_favorite ? 'remove_favorite' : 'add_favorite';
    await invoke(command, { fileId });
    await handleSearch();  // Refresh
  }
}

async function deleteFile(fileId: number) {
  if (confirm('Delete this file?')) {
    await invoke('delete_file', { fileId });
    await handleSearch();  // Refresh
  }
}
```

---

### 24. `app/src/lib/windows/MixerWindow.svelte`

#### Q1: Current mixer channel strips

```svelte
{#each channelList as channel (channel.id)}
  <div class="channel-strip dark:bg-window-subtle p-3 rounded border dark:border-window-border w-20 flex flex-col items-center space-y-2">
    <!-- Track Name -->
    <div class="track-name text-center text-xs dark:text-gray-300 truncate w-full" title={channel.label}>
      {channel.label || `Track ${channel.id}`}
    </div>

    <!-- Volume Fader -->
    <div class="volume-fader flex flex-col items-center space-y-1">
      <label class="volume-label text-xs dark:text-gray-400">Vol</label>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={channel.volume}
        on:input={(e) => updateVolume(channel.id, parseFloat(e.currentTarget.value))}
        class="volume-slider dark:bg-input w-4 h-32"
      />
      <span class="volume-display text-xs dark:text-gray-300">{formatVolume(channel.volume)}</span>
    </div>

    <!-- Pan Control -->
    <div class="pan-control flex flex-col items-center space-y-1">
      <label class="pan-label text-xs dark:text-gray-400">Pan</label>
      <input
        type="range"
        min="-1"
        max="1"
        step="0.01"
        value={channel.pan}
        on:input={(e) => updatePan(channel.id, parseFloat(e.currentTarget.value))}
        class="pan-slider dark:bg-input w-16 h-2"
      />
      <span class="pan-display text-xs dark:text-gray-300">{formatPan(channel.pan)}</span>
    </div>

    <!-- VU Meters -->
    <div class="vu-meters flex space-x-1">
      <div class={getMeterColor(getMeterLevel(meterData, channel.id, 'left'))}
           style="height: {getMeterLevel(meterData, channel.id, 'left')}px; width: 2px;"></div>
      <div class={getMeterColor(getMeterLevel(meterData, channel.id, 'right'))}
           style="height: {getMeterLevel(meterData, channel.id, 'right')}px; width: 2px;"></div>
    </div>

    <!-- Mute/Solo Buttons -->
    <div class="controls flex flex-col space-y-1">
      <button
        on:click={() => toggleMute(channel.id)}
        class="mute-btn {channel.muted ? 'dark:bg-error text-white' : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs hover:opacity-80"
      >
        M
      </button>
      <button
        on:click={() => toggleSolo(channel.id)}
        class="solo-btn {channel.soloed ? 'dark:bg-primary text-white' : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs hover:opacity-80"
      >
        S
      </button>
    </div>
  </div>
{/each}
```

#### Q2: Current effect slots/racks

**ANSWER: Effect slots are NOT currently implemented in MixerWindow.svelte**

The mixer only has:
- Volume faders
- Pan controls
- Mute/Solo buttons
- VU meters
- Master section

No effect rack UI exists yet.

#### Q3: Current UI layout

```svelte
<div class="mixer-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
  {#if mixerState}
    <!-- Channel Strips -->
    <div class="channels flex space-x-4 overflow-x-auto pb-4 flex-1">
      {#if channelList.length === 0}
        <div class="no-tracks ...">No tracks loaded</div>
      {/if}
      {#each channelList as channel (channel.id)}
        <!-- Channel strip component -->
      {/each}
    </div>

    <!-- Master Section -->
    <div class="master dark:bg-menu p-3 rounded mt-auto">
      <h3 class="dark:text-gray-200 mb-2">Master</h3>
      <!-- Master volume, meters -->
    </div>
  {:else}
    <div class="no-mixer ...">Loading mixer...</div>
  {/if}
</div>
```

#### Q4: Any existing effect handling

**None exists.** Backend has effect commands but frontend doesn't use them:
- `mixer_add_effect`
- `mixer_remove_effect`
- `mixer_set_effect_enabled`

---

## Frontend - Stores (app/src/lib/stores/)

### 25. `app/src/lib/stores/databaseStore.ts`

#### Q1: Current store structure

```typescript
export interface DatabaseState {
  searchResults: FileDetails[];
  currentPage: number;
  totalPages: number;
  totalResults: number;
  isLoading: boolean;
  error: string | null;
  currentFilters: SearchFilters;
}

const initialState: DatabaseState = {
  searchResults: [],
  currentPage: 1,
  totalPages: 1,
  totalResults: 0,
  isLoading: false,
  error: null,
  currentFilters: {},
};

export const databaseStore = writable<DatabaseState>(initialState);
```

#### Q2: Existing tag-related functions

**ANSWER: NO tag-related functions exist in databaseStore.ts**

The store only handles:
- Search results
- Pagination
- Loading state
- Filters

Tag operations would need to be added or put in a separate `tagStore.ts`.

#### Q3: Current database interaction pattern

```typescript
import { invoke } from '@tauri-apps/api/core';

export const databaseActions = {
  async search(params?: SearchFilters) {
    databaseStore.update(s => ({ ...s, isLoading: true, error: null }));

    try {
      const filters: SearchFilters = {
        ...params,
        limit: PAGE_SIZE,
        offset: 0,
      };

      const response: SearchResponse = await invoke('search_files', { filters });

      databaseStore.update(s => ({
        ...s,
        searchResults: response.files,
        totalResults: response.total,
        totalPages: Math.ceil(response.total / PAGE_SIZE),
        currentPage: 1,
        currentFilters: filters,
        isLoading: false,
      }));
    } catch (error) {
      databaseStore.update(s => ({
        ...s,
        isLoading: false,
        error: error instanceof Error ? error.message : String(error),
      }));
    }
  },
  // ... nextPage, previousPage, reset
};
```

#### Q4: Any existing CRUD operations for tags

**ANSWER: None exist**

Backend has full tag CRUD but frontend doesn't expose it:

Backend commands available:
- `get_file_tags(file_id)` - Get tags for file
- `get_popular_tags(limit)` - Get most used tags
- `search_tags(query)` - Search tags by name
- `get_tag_categories()` - Get tag category list
- `get_tags_by_category(category)` - Get tags in category
- `update_file_tags(file_id, tags)` - Replace file's tags
- `add_tags_to_file(file_id, tags)` - Add tags to file
- `remove_tag_from_file(file_id, tag_id)` - Remove tag from file
- `get_files_by_tags(tag_ids)` - Find files with tags
- `get_tag_stats()` - Get tag usage statistics

---

## Additional Context

### 1. Missing States (ProfilingState, SystemState, DAWState)

#### ProfilingState
**Location:** `daw/src-tauri/src/profiling/mod.rs`
**NOT REGISTERED in main.rs**

```rust
pub struct ProfilingState {
    active: Arc<RwLock<bool>>,
    start_time: Arc<RwLock<Option<Instant>>>,
    samples: Arc<RwLock<Vec<ProfileSample>>>,
    output_path: Arc<RwLock<Option<PathBuf>>>,
}
```

**Initialization:** `ProfilingState::new()`
**Lifetime:** Application lifetime

#### SystemState
**DOES NOT EXIST** - System info gathered on-demand via `get_system_info`

#### DawState vs DAWState
Both exist and are BOTH registered:
- `daw_state: DawState` - Basic transport (from lib.rs)
- `daw_window_state: DAWState` - Full window state with mixer (from window.rs)

---

### 2. 16 Missing Commands

**Not registered in main.rs:**

1. `undo` - Undo last operation
2. `redo` - Redo undone operation
3. `get_undo_stack` - Get undo history
4. `get_redo_stack` - Get redo history
5. `start_profiling` - Begin performance profiling
6. `stop_profiling` - End profiling
7. `get_profile_results` - Get profiling data
8. `export_profile` - Save profile to file
9. `split_file` - Split MIDI file (in pipeline but not registered)
10. `split_file_batch` - Batch split files
11. `get_audio_settings` - Get audio configuration
12. `set_audio_settings` - Update audio configuration
13. `get_midi_settings` - Get MIDI configuration
14. `set_midi_settings` - Update MIDI configuration
15. `mixer_set_effect_parameter` - Set effect parameter value
16. `mixer_reorder_effects` - Reorder effects in chain

---

### 3. Current Database Schema for Tags

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    category VARCHAR(100),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE file_tags (
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (file_id, tag_id)
);

-- Indexes
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_category ON tags(category);
CREATE INDEX idx_file_tags_file_id ON file_tags(file_id);
CREATE INDEX idx_file_tags_tag_id ON file_tags(tag_id);
```

**Tag operations supported:**
- Create tag
- Delete tag
- Rename tag (no merge)
- Add tag to file
- Remove tag from file
- Get files by tags
- Get tags by file

---

### 4. Effect Rack Requirements

**Backend Effect Types (from mixer.rs):**

```rust
pub enum EffectType {
    Eq3Band,
    Eq8Band,
    Compressor,
    Limiter,
    Reverb,
    Delay,
    Chorus,
    Phaser,
    Distortion,
    Filter,
    Gate,
}

pub struct Effect {
    pub id: u32,
    pub effect_type: EffectType,
    pub enabled: bool,
    pub parameters: HashMap<String, f32>,
}
```

**Missing implementations:**
- Frontend effect rack UI
- Effect parameter editing
- Effect presets
- Effect reordering (drag/drop)

---

### 5. Track Data Structure for MIDI Export

**Current (in export.rs):**
```rust
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,
    pub channel: u8,
    pub note: Option<u8>,
    pub velocity: Option<u8>,
    pub controller: Option<u8>,
    pub value: Option<u8>,
    pub program: Option<u8>,
}
```

**From sequencer (in engine.rs):**
```rust
pub struct SequencerTrack {
    pub id: i32,
    pub file_id: Option<i64>,
    pub channel: u8,
    pub events: Vec<ScheduledEvent>,
    pub muted: bool,
    pub soloed: bool,
    pub volume: f32,
    pub pan: f32,
}

pub struct ScheduledEvent {
    pub tick: u64,
    pub event: MidiMessage,
}
```

**Gap:** Export doesn't read from sequencer tracks.

---

### 6. Current Error Handling Patterns

**Backend (Rust):**
```rust
// Command signature
#[tauri::command]
pub async fn some_command(...) -> Result<ReturnType, String>

// Internal error type
#[derive(Error, Debug)]
pub enum SomeError {
    #[error("Error message: {0}")]
    Variant(String),
}

// Conversion to String for Tauri
impl From<SomeError> for String {
    fn from(e: SomeError) -> Self {
        e.to_string()
    }
}

// Or using map_err
internal_fn().await.map_err(|e| e.to_string())
```

**Frontend (TypeScript):**
```typescript
try {
  const result = await invoke('command_name', { params });
  // Success
} catch (error) {
  // Error is string from backend
  console.error('Failed:', error);
  store.update(s => ({
    ...s,
    error: error instanceof Error ? error.message : String(error),
  }));
}
```

**User feedback:**
- Console logging
- Store error state
- No toast/notification system currently

---

*Document answers all architecture questions. Last updated: 2025-12-01*
