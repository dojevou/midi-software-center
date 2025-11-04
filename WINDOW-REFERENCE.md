# Window Reference - Complete Inventory

**Date:** November 3, 2025
**Total Windows:** 32 components
**Categories:** 4 (Core, Production Tools, Hardware, Utilities)

---

## 📑 Table of Contents

1. [Category 1: Core Windows (4)](#category-1-core-windows)
2. [Category 2: Production Tools (6)](#category-2-production-tools)
3. [Category 3: Hardware Integration (3)](#category-3-hardware-integration)
4. [Category 4: Utilities & Settings (19)](#category-4-utilities--settings)
5. [Window Interaction Map](#window-interaction-map)
6. [Keyboard Shortcuts Reference](#keyboard-shortcuts-reference)

---

## Category 1: Core Windows

### 1. DAW Window (Main Sequencer)

**Purpose:** Primary sequencer/piano roll interface with transport controls and track management.

**Location:**
- Backend: `daw/src-tauri/src/windows/state.rs` (573 lines)
- Backend: `daw/src-tauri/src/windows/commands.rs` (574 lines)
- Frontend: `daw/src/lib/windows/DAWWindow.svelte` (753 lines)

**Key Features:**
- Transport bar (play, pause, stop, record buttons)
- Position display (bars.beats.ticks format)
- Tempo control with +/- increment buttons
- Time signature display (numerator/denominator)
- Key signature display
- Loop enable toggle
- Metronome toggle
- Grid snapping controls
- Track list with add/remove buttons
- Track selection (single/multi-select)
- Track controls per track (mute, solo, arm, delete)
- Arrangement view with timeline
- Playhead indicator (synced with playback position)
- Zoom controls (horizontal and vertical)
- Scroll controls

**Backend Commands (26):**

*Transport:*
1. `play_transport()` - Start playback from current position
2. `stop_transport()` - Stop and reset position to start
3. `pause_transport()` - Pause at current position
4. `set_playback_position(bar, beat, tick)` - Seek to specific position
5. `get_playback_state()` - Query current transport state

*Tempo & Musical Properties:*
6. `set_bpm(bpm)` - Set tempo (20-999 BPM)
7. `get_bpm()` - Get current tempo
8. `set_time_signature(num, denom)` - Set time signature
9. `get_time_signature()` - Get time signature
10. `set_key_signature(key)` - Set key (e.g., "C", "Dm", "F#")
11. `get_key_signature()` - Get current key

*Tracks:*
12. `add_window_track(label)` - Add new track
13. `remove_window_track(track_id)` - Remove track
14. `get_all_window_tracks()` - Get all tracks sorted by ID
15. `set_track_visible(track_id, visible)` - Toggle track visibility
16. `set_track_muted(track_id, muted)` - Mute/unmute track
17. `set_track_soloed(track_id, soloed)` - Solo track
18. `get_track_info(track_id)` - Get single track details
19. `update_track_label(track_id, label)` - Rename track

*State:*
20. `get_daw_state()` - Get complete DAW window state
21. `reset_daw_state()` - Reset to defaults

*Loop & Metronome:*
22. `set_loop_enabled(enabled)` - Enable/disable loop
23. `set_loop_range(start, end)` - Set loop start/end (in ticks)
24. `set_metronome_enabled(enabled)` - Enable/disable metronome
25. `set_metronome_volume(volume)` - Set metronome volume (0.0-1.0)
26. `get_transport_info()` - Get full transport information

**Frontend Stores Used:**
- `playbackStore` - Transport state, position, tempo, loop
- `projectStore` - Tracks, selection, mute/solo state
- `uiStore` - Window position, visibility, z-index

**Keyboard Shortcuts:**
- `Space` - Play/Pause toggle
- `Ctrl+Space` - Stop
- `Ctrl+R` - Start/Stop recording
- `Ctrl+T` - Add new track
- `Delete` - Remove selected track
- `M` - Toggle mute on selected track
- `S` - Toggle solo on selected track
- `Ctrl+L` - Toggle loop
- `Ctrl+M` - Toggle metronome
- `Ctrl++` - Zoom in horizontally
- `Ctrl+-` - Zoom out horizontally
- `Up/Down` - Select previous/next track

**Integration Points:**
- Syncs with **Mixer Window** (track mute/solo state)
- Syncs with **Piano Roll Window** (selected track, playhead)
- Syncs with **Velocity Editor** (playhead position)
- Syncs with **All Windows** (transport state broadcast)

**Component Size:**
- Default: 1200px × 700px
- Min: 800px × 500px
- Resizable: Yes
- Draggable: Yes (via WindowBase)

---

### 2. Mixer Window

**Purpose:** Channel strip mixer with volume, pan, mute, solo, and metering.

**Location:**
- Backend: Integrated with `daw/src-tauri/src/windows/state.rs` (426 lines)
- Frontend: `daw/src/lib/windows/MixerWindow.svelte` (602 lines)

**Key Features:**
- Horizontal channel strip layout
- One channel per DAW track
- Master channel (distinct styling, wider)
- Per-channel controls:
  - Vertical fader (volume 0.0-1.0)
  - Pan knob (-1.0 left to 1.0 right)
  - Mute button
  - Solo button
  - Channel label
- Level metering:
  - Color gradient (green → yellow → red)
  - Clip detection indicator
  - Auto-updating (50ms refresh rate)
- Volume display in dB (-∞ to +12 dB)
- Pan display (L/C/R or percentage)
- Fader dragging with mouse (vertical motion)
- Visual feedback on mute/solo (button highlights)

**Backend Commands (4):**
1. `get_mixer_state()` - Get complete mixer state
2. `set_channel_volume(channel_id, volume)` - Set channel volume (0.0-1.0)
3. `set_channel_pan(channel_id, pan)` - Set channel pan (-1.0 to 1.0)
4. `set_channel_mute(channel_id, muted)` - Mute/unmute channel
5. `set_channel_solo(channel_id, soloed)` - Solo channel (additional command)

**Frontend Stores Used:**
- `projectStore` - Track list, mute/solo state
- `playbackStore` - Metering data (future integration)
- `uiStore` - Window position, visibility

**Keyboard Shortcuts:**
- `Ctrl+M` - Toggle mute on focused channel
- `Ctrl+S` - Toggle solo on focused channel
- `Ctrl+0` - Reset volume to 0 dB
- `Ctrl+C` - Center pan
- `Left/Right` - Select previous/next channel

**Integration Points:**
- **Automatic sync with DAW Window** (bidirectional):
  - Track add → Channel add
  - Track remove → Channel remove
  - Track mute/solo → Channel mute/solo
  - Track label change → Channel label change
- Master channel always present (ID: -1)

**Component Size:**
- Default: 1000px × 600px
- Min: 600px × 400px
- Resizable: Yes (horizontal only for more channels)

**Data Flow:**
```
DAW Window (add track)
    ↓
projectStore.addTrack()
    ↓
Backend: add_window_track()
    ↓
DAWWindowState.add_track()
    ↓
MixerWindowState.add_channel_from_track()
    ↓
Mixer Window (new channel appears)
```

---

### 3. Database Window (File Browser)

**Purpose:** Search, browse, and manage MIDI files in the database with advanced filtering.

**Location:**
- Backend: `pipeline/src-tauri/src/database/window_state.rs` (480 lines)
- Frontend: `daw/src/lib/windows/DatabaseWindow.svelte` (828 lines)

**Key Features:**
- Search box (full-text search, debounced 300ms)
- Filter panel:
  - BPM range slider (0-300)
  - Key dropdown (All, C, C#, D, ..., B, Am, Bm, etc.)
  - Category dropdown (All, Drums, Bass, Melody, Chords, Pads, FX)
  - Tag chips (multi-select, e.g., Techno, House, Trance)
  - Favorites toggle
- Sort controls:
  - Sort by: FileName, DateAdded, Bpm, Duration, FileSize, LastAccessed
  - Sort order: Ascending, Descending
- View modes:
  - List view (compact rows)
  - Grid view (thumbnail cards)
  - Details view (full metadata)
- Results list with pagination:
  - 50 items per page
  - Next/Previous buttons
  - Page number display
  - Total results count
- File selection (single/multi-select)
- File preview pane:
  - File name, path
  - BPM, key, duration
  - Date added, file size
  - Tags, category
  - Favorite star button
- Import buttons:
  - Import single file
  - Import directory (recursive)
- Clear filters button
- Status bar: "Showing X-Y of Z results"

**Backend Commands (8):**
1. `search_files(filters)` - Search with filters
2. `get_file_details(file_id)` - Get full metadata
3. `toggle_favorite(file_id)` - Add/remove from favorites
4. `add_tag(file_id, tag)` - Add tag to file
5. `remove_tag(file_id, tag)` - Remove tag from file
6. `set_category(file_id, category)` - Set file category
7. `import_file(path)` - Import single file
8. `import_directory(path)` - Import directory recursively

**Frontend Stores Used:**
- `databaseStore` - Search state, filters, results, pagination
- `uiStore` - Window position, view mode

**Keyboard Shortcuts:**
- `Ctrl+F` - Focus search box
- `Ctrl+I` - Open import file dialog
- `Ctrl+Shift+I` - Open import directory dialog
- `Ctrl+G` - Toggle grid/list view
- `Ctrl+D` - Toggle details view
- `Delete` - Remove selected file from library
- `F` - Toggle favorite on selected file
- `Page Up/Down` - Navigate pages
- `Enter` - Load selected file into DAW

**Integration Points:**
- **Pipeline Window** - Import operations trigger progress updates
- **DAW Window** - Double-click file loads into sequencer
- **Loop Browser** - Shared search backend

**Component Size:**
- Default: 1000px × 700px
- Min: 700px × 500px

**Search Algorithm:**
```rust
// Backend: window_state.rs
impl DatabaseWindowState {
    pub fn matches_filter(&self, file: &SearchResult) -> bool {
        // Text query
        if query.len() > 0 && !file.name.contains(&query) {
            return false;
        }

        // BPM range
        if file.bpm < filters.bpm_min || file.bpm > filters.bpm_max {
            return false;
        }

        // Key signature
        if filters.key.is_some() && file.key != filters.key {
            return false;
        }

        // Category
        if filters.category.is_some() && file.category != filters.category {
            return false;
        }

        // Tags (ANY match)
        if filters.tags.len() > 0 {
            if !file.tags.iter().any(|t| filters.tags.contains(t)) {
                return false;
            }
        }

        // Favorites
        if filters.favorites_only && !file.is_favorite {
            return false;
        }

        true
    }
}
```

---

### 4. Pipeline Window (Batch Processing)

**Purpose:** Monitor and control batch import, analysis, and archive extraction operations.

**Location:**
- Backend: `pipeline/src-tauri/src/windows/pipeline_state.rs` (472 lines)
- Frontend: `daw/src/lib/windows/PipelineWindow.svelte` (1,834 lines)

**Key Features:**
- Operation selector dropdown:
  - File Import
  - Directory Import
  - Archive Extraction
  - Batch Analysis
  - Auto-Tagging
  - BPM Detection
  - Key Detection
  - Duplicate Scan
- Progress display:
  - Progress bar (0-100%)
  - Current file being processed
  - Files processed / Total files
  - Success count / Error count
  - Success rate percentage
  - Elapsed time
  - ETA (estimated time remaining)
- Control buttons:
  - Start operation
  - Pause operation
  - Resume operation
  - Stop operation
  - Clear log
- Log viewer:
  - Scrollable message list
  - Color-coded by level (Debug, Info, Warning, Error)
  - Timestamps
  - Auto-scroll to bottom
  - Max 100 messages (auto-trim oldest)
- Statistics panel:
  - Total files imported today
  - Total errors today
  - Average processing speed (files/sec)
  - Cache hit rate

**Backend Commands (10):**
1. `start_operation(operation_type, params)` - Start batch operation
2. `pause_operation()` - Pause current operation
3. `resume_operation()` - Resume paused operation
4. `stop_operation()` - Stop and cancel operation
5. `get_pipeline_state()` - Get current state
6. `get_operation_stats()` - Get statistics
7. `clear_log()` - Clear log messages
8. `set_operation_params(params)` - Configure operation
9. `get_recent_operations()` - Get operation history
10. `retry_failed_files()` - Retry files that errored

**Frontend Stores Used:**
- `databaseStore` - Results from import operations
- `uiStore` - Window position, log scroll position

**Keyboard Shortcuts:**
- `Ctrl+Enter` - Start operation
- `Ctrl+P` - Pause operation
- `Ctrl+S` - Stop operation
- `Ctrl+L` - Clear log
- `Ctrl+E` - Toggle error filter (show only errors)
- `Ctrl+W` - Toggle warning filter

**Integration Points:**
- **Database Window** - Import results appear in search
- **DAW Window** - Import completed event notification
- Real-time progress events via Tauri events

**Component Size:**
- Default: 900px × 600px
- Min: 700px × 400px

**State Machine:**
```
Idle ──(start)──> Processing
Processing ──(pause)──> Paused
Processing ──(stop)──> Idle
Processing ──(complete)──> Complete
Processing ──(error)──> Error
Paused ──(resume)──> Processing
Paused ──(stop)──> Idle
Complete ──(start new)──> Idle
Error ──(retry)──> Processing
Error ──(stop)──> Idle
```

**ETA Calculation:**
```rust
// Backend: pipeline_state.rs
impl ProcessingStats {
    pub fn calculate_eta(&self) -> Option<Duration> {
        if self.total_files == 0 || self.processed_files == 0 {
            return None;
        }

        let remaining_files = self.total_files - self.processed_files;
        let elapsed = self.start_time.elapsed();
        let avg_time_per_file = elapsed.as_secs_f64() / self.processed_files as f64;
        let eta_seconds = avg_time_per_file * remaining_files as f64;

        Some(Duration::from_secs_f64(eta_seconds))
    }
}
```

---

## Category 2: Production Tools

### 5. Piano Roll Window

**Purpose:** Visual MIDI note editing with piano keyboard, grid, and velocity control.

**Location:** `daw/src/lib/windows/PianoRollWindow.svelte` (1,030 lines)

**Key Features:**
- Timeline header with ruler (bars, beats, ticks)
- Piano keyboard on left side (C-1 to C8, 128 notes)
- Note grid with background grid lines
- Grid snapping options (1/1, 1/2, 1/4, 1/8, 1/16, 1/32, Off)
- Selected notes highlighted (blue border)
- Velocity strips below each note (draggable height)
- Playhead indicator (vertical line, synced with `$playbackStore.position`)
- Grid toggle (show/hide grid lines)
- Zoom controls:
  - Horizontal zoom slider (zoom timeline)
  - Vertical zoom slider (zoom piano keyboard)
- Note actions:
  - Quantize (snap to grid)
  - Transpose (+Oct, -Oct buttons)
  - Select All
  - Invert Selection
- Keyboard shortcuts:
  - `Delete` - Remove selected notes
  - `Arrow Keys` - Move selected notes
  - `V` - Toggle velocity strips visibility
  - `Ctrl+A` - Select all notes
  - `Ctrl+I` - Invert selection
  - `Q` - Quantize selected notes
- Mouse interactions:
  - Click note to select
  - Drag note to move
  - Resize note handles (start/end)
  - Drag velocity strip to adjust velocity
  - Click empty space to add note
  - Drag selection box to multi-select

**Backend Commands (6):**
1. `get_notes(track_id)` - Get all notes for track
2. `add_note(track_id, note)` - Add new note
3. `remove_note(track_id, note_id)` - Remove note
4. `update_note(track_id, note_id, note)` - Update note properties
5. `quantize_notes(track_id, note_ids, grid)` - Quantize to grid
6. `transpose_notes(track_id, note_ids, semitones)` - Transpose notes

**Stores Used:**
- `projectStore` - Selected track, notes
- `playbackStore` - Playhead position
- `uiStore` - Zoom level, grid snap setting

**Keyboard Shortcuts:**
- `Delete` - Remove selected notes
- `Ctrl+A` - Select all
- `Ctrl+I` - Invert selection
- `Q` - Quantize
- `+` - Transpose up octave
- `-` - Transpose down octave
- `V` - Toggle velocity strips
- `G` - Toggle grid
- `1-6` - Set grid snap (1/1, 1/2, 1/4, 1/8, 1/16, 1/32)

**Integration:** Syncs with DAW Window (selected track, playhead)

**Size:** 1200px × 700px (default)

---

### 6. Velocity Editor Window

**Purpose:** Fine-tune note velocities with faders, curves, and humanization.

**Location:** `daw/src/lib/windows/VelocityEditorWindow.svelte` (861 lines)

**Key Features:**
- Horizontal velocity lanes (one per note)
- Vertical fader per note (0-127 range)
- dB display below each fader (0-127 → dB conversion)
- Humanize button with random slider (0-50% randomness)
- Statistics panel:
  - Min velocity (all notes)
  - Max velocity (all notes)
  - Avg velocity (all notes)
  - Min velocity (selected notes)
  - Max velocity (selected notes)
  - Avg velocity (selected notes)
- Velocity curve dropdown:
  - Linear (no curve)
  - Exponential (gradual increase)
  - Logarithmic (gradual decrease)
- 50-level undo/redo stack
- Action buttons:
  - Normalize (scale to 0-127 range)
  - +10% (increase by 10%)
  - -10% (decrease by 10%)
  - Apply Curve
  - Apply Humanize
  - Undo
  - Redo
- Visual feedback: Selected notes highlighted

**Backend Commands (5):**
1. `set_note_velocity(track_id, note_id, velocity)` - Set single velocity
2. `set_velocities(track_id, velocities)` - Set multiple velocities
3. `normalize_velocities(track_id, note_ids)` - Normalize range
4. `apply_curve(track_id, note_ids, curve_type)` - Apply velocity curve
5. `humanize_velocities(track_id, note_ids, amount)` - Randomize velocities

**Stores:** `projectStore` (notes, selected notes)

**Shortcuts:**
- `Ctrl+Z` - Undo
- `Ctrl+Y` - Redo
- `Ctrl+N` - Normalize
- `H` - Apply humanize
- `C` - Apply curve

**Integration:** Works on notes from selected track in Piano Roll

**Size:** 900px × 600px

---

### 7. Controller Editor Window

**Purpose:** Edit MIDI CC (Continuous Controller) automation curves.

**Location:** `daw/src/lib/windows/ControllerEditorWindow.svelte` (923 lines)

**Key Features:**
- CC selector dropdown (CC 0-127 with standard names):
  - CC 1: Modulation
  - CC 7: Volume
  - CC 10: Pan
  - CC 11: Expression
  - CC 64: Sustain Pedal
  - CC 71: Resonance
  - CC 74: Cutoff
  - ...and 120+ more
- Timeline with curve display (SVG path)
- CC values as draggable points on curve (0-127 range)
- Drag points to adjust value
- Interpolation between points (linear by default)
- Add point button (adds at playhead position)
- Remove point button (removes selected point)
- Curve smoothing options:
  - None (step changes)
  - Linear (straight lines)
  - Bezier (smooth curves)
- Statistics display:
  - Min CC value
  - Max CC value
  - Current CC value (at playhead)
- Real-time preview (playhead shows current value)
- Reset CC button (with confirmation dialog)
- Keyboard shortcuts:
  - `Delete` - Remove selected point
  - `Ctrl+A` - Select all points
  - `A` - Add point at playhead

**Backend Commands (6):**
1. `get_cc_data(track_id, cc_number)` - Get all CC points
2. `add_cc_point(track_id, cc_number, position, value)` - Add point
3. `remove_cc_point(track_id, cc_number, point_id)` - Remove point
4. `update_cc_point(track_id, cc_number, point_id, value)` - Update value
5. `clear_cc_data(track_id, cc_number)` - Clear all points
6. `apply_smoothing(track_id, cc_number, smoothing_type)` - Apply smoothing

**Stores:** `projectStore` (CC data), `playbackStore` (playhead)

**Shortcuts:**
- `Delete` - Remove point
- `A` - Add point
- `Ctrl+A` - Select all
- `Ctrl+R` - Reset CC

**Size:** 1000px × 600px

---

### 8. Tempo Editor Window

**Purpose:** Create and edit tempo map with instant/ramp tempo changes.

**Location:** `daw/src/lib/windows/TempoEditorWindow.svelte` (1,059 lines)

**Key Features:**
- Tempo map visualization (BPM vs timeline, SVG chart)
- Current tempo display (large number at playhead position)
- Tempo marker controls:
  - BPM input (20-300 range, validation)
  - Add marker button
  - Remove marker button
- Marker types:
  - Instant (immediate tempo change)
  - Ramp (gradual change over N bars, with duration input)
- Tempo change list (table display):
  - Columns: Position (bar.beat), BPM, Type (Instant/Ramp), Duration
  - Click row to select marker
  - Double-click row to jump playhead
- Drag tempo markers on timeline
- Grid controls:
  - Snap to bar
  - Snap to beat
  - Snap to 1/8
  - Snap to 1/16
  - No snap
- Visual tempo curve path (SVG, interpolated for ramps)
- Validation: BPM 20-300 range enforced
- Metronome integration (tempo changes affect click tempo)

**Backend Commands (5):**
1. `get_tempo_map()` - Get all tempo markers
2. `add_tempo_marker(position, bpm, type, duration)` - Add marker
3. `remove_tempo_marker(marker_id)` - Remove marker
4. `update_tempo_marker(marker_id, bpm, type, duration)` - Update marker
5. `clear_tempo_map()` - Remove all markers (reset to default BPM)

**Stores:** `playbackStore` (tempo, tempo map, playhead)

**Shortcuts:**
- `A` - Add marker at playhead
- `Delete` - Remove selected marker
- `Ctrl+R` - Reset tempo map

**Size:** 1000px × 650px

---

### 9. Loop Browser Window

**Purpose:** Search, browse, and preview MIDI loops with drag-to-DAW functionality.

**Location:** `daw/src/lib/windows/LoopBrowserWindow.svelte` (1,050 lines)

**Key Features:**
- Search box (full-text, real-time, debounced 300ms)
- Filter section:
  - BPM range slider (60-200 default, adjustable)
  - Duration range slider (0-60s default)
  - Category dropdown: All, Drums, Bass, Melody, Chords, Pads, FX, Vocals
  - Tag chips (multi-select): Techno, House, Trance, Trap, Dubstep, etc.
  - Favorites toggle
- Results list with pagination:
  - 50 loops per page
  - Next/Previous/Goto page buttons
  - Page number display (e.g., "Page 3 of 10")
  - Total results count
- Loop preview player per item:
  - Play/Stop button
  - Loop button (repeat preview)
  - Waveform display (placeholder SVG)
  - Progress bar
- Metadata display per loop:
  - Loop name
  - BPM
  - Key signature
  - Duration (seconds)
  - Sample rate
  - Tempo (detected)
  - Category
  - Tags (comma-separated)
- Drag-to-DAW functionality:
  - HTML5 drag API implemented
  - Drag loop item onto DAW track
  - Automatic tempo matching option
  - Automatic key transposition option
- Favorite star button per loop
- Zoom level selector (0.5x, 1x, 1.5x, 2x waveform zoom)
- Clear filters button
- Status bar: "Showing 1-50 of 1,423 results"
- Demo data: 150 loops loaded with randomized metadata

**Backend Commands (8):**
1. `search_loops(filters)` - Search with filters
2. `get_loop_details(loop_id)` - Get full metadata
3. `preview_loop(loop_id)` - Load loop for preview
4. `stop_preview()` - Stop preview playback
5. `toggle_loop_favorite(loop_id)` - Favorite/unfavorite
6. `add_loop_to_track(loop_id, track_id)` - Insert loop into track
7. `get_loop_waveform(loop_id)` - Get waveform data for visualization
8. `import_loop(path)` - Import new loop file

**Stores:** `databaseStore` (loop search), `projectStore` (drag-to-track)

**Shortcuts:**
- `Ctrl+F` - Focus search
- `Space` - Play/Stop preview
- `F` - Toggle favorite
- `Page Up/Down` - Navigate pages
- `Enter` - Add to selected track

**Size:** 1100px × 750px

**Drag-to-DAW Flow:**
```
User drags loop item
    ↓
ondragstart event (set dataTransfer)
    ↓
DAW Window ondrop event
    ↓
Get drop target track
    ↓
Call add_loop_to_track(loop_id, track_id)
    ↓
Backend loads loop MIDI data
    ↓
Insert notes into track at playhead position
    ↓
Optional: Transpose to match project key
    ↓
Optional: Time-stretch to match project tempo
    ↓
DAW Window updates (new notes appear in Piano Roll)
```

---

### 10. Project Browser Window

**Purpose:** Manage projects with recent/all views, grid/list modes, and context menu actions.

**Location:** `daw/src/lib/windows/ProjectBrowserWindow.svelte` (1,012 lines)

**Key Features:**
- Two-column layout:
  - **Left column: Recent Projects** (LRU list, last 10 projects)
    - Project name
    - Relative time (e.g., "2 hours ago", "3 days ago")
    - Click to open
  - **Right column: All Projects** (grid or list view)
    - Full project library
    - Grid view: Thumbnail cards with project icons
    - List view: Tabular format with sortable columns
- Search box (filters project list, debounced 300ms)
- Grid view features:
  - Project thumbnail (placeholder icon)
  - Project name
  - Favorite star
  - Metadata card (hover to see details):
    - Date modified
    - Size on disk
    - Track count
    - Duration
- List view features:
  - Columns: Name, Date Modified, Size, Tracks, Duration
  - Sortable columns (click header to sort)
  - Select row to highlight
- New Project button (prompts for name, creates project)
- Double-click to open project
- Right-click context menu (6 actions):
  - Open
  - Rename (inline editing or prompt)
  - Delete (with confirmation dialog: "Are you sure?")
  - Show in Folder (opens file manager)
  - Duplicate (creates copy with "(Copy)" suffix)
  - Properties (opens modal with full metadata)
- Sort dropdown: Name, Date Modified, Size
- Favorites toggle (star button, filters to show only favorites)
- Status bar: "25 projects, 3.2 GB total"
- Demo data: 25 projects loaded with randomized metadata

**Backend Commands (10):**
1. `get_recent_projects()` - Get last 10 opened projects
2. `get_all_projects()` - Get full project list
3. `create_project(name)` - Create new project
4. `open_project(project_id)` - Load project into DAW
5. `rename_project(project_id, new_name)` - Rename project
6. `delete_project(project_id)` - Delete project (with files)
7. `duplicate_project(project_id)` - Duplicate project
8. `toggle_project_favorite(project_id)` - Favorite/unfavorite
9. `get_project_properties(project_id)` - Get full metadata
10. `show_in_folder(project_id)` - Open file manager to project folder

**Stores:** `projectStore` (current project), custom project list store

**Shortcuts:**
- `Ctrl+N` - New project
- `Ctrl+O` - Open selected project
- `Delete` - Delete selected project
- `F2` - Rename selected project
- `Ctrl+D` - Duplicate selected project
- `Ctrl+G` - Toggle grid/list view
- `F` - Toggle favorite

**Size:** 1000px × 700px

**Properties Modal Contents:**
```
Project Properties
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Name:          My Awesome Track
Location:      ~/Documents/MIDI Projects/
Created:       October 28, 2025 2:30 PM
Modified:      November 3, 2025 9:15 AM
Size:          12.4 MB
Duration:      3m 42s
Tracks:        8
MIDI Events:   4,521
Tempo:         128 BPM
Key:           Am
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[Close Button]
```

---

## Category 3: Hardware Integration

### 11. Device Manager Window

**Purpose:** Detect, connect, and configure MIDI hardware devices.

**Location:** `daw/src-tauri/src/hardware/device_manager.rs` (584 lines, 19 tests, 5 commands)

**Key Features:**
- Real-time MIDI device detection (scan every 2 seconds)
- Device list display:
  - Available devices (not connected)
  - Connected devices (active)
- Device information per device:
  - Device name
  - Manufacturer
  - Number of input ports
  - Number of output ports
  - Latency (ms)
  - Connection status (Connected/Available)
- Connect button per available device
- Disconnect button per connected device
- Device configuration (when connected):
  - CC mapping (map hardware CC to DAW parameter)
  - Note mapping (map hardware note to different note)
  - Channel mapping (map hardware channel to DAW track)
  - Velocity curve (map input velocity to output velocity)
- Mapping table display:
  - Input CC → Output Parameter
  - Input Note → Output Note
  - Input Channel → Track
- Save mapping profile button
- Load mapping profile button
- Reset mappings button

**Backend Commands (5):**
1. `list_devices()` → `Vec<MidiDevice>`
   - Returns all available MIDI devices
   - Includes connection status

2. `connect_device(device_id: String)` → `Result<(), String>`
   - Opens MIDI ports for device
   - Starts event listener thread
   - Returns error if already connected

3. `disconnect_device(device_id: String)` → `Result<(), String>`
   - Closes MIDI ports
   - Stops event listener
   - Cleans up resources

4. `get_device_info(device_id: String)` → `Result<DeviceInfo, String>`
   - Returns detailed device information
   - Includes port names, latency

5. `set_device_mapping(device_id: String, mapping: DeviceMapping)` → `Result<(), String>`
   - Configures CC/note/channel mappings
   - Validates mapping values
   - Persists to config file

**Data Structures:**
```rust
#[derive(Serialize, Deserialize)]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub input_ports: Vec<String>,
    pub output_ports: Vec<String>,
    pub is_connected: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceMapping {
    pub cc_mappings: HashMap<u8, String>,     // CC number → parameter name
    pub note_mappings: HashMap<u8, u8>,       // Input note → output note
    pub channel_mappings: HashMap<u8, i32>,   // MIDI channel → track ID
    pub velocity_curve: VelocityCurve,
}

#[derive(Serialize, Deserialize)]
pub enum VelocityCurve {
    Linear,
    Exponential,
    Logarithmic,
    Custom(Vec<(u8, u8)>),  // Input velocity → output velocity
}
```

**Frontend (To be created):** `DeviceManagerWindow.svelte` (~700 lines estimated)

**Stores:** Custom `hardwareStore` for device state

**Size:** 900px × 650px

---

### 12. MIDI Monitor Window

**Purpose:** Record and display incoming MIDI events in real-time for debugging.

**Location:** `daw/src-tauri/src/hardware/midi_monitor.rs` (584 lines, 24 tests, 4 commands)

**Key Features:**
- Real-time MIDI event recording
- Event list display (scrollable, auto-scroll to latest):
  - Timestamp (relative, e.g., "+0.023s")
  - Event type (Note On, Note Off, CC, Program Change, etc.)
  - Channel (1-16)
  - Data 1 (note number or CC number)
  - Data 2 (velocity or CC value)
  - Human-readable description
- Event type filter checkboxes:
  - Note On
  - Note Off
  - Control Change
  - Program Change
  - Pitch Bend
  - Aftertouch
  - System Exclusive
- Channel filter (1-16, "All Channels")
- Start/Stop monitoring button
- Clear events button
- Export events to CSV button
- Event count display
- Max events setting (default 1,000, auto-trim oldest)
- Event statistics:
  - Total events
  - Events per second (rolling average)
  - Most common event type
  - Most active channel

**Backend Commands (4):**
1. `start_monitoring()` → `Result<(), String>`
   - Starts recording MIDI events
   - Clears existing events
   - Returns error if no device connected

2. `stop_monitoring()` → `Result<(), String>`
   - Stops recording
   - Keeps existing events

3. `clear_events()` → `Result<(), String>`
   - Clears event history

4. `get_events(limit: Option<usize>)` → `Vec<MidiEvent>`
   - Returns recorded events
   - Optional limit (default: all events)

**Data Structures:**
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct MidiEvent {
    pub timestamp: f64,           // Seconds since monitoring started
    pub event_type: EventType,
    pub channel: u8,              // 1-16
    pub data1: u8,                // Note number or CC number
    pub data2: u8,                // Velocity or CC value
    pub description: String,      // Human-readable (e.g., "C4 Note On, vel 64")
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum EventType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Aftertouch,
    SystemExclusive,
}

impl MidiEvent {
    pub fn to_description(&self) -> String {
        match self.event_type {
            EventType::NoteOn => {
                let note_name = midi_note_to_name(self.data1);
                format!("{} Note On, vel {}", note_name, self.data2)
            }
            EventType::NoteOff => {
                let note_name = midi_note_to_name(self.data1);
                format!("{} Note Off", note_name)
            }
            EventType::ControlChange => {
                let cc_name = cc_number_to_name(self.data1);
                format!("CC {} ({}), value {}", self.data1, cc_name, self.data2)
            }
            // ... other types
        }
    }
}
```

**Frontend (To be created):** `MIDIMonitorWindow.svelte` (~650 lines estimated)

**Size:** 800px × 600px

---

### 13. MIDI Router Window

**Purpose:** Create flexible MIDI routing rules with filtering and transformation.

**Location:** `daw/src-tauri/src/hardware/midi_router.rs` (826 lines, 30 tests, 6 commands)

**Key Features:**
- Route list display:
  - Route ID
  - From (source device/track)
  - To (destination device/track/virtual)
  - Enabled status (toggle)
  - Filter summary (e.g., "Ch 1-4, Notes 60-72")
- Create route button (opens dialog):
  - From dropdown (all connected devices + all tracks)
  - To dropdown (all connected devices + all tracks + virtual ports)
  - Filter configuration:
    - Channel filter (checkboxes 1-16, "All")
    - Message type filter (Note On/Off, CC, PC, etc.)
    - Note range filter (min/max note number 0-127)
    - Velocity range filter (min/max velocity 0-127)
  - Transpose (semitones, -24 to +24, clamped to 0-127)
  - Enable route immediately checkbox
- Edit route button (opens same dialog with current values)
- Delete route button (with confirmation)
- Enable/Disable toggle per route (can disable without deleting)
- Test route button (sends test MIDI event to verify routing)
- Active routes indicator (shows number of routes currently passing MIDI)
- Route statistics:
  - Events routed (total count)
  - Events filtered (didn't match filter)
  - Events transformed (transposed/modified)

**Backend Commands (6):**
1. `create_route(from: String, to: String, filter: RouteFilter)` → `Result<i32, String>`
   - Creates new routing rule
   - Returns route ID
   - Validates from/to exist

2. `delete_route(route_id: i32)` → `Result<(), String>`
   - Removes route
   - Stops routing immediately

3. `enable_route(route_id: i32)` → `Result<(), String>`
   - Enables disabled route

4. `disable_route(route_id: i32)` → `Result<(), String>`
   - Disables route (keeps rule for later)

5. `get_all_routes()` → `Vec<Route>`
   - Returns all routing rules

6. `test_route(route_id: i32)` → `Result<String, String>`
   - Sends test MIDI event through route
   - Returns confirmation message

**Data Structures:**
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Route {
    pub id: i32,
    pub from: RouteEndpoint,
    pub to: RouteEndpoint,
    pub filter: RouteFilter,
    pub transpose: i8,              // -24 to +24 semitones
    pub enabled: bool,
    pub events_routed: u64,
    pub events_filtered: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RouteEndpoint {
    Device(String),                 // Device ID
    Track(i32),                     // Track ID
    Virtual(String),                // Virtual port name
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RouteFilter {
    pub channels: Option<Vec<u8>>,  // None = all channels
    pub message_types: Option<Vec<EventType>>,
    pub note_range: Option<(u8, u8)>,  // (min, max)
    pub velocity_range: Option<(u8, u8)>,
}

impl Route {
    pub fn matches_event(&self, event: &MidiEvent) -> bool {
        // Channel filter
        if let Some(ref channels) = self.filter.channels {
            if !channels.contains(&event.channel) {
                return false;
            }
        }

        // Message type filter
        if let Some(ref types) = self.filter.message_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        // Note range filter
        if let Some((min, max)) = self.filter.note_range {
            if event.event_type == EventType::NoteOn || event.event_type == EventType::NoteOff {
                if event.data1 < min || event.data1 > max {
                    return false;
                }
            }
        }

        // Velocity range filter
        if let Some((min, max)) = self.filter.velocity_range {
            if event.event_type == EventType::NoteOn {
                if event.data2 < min || event.data2 > max {
                    return false;
                }
            }
        }

        true
    }

    pub fn transform_event(&self, mut event: MidiEvent) -> MidiEvent {
        // Apply transpose
        if event.event_type == EventType::NoteOn || event.event_type == EventType::NoteOff {
            let new_note = (event.data1 as i16 + self.transpose as i16).clamp(0, 127) as u8;
            event.data1 = new_note;
            event.description = event.to_description();
        }

        event
    }
}
```

**Frontend (To be created):** `MIDIRouterWindow.svelte` (~800 lines estimated)

**Size:** 950px × 700px

**Example Route:**
```
Route #1: Enabled
From: Akai MPK Mini
To: Track 3 (Bass)
Filter:
  - Channels: 1, 2, 3
  - Message Types: Note On, Note Off
  - Note Range: 36-60 (C2-C4)
  - Velocity Range: 50-127
Transform:
  - Transpose: -12 (down one octave)
```

---

## Category 4: Utilities & Settings

### 14. Command Palette Window

**Purpose:** Quick fuzzy search and execution of any DAW command.

**Location:** `daw/src-tauri/src/command_palette.rs` (420 lines, 30 tests)

**Key Features:**
- Modal overlay (Cmd+K or Ctrl+K to open)
- Search input (autofocus when opened)
- Fuzzy search through 200+ commands
- Command categories (6):
  - Transport
  - Track
  - Edit
  - View
  - Settings
  - Help
- Search results list (max 10 results):
  - Command name
  - Category badge
  - Keyboard shortcut (if available)
  - Description (tooltip on hover)
- Recently used commands section (top 10)
- Arrow keys to navigate results
- Enter to execute selected command
- Esc to close palette

**42 Predefined Commands:**

*Transport:*
- Play
- Pause
- Stop
- Record
- Toggle Loop
- Toggle Metronome

*Track:*
- New Track
- Delete Track
- Mute Track
- Solo Track
- Arm Track
- Duplicate Track

*Edit:*
- Undo
- Redo
- Cut
- Copy
- Paste
- Delete
- Select All
- Quantize

*View:*
- Show/Hide Mixer
- Show/Hide Piano Roll
- Show/Hide Database
- Show/Hide Pipeline
- Zoom In
- Zoom Out
- Fit to Window

*Settings:*
- Open General Settings
- Open Audio Settings
- Open MIDI Settings
- Open Keyboard Settings
- Open Display Settings

*Help:*
- Keyboard Shortcuts
- Documentation
- Report Bug
- Check for Updates
- About

**Backend Commands (2):**
1. `search_commands(query: String)` → `Vec<SearchResult>`
   - Returns top 10 matching commands
   - Sorted by relevance score

2. `execute_command(command_id: String)` → `Result<(), String>`
   - Executes the specified command
   - Returns error if command fails

**Search Algorithm:**
```rust
pub fn search(&self, query: &str) -> Vec<SearchResult> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for cmd in &self.commands {
        let mut score = 0;

        // Exact match (highest score)
        if cmd.name.to_lowercase() == query {
            score = 100;
        }
        // Starts with query
        else if cmd.name.to_lowercase().starts_with(&query) {
            score = 80;
        }
        // Contains query (substring match)
        else if cmd.name.to_lowercase().contains(&query) {
            score = 60;
        }
        // Abbreviation match (e.g., "nt" matches "New Track")
        else if matches_abbreviation(&cmd.name, &query) {
            score = 40;
        }

        // Boost score if recently used
        if self.recently_used.contains(&cmd.id) {
            score += 20;
        }

        if score > 0 {
            results.push(SearchResult {
                command: cmd.clone(),
                score,
            });
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.score.cmp(&a.score));

    // Return top 10
    results.truncate(10);
    results
}

fn matches_abbreviation(name: &str, query: &str) -> bool {
    let words: Vec<&str> = name.split_whitespace().collect();
    let abbrev: String = words.iter()
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_lowercase();

    abbrev.starts_with(query)
}
```

**Frontend (To be created):** `CommandPaletteWindow.svelte` (~400 lines estimated)

**Size:** 600px × 400px (modal)

---

### 15-31. Settings Windows (17 components)

All settings windows follow the same structure:
- Left sidebar with settings categories
- Right panel with settings fields
- Save button (bottom right)
- Reset to defaults button (bottom left)
- Cancel button

**Settings Container:**
- `settings/mod.rs` (130 lines) - Master container

**Individual Settings Modules:**

15. **General Settings** (175 lines, 15 tests)
16. **Audio Settings** (230 lines, 18 tests)
17. **Display Settings** (200 lines, 15 tests)
18. **Keyboard Settings** (270 lines, 20 tests)
19. **MIDI Settings** (210 lines, 18 tests)
20. **Mixer Settings** (195 lines, 16 tests)
21. **Track Settings** (180 lines, 15 tests)
22. **Import/Export Settings** (230 lines, 18 tests)
23. **Performance Settings** (220 lines, 17 tests)
24. **Library Settings** (200 lines, 16 tests)
25. **Playback Settings** (180 lines, 15 tests)
26. **Recording Settings** (230 lines, 17 tests)
27. **Sync Settings** (170 lines, 15 tests)
28. **Privacy Settings** (180 lines, 14 tests)
29. **Advanced Settings** (260 lines, 20 tests)

**Total Settings Backend:** 3,320 lines, 249 tests

**Backend Commands (3):**
1. `get_settings()` → `AppSettings`
2. `update_settings(settings: AppSettings)` → `Result<(), String>`
3. `reset_settings()` → `Result<(), String>`

**Frontend (To be created):**
- `SettingsWindow.svelte` (container, ~300 lines)
- 17 individual settings panels (~200 lines each)

**Total Estimated Frontend:** ~3,700 lines

**Size:** 1000px × 700px

---

(Continued in WINDOW-REFERENCE-PART-2.md due to length...)
