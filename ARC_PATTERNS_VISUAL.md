# Arc<T> Synchronization Patterns - Visual Reference

## Decision Tree: Which Pattern to Use?

```
Need shared state in async context?
│
├─ YES
│  │
│  ├─ Need multiple simultaneous READS?
│  │  │
│  │  ├─ YES (read-heavy, occasional writes)
│  │  │  └─ Use: Arc<RwLock<T>>
│  │  │     Example: Arc<RwLock<PlaybackState>>
│  │  │     Read: .read().await
│  │  │     Write: .write().await
│  │  │
│  │  └─ NO (frequent writes or both read/write equally)
│  │     └─ Use: Arc<Mutex<T>>
│  │        Example: Arc<Mutex<Option<MidiOutputConnection>>>
│  │        Lock: .lock().await
│  │
│  └─ Wrapping Option type for absent state?
│     └─ Yes: Arc<Mutex<Option<T>>> or Arc<RwLock<Option<T>>>
│        Handles None without sentinel values
│
└─ NO
   └─ Don't use Arc, share ownership differently

```

---

## Pattern Comparison Matrix

| Aspect | Arc<RwLock<T>> | Arc<Mutex<T>> |
|--------|---|---|
| **Best For** | Read-heavy workloads | Balanced/write-heavy |
| **Read Performance** | Excellent (N readers) | Good (1 at a time) |
| **Write Performance** | Good | Good |
| **Contention** | Low (many readers) | Medium (single writer) |
| **Async Friendly** | Yes (tokio::sync) | Yes (tokio::sync) |
| **Code Complexity** | Simple | Simple |
| **Example** | Position tracking | Connection state |

---

## MIDI Software Center Usage Examples

### Example 1: Playback Position (Arc<RwLock<>>)

```rust
// Definition in SequencerEngine
current_tick: Arc<RwLock<u64>>,

// Queried frequently by multiple commands
#[tauri::command]
pub async fn get_playback_position(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<PlaybackPosition, String> {
    // Multiple simultaneous queries possible
    Ok(engine.get_position().await)  // Uses .read().await internally
}

// Updated occasionally
pub async fn seek(&self, tick: u64) {
    let mut current_tick = self.current_tick.write().await;
    *current_tick = tick;
}
```

**Why RwLock?**
- Position queried in real-time by UI updates (many reads)
- Seek called occasionally (few writes)
- Multiple commands can check position simultaneously

---

### Example 2: MIDI Connection (Arc<Mutex<>>)

```rust
// Definition in MidiManager
connection: Arc<Mutex<Option<MidiOutputConnection>>>,

// Rarely read, frequently changed
pub async fn connect(&self, device_name: &str) -> Result<(), String> {
    // ... establish connection ...
    let mut conn_lock = self.connection.lock().await;
    *conn_lock = Some(connection);  // Write operation
    Ok(())
}

pub async fn send_message(&self, msg: &[u8]) -> Result<(), String> {
    let conn_lock = self.connection.lock().await;  // Read operation
    match conn_lock.as_ref() {
        Some(conn) => conn.send(msg)?,
        None => return Err("Not connected".to_string()),
    }
    Ok(())
}
```

**Why Mutex?**
- Connection doesn't change frequently
- Send operations block each other (mutual exclusion needed)
- Option wrapper handles "not connected" state
- Simpler semantics than RwLock for exclusive access

---

### Example 3: Database Pool (Arc<RwLock<>> via Database)

```rust
// Definition in Database
pub struct Database {
    pool: Arc<RwLock<PgPool>>,
    // ...
}

// Used in many queries (frequent reads)
pub async fn pool(&self) -> PgPool {
    self.pool.read().await.clone()  // Cheap Arc clone
}

// Reconnection only on failure (rare writes)
pub async fn reconnect(&self) -> Result<(), sqlx::Error> {
    let mut pool = self.pool.write().await;
    *pool = new_pool;
    Ok(())
}
```

**Why RwLock?**
- Queries read the pool constantly
- Reconnection happens only on database failure
- Many commands can acquire pool simultaneously
- Write lock held briefly during reconnection

---

### Example 4: Sequencer State (Arc<RwLock<>> for PlaybackState)

```rust
// Definition
state: Arc<RwLock<PlaybackState>>,

// Started/stopped frequently
pub async fn start(&self) -> Result<(), String> {
    let mut state = self.state.write().await;  // Exclusive write
    if *state == PlaybackState::Playing {
        return Ok(());
    }
    *state = PlaybackState::Playing;
    Ok(())
}

// Checked constantly by UI
pub async fn get_state(&self) -> PlaybackState {
    *self.state.read().await  // Non-exclusive read
}
```

**Why RwLock?**
- State queried constantly for UI updates
- State changes only on user action (start/stop)
- Multiple queries can happen during playback
- One state change at a time

---

## Async Lock Usage Patterns

### The .lock().await Pattern (Mutex)

```rust
let mut guard = some_mutex.lock().await;
*guard = new_value;
// guard is automatically released when it goes out of scope
```

**Characteristics:**
- Waits indefinitely for the lock
- One holder at a time
- Simpler mental model
- Cleaner code for balanced read/write

### The .read().await Pattern (RwLock)

```rust
let guard = some_rwlock.read().await;
let value = *guard;  // Multiple readers simultaneously possible
```

**Characteristics:**
- Multiple simultaneous readers
- Exclusive access to read
- Returns immediately if no writers
- Queues for write lock

### The .write().await Pattern (RwLock)

```rust
let mut guard = some_rwlock.write().await;
*guard = new_value;
// No readers or writers possible while held
```

**Characteristics:**
- Exclusive access
- Blocks all readers and writers
- Use only when modifying
- Hold time should be minimized

---

## Common Mistakes & Fixes

### Mistake 1: Using Mutex When RwLock Needed

```rust
// ❌ BAD - Blocks readers during reads
pub struct Player {
    state: Arc<Mutex<PlaybackState>>,
}

pub async fn get_state(&self) -> PlaybackState {
    *self.state.lock().await  // Blocks other readers
}

// ✅ GOOD - Allows simultaneous readers
pub struct Player {
    state: Arc<RwLock<PlaybackState>>,
}

pub async fn get_state(&self) -> PlaybackState {
    *self.state.read().await  // Other readers can proceed
}
```

### Mistake 2: Using RwLock When Mutex Needed

```rust
// ❌ BAD - Overengineering for simple connection
pub struct Manager {
    conn: Arc<RwLock<Option<Connection>>>,
}

// ✅ GOOD - Simpler, correct semantics
pub struct Manager {
    conn: Arc<Mutex<Option<Connection>>>,
}
```

### Mistake 3: Holding Lock Too Long

```rust
// ❌ BAD - Lock held during I/O
pub async fn process(&self) -> Result<()> {
    let mut data = self.data.write().await;
    let result = expensive_async_call().await;  // Lock held!
    *data = result;
    Ok(())
}

// ✅ GOOD - Lock held only for state change
pub async fn process(&self) -> Result<()> {
    let result = expensive_async_call().await;  // No lock
    let mut data = self.data.write().await;
    *data = result;  // Lock held briefly
    Ok(())
}
```

### Mistake 4: Forgetting .await

```rust
// ❌ BAD - Returns Future, not the lock
let guard = state.lock();  // Not awaited!

// ✅ GOOD - Actually acquires the lock
let guard = state.lock().await;
```

---

## Integration Checklist

- [ ] Identify which state is read-heavy vs write-heavy
- [ ] Choose Arc<RwLock<T>> for read-heavy (5:1 reads:writes ratio)
- [ ] Choose Arc<Mutex<T>> for balanced/write-heavy
- [ ] Wrap in Arc before passing to .manage()
- [ ] Use State<'_, Arc<T>> in command parameters
- [ ] Call .read().await or .write().await (RwLock) OR .lock().await (Mutex)
- [ ] Keep lock scope minimal
- [ ] Don't hold locks during async operations
- [ ] Use Option<T> for optional values
- [ ] Test with multiple concurrent commands
- [ ] Profile lock contention if performance issues

---

## Real Performance Scenario

**Scenario:** UI updates position every 100ms while playback can be paused/resumed occasionally

```rust
// Access Pattern Over 1 Second:
// - 10 position queries (10 reads)
// - 0-1 state changes (0-1 writes)
// - 50 BPM reads for timing calculations
// - 0-1 tempo changes (0-1 writes)

// With Arc<RwLock<>>:
// - All 10+50=60 reads proceed simultaneously
// - 0-2 writes exclusive access

// With Arc<Mutex<>>:
// - First read acquires, others wait
// - 10+50 sequential acquisitions
// - Performance degradation under load

// Result: Arc<RwLock<>> is 5-10x faster for this pattern
```

---

## Summary Table

| State | Type | Lock Pattern | Contention | Best For |
|-------|------|---|---|---|
| PlaybackState | Arc<RwLock<>> | .read()/.write() | Low | Position, state flags |
| BPM | Arc<RwLock<>> | .read()/.write() | Low | Frequently read values |
| MidiConnection | Arc<Mutex<>> | .lock() | Medium | Exclusive resource |
| CurrentDevice | Arc<Mutex<>> | .lock() | Medium | Single value |
| PgPool (via Database) | Arc<RwLock<>> | Via pool() method | Low | Database access |

