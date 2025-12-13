# AppState, AppHandle, and WindowManager Initialization Architecture

## Executive Summary

This document provides comprehensive references for initializing and managing application state in the MIDI Software Center project. It covers AppState struct definitions, AppHandle usage patterns, error handling, and Arc<Mutex<T>> patterns used for shared state management.

---

## 1. AppState Struct Definitions

### 1.1 Pipeline AppState

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/lib.rs` (Lines 25-27)

```rust
/// Application state shared across all Tauri commands
pub struct AppState {
    pub database: Database,
}
```

**Characteristics:**
- Holds single `Database` field
- Wrapped in `Database` which manages `Arc<RwLock<PgPool>>`
- Shared via `tauri::Builder::default().manage(state)`
- Used in Pipeline application for batch import, analysis, and archive operations

**Initialization Pattern (Pipeline main.rs:45-48):**
```rust
let database = match Database::new(&database_url).await {
    Ok(db) => {
        info!("Database connection established");
        db
    }
    Err(e) => {
        info!("Database initialization deferred (will retry on first command): {}", e);
        // Retry once
        Database::new(&database_url).await.map_err(|retry_err| {
            format!("Failed to create database instance after retry: {}", retry_err)
        })?
    }
};

let state = AppState {
    database,
};
```

---

### 1.2 DAW AppState

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/mod.rs` (Lines 16-18)

```rust
/// Shared application state across all commands
///
/// Contains database pool for read-only access to MIDI file metadata.
pub struct AppState {
    pub db_pool: Option<sqlx::PgPool>,
}
```

**Characteristics:**
- Holds optional `sqlx::PgPool` for database connectivity
- Database connection is optional (graceful degradation)
- Separate Arc<MidiManager> and Arc<SequencerEngine> managed independently via `.manage()`
- Used for search, analysis, and file metadata queries

**Initialization Pattern (DAW main.rs:56-59):**
```rust
let state = AppState {
    db_pool,
};
```

---

## 2. Database Wrapper with Arc<RwLock<T>>

### 2.1 Database Struct Definition

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/database/mod.rs` (Lines 75-79)

```rust
pub struct Database {
    pool: Arc<RwLock<PgPool>>,
    database_url: String,
    reconnect_attempts: Arc<RwLock<u32>>,
}
```

**Arc<RwLock<>> Pattern Used:**
1. **`Arc<RwLock<PgPool>>`** - Connection pool with reader-writer lock
   - Multiple commands can read simultaneously
   - Only one writer at a time (reconnection)
   - Cheap clone (just Arc increment)

2. **`Arc<RwLock<u32>>`** - Reconnection attempt counter
   - Shared mutable state across reconnection attempts
   - Protected by RwLock for thread-safe access

### 2.2 Database Initialization

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/database/mod.rs` (Lines 120-187)

**Key Initialization Steps:**

1. **Dynamic pool sizing** (Lines 124-129):
```rust
let (concurrency, pool_size, batch_size) = calculate_all_settings();
println!("🚀 Dynamic pool sizing detected:");
println!("   Concurrency:  {} workers", concurrency);
println!("   Pool Size:    {} connections (auto-tuned)", pool_size);
println!("   Batch Size:   {} records", batch_size);
```

2. **Connection options** (Lines 132-140):
```rust
let mut connect_options = PgConnectOptions::from_str(database_url)?;
connect_options = connect_options.statement_cache_capacity(100);
connect_options = connect_options.application_name("midi-library-pipeline");
```

3. **Pool configuration** (Lines 146-172):
```rust
let pool = PgPoolOptions::new()
    .max_connections(pool_size as u32)
    .min_connections(min_connections)
    .acquire_timeout(Duration::from_secs(10))
    .max_lifetime(Duration::from_secs(1800))
    .idle_timeout(Duration::from_secs(300))
    .test_before_acquire(true)
    .connect_with(connect_options)
    .await?;
```

4. **Arc<RwLock<>> wrapping** (Lines 182-186):
```rust
Ok(Self {
    pool: Arc::new(RwLock::new(pool)),
    database_url: database_url.to_string(),
    reconnect_attempts: Arc::new(RwLock::new(0)),
})
```

**Pool Configuration Details:**
- **Max connections:** Dynamic (auto-tuned, 20-200)
- **Min connections:** 20% of max, minimum 5
- **Acquire timeout:** 10 seconds
- **Max lifetime:** 1800 seconds (30 minutes)
- **Idle timeout:** 300 seconds (5 minutes)
- **Statement cache:** 100 statements per connection
- **Test before acquire:** true (validates connection health)

---

## 3. MidiManager with Arc<Mutex<T>>

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/midi/manager.rs` (Lines 19-31)

```rust
/// Thread-safe MIDI connection manager
///
/// Manages MIDI output connections with thread-safe access.
/// Uses Arc<Mutex<>> for safe concurrent access from multiple threads.
pub struct MidiManager {
    connection: Arc<Mutex<Option<MidiOutputConnection>>>,
    current_device: Arc<Mutex<Option<String>>>,
}

impl MidiManager {
    /// Create a new MIDI manager
    pub fn new() -> Self {
        info!("Creating MIDI manager");
        Self {
            connection: Arc::new(Mutex::new(None)),
            current_device: Arc::new(Mutex::new(None)),
        }
    }
}
```

**Arc<Mutex<>> Pattern Used:**
1. **`Arc<Mutex<Option<MidiOutputConnection>>>`** - MIDI connection state
   - Ensures only one thread modifies connection at a time
   - Option allows None when disconnected
   - Mutex used (not RwLock) because writes are frequent

2. **`Arc<Mutex<Option<String>>>`** - Current device name
   - Stores name of connected device
   - Accessible from multiple async tasks

**Sharing Pattern (DAW main.rs:45):**
```rust
let midi_manager = Arc::new(MidiManager::new());
// ...
.manage(midi_manager)
```

---

## 4. SequencerEngine with Multiple Arc<RwLock<T>>

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/sequencer/engine.rs` (Lines 27-46)

```rust
pub struct SequencerEngine {
    track_manager: Arc<TrackManager>,
    scheduler: Arc<EventScheduler>,
    midi_manager: Arc<MidiManager>,

    // Playback state
    state: Arc<RwLock<PlaybackState>>,
    current_tick: Arc<RwLock<u64>>,
    start_time: Arc<Mutex<Option<Instant>>>,

    // Transport settings
    bpm: Arc<RwLock<f32>>,
    ticks_per_quarter: u16,
    beats_per_bar: u8,

    // Playback control
    loop_enabled: Arc<RwLock<bool>>,
    loop_start: Arc<RwLock<u64>>,
    loop_end: Arc<RwLock<u64>>,
}
```

**Arc<RwLock<T>> Pattern:**
- **Playback State** (`Arc<RwLock<PlaybackState>>`): Multiple tasks read state, only one writer during state changes
- **Current Tick** (`Arc<RwLock<u64>>`): Frequently read for position queries, written by playback task
- **BPM** (`Arc<RwLock<f32>>`): Read during timing calculations, written when tempo changes
- **Loop Settings** (`Arc<RwLock<bool>>`, `Arc<RwLock<u64>>`): Read-heavy, occasional writes

**Arc<Mutex<T>> Pattern:**
- **Start Time** (`Arc<Mutex<Option<Instant>>>`): Write-heavy timing state, needs mutual exclusion

### Usage Example (SequencerEngine lines 83-105):
```rust
pub async fn start(&self) -> Result<(), String> {
    let mut state = self.state.write().await;
    
    if *state == PlaybackState::Playing {
        return Ok(());
    }
    
    if !self.midi_manager.is_connected().await {
        return Err("MIDI device not connected".to_string());
    }
    
    info!("Starting sequencer playback");
    
    *state = PlaybackState::Playing;
    let mut start_time = self.start_time.lock().await;
    *start_time = Some(Instant::now());
    
    // Spawn playback task
    self.spawn_playback_task().await;
    
    Ok(())
}
```

---

## 5. Error Handling Patterns

### 5.1 Error Type Definition

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/error.rs` (Lines 35-54)

```rust
#[derive(Debug)]
pub enum AppError {
    /// Database operation failed
    DatabaseError(sqlx::Error),

    /// Requested resource not found
    NotFound(String),

    /// Input validation failed
    ValidationError(String),

    /// File I/O operation failed
    IOError(std::io::Error),

    /// MIDI parsing or analysis error
    MidiError(String),

    /// Generic application error
    GeneralError(String),
}
```

### 5.2 Error Conversion Traits

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/error.rs` (Lines 92-143)

```rust
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IOError(error)
    }
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
```

### 5.3 Result Type Aliases

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/error.rs` (Lines 215, 232)

```rust
/// Standard Result type for operations
pub type AppResult<T> = Result<T, AppError>;

/// Tauri-compatible Result type (with String error)
pub type TauriResult<T> = Result<T, String>;
```

### 5.4 Error Helper Functions

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/error.rs` (Lines 150-193)

```rust
impl AppError {
    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} not found", resource))
    }

    pub fn validation(message: &str) -> Self {
        AppError::ValidationError(message.to_string())
    }

    pub fn midi(message: &str) -> Self {
        AppError::MidiError(message.to_string())
    }

    pub fn general(message: &str) -> Self {
        AppError::GeneralError(message.to_string())
    }
}
```

---

## 6. Tauri State Injection in Commands

### 6.1 Pipeline Command Pattern

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/commands/files.rs` (Lines 14-32)

```rust
use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn test_db_connection(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.database.pool().await;
    
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| format!("Database connection test failed: {}", e))?;
    
    Ok(())
}
```

### 6.2 DAW Command with Arc<T> State

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/midi.rs` (Lines 15-29)

```rust
use tauri::State;
use std::sync::Arc;
use crate::midi::MidiManager;

#[tauri::command]
pub async fn midi_list_devices(
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<Vec<MidiDevice>, String> {
    midi_manager.list_devices()
}

#[tauri::command]
pub async fn midi_connect(
    device_name: String,
    midi_manager: State<'_, Arc<MidiManager>>,
) -> Result<(), String> {
    midi_manager.connect(&device_name).await
}
```

### 6.3 SequencerEngine State Injection

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/sequencer.rs` (Lines 15-28)

```rust
use tauri::State;
use std::sync::Arc;
use crate::sequencer::SequencerEngine;

#[tauri::command]
pub async fn start_sequencer(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    engine.start().await
}

#[tauri::command]
pub async fn stop_sequencer(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    engine.stop().await;
    Ok(())
}
```

---

## 7. AppHandle Usage in Setup

### 7.1 Pipeline Setup (Minimal)

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/main.rs` (Lines 112-115)

```rust
.setup(|_app| {
    info!("Application setup complete");
    Ok(())
})
```

**Current Pattern:**
- App handle (`_app`) is ignored
- Only logs setup completion
- No window-specific configuration

### 7.2 DAW Setup (No Custom Setup)

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/main.rs` (Lines 62-109)

- No `.setup()` closure defined
- All initialization happens before `tauri::Builder`
- MIDI manager and SequencerEngine created before `.manage()`

---

## 8. WindowManager Initialization (Pipeline Specific)

### 8.1 WindowManager Structure

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/windows/manager.rs` (Lines 11-32)

```rust
pub struct WindowManager {
    state: WindowState,
    layout_storage: Option<LayoutStorage>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        WindowManager {
            state: WindowState::new(),
            layout_storage: None,
        }
    }

    /// Create with layout persistence
    pub fn with_storage(layout_dir: std::path::PathBuf) -> Result<Self, String> {
        let storage = LayoutStorage::new(layout_dir)?;
        Ok(WindowManager {
            state: WindowState::new(),
            layout_storage: Some(storage),
        })
    }
}
```

### 8.2 WindowManager Methods

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/windows/manager.rs` (Lines 36-100)

**Core Operations:**
- `register_window(&mut self, window: WindowInfo)`
- `unregister_window(&mut self, label: &str) -> Result<(), String>`
- `get_window(&self, label: &str) -> Option<WindowInfo>`
- `show_window(&mut self, label: &str) -> Result<(), String>`
- `hide_window(&mut self, label: &str) -> Result<(), String>`
- `toggle_window(&mut self, label: &str) -> Result<(), String>`
- `set_position(&mut self, label: &str, position: Position) -> Result<(), String>`

### 8.3 Initialization Pattern for WindowManager

**Expected Initialization Pattern (similar to MidiManager):**

1. Create WindowManager instance
2. Wrap in Arc for shared ownership
3. Add to AppState or manage separately via `.manage()`
4. Register windows in `.setup()` closure via AppHandle

**Example Implementation Pattern:**
```rust
// In main.rs
let window_manager = Arc::new(Mutex::new(WindowManager::new()));

tauri::Builder::default()
    .manage(window_manager)
    .setup(|app| {
        let window_manager = app.state::<Arc<Mutex<WindowManager>>>();
        let mut wm = window_manager.blocking_lock();
        
        // Register initial windows
        wm.register_window(WindowInfo::new(...));
        
        Ok(())
    })
```

---

## 9. Current Initialization Patterns Summary

### Pipeline Application Flow

```
main.rs
├── Load .env
├── init_logging()
├── Database::new(database_url) [with retry]
│   └── Creates Arc<RwLock<PgPool>>
├── Create AppState { database }
├── tauri::Builder::default()
│   ├── .manage(state)
│   ├── .invoke_handler([commands...])
│   ├── .setup(|_app| { Ok(()) })
│   └── .run()
└── Ok(())
```

### DAW Application Flow

```
main.rs
├── init_logging()
├── initialize_database_pool()
│   └── Creates sqlx::PgPool
├── Arc::new(MidiManager::new())
│   └── Creates Arc<Mutex<MidiOutputConnection>>
│   └── Creates Arc<Mutex<Option<String>>>
├── Arc::new(SequencerEngine::new())
│   └── Creates Arc<RwLock<PlaybackState>>
│   └── Creates Arc<RwLock<u64>> (current_tick)
│   └── Creates Arc<Mutex<Option<Instant>>>
│   └── Creates Arc<RwLock<f32>> (bpm)
│   └── Creates Arc<RwLock<bool>> (loop_enabled)
├── Create AppState { db_pool }
├── tauri::Builder::default()
│   ├── .manage(state)
│   ├── .manage(midi_manager)
│   ├── .manage(sequencer_engine)
│   ├── .invoke_handler([commands...])
│   └── .run()
└── Ok(())
```

---

## 10. Key Design Decisions

### Why Arc<RwLock<T>> for Read-Heavy Data?

**Used in:** Database pool, playback state, BPM, loop settings

**Reasons:**
1. Multiple simultaneous readers (e.g., multiple commands querying position)
2. Infrequent writes (e.g., set_bpm called occasionally)
3. RwLock allows N readers OR 1 writer
4. tokio::sync::RwLock is async-aware (doesn't block executor)

### Why Arc<Mutex<T>> for Write-Heavy Data?

**Used in:** MIDI connection, start time, current device

**Reasons:**
1. Frequent state changes needed
2. Only one handler should modify at a time
3. Simpler semantics than RwLock for this pattern
4. tokio::sync::Mutex is async-aware

### Why Option<> Wrapped States?

**Examples:**
- `Arc<Mutex<Option<MidiOutputConnection>>>` - No connection initially
- `Arc<Mutex<Option<String>>>` - No device selected initially
- `Arc<Mutex<Option<Instant>>>` - No start time until playback begins

**Reasons:**
1. Represents absent state clearly
2. Avoids special/sentinel values
3. Forces explicit handling via `match` or `.ok_or()`
4. Type-safe error handling

---

## 11. Database Reconnection Pattern

**File:** `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/database/mod.rs` (Lines 245-290)

```rust
pub async fn reconnect(&self) -> Result<(), sqlx::Error> {
    const MAX_ATTEMPTS: u32 = 5;
    const MAX_DELAY_SECS: u64 = 30;

    let mut attempts = self.reconnect_attempts.write().await;
    *attempts = 0;

    for attempt in 1..=MAX_ATTEMPTS {
        *attempts = attempt;
        
        // Exponential backoff: 1s, 2s, 4s, 8s, 16s
        let delay_secs = std::cmp::min(2_u64.pow(attempt - 1), MAX_DELAY_SECS);

        if attempt > 1 {
            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
        }

        match Self::create_pool(&self.database_url).await {
            Ok(new_pool) => {
                let mut pool = self.pool.write().await;
                *pool = new_pool;
                *attempts = 0;
                return Ok(());
            }
            Err(e) => {
                if attempt == MAX_ATTEMPTS {
                    return Err(e);
                }
            }
        }
    }

    Err(sqlx::Error::PoolTimedOut)
}
```

**Backoff Strategy:**
- Attempt 1: Immediate
- Attempt 2: 1s wait
- Attempt 3: 2s wait
- Attempt 4: 4s wait
- Attempt 5: 8s wait
- Max delay: 30 seconds

---

## 12. File References Summary

| Component | File | Key Lines |
|-----------|------|-----------|
| Pipeline AppState | lib.rs | 25-27 |
| DAW AppState | commands/mod.rs | 16-18 |
| Database struct | database/mod.rs | 75-79 |
| Database init | database/mod.rs | 120-187 |
| Database reconnect | database/mod.rs | 245-290 |
| Error types | error.rs | 35-54 |
| Error conversions | error.rs | 92-143 |
| Result aliases | error.rs | 215, 232 |
| MidiManager | midi/manager.rs | 19-31 |
| SequencerEngine | sequencer/engine.rs | 27-46 |
| SequencerEngine usage | sequencer/engine.rs | 83-105 |
| WindowManager | windows/manager.rs | 11-32 |
| Pipeline main | main.rs | Full file |
| DAW main | main.rs (daw) | Full file |
| Command injection (pipeline) | commands/files.rs | 14-32 |
| Command injection (MIDI) | commands/midi.rs | 15-29 |
| Command injection (sequencer) | commands/sequencer.rs | 15-28 |

---

## 13. Recommended Initialization Checklist for WindowManager

- [ ] Define `WindowManager` struct with `Arc<Mutex<T>>` wrapping for shared mutable state
- [ ] Implement `.new()` constructor
- [ ] Implement `.with_storage()` for layout persistence
- [ ] Create window registration/unregistration methods
- [ ] Implement window visibility and positioning methods
- [ ] Add layout save/load methods using LayoutStorage
- [ ] Wrap in `Arc` for shared ownership across threads
- [ ] Add to AppState or manage separately via `.manage()`
- [ ] In `.setup()` closure, get Arc<Mutex<WindowManager>> via `app.state()`
- [ ] Use `.blocking_lock()` or spawn_blocking for non-async access
- [ ] Use `.lock().await` for async access
- [ ] Register initial windows in setup
- [ ] Provide Tauri commands for window operations
- [ ] Handle errors with Result types
- [ ] Add tests for registration, visibility, positioning

