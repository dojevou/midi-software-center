# AppState & WindowManager Quick Reference

## One-Page Summary

### Pipeline AppState
- **File:** `pipeline/src-tauri/src/lib.rs:25-27`
- **Struct:** Single `Database` field managing `Arc<RwLock<PgPool>>`
- **Init:** Retry-enabled database connection with exponential backoff
- **Pattern:** `.manage(state)` → `State<'_, AppState>` in commands

### DAW AppState
- **File:** `daw/src-tauri/src/commands/mod.rs:16-18`
- **Struct:** Optional `sqlx::PgPool` for graceful degradation
- **Separate:** `Arc<MidiManager>` and `Arc<SequencerEngine>` via `.manage()`
- **Pattern:** `State<'_, Arc<MidiManager>>` in commands

---

## Arc<T> Patterns at a Glance

### Arc<RwLock<T>> - Read-Heavy State
```rust
state: Arc<RwLock<PlaybackState>>       // Many readers, few writers
bpm: Arc<RwLock<f32>>                   // Tempo (read in timing, write on change)
current_tick: Arc<RwLock<u64>>          // Position (read frequently, write occasionally)
loop_enabled: Arc<RwLock<bool>>         // Loop state (mostly read)
```

**Usage:** `.read().await` for reads, `.write().await` for writes

### Arc<Mutex<T>> - Write-Heavy State
```rust
connection: Arc<Mutex<Option<MidiOutputConnection>>>  // MIDI connection (frequent changes)
current_device: Arc<Mutex<Option<String>>>            // Selected device (few readers)
start_time: Arc<Mutex<Option<Instant>>>              // Timing (frequent updates)
```

**Usage:** `.lock().await` for both reads and writes

---

## Error Handling Pattern

**File:** `pipeline/src-tauri/src/error.rs`

```rust
// Define error enum
#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    IOError(std::io::Error),
    MidiError(String),
    GeneralError(String),
}

// Type aliases for commands
pub type AppResult<T> = Result<T, AppError>;
pub type TauriResult<T> = Result<T, String>;

// In command:
#[tauri::command]
pub async fn get_file(id: i64) -> TauriResult<File> {
    let pool = state.database.pool().await;
    let file = sqlx::query_as::<_, File>(...)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::from(e).into())?;
    
    file.ok_or_else(|| AppError::not_found(&format!("File {}", id)).into())
}
```

---

## State Injection in Commands

### Pattern 1: Direct State
```rust
#[tauri::command]
pub async fn my_command(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.database.pool().await;
    // ...
}
```

### Pattern 2: Arc<T> State
```rust
#[tauri::command]
pub async fn my_command(
    manager: State<'_, Arc<MidiManager>>,
) -> Result<(), String> {
    manager.list_devices()?
    // ...
}
```

### Pattern 3: Multiple Managers
```rust
#[tauri::command]
pub async fn my_command(
    state: State<'_, AppState>,
    midi: State<'_, Arc<MidiManager>>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<(), String> {
    // Access all three states
}
```

---

## Database Pool Configuration

**File:** `pipeline/src-tauri/src/database/mod.rs:120-187`

**Key Settings:**
- Max connections: Dynamic (auto-tuned based on CPU/RAM)
- Min connections: 20% of max (min 5)
- Acquire timeout: 10 seconds
- Max lifetime: 1800 seconds (30 min)
- Idle timeout: 300 seconds (5 min)
- Statement cache: 100 per connection
- Test before acquire: true

**Access Pattern:**
```rust
let pool = database.pool().await;  // Cheap clone (Arc increment)
sqlx::query(...).fetch_all(&pool).await?;
```

---

## WindowManager Setup Pattern

**File:** `pipeline/src-tauri/src/windows/manager.rs:11-32`

**Expected Initialization:**
```rust
// In main.rs
let window_manager = Arc::new(Mutex::new(WindowManager::new()));

tauri::Builder::default()
    .manage(window_manager)
    .setup(|app| {
        let wm = app.state::<Arc<Mutex<WindowManager>>>();
        let mut manager = wm.blocking_lock();
        manager.register_window(WindowInfo::new(...));
        Ok(())
    })
    .run(tauri::generate_context!())?
```

**Key Methods:**
- `register_window()` - Add window to manager
- `show_window()` / `hide_window()` - Control visibility
- `set_position()` - Move/resize window
- `with_storage()` - Enable layout persistence

---

## Database Reconnection

**File:** `pipeline/src-tauri/src/database/mod.rs:245-290`

**Backoff Strategy:**
- Attempt 1: Immediate
- Attempt 2-5: 1s, 2s, 4s, 8s wait (exponential, capped at 30s)
- Max attempts: 5

**Usage:**
```rust
if let Err(e) = database.reconnect().await {
    eprintln!("Failed to reconnect: {}", e);
}
```

---

## Current Setup Closures

**Pipeline** (minimal):
```rust
.setup(|_app| {
    info!("Application setup complete");
    Ok(())
})
```

**DAW** (none - all init before Builder)

---

## File Quick Links

| What | File | Lines |
|------|------|-------|
| Pipeline AppState | lib.rs | 25-27 |
| DAW AppState | commands/mod.rs | 16-18 |
| Database | database/mod.rs | 75-79, 120-187 |
| Errors | error.rs | 35-54, 92-143 |
| MidiManager | midi/manager.rs | 19-31 |
| SequencerEngine | sequencer/engine.rs | 27-46 |
| WindowManager | windows/manager.rs | 11-32 |
| Pipeline main | main.rs (pipeline) | Full |
| DAW main | main.rs (daw) | Full |

---

## Checklist for WindowManager Integration

- [ ] Wrap in `Arc<Mutex<WindowManager>>`
- [ ] Add to `.manage()`
- [ ] Implement `.setup()` to register initial windows
- [ ] Use `.blocking_lock()` in setup (non-async context)
- [ ] Use `.lock().await` in async commands
- [ ] Provide commands for window operations
- [ ] Add layout persistence if needed
- [ ] Handle errors with Result types

---

**For detailed reference with code samples, see: `AppState_WindowManager_Architecture.md`**
