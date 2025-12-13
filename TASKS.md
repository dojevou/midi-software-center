# MIDI Software Center - Parallel Implementation Tasks

## Overview
This document breaks down all remaining implementation tasks from `upgrades.txt` into independent work streams that can be handled by separate Claude Code terminals.

---

## TERMINAL 1: Database Migrations (VIP3 Filtering Tables)
**Status:** IN PROGRESS
**Priority:** CRITICAL (blocks other work)

### Tasks:
1. Create migration `019_vip3_filtering.sql` with:
   - `timbres` table with 21 pre-populated values
   - `styles` table with 24 pre-populated values
   - `articulations` table with 20 pre-populated values
   - `bpm_ranges` table with 8 ranges
   - `musical_keys` table with 24 keys (major/minor)
   - `midi_file_timbres` many-to-many join table
   - `midi_file_styles` many-to-many join table
   - `midi_file_articulations` many-to-many join table
   - `saved_searches` table
   - `recent_searches` table
   - `collections` table
   - `collection_files` join table
   - All required indexes
   - Full-text search triggers

### Files to create/modify:
- `database/migrations/019_vip3_filtering.sql`

---

## TERMINAL 2: Rust Backend - VIP3 Repository Layer
**Status:** NOT STARTED
**Priority:** HIGH

### Tasks:
1. Create Rust models for new tables in `shared/rust/src/db/models/`:
   - `timbre.rs`
   - `style.rs`
   - `articulation.rs`
   - `bpm_range.rs`
   - `musical_key.rs`
   - `collection.rs`
   - `saved_search.rs`

2. Create repository implementations in `shared/rust/src/db/repositories/`:
   - `timbre_repository.rs`
   - `style_repository.rs`
   - `articulation_repository.rs`
   - `collection_repository.rs`
   - `saved_search_repository.rs`

3. Update `shared/rust/src/db/models/mod.rs` and `shared/rust/src/db/repositories/mod.rs`

### Files to create:
- `shared/rust/src/db/models/timbre.rs`
- `shared/rust/src/db/models/style.rs`
- `shared/rust/src/db/models/articulation.rs`
- `shared/rust/src/db/models/bpm_range.rs`
- `shared/rust/src/db/models/musical_key.rs`
- `shared/rust/src/db/models/collection.rs`
- `shared/rust/src/db/models/saved_search.rs`
- `shared/rust/src/db/repositories/timbre_repository.rs`
- `shared/rust/src/db/repositories/style_repository.rs`
- `shared/rust/src/db/repositories/articulation_repository.rs`
- `shared/rust/src/db/repositories/collection_repository.rs`
- `shared/rust/src/db/repositories/saved_search_repository.rs`

---

## TERMINAL 3: Rust Backend - Tauri Commands
**Status:** NOT STARTED
**Priority:** HIGH

### Tasks:
1. Create Tauri commands in `pipeline/src-tauri/src/commands/`:
   - `timbres.rs` - CRUD for timbres, assign to files
   - `styles.rs` - CRUD for styles, assign to files
   - `articulations.rs` - CRUD for articulations, assign to files
   - `collections.rs` - CRUD for collections, add/remove files
   - `saved_searches.rs` - Save, load, execute saved searches
   - `vip3_browser.rs` - Combined filtering endpoint for VIP3 browser

2. Update `pipeline/src-tauri/src/commands/mod.rs` to export new commands

3. Register commands in `pipeline/src-tauri/src/lib.rs`

### Files to create/modify:
- `pipeline/src-tauri/src/commands/timbres.rs`
- `pipeline/src-tauri/src/commands/styles.rs`
- `pipeline/src-tauri/src/commands/articulations.rs`
- `pipeline/src-tauri/src/commands/collections.rs`
- `pipeline/src-tauri/src/commands/saved_searches.rs`
- `pipeline/src-tauri/src/commands/vip3_browser.rs`
- `pipeline/src-tauri/src/commands/mod.rs`
- `pipeline/src-tauri/src/lib.rs`

---

## TERMINAL 4: Frontend - Stores & API Layer
**Status:** NOT STARTED
**Priority:** HIGH

### Tasks:
1. Update/create TypeScript types in `app/src/lib/types.ts`:
   - Add Timbre, Style, Articulation, BpmRange, MusicalKey types
   - Add Collection, SavedSearch types
   - Add VIP3BrowserFilters interface
   - Add VIP3BrowserResults interface

2. Create API functions in `app/src/lib/api/`:
   - `timbres.ts`
   - `styles.ts`
   - `articulations.ts`
   - `collections.ts`
   - `savedSearches.ts`
   - `vip3Browser.ts`

3. Create/update stores in `app/src/lib/stores/`:
   - `browserStore.ts` - VIP3 browser state (filters, results, pagination)
   - `collectionStore.ts` - Collections management
   - `categoryStore.ts` - Timbres, styles, articulations cache

### Files to create:
- `app/src/lib/api/timbres.ts`
- `app/src/lib/api/styles.ts`
- `app/src/lib/api/articulations.ts`
- `app/src/lib/api/collections.ts`
- `app/src/lib/api/savedSearches.ts`
- `app/src/lib/api/vip3Browser.ts`
- `app/src/lib/stores/browserStore.ts`
- `app/src/lib/stores/collectionStore.ts`
- `app/src/lib/stores/categoryStore.ts`

---

## TERMINAL 5: Frontend - VIP3 Browser Components
**Status:** NOT STARTED
**Priority:** HIGH

### Tasks:
1. Create VIP3-style database browser in `app/src/lib/components/`:
   - `VIP3Browser.svelte` - Main container with 8 columns
   - `VIP3Column.svelte` - Single filterable column (reusable)
   - `VIP3FileList.svelte` - Results list with virtual scrolling
   - `VIP3SearchBar.svelte` - Full-text search with suggestions
   - `VIP3TagCloud.svelte` - Tag visualization
   - `VIP3PreviewPanel.svelte` - File preview with MIDI visualization

2. Create window wrapper `app/src/lib/windows/VIP3BrowserWindow.svelte`

### Files to create:
- `app/src/lib/components/VIP3Browser.svelte`
- `app/src/lib/components/VIP3Column.svelte`
- `app/src/lib/components/VIP3FileList.svelte`
- `app/src/lib/components/VIP3SearchBar.svelte`
- `app/src/lib/components/VIP3TagCloud.svelte`
- `app/src/lib/components/VIP3PreviewPanel.svelte`
- `app/src/lib/windows/VIP3BrowserWindow.svelte`

---

## TERMINAL 6: Frontend - Sequencer/Timeline Window
**Status:** NOT STARTED
**Priority:** MEDIUM

### Tasks:
1. Create Sequencer components in `app/src/lib/components/`:
   - `Sequencer.svelte` - Main timeline container
   - `SequencerTrack.svelte` - Single track lane
   - `SequencerClip.svelte` - Draggable MIDI clip on track
   - `SequencerTimeline.svelte` - Time ruler with markers
   - `SequencerTransport.svelte` - Play/stop/record controls
   - `SequencerMarker.svelte` - Marker indicator

2. Create sequencer store `app/src/lib/stores/sequencerStore.ts`

3. Create window wrapper `app/src/lib/windows/SequencerWindow.svelte`

### Files to create:
- `app/src/lib/components/Sequencer.svelte`
- `app/src/lib/components/SequencerTrack.svelte`
- `app/src/lib/components/SequencerClip.svelte`
- `app/src/lib/components/SequencerTimeline.svelte`
- `app/src/lib/components/SequencerTransport.svelte`
- `app/src/lib/components/SequencerMarker.svelte`
- `app/src/lib/stores/sequencerStore.ts`
- `app/src/lib/windows/SequencerWindow.svelte`

---

## TERMINAL 7: Frontend - Window Management System
**Status:** NOT STARTED
**Priority:** MEDIUM

### Tasks:
1. Implement Pro Tools-style window management:
   - Update `app/src/lib/stores/uiStore.ts` with window state management
   - Create `WindowManager.svelte` component for docking/floating
   - Implement window layouts (save/load from database)
   - Add tabbed window groups support
   - Add minimize to dock functionality

2. Update `app/src/App.svelte` to use WindowManager

3. Create layout selector UI

### Files to modify/create:
- `app/src/lib/stores/uiStore.ts`
- `app/src/lib/components/WindowManager.svelte`
- `app/src/lib/components/WindowTab.svelte`
- `app/src/lib/components/WindowDock.svelte`
- `app/src/App.svelte`

---

## TERMINAL 8: Frontend - Component Polish & Theming
**Status:** NOT STARTED
**Priority:** LOW

### Tasks:
1. Apply frogskin camo theme to all components
2. Ensure consistent styling across all windows
3. Add animations and transitions
4. Implement dark/light mode switching
5. Add accessibility attributes (ARIA)
6. Create component style guide

### Files to modify:
- All files in `app/src/lib/components/`
- All files in `app/src/lib/windows/`
- `app/src/app.css`
- `app/tailwind.config.js`

---

## Dependency Graph

```
                     ┌──────────────────┐
                     │  T1: Database    │
                     │  Migrations      │
                     └────────┬─────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
     ┌────────────────┐ ┌──────────────┐ ┌──────────────┐
     │ T2: Rust       │ │ T3: Tauri    │ │ T4: Frontend │
     │ Models/Repos   │ │ Commands     │ │ Stores/API   │
     └───────┬────────┘ └──────┬───────┘ └──────┬───────┘
             │                 │                │
             └────────────┬────┴────────────────┘
                          │
         ┌────────────────┼────────────────┐
         │                │                │
         ▼                ▼                ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ T5: VIP3        │ │ T6: Sequencer   │ │ T7: Window      │
│ Browser         │ │ Components      │ │ Management      │
└─────────────────┘ └─────────────────┘ └─────────────────┘
         │                │                │
         └────────────────┼────────────────┘
                          │
                          ▼
                ┌─────────────────┐
                │ T8: Polish &    │
                │ Theming         │
                └─────────────────┘
```

---

## Parallel Execution Strategy

### Phase 1 (Can start immediately - no dependencies):
- **T1**: Database Migrations
- **T8**: Component Polish (can work on existing components)

### Phase 2 (After T1 completes):
- **T2**: Rust Models/Repos
- **T3**: Tauri Commands
- **T4**: Frontend Stores/API

### Phase 3 (After T2, T3, T4 complete):
- **T5**: VIP3 Browser Components
- **T6**: Sequencer Components
- **T7**: Window Management

### Phase 4 (Final):
- Integration testing
- Final polish and bug fixes

---

## Commands to Start Each Terminal

```bash
# Terminal 1: Database Migrations
cd /home/dojevou/projects/midi-software-center
# Focus: database/migrations/

# Terminal 2: Rust Backend Models
cd /home/dojevou/projects/midi-software-center/shared/rust
# Focus: src/db/models/ and src/db/repositories/

# Terminal 3: Tauri Commands
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri
# Focus: src/commands/

# Terminal 4: Frontend Stores
cd /home/dojevou/projects/midi-software-center/app
# Focus: src/lib/stores/ and src/lib/api/

# Terminal 5: VIP3 Browser
cd /home/dojevou/projects/midi-software-center/app
# Focus: src/lib/components/VIP3*.svelte

# Terminal 6: Sequencer
cd /home/dojevou/projects/midi-software-center/app
# Focus: src/lib/components/Sequencer*.svelte

# Terminal 7: Window Manager
cd /home/dojevou/projects/midi-software-center/app
# Focus: src/lib/components/Window*.svelte and App.svelte

# Terminal 8: Polish
cd /home/dojevou/projects/midi-software-center/app
# Focus: All components, styling, animations
```

---

## Notes

1. **T1 is the critical path** - Most other tasks depend on the database schema being finalized
2. **T2, T3, T4 can run in parallel** once database migrations are complete
3. **T5, T6, T7 are independent** and can run in parallel once the backend is ready
4. **T8 is ongoing** and can start immediately with existing components

---

## File Counts

| Terminal | New Files | Modified Files | Estimated LOC |
|----------|-----------|----------------|---------------|
| T1       | 1         | 0              | ~400          |
| T2       | 12        | 2              | ~1,500        |
| T3       | 7         | 2              | ~1,200        |
| T4       | 9         | 1              | ~800          |
| T5       | 7         | 0              | ~2,000        |
| T6       | 8         | 0              | ~1,800        |
| T7       | 4         | 2              | ~1,000        |
| T8       | 0         | ~40            | ~500          |
| **Total**| **48**    | **47**         | **~9,200**    |

---

## Prompts for Each Terminal

### Terminal 1 Prompt:
```
Read TASKS.md and implement Terminal 1: Database Migrations.
Create database/migrations/019_vip3_filtering.sql with all VIP3 filtering tables (timbres, styles, articulations, bpm_ranges, musical_keys) and their join tables, plus saved_searches, collections tables.
Follow the schema from upgrades.txt lines 1039-1217.
```

### Terminal 2 Prompt:
```
Read TASKS.md and implement Terminal 2: Rust Backend Models/Repos.
Create Rust models and repositories for timbres, styles, articulations, bpm_ranges, musical_keys, collections, and saved_searches.
Follow existing patterns in shared/rust/src/db/models/ and shared/rust/src/db/repositories/.
```

### Terminal 3 Prompt:
```
Read TASKS.md and implement Terminal 3: Tauri Commands.
Create Tauri commands for timbres, styles, articulations, collections, saved_searches, and vip3_browser.
Follow existing patterns in pipeline/src-tauri/src/commands/.
```

### Terminal 4 Prompt:
```
Read TASKS.md and implement Terminal 4: Frontend Stores & API.
Add VIP3 types to types.ts, create API functions in app/src/lib/api/, and create browserStore, collectionStore, categoryStore.
```

### Terminal 5 Prompt:
```
Read TASKS.md and implement Terminal 5: VIP3 Browser Components.
Create VIP3Browser.svelte with 8 filterable columns following the Akai VIP3 design from upgrades.txt.
```

### Terminal 6 Prompt:
```
Read TASKS.md and implement Terminal 6: Sequencer Components.
Create Sequencer.svelte with tracks, clips, timeline, and transport following the DAW sequencer design from upgrades.txt.
```

### Terminal 7 Prompt:
```
Read TASKS.md and implement Terminal 7: Window Management.
Create Pro Tools-style floating/docking window system with tabs and layouts.
```

### Terminal 8 Prompt:
```
Read TASKS.md and implement Terminal 8: Component Polish.
Apply frogskin camo theme consistently to all components, add animations, ensure accessibility.
```
