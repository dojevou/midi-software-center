-- Sample data for testing
-- Inserts 100 sample MIDI file records matching the actual schema

DO $$
DECLARE
    file_id BIGINT;
    batch_id UUID := uuid_generate_v4();
BEGIN
    FOR i IN 1..100 LOOP
        -- Insert file
        INSERT INTO files (
            original_path,
            current_path,
            original_filename,
            new_filename,
            content_hash,
            file_size,
            file_modified,
            manufacturer,
            collection_name,
            folder_tags,
            category,
            subcategory,
            auto_tags,
            user_tags,
            import_batch_id,
            analyzed_at
        ) VALUES (
            '/original/path/file' || i || '.mid',
            '/current/path/file' || i || '.mid',
            'Original_File_' || i || '.mid',
            CASE (i % 5)
                WHEN 0 THEN 'KICK_Hard_Punchy_' || i || '.mid'
                WHEN 1 THEN 'BASS_Cm_140BPM_Deep_' || i || '.mid'
                WHEN 2 THEN 'CHORD_Am_Progressive_' || i || '.mid'
                WHEN 3 THEN 'LEAD_G_Melodic_' || i || '.mid'
                ELSE 'MELODY_Em_Trance_' || i || '.mid'
            END,
            decode(md5('content' || i::text), 'hex'),
            1024 * (100 + (i % 500)),
            NOW() - (i || ' days')::interval,
            CASE (i % 3)
                WHEN 0 THEN 'DMS'
                WHEN 1 THEN 'Loopmasters'
                ELSE 'Vengeance'
            END,
            'Sample Collection Vol.' || ((i % 10) + 1),
            ARRAY['trance', 'electronic', 'sample'],
            CASE (i % 5)
                WHEN 0 THEN 'KICK'::file_category
                WHEN 1 THEN 'BASS'::file_category
                WHEN 2 THEN 'CHORD'::file_category
                WHEN 3 THEN 'LEAD'::file_category
                ELSE 'MELODY'::file_category
            END,
            CASE (i % 3)
                WHEN 0 THEN 'punchy'
                WHEN 1 THEN 'deep'
                ELSE 'melodic'
            END,
            ARRAY['sample', 'loop', 'midi'],
            ARRAY['favorite', 'project1'],
            batch_id,
            NOW() - (i || ' hours')::interval
        ) RETURNING id INTO file_id;

        -- Insert musical metadata
        INSERT INTO musical_metadata (
            file_id,
            midi_format,
            num_tracks,
            ticks_per_quarter_note,
            duration_seconds,
            total_ticks,
            detected_bpm,
            bpm_confidence,
            detected_key,
            key_confidence,
            time_signature_numerator,
            time_signature_denominator,
            total_notes,
            unique_pitches,
            pitch_range_low,
            pitch_range_high,
            avg_velocity,
            avg_note_duration_ms,
            complexity_score,
            polyphony_max,
            polyphony_avg
        ) VALUES (
            file_id,
            1,  -- MIDI format 1
            CASE (i % 3) WHEN 0 THEN 1 WHEN 1 THEN 4 ELSE 8 END,  -- num_tracks
            480,  -- ticks_per_quarter_note
            30.0 + (random() * 90),  -- duration_seconds
            14400 + (i * 100),  -- total_ticks
            120.0 + (i % 60),  -- detected_bpm
            0.85 + (random() * 0.14),  -- bpm_confidence
            CASE (i % 12)
                WHEN 0 THEN 'C'::musical_key
                WHEN 1 THEN 'Cm'::musical_key
                WHEN 2 THEN 'D'::musical_key
                WHEN 3 THEN 'Dm'::musical_key
                WHEN 4 THEN 'E'::musical_key
                WHEN 5 THEN 'Em'::musical_key
                WHEN 6 THEN 'F'::musical_key
                WHEN 7 THEN 'Fm'::musical_key
                WHEN 8 THEN 'G'::musical_key
                WHEN 9 THEN 'Gm'::musical_key
                WHEN 10 THEN 'A'::musical_key
                ELSE 'Am'::musical_key
            END,
            0.80 + (random() * 0.19),  -- key_confidence
            4,  -- time_signature_numerator
            4,  -- time_signature_denominator
            100 + (i % 500),  -- total_notes
            12 + (i % 13),  -- unique_pitches (1-24 semitones)
            36 + (i % 30),  -- pitch_range_low (MIDI note)
            72 + (i % 30),  -- pitch_range_high (MIDI note)
            64.0 + (i % 63),  -- avg_velocity
            250.0 + (random() * 500),  -- avg_note_duration_ms
            0.3 + (random() * 0.6),  -- complexity_score
            1 + (i % 8),  -- polyphony_max
            1.5 + (random() * 2.5)  -- polyphony_avg
        );

        -- Insert some tracks
        INSERT INTO tracks (
            file_id,
            track_number,
            track_name,
            is_drum_track,
            is_melodic,
            midi_channels,
            program_numbers,
            instrument_names,
            note_count,
            event_count
        ) VALUES (
            file_id,
            0,
            'Main Track',
            (i % 5 = 0),  -- Every 5th file is drums
            (i % 5 != 0),  -- Others are melodic
            ARRAY[0],
            ARRAY[i % 128],
            ARRAY['Acoustic Grand Piano'],
            100 + (i % 500),
            200 + (i % 1000)
        );

        -- Insert tags
        IF i <= 10 THEN
            INSERT INTO tags (name, category)
            VALUES ('tag_' || i, CASE (i % 3) WHEN 0 THEN 'genre' WHEN 1 THEN 'mood' ELSE 'style' END)
            ON CONFLICT (name) DO NOTHING;
        END IF;

        -- Link file to tags
        INSERT INTO file_tags (file_id, tag_id, is_auto_tag)
        SELECT file_id, id, true
        FROM tags
        WHERE name = 'tag_' || ((i % 10) + 1)
        LIMIT 1;

    END LOOP;

    RAISE NOTICE 'Inserted 100 sample records with batch_id: %', batch_id;
    RAISE NOTICE 'Files: 100, Musical metadata: 100, Tracks: 100, Tags: 10';
END $$;
