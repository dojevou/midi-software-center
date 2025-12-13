/// Musical metadata repository
/// Aligned with actual schema from 001_initial_schema.sql
use crate::db::models::{MusicalMetadata, NewMusicalMetadata};
use sqlx::PgPool;

pub struct MetadataRepository;

impl MetadataRepository {
    /// Inserts musical metadata
    pub async fn insert(pool: &PgPool, metadata: NewMusicalMetadata) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO musical_metadata (
                file_id,
                bpm,
                bpm_confidence,
                key_signature,
                key_confidence,
                time_signature_numerator,
                time_signature_denominator,
                total_notes,
                unique_pitches,
                pitch_range_min,
                pitch_range_max,
                avg_velocity,
                note_density,
                polyphony_max,
                polyphony_avg,
                is_percussive,
                chord_progression,
                chord_types,
                has_seventh_chords,
                has_extended_chords,
                chord_change_rate,
                chord_complexity_score
            ) VALUES (
                $1, $2, $3, $4::text::musical_key, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22
            )
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                key_signature = EXCLUDED.key_signature,
                key_confidence = EXCLUDED.key_confidence,
                time_signature_numerator = EXCLUDED.time_signature_numerator,
                time_signature_denominator = EXCLUDED.time_signature_denominator,
                total_notes = EXCLUDED.total_notes,
                unique_pitches = EXCLUDED.unique_pitches,
                pitch_range_min = EXCLUDED.pitch_range_min,
                pitch_range_max = EXCLUDED.pitch_range_max,
                avg_velocity = EXCLUDED.avg_velocity,
                note_density = EXCLUDED.note_density,
                polyphony_max = EXCLUDED.polyphony_max,
                polyphony_avg = EXCLUDED.polyphony_avg,
                is_percussive = EXCLUDED.is_percussive,
                chord_progression = EXCLUDED.chord_progression,
                chord_types = EXCLUDED.chord_types,
                has_seventh_chords = EXCLUDED.has_seventh_chords,
                has_extended_chords = EXCLUDED.has_extended_chords,
                chord_change_rate = EXCLUDED.chord_change_rate,
                chord_complexity_score = EXCLUDED.chord_complexity_score
            "#,
            metadata.file_id,
            metadata.bpm,
            metadata.bpm_confidence,
            metadata.key_signature,
            metadata.key_confidence,
            metadata.time_signature_numerator,
            metadata.time_signature_denominator,
            metadata.total_notes,
            metadata.unique_pitches,
            metadata.pitch_range_min,
            metadata.pitch_range_max,
            metadata.avg_velocity,
            metadata.note_density,
            metadata.polyphony_max,
            metadata.polyphony_avg,
            metadata.is_percussive,
            metadata.chord_progression,
            metadata.chord_types.as_deref(),
            metadata.has_seventh_chords,
            metadata.has_extended_chords,
            metadata.chord_change_rate,
            metadata.chord_complexity_score,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Finds metadata by file ID
    pub async fn find_by_file_id(
        pool: &PgPool,
        file_id: i64,
    ) -> Result<Option<MusicalMetadata>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            MusicalMetadata,
            r#"
            SELECT
                file_id,
                bpm,
                bpm_confidence,
                has_tempo_changes,
                tempo_changes,
                key_signature::text as key_signature,
                key_confidence,
                has_key_changes,
                key_changes,
                time_signature_numerator,
                time_signature_denominator,
                has_time_signature_changes,
                time_signature_changes,
                total_notes,
                unique_pitches,
                pitch_range_min,
                pitch_range_max,
                avg_velocity,
                note_density,
                polyphony_max,
                polyphony_avg,
                is_monophonic,
                is_polyphonic,
                is_percussive,
                has_chords,
                chord_complexity,
                has_melody,
                melodic_range,
                created_at as "created_at!",
                chord_progression,
                chord_types,
                has_seventh_chords,
                has_extended_chords,
                chord_change_rate,
                chord_complexity_score
            FROM musical_metadata WHERE file_id = $1
            "#,
            file_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    /// Updates BPM and confidence
    pub async fn update_bpm(
        pool: &PgPool,
        file_id: i64,
        bpm: sqlx::types::BigDecimal,
        confidence: Option<f32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE musical_metadata
            SET bpm = $1,
                bpm_confidence = $2
            WHERE file_id = $3
            "#,
            bpm,
            confidence,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Updates key and confidence
    pub async fn update_key(
        pool: &PgPool,
        file_id: i64,
        key: &str,
        confidence: Option<f32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE musical_metadata
            SET key_signature = $1::text::musical_key,
                key_confidence = $2
            WHERE file_id = $3
            "#,
            key,
            confidence,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Updates note statistics
    pub async fn update_note_stats(
        pool: &PgPool,
        file_id: i64,
        total_notes: i32,
        unique_pitches: Option<i32>,
        pitch_range_min: Option<i16>,
        pitch_range_max: Option<i16>,
        avg_velocity: Option<sqlx::types::BigDecimal>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE musical_metadata
            SET total_notes = $1,
                unique_pitches = $2,
                pitch_range_min = $3,
                pitch_range_max = $4,
                avg_velocity = $5
            WHERE file_id = $6
            "#,
            total_notes,
            unique_pitches,
            pitch_range_min,
            pitch_range_max,
            avg_velocity,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Updates harmonic analysis data
    #[allow(clippy::too_many_arguments)]
    pub async fn update_chords(
        pool: &PgPool,
        file_id: i64,
        chord_progression: Option<serde_json::Value>,
        chord_types: Option<Vec<String>>,
        has_seventh_chords: Option<bool>,
        has_extended_chords: Option<bool>,
        chord_change_rate: Option<sqlx::types::BigDecimal>,
        chord_complexity_score: Option<sqlx::types::BigDecimal>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE musical_metadata
            SET chord_progression = $1,
                chord_types = $2,
                has_seventh_chords = $3,
                has_extended_chords = $4,
                chord_change_rate = $5,
                chord_complexity_score = $6
            WHERE file_id = $7
            "#,
            chord_progression,
            chord_types.as_deref(),
            has_seventh_chords,
            has_extended_chords,
            chord_change_rate,
            chord_complexity_score,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deletes metadata by file ID
    pub async fn delete(pool: &PgPool, file_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM musical_metadata WHERE file_id = $1", file_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Gets metadata count
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM musical_metadata"#)
            .fetch_one(pool)
            .await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    #[ignore] // Only run when database is available
    async fn test_insert_and_find() {
        let pool = setup_test_pool().await;

        let metadata = NewMusicalMetadata {
            file_id: 1,
            bpm: sqlx::types::BigDecimal::from_f64(120.0),
            bpm_confidence: Some(0.95),
            key_signature: Some("C".to_string()),
            key_confidence: Some(0.9),
            time_signature_numerator: Some(4),
            time_signature_denominator: Some(4),
            total_notes: 1000,
            unique_pitches: Some(12),
            pitch_range_min: Some(60),
            pitch_range_max: Some(84),
            avg_velocity: sqlx::types::BigDecimal::from_f64(100.0),
            note_density: sqlx::types::BigDecimal::from_f64(5.5),
            polyphony_max: Some(4),
            polyphony_avg: sqlx::types::BigDecimal::from_f64(2.5),
            is_percussive: Some(false),
            chord_progression: None,
            chord_types: None,
            has_seventh_chords: None,
            has_extended_chords: None,
            chord_change_rate: None,
            chord_complexity_score: None,
        };

        MetadataRepository::insert(&pool, metadata).await.unwrap();

        let found = MetadataRepository::find_by_file_id(&pool, 1).await.unwrap();
        assert!(found.is_some());

        let meta = found.unwrap();
        assert_eq!(meta.file_id, 1);
        assert_eq!(meta.total_notes, 1000);
    }
}
