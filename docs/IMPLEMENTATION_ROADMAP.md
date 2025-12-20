# Implementation Roadmap - MIDI Software Center

**Generated:** 2025-12-15
**Current Status:** ~75% Complete
**Target:** 100% Feature-Complete per Copilot Instructions

---

## Quick Start Guide

**New to this project?** Start here:

1. Read `IMPLEMENTATION_STATUS.md` - Understand what's done vs. what's missing
2. Read `COMPLETE_IMPLEMENTATION_PLAN.md` - See the full feature checklist
3. Follow this roadmap - Implement missing features in priority order

---

## Current State (75% Complete)

### ✅ Solid Foundation
- **Database:** 15 tables, 60+ indexes, migrations working
- **MIDI Analysis:** BPM, key, drums, chords all functional
- **Pipeline:** Import (7,830/sec), split, analyze working
- **Hardware:** JACK/ALSA/CoreMIDI/midir backends auto-detect
- **Testing:** 1,999 tests passing, ~75% coverage

### ⚠️ Partial Features
- **DAW:** Basic sequencer works, mixer/automation/presets incomplete
- **VIP3:** Search works, filter counts/saved searches/collections missing
- **Frontend:** Pipeline UI complete, DAW/VIP3 UI partial

### ❌ Future Features
- **Lua Scripting:** Not implemented (dependency ready)
- **Meilisearch:** Not integrated (Docker configured)
- **Drag-and-Drop:** Planned but not implemented

---

## Priority Roadmap

### Phase 1: VIP3 Completion (1 week)
**Goal:** Make VIP3 browser fully functional

#### Week 1, Day 1-2: Filter Counts System
- [ ] Implement `get_vip3_filter_counts` command
  - [ ] Location: `app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs`
  - [ ] Build dynamic COUNT queries for each filter type
  - [ ] Use partial indexes for performance (<50ms target)
  - [ ] Return `FilterCounts` struct with all counts
- [ ] Update `VIP3Column.svelte` to display counts
  - [ ] Show "Folder Name (123)" format
  - [ ] Update counts on filter change
- [ ] Test with 2.15M files, verify <50ms response

#### Week 1, Day 3: Saved Searches
- [ ] Implement `SearchRepository`
  - [ ] Location: `app/src-tauri/src/db/repositories/search_repository.rs`
  - [ ] `save_search(name, filters)` - Insert into `saved_searches` table
  - [ ] `load_saved_search(id)` - Fetch and increment use_count
  - [ ] `get_saved_searches()` - List all, order by last_used
  - [ ] `delete_saved_search(id)` - Remove search
- [ ] Add Tauri commands: `save_search`, `load_saved_search`, `get_saved_searches`, `delete_saved_search`
- [ ] Create `VIP3SavedSearches.svelte` component
  - [ ] List saved searches with use count
  - [ ] Click to load search (apply filters)
  - [ ] Delete button

#### Week 1, Day 4: Collections
- [ ] Implement `CollectionRepository`
  - [ ] Location: `app/src-tauri/src/db/repositories/collection_repository.rs`
  - [ ] `create_collection(name, description)` - Insert into `collections` table
  - [ ] `add_file_to_collection(collection_id, file_id, position)` - Add to `collection_files`
  - [ ] `remove_file_from_collection(collection_id, file_id)` - Remove from collection
  - [ ] `reorder_collection(collection_id, file_id, new_position)` - Update position
  - [ ] `get_collection_files(collection_id)` - Get files ordered by position
  - [ ] `delete_collection(collection_id)` - Remove collection
- [ ] Add Tauri commands for all collection operations
- [ ] Create `VIP3Collections.svelte` component
  - [ ] List collections
  - [ ] Click to view collection files
  - [ ] Drag to reorder (optional)
  - [ ] Add/remove files from VIP3 results

#### Week 1, Day 5: Favorites & Category Management
- [ ] Implement favorites commands
  - [ ] `toggle_favorite(file_id)` - Insert/delete from `favorites` table
  - [ ] `get_favorites(limit, offset)` - Fetch user favorites
  - [ ] `is_favorite(file_id)` - Check favorite status
- [ ] Implement VIP3 category commands
  - [ ] `add_timbre_to_file(file_id, timbre_id)` - Insert into `file_timbres`
  - [ ] `remove_timbre_from_file(file_id, timbre_id)` - Delete from `file_timbres`
  - [ ] Same for styles and articulations
  - [ ] `get_all_timbres()`, `get_all_styles()`, `get_all_articulations()` - List categories
- [ ] Create `VIP3Favorites.svelte` component
- [ ] Add category editing UI to file detail view

#### Week 1, Deliverables
- [x] VIP3 browser fully functional
- [x] Filter counts update in real-time (<50ms)
- [x] Saved searches working
- [x] Collections working
- [x] Favorites working
- [x] Category management working

---

### Phase 2: DAW Mixer System (1 week)
**Goal:** Complete mixer with all 30+ commands

#### Week 2, Day 1-2: Channel Control Commands
- [ ] Implement mixer commands in `app/src-tauri/src/commands/daw/mixer.rs`
  - [ ] `set_channel_gain(track_id, gain_db)` - Update mixer state
  - [ ] `get_channel_gain(track_id)` - Return current gain
  - [ ] `set_channel_pan(track_id, pan)` - -1.0 (left) to +1.0 (right)
  - [ ] `get_channel_pan(track_id)` - Return current pan
  - [ ] `set_channel_mute(track_id, muted)` - Mute/unmute track
  - [ ] `get_channel_mute(track_id)` - Return mute state
  - [ ] `set_channel_solo(track_id, solo)` - Solo track
  - [ ] `get_channel_solo(track_id)` - Return solo state
- [ ] Update `SequencerEngine` mixer state
  - [ ] Location: `app/src-tauri/src/sequencer/mixer.rs`
  - [ ] Add `MixerState` struct with per-track channels
  - [ ] Apply gain/pan during audio processing
  - [ ] Handle mute/solo logic (mute all non-solo tracks when solo active)

#### Week 2, Day 3: VU Metering
- [ ] Implement real-time metering
  - [ ] Calculate peak and RMS levels during playback
  - [ ] Store in shared state (lock-free if possible)
  - [ ] Detect clipping (>0dB)
- [ ] Add metering commands
  - [ ] `get_channel_meter(track_id)` - Return `{ peak_db, rms_db, clip_detected }`
  - [ ] `get_all_meters()` - Return meters for all tracks
  - [ ] `get_master_meter()` - Return master bus meter
- [ ] Frontend: Update meters 30-60 times/sec via event polling

#### Week 2, Day 4: Master Channel & Routing
- [ ] Implement master channel commands
  - [ ] `set_master_volume(gain_db)` - Global master gain
  - [ ] `get_master_volume()` - Return master gain
- [ ] Implement routing commands
  - [ ] `set_channel_output(track_id, output_device)` - Assign track to MIDI output
  - [ ] `get_channel_output(track_id)` - Return current output
  - [ ] `create_send_bus(name)` - Create aux send (optional, future)
  - [ ] `route_to_send(track_id, send_id, amount)` - Send routing (optional)

#### Week 2, Day 5: Effect Chain (Basic)
- [ ] Implement effect chain structure
  - [ ] 8 effect slots per track
  - [ ] Effect types: EQ, Compressor, Reverb, Delay (basic, can be stubs)
- [ ] Add effect commands
  - [ ] `add_effect(track_id, slot, effect_type, params)` - Insert effect
  - [ ] `remove_effect(track_id, slot)` - Remove effect
  - [ ] `bypass_effect(track_id, slot, bypassed)` - Bypass on/off
  - [ ] `set_effect_param(track_id, slot, param, value)` - Adjust parameter
  - [ ] `get_effect_chain(track_id)` - Return all effects on track

#### Week 2, Deliverables
- [x] All mixer commands implemented (30+)
- [x] Gain, pan, mute, solo working
- [x] VU meters updating in real-time
- [x] Master channel functional
- [x] Basic effect chain structure (stubs OK)
- [x] Frontend MixerPanel component complete

---

### Phase 3: DAW Advanced Features (1 week)
**Goal:** Automation, presets, project management

#### Week 3, Day 1-2: Automation System
- [ ] Design automation data structure
  - [ ] Location: `app/src-tauri/src/sequencer/automation.rs`
  - [ ] `AutomationLane` struct: parameter name, points (tick, value)
  - [ ] Per-track automation lanes (gain, pan, any effect param)
  - [ ] Automation modes: off, read, latch, touch, write
- [ ] Implement automation commands
  - [ ] `record_automation(track_id, param)` - Start recording automation
  - [ ] `play_automation(track_id, param)` - Enable automation playback
  - [ ] `clear_automation(track_id, param)` - Clear all automation points
  - [ ] `add_automation_point(track_id, param, tick, value)` - Manual point editing
  - [ ] `remove_automation_point(track_id, param, tick)` - Remove point
  - [ ] `get_automation_lane(track_id, param)` - Return all points
- [ ] Apply automation during playback (interpolate between points)
- [ ] Frontend: `AutomationLane.svelte` component
  - [ ] Visual curve editor with draggable points
  - [ ] Zoom/pan controls

#### Week 3, Day 3: Preset System
- [ ] Create `presets` database table (if not exists)
  - [ ] `id`, `name`, `preset_type` (track, mixer, effect, project), `data` (JSONB)
- [ ] Implement preset commands
  - [ ] `save_preset(name, preset_type, data)` - Serialize and save
  - [ ] `load_preset(id)` - Fetch and deserialize
  - [ ] `delete_preset(id)` - Remove preset
  - [ ] `get_presets(preset_type)` - List presets (optional filter by type)
- [ ] Preset types:
  - [ ] **Track preset:** All track settings (gain, pan, mute, effects, automation)
  - [ ] **Mixer preset:** All mixer state (all tracks)
  - [ ] **Effect preset:** Single effect settings
  - [ ] **Project template:** Full project structure

#### Week 3, Day 4-5: Project Management
- [ ] Create `projects` database table
  - [ ] `id`, `name`, `file_path`, `data` (JSONB with full session state), `created_at`, `updated_at`
- [ ] Implement project commands
  - [ ] `create_project(name)` - Create new project (empty session)
  - [ ] `open_project(project_id)` - Load project from DB
    - [ ] Deserialize project data
    - [ ] Load all tracks into sequencer
    - [ ] Restore mixer state, automation, tempo, markers
  - [ ] `save_project(project_id)` - Serialize and save current session
  - [ ] `save_project_as(name)` - Save as new project
  - [ ] `close_project()` - Clear sequencer state, prompt to save if modified
  - [ ] `get_recent_projects(limit)` - List recent projects (order by updated_at)
  - [ ] `delete_project(project_id)` - Remove project
- [ ] Project file format (JSON):
  ```json
  {
    "version": "1.0",
    "tempo": 120.0,
    "time_signature": "4/4",
    "tracks": [...],
    "mixer_state": {...},
    "automation": {...},
    "markers": [...]
  }
  ```
- [ ] Frontend: `ProjectManager.svelte` component
  - [ ] File menu: New, Open, Save, Save As, Close
  - [ ] Recent projects list

#### Week 3, Deliverables
- [x] Automation recording/playback working
- [x] Preset save/load working (all types)
- [x] Project create/save/load/close working
- [x] Frontend automation editor
- [x] Frontend project manager UI

---

### Phase 4: VIP3 ↔ DAW Integration (2-3 days)
**Goal:** Seamless file loading from VIP3 to DAW

#### Day 1: Convenience Wrapper
- [ ] Implement `load_file_to_daw` command
  - [ ] Location: `app/src-tauri/src/commands/daw/file_loader.rs`
  - [ ] Signature: `load_file_to_daw(file_id: i64) -> Result<u32, String>`
  - [ ] Steps:
    1. Fetch file path from database: `FileRepository::get_by_id(file_id)`
    2. Parse MIDI: `daw_load_midi_file(file_path)`
    3. Add to sequencer: `add_track(track)`
    4. Return track ID
  - [ ] Error handling: file not found, parse error, sequencer error

#### Day 2: Double-Click Integration
- [ ] Update `VIP3Results.svelte`
  - [ ] Add double-click handler to file cards
  - [ ] Call `invoke('load_file_to_daw', { fileId })`
  - [ ] Show success notification or error toast
  - [ ] Optional: Switch to DAW tab after loading

#### Day 3 (Optional): Drag-and-Drop
- [ ] Make file cards draggable in `VIP3Results.svelte`
  - [ ] Use HTML5 drag API: `draggable="true"`, `ondragstart`
  - [ ] Set drag data: `event.dataTransfer.setData('application/json', JSON.stringify({ file_id }))`
- [ ] Accept drops in `Sequencer.svelte`
  - [ ] `ondrop` handler
  - [ ] Parse drag data, extract file_id
  - [ ] Call `load_file_to_daw(file_id)`
  - [ ] Insert at drop position (optional: calculate bar/beat from drop coordinates)

#### Deliverables
- [x] `load_file_to_daw` command working
- [x] Double-click to load file in DAW
- [x] (Optional) Drag-and-drop working

---

### Phase 5: Testing & Quality (1 week)
**Goal:** Increase coverage to >80%, verify all features

#### Day 1-2: Unit Tests
- [ ] VIP3 filter counts tests
  - [ ] Test all filter combinations
  - [ ] Verify <50ms response time with 2.15M files
  - [ ] Test empty results (no matches)
- [ ] Saved searches tests
  - [ ] Save, load, delete, list
  - [ ] Verify use_count increments
  - [ ] Verify last_used timestamp updates
- [ ] Collections tests
  - [ ] Create, add files, reorder, delete
  - [ ] Verify position ordering
- [ ] Mixer tests
  - [ ] Set/get gain, pan, mute, solo
  - [ ] Verify VU metering accuracy
  - [ ] Test effect chain operations

#### Day 3: Integration Tests
- [ ] VIP3 → DAW workflow
  - [ ] Search → double-click → verify track added
  - [ ] Verify MIDI parsed correctly
  - [ ] Verify playback works
- [ ] Project workflow
  - [ ] Create → add tracks → save → close → reopen → verify state
  - [ ] Test save_as creates new project
- [ ] Automation workflow
  - [ ] Record automation → save → reload → verify playback matches

#### Day 4: Performance Testing
- [ ] Import performance
  - [ ] Verify 7,830 files/sec sustained
  - [ ] Test with 100K+ files
- [ ] Analysis performance
  - [ ] Verify 181-360 files/sec
  - [ ] Test all analysis types (BPM, key, drums, chords)
- [ ] Query performance
  - [ ] Simple queries: <10ms
  - [ ] Complex VIP3 queries: <100ms
  - [ ] Filter counts: <50ms

#### Day 5: Coverage & Verification
- [ ] Run `cargo tarpaulin --workspace --out Html`
  - [ ] Target: >80% coverage
  - [ ] Identify untested code paths
  - [ ] Add missing tests
- [ ] Run `verification/src/main.rs`
  - [ ] Verify all 15 tables exist
  - [ ] Verify all 60+ indexes exist
  - [ ] Run smoke tests for all subsystems
- [ ] Update documentation
  - [ ] Update `IMPLEMENTATION_STATUS.md` to 100%
  - [ ] Update `CLAUDE.md` with new features
  - [ ] Update `copilot-instructions.md` if patterns changed

#### Deliverables
- [x] Test coverage >80%
- [x] All performance targets met
- [x] Verification suite passes
- [x] Documentation updated

---

## Future Features (Post-100%)

### Lua Scripting Runtime
**Effort:** 1 week
**Priority:** Medium

- [ ] Initialize Lua VM (`mlua`)
- [ ] Expose Rust functions to Lua (file ops, DAW control, search)
- [ ] Script loading system
- [ ] Example scripts (auto-organize, batch operations)
- [ ] Security: sandbox Lua scripts (no file system access outside MIDI library)

### Meilisearch Integration
**Effort:** 3-4 days
**Priority:** Medium

- [ ] Initialize Meilisearch client
- [ ] Index MIDI files on import (file_path, tags, bpm, key, etc.)
- [ ] Replace PostgreSQL ILIKE with Meilisearch for full-text search
- [ ] Faceted search (filter by BPM, key, tags)
- [ ] Performance: <10ms searches

### Advanced Effect Chain
**Effort:** 2-3 weeks
**Priority:** Low

- [ ] Implement actual effects (not just stubs)
  - [ ] EQ (parametric, graphic)
  - [ ] Compressor (threshold, ratio, attack, release)
  - [ ] Reverb (room, hall, plate)
  - [ ] Delay (time, feedback, wet/dry)
  - [ ] Chorus, flanger, phaser
- [ ] VST/AU plugin support (very complex, optional)

### Advanced Mixer
**Effort:** 1 week
**Priority:** Low

- [ ] Send/return buses (aux sends)
- [ ] Sidechain routing
- [ ] Subgroups/busses
- [ ] Monitoring (pre-fader, post-fader, AFL, PFL)

---

## Success Metrics

### Phase 1 (VIP3)
- [ ] Filter counts update <50ms with 2.15M files
- [ ] Can save, load, delete searches
- [ ] Can create, manage collections with 1000+ files
- [ ] Favorites toggle works instantly

### Phase 2 (Mixer)
- [ ] All 30+ mixer commands implemented
- [ ] VU meters update 30-60 FPS
- [ ] Gain/pan/mute/solo work correctly during playback
- [ ] Master channel controls all tracks

### Phase 3 (Advanced DAW)
- [ ] Automation records and plays back smoothly
- [ ] Presets save/load entire mixer state
- [ ] Projects save/load full session state

### Phase 4 (Integration)
- [ ] Double-click in VIP3 loads file in DAW <500ms
- [ ] Drag-and-drop works smoothly

### Phase 5 (Quality)
- [ ] Test coverage >80%
- [ ] All performance targets met
- [ ] Zero critical bugs

---

## Weekly Schedule (4 Weeks to 100%)

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | VIP3 Completion | Filter counts, saved searches, collections, favorites |
| 2 | DAW Mixer | All 30+ commands, VU meters, master channel, effect stubs |
| 3 | DAW Advanced | Automation, presets, project management |
| 4 | Integration & Testing | VIP3↔DAW, testing, coverage >80%, docs update |

**Start Date:** 2025-12-16 (Monday)
**Target Completion:** 2026-01-13 (4 weeks)

---

## How to Use This Roadmap

1. **Follow in order** - Each phase builds on previous work
2. **Track progress** - Use checkboxes to mark completed items
3. **Update IMPLEMENTATION_STATUS.md** - Keep status document current
4. **Test as you go** - Don't defer testing to the end
5. **Document changes** - Update copilot-instructions.md if patterns change

---

## Getting Started Tomorrow

**Monday, 2025-12-16:**

1. Read `IMPLEMENTATION_STATUS.md` (5 min)
2. Start Phase 1, Day 1: VIP3 Filter Counts
3. Create `app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs`
4. Implement `get_vip3_filter_counts` command
5. Test with sample data
6. Update `VIP3Column.svelte` to display counts

**Expected time:** 6-8 hours
**Outcome:** Filter counts working, real-time updates

---

**This roadmap will take you from 75% → 100% complete in 4 weeks.** All missing features will be implemented, tested, and documented. Follow this plan and you'll have a production-ready MIDI Software Center matching every detail in the copilot instructions.
