# Phase 5-8 Quality Improvement Plan
## Aligning with Phase 1-4 Standards

### 1. CODE ORGANIZATION

#### Current Phase 1-4 Standard
- Clear section headers (// ==========)
- Tests grouped by functionality (Insert, Find, Update, Delete, etc.)
- Helper functions separated from test code
- Consistent indentation and formatting
- Logical grouping with comments

#### Phase 5-8 Improvements Required

**A. Repository Test Files Structure**
```
// Test file organization:
// 1. Module declarations and imports
// 2. Test helpers section with clear comments
// 3. Test sections with headers:
//    - Setup/Teardown
//    - Basic Operations
//    - Edge Cases
//    - Error Handling
//    - Performance
//    - Concurrent Operations
```

**B. Commands Test Files Structure**
```
// Commands test organization:
// 1. Module imports
// 2. Common test helpers
// 3. Test sections by command:
//    - Initialization
//    - Success scenarios
//    - Error scenarios
//    - State validation
//    - Integration scenarios
```

**C. DAW Model Tests Structure**
```
// DAW test organization:
// 1. Type definitions
// 2. Builder setup
// 3. Serialization tests
// 4. State management tests
// 5. Validation tests
// 6. Performance tests
```

### 2. TEST ISOLATION

#### Phase 1-4 Gold Standard
- Fresh database pool per test
- Two-stage cleanup:
  ```rust
  #[tokio::test]
  async fn test_something() {
      let pool = setup_test_pool().await;
      cleanup_database(&pool).await.expect("Cleanup failed");  // BEFORE
      
      // Test code here
      
      cleanup_database(&pool).await.expect("Cleanup failed");  // AFTER
  }
  ```
- No test interdependencies
- Transaction rollback for atomic operations
- Proper resource cleanup (no file handles, connections left open)

#### Phase 5-8 Improvements

**A. Implement Consistent Cleanup Pattern**
```rust
// Every test must follow this pattern:
#[tokio::test]
async fn test_name() {
    let pool = setup_test_pool().await;
    
    // PRE-TEST CLEANUP - ensures clean state
    cleanup_database(&pool).await.expect("Pre-cleanup failed");
    
    // TEST CODE
    // ...
    
    // POST-TEST CLEANUP - removes test artifacts
    cleanup_database(&pool).await.expect("Post-cleanup failed");
}
```

**B. Test Fixtures Isolation**
- Each test creates its own test data
- Use builders to create isolated instances
- No sharing of data between tests
- Clear setup/teardown in each test

**C. Concurrent Test Safety**
- Use `--test-threads=1` for database tests
- Or use separate database schemas per test
- Ensure no race conditions in assertions

**D. Resource Cleanup Checklist**
```
☐ Database transactions rolled back or cleaned
☐ Temporary files deleted
☐ Connection pools closed
☐ Mock objects reset
☐ File handles closed
```

### 3. ERROR HANDLING

#### Phase 1-4 Gold Standard
- Tests for both success AND failure paths
- Explicit error message validation
- Constraint violation testing
- Invalid input rejection verification
- Specific assertion messages with context

#### Phase 5-8 Improvements

**A. Add Comprehensive Error Path Testing**
```rust
// For EVERY function being tested, add:

// ✅ Success case
#[tokio::test]
async fn test_insert_success() {
    // Test successful operation
}

// ❌ Error cases
#[tokio::test]
async fn test_insert_duplicate_hash_error() {
    // Test duplicate constraint violation
}

#[tokio::test]
async fn test_insert_invalid_format_error() {
    // Test invalid data rejection
}

#[tokio::test]
async fn test_insert_null_required_field_error() {
    // Test NOT NULL constraint violation
}
```

**B. Specific Error Message Validation**
```rust
// BAD: Generic assertion
assert!(result.is_err());

// GOOD: Specific error validation
let err = result.unwrap_err();
assert_eq!(err.kind, ErrorKind::DuplicateFile);
assert!(err.message.contains("content_hash"));
```

**C. Constraint Violation Coverage**
- PRIMARY KEY violations
- FOREIGN KEY violations
- UNIQUE constraint violations
- NOT NULL constraint violations
- CHECK constraint violations
- Type mismatches

**D. Error Message Standards**
```rust
// Each error test must:
// 1. Describe what should fail
// 2. Explain why it should fail
// 3. Validate the specific error

#[tokio::test]
async fn test_delete_nonexistent_file_not_found() {
    // Description: Attempting to delete a file that doesn't exist
    // should return a NotFound error, not silently succeed
    
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    
    let result = FileRepository::delete(&pool, 99999).await;
    
    assert!(result.is_err(), "Should return error for non-existent file");
    let err = result.unwrap_err();
    assert_eq!(err.kind, ErrorKind::NotFound);
    assert!(err.message.contains("99999"), "Error message should include file ID");
    
    cleanup_database(&pool).await.expect("Cleanup failed");
}
```

### 4. DOCUMENTATION

#### Phase 1-4 Gold Standard
- Module-level doc comments explaining test scope
- Test section headers with clear organization
- Helper function documentation
- Inline comments for complex logic
- Self-documenting test names

#### Phase 5-8 Improvements

**A. Add Module Documentation**
```rust
//! Comprehensive tests for FileRepository
//!
//! **Coverage Target:** 85%+ (Trusty Module: 80%+)
//! **Total Tests:** 109 tests
//!
//! Test Categories:
//! 1. Insert Operations (15 tests)
//! 2. Find Operations (12 tests)
//! 3. Update Operations (10 tests)
//! 4. Delete Operations (8 tests)
//! 5. Duplicate Detection (6 tests)
//! 6. List/Pagination (12 tests)
//! 7. Edge Cases (18 tests)
//! 8. Error Handling (10 tests)
//! 9. Concurrent Operations (0 tests - not applicable for single-threaded DB tests)
//!
//! **Database Requirements:**
//! - PostgreSQL 16+
//! - pgvector extension
//! - All migrations applied
//! - Fresh database for each test run
//!
//! **Test Isolation:**
//! - Two-stage cleanup (before/after each test)
//! - No shared state between tests
//! - Run with `--test-threads=1`
```

**B. Add Section Headers**
```rust
// ============================================================================
// SECTION 1: Insert Operations (15 tests)
// ============================================================================
// These tests verify all insertion paths: basic, with all fields, with
// optional fields, with constraints, with special characters, etc.

#[tokio::test]
async fn test_insert_basic_file() {
    // Description: Insert a minimal MIDI file with only required fields
    // Expected: File inserted successfully with auto-generated ID
```

**C. Test Name Standards**
```
Pattern: test_<operation>_<scenario>_<expected_outcome>

✅ GOOD:
- test_insert_basic_file()
- test_insert_with_unicode_filename()
- test_insert_duplicate_hash_returns_constraint_error()
- test_find_by_id_nonexistent_returns_none()
- test_update_timestamps_are_updated()

❌ BAD:
- test_file()
- test_1()
- test_something_works()
```

**D. Helper Function Documentation**
```rust
/// Create a test file with minimal required fields
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `filename` - Name for the test file
///
/// # Returns
/// The inserted file's ID
///
/// # Panics
/// Panics if database insertion fails
///
/// # Example
/// ```
/// let file_id = create_test_file(&pool, "test.mid").await;
/// assert!(file_id > 0);
/// ```
async fn create_test_file(pool: &PgPool, filename: &str) -> i64 {
    // Implementation
}
```

**E. Complex Test Documentation**
```rust
#[tokio::test]
async fn test_insert_with_all_fields_full_validation() {
    // DESCRIPTION:
    // Verify that inserting a file with all fields populated (including
    // optional ones like manufacturer, collection_name, folder_tags)
    // results in complete database record with proper timestamp handling.
    //
    // WHY THIS MATTERS:
    // Tests that optional fields don't cause issues and that the system
    // correctly handles complete file metadata on insertion.
    //
    // WHAT WE'RE CHECKING:
    // ✓ All fields are stored correctly
    // ✓ Timestamps are auto-generated properly
    // ✓ Relationships are created (if applicable)
    // ✓ Data types are respected (no truncation, overflow)
    
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");
    
    // TEST CODE
    
    cleanup_database(&pool).await.expect("Cleanup failed");
}
```

### 5. IMPLEMENTATION CHECKLIST

#### For Each Test File (Phase 5-8)

**Organization**
- ☐ Module-level documentation with coverage target
- ☐ Clear section headers (every 10-15 tests)
- ☐ Helper functions grouped at top or in separate module
- ☐ Consistent formatting and indentation

**Test Isolation**
- ☐ Every test: fresh pool setup
- ☐ Every test: pre-test cleanup
- ☐ Every test: post-test cleanup
- ☐ No test interdependencies
- ☐ No shared static state

**Error Handling**
- ☐ Success paths tested
- ☐ Failure paths tested for each operation
- ☐ Constraint violations tested
- ☐ Invalid input handling tested
- ☐ Specific error messages validated

**Documentation**
- ☐ Module doc comments (scope, coverage, categories)
- ☐ Section headers for test groups
- ☐ Test names follow pattern: `test_<operation>_<scenario>_<outcome>`
- ☐ Helper functions documented with doc comments
- ☐ Complex tests have inline documentation explaining WHY and WHAT

#### Priority Order
1. **Critical** (Do First):
   - Implement two-stage cleanup in all tests
   - Add section headers and organization
   - Fix test naming consistency

2. **High** (Do Second):
   - Add comprehensive error handling tests
   - Add module-level documentation
   - Improve assertion message specificity

3. **Medium** (Do Third):
   - Add helper function documentation
   - Enhance edge case coverage
   - Add performance assertions

4. **Nice-to-Have** (Do If Time):
   - Complex test documentation
   - Performance benchmarking
   - Stress testing scenarios

### 6. EXAMPLE TRANSFORMATION

#### BEFORE (Phase 5-8 Current)
```rust
#[tokio::test]
async fn test_file() {
    let pool = setup_test_pool().await;
    
    let file = NewFileBuilder::new().build();
    let result = FileRepository::insert(&pool, file).await;
    
    assert!(result.is_ok());
}
```

#### AFTER (Phase 1-4 Standard)
```rust
//! Comprehensive tests for FileRepository insert operations
//! Tests verify single/batch insert, duplicates, constraints, and error handling

use sqlx::PgPool;
use midi_pipeline::db::repositories::FileRepository;
use midi_pipeline::db::models::NewFile;

mod fixtures;
mod helpers;

use fixtures::*;
use helpers::db::*;

// ============================================================================
// SECTION 1: Insert Operations (15 tests)
// ============================================================================
// These tests verify all insertion paths: basic, with all fields, with
// optional fields, with constraints, with special characters, etc.

#[tokio::test]
async fn test_insert_basic_file() {
    // DESCRIPTION:
    // Insert a minimal MIDI file with only required fields
    //
    // EXPECTED:
    // File inserted successfully with auto-generated ID, timestamps set,
    // and all optional fields NULL in database
    
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Pre-cleanup failed");
    
    let new_file = NewFileBuilder::new()
        .filename("test.mid")
        .filepath("/test/test.mid")
        .content_hash(random_hash())
        .build();
    
    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert should succeed");
    
    assert!(file_id > 0, "File ID should be positive integer");
    assert_file_exists(&pool, file_id).await;
    assert_file_count(&pool, 1).await;
    
    cleanup_database(&pool).await.expect("Post-cleanup failed");
}

#[tokio::test]
async fn test_insert_duplicate_hash_constraint_error() {
    // DESCRIPTION:
    // Attempting to insert a file with a duplicate content_hash
    // should fail with a constraint violation, not silently succeed
    //
    // WHY THIS MATTERS:
    // Duplicate detection is critical for library management; silent
    // duplicate insertion would corrupt the database
    
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Pre-cleanup failed");
    
    let hash = random_hash();
    
    let file1 = NewFileBuilder::new()
        .filename("original.mid")
        .filepath("/test/original.mid")
        .content_hash(hash.clone())
        .build();
    
    let file2 = NewFileBuilder::new()
        .filename("duplicate.mid")
        .filepath("/test/duplicate.mid")
        .content_hash(hash)  // Same hash!
        .build();
    
    // First insert should succeed
    FileRepository::insert(&pool, file1)
        .await
        .expect("First insert should succeed");
    
    // Second insert with duplicate hash should fail
    let result = FileRepository::insert(&pool, file2).await;
    
    assert!(result.is_err(), "Duplicate hash should cause error");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("duplicate") || err.to_string().contains("constraint"),
        "Error message should mention duplicate or constraint: {}",
        err
    );
    
    // Verify only first file exists in database
    assert_file_count(&pool, 1).await;
    
    cleanup_database(&pool).await.expect("Post-cleanup failed");
}
```

This transformation shows:
- ✅ Clear organization with headers
- ✅ Comprehensive documentation
- ✅ Two-stage cleanup
- ✅ Error path testing
- ✅ Specific assertions with context messages
- ✅ Self-documenting test names

---

## QUICK REFERENCE: Phase 1-4 Standards Checklist

For each test file in Phase 5-8, verify:

### Code Organization
- [ ] Module-level doc comments with scope
- [ ] Section headers every 10-15 tests
- [ ] Clear grouping by operation type
- [ ] Consistent formatting

### Test Isolation
- [ ] Fresh pool per test
- [ ] Pre-test cleanup call
- [ ] Post-test cleanup call
- [ ] No shared state

### Error Handling
- [ ] Success path tested
- [ ] ≥1 failure path per operation
- [ ] Specific error message validation
- [ ] Constraint violations tested

### Documentation
- [ ] Test names: `test_<operation>_<scenario>_<outcome>`
- [ ] Section comments explain purpose
- [ ] Complex tests have inline docs
- [ ] Helper functions documented

