# Technology Stack Best Practices Guide

**MIDI Software Center - Comprehensive Architecture Recommendations**

Generated: 2025-12-01
Research Sources: Official documentation, industry best practices, successful production applications

---

## Table of Contents

1. [Tauri 2.7 Command Organization](#1-tauri-27-command-organization)
2. [Svelte 4.2 Store Patterns](#2-svelte-42-store-patterns)
3. [SQLx Repository Pattern](#3-sqlx-repository-pattern)
4. [Rust Workspace Management](#4-rust-workspace-management)
5. [Cross-Platform Desktop Considerations](#5-cross-platform-desktop-considerations)
6. [Integration Patterns](#6-integration-patterns)
7. [Implementation Roadmap](#7-implementation-roadmap)

---

## 1. Tauri 2.7 Command Organization

### 1.1 Module Structure (MUST HAVE)

**Current Industry Standard:**
```
src-tauri/
├── src/
│   ├── commands/           # All Tauri commands
│   │   ├── mod.rs         # Export all command modules
│   │   ├── database.rs    # Database operations
│   │   ├── pipeline.rs    # Pipeline commands
│   │   ├── daw.rs        # DAW commands
│   │   ├── mixer.rs      # Mixer commands
│   │   └── system.rs     # System info/utilities
│   ├── lib.rs            # Library initialization
│   └── main.rs           # App entry point
```

**Implementation Pattern:**
```rust
// src/commands/mod.rs
pub mod database;
pub mod pipeline;
pub mod daw;
pub mod mixer;
pub mod system;

// Re-export commonly used types
pub use database::*;
pub use pipeline::*;

// src/main.rs
mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::database::search_files,
            commands::pipeline::import_files,
            commands::daw::play_midi,
            // ... all commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Source:** [DEV Community - How to Keep Tauri Commands Organized](https://dev.to/n3rd/how-to-reasonably-keep-your-tauri-commands-organized-in-rust-2gmo)

### 1.2 Command Design Principles (RECOMMENDED)

**Keep Commands Focused:**
```rust
// ✅ GOOD: Single responsibility
#[tauri::command]
async fn search_files_by_bpm(
    db: State<'_, DatabasePool>,
    min_bpm: f32,
    max_bpm: f32,
) -> Result<Vec<MidiFile>, String> {
    // Single, well-defined task
}

// ❌ BAD: Multiple responsibilities
#[tauri::command]
async fn process_everything(/* ... */) -> Result<(), String> {
    // Importing, analyzing, searching, exporting all in one
}
```

**Async for Long-Running Operations:**
```rust
// ✅ GOOD: Prevents UI freezing
#[tauri::command]
async fn import_archive(
    path: String,
    progress: State<'_, ProgressTracker>,
) -> Result<ImportStats, String> {
    // Long operation with progress updates
}

// ❌ BAD: Blocking operation
#[tauri::command]
fn import_archive_sync(path: String) -> Result<ImportStats, String> {
    // Blocks UI thread
}
```

**Source:** [Tauri Tutorials - Command Fundamentals](https://tauritutorials.com/blog/tauri-command-fundamentals)

### 1.3 State Management Pattern (CRITICAL)

**Shared State Between Commands:**
```rust
// Define application state
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub progress: Arc<RwLock<ProgressState>>,
    pub config: Arc<RwLock<AppConfig>>,
}

// Register state in main.rs
fn main() {
    let app_state = AppState {
        db_pool: create_pool().await,
        progress: Arc::new(RwLock::new(ProgressState::default())),
        config: Arc::new(RwLock::new(load_config())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Access state in commands
#[tauri::command]
async fn get_progress(
    state: State<'_, AppState>
) -> Result<ProgressState, String> {
    let progress = state.progress.read().unwrap();
    Ok(progress.clone())
}
```

**Source:** [Tauri State Management Tutorial](https://gist.github.com/isaakengineer/5bdb6fcb141628b6865619bcd1c827fd)

### 1.4 Error Handling Pattern

**Return Results, Not Panics:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
async fn import_file(
    path: String,
    db: State<'_, DatabasePool>,
) -> Result<MidiFile, CommandError> {
    let file = std::fs::read(&path)
        .map_err(|_| CommandError::FileNotFound(path.clone()))?;

    let midi = parse_midi(&file)
        .map_err(|e| CommandError::InvalidInput(e.to_string()))?;

    let result = insert_file(&db, midi).await?;
    Ok(result)
}
```

### 1.5 Tauri 2.0 Permissions System (NEW)

**Capability-Based Security:**
```json
// src-tauri/capabilities/main.json
{
  "identifier": "main-capability",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": [
    "core:window:allow-create",
    "core:window:allow-close",
    "fs:allow-read-text-file",
    "fs:scope-read-files",
    "database:default"
  ],
  "scope": {
    "allow": [
      { "path": "$APPLOCALDATA/**" }
    ],
    "deny": [
      { "path": "$APPLOCALDATA/secrets.txt" }
    ]
  }
}
```

**Custom Permission Definition:**
```toml
# src-tauri/permissions/database.toml
[[permission]]
identifier = "database:read"
description = "Read access to database"
commands.allow = [
    "search_files",
    "get_file_by_id",
    "get_files_by_tag"
]

[[permission]]
identifier = "database:write"
description = "Write access to database"
commands.allow = [
    "import_files",
    "update_file",
    "delete_file"
]
```

**Source:** [Tauri 2.0 Security Documentation](https://v2.tauri.app/security/)

---

## 2. Svelte 4.2 Store Patterns

### 2.1 Store Organization (RECOMMENDED)

**Directory Structure:**
```
src/lib/stores/
├── index.ts              # Central export point
├── uiStore.ts           # UI state (theme, layout)
├── databaseStore.ts     # Database queries/results
├── pipelineStore.ts     # Pipeline progress/stats
├── playbackStore.ts     # DAW playback state
├── archiveStore.ts      # Archive extraction state
└── types.ts             # Shared store types
```

**Central Export Pattern:**
```typescript
// src/lib/stores/index.ts
export { uiStore } from './uiStore';
export { databaseStore } from './databaseStore';
export { pipelineStore } from './pipelineStore';
export { playbackStore } from './playbackStore';

// Usage in components
import { uiStore, databaseStore } from '$lib/stores';
```

### 2.2 Writable Store Pattern (MUST HAVE)

**Type-Safe Store Creation:**
```typescript
// src/lib/stores/databaseStore.ts
import { writable, derived, type Readable } from 'svelte/store';
import type { MidiFile, SearchFilters } from '$lib/types';

interface DatabaseState {
    files: MidiFile[];
    loading: boolean;
    error: string | null;
    filters: SearchFilters;
}

const initialState: DatabaseState = {
    files: [],
    loading: false,
    error: null,
    filters: {}
};

function createDatabaseStore() {
    const { subscribe, set, update } = writable<DatabaseState>(initialState);

    return {
        subscribe,

        setFiles: (files: MidiFile[]) => update(state => ({
            ...state,
            files,
            loading: false
        })),

        setLoading: (loading: boolean) => update(state => ({
            ...state,
            loading
        })),

        setError: (error: string | null) => update(state => ({
            ...state,
            error,
            loading: false
        })),

        updateFilters: (filters: Partial<SearchFilters>) => update(state => ({
            ...state,
            filters: { ...state.filters, ...filters }
        })),

        reset: () => set(initialState)
    };
}

export const databaseStore = createDatabaseStore();
```

**Source:** [Svelte Store State Management Tutorial](https://medium.com/codetodeploy/svelte-store-state-management-the-svelte-way-014035605464)

### 2.3 Derived Stores for Computed State (RECOMMENDED)

**Performance Optimization with Derived Stores:**
```typescript
import { derived } from 'svelte/store';
import { databaseStore } from './databaseStore';

// Filtered files based on current filters
export const filteredFiles = derived(
    databaseStore,
    $db => $db.files.filter(file => {
        if ($db.filters.bpmMin && file.bpm < $db.filters.bpmMin) return false;
        if ($db.filters.bpmMax && file.bpm > $db.filters.bpmMax) return false;
        if ($db.filters.key && file.key !== $db.filters.key) return false;
        return true;
    })
);

// File statistics
export const fileStats = derived(
    databaseStore,
    $db => ({
        total: $db.files.length,
        avgBpm: $db.files.reduce((sum, f) => sum + (f.bpm || 0), 0) / $db.files.length,
        keys: [...new Set($db.files.map(f => f.key))].sort()
    })
);

// Multiple store derivation
export const searchResults = derived(
    [databaseStore, uiStore],
    ([$db, $ui]) => ({
        files: $db.files,
        sortedBy: $ui.sortBy,
        viewMode: $ui.viewMode
    })
);
```

**Performance Benefit:** Derived stores reduce unnecessary re-renders by up to 35% in complex interfaces.

**Source:** [MoldStud - Store Optimization in Svelte](https://moldstud.com/articles/p-from-zero-to-hero-mastering-store-optimization-in-svelte-applications)

### 2.4 Async Store Pattern (CRITICAL)

**For Database Queries and API Calls:**
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

interface AsyncState<T> {
    data: T | null;
    loading: boolean;
    error: string | null;
}

function createAsyncStore<T>(initialData: T | null = null) {
    const { subscribe, set, update } = writable<AsyncState<T>>({
        data: initialData,
        loading: false,
        error: null
    });

    return {
        subscribe,

        async load(command: string, args: any = {}) {
            update(state => ({ ...state, loading: true, error: null }));

            try {
                const data = await invoke<T>(command, args);
                set({ data, loading: false, error: null });
                return data;
            } catch (error) {
                const errorMsg = error instanceof Error ? error.message : 'Unknown error';
                set({ data: null, loading: false, error: errorMsg });
                throw error;
            }
        },

        reset: () => set({ data: initialData, loading: false, error: null })
    };
}

// Usage
export const searchStore = createAsyncStore<MidiFile[]>([]);

// In component
searchStore.load('search_files', {
    tags: ['drums'],
    bpmMin: 120,
    bpmMax: 140
});
```

### 2.5 Context API for Component Trees (OPTIONAL)

**When to Use:** For deeply nested components without prop drilling.

```typescript
// src/lib/contexts/daw.ts
import { setContext, getContext } from 'svelte';
import { writable } from 'svelte/store';

const DAW_CONTEXT_KEY = Symbol('daw');

export interface DAWContext {
    playback: ReturnType<typeof writable<PlaybackState>>;
    timeline: ReturnType<typeof writable<TimelineState>>;
}

export function setDAWContext() {
    const context: DAWContext = {
        playback: writable({ playing: false, position: 0 }),
        timeline: writable({ zoom: 1, scrollX: 0 })
    };

    setContext(DAW_CONTEXT_KEY, context);
    return context;
}

export function getDAWContext(): DAWContext {
    return getContext(DAW_CONTEXT_KEY);
}
```

```svelte
<!-- ParentComponent.svelte -->
<script lang="ts">
    import { setDAWContext } from '$lib/contexts/daw';
    const daw = setDAWContext();
</script>

<!-- ChildComponent.svelte -->
<script lang="ts">
    import { getDAWContext } from '$lib/contexts/daw';
    const { playback, timeline } = getDAWContext();
</script>

<button on:click={() => $playback.playing = !$playback.playing}>
    {$playback.playing ? 'Pause' : 'Play'}
</button>
```

**Source:** [MoldStud - Context API in Svelte](https://moldstud.com/articles/p-mastering-state-management-how-to-use-context-api-in-svelte-components)

### 2.6 Immutability Best Practice (MUST HAVE)

**Triggers Reactivity Correctly:**
```typescript
// ✅ GOOD: Immutable updates
update(state => ({
    ...state,
    files: [...state.files, newFile]
}));

update(state => ({
    ...state,
    files: state.files.map(f =>
        f.id === id ? { ...f, selected: true } : f
    )
}));

// ❌ BAD: Mutating state directly
update(state => {
    state.files.push(newFile); // Doesn't trigger reactivity!
    return state;
});
```

**Performance Gain:** Reduces wasted renders by up to 35% in complex interfaces.

**Source:** [Kodeblog - State Management Patterns in Svelte](https://www.kodeblog.com/state-management-patterns-in-svelte-effective-state-design-and-patterns/)

---

## 3. SQLx Repository Pattern

### 3.1 Repository Structure (RECOMMENDED)

**Clean Separation of Concerns:**
```
src/db/
├── mod.rs                 # Database module exports
├── models.rs             # Domain models
└── repositories/
    ├── mod.rs            # Repository exports
    ├── file_repository.rs
    ├── tag_repository.rs
    ├── metadata_repository.rs
    └── search_repository.rs
```

### 3.2 Repository Implementation (BEST PRACTICE)

**Generic Repository Base:**
```rust
// src/db/repositories/mod.rs
use sqlx::PgPool;

pub trait Repository {
    type Entity;
    type Id;

    async fn find_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Self::Entity>, sqlx::Error>;
    async fn insert(&self, entity: &Self::Entity) -> Result<Self::Id, sqlx::Error>;
    async fn update(&self, entity: &Self::Entity) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: Self::Id) -> Result<(), sqlx::Error>;
}

// Concrete implementation
pub struct FileRepository {
    pool: PgPool,
}

impl FileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Compiled-time checked queries
    pub async fn find_by_id(&self, id: i64) -> Result<Option<MidiFile>, sqlx::Error> {
        sqlx::query_as!(
            MidiFile,
            r#"
            SELECT id, filename, filepath, hash, bpm, key_signature, created_at
            FROM files
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_bpm_range(
        &self,
        min_bpm: f32,
        max_bpm: f32
    ) -> Result<Vec<MidiFile>, sqlx::Error> {
        sqlx::query_as!(
            MidiFile,
            r#"
            SELECT f.id, f.filename, f.filepath, f.hash, m.bpm, m.key_signature, f.created_at
            FROM files f
            JOIN musical_metadata m ON f.id = m.file_id
            WHERE m.bpm BETWEEN $1 AND $2
            ORDER BY m.bpm
            "#,
            min_bpm,
            max_bpm
        )
        .fetch_all(&self.pool)
        .await
    }
}
```

**Source:** [StudyRaid - Implementing Repository Patterns with SQLx](https://app.studyraid.com/en/read/14938/515218/implementing-repository-patterns-with-sqlx)

### 3.3 Domain Model Pattern (CRITICAL)

**Repositories Should Deal with Domain Models, Not DB Entities:**
```rust
// Domain model (what your app works with)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiFile {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub hash: String,
    pub metadata: MusicalMetadata,  // Aggregated from multiple tables
    pub tags: Vec<Tag>,             // Joined data
}

// Database entity (internal to repository)
#[derive(sqlx::FromRow)]
struct FileRow {
    id: i64,
    filename: String,
    filepath: String,
    hash: String,
}

#[derive(sqlx::FromRow)]
struct MetadataRow {
    file_id: i64,
    bpm: Option<f32>,
    key_signature: Option<String>,
}

impl FileRepository {
    pub async fn find_with_metadata(&self, id: i64) -> Result<Option<MidiFile>, sqlx::Error> {
        // Join multiple tables
        let row = sqlx::query!(
            r#"
            SELECT
                f.id, f.filename, f.filepath, f.hash,
                m.bpm, m.key_signature, m.duration,
                ARRAY_AGG(t.name) as "tags!"
            FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            LEFT JOIN tags t ON ft.tag_id = t.id
            WHERE f.id = $1
            GROUP BY f.id, m.file_id
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        // Map to domain model
        Ok(row.map(|r| MidiFile {
            id: r.id,
            filename: r.filename,
            filepath: r.filepath,
            hash: r.hash,
            metadata: MusicalMetadata {
                bpm: r.bpm,
                key_signature: r.key_signature,
                duration: r.duration,
            },
            tags: r.tags.into_iter().map(|name| Tag { name }).collect(),
        }))
    }
}
```

**Source:** [Code Review Stack Exchange - Repository Pattern in Rust](https://codereview.stackexchange.com/questions/156123/repository-pattern-in-rust)

### 3.4 Transaction Handling (MUST HAVE)

**Atomic Multi-Step Operations:**
```rust
impl FileRepository {
    pub async fn import_with_tags(
        &self,
        file: &MidiFile,
        tags: &[String]
    ) -> Result<i64, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Insert file
        let file_id = sqlx::query!(
            "INSERT INTO files (filename, filepath, hash) VALUES ($1, $2, $3) RETURNING id",
            file.filename,
            file.filepath,
            file.hash
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        // Insert metadata
        sqlx::query!(
            "INSERT INTO musical_metadata (file_id, bpm, key_signature) VALUES ($1, $2, $3)",
            file_id,
            file.metadata.bpm,
            file.metadata.key_signature
        )
        .execute(&mut *tx)
        .await?;

        // Insert tags
        for tag_name in tags {
            let tag_id = sqlx::query!(
                "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
                tag_name
            )
            .fetch_one(&mut *tx)
            .await?
            .id;

            sqlx::query!(
                "INSERT INTO file_tags (file_id, tag_id) VALUES ($1, $2)",
                file_id,
                tag_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(file_id)
    }
}
```

### 3.5 Connection Pool Best Practices (CRITICAL)

**Optimal Pool Configuration:**
```rust
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(34)              // Based on your config
        .min_connections(5)               // Keep warm connections
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}
```

**Connection Pool Management:**
- Max connections: Set based on `(CPU cores * 2) + disk I/O` or your current 34
- Min connections: Keep 5-10 warm to reduce connection overhead
- Idle timeout: Release connections after 10 minutes of inactivity
- Max lifetime: Rotate connections every 30 minutes to prevent stale connections

**Source:** [Generalist Programmer - SQLx Complete Guide](https://generalistprogrammer.com/tutorials/sqlx-rust-crate-guide)

### 3.6 Named Parameters for Safety (MUST HAVE)

```rust
// ✅ GOOD: Named parameters prevent SQL injection
sqlx::query!(
    "SELECT * FROM files WHERE bpm BETWEEN $1 AND $2",
    min_bpm,
    max_bpm
)

// ❌ BAD: String formatting is vulnerable
let query = format!("SELECT * FROM files WHERE bpm BETWEEN {} AND {}", min_bpm, max_bpm);
```

**Source:** [SQLx Best Practices Guide](https://sqlx.dev/article/Best_Practices_for_Writing_SQLX_Code.html)

---

## 4. Rust Workspace Management

### 4.1 Workspace Structure (RECOMMENDED)

**Current Structure:**
```toml
# Root Cargo.toml
[workspace]
resolver = "2"  # Use "3" for Rust 2024 edition (1.84+)
members = [
    "pipeline/src-tauri",
    "daw/src-tauri",
    "app/src-tauri",
    "shared/rust"
]

[workspace.dependencies]
# Shared dependencies with consistent versions
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"

# Tauri dependencies
tauri = { version = "2.7", features = ["macos-private-api"] }
tauri-build = "2.7"

# Performance crates
mimalloc = "0.1.48"
parking_lot = "0.12"
ahash = "0.8.12"
dashmap = "6.1.0"
flume = "0.11"

# MIDI processing
midly = "0.5"
midir = "0.10"

# Workspace members
midi-pipeline = { path = "pipeline/src-tauri" }
midi-daw = { path = "daw/src-tauri" }
midi-shared = { path = "shared/rust" }
```

**Member Crate Dependencies:**
```toml
# pipeline/src-tauri/Cargo.toml
[dependencies]
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
tauri = { workspace = true }

# Add features to workspace dependencies
anyhow = { workspace = true, features = ["backtrace"] }

# Workspace member dependency
midi-shared = { workspace = true }
```

**Source:** [Vivek Shukla - How I Use Cargo Workspace](https://vivekshuk.la/tech/2025/use-cargo-workspace-rust/)

### 4.2 Workspace Benefits (KEY ADVANTAGES)

1. **Single Cargo.lock** - All crates use same dependency versions
2. **Shared Target Directory** - Faster builds, less disk space
3. **Cross-Crate Testing** - `cargo test --workspace`
4. **Unified Commands** - `cargo check --workspace`, `cargo clippy --workspace`

**Source:** [Rust Book - Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)

### 4.3 Shared Library Pattern (CRITICAL)

**What Belongs in Shared:**
```
shared/rust/src/
├── core/
│   ├── midi/
│   │   ├── parser.rs      # MIDI parsing logic
│   │   └── mod.rs
│   ├── analysis/
│   │   ├── bpm_detector.rs
│   │   ├── key_detector.rs
│   │   └── auto_tagger.rs
│   └── mod.rs
├── db/
│   ├── models.rs          # Database models
│   ├── repositories/      # Repository traits/implementations
│   └── mod.rs
└── lib.rs
```

**What DOESN'T Belong in Shared:**
- ❌ Tauri commands (pipeline/daw specific)
- ❌ UI components (frontend specific)
- ❌ Binary executables (app-specific)
- ❌ Application configuration (config belongs in each app)

### 4.4 Dependency Management Strategy (RECOMMENDED)

**Minimize Tight Coupling:**
```rust
// ✅ GOOD: Trait-based abstraction
// shared/rust/src/db/repositories/mod.rs
pub trait FileRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<MidiFile>>;
    async fn insert(&self, file: &MidiFile) -> Result<i64>;
}

// pipeline/src-tauri/src/db/repositories/postgres_file_repository.rs
pub struct PostgresFileRepository {
    pool: PgPool,
}

impl FileRepository for PostgresFileRepository {
    async fn find_by_id(&self, id: i64) -> Result<Option<MidiFile>> {
        // PostgreSQL-specific implementation
    }
}

// ❌ BAD: Direct dependency on concrete type
pub fn process_file(repo: &PostgresFileRepository) { }
```

**Source:** [Medium - Mastering Rust Workspaces](https://medium.com/@nishantspatil0408/mastering-rust-workspaces-from-development-to-production-a57ca9545309)

### 4.5 Workspace Publishing (2025 UPDATE)

**Multi-Package Publishing (Cargo 1.90+):**
```bash
# Publish entire workspace
cargo publish --workspace

# Publish specific packages
cargo publish --package midi-shared --package midi-pipeline

# Exclude packages
cargo publish --exclude midi-daw
```

**Available in Stable Rust:** September 2025 (Cargo 1.90)

**Source:** [Tweag - Publish All Your Crates](https://www.tweag.io/blog/2025-07-10-cargo-package-workspace/)

### 4.6 Profile Optimization (CRITICAL)

**Development vs Release Profiles:**
```toml
# Root Cargo.toml
[profile.dev]
opt-level = 0           # No optimization for faster builds

[profile.dev.package."*"]
opt-level = 3           # Optimize dependencies for better dev performance

[profile.release]
opt-level = 3           # Maximum optimization
lto = "thin"            # Link-time optimization (thin is faster than "fat")
codegen-units = 1       # Better optimization, slower build
strip = true            # Remove debug symbols
panic = "abort"         # Smaller binaries

# Custom profile for profiling
[profile.bench]
inherits = "release"
debug = true            # Keep debug info for profiling tools
```

**Your Current Settings (Good!):**
- Dev: O0 code, O3 deps ✅
- Release: O3 + thin LTO + strip ✅

---

## 5. Cross-Platform Desktop Considerations

### 5.1 Platform-Specific Code (RECOMMENDED)

**Conditional Compilation:**
```rust
#[cfg(target_os = "macos")]
fn setup_macos_specific() {
    // macOS-specific setup
}

#[cfg(target_os = "windows")]
fn setup_windows_specific() {
    // Windows-specific setup
}

#[cfg(target_os = "linux")]
fn setup_linux_specific() {
    // Linux-specific setup
}

fn main() {
    #[cfg(target_os = "macos")]
    setup_macos_specific();

    #[cfg(target_os = "windows")]
    setup_windows_specific();

    #[cfg(target_os = "linux")]
    setup_linux_specific();
}
```

### 5.2 File Path Handling (CRITICAL)

**Cross-Platform Paths:**
```rust
use std::path::PathBuf;

// ✅ GOOD: Platform-agnostic
let path = PathBuf::from("/home/user/midi-library");
let file = path.join("files").join("song.mid");

// ❌ BAD: Hardcoded separators
let path = "/home/user/midi-library/files/song.mid";  // Breaks on Windows
```

### 5.3 Security Best Practices (MUST HAVE)

**Granular Permissions:**
```json
{
  "permissions": [
    "fs:allow-read-text-file",
    "fs:scope-read-files"
  ],
  "scope": {
    "allow": [
      { "path": "$APPLOCALDATA/midi-library/**" }
    ],
    "deny": [
      { "path": "$HOME/.ssh/**" },
      { "path": "$HOME/.aws/**" }
    ]
  }
}
```

**Source:** [Tauri 2.0 Security Guide](https://v2.tauri.app/security/)

### 5.4 Environment Variables (RECOMMENDED)

```rust
// src-tauri/build.rs
fn main() {
    // Platform-specific environment variables
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-env=MIDI_LIBRARY_PATH=C:\\Users\\Public\\MIDI");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-env=MIDI_LIBRARY_PATH=/Users/Shared/MIDI");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-env=MIDI_LIBRARY_PATH=/usr/share/midi");

    tauri_build::build()
}
```

### 5.5 CI/CD for Multiple Platforms (RECOMMENDED)

**GitHub Actions Workflow:**
```yaml
name: Build Multi-Platform

on:
  push:
    branches: [main]

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies (Ubuntu only)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev

      - name: Build
        run: |
          cd app
          pnpm install
          pnpm tauri build

      - uses: tauri-apps/tauri-action@v0
        with:
          tagName: v__VERSION__
          releaseName: 'MIDI Software Center v__VERSION__'
```

**Source:** [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)

---

## 6. Integration Patterns

### 6.1 Tauri ↔ Svelte Communication

**Frontend (TypeScript):**
```typescript
// src/lib/api.ts
import { invoke } from '@tauri-apps/api/core';
import type { MidiFile, SearchFilters } from './types';

export const api = {
    async searchFiles(filters: SearchFilters): Promise<MidiFile[]> {
        return await invoke<MidiFile[]>('search_files', { filters });
    },

    async importFiles(paths: string[]): Promise<{ imported: number, failed: number }> {
        return await invoke('import_files', { paths });
    },

    async getProgress(): Promise<{ current: number, total: number }> {
        return await invoke('get_progress');
    }
};
```

**Backend (Rust):**
```rust
#[tauri::command]
async fn search_files(
    filters: SearchFilters,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    let repo = FileRepository::new(state.db_pool.clone());
    repo.search(filters)
        .await
        .map_err(|e| e.to_string())
}
```

### 6.2 Event-Based Updates (RECOMMENDED)

**Progress Updates:**
```rust
use tauri::Manager;

#[tauri::command]
async fn import_files(
    paths: Vec<String>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportStats, String> {
    for (i, path) in paths.iter().enumerate() {
        // Process file
        let file = import_file(path).await?;

        // Emit progress event
        app.emit("import-progress", ImportProgress {
            current: i + 1,
            total: paths.len(),
            filename: file.filename.clone(),
        }).map_err(|e| e.to_string())?;
    }

    Ok(ImportStats { imported: paths.len(), failed: 0 })
}
```

```typescript
// Frontend listening
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<ImportProgress>('import-progress', (event) => {
    console.log(`Progress: ${event.payload.current}/${event.payload.total}`);
    progressStore.set(event.payload);
});
```

### 6.3 Error Propagation Pattern

**Frontend Error Handling:**
```typescript
// src/lib/stores/databaseStore.ts
export const databaseStore = {
    async search(filters: SearchFilters) {
        this.setLoading(true);

        try {
            const files = await api.searchFiles(filters);
            this.setFiles(files);
            return files;
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Unknown error';
            this.setError(message);

            // Optional: Show toast notification
            toast.error(`Search failed: ${message}`);

            throw error;
        }
    }
};
```

---

## 7. Implementation Roadmap

### Phase 1: Command Organization (1-2 days)

**Priority: HIGH - Foundation for all improvements**

1. Create `commands/` module structure
2. Split existing commands into logical modules:
   - `database.rs` - Search, get file, statistics
   - `pipeline.rs` - Import, analyze, split
   - `daw.rs` - Playback, MIDI I/O, sequencer
   - `mixer.rs` - Audio mixing, effects
   - `system.rs` - System info, file dialogs
3. Update `main.rs` to use new structure
4. Add proper error types with `thiserror`

**Files to Modify:**
- `/home/dojevou/projects/midi-software-center/app/src-tauri/src/main.rs`
- `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/commands/*.rs`
- `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/*.rs`

### Phase 2: Store Refactoring (2-3 days)

**Priority: MEDIUM - Improves frontend maintainability**

1. Create store directory structure
2. Implement typed store factories
3. Add derived stores for computed values
4. Implement async store pattern for API calls
5. Add proper TypeScript types

**Files to Modify:**
- `/home/dojevou/projects/midi-software-center/app/src/lib/stores/*.ts`

### Phase 3: Repository Pattern (3-4 days)

**Priority: MEDIUM - Already partially implemented**

1. Review existing repository implementations
2. Add missing repository methods
3. Implement transaction wrappers
4. Add comprehensive error handling
5. Create repository integration tests

**Files to Modify:**
- `/home/dojevou/projects/midi-software-center/shared/rust/src/db/repositories/*.rs`
- `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/*.rs`

### Phase 4: Workspace Optimization (1-2 days)

**Priority: LOW - Nice to have**

1. Move common dependencies to workspace level
2. Update member crates to use `workspace = true`
3. Review and optimize build profiles
4. Update to resolver "3" when stable (Rust 1.84+)

**Files to Modify:**
- `/home/dojevou/projects/midi-software-center/Cargo.toml`
- All member `Cargo.toml` files

### Phase 5: Security & Permissions (2-3 days)

**Priority: HIGH - Production requirement**

1. Audit current command permissions
2. Create capability files for each window
3. Define granular permissions per command
4. Implement scope restrictions
5. Test on all platforms

**Files to Create:**
- `/home/dojevou/projects/midi-software-center/app/src-tauri/capabilities/main.json`
- `/home/dojevou/projects/midi-software-center/app/src-tauri/permissions/*.toml`

### Phase 6: Cross-Platform Testing (Ongoing)

**Priority: MEDIUM - CI/CD integration**

1. Set up GitHub Actions for multi-platform builds
2. Add platform-specific tests
3. Test file path handling on Windows/Mac/Linux
4. Validate database connections across platforms

---

## Summary of Priorities

### MUST HAVE (Immediate Action)
1. ✅ Command organization into modules
2. ✅ Proper error handling with `thiserror`
3. ✅ Type-safe stores with TypeScript
4. ✅ Named SQL parameters (already using)
5. ✅ Immutable store updates
6. ✅ Transaction handling in repositories

### RECOMMENDED (Next Sprint)
1. 🔶 Derived stores for performance
2. 🔶 Async store pattern
3. 🔶 Repository trait abstraction
4. 🔶 Workspace dependency consolidation
5. 🔶 CI/CD for multi-platform builds

### OPTIONAL (Future Enhancement)
1. 🔷 Context API for complex component trees
2. 🔷 Resolver version 3 (when Rust 1.84 stable)
3. 🔷 Custom store implementations
4. 🔷 Advanced profiling and optimization

---

## Sources

### Official Documentation
- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Svelte Documentation](https://svelte.dev/docs/svelte/stores)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Rust Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)

### Best Practices Articles
- [DEV Community - Tauri Command Organization](https://dev.to/n3rd/how-to-reasonably-keep-your-tauri-commands-organized-in-rust-2gmo)
- [Loopwerk - Svelte 5 Store Migration](https://www.loopwerk.io/articles/2025/svelte-5-stores/)
- [SvelteKit State Management](https://kit.svelte.dev/docs/state-management)
- [Generalist Programmer - SQLx Complete Guide](https://generalistprogrammer.com/tutorials/sqlx-rust-crate-guide)
- [Medium - Rust Workspace Guide](https://medium.com/@aleksej.gudkov/rust-workspace-example-a-guide-to-managing-multi-crate-projects-82d318409260)

### GitHub Resources
- [Awesome Tauri](https://github.com/tauri-apps/awesome-tauri)
- [Tauri Architecture](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md)
- [SQLx Repository](https://github.com/launchbadge/sqlx)

---

**Document Version:** 1.0
**Last Updated:** 2025-12-01
**Research Depth:** Official docs + 15 authoritative sources
**Applicability:** Immediate - All recommendations can be implemented today
