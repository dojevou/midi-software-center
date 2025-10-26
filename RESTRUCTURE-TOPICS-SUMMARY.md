# RESTRUCTURE.TXT - COMPREHENSIVE TOPICS SUMMARY

**Source:** restructure.txt (16,371 lines, 469KB conversation export)
**Date Created:** 2025-10-24
**Purpose:** Organized summary of all major topics discussed in the restructure conversation

---

## 📋 TABLE OF CONTENTS

1. [Technology Stack Overview](#1-technology-stack-overview)
2. [Project Architecture](#2-project-architecture)
3. [Three Archetypes Pattern](#3-three-archetypes-pattern)
4. [Component Architecture](#4-component-architecture)
5. [Development Environment Setup](#5-development-environment-setup)
6. [Layer-Specific Architecture](#6-layer-specific-architecture)
7. [Development Workflows](#7-development-workflows)
8. [Code Quality & Testing](#8-code-quality--testing)
9. [Build System & Deployment](#9-build-system--deployment)
10. [AI Assistant Integration](#10-ai-assistant-integration)
11. [Documentation Structure](#11-documentation-structure)
12. [File Organization](#12-file-organization)

---

## 1. TECHNOLOGY STACK OVERVIEW

### Final Summary

The MIDI Library System uses a modern, performance-oriented technology stack organized in three layers:

**DATABASE LAYER:**
- **PostgreSQL 16 + pgvector** - Primary relational database with vector embeddings for semantic search
- **Meilisearch 1.5** - Full-text search engine for fast queries
- **SQL** - Schema definitions, migrations, indexes
- **Docker Compose 3.8** - Containerized database services

**BACKEND LAYER (RUST):**
- **Rust 1.70+** - Primary backend language for performance and safety
- **Tauri 2.7** - Desktop application framework (replaces Electron)
- **tokio 1.35** - Async runtime for concurrent processing
- **sqlx 0.7** - Type-safe SQL with compile-time checking
- **MIDI Libraries:**
  - midly 0.5 - MIDI parsing
  - midir - MIDI hardware I/O
  - rimd - Additional MIDI utilities
- **rayon** - Parallel processing for batch operations
- **memmap2** - Memory-mapped file I/O for large files

**FRONTEND LAYER (SVELTE + TYPESCRIPT):**
- **Svelte 4.2** - Reactive UI framework (more lightweight than React)
- **TypeScript 5.3** - Type safety for complex state management
- **Vite 5.0** - Build tool and dev server (fast HMR)
- **Tone.js 14.7** - Web Audio API wrapper for DAW features
- **HTML5 + CSS3** - Semantic markup and modern styling

**BUILD & OPERATIONS:**
- **Cargo** - Rust package manager and build tool
- **pnpm 8.11** - Fast, disk-efficient Node.js package manager
- **Makefile** - 40+ targets for common development tasks
- **Bash** - Setup and automation scripts

**HARDWARE INTEGRATION:**
- **ALSA** - Linux audio and MIDI subsystem
- **Steinberg UR22** - Audio interface support
- **MPC One** - MIDI controller integration

### Key Architectural Decisions

1. **Rust over Python/JavaScript** - For performance-critical MIDI/audio processing
2. **Tauri over Electron** - Smaller bundle size, native performance, Rust backend
3. **Svelte over React** - Less boilerplate, better performance, simpler reactivity
4. **PostgreSQL over MongoDB** - Relational data with ACID guarantees, pgvector for embeddings
5. **Meilisearch over Elasticsearch** - Simpler setup, faster indexing, better out-of-box relevance

### Language Usage by Phase

- **Phase 1 (Database Setup):** 90% SQL, 5% YAML, 5% Bash
- **Phase 2 (Pipeline Backend):** 95% Rust, 5% TOML
- **Phase 3 (Pipeline Frontend):** 40% Svelte, 40% TypeScript, 20% HTML/CSS
- **Phase 4 (DAW Backend):** 95% Rust, 5% TOML
- **Phase 5 (DAW Frontend):** 35% Svelte, 45% TypeScript, 20% HTML/CSS

---

## 2. PROJECT ARCHITECTURE

### Final Summary

The MIDI Library System follows a **three-tier architecture** with strict separation of concerns:

```
┌────────────────────────────────────────────────┐
│           FRONTEND (Svelte + TypeScript)        │
│  - User Interface                              │
│  - State Management (Stores)                   │
│  - Component Library                           │
└─────────────────┬──────────────────────────────┘
                  │ Tauri IPC
┌─────────────────▼──────────────────────────────┐
│           BACKEND (Rust + Tauri)               │
│  - Business Logic                              │
│  - MIDI Processing                             │
│  - File System Operations                      │
│  - Database Access                             │
└─────────────────┬──────────────────────────────┘
                  │ SQL + HTTP
┌─────────────────▼──────────────────────────────┐
│           DATABASE (PostgreSQL + Meilisearch)  │
│  - Data Persistence                            │
│  - Full-Text Search                            │
│  - Vector Similarity                           │
└────────────────────────────────────────────────┘
```

### Three Main Components

**1. Database Layer**
- **Purpose:** Centralized data storage and search
- **Location:** `database/`
- **Contents:** SQL migrations, seed data, docker-compose.yml
- **Technology:** PostgreSQL 16 + Meilisearch 1.5
- **Scale:** Optimized for 3M+ MIDI files

**2. Pipeline Application (Batch Processor)**
- **Purpose:** Bulk MIDI file import and analysis
- **Location:** `pipeline/`
- **Architecture:** Tauri app (Rust backend + Svelte frontend)
- **Features:**
  - Batch file import
  - Archive extraction (ZIP, RAR, 7z)
  - MIDI analysis (BPM, key, chords)
  - Database indexing
  - Deduplication

**3. DAW Application (Real-Time Sequencer)**
- **Purpose:** Interactive MIDI creation and playback
- **Location:** `daw/`
- **Architecture:** Tauri app (Rust backend + Svelte frontend)
- **Features:**
  - Multi-track sequencer
  - Real-time MIDI playback
  - Hardware MIDI I/O
  - Piano roll editor
  - Mixer panel

### Shared Components

**Shared Rust Library** (`shared/rust/`):
- MIDI parsing (analysis-focused)
- Musical analysis (BPM, key, chord detection)
- Database models and repositories
- Common utilities

**Shared TypeScript Library** (`shared/typescript/`):
- Type definitions (matching Rust models)
- API client utilities
- Common frontend helpers

### Workspace Structure

Uses **Cargo workspace** for code sharing:
```toml
[workspace]
members = [
    "shared/rust",
    "pipeline/src-tauri",
    "daw/src-tauri",
    "scripts/import-tool"
]
```

Benefits:
- Shared dependencies across crates
- Single `cargo build` builds all components
- Type sharing between Pipeline and DAW
- Unified testing

---

## 3. THREE ARCHETYPES PATTERN

### Final Summary

The Three Archetypes Pattern is the **core organizational philosophy** for all code in the project. Every piece of code must be classified as one of three types:

### Archetype 1: Task-O-Matic

**Definition:** Complete standalone tasks that run from start to finish

**Characteristics:**
- ✅ Has `main()` function or equivalent entry point
- ✅ User-facing (CLI, GUI app, script)
- ✅ Complete workflow (setup → process → cleanup)
- ✅ Can use I/O freely
- ❌ NOT imported by other code
- ❌ NOT a library

**Locations:**
- `{component}/src/main.rs` - Application entry points
- `bin/` directories - CLI tools
- `migrations/*.sql` - Database migrations (run once)
- `*.svelte` components - Complete UI components

**Examples:**
- `pipeline/src-tauri/src/main.rs` - Pipeline app launcher
- `daw/src-tauri/src/main.rs` - DAW app launcher
- `scripts/import-tool/src/main.rs` - CLI import tool
- `database/migrations/001_initial_schema.sql` - Schema setup

**Testing:** End-to-end tests

---

### Archetype 2: Grown-up Script

**Definition:** Orchestration and I/O operations that are also reusable

**Characteristics:**
- ✅ Can be imported by other code
- ✅ Does I/O (file system, database, network, hardware)
- ✅ Has side effects
- ✅ Reusable orchestration logic
- ✅ Uses **entry point + implementation pattern**
- ❌ NOT pure (has side effects)

**Locations:**
- `{component}/src-tauri/src/commands/` - Tauri commands
- `{component}/src-tauri/src/services/` - Business logic
- `shared/rust/src/db/repositories/` - Database access
- `shared/rust/src/search/` - Meilisearch integration
- `src/lib/stores/*.ts` - Svelte stores (frontend state management)

**Entry Point + Implementation Pattern:**
```rust
// Entry point (thin wrapper for Tauri)
#[tauri::command]
pub async fn search_files(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<File>, String> {
    search_files_impl(&state.db_pool, &query)
        .await
        .map_err(|e| e.to_string())
}

// Implementation (testable, reusable)
pub async fn search_files_impl(
    pool: &PgPool,
    query: &str
) -> Result<Vec<File>, DatabaseError> {
    // Real logic here - can be tested without Tauri
}
```

**Why This Pattern:**
- Entry point handles framework-specific conversions
- Implementation is framework-agnostic and testable
- Other code can import and use `_impl` function
- Easy to mock dependencies in tests

**Testing:** Integration tests with mocked I/O (60%+ coverage)

---

### Archetype 3: Trusty Module

**Definition:** Pure, well-tested, reusable logic with NO side effects

**Characteristics:**
- ✅ Pure functions (same input = same output)
- ✅ NO I/O operations
- ✅ NO side effects
- ✅ Highly testable
- ✅ Can be imported anywhere
- ✅ Could be extracted to separate crate
- ❌ NO `main()` function
- ❌ NO file/database/network access
- ❌ NO global state changes

**Locations:**
- `shared/rust/src/core/` - **PRIMARY LOCATION** (pure MIDI and analysis logic)
- `shared/rust/src/utils/` - Pure utilities
- `src/lib/utils/*.ts` - Pure TypeScript utilities
- `src/lib/types/*.ts` - Type definitions

**CRITICAL RULE:** Everything in `core/` directories MUST be a Trusty Module

**Examples:**
- `shared/rust/src/core/midi/parser.rs` - Parse MIDI bytes (no I/O)
- `shared/rust/src/core/analysis/bpm_detector.rs` - Detect BPM (pure algorithm)
- `shared/rust/src/core/analysis/key_detector.rs` - Detect musical key (pure)
- `src/lib/utils/validation.ts` - Pure validation functions

**Testing:** Unit tests with 80%+ coverage (REQUIRED)

**Code Quality Requirements:**
- NO `.unwrap()` or `.expect()` in production code
- Comprehensive doc comments on all public functions
- 80%+ test coverage (enforced)
- Examples in documentation
- Single responsibility

---

### Decision Tree

```
┌─────────────────────────────────────────────────────────────┐
│ Question 1: Will other code import/reuse this?              │
├─────────────────────────────────────────────────────────────┤
│  NO  →  Question 2: Is it a complete standalone task?       │
│         ├─ YES → TASK-O-MATIC                               │
│         └─ NO  → Rethink (probably should be reusable)      │
│                                                              │
│  YES →  Question 3: Does it also need to run standalone?    │
│         ├─ YES → GROWN-UP SCRIPT                            │
│         └─ NO  → Question 4: Does it do I/O or side effects?│
│                  ├─ YES → GROWN-UP SCRIPT                   │
│                  └─ NO  → TRUSTY MODULE                     │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. COMPONENT ARCHITECTURE

### Final Summary

Each component has a specific purpose and clear boundaries:

### Database Component

**Structure:**
```
database/
├── migrations/          # SQL schema evolution
│   ├── 001_initial_schema.sql
│   ├── 002_add_search_index.sql
│   ├── 003_add_collections.sql
│   ├── 004_add_tags.sql
│   ├── 005_add_custom_fields.sql
│   └── 006_add_playback_state.sql
├── seed/               # Test data
│   └── sample_data.sql
└── docker-compose.yml  # PostgreSQL + Meilisearch
```

**Key Features:**
- Numbered migrations for version control
- pgvector extension for semantic search
- GiST, GIN, B-tree indexes for performance
- JSONB for flexible metadata
- Optimized for 3M+ files

---

### Shared Rust Library Component

**Structure:**
```
shared/rust/src/
├── lib.rs              # Module exports
├── core/               # TRUSTY MODULES ONLY
│   ├── midi/           # Pure MIDI parsing
│   │   ├── parser.rs   # 921 lines
│   │   ├── types.rs
│   │   ├── events.rs
│   │   └── writer.rs
│   └── analysis/       # Pure analysis algorithms
│       ├── bpm_detector.rs
│       ├── key_detector.rs
│       ├── chord_detector.rs
│       └── pattern_analyzer.rs
├── db/                 # GROWN-UP SCRIPTS (database I/O)
│   ├── models/         # Data structures
│   │   ├── file.rs
│   │   ├── collection.rs
│   │   ├── tag.rs
│   │   └── playback_state.rs
│   ├── repositories/   # Repository pattern
│   │   ├── file_repository.rs
│   │   ├── collection_repository.rs
│   │   ├── tag_repository.rs
│   │   └── search_repository.rs
│   ├── connection.rs   # DB connection management
│   └── error.rs        # Error types
├── search/             # GROWN-UP SCRIPTS (Meilisearch)
│   ├── client.rs
│   ├── indexer.rs
│   └── query.rs
└── utils/              # Mixed (pure = Trusty, I/O = Grown-up)
    ├── validation.rs   # Trusty Module
    └── logging.rs      # Grown-up Script
```

**Critical Rule:** `core/` contains ONLY Trusty Modules (pure logic, no I/O)

---

### Pipeline Component (Batch Processor)

**Backend Structure:**
```
pipeline/src-tauri/src/
├── main.rs             # TASK-O-MATIC (app entry)
├── lib.rs              # Library exports
├── commands/           # GROWN-UP SCRIPTS (Tauri commands)
│   ├── file_commands.rs
│   ├── import_commands.rs
│   ├── search_commands.rs
│   └── analysis_commands.rs
├── services/           # GROWN-UP SCRIPTS (business logic)
│   ├── import_service.rs
│   └── batch_processor.rs
└── state.rs            # App state management
```

**Frontend Structure:**
```
pipeline/src/
├── App.svelte          # Root component
├── main.ts             # Frontend entry
├── lib/
│   ├── components/     # TASK-O-MATICS (UI components)
│   │   ├── library/
│   │   │   ├── FileList.svelte
│   │   │   ├── FileCard.svelte
│   │   │   └── SearchBar.svelte
│   │   ├── import/
│   │   │   ├── ImportDialog.svelte
│   │   │   └── ProgressBar.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       └── Modal.svelte
│   ├── stores/         # GROWN-UP SCRIPTS (state management)
│   │   ├── libraryStore.ts
│   │   ├── importStore.ts
│   │   └── uiStore.ts
│   └── utils/          # TRUSTY MODULES (pure utilities)
│       ├── formatters.ts
│       └── constants.ts
└── assets/             # Static files
```

**Key Features:**
- Batch file import (handles thousands of files)
- Archive extraction (ZIP, RAR, 7z)
- Parallel processing with rayon
- Progress tracking with events
- Deduplication via hashing

---

### DAW Component (Real-Time Sequencer)

**Backend Structure:**
```
daw/src-tauri/src/
├── main.rs             # TASK-O-MATIC (app entry)
├── lib.rs
├── commands/           # GROWN-UP SCRIPTS
│   ├── file_commands.rs
│   ├── playback_commands.rs
│   ├── sequencer_commands.rs
│   └── export_commands.rs
├── audio/              # GROWN-UP SCRIPTS (audio I/O)
│   ├── engine.rs       # Audio engine
│   ├── midi_output.rs  # MIDI device output
│   └── track.rs        # Track management
├── sequencer/          # GROWN-UP SCRIPTS
│   ├── timeline.rs
│   ├── clip.rs
│   └── transport.rs
└── state.rs
```

**Frontend Structure:**
```
daw/src/
├── App.svelte
├── main.ts
├── lib/
│   ├── components/
│   │   ├── sequencer/
│   │   │   ├── Timeline.svelte       # 600+ lines
│   │   │   ├── TrackList.svelte
│   │   │   ├── PianoRoll.svelte      # 800+ lines
│   │   │   └── MixerPanel.svelte
│   │   ├── library/
│   │   │   ├── FileBrowser.svelte
│   │   │   └── QuickSearch.svelte
│   │   └── common/
│   │       ├── Button.svelte
│   │       └── Slider.svelte
│   ├── stores/
│   │   ├── sequencerStore.ts
│   │   ├── playbackStore.ts
│   │   ├── libraryStore.ts
│   │   └── uiStore.ts
│   └── utils/
│       ├── midiUtils.ts
│       └── timeUtils.ts
└── assets/
```

**Key Features:**
- Real-time MIDI playback with midir
- Multi-track sequencing engine (800+ lines)
- Piano roll editor with drag-and-drop
- Hardware MIDI I/O
- Transport controls (play/pause/stop/loop)
- Mixer with volume/pan per track

**Note:** DAW has its own MIDI parsing optimized for real-time playback (separate from shared library's batch parsing)

---

### Scripts Component

**Structure:**
```
scripts/
├── setup/              # One-time setup
│   ├── setup.sh        # Main setup script
│   └── install-deps.sh # Dependency installation
├── launch/             # App launchers
│   ├── launch-all.sh   # Start all apps
│   ├── launch-pipeline.sh
│   ├── launch-daw.sh
│   ├── status.sh       # Check app status
│   └── stop-all.sh     # Stop all apps
├── verify/             # Testing and validation
│   ├── integration_test.sh
│   └── quick_check.sh
├── import-tool/        # TASK-O-MATIC (CLI Rust binary)
│   ├── Cargo.toml
│   └── src/main.rs
└── test-all.sh         # Run all tests
```

---

## 5. DEVELOPMENT ENVIRONMENT SETUP

### Final Summary

Comprehensive VS Code setup optimized for Rust + Svelte + TypeScript development.

### Required Dependencies

**System-level (Ubuntu 25.04):**
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js + pnpm
curl -fsSL https://get.pnpm.io/install.sh | sh

# Tauri dependencies
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# ALSA (MIDI/Audio)
sudo apt install libasound2-dev

# Docker + Docker Compose
# (Follow official Docker installation guide)
```

**Project-level:**
```bash
# Clone and setup
git clone <repo-url>
cd midi-software-center
./scripts/setup.sh

# Or manual setup
docker-compose up -d              # Start database
cargo build --workspace           # Build all Rust crates
cd pipeline && pnpm install       # Install Pipeline frontend deps
cd ../daw && pnpm install         # Install DAW frontend deps
```

### VS Code Extensions (Essential)

**Rust Development:**
- `rust-lang.rust-analyzer` - IntelliSense, formatting, linting
- `vadimcn.vscode-lldb` - Debugging
- `serayuzgur.crates` - Cargo.toml dependency management
- `tamasfe.even-better-toml` - TOML syntax highlighting

**Svelte/TypeScript:**
- `svelte.svelte-vscode` - Svelte language support
- `dbaeumer.vscode-eslint` - JavaScript/TypeScript linting
- `esbenp.prettier-vscode` - Code formatting

**Database:**
- `mtxr.sqltools` - SQL client and query execution
- `mtxr.sqltools-driver-pg` - PostgreSQL driver

**General:**
- `eamodio.gitlens` - Git supercharger
- `wayou.vscode-todo-highlight` - TODO highlighting
- `usernamehw.errorlens` - Inline error messages

### VS Code Configuration

**Workspace Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all",

  "svelte.plugin.typescript.diagnostics.enable": true,
  "svelte.plugin.css.diagnostics.enable": true,

  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },

  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  },

  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  },

  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

**Tasks** (`.vscode/tasks.json`):
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "run-database",
      "type": "shell",
      "command": "docker-compose up -d",
      "problemMatcher": []
    },
    {
      "label": "tauri-dev-pipeline",
      "type": "shell",
      "command": "cd pipeline && pnpm tauri dev",
      "problemMatcher": []
    },
    {
      "label": "tauri-dev-daw",
      "type": "shell",
      "command": "cd daw && pnpm tauri dev",
      "problemMatcher": []
    },
    {
      "label": "test-all",
      "type": "shell",
      "command": "cargo test --workspace",
      "problemMatcher": ["$rustc"]
    }
  ]
}
```

**Launch Configuration** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Pipeline",
      "cargo": {
        "args": ["build", "--package", "pipeline", "--bin", "pipeline"]
      },
      "cwd": "${workspaceFolder}/pipeline/src-tauri"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug DAW",
      "cargo": {
        "args": ["build", "--package", "daw", "--bin", "daw"]
      },
      "cwd": "${workspaceFolder}/daw/src-tauri"
    }
  ]
}
```

### Keyboard Shortcuts

**Recommended** (`.vscode/keybindings.json`):
```json
[
  {
    "key": "ctrl+shift+b",
    "command": "workbench.action.tasks.runTask",
    "args": "build-workspace"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.runTask",
    "args": "test-all"
  },
  {
    "key": "ctrl+shift+d",
    "command": "workbench.action.tasks.runTask",
    "args": "tauri-dev-pipeline"
  },
  {
    "key": "ctrl+shift+r",
    "command": "workbench.action.tasks.runTask",
    "args": "run-database"
  }
]
```

### Database Connection (SQLTools)

**Configuration** (`.vscode/sqltools.json`):
```json
{
  "connections": [
    {
      "name": "MIDI Library Database",
      "driver": "PostgreSQL",
      "server": "localhost",
      "port": 5432,
      "database": "midi_library",
      "username": "midi_user",
      "password": "midi_password"
    }
  ]
}
```

---

## 6. LAYER-SPECIFIC ARCHITECTURE

### Final Summary

Each layer (Backend, Frontend, Database, Hardware) has specific manifestations of the Three Archetypes:

### Backend Architecture (Rust/Tauri)

**Archetype Manifestations:**

**Task-O-Matic (Backend):**
- Location: `src-tauri/src/main.rs`, `bin/*.rs`
- Characteristics: `#[tokio::main]`, CLI tools, application launchers
- Examples:
  ```rust
  #[tokio::main]
  async fn main() {
      tauri::Builder::default()
          .setup(|app| {
              // Initialize database
              // Setup MIDI devices
              // Register commands
              Ok(())
          })
          .run(tauri::generate_context!())
          .expect("error while running application");
  }
  ```

**Grown-up Script (Backend):**
- Location: `src-tauri/src/commands/*.rs`, `src-tauri/src/db/repositories/*.rs`
- Characteristics: `#[tauri::command]`, async, error handling, I/O
- Pattern:
  ```rust
  // Entry point
  #[tauri::command]
  pub async fn search_files(query: String, state: State<'_, AppState>) -> Result<Vec<File>, String> {
      search_files_impl(&state.db_pool, &query).await.map_err(|e| e.to_string())
  }

  // Implementation
  pub async fn search_files_impl(pool: &PgPool, query: &str) -> Result<Vec<File>, DbError> {
      // Real logic - testable without Tauri
  }
  ```

**Trusty Module (Backend):**
- Location: `src-tauri/src/core/*.rs`, `shared/rust/src/core/*.rs`
- Characteristics: Pure functions, no async, no I/O
- Examples:
  ```rust
  pub fn parse_midi(data: &[u8]) -> Result<MidiFile, ParseError> {
      // Pure parsing logic
  }

  pub fn detect_bpm(midi: &MidiFile) -> Result<f64, BpmError> {
      // Pure analysis algorithm
  }
  ```

**Backend-Specific Patterns:**

1. **Tauri Command Pattern** - Entry point + implementation
2. **Repository Pattern** - Database access abstraction
3. **MIDI Processing Pattern** - I/O wrapper around pure parsing
4. **Error Conversion** - `thiserror` for libraries, `anyhow` for apps

---

### Frontend Architecture (Svelte/TypeScript)

**Archetype Manifestations:**

**Task-O-Matic (Frontend):**
- Location: `src/lib/components/*.svelte`, `src/routes/*.svelte`
- Characteristics: Complete UI components, user interactions
- Examples: `FileBrowser.svelte`, `PianoRoll.svelte`, `+page.svelte`
- Pattern:
  ```svelte
  <script lang="ts">
    import { fileStore } from '$lib/stores/fileStore';

    $: files = $fileStore.filteredFiles;

    function handleClick(file: File) {
      // Handle user interaction
    }
  </script>

  <div class="file-list">
    {#each files as file}
      <FileItem {file} on:click={() => handleClick(file)} />
    {/each}
  </div>
  ```

**Grown-up Script (Frontend):**
- Location: `src/lib/stores/*.ts`
- Characteristics: State management, Tauri IPC, side effects
- Examples: `workspaceStore.ts`, `midiStore.ts`
- Pattern:
  ```typescript
  import { writable } from 'svelte/store';
  import { invoke } from '@tauri-apps/api';

  export const fileStore = writable<FileState>({
    files: [],
    loading: false,
    error: null
  });

  export const fileActions = {
    async loadFiles() {
      fileStore.update(s => ({ ...s, loading: true }));
      try {
        const files = await invoke<File[]>('get_files'); // Tauri IPC
        fileStore.set({ files, loading: false, error: null });
      } catch (error) {
        fileStore.update(s => ({ ...s, loading: false, error }));
      }
    }
  };
  ```

**Trusty Module (Frontend):**
- Location: `src/lib/utils/*.ts`, `src/lib/types/*.ts`
- Characteristics: Pure functions, type definitions
- Examples: `formatting.ts`, `validation.ts`, `models.ts`
- Pattern:
  ```typescript
  export function validateFile(file: File): boolean {
    return file.size > 0 && file.name.endsWith('.mid');
  }

  export function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  ```

**Frontend-Specific Patterns:**

1. **Component Pattern** - Reactive UI with Svelte
2. **Store Pattern** - State management with side effects
3. **Utility Pattern** - Pure helper functions
4. **Type Definition Pattern** - TypeScript interfaces matching Rust models

---

### Database Architecture (PostgreSQL/SQLx)

**Archetype Manifestations:**

**Task-O-Matic (Database):**
- Location: `migrations/*.sql`, `seed/*.sql`
- Characteristics: Run-once scripts, schema changes
- Examples:
  ```sql
  -- migrations/002_add_analysis.sql
  CREATE TABLE midi_analysis (
      id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
      midi_file_id UUID NOT NULL REFERENCES midi_files(id),
      bpm DOUBLE PRECISION,
      key TEXT,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```

**Grown-up Script (Database):**
- Location: `src/repositories/*.rs`
- Characteristics: SQL queries, connection pooling, transactions
- Pattern:
  ```rust
  pub struct FileRepository {
      pool: PgPool,
  }

  impl FileRepository {
      pub async fn create(&self, file: NewFile) -> Result<File, DbError> {
          sqlx::query_as!(
              File,
              r#"INSERT INTO files (...) VALUES (...) RETURNING *"#,
              file.path, file.name
          )
          .fetch_one(&self.pool)
          .await
          .map_err(DbError::from)
      }
  }
  ```

**Trusty Module (Database):**
- Location: `src/models/*.rs`, `src/queries/*.rs`
- Characteristics: Data structures, validation
- Pattern:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
  pub struct File {
      pub id: Uuid,
      pub path: String,
      pub name: String,
      pub size: i64,
  }

  impl File {
      pub fn is_valid(&self) -> bool {
          !self.path.is_empty() && self.size > 0
      }
  }
  ```

**Database-Specific Patterns:**

1. **Repository Pattern** - Encapsulate database access
2. **Model Pattern** - Data structures with validation
3. **Migration Pattern** - Versioned schema changes
4. **Query Builder Pattern** - Type-safe query construction

---

### Hardware Architecture (MIDI/ALSA)

**MIDI Hardware Integration:**

**Supported Devices:**
- Steinberg UR22 (audio interface)
- MPC One (MIDI controller)
- Generic ALSA MIDI devices

**MIDI I/O Pattern:**
```rust
// GROWN-UP SCRIPT - Hardware abstraction
pub struct MidiDeviceManager {
    input_port: MidiInput,
    output_port: MidiOutput,
}

impl MidiDeviceManager {
    pub async fn send_note_on(&mut self, note: u8, velocity: u8) -> Result<(), MidiError> {
        // Hardware I/O
        self.output_port.send(&[0x90, note, velocity])?;
        Ok(())
    }

    pub async fn receive_events(&mut self) -> Result<Vec<MidiEvent>, MidiError> {
        // Read from hardware
    }
}

// TRUSTY MODULE - MIDI message parsing
pub fn parse_midi_message(bytes: &[u8]) -> Result<MidiMessage, ParseError> {
    // Pure parsing logic
}
```

**ALSA Configuration:**
```bash
# System dependencies
sudo apt install libasound2-dev

# Set real-time priority for audio
sudo usermod -aG audio $USER
```

---

## 7. DEVELOPMENT WORKFLOWS

### Final Summary

Structured 8-step processes for backend and frontend development.

### Backend Development Workflow (Rust/Tauri)

**Step 1: Understand the Feature**
- Read requirements
- Identify affected components
- Determine if Task-O-Matic, Grown-up Script, or Trusty Module

**Step 2: Design the Solution**
- Use archetype decision tree
- Plan file structure
- Design interfaces and data structures
- Identify dependencies

**Step 3: Set Up Environment**
```bash
git checkout -b feature/name
docker-compose up -d postgres meilisearch
cargo test --workspace  # Ensure clean state
```

**Step 4: Implement Trusty Modules First**
- Write pure logic with no I/O
- Add doc comments while coding
- Write tests while coding (80%+ coverage)
- Example:
  ```rust
  /// Detect BPM from MIDI file
  ///
  /// # Examples
  /// ```
  /// let midi = parse_midi(&bytes)?;
  /// let bpm = detect_bpm(&midi)?;
  /// assert_eq!(bpm, 120.0);
  /// ```
  pub fn detect_bpm(midi: &MidiFile) -> Result<f64, BpmError> {
      // Pure implementation
  }

  #[cfg(test)]
  mod tests {
      #[test]
      fn test_single_tempo() { /* ... */ }

      #[test]
      fn test_multiple_tempos() { /* ... */ }

      #[test]
      fn test_no_tempo() { /* ... */ }
  }
  ```

**Step 5: Implement Grown-up Scripts**
- Use entry + implementation pattern
- Write integration tests
- Example:
  ```rust
  #[tauri::command]
  pub async fn analyze_file(path: String, state: State<'_, AppState>) -> Result<Analysis, String> {
      analyze_file_impl(&path, &state.db_pool).await.map_err(|e| e.to_string())
  }

  pub async fn analyze_file_impl(path: &str, pool: &PgPool) -> Result<Analysis, AnalysisError> {
      let bytes = tokio::fs::read(path).await?;
      let midi = parse_midi(&bytes)?;  // Trusty Module
      let bpm = detect_bpm(&midi)?;    // Trusty Module
      // Save to database
      Ok(Analysis { bpm })
  }

  #[cfg(test)]
  mod tests {
      #[tokio::test]
      async fn test_analyze_file_impl() {
          let pool = create_test_pool().await;
          let result = analyze_file_impl("test.mid", &pool).await;
          assert!(result.is_ok());
      }
  }
  ```

**Step 6: Write Tests**
```bash
# Run tests
cargo test --package my-package

# Check coverage
cargo tarpaulin --out Html --output-dir coverage

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

**Step 7: Document**
```bash
# Generate documentation
cargo doc --no-deps --open

# Check for missing docs
cargo doc --no-deps 2>&1 | grep "missing documentation"
```

**Step 8: Review and Commit**
- Use code review checklist
- Create meaningful commit message
- Push and create PR

---

### Frontend Development Workflow (Svelte/TypeScript)

**Step 1: Understand the Feature**
- Identify if it's a Page, Component, Store, or Utility
- Determine archetype

**Step 2: Determine Archetype**
```
Is this a complete UI element?
├─ YES → TASK-O-MATIC (components/*.svelte)
└─ NO → Does it manage state/side effects?
    ├─ YES → GROWN-UP SCRIPT (stores/*.ts)
    └─ NO → TRUSTY MODULE (utils/*.ts, types/*.ts)
```

**Step 3: Set Up File Structure**
```bash
# For component
touch src/lib/components/library/FileBrowser.svelte

# For store
touch src/lib/stores/fileBrowserStore.ts

# For utility
touch src/lib/utils/fileValidation.ts
```

**Step 4: Implement**

**Component (Task-O-Matic):**
```svelte
<script lang="ts">
  import { fileBrowserStore } from '$lib/stores/fileBrowserStore';
  import { formatFileSize } from '$lib/utils/fileValidation';

  $: files = $fileBrowserStore.files;
  $: loading = $fileBrowserStore.loading;

  function handleSelect(file: File) {
    // User interaction handler
  }
</script>

<div class="file-browser">
  {#if loading}
    <div class="loading">Loading...</div>
  {:else}
    {#each files as file}
      <div class="file-item" on:click={() => handleSelect(file)}>
        {file.name} - {formatFileSize(file.size)}
      </div>
    {/each}
  {/if}
</div>

<style>
  .file-browser {
    /* Styling */
  }
</style>
```

**Store (Grown-up Script):**
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api';

interface State {
  files: File[];
  loading: boolean;
  error: string | null;
}

export const fileBrowserStore = writable<State>({
  files: [],
  loading: false,
  error: null
});

export const fileBrowserActions = {
  async loadFiles() {
    fileBrowserStore.update(s => ({ ...s, loading: true }));
    try {
      const files = await invoke<File[]>('get_files');
      fileBrowserStore.set({ files, loading: false, error: null });
    } catch (error) {
      fileBrowserStore.update(s => ({
        ...s,
        loading: false,
        error: error as string
      }));
    }
  }
};
```

**Utility (Trusty Module):**
```typescript
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

export function validateMidiFile(file: File): boolean {
  return file.size > 0 && file.name.endsWith('.mid');
}

// Tests (using Vitest)
describe('formatFileSize', () => {
  test('formats bytes', () => {
    expect(formatFileSize(500)).toBe('500.00 B');
  });

  test('formats kilobytes', () => {
    expect(formatFileSize(2048)).toBe('2.00 KB');
  });
});
```

**Step 5: Test**
```bash
# Type check
pnpm check

# Run tests
pnpm test

# Lint
pnpm lint

# Format
pnpm format
```

**Step 6: Review and Commit**

---

## 8. CODE QUALITY & TESTING

### Final Summary

Strict code quality requirements with enforced coverage targets.

### Code Quality Requirements

**Universal Rules (All Code):**

1. **Error Handling:**
   ```rust
   // ❌ BAD
   let value = option.unwrap();
   let result = operation.expect("failed");

   // ✅ GOOD
   let value = option.ok_or(MyError::MissingValue)?;
   let result = operation.map_err(|e| MyError::from(e))?;
   ```

2. **Documentation:**
   ```rust
   /// Brief description
   ///
   /// More detailed explanation.
   ///
   /// # Arguments
   /// * `param` - Parameter description
   ///
   /// # Returns
   /// * `Ok(Value)` - Success case
   /// * `Err(Error)` - Error case
   ///
   /// # Examples
   /// ```
   /// let result = my_function(param)?;
   /// assert_eq!(result, expected);
   /// ```
   pub fn my_function(param: Type) -> Result<Value, Error> {
       // Implementation
   }
   ```

3. **Performance:**
   ```rust
   // ✅ Use &str for parameters
   pub fn process(input: &str) -> String { }

   // ❌ Don't use String for parameters
   pub fn process(input: String) -> String { }  // Unnecessary allocation

   // ✅ Use Vec<T> for sequences
   let items: Vec<Item> = vec![];

   // ❌ Avoid LinkedList
   let items: LinkedList<Item> = LinkedList::new();  // Poor cache locality
   ```

4. **Code Style:**
   ```bash
   # Rust
   cargo fmt            # Format code
   cargo clippy         # Linting

   # TypeScript/Svelte
   pnpm format          # Prettier
   pnpm lint            # ESLint
   ```

### Testing Requirements

**By Archetype:**

| Archetype | Test Type | Coverage | Location |
|-----------|-----------|----------|----------|
| Task-O-Matic | E2E | As needed | `tests/e2e/` |
| Grown-up Script | Integration | 60%+ | `#[cfg(test)]` + `tests/integration/` |
| Trusty Module | Unit | 80%+ (REQUIRED) | `#[cfg(test)]` in same file |

**Test Organization:**

```rust
// Unit tests (Trusty Module)
pub fn my_function(input: &str) -> Result<Output, Error> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        let result = my_function("valid").unwrap();
        assert_eq!(result.field, "expected");
    }

    #[test]
    fn test_edge_case_empty() {
        let result = my_function("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_invalid() {
        let result = my_function("invalid");
        assert!(matches!(result, Err(Error::Invalid(_))));
    }
}
```

```rust
// Integration tests (Grown-up Script)
// tests/integration/repository_test.rs

use sqlx::PgPool;
use my_app::repositories::FileRepository;

#[sqlx::test]
async fn test_file_repository_crud(pool: PgPool) -> sqlx::Result<()> {
    let repo = FileRepository::new(&pool);

    // Create
    let file = create_test_file();
    let created = repo.insert(&file).await?;

    // Read
    let found = repo.find_by_id(created.id).await?;
    assert!(found.is_some());

    // Update
    let updated = repo.update(&created).await?;

    // Delete
    repo.delete(created.id).await?;

    Ok(())
}
```

**Coverage Enforcement:**

```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Check specific module
cargo tarpaulin --out Stdout -- module_name

# CI/CD enforcement
cargo tarpaulin --out Xml
# Parse XML and fail if < 80% for core/ modules
```

**Testing Best Practices:**

1. **Unit Tests** - Fast, focused, deterministic
2. **Integration Tests** - Test with real database (use test fixtures)
3. **E2E Tests** - Complete user workflows
4. **Test Data** - Use helper functions for test data creation
5. **Mocking** - Mock external services, use real database with test pool

---

## 9. BUILD SYSTEM & DEPLOYMENT

### Final Summary

Makefile-based build system with optimized Cargo profiles.

### Makefile Targets (40+)

**Setup:**
```bash
make setup          # Complete project setup
make install-deps   # Install all dependencies
make docker-up      # Start database services
```

**Development:**
```bash
make dev-pipeline   # Launch Pipeline in dev mode
make dev-daw        # Launch DAW in dev mode
make dev-both       # Launch both applications
```

**Building:**
```bash
make build          # Build all workspace members
make build-pipeline # Build Pipeline for production
make build-daw      # Build DAW for production
make release        # Create optimized release bundles
```

**Testing:**
```bash
make test           # Run all tests
make test-rust      # Rust tests only
make test-frontend  # Frontend tests only
make coverage       # Generate coverage report
```

**Database:**
```bash
make db-migrate     # Run migrations
make db-reset       # ⚠️ Reset database (destructive)
make db-backup      # Create database backup
make docker-logs    # View database logs
```

**Code Quality:**
```bash
make format         # Format all code (Rust + TypeScript)
make lint           # Run all linters
make check          # Format + lint + test
make clippy         # Run Rust clippy
```

**Cleanup:**
```bash
make clean          # Clean build artifacts
make clean-all      # Clean everything (build + cache)
```

### Cargo Workspace Configuration

**Root Cargo.toml:**
```toml
[workspace]
members = [
    "shared/rust",
    "pipeline/src-tauri",
    "daw/src-tauri",
    "scripts/import-tool"
]

# Workspace-level dependencies
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"

# Optimized build profiles
[profile.dev]
opt-level = 0  # Your code: no optimization
split-debuginfo = "unpacked"

[profile.dev.package."*"]
opt-level = 3  # Dependencies: maximum optimization

[profile.release]
opt-level = 3  # Maximum optimization
lto = "thin"   # Link-time optimization
strip = true   # Remove debug symbols
codegen-units = 1
```

**Why This Profile?**
- Dev: Fast compilation of your code, fast execution of dependencies
- Release: Maximum performance, small binary size

### Docker Compose Configuration

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: midi_user
      POSTGRES_PASSWORD: midi_password
      POSTGRES_DB: midi_library
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./database/migrations:/docker-entrypoint-initdb.d

  meilisearch:
    image: getmeili/meilisearch:v1.5
    environment:
      MEILI_MASTER_KEY: your-master-key
    ports:
      - "7700:7700"
    volumes:
      - meili_data:/meili_data

volumes:
  postgres_data:
  meili_data:
```

### Build Times

**First Build:** 10-15 minutes (compiling all Rust dependencies)
**Incremental Builds:** 30s-2min (only changed code)
**Dev Builds:** Fast, unoptimized
**Release Builds:** 2-3 min, maximum optimization

### Deployment Strategy

**Database (Separate Deployment):**
```bash
docker-compose up -d postgres meilisearch
sqlx migrate run
```

**Pipeline (Independent):**
```bash
cd pipeline
cargo build --release
# Bundle includes frontend assets
./target/release/pipeline
```

**DAW (Independent):**
```bash
cd daw
cargo build --release
./target/release/daw
```

---

## 10. AI ASSISTANT INTEGRATION

### Final Summary

Comprehensive setup for Claude Code, Cursor, and other AI assistants.

### Claude Code Setup

**Directory Structure:**
```
.cursor/rules/
├── project-rules.mdc        # Overall architecture
├── database-rules.mdc       # Database layer rules
├── shared-rules.mdc         # Shared library rules
├── workspace-rules.mdc      # Frontend/backend rules
├── rust-rules.mdc           # Rust-specific standards
└── svelte-rules.mdc         # Svelte/TypeScript standards
```

**project-rules.mdc:**
```markdown
# MIDI Library System - Project Rules

## Project Structure
- Multi-crate Rust workspace with Tauri + Svelte frontend
- Three main components: database, shared, pipeline, daw

## Architecture Patterns
- Three Archetypes: Task-O-Matic, Grown-up Script, Trusty Module
- Clean Architecture layers
- Repository pattern for database access

## Code Style Requirements
- No .unwrap() or .expect() in production code
- All public APIs must be documented
- Trusty Modules require 80%+ test coverage

## Key Dependencies
- Backend: Rust 1.70+, Tauri 2.7, tokio, sqlx
- Frontend: Svelte 4.2, TypeScript 5.3, Vite 5.0
- Database: PostgreSQL 16, Meilisearch 1.5

## Development Commands
- make dev-pipeline - Start Pipeline development server
- make dev-daw - Start DAW development server
- make test - Run all tests
- make db-migrate - Run database migrations
```

**database-rules.mdc:**
```markdown
# Database Layer Rules

## Responsibilities
- Schema design and migrations
- Type-safe queries with sqlx
- Connection pooling and transactions

## Patterns
- Repository pattern for all database access
- Models in shared/rust/src/db/models/
- Repositories in shared/rust/src/db/repositories/

## Important
- All SQL queries must be in queries.rs or repository files
- Migrations must be backward-compatible when possible
- Use sqlx::query_as! for type-safe queries
- All queries must handle errors properly
```

**shared-rules.mdc:**
```markdown
# Shared Library Rules

## Trusty Modules Only
- shared/rust/src/core/ contains ONLY pure functions
- NO I/O operations in core/
- NO database access in core/
- All core/ modules require 80%+ test coverage

## MIDI Processing
- MIDI parsing in core/midi/parser.rs
- MIDI writing in core/midi/writer.rs
- Analysis algorithms in core/analysis/

## Musical Intelligence
- BPM detection: core/analysis/bpm_detector.rs
- Key detection: core/analysis/key_detector.rs
- Chord detection: core/analysis/chord_detector.rs
```

**rust-rules.mdc:**
```markdown
# Rust Code Rules

## Error Handling
- Use `anyhow::Result` in application code
- Use `thiserror` for library error types
- Never use .unwrap() or .expect() in production code
- Propagate errors with ? operator

## Testing
- Unit tests in #[cfg(test)] modules
- Integration tests in tests/integration/
- Use sqlx::test for database tests

## Performance
- Use &str instead of String for function parameters when possible
- Prefer Vec over LinkedList for most cases
- Use #[derive] instead of manual implementations when possible
```

**svelte-rules.mdc:**
```markdown
# Svelte & TypeScript Rules

## Component Structure
- Use <script lang="ts"> for all components
- Props at top, then reactive statements, then functions
- Keep components under 300 lines (split if larger)

## State Management
- Use Svelte stores for global state
- Prefer derived stores over duplicating state
- Use writable for mutable state, readable for immutable

## Tauri Integration
- Use @tauri-apps/api/tauri for commands
- Handle all Tauri errors gracefully
- Show loading states during async operations
```

### VS Code Settings

**Integration** (`.vscode/settings.json`):
```json
{
  "claude.code.anthropic.include": [
    "**/*.rs",
    "**/*.svelte",
    "**/*.ts",
    "**/*.sql",
    "**/*.md"
  ],
  "claude.code.anthropic.exclude": [
    "target/**",
    "node_modules/**",
    "dist/**",
    ".git/**"
  ]
}
```

### MCP Servers Configuration

**Recommended MCP Servers:**
- `@modelcontextprotocol/server-rust` - Rust code analysis
- `@modelcontextprotocol/server-typescript` - TypeScript support
- `@modelcontextprotocol/server-svelte` - Svelte operations
- `@modelcontextprotocol/server-postgres` - PostgreSQL operations

**Setup:**
```bash
# Install globally
npm install -g @modelcontextprotocol/server-rust
npm install -g @modelcontextprotocol/server-typescript
npm install -g @modelcontextprotocol/server-svelte
npm install -g @modelcontextprotocol/server-postgres
```

**Configuration** (in `.vscode/settings.json`):
```json
{
  "claude.code.anthropic.mcpServers": {
    "rust": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-rust"]
    },
    "typescript": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-typescript"]
    },
    "postgres": {
      "command": "npx",
      "args": ["@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgres://midi_user:midi_password@localhost:5432/midi_library"
      }
    }
  }
}
```

### Grok 4 Fast API Setup (with Cline)

**Configuration:**
```json
{
  "grok4fastApi.apiKey": "your-api-key",
  "grok4fastApi.endpoint": "https://api.grok.com/v1",
  "grok4fastApi.model": "grok-4-fast"
}
```

---

## 11. DOCUMENTATION STRUCTURE

### Final Summary

Proposed layered documentation structure for the project.

### Core Foundation (High-Level)

**ARCHITECTURE-REFERENCE.md:**
- Three Archetypes definitions
- Why they matter
- Decision tree
- Cross-component relationships

### Layer-Specific Architecture

**docs/architecture/layered/**

1. **backend-architecture.md** - Rust/Tauri specific patterns
2. **frontend-architecture.md** - Svelte/TypeScript patterns
3. **database-architecture.md** - SQL/Repository patterns
4. **hardware-architecture.md** - MIDI/ALSA patterns

### Concrete Examples

**docs/architecture/examples/**

1. **backend-examples.md** - Complete Rust code examples
2. **frontend-examples.md** - Complete Svelte examples
3. **database-examples.md** - Complete SQL examples

### Development Workflows

**docs/architecture/workflows/**

1. **backend-workflow.md** - 8-step Rust development process
2. **frontend-workflow.md** - 8-step Svelte development process
3. **database-workflow.md** - Database development process

### Why This Structure?

**Problem with Single Files:**
- Generic patterns don't translate well to specific layers
- Frontend and backend developers need different guidance
- Examples get lost in theory

**Solution - Layered Approach:**
- High-level concepts in core file
- Layer-specific manifestations in separate files
- Concrete examples for each layer
- Step-by-step workflows per technology

**How They Work Together:**

1. **Developer starts with:** ARCHITECTURE-REFERENCE.md (big picture)
2. **Then consults:** Layer-specific file (Rust, Svelte, Database, Hardware)
3. **Looks at:** Concrete examples for their layer
4. **Follows:** Step-by-step workflow for their layer

---

## 12. FILE ORGANIZATION

### Final Summary

Complete project structure with clear archetype classification.

### Root Structure

```
midi-software-center/
├── database/               # Database component
├── shared/                 # Shared libraries
│   ├── rust/              # Rust shared library
│   └── typescript/        # TypeScript shared library
├── pipeline/              # Pipeline application
│   ├── src/               # Svelte frontend
│   └── src-tauri/         # Rust backend
├── daw/                   # DAW application
│   ├── src/               # Svelte frontend
│   └── src-tauri/         # Rust backend
├── scripts/               # Automation scripts
│   ├── setup/
│   ├── launch/
│   ├── verify/
│   └── import-tool/       # CLI Rust binary
├── docs/                  # Documentation
│   └── architecture/      # Architecture docs
│       ├── layered/
│       ├── examples/
│       └── workflows/
├── .vscode/               # VS Code configuration
├── .cursor/               # Cursor/Claude rules
│   └── rules/
├── Cargo.toml             # Rust workspace config
├── package.json           # Root package config
├── Makefile               # Build automation
├── docker-compose.yml     # Database services
└── README.md              # Project overview
```

### File Placement Rules

**Shared Code Decision:**
```
Will both Pipeline AND DAW use this?
├─ YES → shared/ (rust or typescript)
└─ NO → Component-specific (pipeline/ or daw/)
```

**core/ vs Non-core:**
```
Does this code do ANY I/O or have ANY side effects?
├─ YES → NOT in core/ (use services/, commands/, db/)
└─ NO → core/ (pure logic only)
```

**commands/ vs services/:**
```
Is this a #[tauri::command] function?
├─ YES → commands/
└─ NO → services/ (if reusable business logic)
```

**Frontend Components:**
```
src/lib/components/
├── {feature}/          # Feature-specific components
└── common/             # Shared UI components
```

### Archetype-to-Directory Mapping

| Archetype | Rust Location | TypeScript Location |
|-----------|---------------|---------------------|
| Task-O-Matic | `src/main.rs`, `bin/` | `*.svelte`, `routes/` |
| Grown-up Script | `commands/`, `services/`, `db/repositories/` | `stores/` |
| Trusty Module | `core/`, `utils/` | `utils/`, `types/` |

---

## 🎯 KEY TAKEAWAYS

### Technology Decisions

1. **Rust + Tauri** - Performance-critical audio/MIDI with native UI
2. **Svelte + TypeScript** - Reactive, type-safe frontend
3. **PostgreSQL + Meilisearch** - Structured data + fast search
4. **Three Archetypes Pattern** - Clear code organization

### Critical Rules

1. **NO `.unwrap()` in production** - Always handle errors properly
2. **80%+ coverage for Trusty Modules** - Required, enforced
3. **Everything in `core/` is pure** - No I/O, no side effects
4. **Entry + implementation pattern** - For all Grown-up Scripts

### Development Process

1. **Classify first** - Use decision tree before coding
2. **Test while coding** - Don't write tests after
3. **Document while coding** - Don't write docs after
4. **Review checklist** - Before every commit

### Project Organization

1. **Shared-first** - Common code goes in `shared/`
2. **Component isolation** - Each app is self-contained
3. **Database centralization** - Single instance, shared schema
4. **Clear separation** - Three Archetypes determine location

---

## 📊 STATISTICS

- **Total lines in restructure.txt:** 16,371
- **Total conversation topics:** 100+
- **Major architectural sections:** 12
- **Code examples:** 50+
- **Makefile targets:** 40+
- **Dependencies:** 30+ (Rust crates + npm packages)
- **VS Code extensions:** 15+
- **Testing coverage requirement:** 80% for Trusty Modules
- **Estimated migration time:** 5 days (full-time)

---

## 🔗 RELATED DOCUMENTS

This summary references content that has been implemented in:

- `ARCHITECTURE-REFERENCE.md` - Three Archetypes pattern guide
- `PROJECT-STRUCTURE.md` - Complete directory structure
- `DEVELOPMENT-WORKFLOW.md` - 8-step implementation process
- `CRITICAL-REQUIREMENTS-ADDENDUM.md` - Mandatory code quality rules
- `PRE-MIGRATION-ALIGNMENT-RECOMMENDATION.md` - Pre-migration analysis
- `FINAL-FILE-SEPARATION.md` - Migration file mapping
- `CLAUDE.md` - AI assistant guidance

---

**This comprehensive summary captures all major topics discussed in restructure.txt and provides quick reference to the decisions, patterns, and processes defined during the architectural planning phase.**
