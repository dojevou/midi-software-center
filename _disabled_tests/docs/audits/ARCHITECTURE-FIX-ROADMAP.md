# Architecture Fix Roadmap - Detailed Implementation Guide

**Reference:** ARCHITECTURE-AUDIT-REPORT.md
**Timeline:** 5-8 weeks to complete all recommendations
**Status:** Actionable items with code examples

---

## QUICK START: What to Fix First

### This Week (P0 - Critical)
1. [ ] Audit all 351 unwrap() calls - categorize by risk
2. [ ] Map command-to-command dependencies
3. [ ] Create plan for critical unwrap() replacement

### Week 2-3 (P1 - High Priority)
1. [ ] Replace critical unwrap() with proper error handling
2. [ ] Extract shared logic from command-to-command calls
3. [ ] Create core module structure for common patterns

### Week 4-5 (P2 - Medium Priority)
1. [ ] Refactor large modules (orchestrator, file_import, analyze)
2. [ ] Implement error recovery in import loop
3. [ ] Add database telemetry

---

## DETAILED FIXES

### Fix 1: Reduce unwrap() Calls (P0)

**Current State:** 351 unwrap() calls in production code

**Target State:** <50 unwrap() calls (only safe after defensive checks)

#### Step 1.1: Audit and Categorize

```bash
# First, find all unwrap locations
grep -r "unwrap()" pipeline/src-tauri/src --include="*.rs" | \
  grep -v "test\|example" | \
  sed 's/:.*unwrap()//' | \
  sort | uniq -c | sort -rn

# Output will show which files have most unwrap() calls
# Focus on: orchestrator.rs, file_import.rs, analyze.rs first
```

#### Step 1.2: Create Unwrap Audit Spreadsheet

Create `docs/unwrap-audit.csv`:
```csv
file,line,context,risk_level,can_panic,replacement
pipeline/src-tauri/src/core/pipeline/orchestrator.rs,123,after validation,LOW,No,unnecessary
pipeline/src-tauri/src/commands/file_import.rs,456,database operation,HIGH,Yes,Use map_err
```

#### Step 1.3: Replace by Risk Level

**HIGH RISK unwrap() to fix immediately:**
```rust
// BEFORE: Can panic during import
let value = database_operation().unwrap();

// AFTER: Proper error handling
let value = database_operation()
    .map_err(|e| AppError::DatabaseError(e))?;
```

**PATTERN 1: After error check (Safe to keep)**
```rust
// OK to keep - already validated
if value.is_some() {
    let v = value.unwrap();  // ✓ Safe
}
```

**PATTERN 2: After defensive check (OK to keep)**
```rust
// OK to keep - defensively safe
let value = vec![1, 2, 3];
let first = value.get(0).unwrap();  // After explicit check

// Better:
let first = value.first();  // Returns Option, no unwrap
```

**PATTERN 3: During initialization (Keep documented)**
```rust
// OK if panicking on init is acceptable
let config = Config::from_file()
    .expect("Config file required at startup");
```

**PATTERN 4: In test code (Always OK)**
```rust
#[test]
fn test_something() {
    let result = expensive_operation().unwrap();  // ✓ OK in tests
}
```

#### Step 1.4: Implement Phased Replacement

**Phase A: Database operations** (50 unwrap calls)
```rust
// file_repository.rs
// BEFORE:
let result = sqlx::query!(...).fetch_one(pool).await.unwrap();

// AFTER:
let result = sqlx::query!(...).fetch_one(pool).await
    .map_err(|e| AppError::DatabaseError(e))?;
```

**Phase B: File I/O operations** (80 unwrap calls)
```rust
// file_import.rs
// BEFORE:
let metadata = std::fs::metadata(path).unwrap();

// AFTER:
let metadata = std::fs::metadata(path)
    .map_err(|e| AppError::IOError(e))?;
```

**Phase C: Parsing operations** (90 unwrap calls)
```rust
// midi_parser.rs
// BEFORE:
let midi_data = midly::Smf::parse(bytes).unwrap();

// AFTER:
let midi_data = midly::Smf::parse(bytes)
    .map_err(|e| AppError::MidiError(format!("{}", e)))?;
```

**Phase D: Configuration** (30 unwrap calls)
```rust
// main.rs
// BEFORE:
let db_url = std::env::var("DATABASE_URL").unwrap();

// AFTER:
let db_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
```

#### Step 1.5: Add Error Recovery in Import Loop

```rust
// commands/file_import.rs - NEW ERROR RECOVERY PATTERN

pub async fn import_with_recovery(
    pool: &PgPool,
    files: Vec<PathBuf>,
) -> Result<ImportStats, AppError> {
    let mut successful = 0;
    let mut failed = 0;
    let mut failed_files = Vec::new();

    for (index, file_path) in files.iter().enumerate() {
        match import_single_file(pool, file_path).await {
            Ok(_) => successful += 1,
            Err(e) => {
                failed += 1;
                failed_files.push((file_path.clone(), e.to_string()));
                // Log but continue - don't panic
                tracing::warn!(
                    "Failed to import file {}/{}: {}",
                    index + 1,
                    files.len(),
                    e
                );
            }
        }
    }

    // Return success even if some files failed
    Ok(ImportStats {
        successful,
        failed,
        failed_files,
    })
}
```

---

### Fix 2: Decouple Command-to-Command Dependencies (P0)

**Current Problem:** Commands call other commands, creating circular dependencies

**Current Pattern (BAD):**
```
split_file.rs ──→ file_import.rs  (depends on command)
archive_import.rs ──→ file_import.rs
```

**Target Pattern (GOOD):**
```
split_file.rs ──┐
archive_import.rs ├──→ core/import_operations.rs (shared module)
file_import.rs ──┘
```

#### Step 2.1: Extract Common Import Logic

Create `pipeline/src-tauri/src/core/import_operations.rs`:

```rust
/// Core import operations module (Trusty Module)
/// NO I/O, NO side effects, pure business logic

use crate::db::models::NewFile;
use crate::core::hash::calculate_file_hash;

/// Validate and prepare file for import
pub fn validate_file_for_import(
    path: &std::path::Path,
    file_size: u64,
) -> Result<ValidatedFile, ValidationError> {
    // Pure validation logic
    if !path.exists() {
        return Err(ValidationError::NotFound);
    }
    if file_size == 0 {
        return Err(ValidationError::Empty);
    }
    Ok(ValidatedFile { path: path.to_path_buf() })
}

/// Prepare file metadata (no I/O, pure calculation)
pub fn prepare_file_metadata(
    path: &std::path::Path,
    file_size: u64,
) -> NewFile {
    // Pure transformation - no I/O
    NewFile {
        filename: path.file_name().unwrap().to_string_lossy().to_string(),
        filepath: path.to_string_lossy().to_string(),
        file_size_bytes: file_size as i64,
        // ... other fields
    }
}

#[derive(Debug)]
pub struct ValidatedFile {
    pub path: std::path::PathBuf,
}

#[derive(Debug)]
pub enum ValidationError {
    NotFound,
    Empty,
    InvalidFormat,
}
```

#### Step 2.2: Update commands/file_import.rs

```rust
/// File import command (Grown-up Script)
/// Performs I/O, delegates logic to core modules

use crate::core::import_operations;

pub async fn import_single_file(
    pool: &PgPool,
    path: &std::path::Path,
) -> Result<i64, AppError> {
    // Step 1: Read file metadata (I/O)
    let file_size = std::fs::metadata(path)
        .map_err(|e| AppError::IOError(e))?
        .len();

    // Step 2: Validate using core module (pure logic)
    let _validated = import_operations::validate_file_for_import(path, file_size)
        .map_err(|e| AppError::ValidationError(format!("{:?}", e)))?;

    // Step 3: Prepare metadata using core module (pure logic)
    let new_file = import_operations::prepare_file_metadata(path, file_size);

    // Step 4: Insert into database (I/O)
    let file_id = FileRepository::insert(pool, new_file).await
        .map_err(|e| AppError::DatabaseError(e))?;

    Ok(file_id)
}
```

#### Step 2.3: Update commands/split_file.rs

```rust
/// Split file command (Grown-up Script)
/// No longer depends on file_import command

use crate::core::import_operations;
use crate::core::splitting::split_tracks;

pub async fn split_and_import(
    pool: &PgPool,
    parent_path: &std::path::Path,
) -> Result<SplitResult, AppError> {
    // Step 1: Read parent file (I/O)
    let midi_data = std::fs::read(parent_path)
        .map_err(|e| AppError::IOError(e))?;

    // Step 2: Parse MIDI (core logic - pure)
    let smf = midly::Smf::parse(&midi_data)
        .map_err(|e| AppError::MidiError(format!("{}", e)))?;

    // Step 3: Split tracks (core logic - pure)
    let tracks = split_tracks(&smf)?;

    // Step 4: For each track, use SHARED core module (not file_import)
    for (index, track_data) in tracks.iter().enumerate() {
        // Prepare metadata using core module
        let new_file = import_operations::prepare_file_metadata(
            &parent_path,
            track_data.len() as u64,
        );

        // Insert to database
        FileRepository::insert(pool, new_file).await
            .map_err(|e| AppError::DatabaseError(e))?;
    }

    Ok(SplitResult { track_count: tracks.len() })
}
```

#### Step 2.4: Update module structure

```rust
// pipeline/src-tauri/src/core/mod.rs

pub mod analysis;
pub mod hash;
pub mod naming;
pub mod normalization;
pub mod performance;
pub mod pipeline;
pub mod splitting;
pub mod import_operations;  // ← NEW: shared import logic

// Re-export commonly used types
pub use import_operations::{validate_file_for_import, prepare_file_metadata};
```

---

### Fix 3: Refactor Large Modules (P1)

**Target:** Break modules with 500+ LOC into focused components

#### 3.1: Split orchestrator.rs (17,649 bytes)

**Current:** One file handles all pipeline coordination

**Target:** 4 focused files
```
orchestrator/
├── mod.rs              (500 LOC - main coordinator)
├── stage_manager.rs    (400 LOC - stage lifecycle)
├── queue_manager.rs    (350 LOC - queue operations)
└── error_handling.rs   (250 LOC - error recovery)
```

**Implementation:**

Create `pipeline/src-tauri/src/core/pipeline/stage_manager.rs`:
```rust
/// Stage lifecycle management (Trusty Module)
pub struct StageManager {
    stages: Vec<PipelineStage>,
}

impl StageManager {
    pub fn new() -> Self { /* ... */ }
    pub async fn initialize_stages(&mut self) -> Result<(), StageError> { /* ... */ }
    pub async fn shutdown_stages(&mut self) -> Result<(), StageError> { /* ... */ }
}
```

#### 3.2: Split file_import.rs (Complex I/O)

**Current:** Single file with import, archive, batch handling

**Target:** Clear separation
```
file_import/
├── mod.rs              (single file import)
├── archive.rs          (archive extraction - move to io/)
└── batch.rs            (batch operation orchestration)
```

#### 3.3: Split analyze.rs (6+ algorithms)

**Current:** One file with BPM, key, chord, drum analysis

**Target:** Coordinating module
```
analyze/
├── mod.rs              (orchestration)
├── bpm.rs              (move existing detector)
├── key.rs              (move existing detector)
├── chords.rs           (move chord analysis)
└── drums.rs            (move drum analyzer)
```

---

### Fix 4: Implement Proper Dependency Injection (P2)

**Current Problem:** AppState as service locator pattern

```rust
// CURRENT: Service locator anti-pattern
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

// Used everywhere as singleton
#[tauri::command]
async fn get_file(state: State<AppState>) {
    // state.database used directly
}
```

**Target:** Constructor injection

```rust
// BETTER: Dependency injection trait
pub trait FileService {
    fn get_file(&self, id: i64) -> Result<File>;
}

pub struct FileServiceImpl {
    repository: Arc<FileRepository>,
}

#[tauri::command]
async fn get_file(
    state: State<Arc<dyn FileService>>
) -> Result<File> {
    state.get_file(id)
}
```

**Implementation (Gradual):**

Step 1: Create service traits
```rust
// pipeline/src-tauri/src/services/mod.rs
pub mod file_service;
pub mod import_service;
pub mod analysis_service;

pub trait FileService: Send + Sync {
    fn get_file(&self, id: i64) -> Result<File>;
    fn list_files(&self, limit: usize) -> Result<Vec<File>>;
}
```

Step 2: Implement service wrapper
```rust
pub struct FileServiceImpl {
    repository: Arc<FileRepository>,
}

impl FileService for FileServiceImpl {
    fn get_file(&self, id: i64) -> Result<File> {
        // Delegates to repository
        self.repository.find_by_id(id)
    }
}
```

Step 3: Update main.rs
```rust
let file_service = Arc::new(FileServiceImpl::new(database.clone()));
builder.manage(file_service)
```

---

### Fix 5: Add Database Telemetry (P2)

**Current:** No visibility into database operations

**Target:** Metrics for all database operations

#### 5.1: Create metrics module

Create `pipeline/src-tauri/src/observability/metrics.rs`:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct DatabaseMetrics {
    queries_total: AtomicU64,
    queries_failed: AtomicU64,
    avg_query_time_ms: parking_lot::Mutex<Vec<u64>>,
}

impl DatabaseMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            queries_total: AtomicU64::new(0),
            queries_failed: AtomicU64::new(0),
            avg_query_time_ms: parking_lot::Mutex::new(Vec::new()),
        })
    }

    pub fn record_query(&self, duration_ms: u64, success: bool) {
        self.queries_total.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.queries_failed.fetch_add(1, Ordering::Relaxed);
        }
        self.avg_query_time_ms.lock().push(duration_ms);
    }

    pub fn get_stats(&self) -> DatabaseStats {
        let total = self.queries_total.load(Ordering::Relaxed);
        let failed = self.queries_failed.load(Ordering::Relaxed);
        let times = self.avg_query_time_ms.lock();
        let avg = times.iter().sum::<u64>() / times.len().max(1) as u64;

        DatabaseStats {
            total_queries: total,
            failed_queries: failed,
            success_rate: ((total - failed) as f64 / total as f64 * 100.0),
            avg_query_time_ms: avg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_queries: u64,
    pub failed_queries: u64,
    pub success_rate: f64,
    pub avg_query_time_ms: u64,
}
```

#### 5.2: Instrument repository operations

```rust
// db/repositories/file_repository.rs

impl FileRepository {
    pub async fn insert_with_metrics(
        pool: &PgPool,
        new_file: NewFile,
        metrics: Arc<DatabaseMetrics>,
    ) -> Result<i64, sqlx::Error> {
        let start = std::time::Instant::now();

        let result = sqlx::query_scalar!(
            r#"INSERT INTO files (...) VALUES (...) RETURNING id"#,
            // ... params
        )
        .fetch_one(pool)
        .await;

        let duration = start.elapsed().as_millis() as u64;
        metrics.record_query(duration, result.is_ok());

        result
    }
}
```

#### 5.3: Expose telemetry endpoint

```rust
// commands/system.rs

#[tauri::command]
async fn get_database_metrics(
    state: State<Arc<DatabaseMetrics>>
) -> DatabaseStats {
    state.get_stats()
}

// Frontend can call: invoke('get_database_metrics')
```

---

### Fix 6: Schema Migration Testing (P2)

**Current:** Migrations created but not tested at scale

**Target:** Automated migration testing with production-scale data

#### 6.1: Create migration test framework

```rust
// tests/migrations_test.rs

#[tokio::test]
async fn test_001_initial_schema_with_large_data() {
    // Setup: Create test database
    let db = TestDatabase::new().await;

    // Create schema
    db.execute_migration("001_initial_schema.sql").await.unwrap();

    // Load test data (1M records)
    let test_data = generate_test_files(1_000_000);
    db.insert_batch(&test_data, 1000).await.unwrap();

    // Verify integrity
    let count = db.count_files().await.unwrap();
    assert_eq!(count, 1_000_000);

    // Verify indexes are working
    let start = std::time::Instant::now();
    db.query("SELECT * FROM files WHERE content_hash = $1", &[])
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100, "Index query too slow: {:?}", elapsed);
}
```

#### 6.2: Use Docker Compose for testing

```yaml
# tests/docker-compose.test.yml
version: '3'
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: midi_test
      POSTGRES_PASSWORD: test
    volumes:
      - /tmp/pg_data:/var/lib/postgresql/data
    ports:
      - "5433:5432"
```

---

## IMPLEMENTATION TIMELINE

### Week 1: Preparation & Planning
- [ ] Audit all 351 unwrap() calls
- [ ] Create categorization spreadsheet
- [ ] Map all command-to-command dependencies
- [ ] Plan module refactoring

### Week 2-3: Critical Fixes (P0)
- [ ] Replace critical unwrap() calls
- [ ] Extract import_operations core module
- [ ] Update file_import, split_file, archive_import
- [ ] Test critical path with real files

### Week 4: High Priority (P1)
- [ ] Refactor orchestrator.rs
- [ ] Refactor file_import.rs
- [ ] Refactor analyze.rs
- [ ] Run test suite

### Week 5: Medium Priority (P2)
- [ ] Implement error recovery
- [ ] Add database metrics/telemetry
- [ ] Create migration tests
- [ ] Documentation

### Week 6: Integration & QA
- [ ] Full integration testing
- [ ] Performance regression testing
- [ ] Production readiness review
- [ ] Deployment planning

### Week 7-8: Optional (P3)
- [ ] Implement dependency injection refactor
- [ ] Add more telemetry
- [ ] Performance optimization
- [ ] Community documentation

---

## TESTING STRATEGY

### Before & After Each Fix

```bash
# Step 1: Run full test suite
cargo test --workspace --lib -- --test-threads=1

# Step 2: Run clippy for code quality
cargo clippy --workspace -- -D warnings

# Step 3: Check compilation
cargo check --workspace

# Step 4: Integration tests with real files
cargo test --test file_import_test -- --test-threads=1

# Step 5: Performance benchmark
cargo bench --no-run
```

### Regression Testing

```bash
# Test critical paths before/after changes
cargo test import_single_file --lib
cargo test split_and_import --lib
cargo test batch_insert --lib
cargo test analyze_file --lib
```

---

## SUCCESS CRITERIA

### P0 Completion
- [ ] All high-risk unwrap() replaced with proper error handling
- [ ] Zero command-to-command circular dependencies
- [ ] import_operations core module created and tested
- [ ] Error recovery in place for import failures
- [ ] All tests passing

### P1 Completion
- [ ] Large modules refactored (orchestrator, file_import, analyze)
- [ ] Database telemetry added
- [ ] Migration tests passing at scale
- [ ] No compilation warnings

### P2 Completion
- [ ] Dependency injection implemented for key services
- [ ] Error recovery working in production scenario
- [ ] Full test coverage >80%
- [ ] Production deployment approved

---

## ROLLBACK PROCEDURES

If any fix causes regressions:

```bash
# Immediate rollback
git revert <commit-hash>

# Or restore from branch
git checkout main
git pull
cargo build --release

# Data is safe - no schema changes in initial fixes
```

---

## METRICS TO TRACK

Track these metrics before/after fixes:

| Metric | Before | After | Goal |
|--------|--------|-------|------|
| unwrap() calls | 351 | <50 | <50 |
| Compilation warnings | N | 0 | 0 |
| Test coverage | 54.53% | >65% | >80% |
| Large modules (>500 LOC) | 5+ | <3 | <2 |
| Command dependencies | Circular | Linear | Linear |
| Import error recovery | None | Partial | Full |
| Database query latency | Unknown | Tracked | <100ms avg |

---

## NEXT STEPS

1. **Create git branch for fixes**
   ```bash
   git checkout -b architecture-fixes/phase-1
   ```

2. **Start with Fix 1 (unwrap audit)**
   - Estimated effort: 16-20 hours
   - Risk: Low (error handling improvements)

3. **Progress to Fix 2 (decouple commands)**
   - Estimated effort: 12-16 hours
   - Risk: Low-Medium (refactoring with tests)

4. **Continue with P1 fixes**
   - Continue only after P0 is stable

5. **Plan weekly sync with team**
   - Review progress against timeline
   - Adjust priorities if needed
   - Document learnings

---

**Remember:** These are architectural improvements to support long-term growth. Take time to do them right, and test thoroughly after each phase.

Good luck! 🚀
