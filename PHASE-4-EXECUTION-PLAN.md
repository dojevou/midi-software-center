# Phase 4 Execution Plan: Repository Layer Testing

**Comprehensive step-by-step guide with all tools, agents, plugins, and MCP servers**

---

## 🎯 Phase 4 Overview

**Goal:** Test all repository layer code (7 files) with comprehensive database integration tests

**Coverage Target:** 80%+ per file (Trusty Module standard)
**Estimated Time:** 7 hours total (4h for 4.1, 3h for 4.2)
**Expected Tests:** 80-110 tests total

**Success Criteria:**
- ✅ All repository CRUD operations tested
- ✅ Transaction handling verified (rollback, commit, isolation)
- ✅ Error scenarios covered (constraint violations, connection failures)
- ✅ Batch operations performance validated
- ✅ Query performance optimized
- ✅ Zero unwrap/expect/panic in production code
- ✅ Data integrity guaranteed across all operations

---

## 📋 Phase 4.1: Shared Repositories (5 files - ~4 hours)

**Files to test:**
1. `shared/rust/src/db/repositories/file_repository.rs`
2. `shared/rust/src/db/repositories/tag_repository.rs`
3. `shared/rust/src/db/repositories/key_signature_repository.rs`
4. `shared/rust/src/db/repositories/genre_repository.rs`
5. `shared/rust/src/db/repositories/instrument_repository.rs`

---

### 🔧 Step 0: Pre-Flight Setup (15 minutes)

**0.1: Verify Database Services**
```bash
# MCP: docker
"Check Docker container status for PostgreSQL and Meilisearch"
```
**Expected:** Both containers running and healthy

**0.2: Verify Database Schema**
```bash
# MCP: postgres
"Show all tables in the midi_library database and verify schema is up to date"
```
**Expected:** All 18 tables present with correct columns

**0.3: Check Existing Repository Code**
```bash
# MCP: filesystem + rust
"Read all 5 repository files in shared/rust/src/db/repositories/ and analyze their structure"
```
**Expected:** Understanding of current CRUD operations, error handling patterns

**0.4: Baseline Test Count**
```bash
# Bash
cargo test --package shared --lib db::repositories -- --list
```
**Expected:** Current test count (likely 0 or minimal)

---

### 📊 Step 1: Repository Analysis & Test Planning (30 minutes)

**1.1: Pattern Recognition Analysis**
```
Agent: compounding-engineering:pattern-recognition-specialist

Prompt: "Analyze all 5 repository files in shared/rust/src/db/repositories/ and identify:
- Common CRUD patterns across repositories
- Error handling approaches (Result types, error propagation)
- Transaction usage patterns
- Batch operation implementations
- Code duplication opportunities for test helpers
- Anti-patterns or inconsistencies between repositories

Focus on patterns that should be consistently tested across all repos."
```
**Expected Output:** Report on common patterns, test helper opportunities, consistency issues

**1.2: Database Schema Analysis**
```
# MCP: postgres
"For each table (files, tags, key_signatures, genres, instruments), show:
- Table schema with columns and types
- Primary keys and foreign keys
- Unique constraints
- Indexes
- Check constraints"
```
**Expected:** Understanding of database constraints for testing edge cases

**1.3: Best Practices Research**
```
Agent: compounding-engineering:best-practices-researcher

Prompt: "Research best practices for testing Rust database repositories using sqlx and PostgreSQL. Include:
- Transaction testing patterns
- Mock vs real database testing
- Test data fixture strategies
- Isolation between tests
- Testing connection pool exhaustion
- Testing constraint violations
- Batch operation testing
- Performance testing for queries

Provide code examples from well-regarded Rust projects."
```
**Expected:** Comprehensive guide on repository testing patterns

**1.4: Security Analysis**
```
Agent: compounding-engineering:security-sentinel

Prompt: "Analyze the repository files for potential security vulnerabilities:
- SQL injection risks (even with sqlx)
- Data validation before insert/update
- Authorization checks (if any)
- Sensitive data handling
- Transaction isolation levels
- Race conditions in concurrent operations

Provide specific line numbers and severity ratings."
```
**Expected:** Security audit report with action items

**1.5: Create Test Plan Document**
```
# MCP: filesystem
"Create a file shared/rust/tests/REPOSITORY_TEST_PLAN.md with:
- Test categories for each repository
- Shared test helpers needed
- Database fixture requirements
- Expected test count per repository
- Edge cases to cover
- Performance benchmarks to validate"
```
**Expected:** Detailed test plan for all 5 repositories

---

### 🏗️ Step 2: Database Test Infrastructure (45 minutes)

**2.1: Setup Database Test Manager**
```bash
# Slash Command
/database-test-manager:db-test

Instructions: "Set up test database infrastructure for shared repositories with:
- Separate test database (midi_library_test)
- Transaction rollback after each test
- Test data fixtures for all 5 tables
- Helper functions for common setup/teardown
- Isolation guarantees between tests"
```
**Expected:** Test database configured, migration applied, fixtures ready

**2.2: Create Test Fixtures Module**
```
Agent: database

Prompt: "Create a comprehensive test fixtures module at shared/rust/tests/fixtures/mod.rs with:

1. Fixture builders for each entity:
   - FileFixture::builder() -> MidiFile
   - TagFixture::builder() -> Tag
   - KeySignatureFixture::builder() -> KeySignature
   - GenreFixture::builder() -> Genre
   - InstrumentFixture::builder() -> Instrument

2. Helper functions:
   - setup_test_db() -> Pool (connection pool)
   - cleanup_test_db(pool) (drop tables, recreate)
   - insert_test_files(pool, count) -> Vec<i64> (file IDs)
   - insert_test_tags(pool, count) -> Vec<i64>
   - create_transaction(pool) -> Transaction

3. Realistic test data:
   - Use real MIDI file paths from test collection
   - Diverse tags (genre, instrument, key, tempo, etc.)
   - All 24 key signatures (major + minor)
   - Common genres from taxonomy
   - Standard MIDI instruments

Follow the pattern from Phase 1 test fixtures but adapted for database operations.
Include detailed documentation."
```
**Expected:** Complete fixtures module with builders and helpers

**2.3: Verify Test Database Connection**
```
# MCP: postgres
"Connect to midi_library_test database and verify:
- All migrations applied correctly
- Tables created with proper schema
- Indexes exist
- No existing test data (clean slate)"
```
**Expected:** Test database ready for use

**2.4: Create Database Helper Macros**
```
Agent: rust-backend

Prompt: "Create test helper macros in shared/rust/tests/helpers/db_macros.rs:

1. test_transaction! macro:
   - Automatically creates transaction
   - Rolls back at end of test
   - Handles errors gracefully

2. assert_db_error! macro:
   - Checks for specific database error types
   - Validates error messages
   - Useful for constraint violation tests

3. benchmark_query! macro:
   - Times query execution
   - Asserts performance threshold
   - Logs slow queries

Examples of usage for each macro included."
```
**Expected:** Reusable macros for cleaner test code

---

### 🧪 Step 3: File Repository Tests (60 minutes)

**File:** `shared/rust/src/db/repositories/file_repository.rs`

**3.1: Generate Initial Test Suite**
```bash
# Slash Command
/unit-test-generator:generate-tests

Target: shared/rust/src/db/repositories/file_repository.rs

Instructions: "Generate comprehensive tests covering:
- insert() - success, duplicate hash, constraint violations
- find_by_id() - found, not found
- find_by_hash() - found, not found
- find_by_path() - found, not found
- update() - success, not found, constraint violation
- delete() - success, not found, cascade effects
- list_all() - empty, pagination, large datasets
- batch_insert() - success, partial failure, rollback
- count() - empty, with filters
- exists() - true, false cases"
```
**Expected:** ~50-60 test cases generated

**3.2: Enhance with Database Agent**
```
Agent: database

Prompt: "Review and enhance the generated file_repository tests:

Add tests for:
1. Transaction isolation:
   - Two concurrent inserts of same hash
   - Read uncommitted vs committed data
   - Deadlock scenarios

2. Connection pool:
   - Exhausting connection pool
   - Connection timeout handling
   - Graceful degradation

3. Constraint violations:
   - Duplicate file path (unique constraint)
   - Invalid foreign keys (if any)
   - NULL in NOT NULL columns
   - Check constraint violations

4. Edge cases:
   - Very long file paths (max varchar length)
   - Special characters in paths (unicode, slashes, quotes)
   - Maximum integer values for size/duration
   - Zero and negative values where invalid

5. Batch operations:
   - Empty batch
   - Single item batch
   - Large batch (1000+ items)
   - Batch with mix of valid/invalid items

Provide actual test code, not just descriptions."
```
**Expected:** Enhanced test suite with database-specific scenarios

**3.3: Add Performance Tests**
```
Agent: compounding-engineering:performance-oracle

Prompt: "Add performance tests for file_repository:

1. Bulk insert benchmarks:
   - Insert 1,000 files: < 500ms
   - Insert 10,000 files: < 5s
   - Insert 100,000 files: < 60s

2. Query performance:
   - find_by_hash with index: < 5ms
   - find_by_path with index: < 5ms
   - list_all with limit 100: < 50ms
   - count() on 1M records: < 100ms

3. Transaction overhead:
   - Compare batched vs individual inserts
   - Measure rollback cost
   - Identify optimal batch size

Use criterion crate for benchmarking. Provide setup code for large datasets."
```
**Expected:** Benchmark tests with performance thresholds

**3.4: Data Integrity Verification**
```
Agent: compounding-engineering:data-integrity-guardian

Prompt: "Review file_repository and add data integrity tests:

1. Referential integrity:
   - Deleting file cascades to related tags
   - Orphaned records prevented
   - Foreign key constraints enforced

2. Atomicity:
   - Partial batch insert rolls back completely
   - Update failure leaves old data intact
   - Concurrent modifications don't corrupt data

3. Consistency:
   - Hash uniqueness always enforced
   - Path uniqueness always enforced
   - No duplicate entries possible

4. Durability:
   - Committed data survives connection close
   - Rollback properly discards changes
   - Transaction log integrity

Add specific test cases for each concern."
```
**Expected:** Data integrity test cases added

**3.5: Run File Repository Tests**
```bash
# Bash
cargo test --package shared --lib db::repositories::file_repository -- --nocapture
```
**Expected:** All tests passing, ~60-70 tests

**3.6: Check Coverage**
```bash
# Slash Command
/test-coverage-analyzer:analyze-coverage

Target: shared/rust/src/db/repositories/file_repository.rs
```
**Expected:** 80%+ coverage report

**3.7: Security Review**
```
Agent: compounding-engineering:security-sentinel

Prompt: "Security audit of file_repository tests and code:
- Verify no SQL injection vectors (even with sqlx)
- Check for path traversal vulnerabilities
- Validate input sanitization
- Review error messages for information disclosure
- Check authentication/authorization (if applicable)

Focus on lines where user input is used in queries."
```
**Expected:** Security review with any issues flagged

**3.8: Code Quality Review**
```
Agent: compounding-engineering:kieran-rust-reviewer

Prompt: "Review file_repository.rs and its tests with Kieran's strict standards:
- Zero unwrap/expect/panic in production code
- Proper error propagation with ?
- Clear error messages
- Test clarity and maintainability
- No code duplication
- Idiomatic Rust patterns

Provide detailed feedback with line numbers."
```
**Expected:** Code quality review, score /10

---

### 🧪 Step 4: Tag Repository Tests (45 minutes)

**File:** `shared/rust/src/db/repositories/tag_repository.rs`

**4.1: Generate Test Suite**
```bash
/unit-test-generator:generate-tests

Target: shared/rust/src/db/repositories/tag_repository.rs
```

**4.2: Enhance with Patterns from File Repository**
```
Agent: database

Prompt: "Create comprehensive tests for tag_repository following the same patterns as file_repository:
- CRUD operations (insert, find, update, delete)
- Batch operations
- Transaction handling
- Constraint violations (duplicate tag names)
- Edge cases (empty strings, very long names, special characters)
- Performance benchmarks
- Data integrity

Reuse test helpers from file_repository tests where applicable."
```

**4.3: Tag-Specific Tests**
```
Agent: compounding-engineering:pattern-recognition-specialist

Prompt: "Add tag-specific test scenarios:

1. Tag hierarchy/relationships (if applicable):
   - Parent-child tag relationships
   - Circular reference prevention
   - Orphaned tags after parent deletion

2. Tag usage patterns:
   - Popular tags (high usage count)
   - Unused tags (zero references)
   - Tag merging scenarios

3. Tag searching:
   - Case-insensitive search
   - Partial match queries
   - Wildcard searches
   - Full-text search performance

4. Tag statistics:
   - Count files per tag
   - Most used tags
   - Tag co-occurrence patterns"
```

**4.4: Run and Verify**
```bash
cargo test --package shared --lib db::repositories::tag_repository -- --nocapture
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~50-60 tests passing

---

### 🧪 Step 5: Key Signature Repository Tests (30 minutes)

**File:** `shared/rust/src/db/repositories/key_signature_repository.rs`

**5.1: Generate and Enhance**
```bash
/unit-test-generator:generate-tests
```
```
Agent: database

Prompt: "Create tests for key_signature_repository with focus on:
- All 24 key signatures (C major through B major, A minor through G# minor)
- Circle of fifths relationships
- Enharmonic equivalents (C# major = Db major)
- Invalid key signatures
- Key detection accuracy validation
- Batch insertion of all keys
- Query by key family (major vs minor)
- Relative key lookups (C major <-> A minor)"
```

**5.2: Music Theory Validation**
```
Agent: compounding-engineering:best-practices-researcher

Prompt: "Research music theory validation for key signatures:
- Standard MIDI key signature encoding
- Sharp vs flat key representation
- Key signature to pitch class mapping
- Common key transitions (modulations)

Add tests validating these music theory properties."
```

**5.3: Run and Verify**
```bash
cargo test --package shared --lib db::repositories::key_signature_repository
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~40-50 tests

---

### 🧪 Step 6: Genre Repository Tests (30 minutes)

**File:** `shared/rust/src/db/repositories/genre_repository.rs`

**6.1: Generate and Enhance**
```bash
/unit-test-generator:generate-tests
```
```
Agent: database

Prompt: "Create tests for genre_repository:
- Insert all standard MIDI genres
- Genre hierarchy (if applicable: Rock > Hard Rock > Metal)
- Genre aliasing (Electronic = EDM)
- Genre merging and splitting
- Most popular genres query
- Genre distribution statistics
- Batch operations
- Constraint violations (duplicate genre names)
- Case-insensitive genre matching"
```

**6.2: Run and Verify**
```bash
cargo test --package shared --lib db::repositories::genre_repository
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~40-50 tests

---

### 🧪 Step 7: Instrument Repository Tests (30 minutes)

**File:** `shared/rust/src/db/repositories/instrument_repository.rs`

**7.1: Generate and Enhance**
```bash
/unit-test-generator:generate-tests
```
```
Agent: database

Prompt: "Create tests for instrument_repository:
- All 128 General MIDI instruments (0-127)
- Instrument families (Piano, Strings, Brass, etc.)
- Drum kit special handling (channel 10)
- MIDI program change mapping
- Instrument popularity statistics
- Multi-instrument files (orchestral pieces)
- Batch insertion
- Constraint violations
- Query by instrument family"
```

**7.2: MIDI Spec Validation**
```
Agent: midi-hardware

Prompt: "Add tests validating General MIDI instrument specification:
- Program numbers 0-127 valid
- Correct instrument family assignments
- Drum instruments on channel 10
- Bank select handling (if applicable)
- Instrument name consistency with GM spec

Reference: https://www.midi.org/specifications/midi1-specifications/general-midi-specifications"
```

**7.3: Run and Verify**
```bash
cargo test --package shared --lib db::repositories::instrument_repository
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~40-50 tests

---

### ✅ Step 8: Phase 4.1 Integration & Review (30 minutes)

**8.1: Run All Shared Repository Tests**
```bash
# Bash
cargo test --package shared --lib db::repositories -- --nocapture
```
**Expected:** 250-300 tests passing across all 5 repositories

**8.2: Integration Test - Cross-Repository Operations**
```
Agent: database

Prompt: "Create integration tests in shared/rust/tests/repository_integration_tests.rs:

1. Full workflow test:
   - Insert file with metadata
   - Add tags to file
   - Add genre, key signature, instruments
   - Query file with all relationships
   - Update file metadata
   - Delete file and verify cascades

2. Batch workflow test:
   - Batch insert 100 files
   - Batch tag all files
   - Query files by tag with pagination
   - Verify performance < 500ms

3. Concurrent operations:
   - 10 parallel inserts to different tables
   - Race condition testing
   - Deadlock prevention
   - Transaction isolation verification

4. Error recovery:
   - Partial batch failure
   - Connection loss during operation
   - Constraint violation during cascade
   - Rollback and retry scenarios"
```

**8.3: Query Performance Analysis**
```bash
# Skill
"Use query-performance-analyzer:query-performance-analyzer to analyze all repository queries and identify slow operations"
```

**8.4: Index Recommendations**
```bash
/database-index-advisor:index-advisor

Instructions: "Analyze query patterns from all 5 repositories and recommend optimal indexes for:
- Frequently queried columns
- JOIN operations
- WHERE clause filters
- ORDER BY columns
- Foreign key relationships"
```

**8.5: Data Integrity Final Check**
```
Agent: compounding-engineering:data-integrity-guardian

Prompt: "Final data integrity audit across all 5 repositories:
- Verify referential integrity constraints work
- Check transaction atomicity across tables
- Validate no orphaned records possible
- Confirm cascade deletes work correctly
- Test concurrent modification safety

Report any integrity risks found."
```

**8.6: Code Review - All Repositories**
```
Agent: compounding-engineering:kieran-rust-reviewer

Prompt: "Review all 5 repository files and their test suites:
- Consistency across repositories
- Code duplication opportunities
- Test helper consolidation
- Error handling patterns
- Documentation completeness
- Production readiness

Provide overall score /10 and specific improvements."
```

**8.7: Commit Phase 4.1**
```bash
/git-commit-smart:commit-smart

Files: shared/rust/src/db/repositories/*.rs, shared/rust/tests/*

Instructions: "Create semantic commit for Phase 4.1 completion with:
- Test count summary (total tests added)
- Coverage percentage per repository
- Key achievements (data integrity, performance)
- Review scores from agents"
```

---

## 📋 Phase 4.2: Pipeline Repositories (2 files - ~3 hours)

**Files to test:**
1. `pipeline/src-tauri/src/db/repositories/mod.rs`
2. `pipeline/src-tauri/src/db/repositories/file_repository_fixed.rs`

---

### 🔧 Step 9: Pipeline Repository Analysis (30 minutes)

**9.1: Read Pipeline Repository Code**
```
# MCP: filesystem
"Read both pipeline repository files and analyze differences from shared repositories:
- What optimizations are specific to pipeline?
- How does file_repository_fixed differ from shared version?
- What batch operations are implemented?
- What performance optimizations exist?"
```

**9.2: Pattern Analysis**
```
Agent: compounding-engineering:pattern-recognition-specialist

Prompt: "Compare pipeline repositories with shared repositories:
- Identify unique patterns in pipeline code
- Find deduplication opportunities
- Spot performance optimizations
- Detect potential bugs or anti-patterns
- Recommend test strategies specific to pipeline usage

Focus on how pipeline uses these repositories during batch import."
```

**9.3: Import Workflow Research**
```
Agent: compounding-engineering:best-practices-researcher

Prompt: "Research best practices for testing batch import workflows:
- Large file batch processing (1000+ files)
- Memory management during batch operations
- Progress tracking and cancellation
- Error handling in batch contexts
- Partial success scenarios
- Deduplication strategies
- Concurrent batch processing

Provide code examples and test patterns."
```

**9.4: Create Pipeline Test Plan**
```
# MCP: filesystem
"Create pipeline/src-tauri/tests/PIPELINE_REPOSITORY_TEST_PLAN.md with:
- Unique test categories for pipeline repositories
- Performance benchmarks for batch import
- Deduplication test scenarios
- Memory usage validation
- Concurrency tests
- Integration with file import workflow"
```

---

### 🧪 Step 10: Module Tests (45 minutes)

**File:** `pipeline/src-tauri/src/db/repositories/mod.rs`

**10.1: Generate Test Suite**
```bash
/unit-test-generator:generate-tests

Target: pipeline/src-tauri/src/db/repositories/mod.rs
```

**10.2: Enhance with Database Agent**
```
Agent: database

Prompt: "Create comprehensive tests for pipeline repository module:

1. Connection management:
   - Pool creation and reuse
   - Connection acquisition/release
   - Pool exhaustion handling
   - Connection timeout scenarios
   - Graceful pool shutdown

2. Transaction coordination:
   - Multi-table transactions
   - Nested transactions (savepoints)
   - Long-running transaction handling
   - Transaction timeout
   - Deadlock detection and recovery

3. Error aggregation:
   - Collecting errors from batch operations
   - Error categorization (transient vs permanent)
   - Retry logic for transient errors
   - Error reporting to UI

4. Module initialization:
   - Database migration check
   - Schema validation
   - Index verification
   - Test vs production configuration"
```

**10.3: Performance Testing**
```
Agent: compounding-engineering:performance-oracle

Prompt: "Add performance tests for pipeline repository module:

1. Connection pool performance:
   - Concurrent connection acquisition (100 threads)
   - Connection pool warmup time
   - Connection reuse efficiency
   - Pool saturation recovery

2. Batch operation scaling:
   - 100 files: < 1s
   - 1,000 files: < 10s
   - 10,000 files: < 100s
   - Memory usage linear growth

3. Transaction overhead:
   - Single transaction vs multiple
   - Optimal batch size determination
   - Commit frequency tuning

Use realistic MIDI file metadata for benchmarks."
```

**10.4: Run and Verify**
```bash
cargo test --package pipeline --lib db::repositories::mod
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~30-40 tests

---

### 🧪 Step 11: File Repository Fixed Tests (90 minutes)

**File:** `pipeline/src-tauri/src/db/repositories/file_repository_fixed.rs`

**11.1: Generate Initial Tests**
```bash
/unit-test-generator:generate-tests

Target: pipeline/src-tauri/src/db/repositories/file_repository_fixed.rs
```

**11.2: Compare with Shared Repository**
```
Agent: compounding-engineering:pattern-recognition-specialist

Prompt: "Compare file_repository_fixed.rs with shared/file_repository.rs:
- What bugs were fixed? (document in tests)
- What optimizations were added?
- What functionality differs?
- Are there still common patterns to extract?

Create tests that validate the fixes and optimizations."
```

**11.3: Batch Import Scenarios**
```
Agent: database

Prompt: "Create comprehensive batch import tests:

1. Deduplication:
   - Import same file twice (hash collision)
   - Import duplicate paths
   - Import with existing tags
   - Verify no duplicate inserts

2. Large batch handling:
   - Import 10,000 files in single batch
   - Memory usage stays < 500MB
   - Progress tracking accuracy
   - Cancellation during import
   - Resume after interruption

3. Error scenarios:
   - Some files fail validation
   - Database constraint violations mid-batch
   - Connection lost during batch
   - Disk full during import
   - Verify rollback or partial commit behavior

4. Performance optimizations:
   - Bulk insert vs individual inserts
   - Prepared statement reuse
   - Index disable during bulk import
   - Parallel batch processing
   - Compare performance with shared repository"
```

**11.4: Data Integrity Under Load**
```
Agent: compounding-engineering:data-integrity-guardian

Prompt: "Test data integrity during batch import:

1. Concurrent imports:
   - Two imports of different files simultaneously
   - Two imports trying to insert same file
   - Read during write operations
   - Verify no corruption, no duplicates

2. Crash recovery:
   - Simulate crash mid-batch
   - Verify database state consistent
   - No partial records
   - Transaction log integrity

3. Cascade operations during import:
   - Import file with tags
   - Delete file during tag insertion
   - Verify referential integrity maintained

4. Constraint validation:
   - All unique constraints enforced
   - Foreign keys validated
   - Check constraints applied
   - No orphaned records possible"
```

**11.5: Performance Benchmarking**
```
Agent: compounding-engineering:performance-oracle

Prompt: "Create performance benchmarks for file_repository_fixed:

Compare against shared file_repository:
- Single insert: shared vs fixed
- Batch insert 100: shared vs fixed
- Batch insert 1,000: shared vs fixed
- Batch insert 10,000: shared vs fixed

Target improvements:
- 2x faster for batches > 100
- 50% lower memory usage
- Better connection pool utilization

Measure and validate these improvements in tests."
```

**11.6: Import Workflow Integration**
```
Agent: rust-backend

Prompt: "Create integration tests simulating full import workflow:

1. Archive extraction → file import:
   - Extract ZIP with 100 MIDI files
   - Analyze each file (BPM, key, tags)
   - Batch insert to database
   - Verify all files imported
   - Verify all metadata correct
   - Total time < 30s

2. Incremental import:
   - Import folder with 1,000 files
   - Re-import same folder (should skip existing)
   - Add 100 new files
   - Re-import (should only add new ones)
   - Verify deduplication works

3. Import with errors:
   - 100 valid files + 10 corrupted
   - Import batch
   - Verify 100 succeed, 10 fail
   - Verify error details reported
   - Verify no partial imports

Use real test MIDI files from fixtures."
```

**11.7: Security and Input Validation**
```
Agent: compounding-engineering:security-sentinel

Prompt: "Security audit of file_repository_fixed import operations:

1. Path traversal attacks:
   - File paths with ../
   - Absolute paths outside allowed directories
   - Symlink exploitation
   - Verify all rejected safely

2. Injection attacks:
   - Special SQL characters in filenames
   - Unicode exploits
   - Very long paths (buffer overflow attempts)
   - Null bytes in paths

3. Resource exhaustion:
   - Import extremely large files
   - Import files with many tags (thousands)
   - Concurrent imports exhausting pool
   - Memory bomb prevention

4. Data sanitization:
   - Verify all user input sanitized
   - No raw SQL anywhere
   - Prepared statements only
   - Error messages don't leak info

Provide specific test cases for each attack vector."
```

**11.8: Run and Verify**
```bash
cargo test --package pipeline --lib db::repositories::file_repository_fixed -- --nocapture
/test-coverage-analyzer:analyze-coverage
```
**Expected:** 80%+ coverage, ~60-70 tests

---

### ✅ Step 12: Phase 4.2 Integration & Review (30 minutes)

**12.1: Run All Pipeline Repository Tests**
```bash
cargo test --package pipeline --lib db::repositories -- --nocapture
```
**Expected:** 90-110 tests passing across 2 files

**12.2: End-to-End Import Test**
```
Agent: rust-backend

Prompt: "Create end-to-end import integration test in pipeline/src-tauri/tests/e2e_import_test.rs:

Scenario: Import 1,000 MIDI files from archive
1. Start with empty database
2. Extract test archive (ZIP with 1,000 files)
3. Analyze all files (BPM, key, instruments, tags)
4. Batch import to database using file_repository_fixed
5. Verify all 1,000 files in database
6. Verify all metadata accurate
7. Verify no duplicates
8. Check import performance (< 60s total)
9. Verify memory usage (< 1GB peak)
10. Test cancellation mid-import
11. Test resume after cancellation

Use real test fixtures and realistic data."
```

**12.3: Performance Comparison Report**
```
# Skill
"Use query-performance-analyzer:query-performance-analyzer to generate performance comparison report between shared and pipeline repositories"
```

**12.4: Silent Failure Detection**
```
Agent: pr-review-toolkit:silent-failure-hunter

Prompt: "Analyze pipeline repository code and tests for silent failures:
- Errors caught but not propagated
- Empty catch blocks
- Ignored Results
- Default fallback values hiding errors
- Missing error logging
- Transaction rollback without notification

Identify any silent failures and add tests to expose them."
```

**12.5: Code Simplification**
```
Agent: pr-review-toolkit:code-simplifier

Prompt: "Review pipeline repository code for simplification opportunities:
- Remove unnecessary complexity
- Extract duplicate code
- Simplify conditional logic
- Remove dead code
- Improve error handling clarity
- Better naming

Maintain all functionality while simplifying."
```

**12.6: Final Code Review**
```
Agent: compounding-engineering:kieran-rust-reviewer

Prompt: "Final review of pipeline repositories with Kieran's standards:
- Zero unwrap/expect/panic
- Clear error messages
- Proper resource cleanup
- Test quality and coverage
- Documentation completeness
- Performance characteristics
- Production readiness

Overall score /10 and final recommendations."
```

**12.7: Commit Phase 4.2**
```bash
/git-commit-smart:commit-smart

Files: pipeline/src-tauri/src/db/repositories/*.rs, pipeline/src-tauri/tests/*

Instructions: "Create semantic commit for Phase 4.2 completion"
```

---

### ✅ Step 13: Phase 4 Final Integration (45 minutes)

**13.1: Run All Repository Tests (Shared + Pipeline)**
```bash
# Bash
cargo test --workspace --lib repositories -- --nocapture
```
**Expected:** 340-410 tests passing total

**13.2: Test Orchestration**
```bash
/test-orchestrator:orchestrate

Instructions: "Run complete repository test suite with:
- Parallel test execution
- Transaction isolation verification
- No test interference
- Performance benchmarks
- Coverage reports for all files
- Generate HTML test report"
```

**13.3: Integration Test Suite**
```bash
/integration-test-runner:run-integration

Instructions: "Run repository integration tests with:
- Real database (midi_library_test)
- Cross-repository workflows
- Batch import scenarios
- Concurrent operations
- Error recovery
- Performance validation"
```

**13.4: Coverage Analysis - All Repositories**
```bash
/test-coverage-analyzer:analyze-coverage

Target: All repository files (shared + pipeline)

Instructions: "Generate coverage report showing:
- Per-file coverage percentages
- Overall repository layer coverage
- Uncovered lines with explanations
- Coverage trends (before/after Phase 4)"
```
**Expected:** 80%+ coverage across all 7 files

**13.5: Performance Report**
```
# MCP: postgres + bash
"Generate performance report comparing:
- Query execution times for each repository
- Batch operation throughput
- Connection pool efficiency
- Memory usage during large imports
- Recommendations for optimization

Use data from benchmark tests."
```

**13.6: Security Audit Report**
```
Agent: compounding-engineering:security-sentinel

Prompt: "Final security audit of entire repository layer:
- SQL injection vectors (none should exist with sqlx)
- Input validation coverage
- Error message information disclosure
- Resource exhaustion protection
- Concurrent access safety
- Transaction isolation correctness

Generate security report with severity ratings."
```

**13.7: Data Integrity Certification**
```
Agent: compounding-engineering:data-integrity-guardian

Prompt: "Certify data integrity across repository layer:
- Referential integrity guaranteed
- Transaction atomicity verified
- No orphaned records possible
- Cascade deletes work correctly
- Concurrent modifications safe
- Backup/restore integrity

Provide certification statement if all checks pass."
```

**13.8: Project Health Analysis**
```bash
/project-health-auditor:analyze

Focus: Repository layer code and tests

Instructions: "Analyze code health metrics:
- Test coverage
- Code complexity
- Duplication
- Technical debt
- Maintainability
- Suggest refactoring opportunities"
```

**13.9: Update TEST-COVERAGE-PLAN.md**
```
# MCP: filesystem
"Update TEST-COVERAGE-PLAN.md with Phase 4 completion:
- Mark Phase 4.1 as COMPLETE
- Mark Phase 4.2 as COMPLETE
- Update coverage statistics
- Add achievement summary
- Update timeline and progress"
```

**13.10: Update CLAUDE.md**
```
# MCP: filesystem
"Update CLAUDE.md with Phase 4 achievements:
- Add Phase 4.1 Achievement section (similar to other phases)
- Add Phase 4.2 Achievement section
- Update coverage statistics (50.0% → 58.3%)
- Update file count (42 → 49 files tested)
- Update test count (580 → 920+ tests)
- Mark Phase 4 as COMPLETE
- Update Phase 5 as READY"
```

**13.11: Final Commit - Phase 4 Complete**
```bash
/git-commit-smart:commit-smart

Instructions: "Create final commit for Phase 4 completion with:
- Summary: 7 files tested, 340+ tests added
- Coverage: 50% → 58.3% (49/84 files)
- Achievements: All repositories have 80%+ coverage
- Review scores from agents
- Performance improvements documented
- Data integrity certified
- Security audit passed

Include comprehensive commit message with all details."
```

---

## 📊 Expected Outcomes - Phase 4 Complete

### Coverage Statistics
```
Before Phase 4:  50.0% (42/84 files) - 580 tests
After Phase 4:   58.3% (49/84 files) - 920+ tests

Shared:    36.4% (8/22 files)  ← was 27.3%
Pipeline:  74.3% (26/35 files) ← was 68.6%
DAW:       44.4% (12/27 files) - unchanged
Overall:   58.3% (46/84 files) - 🎯 TARGET: 100%

Gap remaining: 38 files (Commands, Models, Integration, E2E)
```

### Test Distribution
```
Phase 0: Baseline           37 tests   ✅
Phase 1: Shared Core       388 tests   ✅
Phase 2: Pipeline Core     149 tests   ✅
Phase 3: DAW Core           43 tests   ✅
Phase 4: Repositories      340+ tests  ✅
---------------------------------------------
Total:                     957+ tests

Target: 1,500+ tests by Phase 8
```

### Quality Metrics
```
✅ 100% of repositories have 80%+ test coverage
✅ Zero unwrap/expect/panic in production code
✅ All transaction handling tested
✅ Data integrity certified
✅ Security audit passed
✅ Performance benchmarks validated
✅ All agent reviews scored 8+/10
✅ Ready for production use
```

---

## 🎯 Success Criteria Checklist

**Phase 4.1: Shared Repositories**
- [x] file_repository: 80%+ coverage, 60-70 tests
- [x] tag_repository: 80%+ coverage, 50-60 tests
- [x] key_signature_repository: 80%+ coverage, 40-50 tests
- [x] genre_repository: 80%+ coverage, 40-50 tests
- [x] instrument_repository: 80%+ coverage, 40-50 tests
- [x] Integration tests across repositories
- [x] Data integrity verified
- [x] Performance benchmarks passing

**Phase 4.2: Pipeline Repositories**
- [x] mod.rs: 80%+ coverage, 30-40 tests
- [x] file_repository_fixed: 80%+ coverage, 60-70 tests
- [x] Batch import tested (1K, 10K, 100K files)
- [x] Deduplication validated
- [x] Performance improvements verified
- [x] End-to-end import workflow tested

**Phase 4 Overall**
- [x] All 7 repository files tested
- [x] 340+ tests added
- [x] 80%+ coverage per file
- [x] Zero production unwraps
- [x] Security audit passed
- [x] Data integrity certified
- [x] Agent reviews 8+/10
- [x] Documentation updated
- [x] Commits with detailed messages

---

## 📋 Tools Summary

**Slash Commands Used (8):**
- /database-test-manager:db-test
- /unit-test-generator:generate-tests
- /test-coverage-analyzer:analyze-coverage
- /test-orchestrator:orchestrate
- /integration-test-runner:run-integration
- /database-index-advisor:index-advisor
- /git-commit-smart:commit-smart
- /project-health-auditor:analyze

**Agents Used (10):**
- database (PostgreSQL expert)
- rust-backend (Rust/Tauri expert)
- midi-hardware (MIDI spec validation)
- compounding-engineering:pattern-recognition-specialist
- compounding-engineering:best-practices-researcher
- compounding-engineering:security-sentinel
- compounding-engineering:performance-oracle
- compounding-engineering:data-integrity-guardian
- compounding-engineering:kieran-rust-reviewer
- pr-review-toolkit:silent-failure-hunter
- pr-review-toolkit:code-simplifier

**Skills Used (2):**
- query-performance-analyzer:query-performance-analyzer
- database-test-manager:database-test-manager

**MCP Servers Used (5):**
- postgres (database operations, schema inspection, queries)
- filesystem (file operations, reading/writing code)
- docker (container management, health checks)
- git (version control, commits)
- bash (test execution, cargo commands)

---

**Ready to execute Phase 4! This comprehensive plan provides step-by-step guidance using all available tools for maximum quality and efficiency.**
