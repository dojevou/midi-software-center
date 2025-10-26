# ARCHITECTURE REFERENCE

**The Building Code for MIDI Software Center**

**Date:** 2025-10-24
**Purpose:** Definitive guide to the Three Archetypes pattern and code quality standards
**Audience:** Developers, AI assistants, code reviewers

---

## 📋 TABLE OF CONTENTS

1. [What is the Three Archetypes Pattern?](#what-is-the-three-archetypes-pattern)
2. [The Three Archetypes Explained](#the-three-archetypes-explained)
3. [Decision Tree: Choosing the Right Archetype](#decision-tree-choosing-the-right-archetype)
4. [Code Quality Requirements](#code-quality-requirements)
5. [Testing Requirements](#testing-requirements)
6. [Examples from This Project](#examples-from-this-project)
7. [Common Mistakes](#common-mistakes)
8. [Integration Patterns](#integration-patterns)

---

## 🎯 WHAT IS THE THREE ARCHETYPES PATTERN?

The Three Archetypes Pattern is a code organization philosophy that **classifies every piece of code** into one of three categories based on its **purpose and characteristics**.

### Why Three Archetypes?

**Problem:** Traditional code organization creates confusion:
- "Is this a library or an application?"
- "Should this have a main() function?"
- "Can other code import this?"
- "How much testing is enough?"

**Solution:** Three clear categories with explicit rules:

```
┌─────────────────────────────────────────────────────────────┐
│                    THREE ARCHETYPES                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. TASK-O-MATIC    = Complete standalone tasks            │
│     "I do ONE thing from start to finish"                   │
│                                                             │
│  2. GROWN-UP SCRIPT = Orchestration + I/O + reusability     │
│     "I coordinate things AND can be imported"               │
│                                                             │
│  3. TRUSTY MODULE   = Pure, tested, reusable logic          │
│     "I'm a building block others can trust"                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Core Principle

**Every file must clearly be ONE archetype.** No mixing, no confusion.

---

## 🔧 THE THREE ARCHETYPES EXPLAINED

### Archetype 1: Task-O-Matic

**Purpose:** Execute a complete, standalone task from start to finish

**Characteristics:**
- ✅ Has a `main()` function or equivalent entry point
- ✅ User-facing (CLI, GUI app, background service)
- ✅ Complete workflow (setup → process → cleanup)
- ✅ Can use I/O freely (files, network, database)
- ❌ NOT imported by other code
- ❌ NOT a library

**File Location:** Component root or `bin/` directory

**Testing:** End-to-end testing (does the whole task work?)

**Example:**
```rust
// bin/import-tool.rs - A Task-O-Matic

use clap::Parser;
use midi_library::db::FileRepository;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    path: String,
}

fn main() -> anyhow::Result<()> {
    // Entry point - complete task from start to finish
    let args = Args::parse();

    println!("Importing MIDI files from {}", args.path);

    // 1. Setup
    let db_pool = setup_database().await?;
    let repo = FileRepository::new(&db_pool);

    // 2. Process
    let files = scan_directory(&args.path)?;
    for file in files {
        repo.insert_file(file).await?;
    }

    // 3. Cleanup
    println!("Imported {} files", files.len());
    Ok(())
}
```

**When to Use:**
- Building a CLI tool
- Creating a background worker
- Writing a complete application

---

### Archetype 2: Grown-up Script

**Purpose:** Orchestrate I/O operations while remaining reusable

**Characteristics:**
- ✅ Can be imported by other code
- ✅ Does I/O (file system, database, network)
- ✅ Has side effects (logging, state changes)
- ✅ Reusable orchestration logic
- ✅ Has both entry point AND implementation
- ❌ NOT pure (has side effects)

**File Location:** Component `src/` directory (not `core/`)

**Testing:** Integration tests with mocked I/O

**Example:**
```rust
// daw/src-tauri/src/commands/file_commands.rs - A Grown-up Script

use sqlx::PgPool;
use crate::models::File;

// ✅ Entry point (Tauri command)
#[tauri::command]
pub async fn search_files(
    query: String,
    state: tauri::State<'_, AppState>
) -> Result<Vec<File>, String> {
    // Thin wrapper - converts errors for Tauri
    search_files_impl(&state.db_pool, &query)
        .await
        .map_err(|e| e.to_string())
}

// ✅ Reusable implementation (can be tested and imported)
pub async fn search_files_impl(
    pool: &PgPool,
    query: &str
) -> Result<Vec<File>, DatabaseError> {
    // Real logic here - orchestrates database I/O
    sqlx::query_as!(
        File,
        "SELECT * FROM files WHERE name ILIKE $1",
        format!("%{}%", query)
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_files_impl() {
        // Integration test with test database
        let pool = create_test_pool().await;
        let result = search_files_impl(&pool, "test").await.unwrap();
        assert!(!result.is_empty());
    }
}
```

**Pattern: Entry Point + Implementation**
```rust
// Entry point (not testable without full app)
#[tauri::command]
pub async fn do_thing(state: State) -> Result<T, String> {
    do_thing_impl(&state.dependency).await.map_err(|e| e.to_string())
}

// Implementation (fully testable with mocks)
pub async fn do_thing_impl(dep: &Dependency) -> Result<T, Error> {
    // Real logic here
}
```

**When to Use:**
- Tauri commands
- HTTP API handlers
- Repository layer (database access)
- File system operations that need to be reusable

---

### Archetype 3: Trusty Module

**Purpose:** Provide pure, well-tested, reusable logic

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

**File Location:** `shared/rust/src/core/` or component `core/` directory

**Testing:** Unit tests with 80%+ coverage (REQUIRED)

**Example:**
```rust
// shared/rust/src/core/analysis/bpm_detector.rs - A Trusty Module

use crate::core::midi::MidiFile;

/// Detects BPM from MIDI tempo events
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file to analyze
///
/// # Returns
/// * `Ok(BpmAnalysis)` - BPM and confidence score
/// * `Err(BpmError)` - If analysis fails
///
/// # Examples
/// ```
/// use midi_library::core::analysis::detect_bpm;
/// use midi_library::core::midi::parse_midi;
///
/// let midi = parse_midi(&data)?;
/// let bpm = detect_bpm(&midi)?;
/// assert_eq!(bpm.value, 120.0);
/// ```
pub fn detect_bpm(midi_file: &MidiFile) -> Result<BpmAnalysis, BpmError> {
    // Pure logic - no I/O, no side effects
    let tempo_events = extract_tempo_events(midi_file)?;

    if tempo_events.is_empty() {
        return Ok(BpmAnalysis {
            value: 120.0,  // Default
            confidence: 0.0,
        });
    }

    // Calculate weighted average
    let weighted_bpm = calculate_weighted_average(&tempo_events)?;
    let confidence = calculate_confidence(&tempo_events);

    Ok(BpmAnalysis {
        value: weighted_bpm,
        confidence,
    })
}

#[derive(Debug, Clone)]
pub struct BpmAnalysis {
    pub value: f64,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tempo() {
        let midi = create_test_midi_with_tempo(120.0);
        let result = detect_bpm(&midi).unwrap();
        assert_eq!(result.value, 120.0);
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_multiple_tempos() {
        let midi = create_test_midi_with_tempos(vec![120.0, 140.0, 100.0]);
        let result = detect_bpm(&midi).unwrap();
        // Should return weighted average
        assert!(result.value > 100.0 && result.value < 140.0);
    }

    #[test]
    fn test_no_tempo_events() {
        let midi = create_test_midi_empty();
        let result = detect_bpm(&midi).unwrap();
        assert_eq!(result.value, 120.0);  // Default
        assert_eq!(result.confidence, 0.0);
    }
}
```

**When to Use:**
- Algorithms and business logic
- Data transformations
- Parsing and serialization
- Mathematical calculations
- Any logic that should be 100% reliable

**CRITICAL RULE:** Everything in `core/` directories MUST be a Trusty Module.

---

## 🌲 DECISION TREE: CHOOSING THE RIGHT ARCHETYPE

Use this flowchart to classify any new code:

```
┌─────────────────────────────────────────────────────────────┐
│ START: I need to write some code...                         │
└─────────────────────────────────────────────────────────────┘
                           ↓
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

### Detailed Decision Guide

#### Question 1: Will other code import/reuse this?

**Ask yourself:**
- Will this be called from multiple places?
- Could this logic be useful in other contexts?
- Does this implement a reusable algorithm or pattern?

**If NO** → Probably a **Task-O-Matic**
- Example: A one-off import script
- Example: A CLI tool that's never imported

**If YES** → Continue to Question 3

---

#### Question 2: Is it a complete standalone task?

**Ask yourself:**
- Does this have a clear start and end?
- Is this user-facing (CLI, GUI, service)?
- Does this orchestrate a complete workflow?

**If YES** → **Task-O-Matic**
- Example: `bin/import-tool.rs` (complete import workflow)
- Example: Tauri app `main.rs` (launches GUI)

**If NO** → **Rethink your approach**
- If it's not reusable AND not standalone, what is it?
- Consider: Should this be broken into reusable pieces?

---

#### Question 3: Does it also need to run standalone?

**Ask yourself:**
- Does this need a `main()` function?
- Is this both a library AND an executable?

**If YES** → **Grown-up Script**
- Example: A Tauri command that also needs a CLI
- Pattern: Use `src/lib.rs` for logic + `src/main.rs` for entry point

**If NO** → Continue to Question 4

---

#### Question 4: Does it do I/O or side effects?

**Ask yourself:**
- Does this read/write files?
- Does this access a database?
- Does this make network calls?
- Does this print to console?
- Does this modify global state?

**If YES** → **Grown-up Script**
- Example: File repository (database I/O)
- Example: Tauri command (app state changes)

**If NO** → **Trusty Module**
- Example: BPM detection algorithm
- Example: MIDI parser
- Example: Data validation logic

---

### Quick Reference Table

| Code Type | Import? | I/O? | Entry? | → Archetype |
|-----------|---------|------|--------|-------------|
| CLI tool that's never imported | No | Yes | Yes | Task-O-Matic |
| Tauri command | Yes | Yes | No* | Grown-up Script |
| Database repository | Yes | Yes | No | Grown-up Script |
| MIDI parser | Yes | No | No | Trusty Module |
| BPM detector | Yes | No | No | Trusty Module |
| Main app entry | No | Yes | Yes | Task-O-Matic |
| Pure algorithm | Yes | No | No | Trusty Module |

*Has entry via `#[tauri::command]` but not standalone

---

## 📏 CODE QUALITY REQUIREMENTS

### Universal Requirements (All Archetypes)

#### 1. Error Handling (CRITICAL)

**NEVER use `.unwrap()` or `.expect()` in production code.**

```rust
// ❌ BAD - Will panic in production
let value = some_option.unwrap();
let result = some_result.expect("Failed");

// ✅ GOOD - Proper error propagation
let value = some_option.ok_or(MyError::MissingValue)?;
let result = some_result.map_err(|e| MyError::from(e))?;

// ✅ GOOD - With context
let value = some_option.ok_or_else(|| {
    MyError::MissingValue {
        context: "Expected configuration value".to_string()
    }
})?;
```

**Use proper error types:**
```rust
// Application code - use anyhow::Result
use anyhow::{Result, Context};

pub async fn load_config(path: &str) -> Result<Config> {
    let data = tokio::fs::read_to_string(path)
        .await
        .context(format!("Failed to read config from {}", path))?;

    serde_json::from_str(&data)
        .context("Failed to parse config JSON")
}

// Library code - use thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("Invalid MIDI header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Exception:** `.unwrap()` is allowed in tests and prototyping:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let value = some_option.unwrap();  // OK in tests
        assert_eq!(value, 42);
    }
}
```

---

#### 2. Documentation (MANDATORY)

**All public APIs must have documentation comments.**

```rust
/// Brief one-line summary
///
/// More detailed explanation if needed. Explain what the function does,
/// any important assumptions, and special behavior.
///
/// # Arguments
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
/// * `Ok(Value)` - Success case description
/// * `Err(Error)` - Error case description
///
/// # Errors
/// Returns `Error::Type1` if condition 1.
/// Returns `Error::Type2` if condition 2.
///
/// # Examples
/// ```
/// use my_crate::my_function;
///
/// let result = my_function("input")?;
/// assert_eq!(result.field, expected);
/// ```
///
/// # Panics
/// This function panics if... (only if it can panic)
pub fn my_function(param1: &str, param2: i32) -> Result<Value, Error> {
    // Implementation
}
```

**Verify documentation coverage:**
```bash
# Generate documentation and check for warnings
cargo doc --no-deps --document-private-items

# Look for missing documentation warnings
cargo doc --no-deps 2>&1 | grep "missing documentation"
```

---

#### 3. Code Style

**Use `rustfmt` and `clippy`:**
```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy -- -D warnings
```

**Performance guidelines:**
```rust
// ✅ Use &str for parameters (avoid unnecessary allocation)
pub fn process(input: &str) -> String { }

// ❌ Don't use String for parameters
pub fn process(input: String) -> String { }  // Unnecessary allocation

// ✅ Use Vec<T> for sequences (better performance)
let items: Vec<Item> = vec![];

// ❌ Avoid LinkedList (poor cache locality)
let items: LinkedList<Item> = LinkedList::new();

// ✅ Use #[derive] when possible
#[derive(Debug, Clone, PartialEq, Eq)]
struct MyStruct { }

// ❌ Don't manually implement if derive works
impl Debug for MyStruct { }  // Unnecessary
```

---

### Archetype-Specific Requirements

#### Task-O-Matic Requirements

**Structure:**
```rust
// bin/my-tool.rs or src/main.rs

use clap::Parser;  // CLI argument parsing
use anyhow::Result;  // Error handling

#[derive(Parser)]
struct Args {
    // CLI arguments
}

fn main() -> Result<()> {
    // 1. Parse arguments
    let args = Args::parse();

    // 2. Setup (load config, connect to DB, etc.)
    let dependencies = setup()?;

    // 3. Execute main logic
    run(args, dependencies)?;

    // 4. Cleanup (optional)
    cleanup()?;

    Ok(())
}

fn run(args: Args, deps: Dependencies) -> Result<()> {
    // Main logic here
}
```

**Testing:** End-to-end integration tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_full_workflow() {
        // Test the complete task from start to finish
    }
}
```

**Documentation:** Focus on usage instructions (CLI help, examples)

---

#### Grown-up Script Requirements

**Structure: Entry Point + Implementation Pattern**
```rust
// ALWAYS separate entry point from implementation

// ✅ Entry point (thin wrapper)
#[tauri::command]
pub async fn api_endpoint(
    params: Params,
    state: State<'_, AppState>
) -> Result<Response, String> {
    api_endpoint_impl(&state.db_pool, params)
        .await
        .map_err(|e| e.to_string())
}

// ✅ Implementation (testable, reusable)
pub async fn api_endpoint_impl(
    pool: &PgPool,
    params: Params
) -> Result<Response, ApiError> {
    // Real logic here
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_api_endpoint_impl() {
        // Test implementation with mocks
        let pool = create_test_pool().await;
        let result = api_endpoint_impl(&pool, test_params()).await;
        assert!(result.is_ok());
    }
}
```

**Why This Pattern?**
- Entry point handles framework-specific conversions
- Implementation is framework-agnostic and testable
- Other code can import and use `_impl` function
- Easy to mock dependencies in tests

**Testing:** Integration tests with mocked I/O (60%+ coverage)

**Documentation:** Document both entry point and implementation

---

#### Trusty Module Requirements (STRICTEST)

**Requirements (ALL MANDATORY):**
- ✅ Pure functions only
- ✅ No I/O operations
- ✅ No side effects
- ✅ 80%+ test coverage (REQUIRED)
- ✅ Comprehensive documentation
- ✅ Single responsibility
- ❌ NO `.unwrap()` or `.expect()`
- ❌ NO file/database/network access
- ❌ NO printing or logging
- ❌ NO global state modification

**Structure:**
```rust
// shared/rust/src/core/my_module.rs

/// Module documentation explaining purpose and usage
///
/// # Examples
/// Basic usage example here

use serde::{Serialize, Deserialize};

/// Function documentation (REQUIRED for all pub fns)
pub fn pure_function(input: &InputType) -> Result<OutputType, Error> {
    // Pure logic only - no I/O, no side effects
    // Same input ALWAYS produces same output
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputType {
    pub field: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQUIRED: Comprehensive unit tests

    #[test]
    fn test_basic_case() {
        let input = create_test_input();
        let result = pure_function(&input).unwrap();
        assert_eq!(result.field, "expected");
    }

    #[test]
    fn test_edge_case_1() {
        // Test edge cases
    }

    #[test]
    fn test_error_case() {
        // Test error conditions
    }
}
```

**CRITICAL RULE:** Everything in `core/` directories MUST be a Trusty Module.

**Verification:**
```bash
# Check for I/O in core/ (should return nothing)
grep -r "std::fs\|tokio::fs\|File::" shared/rust/src/core/
grep -r "sqlx\|database" shared/rust/src/core/
grep -r "println!\|print!" shared/rust/src/core/
```

**Testing:** 80%+ code coverage REQUIRED
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Check coverage
cargo tarpaulin --out Html --output-dir coverage

# Target: 80%+ for all files in core/
```

---

## 🧪 TESTING REQUIREMENTS

### By Archetype

| Archetype | Test Type | Coverage | Location |
|-----------|-----------|----------|----------|
| **Task-O-Matic** | End-to-end | As needed | `tests/e2e/` |
| **Grown-up Script** | Integration | 60%+ | `#[cfg(test)]` + `tests/integration/` |
| **Trusty Module** | Unit | 80%+ (REQUIRED) | `#[cfg(test)]` in same file |

### Test Organization

```
project/
├── src/
│   ├── core/              # Trusty Modules
│   │   └── my_module.rs   # Contains #[cfg(test)] mod tests
│   └── commands/          # Grown-up Scripts
│       └── my_cmd.rs      # Contains #[cfg(test)] mod tests
├── tests/
│   ├── integration/       # Integration tests for Grown-up Scripts
│   │   └── my_cmd_test.rs
│   └── e2e/              # End-to-end tests for Task-O-Matics
│       └── full_workflow_test.rs
└── bin/
    └── my-tool.rs         # Task-O-Matic
```

### Testing Best Practices

**Unit Tests (Trusty Modules):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test basic functionality
    #[test]
    fn test_basic_case() {
        let result = my_function(&valid_input());
        assert_eq!(result.unwrap(), expected_output());
    }

    // Test edge cases
    #[test]
    fn test_empty_input() {
        let result = my_function(&empty_input());
        assert!(result.is_ok());
    }

    // Test error conditions
    #[test]
    fn test_invalid_input() {
        let result = my_function(&invalid_input());
        assert!(matches!(result, Err(Error::InvalidInput(_))));
    }

    // Helper functions for test data
    fn valid_input() -> Input {
        Input { field: "test".to_string() }
    }
}
```

**Integration Tests (Grown-up Scripts):**
```rust
// tests/integration/repository_test.rs

use sqlx::PgPool;
use my_app::repositories::FileRepository;

#[sqlx::test]
async fn test_insert_and_retrieve(pool: PgPool) -> sqlx::Result<()> {
    let repo = FileRepository::new(&pool);

    // Insert test data
    let file = create_test_file();
    repo.insert(&file).await?;

    // Retrieve and verify
    let retrieved = repo.get_by_id(file.id).await?;
    assert_eq!(retrieved.name, file.name);

    Ok(())
}
```

### Coverage Enforcement

```bash
# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

# Check specific module coverage
cargo tarpaulin --out Stdout -- --test-threads 1 | grep "shared/rust/src/core"

# Fail build if coverage below threshold (CI/CD)
cargo tarpaulin --out Xml
# Parse XML and fail if < 80% for core/ modules
```

---

## 📚 EXAMPLES FROM THIS PROJECT

### Task-O-Matic Examples

#### 1. Import Tool CLI
```
Location: scripts/import-tool/src/main.rs
Purpose: Import MIDI files into database from command line
Entry: main() function
Reusable: No (standalone CLI only)
I/O: Yes (file system, database)
```

#### 2. Pipeline Tauri App
```
Location: pipeline/src-tauri/src/main.rs
Purpose: Launch Pipeline GUI application
Entry: main() function
Reusable: No (app launcher only)
I/O: Yes (initializes Tauri, database, etc.)
```

#### 3. DAW Tauri App
```
Location: daw/src-tauri/src/main.rs
Purpose: Launch DAW GUI application
Entry: main() function
Reusable: No (app launcher only)
I/O: Yes (initializes Tauri, audio, database)
```

---

### Grown-up Script Examples

#### 1. File Commands (Tauri Commands)
```
Location: daw/src-tauri/src/commands/file_commands.rs
Purpose: Handle file operations from GUI
Entry: #[tauri::command] functions
Reusable: Yes (_impl functions)
I/O: Yes (database)
Pattern: Entry point + implementation
```

```rust
// Entry point
#[tauri::command]
pub async fn search_files(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<File>, String> {
    search_files_impl(&state.db_pool, &query)
        .await
        .map_err(|e| e.to_string())
}

// Implementation
pub async fn search_files_impl(
    pool: &PgPool,
    query: &str
) -> Result<Vec<File>, DatabaseError> {
    // Database I/O here
}
```

#### 2. File Repository
```
Location: shared/rust/src/db/repositories/file_repository.rs
Purpose: Database operations for files table
Entry: No main, but public methods
Reusable: Yes (imported by commands)
I/O: Yes (PostgreSQL)
```

```rust
pub struct FileRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> FileRepository<'a> {
    pub async fn insert(&self, file: &File) -> Result<File, DbError> {
        // SQL query execution (I/O)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<File>, DbError> {
        // SQL query execution (I/O)
    }
}
```

#### 3. Playback Manager
```
Location: daw/src-tauri/src/playback/manager.rs
Purpose: Manage MIDI playback state
Entry: Public methods
Reusable: Yes
I/O: Yes (MIDI output devices)
```

---

### Trusty Module Examples

#### 1. MIDI Parser
```
Location: shared/rust/src/core/midi/parser.rs
Purpose: Parse MIDI file bytes into structured data
Entry: None (library only)
Reusable: Yes
I/O: No (pure parsing)
```

```rust
/// Parse MIDI file from bytes
pub fn parse_midi(bytes: &[u8]) -> Result<MidiFile, MidiError> {
    // Pure parsing logic - no I/O
    // Same bytes always produce same result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_valid_midi() {
        let bytes = include_bytes!("../../../tests/fixtures/test.mid");
        let result = parse_midi(bytes);
        assert!(result.is_ok());
    }
}
```

#### 2. BPM Detector
```
Location: shared/rust/src/core/analysis/bpm_detector.rs
Purpose: Detect BPM from MIDI tempo events
Entry: None
Reusable: Yes
I/O: No (pure analysis)
```

```rust
/// Detect BPM from parsed MIDI file
pub fn detect_bpm(midi: &MidiFile) -> Result<BpmAnalysis, BpmError> {
    // Pure algorithm - no I/O, no side effects
}
```

#### 3. Key Detector
```
Location: shared/rust/src/core/analysis/key_detector.rs
Purpose: Detect musical key from MIDI notes
Entry: None
Reusable: Yes
I/O: No (pure analysis)
```

---

## ⚠️ COMMON MISTAKES

### Mistake 1: Mixing Archetypes

**Problem:** Putting I/O code in `core/` directory

```rust
// ❌ BAD - This is in core/ but does I/O
// shared/rust/src/core/file_loader.rs

pub fn load_midi_file(path: &str) -> Result<MidiFile, Error> {
    let bytes = std::fs::read(path)?;  // ❌ I/O in core/
    parse_midi(&bytes)
}
```

**Solution:** Separate I/O from logic
```rust
// ✅ GOOD - I/O in grown-up script
// shared/rust/src/io/file_loader.rs

pub fn load_midi_file(path: &str) -> Result<MidiFile, Error> {
    let bytes = std::fs::read(path)?;  // ✅ I/O allowed here
    core::midi::parse_midi(&bytes)  // Calls pure function
}

// ✅ GOOD - Pure function in core/
// shared/rust/src/core/midi/parser.rs

pub fn parse_midi(bytes: &[u8]) -> Result<MidiFile, MidiError> {
    // Pure parsing - no I/O
}
```

---

### Mistake 2: Using .unwrap() in Production

```rust
// ❌ BAD
pub fn get_config() -> Config {
    let path = std::env::var("CONFIG_PATH").unwrap();  // Panic!
    let data = std::fs::read_to_string(path).unwrap();  // Panic!
    serde_json::from_str(&data).unwrap()  // Panic!
}

// ✅ GOOD
pub fn get_config() -> Result<Config, ConfigError> {
    let path = std::env::var("CONFIG_PATH")
        .map_err(|_| ConfigError::MissingEnvVar("CONFIG_PATH"))?;

    let data = std::fs::read_to_string(&path)
        .map_err(|e| ConfigError::ReadError { path, source: e })?;

    serde_json::from_str(&data)
        .map_err(ConfigError::ParseError)
}
```

---

### Mistake 3: Skipping Documentation

```rust
// ❌ BAD - No documentation
pub fn detect_bpm(midi: &MidiFile) -> Result<BpmAnalysis, BpmError> {
    // ...
}

// ✅ GOOD - Comprehensive documentation
/// Detects BPM from MIDI tempo events
///
/// Analyzes all tempo change events in the MIDI file and calculates
/// a weighted average BPM. If no tempo events exist, returns 120 BPM.
///
/// # Arguments
/// * `midi` - Parsed MIDI file to analyze
///
/// # Returns
/// * `Ok(BpmAnalysis)` - Detected BPM and confidence score (0.0-1.0)
/// * `Err(BpmError)` - If analysis fails
///
/// # Examples
/// ```
/// let midi = parse_midi(bytes)?;
/// let bpm = detect_bpm(&midi)?;
/// println!("BPM: {}", bpm.value);
/// ```
pub fn detect_bpm(midi: &MidiFile) -> Result<BpmAnalysis, BpmError> {
    // ...
}
```

---

### Mistake 4: Not Following Entry + Implementation Pattern

```rust
// ❌ BAD - All logic in Tauri command (not testable)
#[tauri::command]
pub async fn search_files(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<File>, String> {
    // All logic here - can't test without Tauri
    let result = sqlx::query_as!(
        File,
        "SELECT * FROM files WHERE name ILIKE $1",
        format!("%{}%", query)
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

// ✅ GOOD - Separate implementation
#[tauri::command]
pub async fn search_files(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<File>, String> {
    search_files_impl(&state.db_pool, &query)
        .await
        .map_err(|e| e.to_string())
}

pub async fn search_files_impl(
    pool: &PgPool,
    query: &str
) -> Result<Vec<File>, DbError> {
    // Testable implementation
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_search_files_impl() {
        // Easy to test!
        let pool = create_test_pool().await;
        let result = search_files_impl(&pool, "test").await;
        assert!(result.is_ok());
    }
}
```

---

### Mistake 5: Low Test Coverage

```rust
// ❌ BAD - Trusty Module with no tests
pub fn complex_algorithm(input: &Data) -> Result<Output, Error> {
    // 200 lines of complex logic
    // No tests!
}

// ✅ GOOD - Comprehensive tests
pub fn complex_algorithm(input: &Data) -> Result<Output, Error> {
    // 200 lines of complex logic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() { /* ... */ }

    #[test]
    fn test_edge_case_empty() { /* ... */ }

    #[test]
    fn test_edge_case_large() { /* ... */ }

    #[test]
    fn test_error_invalid_input() { /* ... */ }

    #[test]
    fn test_error_overflow() { /* ... */ }

    // 80%+ coverage achieved
}
```

---

## 🔗 INTEGRATION PATTERNS

### Pattern 1: Task-O-Matic → Grown-up Script → Trusty Module

**Flow:** User action → Orchestration → Pure logic

```
bin/import-tool.rs (Task-O-Matic)
    ↓ calls
src/import/processor.rs (Grown-up Script)
    ↓ calls
shared/core/midi/parser.rs (Trusty Module)
```

**Example:**
```rust
// bin/import-tool.rs (Task-O-Matic)
fn main() -> Result<()> {
    let args = Args::parse();

    // Setup
    let db = setup_database().await?;

    // Use Grown-up Script
    import::process_directory(&db, &args.path).await?;

    Ok(())
}

// src/import/processor.rs (Grown-up Script)
pub async fn process_directory(db: &PgPool, path: &str) -> Result<()> {
    for file_path in scan_dir(path)? {
        // Read file (I/O)
        let bytes = tokio::fs::read(&file_path).await?;

        // Use Trusty Module
        let midi = core::midi::parse_midi(&bytes)?;

        // Save to database (I/O)
        repository::insert_file(db, &midi).await?;
    }
    Ok(())
}

// shared/core/midi/parser.rs (Trusty Module)
pub fn parse_midi(bytes: &[u8]) -> Result<MidiFile, MidiError> {
    // Pure parsing logic
}
```

---

### Pattern 2: Frontend → Tauri Command → Repository → Database

```
Svelte Component
    ↓ invokes
Tauri Command (Grown-up Script entry point)
    ↓ calls
Command Implementation (Grown-up Script)
    ↓ uses
Repository (Grown-up Script)
    ↓ queries
PostgreSQL (external)
```

---

### Pattern 3: Shared Library Usage

**Principle:** Trusty Modules are shared across all components

```
shared/rust/src/core/
├── midi/parser.rs (Trusty Module)
├── analysis/bpm_detector.rs (Trusty Module)
└── analysis/key_detector.rs (Trusty Module)
    ↑ imported by ↑
┌───────────────────┴───────────────────┐
│                                       │
Pipeline (uses for batch)    DAW (uses for realtime)
```

---

## 📋 QUICK REFERENCE CHECKLIST

### When Creating New Code

- [ ] Which archetype does this belong to? (Use decision tree)
- [ ] Is it in the correct directory?
- [ ] If in `core/`, does it have NO I/O? (CRITICAL)
- [ ] Have I removed all `.unwrap()` and `.expect()`?
- [ ] Have I added documentation to all public functions?
- [ ] Have I written tests?
  - [ ] Task-O-Matic: E2E test
  - [ ] Grown-up Script: Integration test + 60%+ coverage
  - [ ] Trusty Module: Unit tests + 80%+ coverage
- [ ] Does it follow the entry + implementation pattern? (if Grown-up Script)
- [ ] Have I run `cargo fmt` and `cargo clippy`?
- [ ] Does `cargo doc` generate clean documentation?

---

## 🎯 CONCLUSION

**The Three Archetypes Pattern provides:**
1. **Clear organization** - Every file has an obvious home
2. **Quality standards** - Each archetype has specific requirements
3. **Testability** - Separation enables proper testing
4. **Maintainability** - Future developers know exactly what code does

**Remember:**
- Task-O-Matics = Complete tasks
- Grown-up Scripts = Orchestration + reusability
- Trusty Modules = Pure, tested logic

**Follow these rules and your codebase will be:**
- Easy to understand
- Easy to test
- Easy to maintain
- Production-ready

---

**For more information, see:**
- [PROJECT-STRUCTURE.md](./PROJECT-STRUCTURE.md) - Where files go
- [DEVELOPMENT-WORKFLOW.md](./DEVELOPMENT-WORKFLOW.md) - How to build features
- [CLAUDE.md](./CLAUDE.md) - Overall project guidance
