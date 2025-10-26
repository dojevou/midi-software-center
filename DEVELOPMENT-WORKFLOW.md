# DEVELOPMENT WORKFLOW

**The Construction Manual for MIDI Software Center**

**Date:** 2025-10-24
**Purpose:** Step-by-step guide for implementing features and fixing bugs
**Audience:** Developers, AI assistants, code reviewers

---

## 📋 TABLE OF CONTENTS

1. [Overview](#overview)
2. [The 8-Step Feature Implementation Process](#the-8-step-feature-implementation-process)
3. [Common Workflows](#common-workflows)
4. [Code Review Checklist](#code-review-checklist)
5. [Testing Strategy](#testing-strategy)
6. [Git Workflow](#git-workflow)
7. [Troubleshooting Guide](#troubleshooting-guide)

---

## 🎯 OVERVIEW

This workflow ensures every feature is:
- **Well-architected** - Follows Three Archetypes pattern
- **Well-tested** - Meets coverage requirements
- **Well-documented** - Has comprehensive documentation
- **Production-ready** - No `.unwrap()`, proper error handling

### Core Principles

1. **Plan before coding** - Classify files before writing
2. **Test as you go** - Write tests during implementation
3. **Document continuously** - Add docs while code is fresh
4. **Review thoroughly** - Use checklist before commits

---

## 🔨 THE 8-STEP FEATURE IMPLEMENTATION PROCESS

### Step 1: Understand the Requirement

**Goal:** Clarify what needs to be built and why

**Activities:**
1. Read the feature request or bug report
2. Identify affected components (Pipeline, DAW, both?)
3. Identify dependencies (database changes, new libraries, API changes)
4. Ask clarifying questions if anything is unclear

**Questions to Answer:**
- What problem does this solve?
- Which app(s) need this feature? (Pipeline, DAW, or both)
- Does this require database changes?
- Does this require new external dependencies?
- What are the acceptance criteria?

**Example:**
```
Feature Request: "Add tempo change detection to BPM analyzer"

Understanding:
- What: Detect multiple tempo changes within a single MIDI file
- Why: Files can have variable tempos, current detector assumes single tempo
- Where: Shared library (both apps use BPM detection)
- Dependencies: None (uses existing MIDI parser)
- Acceptance: Return array of tempo changes with timestamps
```

**Output:** Clear understanding of requirements

---

### Step 2: Design the Solution

**Goal:** Plan the architecture before writing code

**Activities:**
1. Use Three Archetypes decision tree to classify new code
2. Determine file locations using PROJECT-STRUCTURE.md
3. Identify what code is shared vs component-specific
4. Plan interfaces and data structures
5. Identify which existing code needs modification

**Decision Tree:**
```
Question 1: Will other code import/reuse this?
├─ NO  → Question 2: Is it a complete standalone task?
│        ├─ YES → Task-O-Matic (bin/, main.rs)
│        └─ NO  → Rethink (should probably be reusable)
│
└─ YES → Question 3: Does it also need to run standalone?
         ├─ YES → Grown-up Script (commands/, services/)
         └─ NO  → Question 4: Does it do I/O or side effects?
                  ├─ YES → Grown-up Script
                  └─ NO  → Trusty Module (core/)
```

**Template:**
```markdown
## Design: [Feature Name]

### Classification
- **Archetype**: [Task-O-Matic | Grown-up Script | Trusty Module]
- **Location**: [Exact file path]
- **Shared?**: [Yes - both apps | No - component-specific]

### Files to Create
- `path/to/new_file.rs` - [Purpose] - [Archetype]

### Files to Modify
- `path/to/existing.rs` - [What changes]

### Data Structures
```rust
pub struct NewType {
    // Fields
}
```

### Interfaces
```rust
pub fn new_function(params: Type) -> Result<Output, Error> {
    // Signature
}
```

### Dependencies
- [External crates needed]
- [Internal modules needed]
```

**Example:**
```markdown
## Design: Tempo Change Detection

### Classification
- **Archetype**: Trusty Module (pure analysis algorithm)
- **Location**: `shared/rust/src/core/analysis/tempo_detector.rs`
- **Shared?**: Yes - both Pipeline and DAW use it

### Files to Create
- `shared/rust/src/core/analysis/tempo_detector.rs` - Tempo change detection - Trusty Module

### Files to Modify
- `shared/rust/src/core/analysis/mod.rs` - Add `pub mod tempo_detector;`
- `shared/rust/src/core/analysis/bpm_detector.rs` - Use tempo detector

### Data Structures
```rust
pub struct TempoChange {
    pub timestamp_ms: u32,
    pub bpm: f64,
}

pub struct TempoAnalysis {
    pub changes: Vec<TempoChange>,
    pub average_bpm: f64,
}
```

### Interfaces
```rust
pub fn detect_tempo_changes(midi: &MidiFile) -> Result<TempoAnalysis, TempoError> {
    // Pure function - no I/O
}
```
```

**Output:** Written design document or notes

---

### Step 3: Set Up the Environment

**Goal:** Prepare for implementation

**Activities:**
1. Create feature branch: `git checkout -b feature/name`
2. Ensure database is running: `make db-up`
3. Ensure tests pass: `cargo test`
4. Install any new dependencies

**Commands:**
```bash
# Create feature branch
git checkout -b feature/tempo-change-detection

# Ensure clean state
git status

# Ensure database running (if needed)
docker-compose up -d postgres meilisearch

# Run tests to ensure starting from green state
cargo test --workspace
```

**Output:** Clean development environment ready for coding

---

### Step 4: Implement the Core Logic (Trusty Modules First)

**Goal:** Build pure, tested logic foundation

**Activities:**
1. Create Trusty Modules (pure logic) first
2. Write documentation WHILE coding (not after)
3. Write tests WHILE coding (not after)
4. Ensure 80%+ coverage for Trusty Modules

**Pattern:**
```rust
// shared/rust/src/core/analysis/tempo_detector.rs

/// Detect tempo changes in MIDI file
///
/// Analyzes MIDI tempo events and identifies distinct tempo changes.
/// Returns all tempo changes with timestamps and average BPM.
///
/// # Arguments
/// * `midi` - Parsed MIDI file to analyze
///
/// # Returns
/// * `Ok(TempoAnalysis)` - All tempo changes and average
/// * `Err(TempoError)` - If analysis fails
///
/// # Examples
/// ```
/// use midi_library::core::analysis::detect_tempo_changes;
///
/// let midi = parse_midi(&bytes)?;
/// let tempo = detect_tempo_changes(&midi)?;
/// assert_eq!(tempo.changes.len(), 3);  // 3 tempo changes
/// ```
pub fn detect_tempo_changes(midi: &MidiFile) -> Result<TempoAnalysis, TempoError> {
    // 1. Extract tempo events
    let tempo_events = extract_tempo_events(midi)?;

    if tempo_events.is_empty() {
        return Ok(TempoAnalysis {
            changes: vec![],
            average_bpm: 120.0,
        });
    }

    // 2. Group into distinct changes
    let changes = group_tempo_changes(&tempo_events)?;

    // 3. Calculate average
    let average_bpm = calculate_average_tempo(&changes);

    Ok(TempoAnalysis {
        changes,
        average_bpm,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempoChange {
    pub timestamp_ms: u32,
    pub bpm: f64,
}

#[derive(Debug, Clone)]
pub struct TempoAnalysis {
    pub changes: Vec<TempoChange>,
    pub average_bpm: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TempoError {
    #[error("Invalid tempo value: {0}")]
    InvalidTempo(f64),
}

// Helper functions (also pure)
fn extract_tempo_events(midi: &MidiFile) -> Result<Vec<TempoEvent>, TempoError> {
    // Implementation
}

fn group_tempo_changes(events: &[TempoEvent]) -> Result<Vec<TempoChange>, TempoError> {
    // Implementation
}

fn calculate_average_tempo(changes: &[TempoChange]) -> f64 {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tempo() {
        let midi = create_midi_with_tempo(120.0);
        let result = detect_tempo_changes(&midi).unwrap();
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.average_bpm, 120.0);
    }

    #[test]
    fn test_multiple_tempo_changes() {
        let midi = create_midi_with_tempos(vec![120.0, 140.0, 100.0]);
        let result = detect_tempo_changes(&midi).unwrap();
        assert_eq!(result.changes.len(), 3);
        assert_eq!(result.changes[0].bpm, 120.0);
        assert_eq!(result.changes[1].bpm, 140.0);
        assert_eq!(result.changes[2].bpm, 100.0);
    }

    #[test]
    fn test_no_tempo_events() {
        let midi = create_empty_midi();
        let result = detect_tempo_changes(&midi).unwrap();
        assert_eq!(result.changes.len(), 0);
        assert_eq!(result.average_bpm, 120.0);  // Default
    }

    #[test]
    fn test_invalid_tempo() {
        let midi = create_midi_with_tempo(-50.0);
        let result = detect_tempo_changes(&midi);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    // Helper to create test MIDI data
    fn create_midi_with_tempo(bpm: f64) -> MidiFile {
        // Create test data
    }

    fn create_midi_with_tempos(bpms: Vec<f64>) -> MidiFile {
        // Create test data
    }

    fn create_empty_midi() -> MidiFile {
        // Create test data
    }
}
```

**Key Points:**
- ✅ Documentation written FIRST (or alongside code)
- ✅ Tests written WHILE coding (not after)
- ✅ No `.unwrap()` or `.expect()`
- ✅ Proper error types with `thiserror`
- ✅ Pure functions (no I/O)
- ✅ Helper functions for test data

**Verify:**
```bash
# Run tests
cargo test tempo_detector

# Check coverage (should be 80%+)
cargo tarpaulin --out Stdout -- tempo_detector
```

**Output:** Fully tested, documented Trusty Module

---

### Step 5: Implement I/O Layer (Grown-up Scripts)

**Goal:** Add orchestration and I/O operations

**Activities:**
1. Implement Grown-up Scripts using Trusty Modules
2. Use entry point + implementation pattern for Tauri commands
3. Write integration tests
4. Target 60%+ coverage

**Pattern:**
```rust
// pipeline/src-tauri/src/commands/analysis_commands.rs

use midi_library::core::analysis::detect_tempo_changes;
use midi_library::core::midi::parse_midi;
use tauri::State;

/// Analyze tempo changes in a MIDI file (Tauri command entry point)
#[tauri::command]
pub async fn analyze_tempo(
    file_path: String,
    state: State<'_, AppState>
) -> Result<TempoAnalysisResult, String> {
    analyze_tempo_impl(&file_path, &state.db_pool)
        .await
        .map_err(|e| e.to_string())
}

/// Implementation (testable, reusable)
pub async fn analyze_tempo_impl(
    file_path: &str,
    pool: &PgPool
) -> Result<TempoAnalysisResult, AnalysisError> {
    // 1. Read file (I/O)
    let bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| AnalysisError::FileRead { path: file_path.to_string(), source: e })?;

    // 2. Parse (uses Trusty Module)
    let midi = parse_midi(&bytes)
        .map_err(AnalysisError::ParseError)?;

    // 3. Analyze (uses Trusty Module)
    let tempo_analysis = detect_tempo_changes(&midi)
        .map_err(AnalysisError::TempoError)?;

    // 4. Save to database (I/O)
    save_tempo_analysis(pool, file_path, &tempo_analysis).await?;

    // 5. Return result
    Ok(TempoAnalysisResult {
        changes: tempo_analysis.changes,
        average_bpm: tempo_analysis.average_bpm,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct TempoAnalysisResult {
    pub changes: Vec<TempoChange>,
    pub average_bpm: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Failed to read file {path}: {source}")]
    FileRead { path: String, source: std::io::Error },

    #[error("Failed to parse MIDI: {0}")]
    ParseError(#[from] midi_library::core::midi::MidiError),

    #[error("Tempo analysis failed: {0}")]
    TempoError(#[from] midi_library::core::analysis::TempoError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

async fn save_tempo_analysis(
    pool: &PgPool,
    file_path: &str,
    analysis: &TempoAnalysis
) -> Result<(), sqlx::Error> {
    // Save to database
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_tempo_impl() {
        // Create test file
        let test_file = create_test_midi_file();

        // Create test database
        let pool = create_test_pool().await;

        // Test implementation
        let result = analyze_tempo_impl(&test_file, &pool).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.average_bpm > 0.0);
    }

    #[tokio::test]
    async fn test_analyze_tempo_file_not_found() {
        let pool = create_test_pool().await;
        let result = analyze_tempo_impl("/nonexistent/file.mid", &pool).await;
        assert!(matches!(result, Err(AnalysisError::FileRead { .. })));
    }
}
```

**Key Points:**
- ✅ Entry point + implementation pattern
- ✅ Uses Trusty Modules for logic
- ✅ Handles I/O operations
- ✅ Proper error handling with context
- ✅ Integration tests with mocked database

**Output:** Working Grown-up Scripts with integration tests

---

### Step 6: Implement UI (If Needed)

**Goal:** Add frontend interface

**Activities:**
1. Create Svelte components
2. Connect to backend via Tauri commands
3. Add state management
4. Test user interactions

**Pattern:**
```svelte
<!-- pipeline/src/lib/components/analysis/TempoAnalysis.svelte -->

<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { createEventDispatcher } from 'svelte';

  export let filePath: string;

  let loading = false;
  let error: string | null = null;
  let result: TempoAnalysisResult | null = null;

  interface TempoChange {
    timestamp_ms: number;
    bpm: number;
  }

  interface TempoAnalysisResult {
    changes: TempoChange[];
    average_bpm: number;
  }

  const dispatch = createEventDispatcher();

  async function analyzeTempo() {
    loading = true;
    error = null;

    try {
      result = await invoke<TempoAnalysisResult>('analyze_tempo', {
        filePath
      });
      dispatch('analyzed', result);
    } catch (err) {
      error = err as string;
    } finally {
      loading = false;
    }
  }
</script>

<div class="tempo-analysis">
  <button on:click={analyzeTempo} disabled={loading}>
    {loading ? 'Analyzing...' : 'Analyze Tempo'}
  </button>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if result}
    <div class="results">
      <h3>Tempo Analysis</h3>
      <p>Average BPM: {result.average_bpm.toFixed(1)}</p>

      {#if result.changes.length > 1}
        <h4>Tempo Changes:</h4>
        <ul>
          {#each result.changes as change}
            <li>
              {(change.timestamp_ms / 1000).toFixed(2)}s: {change.bpm.toFixed(1)} BPM
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  .tempo-analysis {
    padding: 1rem;
  }

  .error {
    color: red;
    margin-top: 0.5rem;
  }

  .results {
    margin-top: 1rem;
  }

  ul {
    list-style: none;
    padding: 0;
  }

  li {
    padding: 0.25rem 0;
  }
</style>
```

**State Management:**
```typescript
// pipeline/src/lib/stores/analysisStore.ts

import { writable } from 'svelte/store';

interface AnalysisState {
  currentFile: string | null;
  tempo: TempoAnalysisResult | null;
  loading: boolean;
  error: string | null;
}

export const analysisStore = writable<AnalysisState>({
  currentFile: null,
  tempo: null,
  loading: false,
  error: null
});
```

**Output:** Working UI connected to backend

---

### Step 7: Test Everything

**Goal:** Ensure all code meets quality standards

**Activities:**
1. Run all tests: `cargo test --workspace`
2. Check test coverage: `cargo tarpaulin`
3. Verify Trusty Modules have 80%+ coverage
4. Run frontend tests: `pnpm test`
5. Manual testing of UI

**Commands:**
```bash
# Run all Rust tests
cargo test --workspace

# Check coverage
cargo tarpaulin --out Html --output-dir coverage

# Focus on specific module
cargo tarpaulin --out Stdout -- tempo_detector

# Run clippy (linter)
cargo clippy --all-targets --all-features -- -D warnings

# Run formatter
cargo fmt --all -- --check

# Frontend tests
cd pipeline && pnpm test

# Frontend linting
cd pipeline && pnpm lint
```

**Coverage Requirements:**
- Trusty Modules: 80%+ (REQUIRED)
- Grown-up Scripts: 60%+
- Task-O-Matics: As needed

**Manual Testing Checklist:**
- [ ] Feature works with valid input
- [ ] Feature handles invalid input gracefully
- [ ] Error messages are clear
- [ ] UI is responsive
- [ ] Database changes persist correctly

**Output:** All tests passing, coverage requirements met

---

### Step 8: Code Review and Commit

**Goal:** Ensure code quality before merging

**Activities:**
1. Self-review using checklist (below)
2. Generate documentation: `cargo doc`
3. Create meaningful commit message
4. Push and create pull request

**Self-Review Checklist:**
```markdown
## Code Review Checklist

### Architecture
- [ ] Correct archetype classification
- [ ] Files in correct directories
- [ ] No I/O in `core/` directories
- [ ] Follows entry + implementation pattern (if Grown-up Script)

### Code Quality
- [ ] No `.unwrap()` or `.expect()` in production code
- [ ] Proper error handling with `?` operator
- [ ] Proper error types with `thiserror`
- [ ] Performance-conscious (e.g., `&str` not `String` for params)

### Documentation
- [ ] All public functions have doc comments
- [ ] Doc comments include examples
- [ ] Doc comments explain errors
- [ ] `cargo doc` generates clean output

### Testing
- [ ] Trusty Modules have 80%+ coverage
- [ ] Grown-up Scripts have integration tests
- [ ] Edge cases tested
- [ ] Error conditions tested
- [ ] All tests pass

### Security/Safety
- [ ] No SQL injection vulnerabilities
- [ ] No path traversal vulnerabilities
- [ ] Sensitive data not logged
- [ ] No secrets in code

### Git
- [ ] Meaningful commit message
- [ ] One logical change per commit
- [ ] No commented-out code
- [ ] No debug print statements
```

**Commit Message Template:**
```
[Component] Brief description (50 chars or less)

More detailed explanation of what this commit does and why.
Include motivation for the change and contrast with previous behavior.

- Bullet points for key changes
- Another change
- Reference to issue: Fixes #123

Test plan:
- How to test this change
- Expected behavior
```

**Example:**
```
[Shared] Add tempo change detection to BPM analyzer

The existing BPM detector assumed a single constant tempo throughout
the MIDI file. Real-world files often have tempo changes (ritardando,
accelerando, etc.). This commit adds detection of all tempo changes
with timestamps.

- Added TempoChange and TempoAnalysis types
- Implemented detect_tempo_changes() in shared/rust/src/core/analysis/
- Added 80%+ test coverage
- Updated bpm_detector to use new tempo detection
- Added UI component in Pipeline to display tempo changes

Fixes #42

Test plan:
- cargo test tempo_detector
- Import MIDI file with multiple tempos
- Verify tempo changes displayed in Pipeline
```

**Commands:**
```bash
# Generate documentation
cargo doc --no-deps --open

# Stage changes
git add .

# Commit
git commit -m "[Shared] Add tempo change detection to BPM analyzer

... (full message)
"

# Push to remote
git push origin feature/tempo-change-detection

# Create pull request (via GitHub CLI)
gh pr create --title "Add tempo change detection" --body "$(cat <<EOF
## Summary
Adds detection of tempo changes within MIDI files.

## Changes
- Added tempo_detector.rs to shared library
- Updated BPM detector to use tempo changes
- Added Pipeline UI for displaying tempo changes

## Testing
- All tests pass
- Coverage: 85% (exceeds 80% requirement)
- Manual testing with variable-tempo MIDI files

## Screenshots
[Include if UI changes]

Fixes #42
EOF
)"
```

**Output:** Clean commit ready for review

---

## 🔄 COMMON WORKFLOWS

### Workflow 1: Adding a Pure Algorithm (Trusty Module)

**Example:** Add chord progression detector

**Steps:**
1. **Design**: Pure algorithm, no I/O → Trusty Module in `shared/rust/src/core/analysis/`
2. **Create file**: `shared/rust/src/core/analysis/chord_progression.rs`
3. **Implement with tests**:
   ```rust
   pub fn analyze_chord_progression(notes: &[Note]) -> Result<Vec<Chord>, ChordError> {
       // Implementation
   }

   #[cfg(test)]
   mod tests {
       // 80%+ coverage
   }
   ```
4. **Export**: Add to `shared/rust/src/core/analysis/mod.rs`
5. **Test**: `cargo test chord_progression`
6. **Coverage**: `cargo tarpaulin -- chord_progression` (verify 80%+)
7. **Document**: Ensure all pub fns have doc comments
8. **Commit**: "[ Shared] Add chord progression analyzer"

---

### Workflow 2: Adding a Tauri Command (Grown-up Script)

**Example:** Add "export to JSON" feature in Pipeline

**Steps:**
1. **Design**: Tauri command → Grown-up Script in `pipeline/src-tauri/src/commands/`
2. **Create/modify**: `pipeline/src-tauri/src/commands/export_commands.rs`
3. **Implement** with entry + implementation pattern:
   ```rust
   #[tauri::command]
   pub async fn export_to_json(path: String, state: State<'_, AppState>) -> Result<(), String> {
       export_to_json_impl(&path, &state.db_pool).await.map_err(|e| e.to_string())
   }

   pub async fn export_to_json_impl(path: &str, pool: &PgPool) -> Result<(), ExportError> {
       // Implementation with I/O
   }

   #[cfg(test)]
   mod tests {
       // Integration tests
   }
   ```
4. **Register**: Add to `invoke_handler![]` in `pipeline/src-tauri/src/main.rs`
5. **Frontend**: Add UI button that calls `invoke('export_to_json', { path })`
6. **Test**: Integration test + manual UI test
7. **Commit**: "[Pipeline] Add JSON export feature"

---

### Workflow 3: Adding Database Migration

**Example:** Add "favorites" column to files table

**Steps:**
1. **Create migration**: `database/migrations/007_add_favorites.sql`
   ```sql
   -- Add favorites column
   ALTER TABLE files ADD COLUMN is_favorite BOOLEAN NOT NULL DEFAULT false;

   -- Add index for performance
   CREATE INDEX idx_files_favorite ON files(is_favorite) WHERE is_favorite = true;
   ```
2. **Update model**: Modify `shared/rust/src/db/models/file.rs`
   ```rust
   pub struct File {
       // ... existing fields
       pub is_favorite: bool,
   }
   ```
3. **Update repository**: Modify queries in `shared/rust/src/db/repositories/file_repository.rs`
4. **Update TypeScript types**: `shared/typescript/src/types/file.ts`
5. **Test**: Run migration on test database
   ```bash
   sqlx migrate run --database-url $TEST_DATABASE_URL
   ```
6. **Commit**: "[Database] Add favorites feature to files table"

---

### Workflow 4: Fixing a Bug

**Steps:**
1. **Reproduce**: Write a failing test that reproduces the bug
2. **Locate**: Find the code causing the bug
3. **Fix**: Implement fix
4. **Verify**: Ensure test now passes
5. **Regression test**: Ensure fix doesn't break other tests
6. **Commit**: "[Component] Fix bug in ..."

**Example:**
```rust
// Bug: BPM detector crashes on MIDI files with no tempo events

// Step 1: Add failing test
#[test]
fn test_no_tempo_events() {
    let midi = create_empty_midi();
    let result = detect_bpm(&midi);  // Currently panics
    assert!(result.is_ok());  // Should return default BPM
}

// Step 2: Locate - detect_bpm calls tempo_events[0].unwrap()

// Step 3: Fix
pub fn detect_bpm(midi: &MidiFile) -> Result<BpmAnalysis, BpmError> {
    let tempo_events = extract_tempo_events(midi)?;

    if tempo_events.is_empty() {
        return Ok(BpmAnalysis {
            bpm: 120.0,  // Default
            confidence: 0.0,
        });
    }

    // Rest of logic
}

// Step 4: Test passes
// Step 5: All other tests still pass
// Step 6: Commit with message explaining the fix
```

---

### Workflow 5: Refactoring

**Example:** Extract duplicated code into shared utility

**Steps:**
1. **Identify duplication**: Find repeated code across components
2. **Classify**: Determine archetype (usually Trusty Module if pure)
3. **Extract**: Create shared function
4. **Test**: Ensure original behavior preserved
5. **Replace**: Update call sites to use new shared function
6. **Cleanup**: Remove old duplicated code
7. **Commit**: "[Refactor] Extract common ... into shared utility"

**Example:**
```rust
// Before: Duplicated validation in Pipeline and DAW
// pipeline/src-tauri/src/validation.rs
fn validate_midi_path(path: &str) -> bool {
    path.ends_with(".mid") || path.ends_with(".midi")
}

// daw/src-tauri/src/validation.rs
fn validate_midi_path(path: &str) -> bool {
    path.ends_with(".mid") || path.ends_with(".midi")
}

// After: Shared Trusty Module
// shared/rust/src/utils/validation.rs
/// Validate MIDI file path by extension
pub fn is_midi_file(path: &str) -> bool {
    path.ends_with(".mid") || path.ends_with(".midi")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_midi_file() {
        assert!(is_midi_file("test.mid"));
        assert!(is_midi_file("test.midi"));
        assert!(!is_midi_file("test.mp3"));
    }
}

// Update call sites in both components
use midi_library::utils::validation::is_midi_file;
```

---

## ✅ CODE REVIEW CHECKLIST

Use this checklist before every commit:

### Architecture & Structure

- [ ] **Archetype Classification**
  - Is this the correct archetype? (Task-O-Matic, Grown-up Script, Trusty Module)
  - Have I used the decision tree to verify?

- [ ] **File Location**
  - Is the file in the correct directory?
  - If in `core/`, does it contain ONLY pure functions?
  - If shared, is it in `shared/` not component directory?

- [ ] **Separation of Concerns**
  - Trusty Modules contain no I/O
  - Grown-up Scripts use entry + implementation pattern
  - Task-O-Matics have clear entry points

### Code Quality

- [ ] **Error Handling**
  - No `.unwrap()` or `.expect()` in production code
  - All errors propagated with `?` operator
  - Proper error types (use `thiserror` for libraries, `anyhow` for apps)
  - Error messages are clear and actionable

- [ ] **Type Safety**
  - No `unsafe` code (unless absolutely necessary with justification)
  - No `as` casts without bounds checking
  - Proper null/None handling

- [ ] **Performance**
  - Use `&str` instead of `String` for parameters when possible
  - Avoid unnecessary clones
  - Use appropriate data structures (Vec not LinkedList)

### Documentation

- [ ] **Public API Documentation**
  - All `pub fn` have doc comments (`///`)
  - Doc comments include purpose, arguments, returns, errors
  - Doc comments include examples
  - Examples compile and run correctly

- [ ] **Code Comments**
  - Complex logic has explanatory comments
  - No commented-out code (remove it)
  - No TODO comments without issue numbers

### Testing

- [ ] **Test Coverage**
  - Trusty Modules: 80%+ coverage (REQUIRED)
  - Grown-up Scripts: 60%+ coverage with integration tests
  - Task-O-Matics: E2E tests as needed

- [ ] **Test Quality**
  - Tests cover happy path
  - Tests cover edge cases
  - Tests cover error conditions
  - Tests are deterministic (no flaky tests)

- [ ] **Test Organization**
  - Unit tests in `#[cfg(test)]` modules
  - Integration tests in `tests/integration/`
  - Test helpers are reusable

### Security

- [ ] **Input Validation**
  - All user input is validated
  - No SQL injection vulnerabilities (using parameterized queries)
  - No path traversal vulnerabilities (validate file paths)

- [ ] **Data Safety**
  - Sensitive data not logged
  - No secrets in code (use environment variables)
  - Proper access control

### Git

- [ ] **Commit Quality**
  - Meaningful commit message
  - One logical change per commit
  - Commit message explains "why" not just "what"

- [ ] **Clean History**
  - No merge commits (rebase instead)
  - No "WIP" or "tmp" commits in PR

- [ ] **Branch Hygiene**
  - Branch name is descriptive (`feature/name`, `fix/name`)
  - Branch is up to date with main

---

## 🧪 TESTING STRATEGY

### Test Pyramid

```
           ╱───────────╲
          ╱   E2E Tests  ╲         ← Few, slow, comprehensive
         ╱───────────────╲
        ╱  Integration     ╲        ← Medium, moderate speed
       ╱─────────────────────╲
      ╱      Unit Tests       ╲     ← Many, fast, focused
     ╱─────────────────────────╲
```

### Unit Tests (Trusty Modules)

**Purpose:** Test pure logic in isolation

**Location:** `#[cfg(test)]` modules in same file

**Coverage:** 80%+ (REQUIRED)

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let input = create_test_input();
        let result = my_function(&input).unwrap();
        assert_eq!(result, expected_output());
    }

    #[test]
    fn test_edge_case_empty() {
        let result = my_function(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_invalid_input() {
        let result = my_function(&invalid_input());
        assert!(matches!(result, Err(MyError::InvalidInput(_))));
    }
}
```

### Integration Tests (Grown-up Scripts)

**Purpose:** Test I/O operations with mocked/test resources

**Location:** `tests/integration/` or `#[cfg(test)]` modules

**Coverage:** 60%+

**Example:**
```rust
// tests/integration/repository_test.rs

use sqlx::PgPool;
use midi_library::db::repositories::FileRepository;

#[sqlx::test]
async fn test_file_repository_crud(pool: PgPool) -> sqlx::Result<()> {
    let repo = FileRepository::new(&pool);

    // Create
    let file = create_test_file();
    let created = repo.insert(&file).await?;
    assert_eq!(created.name, file.name);

    // Read
    let found = repo.find_by_id(created.id).await?;
    assert!(found.is_some());

    // Update
    let mut updated = created;
    updated.name = "new_name.mid".to_string();
    repo.update(&updated).await?;

    // Delete
    repo.delete(updated.id).await?;
    let deleted = repo.find_by_id(updated.id).await?;
    assert!(deleted.is_none());

    Ok(())
}
```

### End-to-End Tests (Task-O-Matics)

**Purpose:** Test complete workflows

**Location:** `tests/e2e/` or manual testing

**Example:**
```rust
// tests/e2e/import_workflow_test.rs

#[tokio::test]
async fn test_complete_import_workflow() {
    // 1. Set up test environment
    let temp_dir = create_temp_dir();
    let test_midi = copy_test_midi_to(&temp_dir);
    let db = create_test_database().await;

    // 2. Run import
    let result = run_import_cli(&test_midi, &db).await;
    assert!(result.is_ok());

    // 3. Verify in database
    let files = query_database(&db).await;
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "test.mid");

    // 4. Verify analysis results
    assert!(files[0].bpm.is_some());
    assert!(files[0].key.is_some());

    // 5. Cleanup
    cleanup_temp_dir(temp_dir);
    cleanup_database(db).await;
}
```

### Test Data Management

**Fixtures:**
```rust
// tests/fixtures/mod.rs

pub fn sample_midi_bytes() -> &'static [u8] {
    include_bytes!("sample.mid")
}

pub fn create_test_file() -> File {
    File {
        id: 1,
        name: "test.mid".to_string(),
        path: "/tmp/test.mid".to_string(),
        size: 1024,
        // ... other fields
    }
}

pub fn create_test_pool() -> PgPool {
    // Create test database connection
}
```

---

## 🔀 GIT WORKFLOW

### Branch Strategy

```
main (protected)
├── feature/tempo-detection     ← Feature branches
├── feature/chord-analysis
├── fix/bpm-crash              ← Bug fix branches
└── refactor/extract-validation ← Refactoring branches
```

### Branch Naming

**Pattern:** `type/short-description`

**Types:**
- `feature/` - New features
- `fix/` - Bug fixes
- `refactor/` - Code refactoring
- `docs/` - Documentation updates
- `test/` - Test improvements

**Examples:**
- `feature/tempo-change-detection`
- `fix/bpm-detector-crash`
- `refactor/extract-validation`
- `docs/update-architecture-guide`

### Commit Message Format

```
[Component] Brief description (50 chars)

Detailed explanation of what and why (wrap at 72 chars).

- Bullet points for key changes
- Another change
- Fixes #123

Test plan:
- How to test
- Expected results
```

**Component Tags:**
- `[Shared]` - Changes to shared library
- `[Database]` - Database schema changes
- `[Pipeline]` - Pipeline app changes
- `[DAW]` - DAW app changes
- `[Scripts]` - Script/automation changes
- `[Docs]` - Documentation changes
- `[CI]` - CI/CD changes

### Pull Request Process

1. **Create PR** with descriptive title and description
2. **Fill out PR template**:
   ```markdown
   ## Summary
   Brief description of changes

   ## Motivation
   Why this change is needed

   ## Changes
   - List of key changes
   - Another change

   ## Testing
   - How this was tested
   - Coverage results

   ## Screenshots
   (If UI changes)

   ## Checklist
   - [ ] Tests pass
   - [ ] Coverage meets requirements
   - [ ] Documentation updated
   - [ ] Reviewed own code
   ```
3. **Request review** from team members
4. **Address feedback** with additional commits
5. **Merge** when approved (squash or rebase, no merge commits)

---

## 🔧 TROUBLESHOOTING GUIDE

### Problem: Tests Failing After Changes

**Symptoms:** `cargo test` fails

**Steps:**
1. Read error message carefully
2. Identify which test is failing
3. Run specific test: `cargo test test_name -- --nocapture`
4. Check if test expectations match new behavior
5. Update test if behavior change is intentional
6. Fix code if behavior change is a bug

### Problem: Low Test Coverage

**Symptoms:** `cargo tarpaulin` shows < 80% for Trusty Module

**Steps:**
1. Identify uncovered lines: `cargo tarpaulin --out Html`
2. Open `coverage/index.html` in browser
3. Find red (uncovered) lines
4. Add tests for those code paths
5. Verify coverage: `cargo tarpaulin -- module_name`

### Problem: Compilation Errors

**Symptoms:** `cargo build` fails

**Common Causes:**
1. **Missing imports** - Add `use` statements
2. **Type mismatches** - Check function signatures
3. **Lifetime issues** - Adjust lifetime annotations
4. **Missing dependencies** - Add to `Cargo.toml`

**Steps:**
1. Read compiler error message
2. Follow compiler suggestions (Rust errors are very helpful!)
3. Check documentation: `cargo doc --open`

### Problem: Tauri Command Not Found

**Symptoms:** Frontend gets "command not found" error

**Steps:**
1. Verify command is registered in `main.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       commands::my_command,  // ← Add here
   ])
   ```
2. Verify command has `#[tauri::command]` attribute
3. Verify command is `pub`
4. Rebuild Tauri: `cargo build`

### Problem: Database Migration Fails

**Symptoms:** `sqlx migrate run` fails

**Steps:**
1. Check database is running: `docker-compose ps`
2. Check connection string: `echo $DATABASE_URL`
3. Check migration syntax (copy to SQL client and test)
4. Check for schema conflicts
5. Rollback if needed: `sqlx migrate revert`

### Problem: Frontend Not Updating

**Symptoms:** Changes don't appear in GUI

**Steps:**
1. Check if Vite dev server is running
2. Hard refresh browser (Ctrl+Shift+R)
3. Check browser console for errors
4. Check if Tauri backend was rebuilt
5. Restart Tauri dev server: `pnpm tauri dev`

---

## 📋 QUICK REFERENCE

### Essential Commands

```bash
# Development
make dev              # Start all services + apps in dev mode
make db-up            # Start database services only
make test             # Run all tests
make lint             # Run linters
make fmt              # Format code

# Testing
cargo test --workspace              # All Rust tests
cargo test module_name              # Specific module
cargo tarpaulin --out Html          # Coverage report
pnpm test                          # Frontend tests

# Building
cargo build --release              # Production Rust build
pnpm build                         # Frontend build
make build                         # Build everything

# Database
sqlx migrate run                   # Run migrations
sqlx migrate revert                # Rollback last migration
sqlx database create               # Create database

# Documentation
cargo doc --no-deps --open         # Generate and open docs
```

### File Templates

See [ARCHITECTURE-REFERENCE.md](./ARCHITECTURE-REFERENCE.md) for complete templates:
- Trusty Module template
- Grown-up Script template
- Task-O-Matic template
- Test template

---

## 🎯 CONCLUSION

**This workflow ensures:**
1. **Quality** - Every feature is well-tested and documented
2. **Consistency** - All code follows the same patterns
3. **Maintainability** - Future developers can easily understand and extend

**Key Takeaways:**
- **Plan before coding** - Use decision tree to classify archetype
- **Test as you go** - Write tests during implementation
- **Document continuously** - Add docs while code is fresh
- **Review thoroughly** - Use checklist before every commit

**Remember the 8 steps:**
1. Understand requirement
2. Design solution
3. Set up environment
4. Implement Trusty Modules
5. Implement Grown-up Scripts
6. Implement UI
7. Test everything
8. Review and commit

---

**For more information, see:**
- [ARCHITECTURE-REFERENCE.md](./ARCHITECTURE-REFERENCE.md) - Three Archetypes pattern
- [PROJECT-STRUCTURE.md](./PROJECT-STRUCTURE.md) - Where files go
- [CLAUDE.md](./CLAUDE.md) - Overall project guidance
