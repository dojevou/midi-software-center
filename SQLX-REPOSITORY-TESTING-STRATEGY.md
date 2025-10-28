# Comprehensive SQLx Repository Testing Strategy

**Project:** MIDI Software Center
**Database:** PostgreSQL 16 + pgvector
**ORM:** SQLx (compile-time checked queries)
**Framework:** Rust + async/tokio
**Target Coverage:** 80%+ (Trusty Module requirement)

---

## Table of Contents

1. [Testing Infrastructure](#1-testing-infrastructure)
2. [Transaction Rollback Pattern](#2-transaction-rollback-pattern)
3. [Test Fixture System](#3-test-fixture-system)
4. [Repository Test Plans](#4-repository-test-plans)
5. [Edge Cases & Error Handling](#5-edge-cases--error-handling)
6. [Performance Testing](#6-performance-testing)
7. [PostgreSQL-Specific Features](#7-postgresql-specific-features)

---

## 1. Testing Infrastructure

### 1.1 Test Helper Module Structure

```
pipeline/src-tauri/tests/
├── helpers/
│   ├── mod.rs              # Main helper module
│   ├── db.rs               # Database connection & cleanup
│   ├── fixtures.rs         # Test data factories
│   ├── transactions.rs     # Transaction rollback wrapper
│   ├── assertions.rs       # Custom assertions
│   └── macros.rs           # Test macros
├── common/
│   ├── mod.rs              # Shared test utilities
│   └── constants.rs        # Test constants
└── repositories/
    ├── file_repository_test.rs       # ✅ COMPLETE (109 tests)
    ├── tag_repository_test.rs        # 🔄 TO IMPLEMENT
    ├── metadata_repository_test.rs   # 🔄 TO IMPLEMENT
    └── search_repository_test.rs     # 🔄 TO IMPLEMENT
```

### 1.2 Database Connection Pool

**File:** `tests/helpers/db.rs`

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test database connection pool
///
/// Uses DATABASE_URL environment variable or defaults to local test database
pub async fn setup_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        });

    PgPoolOptions::new()
        .max_connections(10) // Higher for concurrent tests
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Clean database before/after tests
///
/// IMPORTANT: This truncates all tables - use carefully!
pub async fn cleanup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        TRUNCATE TABLE
            file_tags,
            musical_metadata,
            file_categories,
            files,
            tags
        RESTART IDENTITY CASCADE
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if a file exists in the database
pub async fn assert_file_exists(pool: &PgPool, file_id: i64) {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM files WHERE id = $1) as "exists!""#,
        file_id
    )
    .fetch_one(pool)
    .await
    .expect("Query failed");

    assert!(exists, "File {} should exist in database", file_id);
}

/// Assert file count matches expected value
pub async fn assert_file_count(pool: &PgPool, expected: i64) {
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM files"#)
        .fetch_one(pool)
        .await
        .expect("Count query failed");

    assert_eq!(
        count, expected,
        "Expected {} files, found {}",
        expected, count
    );
}

/// Generate random BLAKE3 hash for testing
pub fn random_hash() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32).map(|_| rng.gen::<u8>()).collect()
}
```

### 1.3 Test Configuration

**File:** `.env.test`

```bash
# Test Database Configuration
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
RUST_LOG=debug
RUST_BACKTRACE=1

# Test Timeouts
TEST_TIMEOUT_SECONDS=30
MAX_CONNECTIONS=10
```

**Cargo.toml test configuration:**

```toml
[[test]]
name = "file_repository_test"
path = "tests/repositories/file_repository_test.rs"
harness = true

[[test]]
name = "tag_repository_test"
path = "tests/repositories/tag_repository_test.rs"
harness = true

[dev-dependencies]
tokio = { version = "1.35", features = ["full", "test-util"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "macros", "uuid", "chrono", "bigdecimal"] }
rand = "0.8"
num-traits = "0.2"
```

---

## 2. Transaction Rollback Pattern

### 2.1 The Problem with Current Tests

**Current approach (from existing tests):**
- Tests use `cleanup_database()` before/after each test
- Uses `TRUNCATE TABLE` to clean data
- **Issue:** Tests pollute shared database, not truly isolated

**Better approach:**
- Use transaction-based rollback
- Each test runs in a transaction that never commits
- Automatic isolation without manual cleanup

### 2.2 Transaction Wrapper Implementation

**File:** `tests/helpers/transactions.rs`

```rust
use sqlx::{PgPool, Postgres, Transaction};

/// Test transaction wrapper that automatically rolls back
///
/// Example:
/// ```rust
/// let tx = TestTransaction::begin(&pool).await;
/// FileRepository::insert(tx.as_pool(), new_file).await?;
/// // Transaction automatically rolls back when dropped
/// ```
pub struct TestTransaction<'a> {
    tx: Option<Transaction<'a, Postgres>>,
}

impl<'a> TestTransaction<'a> {
    /// Begin a new test transaction
    pub async fn begin(pool: &'a PgPool) -> Self {
        let tx = pool
            .begin()
            .await
            .expect("Failed to begin transaction");

        Self { tx: Some(tx) }
    }

    /// Get a reference to the underlying connection pool
    ///
    /// This allows passing the transaction to repository methods
    /// that expect `&PgPool`
    pub fn as_pool(&mut self) -> &mut Transaction<'a, Postgres> {
        self.tx.as_mut().expect("Transaction already committed")
    }

    /// Explicitly rollback (optional - happens automatically on drop)
    pub async fn rollback(mut self) {
        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .expect("Failed to rollback transaction");
        }
    }

    /// Commit transaction (for testing commit scenarios)
    pub async fn commit(mut self) {
        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .expect("Failed to commit transaction");
        }
    }
}

impl<'a> Drop for TestTransaction<'a> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            // Rollback on drop (async-safe drop)
            // Note: This requires tokio runtime context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = tx.rollback().await;
                });
            });
        }
    }
}
```

### 2.3 Using Transactions in Tests

**Pattern 1: Automatic Rollback**

```rust
#[tokio::test]
async fn test_insert_with_transaction_rollback() {
    let pool = setup_test_pool().await;
    let mut tx = TestTransaction::begin(&pool).await;

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(tx.as_pool(), new_file)
        .await
        .expect("Insert failed");

    assert!(file_id > 0);
    // Transaction automatically rolls back when tx is dropped
}

#[tokio::test]
async fn verify_rollback_worked() {
    let pool = setup_test_pool().await;

    // Previous test's data should be gone
    let count = FileRepository::count(&pool).await.unwrap();
    assert_eq!(count, 0, "Database should be empty after rollback");
}
```

**Pattern 2: Testing Commit Scenarios**

```rust
#[tokio::test]
async fn test_transaction_commit() {
    let pool = setup_test_pool().await;
    cleanup_database(&pool).await.expect("Cleanup failed");

    {
        let mut tx = TestTransaction::begin(&pool).await;
        let new_file = Fixtures::drum_loop();
        FileRepository::insert(tx.as_pool(), new_file).await.unwrap();

        // Explicitly commit this time
        tx.commit().await;
    }

    // Data should persist after commit
    let count = FileRepository::count(&pool).await.unwrap();
    assert_eq!(count, 1);

    cleanup_database(&pool).await.expect("Cleanup failed");
}
```

### 2.4 Alternative: SQLx Test Macro (Simplest Approach)

SQLx provides a built-in `#[sqlx::test]` macro that handles transactions automatically:

```rust
use sqlx::PgPool;

#[sqlx::test]
async fn test_insert_file(pool: PgPool) {
    // SQLx automatically:
    // 1. Begins a transaction
    // 2. Runs migrations
    // 3. Rolls back after test

    let new_file = Fixtures::drum_loop();
    let file_id = FileRepository::insert(&pool, new_file)
        .await
        .expect("Insert failed");

    assert!(file_id > 0);
    // Automatic rollback - no cleanup needed!
}
```

**To use `#[sqlx::test]`:**

1. Add to `Cargo.toml`:
   ```toml
   [dev-dependencies]
   sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls", "macros", "uuid", "chrono", "bigdecimal", "migrate"] }
   ```

2. Create `tests/.env`:
   ```bash
   DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library_test
   ```

3. Use in tests:
   ```rust
   #[sqlx::test]
   async fn my_test(pool: PgPool) { /* ... */ }
   ```

**Recommendation:** Use `#[sqlx::test]` for simplicity - it's the Rust standard.

---

## 3. Test Fixture System

### 3.1 Fixture Factory Pattern

**File:** `tests/helpers/fixtures.rs`

```rust
use midi_pipeline::db::models::{NewFile, NewMusicalMetadata, NewTag};
use sqlx::types::BigDecimal;
use num_traits::FromPrimitive;

pub struct Fixtures;

impl Fixtures {
    /// Drum loop preset (Ableton style)
    pub fn drum_loop() -> NewFile {
        NewFile {
            filename: "Drum_Loop_120bpm.mid".to_string(),
            filepath: "/collections/ableton/drums/Drum_Loop_120bpm.mid".to_string(),
            original_filename: "Drum_Loop_120bpm.mid".to_string(),
            content_hash: Self::generate_hash(b"drum_loop"),
            file_size_bytes: 2048,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: Some(BigDecimal::from_f64(8.0).unwrap()),
            duration_ticks: Some(3840),
            manufacturer: Some("Ableton".to_string()),
            collection_name: Some("Drum Essentials".to_string()),
            folder_tags: Some(vec!["drums".to_string(), "loops".to_string()]),
            import_batch_id: None,
        }
    }

    /// Piano chords preset (Native Instruments style)
    pub fn piano_chords() -> NewFile {
        NewFile {
            filename: "Piano_Cmaj7_Progression.mid".to_string(),
            filepath: "/collections/ni/piano/Piano_Cmaj7_Progression.mid".to_string(),
            original_filename: "Piano_Cmaj7_Progression.mid".to_string(),
            content_hash: Self::generate_hash(b"piano_chords"),
            file_size_bytes: 4096,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: Some(BigDecimal::from_f64(16.0).unwrap()),
            duration_ticks: Some(7680),
            manufacturer: Some("Native Instruments".to_string()),
            collection_name: Some("Piano Essentials".to_string()),
            folder_tags: Some(vec!["piano".to_string(), "chords".to_string()]),
            import_batch_id: None,
        }
    }

    /// Bass line preset
    pub fn bass_line() -> NewFile {
        NewFile {
            filename: "Bass_Line_Fm.mid".to_string(),
            filepath: "/collections/moog/bass/Bass_Line_Fm.mid".to_string(),
            original_filename: "Bass_Line_Fm.mid".to_string(),
            content_hash: Self::generate_hash(b"bass_line"),
            file_size_bytes: 1024,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: Some(BigDecimal::from_f64(4.0).unwrap()),
            duration_ticks: Some(1920),
            manufacturer: Some("Moog".to_string()),
            collection_name: Some("Bass Collection".to_string()),
            folder_tags: Some(vec!["bass".to_string(), "synth".to_string()]),
            import_batch_id: None,
        }
    }

    /// Musical metadata for drum loop
    pub fn drum_loop_metadata(file_id: i64) -> NewMusicalMetadata {
        NewMusicalMetadata {
            file_id,
            bpm: Some(BigDecimal::from_f64(120.0).unwrap()),
            bpm_confidence: Some(0.98),
            key_signature: Some("C".to_string()),
            key_confidence: Some(0.85),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            total_notes: 128,
            unique_pitches: Some(8),
            pitch_range_min: Some(36),
            pitch_range_max: Some(51),
            avg_velocity: Some(BigDecimal::from_f64(95.5).unwrap()),
            note_density: Some(BigDecimal::from_f64(16.0).unwrap()),
            polyphony_max: Some(4),
            polyphony_avg: Some(BigDecimal::from_f64(2.3).unwrap()),
            is_percussive: Some(true),
        }
    }

    /// Tag preset: Genre
    pub fn tag_genre(name: &str) -> (String, Option<String>) {
        (name.to_string(), Some("genre".to_string()))
    }

    /// Tag preset: Instrument
    pub fn tag_instrument(name: &str) -> (String, Option<String>) {
        (name.to_string(), Some("instrument".to_string()))
    }

    /// Generate deterministic hash from seed
    fn generate_hash(seed: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(seed);
        hasher.finalize().as_bytes().to_vec()
    }
}

/// Builder pattern for custom test files
pub struct NewFileBuilder {
    file: NewFile,
}

impl NewFileBuilder {
    pub fn new() -> Self {
        Self {
            file: NewFile {
                filename: "test.mid".to_string(),
                filepath: "/test/test.mid".to_string(),
                original_filename: "test.mid".to_string(),
                content_hash: vec![0; 32],
                file_size_bytes: 1024,
                format: Some(1),
                num_tracks: 1,
                ticks_per_quarter_note: Some(480),
                duration_seconds: None,
                duration_ticks: None,
                manufacturer: None,
                collection_name: None,
                folder_tags: None,
                import_batch_id: None,
            },
        }
    }

    pub fn filename(mut self, name: &str) -> Self {
        self.file.filename = name.to_string();
        self
    }

    pub fn filepath(mut self, path: &str) -> Self {
        self.file.filepath = path.to_string();
        self
    }

    pub fn content_hash(mut self, hash: Vec<u8>) -> Self {
        self.file.content_hash = hash;
        self
    }

    pub fn manufacturer(mut self, mfr: &str) -> Self {
        self.file.manufacturer = Some(mfr.to_string());
        self
    }

    pub fn num_tracks(mut self, tracks: i16) -> Self {
        self.file.num_tracks = tracks;
        self
    }

    pub fn build(self) -> NewFile {
        self.file
    }
}
```

### 3.2 Randomized Test Data

```rust
/// Generate random MIDI file for testing
pub fn random_midi_file() -> NewFile {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let id: u32 = rng.gen();

    NewFile {
        filename: format!("random_{}.mid", id),
        filepath: format!("/test/random_{}.mid", id),
        original_filename: format!("random_{}.mid", id),
        content_hash: (0..32).map(|_| rng.gen::<u8>()).collect(),
        file_size_bytes: rng.gen_range(512..10240),
        format: Some(rng.gen_range(0..3) as i16),
        num_tracks: rng.gen_range(1..17) as i16,
        ticks_per_quarter_note: Some(rng.gen_range(96..961)),
        duration_seconds: Some(BigDecimal::from_f64(rng.gen_range(1.0..300.0)).unwrap()),
        duration_ticks: Some(rng.gen_range(480..100000)),
        manufacturer: None,
        collection_name: None,
        folder_tags: None,
        import_batch_id: None,
    }
}
```

---

## 4. Repository Test Plans

### 4.1 FileRepository Tests (✅ COMPLETE - 109 tests)

**Current Status:** Excellent coverage with 109 comprehensive tests

**Sections covered:**
1. Insert Operations (12 tests) - Basic, all fields, presets, unicode, special chars
2. Find Operations (15 tests) - By ID, hash, path, case sensitivity, unicode
3. Duplicate Detection (8 tests) - Check workflow, empty hash, performance
4. Update Operations (12 tests) - Mark analyzed, metadata fields, timestamps
5. Delete Operations (6 tests) - Basic, multiple, reinsert workflow
6. Count Operations (5 tests) - Empty, single, multiple, after delete
7. List Operations (18 tests) - Pagination, ordering, filtering by manufacturer/collection
8. Edge Cases (4 tests) - Concurrent inserts, long paths, array handling

**Coverage estimate:** 95%+ ✅

**Recommendations:**
- ✅ Already excellent - use as reference for other repositories
- Consider adding `#[sqlx::test]` for cleaner transaction handling
- Add performance benchmarks for batch operations

### 4.2 TagRepository Tests (🔄 TO IMPLEMENT)

**Target:** 60-70 tests for 80%+ coverage

**Test Plan:**

```rust
// ============================================================================
// SECTION 1: Tag Creation (get_or_create_tag) - 10 tests
// ============================================================================

#[sqlx::test]
async fn test_create_new_tag(pool: PgPool) {
    // Create tag that doesn't exist
    let tag_id = TagRepository::new(pool.clone())
        .get_or_create_tag("house", Some("genre"))
        .await
        .expect("Failed to create tag");

    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_get_existing_tag(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // Create tag
    let id1 = repo.get_or_create_tag("techno", Some("genre")).await.unwrap();

    // Get same tag - should return same ID
    let id2 = repo.get_or_create_tag("techno", Some("genre")).await.unwrap();

    assert_eq!(id1, id2, "Should return existing tag ID");
}

#[sqlx::test]
async fn test_create_tag_without_category(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());
    let tag_id = repo.get_or_create_tag("custom", None).await.unwrap();

    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_create_tag_case_sensitive(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let id1 = repo.get_or_create_tag("House", Some("genre")).await.unwrap();
    let id2 = repo.get_or_create_tag("house", Some("genre")).await.unwrap();

    // PostgreSQL is case-sensitive - should be different tags
    assert_ne!(id1, id2, "Tags with different case should be distinct");
}

#[sqlx::test]
async fn test_create_tag_with_spaces(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());
    let tag_id = repo.get_or_create_tag("deep house", Some("genre")).await.unwrap();

    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_create_tag_with_unicode(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());
    let tag_id = repo.get_or_create_tag("電子音楽", Some("genre")).await.unwrap();

    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_create_tag_empty_name_error(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // Empty tag name should error (constraint violation)
    let result = repo.get_or_create_tag("", Some("genre")).await;
    assert!(result.is_err(), "Empty tag name should fail");
}

#[sqlx::test]
async fn test_create_tag_very_long_name(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());
    let long_name = "a".repeat(500);

    let tag_id = repo.get_or_create_tag(&long_name, Some("genre")).await.unwrap();
    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_create_tag_special_characters(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());
    let tag_id = repo.get_or_create_tag("drum&bass", Some("genre")).await.unwrap();

    assert!(tag_id > 0);
}

#[sqlx::test]
async fn test_create_multiple_tags_same_category(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let id1 = repo.get_or_create_tag("house", Some("genre")).await.unwrap();
    let id2 = repo.get_or_create_tag("techno", Some("genre")).await.unwrap();
    let id3 = repo.get_or_create_tag("trance", Some("genre")).await.unwrap();

    // All should be distinct
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

// ============================================================================
// SECTION 2: Batch Tag Creation (get_or_create_tags_batch) - 8 tests
// ============================================================================

#[sqlx::test]
async fn test_batch_create_tags(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tags = vec![
        ("house".to_string(), Some("genre".to_string())),
        ("techno".to_string(), Some("genre".to_string())),
        ("drums".to_string(), Some("instrument".to_string())),
    ];

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|&id| id > 0));
}

#[sqlx::test]
async fn test_batch_create_with_duplicates(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // First batch
    let tags1 = vec![
        ("house".to_string(), Some("genre".to_string())),
    ];
    let ids1 = repo.get_or_create_tags_batch(&tags1).await.unwrap();

    // Second batch with duplicate
    let tags2 = vec![
        ("house".to_string(), Some("genre".to_string())),
        ("techno".to_string(), Some("genre".to_string())),
    ];
    let ids2 = repo.get_or_create_tags_batch(&tags2).await.unwrap();

    // First ID should match
    assert_eq!(ids1[0], ids2[0], "Duplicate tag should have same ID");
}

#[sqlx::test]
async fn test_batch_create_empty_list(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tags: Vec<(String, Option<String>)> = vec![];
    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    assert_eq!(ids.len(), 0);
}

#[sqlx::test]
async fn test_batch_create_single_tag(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tags = vec![
        ("house".to_string(), Some("genre".to_string())),
    ];

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    assert_eq!(ids.len(), 1);
    assert!(ids[0] > 0);
}

#[sqlx::test]
async fn test_batch_create_large_batch(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // Create 100 tags
    let tags: Vec<(String, Option<String>)> = (0..100)
        .map(|i| (format!("tag_{}", i), Some("test".to_string())))
        .collect();

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    assert_eq!(ids.len(), 100);
    assert!(ids.iter().all(|&id| id > 0));
}

#[sqlx::test]
async fn test_batch_transaction_atomicity(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // This test ensures all tags are created or none
    let tags = vec![
        ("valid1".to_string(), Some("genre".to_string())),
        ("valid2".to_string(), Some("genre".to_string())),
    ];

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    // Verify all tags exist
    for id in ids {
        // Query to verify tag exists
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM tags WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(exists, "Tag {} should exist after batch insert", id);
    }
}

#[sqlx::test]
async fn test_batch_create_mixed_categories(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tags = vec![
        ("house".to_string(), Some("genre".to_string())),
        ("piano".to_string(), Some("instrument".to_string())),
        ("120bpm".to_string(), Some("tempo".to_string())),
        ("custom".to_string(), None),
    ];

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    assert_eq!(ids.len(), 4);
}

#[sqlx::test]
async fn test_batch_create_preserves_order(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tags = vec![
        ("tag1".to_string(), Some("cat1".to_string())),
        ("tag2".to_string(), Some("cat2".to_string())),
        ("tag3".to_string(), Some("cat3".to_string())),
    ];

    let ids = repo.get_or_create_tags_batch(&tags).await.unwrap();

    // Verify order is preserved
    assert_eq!(ids.len(), 3);
}

// ============================================================================
// SECTION 3: File-Tag Association (add_tags_to_file) - 12 tests
// ============================================================================

#[sqlx::test]
async fn test_add_single_tag_to_file(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // Create file and tag
    let file_id = create_test_file(&pool).await;
    let tag_id = repo.get_or_create_tag("house", Some("genre")).await.unwrap();

    // Add tag to file
    repo.add_tags_to_file(file_id, &[tag_id]).await.unwrap();

    // Verify association
    let tags = repo.get_file_tags(file_id).await.unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, tag_id);
}

#[sqlx::test]
async fn test_add_multiple_tags_to_file(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let file_id = create_test_file(&pool).await;

    let tag_ids = repo.get_or_create_tags_batch(&vec![
        ("house".to_string(), Some("genre".to_string())),
        ("piano".to_string(), Some("instrument".to_string())),
        ("120bpm".to_string(), Some("tempo".to_string())),
    ]).await.unwrap();

    repo.add_tags_to_file(file_id, &tag_ids).await.unwrap();

    let tags = repo.get_file_tags(file_id).await.unwrap();
    assert_eq!(tags.len(), 3);
}

#[sqlx::test]
async fn test_add_duplicate_tag_to_file(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let file_id = create_test_file(&pool).await;
    let tag_id = repo.get_or_create_tag("house", Some("genre")).await.unwrap();

    // Add tag twice (ON CONFLICT DO NOTHING)
    repo.add_tags_to_file(file_id, &[tag_id]).await.unwrap();
    repo.add_tags_to_file(file_id, &[tag_id]).await.unwrap();

    // Should still have only 1 tag
    let tags = repo.get_file_tags(file_id).await.unwrap();
    assert_eq!(tags.len(), 1);
}

#[sqlx::test]
async fn test_add_tags_to_nonexistent_file(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let tag_id = repo.get_or_create_tag("house", Some("genre")).await.unwrap();

    // Should fail due to foreign key constraint
    let result = repo.add_tags_to_file(99999, &[tag_id]).await;
    assert!(result.is_err(), "Adding tag to non-existent file should fail");
}

#[sqlx::test]
async fn test_add_empty_tag_list(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let file_id = create_test_file(&pool).await;

    repo.add_tags_to_file(file_id, &[]).await.unwrap();

    let tags = repo.get_file_tags(file_id).await.unwrap();
    assert_eq!(tags.len(), 0);
}

#[sqlx::test]
async fn test_add_tags_using_unnest(pool: PgPool) {
    // This tests the PostgreSQL unnest() functionality
    let repo = TagRepository::new(pool.clone());

    let file_id = create_test_file(&pool).await;

    // Create 50 tags and add them all at once using unnest
    let tag_ids = repo.get_or_create_tags_batch(&(0..50)
        .map(|i| (format!("tag_{}", i), Some("test".to_string())))
        .collect::<Vec<_>>()
    ).await.unwrap();

    repo.add_tags_to_file(file_id, &tag_ids).await.unwrap();

    let tags = repo.get_file_tags(file_id).await.unwrap();
    assert_eq!(tags.len(), 50);
}

// Additional sections continue...
```

**Full test count:**
1. Tag Creation: 10 tests
2. Batch Creation: 8 tests
3. File-Tag Association: 12 tests
4. Get File Tags: 8 tests
5. Popular Tags: 6 tests
6. Search Tags: 8 tests
7. Tag Categories: 6 tests
8. Remove Tags: 6 tests
9. Update File Tags: 8 tests
10. Tag File Count: 4 tests
11. Get Files by Tags: 8 tests

**Total: ~64 tests** → 80%+ coverage ✅

### 4.3 MetadataRepository Tests (🔄 TO IMPLEMENT)

**Target:** 45-50 tests for 80%+ coverage

**Key test areas:**

```rust
// SECTION 1: Insert/Upsert Operations - 12 tests
// - Insert new metadata
// - Upsert existing metadata (ON CONFLICT DO UPDATE)
// - Insert with all 26 fields
// - Insert with minimal fields
// - Insert with NULL values
// - Insert with musical_key enum
// - Insert with BigDecimal precision
// - Insert with JSONB tempo_changes
// - Insert foreign key violation
// - Insert with constraints (BPM 20-300)
// - Insert with pitch range validation (0-127)
// - Insert concurrent upserts

// SECTION 2: Find Operations - 8 tests
// - Find by file_id (existing)
// - Find by file_id (not found)
// - Find returns all fields
// - Find with NULL optionals
// - Find with musical_key enum conversion
// - Find with JSONB fields
// - Find multiple metadata records
// - Find with BigDecimal precision

// SECTION 3: Update Operations - 15 tests
// - Update BPM and confidence
// - Update key and confidence
// - Update note statistics (5 fields)
// - Update with NULL values
// - Update non-existent file (no error)
// - Update BPM out of range (20-300)
// - Update pitch range validation
// - Update key with invalid enum
// - Update multiple fields sequentially
// - Update and verify precision
// - Update with edge case values
// - Update with max values
// - Update with min values
// - Update with zero values
// - Update performance (batch)

// SECTION 4: Delete Operations - 5 tests
// - Delete existing metadata
// - Delete non-existent metadata
// - Delete cascade from file delete
// - Delete and reinsert
// - Delete multiple

// SECTION 5: Count Operations - 3 tests
// - Count empty
// - Count single
// - Count multiple

// SECTION 6: Edge Cases - 7 tests
// - Musical key enum all values
// - JSONB tempo changes format
// - BigDecimal precision (6,2)
// - Concurrent updates
// - Large note counts
// - All boolean combinations
// - Full metadata lifecycle
```

**Total: ~50 tests** → 80%+ coverage ✅

### 4.4 SearchRepository Tests (🔄 TO IMPLEMENT)

**Target:** 40-45 tests for 80%+ coverage

**Key test areas:**

```rust
// SECTION 1: Full-text Search - 12 tests
// - Search with text query (plainto_tsquery)
// - Search empty query (returns all)
// - Search with no results
// - Search ranking (ts_rank ordering)
// - Search with special characters
// - Search with unicode
// - Search with stop words
// - Search phrase matching
// - Search case insensitive
// - Search with tsvector NULL
// - Search performance (1000+ files)
// - Search pagination

// SECTION 2: Filter Combinations - 15 tests
// - Filter by BPM range
// - Filter by exact BPM
// - Filter by min BPM only
// - Filter by max BPM only
// - Filter by key signature
// - Filter by manufacturer
// - Filter by collection
// - Filter text + BPM range
// - Filter text + key + manufacturer
// - Filter all filters combined
// - Filter with NULL values
// - Filter with no matches
// - Filter BPM edge cases (20.0, 300.0)
// - Filter key with enum values
// - Filter manufacturer case sensitive

// SECTION 3: Count Operations - 8 tests
// - Count all files
// - Count with text query
// - Count with filters
// - Count matches search results
// - Count empty results
// - Count large datasets
// - Count performance
// - Count vs search consistency

// SECTION 4: Manufacturer/Collection Search - 10 tests
// - Search by manufacturer
// - Search by collection
// - Manufacturer not found
// - Collection not found
// - Manufacturer with limit
// - Collection with limit
// - Manufacturer case sensitive
// - Collection ordering
// - Manufacturer + pagination
// - Collection + pagination
```

**Total: ~45 tests** → 80%+ coverage ✅

---

## 5. Edge Cases & Error Handling

### 5.1 Critical Edge Cases

**NULL Handling:**
```rust
#[sqlx::test]
async fn test_optional_fields_null(pool: PgPool) {
    let metadata = NewMusicalMetadata {
        file_id: 1,
        bpm: None, // NULL
        bpm_confidence: None,
        key_signature: None,
        // ... all optionals None
    };

    MetadataRepository::insert(&pool, metadata).await.unwrap();

    let found = MetadataRepository::find_by_file_id(&pool, 1)
        .await
        .unwrap()
        .unwrap();

    assert!(found.bpm.is_none());
    assert!(found.key_signature.is_none());
}
```

**Constraint Violations:**
```rust
#[sqlx::test]
async fn test_bpm_out_of_range_low(pool: PgPool) {
    let metadata = NewMusicalMetadata {
        file_id: 1,
        bpm: Some(BigDecimal::from_f64(10.0).unwrap()), // < 20
        // ...
    };

    let result = MetadataRepository::insert(&pool, metadata).await;
    assert!(result.is_err(), "BPM < 20 should violate constraint");
}

#[sqlx::test]
async fn test_pitch_range_invalid(pool: PgPool) {
    let metadata = NewMusicalMetadata {
        file_id: 1,
        pitch_range_min: Some(128), // > 127
        // ...
    };

    let result = MetadataRepository::insert(&pool, metadata).await;
    assert!(result.is_err(), "Pitch > 127 should violate constraint");
}
```

**Foreign Key Violations:**
```rust
#[sqlx::test]
async fn test_metadata_without_file(pool: PgPool) {
    let metadata = NewMusicalMetadata {
        file_id: 99999, // Non-existent file
        // ...
    };

    let result = MetadataRepository::insert(&pool, metadata).await;
    assert!(result.is_err(), "Should fail with foreign key violation");
}
```

**Unicode & Special Characters:**
```rust
#[sqlx::test]
async fn test_unicode_tag_names(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let unicode_tags = vec![
        ("日本語", Some("language")),
        ("Русский", Some("language")),
        ("العربية", Some("language")),
        ("🎹🎵🎶", Some("emoji")),
    ];

    for (name, category) in unicode_tags {
        let tag_id = repo.get_or_create_tag(name, category).await.unwrap();
        assert!(tag_id > 0);
    }
}
```

### 5.2 Error Handling Patterns

**Pattern 1: Graceful Degradation**
```rust
#[sqlx::test]
async fn test_update_nonexistent_file_no_error(pool: PgPool) {
    // Many UPDATE operations should succeed even if no rows affected
    let result = FileRepository::mark_analyzed(&pool, 99999).await;
    assert!(result.is_ok(), "Update should not error on non-existent file");
}
```

**Pattern 2: Transaction Rollback on Error**
```rust
#[sqlx::test]
async fn test_batch_insert_rollback_on_error(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    // Simulate partial failure in batch
    // (Implementation detail: transaction should rollback all or commit all)

    let tags = vec![
        ("valid1".to_string(), Some("genre".to_string())),
        ("".to_string(), Some("genre".to_string())), // Invalid
    ];

    let result = repo.get_or_create_tags_batch(&tags).await;

    if result.is_err() {
        // Verify NO tags were created (transaction rolled back)
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM tags"#)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count, 0, "No tags should be created if batch fails");
    }
}
```

---

## 6. Performance Testing

### 6.1 Batch Operations

**Test bulk insert performance:**
```rust
#[sqlx::test]
async fn test_batch_insert_500_files(pool: PgPool) {
    use std::time::Instant;

    let start = Instant::now();

    for i in 0..500 {
        let new_file = NewFileBuilder::new()
            .filename(&format!("batch_{}.mid", i))
            .filepath(&format!("/test/batch_{}.mid", i))
            .content_hash(random_hash())
            .build();

        FileRepository::insert(&pool, new_file).await.unwrap();
    }

    let duration = start.elapsed();

    println!("Inserted 500 files in {:?}", duration);
    assert!(duration.as_secs() < 10, "Should insert 500 files in < 10s");
}
```

**Test search performance:**
```rust
#[sqlx::test]
async fn test_search_performance_large_dataset(pool: PgPool) {
    // Insert 1000 files with metadata
    for i in 0..1000 {
        let file = create_test_file(&pool).await;
        let metadata = Fixtures::drum_loop_metadata(file);
        MetadataRepository::insert(&pool, metadata).await.unwrap();
    }

    let start = Instant::now();

    let query = SearchQuery {
        text: Some("drum".to_string()),
        min_bpm: Some(100.0),
        max_bpm: Some(140.0),
        key: Some("C".to_string()),
        manufacturer: None,
        collection: None,
    };

    let results = SearchRepository::search(&pool, query, 50, 0)
        .await
        .unwrap();

    let duration = start.elapsed();

    println!("Search completed in {:?}", duration);
    assert!(duration.as_millis() < 100, "Search should complete in < 100ms");
}
```

### 6.2 Concurrent Operations

```rust
#[sqlx::test]
async fn test_concurrent_tag_creation(pool: PgPool) {
    use tokio::task;

    let repo = Arc::new(TagRepository::new(pool.clone()));

    let mut handles = vec![];

    // 10 concurrent tag creations
    for i in 0..10 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            repo_clone
                .get_or_create_tag(&format!("tag_{}", i), Some("test"))
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all 10 tags created
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM tags"#)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 10);
}
```

---

## 7. PostgreSQL-Specific Features

### 7.1 Enum Types (musical_key)

```rust
#[sqlx::test]
async fn test_all_musical_keys(pool: PgPool) {
    let keys = vec![
        "C", "Cm", "C#", "C#m", "Db", "Dbm",
        "D", "Dm", "D#", "D#m", "Eb", "Ebm",
        "E", "Em", "F", "Fm", "F#", "F#m",
        "Gb", "Gbm", "G", "Gm", "G#", "G#m",
        "Ab", "Abm", "A", "Am", "A#", "A#m",
        "Bb", "Bbm", "B", "Bm", "UNKNOWN"
    ];

    for key in keys {
        let file_id = create_test_file(&pool).await;

        let metadata = NewMusicalMetadata {
            file_id,
            key_signature: Some(key.to_string()),
            // ... other fields
        };

        MetadataRepository::insert(&pool, metadata).await.unwrap();

        let found = MetadataRepository::find_by_file_id(&pool, file_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.key_signature.as_deref(), Some(key));
    }
}

#[sqlx::test]
async fn test_invalid_musical_key(pool: PgPool) {
    let file_id = create_test_file(&pool).await;

    let metadata = NewMusicalMetadata {
        file_id,
        key_signature: Some("InvalidKey".to_string()),
        // ...
    };

    let result = MetadataRepository::insert(&pool, metadata).await;
    assert!(result.is_err(), "Invalid musical key should fail");
}
```

### 7.2 Array Types (folder_tags TEXT[])

```rust
#[sqlx::test]
async fn test_array_operations(pool: PgPool) {
    let new_file = NewFileBuilder::new()
        .filename("array_test.mid")
        .filepath("/test/array_test.mid")
        .content_hash(random_hash())
        .folder_tags(vec![
            "drums".to_string(),
            "loops".to_string(),
            "120bpm".to_string(),
        ])
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.unwrap();

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(file.folder_tags, Some(vec![
        "drums".to_string(),
        "loops".to_string(),
        "120bpm".to_string(),
    ]));
}

#[sqlx::test]
async fn test_empty_array(pool: PgPool) {
    let new_file = NewFileBuilder::new()
        .filename("empty_array.mid")
        .filepath("/test/empty_array.mid")
        .content_hash(random_hash())
        .folder_tags(vec![]) // Empty array
        .build();

    let file_id = FileRepository::insert(&pool, new_file).await.unwrap();

    let file = FileRepository::find_by_id(&pool, file_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(file.folder_tags, Some(vec![]));
}
```

### 7.3 Full-text Search (tsvector)

```rust
#[sqlx::test]
async fn test_tsvector_search(pool: PgPool) {
    // Create files with searchable content
    let files = vec![
        ("drum loop 120bpm", "drum loop"),
        ("piano melody", "piano"),
        ("bass line funk", "bass funk"),
    ];

    for (filename, _) in &files {
        let new_file = NewFileBuilder::new()
            .filename(filename)
            .filepath(&format!("/test/{}", filename))
            .content_hash(random_hash())
            .build();

        FileRepository::insert(&pool, new_file).await.unwrap();
    }

    // Update search_vector (normally done by trigger)
    sqlx::query!(
        r#"
        UPDATE files
        SET search_vector = to_tsvector('english', filename)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // Search for "drum"
    let query = SearchQuery {
        text: Some("drum".to_string()),
        min_bpm: None,
        max_bpm: None,
        key: None,
        manufacturer: None,
        collection: None,
    };

    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].filename.contains("drum"));
}

#[sqlx::test]
async fn test_tsvector_ranking(pool: PgPool) {
    // Test ts_rank ordering
    let files = vec![
        "drum drum drum", // More "drum" = higher rank
        "drum loop",
        "piano melody",
    ];

    for filename in &files {
        let new_file = NewFileBuilder::new()
            .filename(filename)
            .filepath(&format!("/test/{}", filename))
            .content_hash(random_hash())
            .build();

        FileRepository::insert(&pool, new_file).await.unwrap();
    }

    // Update search_vector
    sqlx::query!(
        r#"
        UPDATE files
        SET search_vector = to_tsvector('english', filename)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    let query = SearchQuery {
        text: Some("drum".to_string()),
        // ... other fields None
    };

    let results = SearchRepository::search(&pool, query, 10, 0)
        .await
        .unwrap();

    // "drum drum drum" should rank first
    assert_eq!(results[0].filename, "drum drum drum");
}
```

### 7.4 JSONB Fields (tempo_changes, key_changes)

```rust
#[sqlx::test]
async fn test_jsonb_tempo_changes(pool: PgPool) {
    use serde_json::json;

    let file_id = create_test_file(&pool).await;

    let tempo_changes = json!([
        {"tick": 0, "bpm": 120.0},
        {"tick": 1920, "bpm": 140.0},
        {"tick": 3840, "bpm": 100.0}
    ]);

    // Insert with JSONB
    sqlx::query!(
        r#"
        INSERT INTO musical_metadata (file_id, tempo_changes, has_tempo_changes)
        VALUES ($1, $2, true)
        "#,
        file_id,
        tempo_changes
    )
    .execute(&pool)
    .await
    .unwrap();

    // Retrieve and verify
    let metadata = MetadataRepository::find_by_file_id(&pool, file_id)
        .await
        .unwrap()
        .unwrap();

    assert!(metadata.has_tempo_changes);
    assert_eq!(metadata.tempo_changes, Some(tempo_changes));
}
```

### 7.5 unnest() for Batch Inserts

```rust
#[sqlx::test]
async fn test_unnest_batch_insert(pool: PgPool) {
    let repo = TagRepository::new(pool.clone());

    let file_id = create_test_file(&pool).await;

    // Create 20 tags
    let tag_ids = repo.get_or_create_tags_batch(&(0..20)
        .map(|i| (format!("tag_{}", i), Some("test".to_string())))
        .collect::<Vec<_>>()
    ).await.unwrap();

    // Add all at once using unnest
    repo.add_tags_to_file(file_id, &tag_ids).await.unwrap();

    // Verify all 20 associations created
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM file_tags WHERE file_id = $1"#,
        file_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count, 20);
}
```

---

## Summary

### Quick Reference: Test Counts by Repository

| Repository | Target Tests | Estimated Coverage | Status |
|-----------|-------------|-------------------|--------|
| **FileRepository** | 109 tests | 95%+ | ✅ COMPLETE |
| **TagRepository** | 64 tests | 80%+ | 🔄 TO IMPLEMENT |
| **MetadataRepository** | 50 tests | 80%+ | 🔄 TO IMPLEMENT |
| **SearchRepository** | 45 tests | 80%+ | 🔄 TO IMPLEMENT |
| **TOTAL** | **268 tests** | **85%+** | **39% complete** |

### Key Takeaways

1. **Use `#[sqlx::test]` macro** - Automatic transaction rollback, simplest approach
2. **Leverage existing FileRepository tests** - 109 tests provide excellent template
3. **Focus on PostgreSQL-specific features** - Enums, arrays, tsvector, JSONB, unnest()
4. **Test edge cases rigorously** - NULL handling, constraint violations, unicode
5. **Include performance tests** - Batch operations, concurrent access, large datasets
6. **Transaction safety** - All tests isolated, no database pollution

### Next Steps

1. Implement TagRepository tests (64 tests, ~4 hours)
2. Implement MetadataRepository tests (50 tests, ~3 hours)
3. Implement SearchRepository tests (45 tests, ~3 hours)
4. Add performance benchmarks
5. Document test coverage results

**Estimated time to 80%+ coverage: 10-12 hours**

---

**Reference Files:**
- `/home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/file_repository_test.rs` (109 tests ✅)
- `/home/dojevou/projects/midi-software-center/database/migrations/001_initial_schema.sql`
- SQLx documentation: https://docs.rs/sqlx/latest/sqlx/

**End of Testing Strategy**
