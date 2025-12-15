//! Metadata repository for musical metadata database operations.
//!
//! Provides CRUD operations and queries for the musical_metadata table.

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use tracing::info;

#[cfg(feature = "database")]
use crate::db::models::{CreateMidiMetadata, DbError, DbResult, MidiMetadata, UpdateMidiMetadata};

/// Repository for musical metadata operations.
#[derive(Clone)]
pub struct MetadataRepository {
    #[cfg(feature = "database")]
    pool: PgPool,
}

#[cfg(feature = "database")]
impl MetadataRepository {
    /// Create new repository with connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert metadata for a file (upsert based on file_id).
    pub async fn insert(&self, metadata: CreateMidiMetadata) -> DbResult<i64> {
        let file_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO musical_metadata (
                file_id, bpm, bpm_confidence, has_tempo_changes, tempo_changes,
                key_signature, key_confidence,
                time_signature_numerator, time_signature_denominator,
                total_notes, unique_pitches, pitch_range_min, pitch_range_max,
                avg_velocity, note_density, polyphony_max, polyphony_avg,
                is_monophonic, is_polyphonic, is_percussive
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                has_tempo_changes = EXCLUDED.has_tempo_changes,
                tempo_changes = EXCLUDED.tempo_changes,
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
                is_monophonic = EXCLUDED.is_monophonic,
                is_polyphonic = EXCLUDED.is_polyphonic,
                is_percussive = EXCLUDED.is_percussive
            RETURNING file_id
            "#,
        )
        .bind(metadata.file_id)
        .bind(metadata.bpm)
        .bind(metadata.bpm_confidence)
        .bind(metadata.has_tempo_changes)
        .bind(&metadata.tempo_changes)
        .bind(&metadata.key_signature)
        .bind(metadata.key_confidence)
        .bind(metadata.time_signature_numerator)
        .bind(metadata.time_signature_denominator)
        .bind(metadata.total_notes)
        .bind(metadata.unique_pitches)
        .bind(metadata.pitch_range_min)
        .bind(metadata.pitch_range_max)
        .bind(metadata.avg_velocity)
        .bind(metadata.note_density)
        .bind(metadata.polyphony_max)
        .bind(metadata.polyphony_avg)
        .bind(metadata.is_monophonic)
        .bind(metadata.is_polyphonic)
        .bind(metadata.is_percussive)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        info!("Inserted metadata for file {}", file_id);
        Ok(file_id)
    }

    /// Get metadata by file ID.
    pub async fn find_by_file_id(&self, file_id: i64) -> DbResult<Option<MidiMetadata>> {
        sqlx::query_as::<_, MidiMetadata>(
            r#"
            SELECT * FROM musical_metadata
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Update metadata.
    pub async fn update(&self, file_id: i64, update: UpdateMidiMetadata) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE musical_metadata
            SET
                bpm = COALESCE($2, bpm),
                bpm_confidence = COALESCE($3, bpm_confidence),
                key_signature = COALESCE($4, key_signature),
                key_confidence = COALESCE($5, key_confidence),
                has_chords = COALESCE($6, has_chords),
                chord_complexity = COALESCE($7, chord_complexity),
                has_melody = COALESCE($8, has_melody),
                melodic_range = COALESCE($9, melodic_range)
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .bind(update.bpm)
        .bind(update.bpm_confidence)
        .bind(&update.key_signature)
        .bind(update.key_confidence)
        .bind(update.has_chords)
        .bind(update.chord_complexity)
        .bind(update.has_melody)
        .bind(update.melodic_range)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Delete metadata by file ID.
    pub async fn delete(&self, file_id: i64) -> DbResult<bool> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM musical_metadata
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    /// Get files by BPM range.
    pub async fn find_by_bpm_range(
        &self,
        min_bpm: f64,
        max_bpm: f64,
        limit: i64,
    ) -> DbResult<Vec<MidiMetadata>> {
        sqlx::query_as::<_, MidiMetadata>(
            r#"
            SELECT * FROM musical_metadata
            WHERE bpm BETWEEN $1 AND $2
            ORDER BY bpm
            LIMIT $3
            "#,
        )
        .bind(rust_decimal::Decimal::try_from(min_bpm).unwrap_or_default())
        .bind(rust_decimal::Decimal::try_from(max_bpm).unwrap_or_default())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get files by key signature.
    pub async fn find_by_key(&self, key: &str, limit: i64) -> DbResult<Vec<MidiMetadata>> {
        sqlx::query_as::<_, MidiMetadata>(
            r#"
            SELECT * FROM musical_metadata
            WHERE key_signature = $1
            ORDER BY bpm
            LIMIT $2
            "#,
        )
        .bind(key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Get files by time signature.
    pub async fn find_by_time_signature(
        &self,
        numerator: i16,
        denominator: i16,
        limit: i64,
    ) -> DbResult<Vec<MidiMetadata>> {
        sqlx::query_as::<_, MidiMetadata>(
            r#"
            SELECT * FROM musical_metadata
            WHERE time_signature_numerator = $1 AND time_signature_denominator = $2
            ORDER BY bpm
            LIMIT $3
            "#,
        )
        .bind(numerator)
        .bind(denominator)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }

    /// Count total metadata records.
    pub async fn count(&self) -> DbResult<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM musical_metadata
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(count.unwrap_or(0))
    }

    /// Get metadata statistics.
    pub async fn get_statistics(&self) -> DbResult<MetadataStatistics> {
        let stats = sqlx::query_as::<_, MetadataStatistics>(
            r#"
            SELECT
                COUNT(*) as total_files,
                AVG(bpm::float8) as avg_bpm,
                MIN(bpm::float8) as min_bpm,
                MAX(bpm::float8) as max_bpm,
                AVG(total_notes::float8) as avg_notes
            FROM musical_metadata
            WHERE bpm IS NOT NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(stats)
    }

    /// Batch insert metadata records.
    pub async fn batch_insert(&self, records: Vec<CreateMidiMetadata>) -> DbResult<Vec<i64>> {
        let mut tx = self.pool.begin().await.map_err(DbError::from)?;
        let mut inserted_ids = Vec::with_capacity(records.len());

        for metadata in records {
            let file_id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO musical_metadata (
                    file_id, bpm, bpm_confidence, has_tempo_changes, tempo_changes,
                    key_signature, key_confidence,
                    time_signature_numerator, time_signature_denominator,
                    total_notes, unique_pitches, pitch_range_min, pitch_range_max,
                    avg_velocity, note_density, polyphony_max, polyphony_avg,
                    is_monophonic, is_polyphonic, is_percussive
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                ON CONFLICT (file_id) DO UPDATE SET
                    bpm = EXCLUDED.bpm,
                    bpm_confidence = EXCLUDED.bpm_confidence,
                    has_tempo_changes = EXCLUDED.has_tempo_changes,
                    tempo_changes = EXCLUDED.tempo_changes,
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
                    is_monophonic = EXCLUDED.is_monophonic,
                    is_polyphonic = EXCLUDED.is_polyphonic,
                    is_percussive = EXCLUDED.is_percussive
                RETURNING file_id
                "#,
            )
            .bind(metadata.file_id)
            .bind(metadata.bpm)
            .bind(metadata.bpm_confidence)
            .bind(metadata.has_tempo_changes)
            .bind(&metadata.tempo_changes)
            .bind(&metadata.key_signature)
            .bind(metadata.key_confidence)
            .bind(metadata.time_signature_numerator)
            .bind(metadata.time_signature_denominator)
            .bind(metadata.total_notes)
            .bind(metadata.unique_pitches)
            .bind(metadata.pitch_range_min)
            .bind(metadata.pitch_range_max)
            .bind(metadata.avg_velocity)
            .bind(metadata.note_density)
            .bind(metadata.polyphony_max)
            .bind(metadata.polyphony_avg)
            .bind(metadata.is_monophonic)
            .bind(metadata.is_polyphonic)
            .bind(metadata.is_percussive)
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::from)?;

            inserted_ids.push(file_id);
        }

        tx.commit().await.map_err(DbError::from)?;
        info!("Batch inserted {} metadata records", inserted_ids.len());

        Ok(inserted_ids)
    }

    /// Get files without metadata.
    pub async fn get_files_without_metadata(&self, limit: i64) -> DbResult<Vec<i64>> {
        let file_ids: Vec<i64> = sqlx::query_scalar(
            r#"
            SELECT f.id FROM files f
            LEFT JOIN musical_metadata m ON f.id = m.file_id
            WHERE m.file_id IS NULL
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(file_ids)
    }

    /// Get drum/percussion files.
    pub async fn find_percussive(&self, limit: i64) -> DbResult<Vec<MidiMetadata>> {
        sqlx::query_as::<_, MidiMetadata>(
            r#"
            SELECT * FROM musical_metadata
            WHERE is_percussive = true
            ORDER BY bpm
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)
    }
}

/// Statistics about metadata in the database.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MetadataStatistics {
    pub total_files: Option<i64>,
    pub avg_bpm: Option<f64>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub avg_notes: Option<f64>,
}

// Stub implementation when database feature is not enabled
#[cfg(not(feature = "database"))]
impl MetadataRepository {
    /// Placeholder - database feature not enabled.
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(all(test, feature = "database"))]
mod tests {
    #[test]
    fn test_repository_creation() {
        // Compile-time check
    }
}
