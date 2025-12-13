# PROJECT STRUCTURE

**The City Map for MIDI Software Center**

**Date:** 2025-10-24
**Purpose:** Complete directory structure and file placement rules
**Audience:** Developers, AI assistants, code reviewers

---

## 📋 TABLE OF CONTENTS

1. [Overview](#overview)
2. [Complete Directory Tree](#complete-directory-tree)
3. [Component Breakdown](#component-breakdown)
4. [Archetype to Directory Mapping](#archetype-to-directory-mapping)
5. [File Placement Rules](#file-placement-rules)
6. [Examples](#examples)

---

## 🎯 OVERVIEW

This project follows a **component-based architecture** with three main applications:

```
┌──────────────────────────────────────────────────────────┐
│                  MIDI SOFTWARE CENTER                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  📦 DATABASE      Centralized PostgreSQL + Meilisearch  │
│  🔧 SHARED        Core libraries (Rust + TypeScript)    │
│  ⚙️  PIPELINE      Batch analysis GUI (Tauri + Svelte)   │
│  🎹 DAW           Sequencer/player GUI (Tauri + Svelte) │
│  📜 SCRIPTS       Automation and utilities              │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**Key Principles:**
1. **Shared-first:** Common code goes in `shared/` (both Rust and TypeScript)
2. **Component isolation:** Each app (pipeline, daw) is self-contained
3. **Database centralization:** Single PostgreSQL instance, shared schema
4. **Clear separation:** Three Archetypes determine file location

---

## 🌳 COMPLETE DIRECTORY TREE

```
midi-software-center/
│
├── 📁 database/                    # Centralized database component
│   ├── migrations/                 # SQL migration files (numbered)
│   │   ├── 001_initial_schema.sql
│   │   ├── 002_add_search_index.sql
│   │   ├── 003_add_collections.sql
│   │   ├── 004_add_tags.sql
│   │   ├── 005_add_custom_fields.sql
│   │   └── 006_add_playback_state.sql
│   ├── seed/                       # Test/development data
│   │   └── sample_data.sql
│   └── README.md                   # Database setup instructions
│
├── 📁 shared/                      # Shared libraries
│   ├── rust/                       # Rust shared library
│   │   ├── Cargo.toml              # Rust library config
│   │   ├── src/
│   │   │   ├── lib.rs              # Library root (exports all modules)
│   │   │   │
│   │   │   ├── core/               # ⭐ TRUSTY MODULES ONLY (pure logic)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── midi/           # MIDI parsing and types
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parser.rs   # Parse MIDI files
│   │   │   │   │   ├── types.rs    # MIDI data structures
│   │   │   │   │   ├── events.rs   # MIDI event types
│   │   │   │   │   └── writer.rs   # Write MIDI files
│   │   │   │   └── analysis/       # Analysis algorithms
│   │   │   │       ├── mod.rs
│   │   │   │       ├── bpm_detector.rs
│   │   │   │       ├── key_detector.rs
│   │   │   │       ├── chord_detector.rs
│   │   │   │       └── pattern_analyzer.rs
│   │   │   │
│   │   │   ├── db/                 # Database layer (GROWN-UP SCRIPTS)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── models/         # Database models
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── file.rs
│   │   │   │   │   ├── collection.rs
│   │   │   │   │   ├── tag.rs
│   │   │   │   │   └── playback_state.rs
│   │   │   │   ├── repositories/   # Repository pattern (database access)
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── file_repository.rs
│   │   │   │   │   ├── collection_repository.rs
│   │   │   │   │   ├── tag_repository.rs
│   │   │   │   │   └── search_repository.rs
│   │   │   │   ├── connection.rs   # Database connection management
│   │   │   │   └── error.rs        # Database error types
│   │   │   │
│   │   │   ├── search/             # Meilisearch integration (GROWN-UP)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── client.rs       # Meilisearch client
│   │   │   │   ├── indexer.rs      # Index documents
│   │   │   │   └── query.rs        # Search queries
│   │   │   │
│   │   │   └── utils/              # Utilities (TRUSTY if pure, else GROWN-UP)
│   │   │       ├── mod.rs
│   │   │       ├── validation.rs   # Trusty Module (pure validation)
│   │   │       └── logging.rs      # Grown-up (has I/O side effects)
│   │   │
│   │   └── tests/                  # Integration tests
│   │       ├── integration/
│   │       │   ├── db_test.rs
│   │       │   └── search_test.rs
│   │       └── fixtures/           # Test data
│   │           └── test.mid
│   │
│   └── typescript/                 # TypeScript shared library
│       ├── package.json
│       ├── tsconfig.json
│       ├── src/
│       │   ├── index.ts            # Exports all modules
│       │   ├── types/              # Shared TypeScript types
│       │   │   ├── file.ts
│       │   │   ├── collection.ts
│       │   │   └── midi.ts
│       │   ├── api/                # API client utilities
│       │   │   └── client.ts
│       │   └── utils/              # Shared utilities
│       │       ├── formatters.ts
│       │       └── validators.ts
│       └── tests/
│           └── types.test.ts
│
├── 📁 pipeline/                    # Pipeline application (batch analysis)
│   ├── package.json                # Frontend dependencies
│   ├── vite.config.ts              # Vite build config
│   ├── tsconfig.json               # TypeScript config
│   │
│   ├── src/                        # Svelte frontend
│   │   ├── App.svelte              # Root component
│   │   ├── main.ts                 # Frontend entry point
│   │   │
│   │   ├── lib/                    # Library code
│   │   │   ├── components/         # Svelte components
│   │   │   │   ├── library/
│   │   │   │   │   ├── FileList.svelte
│   │   │   │   │   ├── FileCard.svelte
│   │   │   │   │   └── SearchBar.svelte
│   │   │   │   ├── import/
│   │   │   │   │   ├── ImportDialog.svelte
│   │   │   │   │   └── ProgressBar.svelte
│   │   │   │   └── common/
│   │   │   │       ├── Button.svelte
│   │   │   │       └── Modal.svelte
│   │   │   │
│   │   │   ├── stores/             # Svelte stores (state management)
│   │   │   │   ├── libraryStore.ts
│   │   │   │   ├── importStore.ts
│   │   │   │   └── uiStore.ts
│   │   │   │
│   │   │   └── utils/              # Frontend utilities
│   │   │       ├── formatters.ts
│   │   │       └── constants.ts
│   │   │
│   │   └── assets/                 # Static assets
│   │       ├── styles.css
│   │       └── logo.png
│   │
│   ├── src-tauri/                  # Rust backend (Tauri)
│   │   ├── Cargo.toml              # Backend dependencies
│   │   ├── tauri.conf.json         # Tauri configuration
│   │   ├── icons/                  # App icons
│   │   │
│   │   └── src/
│   │       ├── main.rs             # ⭐ TASK-O-MATIC (app entry point)
│   │       ├── lib.rs              # Library exports (if needed)
│   │       │
│   │       ├── commands/           # ⭐ GROWN-UP SCRIPTS (Tauri commands)
│   │       │   ├── mod.rs
│   │       │   ├── file_commands.rs
│   │       │   ├── import_commands.rs
│   │       │   ├── search_commands.rs
│   │       │   └── analysis_commands.rs
│   │       │
│   │       ├── services/           # ⭐ GROWN-UP SCRIPTS (business logic)
│   │       │   ├── mod.rs
│   │       │   ├── import_service.rs
│   │       │   └── batch_processor.rs
│   │       │
│   │       └── state.rs            # App state management
│   │
│   └── README.md                   # Pipeline app documentation
│
├── 📁 daw/                         # DAW application (sequencer/player)
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   │
│   ├── src/                        # Svelte frontend
│   │   ├── App.svelte
│   │   ├── main.ts
│   │   │
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   │   ├── sequencer/
│   │   │   │   │   ├── Timeline.svelte
│   │   │   │   │   ├── TrackList.svelte
│   │   │   │   │   ├── PianoRoll.svelte
│   │   │   │   │   └── MixerPanel.svelte
│   │   │   │   ├── library/        # Browse library in DAW
│   │   │   │   │   ├── FileBrowser.svelte
│   │   │   │   │   └── QuickSearch.svelte
│   │   │   │   └── common/
│   │   │   │       ├── Button.svelte
│   │   │   │       └── Slider.svelte
│   │   │   │
│   │   │   ├── stores/
│   │   │   │   ├── sequencerStore.ts
│   │   │   │   ├── playbackStore.ts
│   │   │   │   ├── libraryStore.ts
│   │   │   │   └── uiStore.ts
│   │   │   │
│   │   │   └── utils/
│   │   │       ├── midiUtils.ts
│   │   │       └── timeUtils.ts
│   │   │
│   │   └── assets/
│   │       └── styles.css
│   │
│   ├── src-tauri/                  # Rust backend
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── icons/
│   │   │
│   │   └── src/
│   │       ├── main.rs             # ⭐ TASK-O-MATIC (app entry point)
│   │       ├── lib.rs
│   │       │
│   │       ├── commands/           # ⭐ GROWN-UP SCRIPTS
│   │       │   ├── mod.rs
│   │       │   ├── file_commands.rs
│   │       │   ├── playback_commands.rs
│   │       │   ├── sequencer_commands.rs
│   │       │   └── export_commands.rs
│   │       │
│   │       ├── audio/              # ⭐ GROWN-UP SCRIPTS (audio I/O)
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs       # Audio engine
│   │       │   ├── midi_output.rs  # MIDI device output
│   │       │   └── track.rs        # Track management
│   │       │
│   │       ├── sequencer/          # ⭐ GROWN-UP SCRIPTS
│   │       │   ├── mod.rs
│   │       │   ├── timeline.rs
│   │       │   ├── clip.rs
│   │       │   └── transport.rs
│   │       │
│   │       └── state.rs
│   │
│   └── README.md
│
├── 📁 scripts/                     # Automation scripts
│   ├── setup/                      # Setup automation
│   │   ├── setup.sh                # Main setup script
│   │   └── install-deps.sh         # Install dependencies
│   │
│   ├── launch/                     # Application launchers
│   │   ├── launch-all.sh           # Launch all apps
│   │   ├── launch-pipeline.sh      # Launch pipeline only
│   │   ├── launch-daw.sh           # Launch DAW only
│   │   ├── status.sh               # Check app status
│   │   └── stop-all.sh             # Stop all apps
│   │
│   ├── verify/                     # Verification scripts
│   │   ├── integration_test.sh     # Run integration tests
│   │   └── quick_check.sh          # Quick sanity check
│   │
│   ├── import-tool/                # ⭐ TASK-O-MATIC (CLI Rust binary)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs             # CLI import tool
│   │
│   └── test-all.sh                 # Run all tests
│
├── 📁 docs/                        # Project documentation
│   ├── ARCHITECTURE-REFERENCE.md   # This document's companion
│   ├── PROJECT-STRUCTURE.md        # This file
│   ├── DEVELOPMENT-WORKFLOW.md     # Development process
│   └── api/                        # API documentation
│       └── README.md
│
├── 📁 .github/                     # GitHub configuration
│   └── workflows/                  # CI/CD workflows
│       ├── test.yml
│       └── build.yml
│
├── 📁 .cursor/                     # AI assistant rules
│   └── rules/
│       ├── project-rules.mdc
│       ├── database-rules.mdc
│       ├── shared-rules.mdc
│       ├── workspace-rules.mdc
│       ├── rust-rules.mdc
│       └── svelte-rules.mdc
│
├── 📄 Cargo.toml                   # Rust workspace config
├── 📄 package.json                 # Root package config
├── 📄 Makefile                     # Common tasks
├── 📄 docker-compose.yml           # Database services
├── 📄 .env.example                 # Environment variables template
├── 📄 .gitignore
├── 📄 README.md                    # Project overview
└── 📄 CLAUDE.md                    # AI assistant guidance

```

---

## 🧩 COMPONENT BREAKDOWN

### 1. Database Component

**Purpose:** Centralized data storage and search

**Location:** `database/`

**Contents:**
- SQL migrations (PostgreSQL schema)
- Seed data for testing
- Database documentation

**Technology:**
- PostgreSQL 16 with pgvector extension
- Meilisearch 1.5 for full-text search

**Key Files:**
```
database/
├── migrations/001_initial_schema.sql    # Core tables (files, collections)
├── migrations/002_add_search_index.sql  # Meilisearch integration
├── migrations/003_add_collections.sql   # Collection support
├── migrations/004_add_tags.sql          # Tagging system
├── migrations/005_add_custom_fields.sql # Extensibility
└── migrations/006_add_playback_state.sql # DAW state persistence
```

**Access:** Both Pipeline and DAW connect to same database instance

---

### 2. Shared Component

**Purpose:** Code shared between Pipeline and DAW

**Location:** `shared/`

**Subcomponents:**
- `shared/rust/` - Rust libraries
- `shared/typescript/` - TypeScript types and utilities

#### Shared Rust Library

**Critical Rule:** `shared/rust/src/core/` contains ONLY Trusty Modules

**Structure:**
```
shared/rust/src/
├── core/                # ⭐ TRUSTY MODULES ONLY
│   ├── midi/            # Pure MIDI logic
│   └── analysis/        # Pure analysis algorithms
│
├── db/                  # GROWN-UP SCRIPTS (database I/O)
├── search/              # GROWN-UP SCRIPTS (Meilisearch I/O)
└── utils/               # Mixed (pure utils = Trusty, I/O utils = Grown-up)
```

**Exports:**
```rust
// shared/rust/src/lib.rs

pub mod core;     // Trusty Modules (pure logic)
pub mod db;       // Database layer
pub mod search;   // Search integration
pub mod utils;    // Utilities
```

**Usage:**
```rust
// In pipeline/src-tauri/Cargo.toml
[dependencies]
midi-library = { path = "../../shared/rust" }

// In pipeline code
use midi_library::core::midi::parse_midi;
use midi_library::db::repositories::FileRepository;
```

#### Shared TypeScript Library

**Purpose:** Type definitions and utilities for frontends

**Structure:**
```
shared/typescript/src/
├── types/           # TypeScript interfaces (match Rust models)
├── api/             # API client helpers
└── utils/           # Shared utilities
```

**Usage:**
```typescript
// In pipeline/package.json
"dependencies": {
  "midi-library-types": "file:../../shared/typescript"
}

// In pipeline code
import { File, Collection } from 'midi-library-types';
```

---

### 3. Pipeline Component

**Purpose:** Batch MIDI file analysis and library management

**Location:** `pipeline/`

**Architecture:** Tauri 2 (Rust backend + Svelte frontend)

**Backend (`pipeline/src-tauri/`):**
```
src/
├── main.rs              # TASK-O-MATIC (app entry)
├── commands/            # GROWN-UP SCRIPTS (Tauri commands)
└── services/            # GROWN-UP SCRIPTS (business logic)
```

**Frontend (`pipeline/src/`):**
```
src/
├── App.svelte           # Root component
├── lib/
│   ├── components/      # Svelte UI components
│   ├── stores/          # State management
│   └── utils/           # Frontend utilities
```

**Key Features:**
- Import MIDI files in batches
- Analyze BPM, key, chords
- Full-text search
- Collection management

---

### 4. DAW Component

**Purpose:** Interactive MIDI sequencer and playback

**Location:** `daw/`

**Architecture:** Tauri 2 (Rust backend + Svelte frontend)

**Backend (`daw/src-tauri/`):**
```
src/
├── main.rs              # TASK-O-MATIC (app entry)
├── commands/            # GROWN-UP SCRIPTS
├── audio/               # GROWN-UP SCRIPTS (audio engine)
└── sequencer/           # GROWN-UP SCRIPTS (timeline logic)
```

**Frontend (`daw/src/`):**
```
src/
├── App.svelte
├── lib/
│   ├── components/
│   │   ├── sequencer/   # Timeline, piano roll, tracks
│   │   └── library/     # Browse library files
│   └── stores/
```

**Key Features:**
- Real-time MIDI playback
- Multi-track sequencing
- Piano roll editor
- Mixer panel
- Library integration

**Note on Duplicate MIDI Modules:**
DAW has its own MIDI parsing (`daw/src-tauri/src/midi/`) optimized for real-time playback, separate from shared library's batch parsing. This is intentional.

---

### 5. Scripts Component

**Purpose:** Automation, setup, and utilities

**Location:** `scripts/`

**Structure:**
```
scripts/
├── setup/              # One-time setup
├── launch/             # App launchers
├── verify/             # Testing and validation
└── import-tool/        # TASK-O-MATIC (CLI binary)
```

**Key Scripts:**
- `setup/setup.sh` - Initial project setup (install deps, create DB, build)
- `launch/launch-all.sh` - Start database + pipeline + DAW
- `verify/quick_check.sh` - Sanity check before commits
- `import-tool/` - Standalone CLI for importing files

---

## 🗺️ ARCHETYPE TO DIRECTORY MAPPING

### Task-O-Matics (Complete standalone tasks)

**Locations:**
- `pipeline/src-tauri/src/main.rs`
- `daw/src-tauri/src/main.rs`
- `scripts/import-tool/src/main.rs`
- Any `bin/` directory

**Characteristics:**
- Has `main()` function
- Never imported by other code
- Complete workflow from start to finish

**Example:**
```rust
// daw/src-tauri/src/main.rs

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database connection
            // Initialize audio engine
            // Register Tauri commands
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::playback::play,
            commands::sequencer::add_track,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### Grown-up Scripts (Orchestration + I/O + reusability)

**Locations:**
- `{component}/src-tauri/src/commands/` - Tauri command handlers
- `{component}/src-tauri/src/services/` - Business logic services
- `shared/rust/src/db/` - Database layer
- `shared/rust/src/search/` - Search integration
- Any code that does I/O but needs to be reusable

**Characteristics:**
- Can be imported by other code
- Does I/O (files, database, network, audio devices)
- Has side effects
- Uses entry point + implementation pattern

**Example:**
```rust
// pipeline/src-tauri/src/commands/import_commands.rs

// Entry point
#[tauri::command]
pub async fn import_files(
    paths: Vec<String>,
    state: tauri::State<'_, AppState>
) -> Result<ImportResult, String> {
    import_files_impl(&state.db_pool, &paths)
        .await
        .map_err(|e| e.to_string())
}

// Implementation
pub async fn import_files_impl(
    pool: &PgPool,
    paths: &[String]
) -> Result<ImportResult, ImportError> {
    // File I/O + database I/O
}
```

---

### Trusty Modules (Pure, tested, reusable logic)

**Locations:**
- `shared/rust/src/core/` - **PRIMARY LOCATION** (CRITICAL)
- `{component}/src-tauri/src/core/` - Component-specific pure logic (rare)

**CRITICAL RULE:** Everything in `core/` directories MUST be a Trusty Module

**Characteristics:**
- Pure functions (no I/O, no side effects)
- 80%+ test coverage required
- Comprehensive documentation
- Could be extracted to separate crate

**Example:**
```rust
// shared/rust/src/core/analysis/key_detector.rs

/// Detect musical key from MIDI note events
pub fn detect_key(notes: &[Note]) -> Result<MusicalKey, KeyError> {
    // Pure algorithm - no I/O
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_c_major() { /* ... */ }

    #[test]
    fn test_a_minor() { /* ... */ }

    // 80%+ coverage
}
```

---

## 📝 FILE PLACEMENT RULES

### Rule 1: Shared vs Component-Specific

**Question:** Should this code be in `shared/` or in a component (`pipeline/`, `daw/`)?

**Decision Tree:**
```
Will both Pipeline AND DAW use this code?
├─ YES → shared/rust/ or shared/typescript/
└─ NO  → component-specific (pipeline/ or daw/)
```

**Examples:**
- MIDI parser → `shared/rust/src/core/midi/parser.rs` (both use it)
- File repository → `shared/rust/src/db/repositories/file_repository.rs` (both use it)
- Sequencer logic → `daw/src-tauri/src/sequencer/` (only DAW uses it)
- Batch processor → `pipeline/src-tauri/src/services/batch_processor.rs` (only Pipeline uses it)

---

### Rule 2: core/ vs Non-core

**Question:** Should this code be in a `core/` directory?

**Decision Tree:**
```
Does this code do ANY I/O or have ANY side effects?
├─ YES → NOT in core/ (use services/, commands/, db/, etc.)
└─ NO  → core/ (pure logic only)
```

**CRITICAL:** Never put I/O code in `core/` directories

**Examples:**
- BPM detection algorithm → `shared/rust/src/core/analysis/bpm_detector.rs` (pure)
- File reading + parsing → `shared/rust/src/io/file_loader.rs` NOT core (has I/O)
- Database query → `shared/rust/src/db/repositories/` NOT core (has I/O)

---

### Rule 3: commands/ vs services/

**Question:** Where do Tauri backend files go?

**Decision Tree:**
```
Is this a #[tauri::command] function?
├─ YES → commands/ (Tauri command handlers)
└─ NO  → Is this reusable business logic?
          ├─ YES → services/ (orchestration logic)
          └─ NO  → Other appropriate directory (audio/, sequencer/, etc.)
```

**Examples:**
```
commands/
├── file_commands.rs         # #[tauri::command] functions
└── import_commands.rs       # #[tauri::command] functions

services/
├── import_service.rs        # Reusable import logic
└── batch_processor.rs       # Batch processing orchestration

audio/
└── engine.rs                # Audio engine (not a command, not pure service)
```

---

### Rule 4: Frontend Components

**Question:** Where do Svelte components go?

**Categorization:**
```
src/lib/components/
├── {feature}/               # Feature-specific components
│   └── *.svelte
└── common/                  # Shared UI components
    └── *.svelte
```

**Examples:**
```
pipeline/src/lib/components/
├── library/
│   ├── FileList.svelte      # Library feature
│   └── SearchBar.svelte     # Library feature
├── import/
│   └── ImportDialog.svelte  # Import feature
└── common/
    ├── Button.svelte        # Shared component
    └── Modal.svelte         # Shared component
```

---

### Rule 5: Tests

**Question:** Where do tests go?

**Decision Tree:**
```
What type of test is this?
├─ Unit test for specific module → #[cfg(test)] in same file
├─ Integration test (database, services) → tests/integration/
└─ E2E test (full workflow) → tests/e2e/
```

**Examples:**
```rust
// Unit test - in same file
// shared/rust/src/core/midi/parser.rs

pub fn parse_midi(bytes: &[u8]) -> Result<MidiFile> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_midi() {
        // Test here
    }
}
```

```rust
// Integration test - separate file
// shared/rust/tests/integration/db_test.rs

use midi_library::db::repositories::FileRepository;

#[sqlx::test]
async fn test_file_crud(pool: PgPool) -> sqlx::Result<()> {
    // Test with real database
}
```

---

## 📚 EXAMPLES

### Example 1: Adding MIDI Export Feature

**Requirement:** Export edited MIDI files from DAW

**Analysis:**
- Writing MIDI files = I/O operation
- Used only by DAW (not Pipeline)
- Needs to be testable

**File Placement:**
```
daw/src-tauri/src/
├── commands/
│   └── export_commands.rs        # GROWN-UP SCRIPT
│       ├── #[tauri::command] export_midi(...)  # Entry point
│       └── export_midi_impl(...)               # Implementation
```

**Use Shared Library:**
```rust
// daw/src-tauri/src/commands/export_commands.rs

use midi_library::core::midi::write_midi;  // Use shared Trusty Module

#[tauri::command]
pub async fn export_midi(
    file_data: MidiData,
    output_path: String
) -> Result<(), String> {
    export_midi_impl(&file_data, &output_path)
        .await
        .map_err(|e| e.to_string())
}

pub async fn export_midi_impl(
    file_data: &MidiData,
    output_path: &str
) -> Result<(), ExportError> {
    // 1. Use shared Trusty Module to create MIDI bytes
    let bytes = midi_library::core::midi::write_midi(file_data)?;

    // 2. Write to file (I/O - why this is Grown-up Script)
    tokio::fs::write(output_path, bytes).await?;

    Ok(())
}
```

**Why This Placement?**
- Entry + implementation = Grown-up Script pattern
- DAW-specific = goes in `daw/` not `shared/`
- Command handler = goes in `commands/`
- Uses shared `write_midi` Trusty Module from `shared/rust/src/core/midi/`

---

### Example 2: Adding Chord Progression Analysis

**Requirement:** Analyze chord progressions in MIDI files

**Analysis:**
- Pure algorithm (input notes → output chords)
- No I/O needed
- Used by both Pipeline (batch) and DAW (real-time)
- Needs high test coverage

**File Placement:**
```
shared/rust/src/core/analysis/
└── chord_progression.rs    # TRUSTY MODULE
```

**Implementation:**
```rust
// shared/rust/src/core/analysis/chord_progression.rs

use crate::core::midi::{Note, Chord};

/// Analyze chord progressions from MIDI notes
///
/// Takes a sequence of MIDI notes and identifies chord progressions
/// using harmonic analysis algorithms.
///
/// # Arguments
/// * `notes` - Slice of MIDI notes to analyze
/// * `time_window_ms` - Time window for grouping notes into chords
///
/// # Returns
/// * `Ok(Vec<Chord>)` - Detected chord progression
/// * `Err(ChordError)` - If analysis fails
///
/// # Examples
/// ```
/// use midi_library::core::analysis::analyze_chord_progression;
///
/// let notes = vec![/* MIDI notes */];
/// let chords = analyze_chord_progression(&notes, 500)?;
/// assert_eq!(chords[0].name, "Cmaj");
/// ```
pub fn analyze_chord_progression(
    notes: &[Note],
    time_window_ms: u32
) -> Result<Vec<Chord>, ChordError> {
    // Pure algorithm - no I/O
    // Deterministic - same notes always produce same chords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_chord() {
        let notes = vec![
            Note { pitch: 60, .. },  // C
            Note { pitch: 64, .. },  // E
            Note { pitch: 67, .. },  // G
        ];
        let chords = analyze_chord_progression(&notes, 500).unwrap();
        assert_eq!(chords[0].name, "Cmaj");
    }

    #[test]
    fn test_chord_progression() { /* ... */ }

    #[test]
    fn test_invalid_notes() { /* ... */ }

    // 80%+ coverage required
}
```

**Why This Placement?**
- Pure function = Trusty Module
- Shared between apps = `shared/rust/`
- Analysis algorithm = `core/analysis/`

**Usage:**
```rust
// In pipeline/src-tauri/src/services/batch_processor.rs
use midi_library::core::analysis::analyze_chord_progression;

let chords = analyze_chord_progression(&midi.notes, 500)?;

// In daw/src-tauri/src/commands/analysis_commands.rs
use midi_library::core::analysis::analyze_chord_progression;

let chords = analyze_chord_progression(&current_track.notes, 500)?;
```

---

### Example 3: Adding Import Progress Notifications

**Requirement:** Show real-time import progress in Pipeline

**Analysis:**
- Backend sends progress updates to frontend
- Pipeline-specific (DAW doesn't need this)
- Has side effects (emits events)
- Orchestrates file I/O + database I/O

**File Placement:**
```
pipeline/src-tauri/src/
├── commands/
│   └── import_commands.rs       # Entry points
└── services/
    └── import_service.rs        # GROWN-UP SCRIPT with events
```

**Implementation:**
```rust
// pipeline/src-tauri/src/services/import_service.rs

use tauri::Window;
use midi_library::core::midi::parse_midi;
use midi_library::db::repositories::FileRepository;

/// Import files with progress notifications
pub async fn import_with_progress(
    pool: &PgPool,
    paths: &[String],
    window: &Window
) -> Result<ImportResult, ImportError> {
    let total = paths.len();

    for (index, path) in paths.iter().enumerate() {
        // 1. Emit progress (side effect)
        window.emit("import-progress", ImportProgress {
            current: index + 1,
            total,
            file_name: path.to_string(),
        }).ok();

        // 2. Read file (I/O)
        let bytes = tokio::fs::read(path).await?;

        // 3. Parse (uses Trusty Module)
        let midi = parse_midi(&bytes)?;

        // 4. Save to database (I/O)
        let repo = FileRepository::new(pool);
        repo.insert(&midi).await?;
    }

    Ok(ImportResult { imported: total })
}
```

**Why This Placement?**
- Has I/O and side effects = Grown-up Script
- Pipeline-specific = `pipeline/` not `shared/`
- Reusable orchestration = `services/` not `commands/`
- Uses shared Trusty Module (`parse_midi`)

---

## ✅ QUICK REFERENCE

### "Where does this file go?" Checklist

1. **Is it used by both Pipeline AND DAW?**
   - YES → `shared/`
   - NO → Component directory (`pipeline/` or `daw/`)

2. **Is it pure logic with no I/O?**
   - YES → `core/` subdirectory (Trusty Module)
   - NO → Continue to #3

3. **What type of code is it?**
   - Tauri command → `commands/`
   - Business logic service → `services/`
   - Database access → `db/repositories/`
   - Audio/MIDI hardware → `audio/` or component-specific directory
   - Frontend component → `lib/components/{feature}/`
   - State management → `lib/stores/`

4. **Does it have tests?**
   - Unit tests → `#[cfg(test)]` in same file
   - Integration tests → `tests/integration/`
   - E2E tests → `tests/e2e/`

---

### Directory Quick Reference

| Code Type | Example | Location |
|-----------|---------|----------|
| App entry point | `main.rs` | `{component}/src-tauri/src/main.rs` |
| Tauri command | `search_files` | `{component}/src-tauri/src/commands/*.rs` |
| Business logic | Import service | `{component}/src-tauri/src/services/*.rs` |
| Pure MIDI logic | Parser, analysis | `shared/rust/src/core/midi/` or `core/analysis/` |
| Database access | Repository | `shared/rust/src/db/repositories/*.rs` |
| Database model | File struct | `shared/rust/src/db/models/*.rs` |
| Frontend component | `FileList.svelte` | `{component}/src/lib/components/{feature}/*.svelte` |
| State management | `libraryStore.ts` | `{component}/src/lib/stores/*.ts` |
| Shared TypeScript type | `File` interface | `shared/typescript/src/types/*.ts` |
| CLI tool | Import tool | `scripts/import-tool/src/main.rs` |
| Shell script | Setup script | `scripts/setup/*.sh` or `scripts/launch/*.sh` |
| SQL migration | Schema change | `database/migrations/*.sql` |

---

## 🎯 CONCLUSION

**This structure provides:**
1. **Clear boundaries** - Components are isolated
2. **Code reuse** - Shared libraries prevent duplication
3. **Quality enforcement** - `core/` = pure, tested code
4. **Scalability** - Easy to add features or components

**Remember:**
- Shared code → `shared/`
- Component code → `pipeline/` or `daw/`
- Pure logic → `core/` subdirectories
- I/O operations → `commands/`, `services/`, `db/`, etc.

**For more information, see:**
- [ARCHITECTURE-REFERENCE.md](./ARCHITECTURE-REFERENCE.md) - Three Archetypes pattern
- [DEVELOPMENT-WORKFLOW.md](./DEVELOPMENT-WORKFLOW.md) - How to build features
- [CLAUDE.md](./CLAUDE.md) - Overall project guidance
