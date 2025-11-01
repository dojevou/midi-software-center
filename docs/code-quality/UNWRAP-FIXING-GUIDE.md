# .unwrap() Fixing Guide
**MIDI Library System - Code Quality Improvements**

---

## 🚨 Why .unwrap() Is Forbidden in Production Code

**The Rule:** NO `.unwrap()` or `.expect()` in production code

**Why:**
- `.unwrap()` causes panics (program crashes)
- Panics cannot be recovered from
- Users see cryptic error messages
- Makes debugging harder
- Violates Rust's error handling best practices

**Exception:** `.unwrap()` is ACCEPTABLE in test code (`#[cfg(test)]` blocks)

---

## 🎯 Common .unwrap() Patterns & Fixes

### Pattern 1: Option::unwrap() → Use ?

**Context:** When you have an Option and the function returns Result

```rust
// ❌ BAD
pub fn get_bpm(metadata: &MidiMetadata) -> Result<f64, AnalysisError> {
    let bpm = metadata.bpm.unwrap();  // Panic if None!
    Ok(bpm)
}

// ✅ GOOD - Use ok_or to convert Option to Result
pub fn get_bpm(metadata: &MidiMetadata) -> Result<f64, AnalysisError> {
    let bpm = metadata.bpm
        .ok_or(AnalysisError::MissingBpm)?;
    Ok(bpm)
}

// ✅ ALSO GOOD - With custom error message
pub fn get_bpm(metadata: &MidiMetadata) -> Result<f64, AnalysisError> {
    metadata.bpm
        .ok_or_else(|| AnalysisError::MissingBpm {
            file_id: metadata.file_id.clone()
        })
}
```

### Pattern 2: Result::unwrap() → Use ?

**Context:** Propagate errors up the call stack

```rust
// ❌ BAD
pub fn parse_midi_file(path: &Path) -> Result<MidiFile, ParseError> {
    let bytes = std::fs::read(path).unwrap();  // Panic on I/O error!
    let smf = midly::parse(&bytes).unwrap();   // Panic on parse error!
    Ok(convert_to_midi_file(smf))
}

// ✅ GOOD - Propagate errors with ?
pub fn parse_midi_file(path: &Path) -> Result<MidiFile, ParseError> {
    let bytes = std::fs::read(path)
        .map_err(|e| ParseError::IoError(e))?;
    let smf = midly::parse(&bytes)
        .map_err(|e| ParseError::MidlyError(e))?;
    Ok(convert_to_midi_file(smf))
}
```

### Pattern 3: Default Values for Options

**Context:** When None is a valid case with a sensible default

```rust
// ❌ BAD
pub fn get_default_bpm(metadata: &MidiMetadata) -> f64 {
    metadata.bpm.unwrap()  // Panic if None!
}

// ✅ GOOD - Use unwrap_or for default
pub fn get_default_bpm(metadata: &MidiMetadata) -> f64 {
    metadata.bpm.unwrap_or(120.0)  // Standard MIDI default
}

// ✅ ALSO GOOD - Use unwrap_or_else for computed default
pub fn get_bpm_or_calculate(metadata: &MidiMetadata) -> f64 {
    metadata.bpm.unwrap_or_else(|| {
        // Compute default from other data
        calculate_approximate_bpm(&metadata.tempo_events)
    })
}

// ✅ ALSO GOOD - Use unwrap_or_default
pub fn get_key(metadata: &MidiMetadata) -> String {
    metadata.key.clone().unwrap_or_default()  // Empty string
}
```

### Pattern 4: Lock::unwrap() → Use expect with context

**Context:** Mutex locks that "should never" fail

```rust
use std::sync::Mutex;

// ❌ BAD
pub fn get_state(&self) -> AppState {
    self.state.lock().unwrap().clone()
}

// ✅ ACCEPTABLE - Use expect with clear context
pub fn get_state(&self) -> AppState {
    self.state.lock()
        .expect("State mutex poisoned - this is a bug")
        .clone()
}

// ✅ BETTER - Handle the error properly
pub fn get_state(&self) -> Result<AppState, StateError> {
    self.state.lock()
        .map(|guard| guard.clone())
        .map_err(|_| StateError::MutexPoisoned)
}
```

### Pattern 5: Collection Access → Use get()

**Context:** Accessing vectors, hashmaps by index/key

```rust
// ❌ BAD
pub fn get_first_track(tracks: &[Track]) -> Track {
    tracks[0].clone()  // Panic if empty!
}

// ✅ GOOD - Use get() which returns Option
pub fn get_first_track(tracks: &[Track]) -> Option<Track> {
    tracks.get(0).cloned()
}

// ✅ ALSO GOOD - With error handling
pub fn get_first_track(tracks: &[Track]) -> Result<Track, TrackError> {
    tracks.get(0)
        .cloned()
        .ok_or(TrackError::NoTracksFound)
}

// For HashMap:
// ❌ BAD
let value = map.get(&key).unwrap();

// ✅ GOOD
let value = map.get(&key).ok_or(Error::KeyNotFound)?;
```

### Pattern 6: String Conversion → Use proper error handling

**Context:** Parsing strings to numbers or other types

```rust
// ❌ BAD
pub fn parse_bpm(s: &str) -> f64 {
    s.parse::<f64>().unwrap()  // Panic on invalid input!
}

// ✅ GOOD - Propagate parse error
pub fn parse_bpm(s: &str) -> Result<f64, ParseError> {
    s.parse::<f64>()
        .map_err(|e| ParseError::InvalidBpm {
            input: s.to_string(),
            error: e.to_string(),
        })
}

// ✅ ALSO GOOD - With validation
pub fn parse_bpm(s: &str) -> Result<f64, ParseError> {
    let bpm = s.parse::<f64>()
        .map_err(|e| ParseError::InvalidBpm(e.to_string()))?;
    
    if bpm <= 0.0 || bpm > 300.0 {
        return Err(ParseError::BpmOutOfRange(bpm));
    }
    
    Ok(bpm)
}
```

### Pattern 7: First/Last in Iterator → Use proper methods

```rust
// ❌ BAD
pub fn get_first_note(notes: Vec<Note>) -> Note {
    notes.into_iter().next().unwrap()
}

// ✅ GOOD - Return Option
pub fn get_first_note(notes: Vec<Note>) -> Option<Note> {
    notes.into_iter().next()
}

// ✅ ALSO GOOD - With default
pub fn get_first_note_or_default(notes: Vec<Note>) -> Note {
    notes.into_iter().next().unwrap_or_default()
}
```

---

## 🔧 Systematic Fixing Process

### Step 1: Identify the Pattern
Look at what type is calling `.unwrap()`:
- `Option::unwrap()` → See Pattern 1 or 3
- `Result::unwrap()` → See Pattern 2
- `Mutex::unwrap()` → See Pattern 4
- Collection indexing → See Pattern 5
- String parsing → See Pattern 6

### Step 2: Determine Context
Ask:
1. **Should this error propagate?** → Use `?` operator
2. **Is there a sensible default?** → Use `unwrap_or(default)`
3. **Is None/Error a bug?** → Use `.expect("clear message")`
4. **Should I return Option?** → Return `Option<T>` instead

### Step 3: Choose the Right Replacement

```rust
// Error should propagate?
result.unwrap()  →  result?
option.unwrap()  →  option.ok_or(Error)?

// Has sensible default?
option.unwrap()  →  option.unwrap_or(default)
option.unwrap()  →  option.unwrap_or_default()

// Truly should never fail (rare)?
result.unwrap()  →  result.expect("Bug: this should never fail because...")

// Can return None?
option.unwrap()  →  return None earlier, or just return option
```

---

## 📋 Project-Specific Patterns

### MIDI File Parsing

```rust
// ❌ BAD
let smf = midly::parse(&bytes).unwrap();

// ✅ GOOD
let smf = midly::parse(&bytes)
    .map_err(|e| MidiError::ParseError {
        reason: format!("Failed to parse MIDI: {}", e)
    })?;
```

### Database Queries

```rust
// ❌ BAD
let file = sqlx::query_as!(MidiFile, "SELECT * FROM files WHERE id = $1", id)
    .fetch_one(&pool)
    .await
    .unwrap();

// ✅ GOOD
let file = sqlx::query_as!(MidiFile, "SELECT * FROM files WHERE id = $1", id)
    .fetch_one(&pool)
    .await
    .map_err(|e| DatabaseError::QueryFailed {
        query: "fetch file by id".to_string(),
        error: e.to_string(),
    })?;
```

### BPM Detection

```rust
// ❌ BAD
pub fn detect_bpm(midi: &MidiFile) -> f64 {
    let tempo = midi.tempo_events.first().unwrap();
    calculate_bpm(tempo.value).unwrap()
}

// ✅ GOOD
pub fn detect_bpm(midi: &MidiFile) -> Result<f64, AnalysisError> {
    let tempo = midi.tempo_events.first()
        .ok_or(AnalysisError::NoTempoEvents)?;
    
    calculate_bpm(tempo.value)
        .ok_or(AnalysisError::InvalidTempo(tempo.value))
}
```

### File Path Operations

```rust
// ❌ BAD
let filename = path.file_name().unwrap().to_str().unwrap();

// ✅ GOOD
let filename = path.file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| PathError::InvalidFilename {
        path: path.to_path_buf()
    })?;
```

---

## 🎯 Priority Fixing Order

1. **CRITICAL**: Fix all `.unwrap()` in `core/` directories (Trusty Modules)
   - `shared/src/`
   - `*/src-tauri/src/core/`
   - `database/src/models/`

2. **HIGH**: Fix all `.unwrap()` in orchestration layer
   - `database/src/repositories/`
   - `*/src-tauri/src/commands/`

3. **MEDIUM**: Fix all `.unwrap()` in entry points
   - `main.rs`
   - `lib.rs`

4. **LOW**: Fix remaining `.unwrap()` in standalone tools
   - `bin/*`

---

## ✅ Testing After Fixes

After fixing `.unwrap()` calls, verify:

```bash
# 1. Code compiles
cargo build --workspace

# 2. No clippy warnings about unwrap
cargo clippy --workspace -- -D warnings

# 3. Tests still pass
cargo test --workspace

# 4. Check for remaining unwraps
./find-unwraps.sh
```

---

## 🔍 Common Mistakes to Avoid

### Mistake 1: Using expect instead of unwrap
```rust
// ❌ Still panics!
let value = result.expect("Should work");

// ✅ GOOD - Proper error handling
let value = result.map_err(|e| MyError::from(e))?;
```

### Mistake 2: Hiding unwrap with a helper
```rust
// ❌ Still panics, just hidden!
fn must_get<T>(opt: Option<T>) -> T {
    opt.unwrap()
}

// ✅ GOOD - Return Result
fn must_get<T>(opt: Option<T>) -> Result<T, Error> {
    opt.ok_or(Error::MissingValue)
}
```

### Mistake 3: Using unwrap in test setup
```rust
// ⚠️ ACCEPTABLE in tests, but fragile
#[test]
fn test_something() {
    let data = setup_test_data().unwrap();  // OK in tests
    // ... test code
}

// ✅ BETTER - Still in tests
#[test]
fn test_something() -> Result<(), Box<dyn std::error::Error>> {
    let data = setup_test_data()?;  // Clearer error messages
    // ... test code
    Ok(())
}
```

---

## 📚 Additional Resources

- [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Rust By Example - Error Handling](https://doc.rust-lang.org/rust-by-example/error.html)
- [anyhow crate documentation](https://docs.rs/anyhow/)
- [thiserror crate documentation](https://docs.rs/thiserror/)

---

**Remember:** Every `.unwrap()` is a potential panic. Proper error handling makes your code more robust and user-friendly!
