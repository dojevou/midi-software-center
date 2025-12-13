# MIDI Software Center - 100% Completion Task List

## Overview
This document lists ALL remaining tasks to complete the upgrades.txt specification.
**Total Tasks: 89** | **Estimated Terminal Sessions: 6**

---

## TERMINAL 1: VIP3 Database Browser (Frontend)
**Focus: Complete the VIP3-style filtering system**

### 1.1 VIP3 Column Filters (HIGH PRIORITY)
- [ ] Implement 6-column filter layout (Folder, Instrument, Timbre, Style, Articulation, BPM)
- [ ] Add KEY column filter with all 24 keys (C, Cm, C#, C#m, etc.)
- [ ] Add CHANNEL column filter (1-16, grouped options)
- [ ] Implement multi-select mode (checkboxes) for Timbre column
- [ ] Implement multi-select mode for Style column
- [ ] Add file counts next to each filter option (e.g., "Bass (245,891)")
- [ ] Add "Clear All" button for active filters
- [ ] Display active filters as removable chips/tags

### 1.2 Result List Features
- [ ] Add star rating system (0-5 stars, clickable)
- [ ] Add favorite toggle (star icon)
- [ ] Add mini piano roll preview thumbnail (optional toggle)
- [ ] Add context menu (right-click): Preview, Add to Sequencer, Edit Tags, Show in Folder, Delete
- [ ] Implement pagination controls (Page X of Y, items per page dropdown)
- [ ] Add sort options: Name, Date Added, Date Modified, BPM, Key, Duration, Rating

### 1.3 Preview Panel Integration
- [ ] Show MIDI info panel: Channel, Notes, Velocity range, Duration, Format
- [ ] Add "Add to Sequencer" button
- [ ] Add "Similar Files" section
- [ ] Implement preview playback with transport controls

### 1.4 Saved Searches
- [ ] Implement "Save Current Search" functionality
- [ ] Load saved searches from database
- [ ] Display saved searches in UI
- [ ] Add recent searches history

---

## TERMINAL 2: MIDI Mixer Completion (Frontend + Backend)
**Focus: Complete all mixer parameters from upgrades.txt**

### 2.1 Missing Mixer Parameters (per-channel)
- [ ] Velocity Scale slider (1-200%)
- [ ] Velocity Min/Max range inputs (0-127)
- [ ] Velocity Offset slider (-127 to +127)
- [ ] Pitch Bend slider (-8192 to +8191)
- [ ] Note Range Low/High dropdowns (C-1 to G9)
- [ ] Quantize dropdown (Off, 1/4, 1/8, 1/16, 1/32)
- [ ] Quantize Strength slider (0-100%)
- [ ] Swing slider (0-100%)
- [ ] Delay slider (-100 to +100 ms)
- [ ] Program Change input (0-127)
- [ ] Bank Select MSB/LSB inputs

### 2.2 Master Channel
- [ ] Implement Master channel strip with global controls
- [ ] Link Master to all channels
- [ ] Master Transpose
- [ ] Master Velocity Scale
- [ ] Master Mute All button

### 2.3 Backend Commands (daw/src-tauri/src/commands/mixer.rs)
- [ ] `set_velocity_scale` command
- [ ] `set_velocity_range` command
- [ ] `set_velocity_offset` command
- [ ] `set_pitch_bend` command
- [ ] `set_note_range` command
- [ ] `set_quantize` command
- [ ] `set_quantize_strength` command
- [ ] `set_swing` command
- [ ] `set_delay` command
- [ ] `set_program_change` command
- [ ] `set_bank_select` command

---

## TERMINAL 3: Sequencer & Project System (Frontend + Backend)
**Focus: Complete sequencer and project persistence**

### 3.1 Sequencer Features
- [ ] Implement clip drag-and-drop from database browser
- [ ] Add track collapse/expand functionality
- [ ] Implement loop region selection and display
- [ ] Add playhead visualization
- [ ] Implement MIDI drag-drop onto tracks
- [ ] Track output port selection (Port A, B, C...)

### 3.2 Project System
- [ ] Save project to file (.msproj or JSON)
- [ ] Load project from file
- [ ] Auto-save functionality
- [ ] Recent projects list
- [ ] Project browser window
- [ ] New Project / Save As dialogs

### 3.3 Transport
- [ ] MIDI sync source dropdown (Internal, External, MTC)
- [ ] Send MIDI Clock toggle per port
- [ ] Send Start/Stop toggle
- [ ] Loop toggle with region display
- [ ] BPM tap tempo button

### 3.4 Backend Commands
- [ ] `save_project` command
- [ ] `load_project` command
- [ ] `get_recent_projects` command
- [ ] `add_clip_to_track` command
- [ ] `remove_clip_from_track` command
- [ ] `set_loop_region` command
- [ ] `set_sync_source` command

---

## TERMINAL 4: Window System & Layouts (Frontend)
**Focus: Pro Tools-style window management**

### 4.1 Window Management
- [ ] Implement window snap to edges (left 50%, right 50%, maximize)
- [ ] Implement tabbed windows (drag window onto another to tab)
- [ ] Persistent window positions (save/restore on app restart)
- [ ] Window z-index management (click to bring to front)
- [ ] Double-click title bar to maximize/restore

### 4.2 Window Dock
- [ ] Implement bottom dock bar for minimized windows
- [ ] Click dock item to restore window
- [ ] Right-click dock item for options
- [ ] Drag to reorder dock items

### 4.3 Saved Layouts
- [ ] Save current window layout to named preset
- [ ] Load saved layout
- [ ] Built-in preset layouts: "Browse & Edit", "Performance", "Full Edit"
- [ ] Reset to Default layout option

### 4.4 Window Menu
- [ ] View > Windows submenu with all windows
- [ ] Show checkmarks for visible windows
- [ ] Keyboard shortcuts for each window (Cmd+1 through Cmd+7)

### 4.5 Keyboard Shortcuts
- [ ] Cmd+1: Database
- [ ] Cmd+2: Sequencer
- [ ] Cmd+3: MIDI Mixer
- [ ] Cmd+4: Piano Roll
- [ ] Cmd+5: CC Editor
- [ ] Cmd+6: MIDI Monitor
- [ ] Cmd+7: I/O Setup
- [ ] Cmd+M: Minimize All
- [ ] Cmd+T: Tile Windows
- [ ] Cmd+K: Command Palette

---

## TERMINAL 5: Piano Roll & Automation (Frontend + Backend)
**Focus: Complete editing tools**

### 5.1 Piano Roll Tools
- [ ] Implement Slice tool (split notes)
- [ ] Implement Stretch tool (resize notes)
- [ ] Scale quantize (snap to scale)
- [ ] Note info display (Start, End, Velocity, Channel, Length)
- [ ] Velocity lane editing with draw tool

### 5.2 CC Automation
- [ ] Live CC recording from MIDI input
- [ ] Curve types: Linear, Exponential, S-Curve, Hold
- [ ] Copy/paste automation
- [ ] Clear automation for range

### 5.3 Backend Commands
- [ ] `slice_note` command
- [ ] `stretch_note` command
- [ ] `quantize_to_scale` command
- [ ] `record_cc_automation` command
- [ ] `copy_automation` command
- [ ] `paste_automation` command

---

## TERMINAL 6: Database Schema & Backend APIs (Backend)
**Focus: Complete VIP3 schema and APIs**

### 6.1 Database Migrations
- [ ] Create `timbres` table with 21 entries
- [ ] Create `styles` table with 24 entries
- [ ] Create `articulations` table with 20 entries
- [ ] Create `bpm_ranges` table with 8 entries
- [ ] Create `musical_keys` table with 24 entries
- [ ] Create `midi_file_timbres` junction table
- [ ] Create `midi_file_styles` junction table
- [ ] Create `midi_file_articulations` junction table
- [ ] Create `saved_searches` table
- [ ] Create `collections` table
- [ ] Create `collection_files` junction table
- [ ] Add all indexes from upgrades.txt schema

### 6.2 Backend Search/Filter APIs
- [ ] `search_files_vip3` - VIP3-style multi-filter search
- [ ] `get_filter_counts` - Get counts for each filter option
- [ ] `get_timbres` / `get_styles` / `get_articulations` - List lookups
- [ ] `set_file_timbre` / `remove_file_timbre` - Tag management
- [ ] `set_file_style` / `remove_file_style`
- [ ] `set_file_articulation` / `remove_file_articulation`
- [ ] `save_search` / `get_saved_searches` / `delete_saved_search`
- [ ] `create_collection` / `add_to_collection` / `remove_from_collection`

### 6.3 File Rating/Favorites
- [ ] `set_file_rating` command (0-5)
- [ ] `toggle_favorite` command
- [ ] `get_favorites` command
- [ ] `increment_play_count` command

---

## Quick Reference: What Each Terminal Does

| Terminal | Focus Area | Files Modified |
|----------|------------|----------------|
| **T1** | VIP3 Browser UI | `VIP3BrowserWindow.svelte`, `vip3Store.ts`, `DatabaseWindow.svelte` |
| **T2** | MIDI Mixer | `MidiMixerWindow.svelte`, `midiMixerStore.ts`, `daw/commands/mixer.rs` |
| **T3** | Sequencer/Projects | `SequencerWindow.svelte`, `sequencerStore.ts`, `daw/commands/project.rs` |
| **T4** | Window System | `WindowBase.svelte`, `uiStore.ts`, `WindowDockBar.svelte` |
| **T5** | Piano Roll/CC | `PianoRollWindow.svelte`, `AutomationWindow.svelte`, `daw/commands/` |
| **T6** | Database/Backend | `database/migrations/`, `pipeline/src-tauri/src/commands/`, repositories |

---

## Priority Order

1. **T6 (Database)** - Start first, others depend on it
2. **T1 (VIP3 Browser)** - Core user-facing feature
3. **T2 (Mixer)** - Essential for MIDI control
4. **T3 (Sequencer)** - Project workflow
5. **T4 (Windows)** - Polish/UX
6. **T5 (Editing)** - Advanced features

---

## Total Estimates

- **Tasks**: 89
- **Backend commands to add**: ~25
- **Database migrations**: ~12 tables/indexes
- **Frontend components to update**: ~15
- **Estimated completion**: 6 parallel terminals working ~2-4 hours each
