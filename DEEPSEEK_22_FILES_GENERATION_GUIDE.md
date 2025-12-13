# DeepSeek: Generate 22 Project Files for MIDI Software Center

## PROJECT CONTEXT

You are generating files for a **Tauri 2.7 + Svelte 4.2 + TypeScript** desktop application for MIDI library management (1.7M+ files).

### Critical Technical Requirements

1. **Svelte 4.2** - Use `let` variables, NOT `$state()` runes (that's Svelte 5)
2. **TypeScript strict mode** - All types must be explicit
3. **API calls** - Use `import { api } from '$lib/api'` then `api.category.method()`
4. **Stores** - Use `import { writable, derived } from 'svelte/store'`
5. **Types** - Import from `'$lib/types'`
6. **Dark theme** - Use `dark:` Tailwind prefixes (e.g., `dark:bg-window`)
7. **No emojis in code** unless explicitly in UI text

---

## EXISTING PATTERNS TO FOLLOW

### Store Pattern (from uiStore.ts)
```typescript
import { writable, derived } from 'svelte/store';
import type { SomeType } from '$lib/types';

export interface StoreState {
  // state shape
}

const initialState: StoreState = {
  // defaults
};

export const myStore = writable<StoreState>(initialState);

export const myActions = {
  someAction(param: string) {
    myStore.update(state => ({ ...state, field: param }));
  },
};
```

### Component Pattern (from DAWWindow.svelte)
```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '$lib/api';
  import type { SomeType } from '$lib/types';

  let someData: SomeType[] = [];
  let isLoading = false;

  onMount(async () => {
    try {
      someData = await api.category.method();
    } catch (error) {
      console.error('Failed:', error);
    }
  });

  async function handleAction() {
    try {
      await api.category.method();
    } catch (error) {
      console.error('Failed:', error);
    }
  }
</script>

<div class="component-name dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
  <!-- content -->
</div>

<style>
  /* component-specific styles */
</style>
```

### API Pattern (from api.ts)
```typescript
import { invoke } from '@tauri-apps/api/core';
import { Commands } from './api/commands';

export const api = {
  category: {
    method: async (param: Type): Promise<ReturnType> => {
      try {
        return await invoke(Commands.COMMAND_NAME, { param });
      } catch (error) {
        console.error('Failed:', error);
        throw error;
      }
    },
  },
};
```

---

## EXISTING TYPES (from types.ts) - USE THESE

```typescript
// Already defined - DO NOT redefine:
export interface FileDetails { id, filename, filepath, bpm?, key_signature?, tags, ... }
export interface SearchFilters { search_text?, min_bpm?, max_bpm?, ... }
export interface Track { id, file_id, channel, muted, soloed, volume, pan }
export interface TrackInfo { id, label, visible, muted, soloed }
export interface PlaybackPosition { current_bar, current_beat, current_tick }
export interface PlaybackState { current_bar, current_beat, current_tick, is_playing, is_recording, bpm }
export interface MidiDevice { name, port_number, is_available }
export interface AutomationLane { id, track_id, parameter_type, curve_type, points }
export interface AutomationPoint { id, tick, value }
export interface Tag { id, name, category, count }
export interface TagStats { total_tags, total_file_tags, top_tags, categories }
export interface MixerState { channels, master, show_meters, show_effects }
export interface MixerChannel { id, channel_type, label, volume, pan, muted, soloed, meter_level }
export interface EffectSlot { id, name, enabled, wet_dry, parameters }
export type WindowId = 'database' | 'daw' | 'mixer' | 'pipeline';
export type ParameterType = 'volume' | 'pan' | 'pitch' | 'filter' | 'custom';
export type CurveType = 'linear' | 'exponential' | 'logarithmic' | 'bezier';
```

---

## EXISTING API METHODS (from api.ts) - USE THESE

```typescript
// Already available - call these directly:
api.midi.listDevices()
api.midi.connect(deviceName)
api.midi.disconnect()
api.midi.sendTestNote(channel, note, velocity)

api.sequencer.start() / stop() / pause() / resume()
api.sequencer.getPosition() / seekPosition(bar, beat)
api.sequencer.setTempo(bpm) / getTempo()
api.sequencer.addTrack(fileId, channel) / removeTrack(trackId)

api.search.files(filters) / getDetails(fileId) / getSuggestions(query, field)

api.analysis.findCompatible(fileId) / addFavorite(fileId) / removeFavorite(fileId)
api.analysis.getFavorites() / isFavorite(fileId)

api.window.playTransport() / stopTransport() / pauseTransport()
api.window.setBpm(bpm) / getBpm()
api.window.setTimeSignature(num, den) / getTimeSignature()
api.window.setKeySignature(key) / getKeySignature()
api.window.addWindowTrack(label) / removeWindowTrack(trackId)
api.window.getAllWindowTracks() / getTrackInfo(trackId)
api.window.setTrackMuted(trackId, muted) / setTrackSoloed(trackId, soloed)
api.window.getMixerState() / setChannelVolume(trackId, volume) / setChannelPan(trackId, pan)

api.automation.createLane(trackId, paramType) / deleteLane(laneId)
api.automation.getAllLanes(trackId)
api.automation.addPoint(laneId, tick, value) / updatePoint(pointId, tick, value) / deletePoint(pointId)
api.automation.setCurveType(laneId, curveType)

api.tags.getAll() / getPopular(limit) / search(query)
api.tags.getCategories() / getByCategory(category)
api.tags.getFileTags(fileId) / addToFile(fileId, tags) / removeFromFile(fileId, tag)
api.tags.getStats()

api.database.search(filters) / getStats() / getFileMetadata(id)
api.pipeline.importFiles(paths) / analyzeFiles(ids) / getProgress() / cancel()
api.split.file(fileId, outputDir) / batch(fileIds, outputDir)
```

---

## 22 FILES TO GENERATE

### SECTION 1: FRONTEND WINDOWS (12 files)
Location: `app/src/lib/windows/`

#### File 1: PianoRollWindow.svelte
**Purpose:** MIDI note editor with grid display
**Required Features:**
- Note grid with piano keyboard on left
- Tools: select, draw, erase, slice
- Quantize, transpose, delete operations
- Velocity editing sidebar
- Playback cursor and loop range display
- Grid size selector (1/4, 1/8, 1/16, 1/32)
- Zoom controls
- Snap to grid toggle

**API calls needed:**
```typescript
// These DON'T exist yet - mark as TODO or stub
await invoke('get_track_notes', { trackId })
await invoke('add_note', { trackId, note })
await invoke('update_notes_batch', { notes })
await invoke('delete_notes', { noteIds })
```

**Props:** `export let trackId: number;`

---

#### File 2: AutomationWindow.svelte
**Purpose:** Automation lane editor for parameter curves
**Required Features:**
- Lane list sidebar (volume, pan, filter, etc.)
- Visual curve editor with points
- Point drag/drop editing
- Curve type selector (linear, cubic, hold)
- Recording mode for live automation
- Point list table with precise editing

**API calls:** Use `api.automation.*` methods

**Props:** `export let trackId: number;`

---

#### File 3: TagEditorWindow.svelte
**Purpose:** Tag management interface
**Required Features:**
- Tag list with search/filter
- Category dropdown filter
- Create/edit/delete tags
- Tag cloud visualization
- Bulk operations (merge, delete selected)
- Import/export CSV
- Tag statistics display

**API calls:** Use `api.tags.*` methods

**Props:** None (global tag management)

---

#### File 4: FavoritesWindow.svelte
**Purpose:** Favorites browser
**Required Features:**
- Grid/list/detailed view modes
- Sort by: filename, bpm, date added
- Search within favorites
- Category grouping
- Play file, analyze compatibility
- Remove from favorites
- Export favorites list

**API calls:** Use `api.analysis.getFavorites()`, `api.analysis.removeFavorite()`

**Props:** None

---

#### File 5: SettingsWindow.svelte
**Purpose:** Application settings
**Required Features:**
- Tabbed interface: General, Audio, MIDI, Appearance, Shortcuts, Paths, Advanced
- General: language, default BPM, time sig, auto-save
- Audio: driver, sample rate, buffer size, device selection
- MIDI: input/output device configuration
- Appearance: theme toggle
- Paths: library path, audio path selection
- Import/export settings JSON

**API calls needed:**
```typescript
// These DON'T exist yet - stub them
await invoke('get_settings')
await invoke('save_settings', { settings })
await invoke('get_audio_devices')
await invoke('test_audio_output')
```

**Props:** None

---

#### File 6: ExportWindow.svelte
**Purpose:** Export dialog
**Required Features:**
- Format selection (MIDI, WAV, MP3)
- Output path selection
- Export options (normalize, include metadata)
- Progress indicator during export
- Batch export for multiple files

**API calls:** Use `api.export.projectAsMidi()`

**Props:** `export let fileIds: number[] = [];`

---

#### File 7: ProjectBrowserWindow.svelte
**Purpose:** Project file browser
**Required Features:**
- Tree view file browser
- Recent projects list
- New project creation
- Open/save project dialogs
- Project templates

**API calls needed:**
```typescript
// Stub these
await invoke('list_projects')
await invoke('open_project', { path })
await invoke('save_project', { path })
await invoke('create_project', { name, template })
```

**Props:** None

---

#### File 8: LoopBrowserWindow.svelte
**Purpose:** Loop/sample browser
**Required Features:**
- File tree navigation
- Preview playback
- BPM/key filtering
- Drag to DAW support
- Favorites marking
- Tag filtering

**API calls:** Use `api.search.files()`, `api.analysis.*`

**Props:** None

---

#### File 9: VelocityEditorWindow.svelte
**Purpose:** Velocity editing for selected notes
**Required Features:**
- Bar chart visualization of velocities
- Drag to adjust individual velocities
- Humanize function (randomize within range)
- Compress/expand velocity range
- Set all to value
- Velocity curve presets

**API calls needed:**
```typescript
await invoke('get_note_velocities', { trackId })
await invoke('update_velocities', { noteIds, velocities })
```

**Props:** `export let trackId: number;`

---

#### File 10: CommandPaletteWindow.svelte
**Purpose:** Command palette UI (Ctrl+P style)
**Required Features:**
- Search input with fuzzy matching
- Command list with keyboard navigation
- Recent commands section
- Category filtering
- Keyboard shortcut display

**API calls needed:**
```typescript
await invoke('get_available_commands')
await invoke('execute_command', { commandId })
```

**Props:** None (modal overlay)

---

#### File 11: MIDIDeviceWindow.svelte
**Purpose:** MIDI device configuration
**Required Features:**
- Input device list with enable/disable
- Output device list with enable/disable
- Channel routing configuration
- Test buttons for each device
- Auto-connect settings
- Device activity indicators

**API calls:** Use `api.midi.*` methods

**Props:** None

---

#### File 12: FileDetailsWindow.svelte
**Purpose:** File info panel
**Required Features:**
- File metadata display (name, path, size, dates)
- Musical metadata (BPM, key, time sig, duration)
- Track list with instruments
- Tag display/editing
- Waveform preview
- Compatibility suggestions
- Add/remove favorite button

**API calls:** Use `api.search.getDetails()`, `api.tags.getFileTags()`, `api.analysis.*`

**Props:** `export let fileId: number;`

---

### SECTION 2: FRONTEND COMPONENTS (15 files)
Location: `app/src/lib/components/`

#### File 13: TransportBar.svelte
**Purpose:** Unified transport controls
**Features:** Play, pause, stop, record, rewind, fast-forward, loop toggle, metronome toggle, BPM display/edit, time signature, position display
**API calls:** Use `api.window.*` transport methods

---

#### File 14: TrackList.svelte
**Purpose:** Reusable track listing
**Features:** Track rows with mute/solo/volume, drag reorder, selection, context menu
**Props:** `export let tracks: TrackInfo[] = [];`

---

#### File 15: NoteGrid.svelte
**Purpose:** Piano roll grid component
**Features:** SVG-based grid, note rectangles, selection highlighting, playback cursor, loop markers
**Props:** `export let notes, gridSize, zoomLevel, playbackPosition, selectedNotes, loopRange`

---

#### File 16: AutomationLane.svelte
**Purpose:** Automation curve editor
**Features:** SVG curve rendering, draggable points, curve interpolation visualization
**Props:** `export let laneId, points, timeRange, curveType`
**Events:** `on:pointMoved`, `on:pointDeleted`, `on:pointAdded`

---

#### File 17: EffectRack.svelte
**Purpose:** Effect slot chain
**Features:** Vertical effect slots, enable/disable, wet/dry knob, reorder drag, add/remove effects
**Props:** `export let trackId: number;`

---

#### File 18: MIDIEventList.svelte
**Purpose:** Event editing table
**Features:** Sortable columns (time, type, channel, data), inline editing, selection, delete
**Props:** `export let events: MidiEvent[] = [];`

---

#### File 19: ProgressIndicator.svelte
**Purpose:** Long operation progress
**Features:** Progress bar, percentage, ETA, current file, cancel button, rate display
**Props:** `export let progress: PipelineProgress;`

---

#### File 20: FileBrowser.svelte
**Purpose:** Tree view browser
**Features:** Expandable folders, file icons, selection, double-click to open, context menu
**Props:** `export let rootPath: string;`
**Events:** `on:fileSelect`, `on:fileOpen`

---

#### File 21: TagCloud.svelte
**Purpose:** Tag visualization
**Features:** Weighted font sizes based on count, clickable tags, hover effects
**Props:** `export let tags: Tag[] = []; export let maxFontSize = 48; export let minFontSize = 12;`
**Events:** `on:tagClick`

---

#### File 22: WaveformView.svelte
**Purpose:** Audio waveform display
**Features:** Canvas-based waveform, zoom, playhead position, selection range
**Props:** `export let audioData: number[] = []; export let position = 0;`

---

#### File 23: VirtualKeyboard.svelte
**Purpose:** On-screen keyboard
**Features:** 2-3 octave piano keyboard, click/touch to play, velocity based on click position
**Props:** `export let octaves: number[] = [3, 4, 5];`
**Events:** `on:noteOn(note, velocity)`, `on:noteOff(note)`

---

#### File 24: Toolbar.svelte
**Purpose:** Contextual toolbar
**Features:** Icon buttons with tooltips, separators, dropdown menus, toggle states
**Props:** `export let items: ToolbarItem[] = []; export let selectedTool = '';`
**Type needed:** `interface ToolbarItem { id: string; icon: string; tooltip: string; action?: () => void; separator?: boolean; }`

---

#### File 25: Knob.svelte
**Purpose:** Rotary knob control
**Features:** Drag rotation, value display, min/max/step, label
**Props:** `export let value = 0; export let min = 0; export let max = 100; export let step = 1; export let label = '';`
**Events:** `on:change`

---

#### File 26: Slider.svelte
**Purpose:** Vertical/horizontal slider
**Features:** Drag handle, value display, orientation prop
**Props:** `export let value = 0; export let min = 0; export let max = 100; export let orientation: 'vertical' | 'horizontal' = 'horizontal';`
**Events:** `on:change`

---

#### File 27: VUMeter.svelte
**Purpose:** Audio level meter
**Features:** Animated level display, peak hold, stereo option, clip indicator
**Props:** `export let level = 0; export let peak = 0; export let stereo = false;`

---

### SECTION 3: FRONTEND STORES (7 files)
Location: `app/src/lib/stores/`

#### File 28: pianoRollStore.ts
**State:** selectedNotes, gridSize, zoomLevel, snapToGrid, currentTool, loopRange, clipboard
**Actions:** selectNotes, clearSelection, setTool, setGridSize, setZoom, toggleSnap, copy, paste, cut

---

#### File 29: automationStore.ts
**State:** lanes, selectedLaneId, curveType, recording, recordingParameter
**Actions:** selectLane, addLane, removeLane, setCurveType, startRecording, stopRecording

---

#### File 30: tagStore.ts (ALREADY EXISTS - check and extend if needed)
**State:** allTags, selectedTags, categories, searchQuery, filterCategory
**Actions:** loadTags, selectTag, deselectTag, filterByCategory, searchTags

---

#### File 31: favoritesStore.ts
**State:** favorites, viewMode, sortBy, sortDesc, searchQuery
**Actions:** loadFavorites, addFavorite, removeFavorite, setViewMode, setSortBy, search

---

#### File 32: settingsStore.ts
**State:** settings (nested: general, audio, midi, appearance, shortcuts, paths)
**Actions:** loadSettings, saveSettings, updateSetting, resetToDefaults

---

#### File 33: midiDeviceStore.ts
**State:** inputDevices, outputDevices, connectedInputs, connectedOutputs, isScanning
**Actions:** scanDevices, connectInput, disconnectInput, connectOutput, disconnectOutput

---

#### File 34: undoStore.ts
**State:** undoStack, redoStack, canUndo, canRedo
**Actions:** push, undo, redo, clear

---

### SECTION 4: BACKEND DAW COMMANDS (3 files)
Location: `daw/src-tauri/src/commands/`

#### File 35: piano_roll.rs
**Commands:**
```rust
#[tauri::command]
pub async fn get_track_notes(track_id: i32) -> Result<Vec<Note>, String>

#[tauri::command]
pub async fn add_note(track_id: i32, note: NoteInput) -> Result<Note, String>

#[tauri::command]
pub async fn update_notes_batch(notes: Vec<NoteUpdate>) -> Result<(), String>

#[tauri::command]
pub async fn delete_notes(note_ids: Vec<i32>) -> Result<(), String>

#[tauri::command]
pub async fn quantize_notes(note_ids: Vec<i32>, grid_size: i32) -> Result<(), String>

#[tauri::command]
pub async fn transpose_notes(note_ids: Vec<i32>, semitones: i32) -> Result<(), String>
```

---

#### File 36: effect.rs
**Commands:**
```rust
#[tauri::command]
pub async fn add_effect(track_id: i32, effect_type: String) -> Result<EffectSlot, String>

#[tauri::command]
pub async fn remove_effect(track_id: i32, effect_id: i32) -> Result<(), String>

#[tauri::command]
pub async fn get_track_effects(track_id: i32) -> Result<Vec<EffectSlot>, String>

#[tauri::command]
pub async fn set_effect_enabled(effect_id: i32, enabled: bool) -> Result<(), String>

#[tauri::command]
pub async fn set_effect_wet_dry(effect_id: i32, wet_dry: f32) -> Result<(), String>
```

---

#### File 37: settings.rs
**Commands:**
```rust
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String>

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String>

#[tauri::command]
pub async fn get_audio_devices() -> Result<AudioDevices, String>

#[tauri::command]
pub async fn test_audio_output() -> Result<(), String>

#[tauri::command]
pub async fn reset_settings() -> Result<AppSettings, String>
```

---

### SECTION 5: BACKEND PIPELINE COMMANDS (2 files)
Location: `pipeline/src-tauri/src/commands/`

#### File 38: repair.rs
**Commands:**
```rust
#[tauri::command]
pub async fn repair_midi_file(file_id: i64) -> Result<RepairResult, String>

#[tauri::command]
pub async fn repair_batch(file_ids: Vec<i64>) -> Result<Vec<RepairResult>, String>

#[tauri::command]
pub async fn get_corruption_report(file_id: i64) -> Result<CorruptionReport, String>
```

---

#### File 39: trim.rs
**Commands:**
```rust
#[tauri::command]
pub async fn trim_leading_silence(file_id: i64) -> Result<TrimResult, String>

#[tauri::command]
pub async fn trim_batch(file_ids: Vec<i64>) -> Result<Vec<TrimResult>, String>

#[tauri::command]
pub async fn get_silence_info(file_id: i64) -> Result<SilenceInfo, String>
```

---

### SECTION 6: DATABASE MIGRATIONS (5 files)
Location: `database/migrations/`

#### File 40: 004_missing.sql
```sql
-- Fill numbering gap - placeholder migration
-- No actual changes, maintains sequence integrity
SELECT 1;
```

---

#### File 41: 005_missing.sql
```sql
-- Fill numbering gap - placeholder migration
SELECT 1;
```

---

#### File 42: database/rollbacks/001_rollback.sql
```sql
-- Rollback for initial schema
DROP TABLE IF EXISTS file_tags CASCADE;
DROP TABLE IF EXISTS tags CASCADE;
DROP TABLE IF EXISTS musical_metadata CASCADE;
DROP TABLE IF EXISTS files CASCADE;
DROP INDEX IF EXISTS idx_files_hash;
DROP INDEX IF EXISTS idx_files_filename;
```

---

#### File 43: database/rollbacks/002_rollback.sql
```sql
-- Rollback for parent folder migration
ALTER TABLE files DROP COLUMN IF EXISTS parent_folder;
DROP INDEX IF EXISTS idx_files_parent_folder;
```

---

#### File 44: database/rollbacks/003_rollback.sql
```sql
-- Rollback for favorite migration
ALTER TABLE files DROP COLUMN IF EXISTS is_favorite;
ALTER TABLE files DROP COLUMN IF EXISTS favorited_at;
DROP INDEX IF EXISTS idx_files_is_favorite;
```

---

## STYLE GUIDE

### Tailwind Classes to Use
```
dark:bg-window          - Main window background
dark:bg-window-subtle   - Subtle background variation
dark:bg-menu            - Menu/header background
dark:bg-input           - Input field background
dark:bg-primary         - Primary action buttons
dark:bg-secondary       - Secondary buttons
dark:bg-error           - Error/destructive actions
dark:bg-success         - Success indicators
dark:text-app-text      - Primary text
dark:text-gray-300      - Secondary text
dark:text-gray-400      - Tertiary/muted text
dark:border-window-border - Borders
```

### Event Handling
```svelte
on:click={handleClick}
on:click|stopPropagation={handleClick}
on:click|preventDefault={handleClick}
on:mousedown={handleMouseDown}
on:input={(e) => value = e.target.value}
on:change={(e) => handleChange(e.target.value)}
```

### Conditional Classes
```svelte
class:dark:bg-primary={isActive}
class:selected={selectedId === item.id}
```

---

## GENERATION ORDER

Generate in this order for best results:
1. Stores first (they're dependencies)
2. Small components (Knob, Slider, VUMeter)
3. Medium components (TransportBar, TrackList)
4. Complex components (NoteGrid, AutomationLane)
5. Window components (they use the components)
6. Backend commands
7. Database migrations

---

## FINAL CHECKLIST FOR EACH FILE

- [ ] Uses `let` not `$state()` (Svelte 4)
- [ ] Imports from `'$lib/api'` for API calls
- [ ] Imports types from `'$lib/types'`
- [ ] Uses `dark:` Tailwind prefixes
- [ ] Has proper TypeScript types
- [ ] Includes error handling with try/catch
- [ ] Has `onMount`/`onDestroy` for lifecycle
- [ ] Component has descriptive class name
- [ ] Stores export both store and actions objects
