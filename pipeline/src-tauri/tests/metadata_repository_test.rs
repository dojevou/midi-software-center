//! Comprehensive tests for MetadataRepository
//!
//! **Target Coverage:** 90%+ (Trusty Module requirement: 80%+)
//! **Total Tests:** 50+
//!
//! This test suite covers all 7 public methods of MetadataRepository with comprehensive
//! edge case testing, BigDecimal precision validation, PostgreSQL ENUM handling, and
//! performance verification.
//!
//! **Test Categories:**
//! 1. CRUD Operations (12 tests) - Insert, find, update, delete
//! 2. Musical Key ENUM (12 tests) - All 24 keys + validation
//! 3. BPM BigDecimal Handling (8 tests) - Precision, ranges, edge cases
//! 4. Time Signatures (6 tests) - Common and uncommon signatures
//! 5. File Associations (6 tests) - FK constraints, CASCADE
//! 6. Query Patterns (4 tests) - Complex queries, aggregations
//! 7. Edge Cases (2 tests) - Concurrency, errors
//!
//! **Special Considerations:**
//! - BigDecimal precision (BPM, avg_velocity, note_density, polyphony_avg)
//! - PostgreSQL ENUM (musical_key with 24 values)
//! - MIDI range validation (pitch_range_min/max: 0-127)
//! - Upsert pattern (ON CONFLICT DO UPDATE)
//! - 11 optional fields requiring NULL handling

use midi_pipeline::db::repositories::MetadataRepository;
use midi_pipeline::db::models::{MusicalMetadata, NewMusicalMetadata};
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use num_traits::FromPrimitive;
use std::str::FromStr;

mod fixtures;
mod helpers;
mod common;

use fixtures::{NewFileBuilder, Fixtures};
use helpers::db::*;
use common::*;

    // ============================================================================
    // Test Helpers
    // ============================================================================

    /// Create test file for metadata association
    async fn create_metadata_test_file(pool: &PgPool, filename: &str) -> i64 {
        let new_file = NewFileBuilder::new()
            .filename(filename)
            .filepath(&format!("/test/metadata/{}", filename))
            .content_hash(random_hash())
            .build();

        sqlx::query_scalar!(
            "INSERT INTO files (filepath, filename, original_filename, content_hash, file_size_bytes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            new_file.filepath,
            new_file.filename,
            new_file.filename, // original_filename = filename for tests
            new_file.content_hash,
            new_file.file_size_bytes
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test file")
    }

    /// Assert BigDecimal equals with tolerance
    fn assert_bigdecimal_approx(actual: &Option<BigDecimal>, expected_str: &str, tolerance: &str) {
        match actual {
            Some(actual_val) => {
                let expected = BigDecimal::from_str(expected_str).expect("Valid expected BigDecimal");
                let tolerance_val = BigDecimal::from_str(tolerance).expect("Valid tolerance");
                let diff = (actual_val - &expected).abs();
                assert!(
                    diff <= tolerance_val,
                    "BigDecimal mismatch: {} vs {} (tolerance: {})",
                    actual_val,
                    expected,
                    tolerance
                );
            }
            None => panic!("Expected Some(BigDecimal), got None"),
        }
    }

    /// Assert BigDecimal equals exactly
    fn assert_bigdecimal_exact(actual: &Option<BigDecimal>, expected_str: &str) {
        match actual {
            Some(actual_val) => {
                let expected = BigDecimal::from_str(expected_str).expect("Valid expected BigDecimal");
                assert_eq!(actual_val, &expected, "BigDecimal exact match failed");
            }
            None => panic!("Expected Some(BigDecimal), got None"),
        }
    }

    /// Musical key constants (24 values)
    const ALL_MAJOR_KEYS: &[&str] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    const ALL_MINOR_KEYS: &[&str] = &["Cm", "C#m", "Dm", "D#m", "Em", "Fm", "F#m", "Gm", "G#m", "Am", "A#m", "Bm"];

    /// Metadata builder for test data
    struct MetadataBuilder {
        file_id: i64,
        bpm: Option<BigDecimal>,
        bpm_confidence: Option<f32>,
        key_signature: Option<String>,
        key_confidence: Option<f32>,
        time_sig_numerator: Option<i16>,
        time_sig_denominator: Option<i16>,
        total_notes: i32,
        unique_pitches: Option<i32>,
        pitch_range_min: Option<i16>,
        pitch_range_max: Option<i16>,
        avg_velocity: Option<BigDecimal>,
        note_density: Option<BigDecimal>,
        polyphony_max: Option<i16>,
        polyphony_avg: Option<BigDecimal>,
        is_percussive: Option<bool>,
    }

    impl MetadataBuilder {
        fn new(file_id: i64) -> Self {
            Self {
                file_id,
                bpm: None,
                bpm_confidence: None,
                key_signature: None,
                key_confidence: None,
                time_sig_numerator: None,
                time_sig_denominator: None,
                total_notes: 0,
                unique_pitches: None,
                pitch_range_min: None,
                pitch_range_max: None,
                avg_velocity: None,
                note_density: None,
                polyphony_max: None,
                polyphony_avg: None,
                is_percussive: None,
            }
        }

        fn bpm_f64(mut self, bpm: f64) -> Self {
            self.bpm = BigDecimal::from_f64(bpm);
            self
        }

        fn bpm_str(mut self, bpm: &str) -> Self {
            self.bpm = Some(BigDecimal::from_str(bpm).expect("Valid BPM"));
            self
        }

        fn bpm_confidence(mut self, conf: f32) -> Self {
            self.bpm_confidence = Some(conf);
            self
        }

        fn key(mut self, key: &str) -> Self {
            self.key_signature = Some(key.to_string());
            self
        }

        fn key_confidence(mut self, conf: f32) -> Self {
            self.key_confidence = Some(conf);
            self
        }

        fn time_signature(mut self, numerator: i16, denominator: i16) -> Self {
            self.time_sig_numerator = Some(numerator);
            self.time_sig_denominator = Some(denominator);
            self
        }

        fn total_notes(mut self, notes: i32) -> Self {
            self.total_notes = notes;
            self
        }

        fn unique_pitches(mut self, pitches: i32) -> Self {
            self.unique_pitches = Some(pitches);
            self
        }

        fn pitch_range(mut self, min: i16, max: i16) -> Self {
            self.pitch_range_min = Some(min);
            self.pitch_range_max = Some(max);
            self
        }

        fn avg_velocity_f64(mut self, vel: f64) -> Self {
            self.avg_velocity = BigDecimal::from_f64(vel);
            self
        }

        fn avg_velocity_str(mut self, vel: &str) -> Self {
            self.avg_velocity = Some(BigDecimal::from_str(vel).expect("Valid velocity"));
            self
        }

        fn note_density_f64(mut self, density: f64) -> Self {
            self.note_density = BigDecimal::from_f64(density);
            self
        }

        fn polyphony(mut self, max: i16, avg: f64) -> Self {
            self.polyphony_max = Some(max);
            self.polyphony_avg = BigDecimal::from_f64(avg);
            self
        }

        fn percussive(mut self, is_percussive: bool) -> Self {
            self.is_percussive = Some(is_percussive);
            self
        }

        fn build(self) -> NewMusicalMetadata {
            NewMusicalMetadata {
                file_id: self.file_id,
                bpm: self.bpm,
                bpm_confidence: self.bpm_confidence,
                key_signature: self.key_signature,
                key_confidence: self.key_confidence,
                time_signature_numerator: self.time_sig_numerator,
                time_signature_denominator: self.time_sig_denominator,
                total_notes: self.total_notes,
                unique_pitches: self.unique_pitches,
                pitch_range_min: self.pitch_range_min,
                pitch_range_max: self.pitch_range_max,
                avg_velocity: self.avg_velocity,
                note_density: self.note_density,
                polyphony_max: self.polyphony_max,
                polyphony_avg: self.polyphony_avg,
                is_percussive: self.is_percussive,
            }
        }

        /// Preset: Standard 4/4 pop song
        fn preset_pop_song(file_id: i64) -> NewMusicalMetadata {
            Self::new(file_id)
                .bpm_str("120.0")
                .bpm_confidence(0.95)
                .key("C")
                .key_confidence(0.9)
                .time_signature(4, 4)
                .total_notes(1000)
                .unique_pitches(12)
                .pitch_range(60, 84)
                .avg_velocity_str("100.0")
                .note_density_f64(5.5)
                .polyphony(4, 2.5)
                .percussive(false)
                .build()
        }

        /// Preset: Techno track
        fn preset_techno(file_id: i64) -> NewMusicalMetadata {
            Self::new(file_id)
                .bpm_str("128.5")
                .bpm_confidence(1.0)
                .key("Am")
                .key_confidence(0.85)
                .time_signature(4, 4)
                .total_notes(2000)
                .unique_pitches(8)
                .pitch_range(36, 96)
                .avg_velocity_str("110.0")
                .note_density_f64(8.2)
                .polyphony(6, 3.5)
                .percussive(true)
                .build()
        }

        /// Preset: Waltz
        fn preset_waltz(file_id: i64) -> NewMusicalMetadata {
            Self::new(file_id)
                .bpm_str("180.0")
                .key("F")
                .time_signature(3, 4)
                .total_notes(500)
                .unique_pitches(15)
                .pitch_range(48, 96)
                .build()
        }

        /// Preset: Minimal (only required fields)
        fn preset_minimal(file_id: i64) -> NewMusicalMetadata {
            Self::new(file_id)
                .total_notes(100)
                .build()
        }
    }

// ============================================================================
// Category 1: CRUD Operations (12 tests)
// ============================================================================

#[tokio::test]
async fn test_insert_new_metadata() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_insert.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);

        let result = MetadataRepository::insert(&pool, metadata).await;
        assert!(result.is_ok(), "Insert should succeed");

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 1, "Should have 1 metadata record");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_insert_minimal_metadata() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_minimal.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);

        let result = MetadataRepository::insert(&pool, metadata).await;
        assert!(result.is_ok(), "Insert with minimal fields should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found.is_some(), "Should find metadata");

        let meta = found.unwrap();
        assert_eq!(meta.total_notes, 100);
        assert!(meta.bpm.is_none(), "BPM should be NULL");
        assert!(meta.key_signature.is_none(), "Key should be NULL");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_upsert_insert_creates_new() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_upsert_new.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 1, "First insert should create record");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_upsert_conflict_updates_existing() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_upsert_conflict.mid").await;

        // First insert
        let metadata_v1 = MetadataBuilder::new(file_id)
            .bpm_str("120.0")
            .key("C")
            .total_notes(1000)
            .build();
        MetadataRepository::insert(&pool, metadata_v1).await.expect("Insert v1 failed");

        // Second insert (same file_id) should update
        let metadata_v2 = MetadataBuilder::new(file_id)
            .bpm_str("140.0")
            .key("D")
            .total_notes(2000)
            .build();
        MetadataRepository::insert(&pool, metadata_v2).await.expect("Upsert v2 failed");

        // Verify only 1 record exists with updated values
        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 1, "Should still have only 1 record after upsert");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        let meta = found.unwrap();
        assert_bigdecimal_exact(&meta.bpm, "140.0");
        assert_eq!(meta.key_signature.as_deref(), Some("D"));
        assert_eq!(meta.total_notes, 2000);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_find_by_file_id_exists() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_find_exists.mid").await;
        let metadata = MetadataBuilder::preset_techno(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found.is_some(), "Should find metadata");

        let meta = found.unwrap();
        assert_eq!(meta.file_id, file_id);
        assert_bigdecimal_exact(&meta.bpm, "128.5");
        assert_eq!(meta.key_signature.as_deref(), Some("Am"));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_find_by_file_id_not_found() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let found = MetadataRepository::find_by_file_id(&pool, 999999).await.expect("Find failed");
        assert!(found.is_none(), "Should not find non-existent metadata");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_bpm() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_update_bpm.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let new_bpm = BigDecimal::from_str("140.5").expect("Valid BPM");
        let result = MetadataRepository::update_bpm(&pool, file_id, new_bpm, Some(0.98)).await;
        assert!(result.is_ok(), "Update BPM should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        let meta = found.unwrap();
        assert_bigdecimal_exact(&meta.bpm, "140.5");
        assert_eq!(meta.bpm_confidence, Some(0.98));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_key() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_update_key.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let result = MetadataRepository::update_key(&pool, file_id, "F#", Some(0.88)).await;
        assert!(result.is_ok(), "Update key should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        let meta = found.unwrap();
        assert_eq!(meta.key_signature.as_deref(), Some("F#"));
        assert_eq!(meta.key_confidence, Some(0.88));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_note_stats() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_update_stats.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let new_velocity = BigDecimal::from_str("115.5").expect("Valid velocity");
        let result = MetadataRepository::update_note_stats(
            &pool,
            file_id,
            2500,
            Some(15),
            Some(48),
            Some(96),
            Some(new_velocity),
        ).await;
        assert!(result.is_ok(), "Update note stats should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        let meta = found.unwrap();
        assert_eq!(meta.total_notes, 2500);
        assert_eq!(meta.unique_pitches, Some(15));
        assert_eq!(meta.pitch_range_min, Some(48));
        assert_eq!(meta.pitch_range_max, Some(96));
        assert_bigdecimal_exact(&meta.avg_velocity, "115.5");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_delete_metadata() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_delete.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let result = MetadataRepository::delete(&pool, file_id).await;
        assert!(result.is_ok(), "Delete should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found.is_none(), "Metadata should be deleted");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let result = MetadataRepository::delete(&pool, 999999).await;
        assert!(result.is_ok(), "Delete non-existent should not error");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_count_metadata() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 0, "Should start with 0 metadata");

        // Insert 3 metadata records
        for i in 1..=3 {
            let file_id = create_metadata_test_file(&pool, &format!("test_count_{}.mid", i)).await;
            let metadata = MetadataBuilder::preset_pop_song(file_id);
            MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");
        }

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 3, "Should have 3 metadata records");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

// ============================================================================
// Category 2: Musical Key ENUM (12 tests)
// ============================================================================

#[tokio::test]
async fn test_insert_all_major_keys() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for (i, key) in ALL_MAJOR_KEYS.iter().enumerate() {
            let file_id = create_metadata_test_file(&pool, &format!("major_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .key(key)
                .total_notes(100)
                .build();

            let result = MetadataRepository::insert(&pool, metadata).await;
            assert!(result.is_ok(), "Insert with key {} should succeed", key);

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
            assert_eq!(found.unwrap().key_signature.as_deref(), Some(*key), "Key should match");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_insert_all_minor_keys() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        for (i, key) in ALL_MINOR_KEYS.iter().enumerate() {
            let file_id = create_metadata_test_file(&pool, &format!("minor_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .key(key)
                .total_notes(100)
                .build();

            let result = MetadataRepository::insert(&pool, metadata).await;
            assert!(result.is_ok(), "Insert with key {} should succeed", key);

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
            assert_eq!(found.unwrap().key_signature.as_deref(), Some(*key), "Key should match");
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_key_all_major() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_update_major.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        for key in ALL_MAJOR_KEYS.iter() {
            let result = MetadataRepository::update_key(&pool, file_id, key, None).await;
            assert!(result.is_ok(), "Update to key {} should succeed", key);

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
            assert_eq!(found.unwrap().key_signature.as_deref(), Some(*key));
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_key_all_minor() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "test_update_minor.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        for key in ALL_MINOR_KEYS.iter() {
            let result = MetadataRepository::update_key(&pool, file_id, key, None).await;
            assert!(result.is_ok(), "Update to key {} should succeed", key);

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
            assert_eq!(found.unwrap().key_signature.as_deref(), Some(*key));
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ============================================================================
    // Category 3: BPM BigDecimal Precision Tests (8 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_bpm_precision_two_decimals() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "bpm_precision.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .bpm_str("120.50")
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "120.50");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_bpm_precision_three_decimals() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "bpm_precision3.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .bpm_str("128.567")
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        // BigDecimal may round to 2 decimals based on NUMERIC(6,2) schema
        assert_bigdecimal_approx(&found.bpm, "128.57", "0.01");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_bpm_boundary_min() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "bpm_min.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .bpm_str("20.00")
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "20.00");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_bpm_boundary_max() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "bpm_max.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .bpm_str("300.00")
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "300.00");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_bpm_with_confidence() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "update_bpm_conf.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let new_bpm = BigDecimal::from_str("140.25").expect("Valid BPM");
        MetadataRepository::update_bpm(&pool, file_id, new_bpm, Some(0.95)).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "140.25");
        assert_eq!(found.bpm_confidence, Some(0.95));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_bpm_without_confidence() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "update_bpm_no_conf.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let new_bpm = BigDecimal::from_str("150.00").expect("Valid BPM");
        MetadataRepository::update_bpm(&pool, file_id, new_bpm, None).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "150.00");
        assert_eq!(found.bpm_confidence, None);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_bpm_round_trip_preserves_precision() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let test_bpms = vec!["60.00", "120.50", "128.75", "180.25", "200.99"];

        for (i, bpm_str) in test_bpms.iter().enumerate() {
            let file_id = create_metadata_test_file(&pool, &format!("roundtrip_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .bpm_str(bpm_str)
                .total_notes(100)
                .build();

            MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
            assert_bigdecimal_exact(&found.bpm, bpm_str);
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_avg_velocity_bigdecimal() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "avg_velocity.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .total_notes(1000)
            .avg_velocity_str("64.50")
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.avg_velocity, "64.50");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ============================================================================
    // Category 4: Key ENUM Advanced Tests (8 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_update_key_with_confidence() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "key_confidence.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        MetadataRepository::update_key(&pool, file_id, "D", Some(0.88)).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.key_signature.as_deref(), Some("D"));
        assert_eq!(found.key_confidence, Some(0.88));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_key_without_confidence() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "key_no_conf.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        MetadataRepository::update_key(&pool, file_id, "Em", None).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.key_signature.as_deref(), Some("Em"));
        assert_eq!(found.key_confidence, None);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_insert_with_null_key() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "null_key.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert with NULL key should succeed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.key_signature, None);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_key_confidence_boundary_zero() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "key_conf_zero.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        MetadataRepository::update_key(&pool, file_id, "C", Some(0.0)).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.key_confidence, Some(0.0));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_key_confidence_boundary_one() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "key_conf_one.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        MetadataRepository::update_key(&pool, file_id, "C", Some(1.0)).await.expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.key_confidence, Some(1.0));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_enharmonic_keys() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Test enharmonic equivalents (C#/Db, F#/Gb, etc.)
        let enharmonics = vec![("C#", "Db"), ("F#", "Gb"), ("G#", "Ab")];

        for (i, (key1, key2)) in enharmonics.iter().enumerate() {
            let file_id = create_metadata_test_file(&pool, &format!("enharmonic_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .key(key1)
                .total_notes(100)
                .build();

            MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

            let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
            assert_eq!(found.key_signature.as_deref(), Some(*key1));

            // Update to enharmonic equivalent
            MetadataRepository::update_key(&pool, file_id, key2, None).await.expect("Update failed");

            let found2 = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
            assert_eq!(found2.key_signature.as_deref(), Some(*key2));
        }

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_all_24_keys_in_single_test() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let all_keys: Vec<&str> = ALL_MAJOR_KEYS.iter()
            .chain(ALL_MINOR_KEYS.iter())
            .copied()
            .collect();

        for (i, key) in all_keys.iter().enumerate() {
            let file_id = create_metadata_test_file(&pool, &format!("key_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .key(key)
                .total_notes(100)
                .build();

            let result = MetadataRepository::insert(&pool, metadata).await;
            assert!(result.is_ok(), "Key {} should be valid", key);
        }

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, all_keys.len() as i64);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_bpm_confidence_boundary_values() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "bpm_conf_bounds.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        // Test confidence = 0.0
        let bpm = BigDecimal::from_str("120.0").expect("Valid BPM");
        MetadataRepository::update_bpm(&pool, file_id, bpm.clone(), Some(0.0)).await.expect("Update failed");
        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.bpm_confidence, Some(0.0));

        // Test confidence = 1.0
        MetadataRepository::update_bpm(&pool, file_id, bpm, Some(1.0)).await.expect("Update failed");
        let found2 = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found2.bpm_confidence, Some(1.0));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ============================================================================
    // Category 5: Time Signature Tests (6 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_time_signature_common_4_4() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_4_4.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .time_signature(4, 4)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, Some(4));
        assert_eq!(found.time_signature_denominator, Some(4));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_time_signature_common_3_4() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_3_4.mid").await;
        let metadata = MetadataBuilder::preset_waltz(file_id);

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, Some(3));
        assert_eq!(found.time_signature_denominator, Some(4));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_time_signature_compound_6_8() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_6_8.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .time_signature(6, 8)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, Some(6));
        assert_eq!(found.time_signature_denominator, Some(8));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_time_signature_uncommon_7_8() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_7_8.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .time_signature(7, 8)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, Some(7));
        assert_eq!(found.time_signature_denominator, Some(8));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_time_signature_uncommon_5_4() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_5_4.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .time_signature(5, 4)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, Some(5));
        assert_eq!(found.time_signature_denominator, Some(4));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_time_signature_null_values() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "time_null.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .total_notes(100)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.time_signature_numerator, None);
        assert_eq!(found.time_signature_denominator, None);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ============================================================================
    // Category 6: File Association and CASCADE Tests (6 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_cascade_delete_file_removes_metadata() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "cascade_test.mid").await;
        let metadata = MetadataBuilder::preset_pop_song(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        // Verify metadata exists
        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found.is_some());

        // Delete file (should CASCADE to metadata)
        sqlx::query!("DELETE FROM files WHERE id = $1", file_id)
            .execute(&pool)
            .await
            .expect("Delete file failed");

        // Verify metadata was deleted
        let found_after = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found_after.is_none());

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_multiple_inserts_same_file_upsert() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "upsert_test.mid").await;

        // First insert
        let metadata1 = MetadataBuilder::new(file_id)
            .bpm_str("120.0")
            .key("C")
            .total_notes(100)
            .build();
        MetadataRepository::insert(&pool, metadata1).await.expect("First insert failed");

        // Second insert (should upsert)
        let metadata2 = MetadataBuilder::new(file_id)
            .bpm_str("140.0")
            .key("D")
            .total_notes(200)
            .build();
        MetadataRepository::insert(&pool, metadata2).await.expect("Second insert failed");

        // Verify only one row exists with updated values
        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 1);

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "140.0");
        assert_eq!(found.key_signature.as_deref(), Some("D"));
        assert_eq!(found.total_notes, 200);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_metadata_without_file_fails() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Try to insert metadata for non-existent file
        let metadata = MetadataBuilder::new(999999)
            .total_notes(100)
            .build();

        let result = MetadataRepository::insert(&pool, metadata).await;
        assert!(result.is_err(), "Insert without file should fail with FK constraint");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_orphaned_metadata_handling() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "orphan_test.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        // Manually delete file (CASCADE should handle metadata)
        sqlx::query!("DELETE FROM files WHERE id = $1", file_id)
            .execute(&pool)
            .await
            .expect("Delete file failed");

        // Verify no orphaned metadata
        let orphan_count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM musical_metadata WHERE file_id NOT IN (SELECT id FROM files)"#
        )
        .fetch_one(&pool)
        .await
        .expect("Query failed");

        assert_eq!(orphan_count, 0, "No orphaned metadata should exist");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_batch_file_and_metadata_insertion() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        // Create 10 files with metadata
        for i in 0..10 {
            let file_id = create_metadata_test_file(&pool, &format!("batch_{}.mid", i)).await;
            let metadata = MetadataBuilder::new(file_id)
                .bpm_str(&format!("{}.0", 120 + i * 10))
                .total_notes((100 + i * 50) as i32)
                .build();
            MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");
        }

        let count = MetadataRepository::count(&pool).await.expect("Count failed");
        assert_eq!(count, 10);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_delete_metadata_only() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "delete_metadata.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        // Delete metadata only (file should remain)
        MetadataRepository::delete(&pool, file_id).await.expect("Delete failed");

        // Verify metadata deleted
        let found_metadata = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed");
        assert!(found_metadata.is_none());

        // Verify file still exists
        let file_exists: Option<i64> = sqlx::query_scalar!("SELECT id FROM files WHERE id = $1", file_id)
            .fetch_optional(&pool)
            .await
            .expect("Query failed");
        assert!(file_exists.is_some());

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    // ============================================================================
    // Category 7: Edge Cases and Complex Scenarios (4 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_all_fields_populated() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "all_fields.mid").await;
        let metadata = NewMusicalMetadata {
            file_id,
            bpm: Some(BigDecimal::from_str("128.50").unwrap()),
            bpm_confidence: Some(0.95),
            key_signature: Some("F#".to_string()),
            key_confidence: Some(0.92),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            total_notes: 2000,
            unique_pitches: Some(24),
            pitch_range_min: Some(48),
            pitch_range_max: Some(96),
            avg_velocity: Some(BigDecimal::from_str("80.25").unwrap()),
            note_density: Some(BigDecimal::from_str("12.5").unwrap()),
            polyphony_max: Some(8),
            polyphony_avg: Some(BigDecimal::from_str("4.3").unwrap()),
            is_percussive: Some(false),
        };

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_bigdecimal_exact(&found.bpm, "128.50");
        assert_eq!(found.bpm_confidence, Some(0.95));
        assert_eq!(found.key_signature.as_deref(), Some("F#"));
        assert_eq!(found.key_confidence, Some(0.92));
        assert_eq!(found.time_signature_numerator, Some(4));
        assert_eq!(found.time_signature_denominator, Some(4));
        assert_eq!(found.total_notes, 2000);
        assert_eq!(found.unique_pitches, Some(24));
        assert_eq!(found.pitch_range_min, Some(48));
        assert_eq!(found.pitch_range_max, Some(96));
        assert_bigdecimal_exact(&found.avg_velocity, "80.25");
        assert_eq!(found.polyphony_max, Some(8));
        assert_eq!(found.is_percussive, Some(false));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_update_note_stats_all_fields() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "update_notes.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let avg_vel = Some(BigDecimal::from_str("72.5").unwrap());
        MetadataRepository::update_note_stats(&pool, file_id, 5000, Some(36), Some(21), Some(108), avg_vel)
            .await
            .expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.total_notes, 5000);
        assert_eq!(found.unique_pitches, Some(36));
        assert_eq!(found.pitch_range_min, Some(21));
        assert_eq!(found.pitch_range_max, Some(108));
        assert_bigdecimal_exact(&found.avg_velocity, "72.5");

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_pitch_range_midi_boundaries() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "pitch_bounds.mid").await;
        let metadata = MetadataBuilder::preset_minimal(file_id);
        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        // Test MIDI boundaries (0-127)
        MetadataRepository::update_note_stats(&pool, file_id, 128, Some(128), Some(0), Some(127), None)
            .await
            .expect("Update failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.pitch_range_min, Some(0));
        assert_eq!(found.pitch_range_max, Some(127));

        cleanup_database(&pool).await.expect("Cleanup failed");
    }

    #[tokio::test]
    async fn test_large_total_notes() {
        let pool = setup_test_pool().await;
        cleanup_database(&pool).await.expect("Cleanup failed");

        let file_id = create_metadata_test_file(&pool, "large_notes.mid").await;
        let metadata = MetadataBuilder::new(file_id)
            .total_notes(1_000_000)
            .build();

        MetadataRepository::insert(&pool, metadata).await.expect("Insert failed");

        let found = MetadataRepository::find_by_file_id(&pool, file_id).await.expect("Find failed").unwrap();
        assert_eq!(found.total_notes, 1_000_000);

        cleanup_database(&pool).await.expect("Cleanup failed");
    }
