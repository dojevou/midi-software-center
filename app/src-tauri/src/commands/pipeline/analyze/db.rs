//! Database operations for analysis results

use super::types::AnalyzedFile;

/// Batch insert analyzed files into musical_metadata and update files.analyzed_at
pub async fn batch_insert_analyzed_files(
    files: &[AnalyzedFile],
    pool: &sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if files.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for file in files {
        sqlx::query(
            r#"
            INSERT INTO musical_metadata (
                file_id,
                bpm,
                bpm_confidence,
                has_tempo_changes,
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
                is_monophonic,
                is_polyphonic,
                is_percussive,
                has_chords,
                chord_progression,
                chord_types,
                has_seventh_chords,
                has_extended_chords,
                chord_change_rate,
                chord_complexity_score,
                has_melody,
                melodic_range,
                tempo_changes,
                key_changes,
                time_signature_changes,
                controller_data,
                articulation_data,
                structure_data
            ) VALUES ($1, $2, $3, $4, $5::musical_key, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21::jsonb, $22, $23, $24, $25, $26, $27, $28, $29::jsonb, $30::jsonb, $31::jsonb, $32::jsonb, $33::jsonb, $34::jsonb)
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                bpm_confidence = EXCLUDED.bpm_confidence,
                has_tempo_changes = EXCLUDED.has_tempo_changes,
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
                is_percussive = EXCLUDED.is_percussive,
                has_chords = EXCLUDED.has_chords,
                chord_progression = EXCLUDED.chord_progression,
                chord_types = EXCLUDED.chord_types,
                has_seventh_chords = EXCLUDED.has_seventh_chords,
                has_extended_chords = EXCLUDED.has_extended_chords,
                chord_change_rate = EXCLUDED.chord_change_rate,
                chord_complexity_score = EXCLUDED.chord_complexity_score,
                has_melody = EXCLUDED.has_melody,
                melodic_range = EXCLUDED.melodic_range,
                tempo_changes = EXCLUDED.tempo_changes,
                key_changes = EXCLUDED.key_changes,
                time_signature_changes = EXCLUDED.time_signature_changes,
                controller_data = EXCLUDED.controller_data,
                articulation_data = EXCLUDED.articulation_data,
                structure_data = EXCLUDED.structure_data
            "#,
        )
        .bind(file.file_id)
        .bind(file.tempo_bpm)
        .bind(file.bpm_confidence)
        .bind(file.has_tempo_variation)
        .bind(&file.key_signature)
        .bind(file.key_confidence)
        .bind(file.time_signature_num)
        .bind(file.time_signature_den)
        .bind(file.note_count)
        .bind(file.unique_pitches)
        .bind(file.pitch_range_low)
        .bind(file.pitch_range_high)
        .bind(file.avg_velocity)
        .bind(file.note_density)
        .bind(file.polyphony_max)
        .bind(file.polyphony_avg)
        .bind(file.is_monophonic)
        .bind(file.is_polyphonic)
        .bind(file.is_percussive)
        .bind(file.has_chords)
        .bind(file.chord_progression.as_ref().and_then(|v| serde_json::to_string(v).ok()))
        .bind(&file.chord_types)
        .bind(file.has_seventh_chords)
        .bind(file.has_extended_chords)
        .bind(file.chord_change_rate)
        .bind(file.chord_complexity_score)
        .bind(file.has_melody)
        .bind(file.melodic_range)
        .bind(&file.tempo_changes)
        .bind(&file.key_changes)
        .bind(&file.time_signature_changes)
        .bind(&file.controller_data)
        .bind(&file.articulation_data)
        .bind(&file.structure_data)
        .execute(&mut *tx)
        .await?;

        // Insert track instruments into file_instruments table
        for inst in &file.track_instruments {
            sqlx::query(
                r#"
                INSERT INTO file_instruments (
                    file_id, channel, program_number, program_name,
                    instrument_family, instrument_type, note_count,
                    is_primary, avg_velocity, pitch_range_low, pitch_range_high
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (file_id, channel, program_number) DO UPDATE SET
                    program_name = EXCLUDED.program_name,
                    instrument_family = EXCLUDED.instrument_family,
                    instrument_type = EXCLUDED.instrument_type,
                    note_count = EXCLUDED.note_count,
                    is_primary = EXCLUDED.is_primary,
                    avg_velocity = EXCLUDED.avg_velocity,
                    pitch_range_low = EXCLUDED.pitch_range_low,
                    pitch_range_high = EXCLUDED.pitch_range_high
                "#,
            )
            .bind(file.file_id)
            .bind(inst.channel)
            .bind(inst.program_number)
            .bind(&inst.program_name)
            .bind(&inst.instrument_family)
            .bind(&inst.instrument_type)
            .bind(inst.note_count)
            .bind(inst.is_primary)
            .bind(inst.avg_velocity)
            .bind(inst.pitch_range_low)
            .bind(inst.pitch_range_high)
            .execute(&mut *tx)
            .await?;
        }

        // Update files.analyzed_at timestamp
        sqlx::query("UPDATE files SET analyzed_at = NOW() WHERE id = $1")
            .bind(file.file_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}
