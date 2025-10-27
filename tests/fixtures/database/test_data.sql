-- =============================================================================
-- MIDI Software Center - Test Fixtures
-- =============================================================================
-- Purpose: Comprehensive test data for all 18 database tables
-- Coverage: All tables from migrations 001, 002, 003, and 006
-- Use: Testing, development, and integration tests
-- =============================================================================

BEGIN;

-- =============================================================================
-- SECTION 1: CORE FILE DATA (files table)
-- =============================================================================
-- Purpose: Sample MIDI files covering various scenarios
-- Coverage: Different BPMs, formats, sizes, multi-track scenarios
-- =============================================================================

-- Sample 1: Simple bass line (60 BPM, slow)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    1,
    'bass_slow_60bpm.mid',
    '/library/bass/bass_slow_60bpm.mid',
    'bass_slow_60bpm.mid',
    decode('a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456', 'hex'),
    1024,
    1, -- Format 1 (multi-track)
    2,
    480,
    32.0,
    61440,
    false,
    'bass',
    'Splice',
    'Bass Essentials',
    ARRAY['bass', 'slow', 'deep']
);

-- Sample 2: Drum loop (90 BPM, medium)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    2,
    'drum_loop_90bpm.mid',
    '/library/drums/drum_loop_90bpm.mid',
    'drum_loop_90bpm.mid',
    decode('b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567', 'hex'),
    2048,
    0, -- Format 0 (single track)
    1,
    480,
    8.0,
    15360,
    false,
    'drums',
    'Loopmasters',
    'Drum Loops Pro',
    ARRAY['drums', 'loop', 'percussion']
);

-- Sample 3: Piano melody (120 BPM, standard)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    3,
    'piano_melody_120bpm.mid',
    '/library/melody/piano_melody_120bpm.mid',
    'piano_melody_120bpm.mid',
    decode('c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678', 'hex'),
    3072,
    1,
    3,
    480,
    16.0,
    30720,
    false,
    'melody',
    'Native Instruments',
    'Piano Melodies',
    ARRAY['piano', 'melody', 'keys']
);

-- Sample 4: Upbeat house chord (140 BPM, fast)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    4,
    'house_chord_140bpm.mid',
    '/library/chords/house_chord_140bpm.mid',
    'house_chord_140bpm.mid',
    decode('d4e5f6789012345678901234567890abcdef1234567890abcdef123456789', 'hex'),
    1536,
    1,
    2,
    480,
    4.0,
    7680,
    false,
    'chords',
    'Vengeance',
    'House Essentials',
    ARRAY['chords', 'house', 'electronic']
);

-- Sample 5: DnB arp (174 BPM, very fast)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    5,
    'dnb_arp_174bpm.mid',
    '/library/arps/dnb_arp_174bpm.mid',
    'dnb_arp_174bpm.mid',
    decode('e5f6789012345678901234567890abcdef1234567890abcdef1234567890a', 'hex'),
    2560,
    1,
    2,
    480,
    8.0,
    15360,
    false,
    'arps',
    'Sample Magic',
    'DnB Collection',
    ARRAY['arp', 'dnb', 'fast']
);

-- Sample 6: Pad texture (80 BPM, atmospheric)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    6,
    'pad_ambient_80bpm.mid',
    '/library/pads/pad_ambient_80bpm.mid',
    'pad_ambient_80bpm.mid',
    decode('f6789012345678901234567890abcdef1234567890abcdef1234567890ab', 'hex'),
    4096,
    1,
    4,
    480,
    64.0,
    122880,
    false,
    'pads',
    'Spitfire Audio',
    'Ambient Textures',
    ARRAY['pad', 'ambient', 'texture']
);

-- Sample 7: Multi-track full composition (100 BPM, parent file)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    7,
    'full_song_100bpm.mid',
    '/library/songs/full_song_100bpm.mid',
    'full_song_100bpm.mid',
    decode('789012345678901234567890abcdef1234567890abcdef1234567890abcd', 'hex'),
    102400, -- ~100KB
    1,
    8, -- 8 tracks
    480,
    240.0,
    460800,
    false,
    'songs',
    'User Created',
    'Original Compositions',
    ARRAY['song', 'multi-track', 'complete']
);

-- Sample 8: Split track 1 from multi-track (drums)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_file_id, track_number, total_tracks, parent_folder
) VALUES (
    8,
    'full_song_100bpm_track_1_drums.mid',
    '/library/songs/splits/full_song_100bpm_track_1_drums.mid',
    'full_song_100bpm_track_1_drums.mid',
    decode('89012345678901234567890abcdef1234567890abcdef1234567890abcde', 'hex'),
    8192,
    0, -- Single track
    1,
    480,
    240.0,
    460800,
    true,
    7, -- Parent is file 7
    1,
    8,
    'songs'
);

-- Sample 9: Split track 2 from multi-track (bass)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_file_id, track_number, total_tracks, parent_folder
) VALUES (
    9,
    'full_song_100bpm_track_2_bass.mid',
    '/library/songs/splits/full_song_100bpm_track_2_bass.mid',
    'full_song_100bpm_track_2_bass.mid',
    decode('9012345678901234567890abcdef1234567890abcdef1234567890abcdef', 'hex'),
    6144,
    0,
    1,
    480,
    240.0,
    460800,
    true,
    7,
    2,
    8,
    'songs'
);

-- Sample 10: Duplicate file (same content_hash as file 1)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    10,
    'bass_slow_60bpm_copy.mid',
    '/library/bass/duplicates/bass_slow_60bpm_copy.mid',
    'bass_slow_60bpm_copy.mid',
    decode('a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456', 'hex'), -- Same as file 1
    1024,
    1,
    2,
    480,
    32.0,
    61440,
    false,
    'bass',
    'Splice',
    'Bass Essentials',
    ARRAY['bass', 'slow', 'deep']
);

-- Sample 11: Edge case - minimal file (100 bytes)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder
) VALUES (
    11,
    'minimal_test.mid',
    '/library/test/minimal_test.mid',
    'minimal_test.mid',
    decode('012345678901234567890abcdef1234567890abcdef1234567890abcdef1', 'hex'),
    100, -- Minimal size
    0,
    1,
    96,
    1.0,
    960,
    false,
    'test'
);

-- Sample 12: Edge case - NULL manufacturer/collection
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder
) VALUES (
    12,
    'unknown_origin.mid',
    '/library/unknown/unknown_origin.mid',
    'unknown_origin.mid',
    decode('12345678901234567890abcdef1234567890abcdef1234567890abcdef12', 'hex'),
    512,
    1,
    1,
    480,
    4.0,
    7680,
    false,
    'unknown'
);

-- Sample 13: Vocal sample (110 BPM)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    13,
    'vocal_melody_110bpm.mid',
    '/library/vocals/vocal_melody_110bpm.mid',
    'vocal_melody_110bpm.mid',
    decode('2345678901234567890abcdef1234567890abcdef1234567890abcdef123', 'hex'),
    2048,
    1,
    2,
    480,
    16.0,
    30720,
    false,
    'vocals',
    'Splice',
    'Vocal Samples',
    ARRAY['vocal', 'melody', 'sung']
);

-- Sample 14: FX riser (variable BPM)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    14,
    'fx_riser_tension.mid',
    '/library/fx/fx_riser_tension.mid',
    'fx_riser_tension.mid',
    decode('345678901234567890abcdef1234567890abcdef1234567890abcdef1234', 'hex'),
    768,
    0,
    1,
    480,
    2.0,
    3840,
    false,
    'fx',
    'Vengeance',
    'FX Collection',
    ARRAY['fx', 'riser', 'tension']
);

-- Sample 15: Lead synth (128 BPM, standard house)
INSERT INTO files (
    id, filename, filepath, original_filename, content_hash, file_size_bytes,
    format, num_tracks, ticks_per_quarter_note, duration_seconds, duration_ticks,
    is_multi_track, parent_folder, manufacturer, collection_name, folder_tags
) VALUES (
    15,
    'lead_synth_128bpm.mid',
    '/library/leads/lead_synth_128bpm.mid',
    'lead_synth_128bpm.mid',
    decode('45678901234567890abcdef1234567890abcdef1234567890abcdef12345', 'hex'),
    3584,
    1,
    2,
    480,
    32.0,
    61440,
    false,
    'leads',
    'Native Instruments',
    'Lead Synths',
    ARRAY['lead', 'synth', 'house']
);

-- Set sequence to next available value
SELECT setval('files_id_seq', 15);

-- =============================================================================
-- SECTION 2: MUSICAL METADATA
-- =============================================================================
-- Purpose: BPM, key, time signature, and note statistics
-- Coverage: Various musical characteristics
-- =============================================================================

-- File 1: Bass (60 BPM, C minor, monophonic)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, has_melody
) VALUES (
    1, 60.0, 0.95, 'Cm', 0.88,
    4, 4,
    128, 12, 36, 48, 85.5,
    4.0, 1, 1.0,
    true, false, false,
    false, true
);

-- File 2: Drums (90 BPM, no key, percussive)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, has_melody
) VALUES (
    2, 90.0, 0.99, 'UNKNOWN', NULL,
    4, 4,
    256, 8, 36, 49, 100.0,
    32.0, 4, 2.5,
    false, true, true,
    false, false
);

-- File 3: Piano (120 BPM, C major, polyphonic with chords)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, chord_complexity, has_melody, melodic_range
) VALUES (
    3, 120.0, 0.97, 'C', 0.92,
    4, 4,
    512, 24, 48, 84, 70.0,
    32.0, 6, 3.5,
    false, true, false,
    true, 0.65, true, 36
);

-- File 4: House chord (140 BPM, A minor)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, chord_complexity
) VALUES (
    4, 140.0, 0.98, 'Am', 0.85,
    4, 4,
    64, 8, 60, 76, 90.0,
    16.0, 4, 4.0,
    false, true, false,
    true, 0.45
);

-- File 5: DnB arp (174 BPM, D minor)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_melody, melodic_range
) VALUES (
    5, 174.0, 0.96, 'Dm', 0.90,
    4, 4,
    320, 16, 50, 74, 95.0,
    40.0, 1, 1.0,
    true, false, false,
    true, 24
);

-- File 6: Pad (80 BPM, G major, sustained)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, chord_complexity
) VALUES (
    6, 80.0, 0.92, 'G', 0.87,
    4, 4,
    64, 12, 48, 72, 50.0,
    1.0, 8, 6.5,
    false, true, false,
    true, 0.75
);

-- File 7: Full song (100 BPM, E minor, complex)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    has_tempo_changes, tempo_changes,
    has_key_changes, key_changes,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, chord_complexity, has_melody, melodic_range
) VALUES (
    7, 100.0, 0.99, 'Em', 0.93,
    4, 4,
    true, '[{"tick": 0, "bpm": 100.0}, {"tick": 115200, "bpm": 120.0}]'::jsonb,
    true, '[{"tick": 0, "key": "Em"}, {"tick": 230400, "key": "G"}]'::jsonb,
    2048, 48, 24, 96, 80.0,
    8.53, 12, 6.8,
    false, true, false,
    true, 0.82, true, 72
);

-- File 8: Split track (drums from file 7)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_chords, has_melody
) VALUES (
    8, 100.0, 0.99, 'UNKNOWN', NULL,
    4, 4,
    512, 10, 35, 51, 105.0,
    2.13, 3, 2.1,
    false, true, true,
    false, false
);

-- File 9: Split track (bass from file 7)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_melody
) VALUES (
    9, 100.0, 0.99, 'Em', 0.93,
    4, 4,
    256, 16, 28, 52, 88.0,
    1.07, 1, 1.0,
    true, false, false,
    true
);

-- File 13: Vocal (110 BPM, F major)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_melody, melodic_range
) VALUES (
    13, 110.0, 0.94, 'F', 0.89,
    4, 4,
    180, 20, 60, 84, 75.0,
    11.25, 1, 1.0,
    true, false, false,
    true, 24
);

-- File 15: Lead synth (128 BPM, A major)
INSERT INTO musical_metadata (
    file_id, bpm, bpm_confidence, key_signature, key_confidence,
    time_signature_numerator, time_signature_denominator,
    total_notes, unique_pitches, pitch_range_min, pitch_range_max, avg_velocity,
    note_density, polyphony_max, polyphony_avg,
    is_monophonic, is_polyphonic, is_percussive,
    has_melody, melodic_range
) VALUES (
    15, 128.0, 0.97, 'A', 0.91,
    4, 4,
    640, 18, 55, 91, 92.0,
    20.0, 2, 1.2,
    false, true, false,
    true, 36
);

-- =============================================================================
-- SECTION 3: FILE CATEGORIES
-- =============================================================================
-- Purpose: Category classification for files
-- =============================================================================

INSERT INTO file_categories (file_id, primary_category, secondary_category, confidence_score, is_manual, detected_from) VALUES
    (1, 'BASS', 'SUB_BASS', 0.92, false, 'filename_analysis'),
    (2, 'DRUM_LOOP', 'PERC', 0.98, false, 'midi_channel_analysis'),
    (3, 'PIANO', 'CHORD', 0.85, false, 'program_change_analysis'),
    (4, 'CHORD', 'STAB', 0.88, false, 'note_analysis'),
    (5, 'ARP', 'SEQUENCE', 0.90, false, 'pattern_analysis'),
    (6, 'PAD', 'ATMOSPHERE', 0.95, false, 'note_density_analysis'),
    (7, 'FULL_MIX', NULL, 0.99, true, 'user_tagged'),
    (8, 'DRUM_PATTERN', 'PERC', 0.97, false, 'track_split_analysis'),
    (9, 'BASS', NULL, 0.94, false, 'track_split_analysis'),
    (13, 'VOCAL', 'MELODY', 0.87, false, 'filename_analysis'),
    (14, 'RISER', 'FX', 0.93, false, 'filename_analysis'),
    (15, 'LEAD', 'MELODY', 0.91, false, 'program_change_analysis');

-- =============================================================================
-- SECTION 4: FILE INSTRUMENTS
-- =============================================================================
-- Purpose: Detected MIDI instruments in files
-- =============================================================================

-- File 1: Bass (channel 0, program 33 - Acoustic Bass)
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (1, 0, 33, 'Acoustic Bass', 'Bass', 'Acoustic Bass', 128, true, 85.5, 36, 48);

-- File 2: Drums (channel 9, GM Drum Kit)
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (2, 9, 0, 'Standard Drum Kit', 'Percussion', 'Drum Kit', 256, true, 100.0, 36, 49);

-- File 3: Piano (channel 0, program 0 - Acoustic Grand Piano)
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (3, 0, 0, 'Acoustic Grand Piano', 'Piano', 'Grand Piano', 512, true, 70.0, 48, 84);

-- File 4: Synth pad (channel 0, program 81 - Lead 2 (sawtooth))
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (4, 0, 81, 'Lead 2 (sawtooth)', 'Synth Lead', 'Sawtooth Lead', 64, true, 90.0, 60, 76);

-- File 5: Synth arp (channel 0, program 88 - Pad 1 (new age))
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (5, 0, 88, 'Pad 1 (new age)', 'Synth Pad', 'New Age Pad', 320, true, 95.0, 50, 74);

-- File 6: Pad (channel 0, program 89 - Pad 2 (warm))
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (6, 0, 89, 'Pad 2 (warm)', 'Synth Pad', 'Warm Pad', 64, true, 50.0, 48, 72);

-- File 7: Multiple instruments (full song)
INSERT INTO file_instruments (file_id, channel, program_number, program_name, instrument_family, instrument_type, note_count, is_primary, avg_velocity, pitch_range_low, pitch_range_high) VALUES
    (7, 9, 0, 'Standard Drum Kit', 'Percussion', 'Drum Kit', 512, true, 105.0, 35, 51),
    (7, 0, 33, 'Acoustic Bass', 'Bass', 'Acoustic Bass', 256, false, 88.0, 28, 52),
    (7, 1, 0, 'Acoustic Grand Piano', 'Piano', 'Grand Piano', 384, false, 72.0, 48, 84),
    (7, 2, 81, 'Lead 2 (sawtooth)', 'Synth Lead', 'Sawtooth Lead', 256, false, 85.0, 60, 91);

-- =============================================================================
-- SECTION 5: TAGS SYSTEM
-- =============================================================================
-- Purpose: Tag definitions and file-tag relationships
-- =============================================================================

-- Create tags
INSERT INTO tags (id, name, category, usage_count) VALUES
    (1, 'bass', 'instrument', 2),
    (2, 'drums', 'instrument', 2),
    (3, 'piano', 'instrument', 1),
    (4, 'synth', 'instrument', 3),
    (5, 'house', 'genre', 2),
    (6, 'dnb', 'genre', 1),
    (7, 'ambient', 'genre', 1),
    (8, 'dark', 'mood', 1),
    (9, 'energetic', 'mood', 2),
    (10, 'chill', 'mood', 2),
    (11, 'loop', 'type', 2),
    (12, 'one-shot', 'type', 1),
    (13, 'melody', 'type', 3),
    (14, 'chord', 'type', 2),
    (15, 'arp', 'type', 1);

SELECT setval('tags_id_seq', 15);

-- Tag files
INSERT INTO file_tags (file_id, tag_id, added_by) VALUES
    -- File 1: Bass
    (1, 1, 'system'),
    (1, 8, 'system'),
    (1, 10, 'system'),
    -- File 2: Drums
    (2, 2, 'system'),
    (2, 9, 'system'),
    (2, 11, 'system'),
    -- File 3: Piano
    (3, 3, 'system'),
    (3, 13, 'system'),
    (3, 10, 'system'),
    -- File 4: House chord
    (4, 4, 'system'),
    (4, 5, 'system'),
    (4, 9, 'system'),
    (4, 14, 'system'),
    -- File 5: DnB arp
    (5, 4, 'system'),
    (5, 6, 'system'),
    (5, 9, 'system'),
    (5, 15, 'system'),
    -- File 6: Ambient pad
    (6, 4, 'system'),
    (6, 7, 'system'),
    (6, 10, 'system'),
    -- File 7: Full song
    (7, 1, 'user'),
    (7, 2, 'user'),
    (7, 5, 'user'),
    -- File 13: Vocal
    (13, 13, 'system'),
    -- File 15: Lead synth
    (15, 4, 'system'),
    (15, 5, 'system'),
    (15, 13, 'system');

-- =============================================================================
-- SECTION 6: FAVORITES
-- =============================================================================
-- Purpose: User-favorited files
-- =============================================================================

INSERT INTO favorites (file_id) VALUES
    (3),  -- Piano melody
    (5),  -- DnB arp
    (7),  -- Full song
    (15); -- Lead synth

-- =============================================================================
-- SECTION 7: TRACK SPLITS
-- =============================================================================
-- Purpose: Parent-child relationships for split multi-track files
-- =============================================================================

INSERT INTO track_splits (parent_file_id, split_file_id, track_number, track_name, instrument, note_count) VALUES
    (7, 8, 1, 'Drums', 'Standard Drum Kit', 512),
    (7, 9, 2, 'Bass', 'Acoustic Bass', 256);

-- =============================================================================
-- SECTION 8: DUPLICATE GROUPS & FILES
-- =============================================================================
-- Purpose: Track duplicate files by content hash
-- =============================================================================

-- Duplicate group for files 1 and 10 (same content_hash)
INSERT INTO duplicate_groups (id, content_hash, canonical_file_id, duplicate_count, total_size_bytes) VALUES
    (1, decode('a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456', 'hex'), 1, 2, 2048);

SELECT setval('duplicate_groups_id_seq', 1);

INSERT INTO duplicate_files (group_id, file_id, is_canonical) VALUES
    (1, 1, true),   -- Original
    (1, 10, false); -- Duplicate

-- =============================================================================
-- SECTION 9: VECTOR EMBEDDINGS (SAMPLE DATA)
-- =============================================================================
-- Purpose: Vector embeddings for similarity search
-- Note: Using small random vectors for testing (not real embeddings)
-- =============================================================================

-- File 1: Bass (sample 768-dim vector)
INSERT INTO file_embeddings (file_id, overall_embedding, model_version, embedding_quality) VALUES
    (1, array_fill(0.1, ARRAY[768])::vector, 'test-v1', 0.85);

-- File 2: Drums
INSERT INTO file_embeddings (file_id, overall_embedding, rhythmic_embedding, model_version, embedding_quality) VALUES
    (2, array_fill(0.2, ARRAY[768])::vector, array_fill(0.9, ARRAY[256])::vector, 'test-v1', 0.92);

-- File 3: Piano
INSERT INTO file_embeddings (file_id, overall_embedding, harmonic_embedding, melodic_embedding, model_version, embedding_quality) VALUES
    (3, array_fill(0.3, ARRAY[768])::vector, array_fill(0.7, ARRAY[256])::vector, array_fill(0.6, ARRAY[256])::vector, 'test-v1', 0.88);

-- =============================================================================
-- SECTION 10: FILE COMPATIBILITY (SAMPLE PAIRS)
-- =============================================================================
-- Purpose: Pre-computed compatibility scores between files
-- Note: ordered_pair constraint requires file_id_a < file_id_b
-- =============================================================================

-- Files 1 and 3: Bass and Piano (compatible key)
INSERT INTO file_compatibility (file_id_a, file_id_b, overall_score, rhythmic_score, harmonic_score, melodic_score, timbral_score, key_compatible, bpm_compatible, time_signature_compatible) VALUES
    (1, 3, 0.72, 0.65, 0.88, 0.70, 0.65, true, false, true);

-- Files 2 and 4: Drums and House chord (compatible BPM range)
INSERT INTO file_compatibility (file_id_a, file_id_b, overall_score, rhythmic_score, harmonic_score, melodic_score, timbral_score, key_compatible, bpm_compatible, time_signature_compatible) VALUES
    (2, 4, 0.68, 0.85, 0.60, 0.55, 0.72, false, true, true);

-- Files 3 and 6: Piano and Pad (harmonic compatibility)
INSERT INTO file_compatibility (file_id_a, file_id_b, overall_score, rhythmic_score, harmonic_score, melodic_score, timbral_score, key_compatible, bpm_compatible, time_signature_compatible) VALUES
    (3, 6, 0.82, 0.70, 0.95, 0.78, 0.85, true, true, true);

-- =============================================================================
-- SECTION 11: RHYTHM PATTERNS
-- =============================================================================
-- Purpose: Rhythmic analysis data
-- =============================================================================

-- File 2: Drum loop rhythm pattern
INSERT INTO rhythm_patterns (file_id, pattern_type, pattern_signature, onset_times, inter_onset_intervals, swing_factor, groove_template, syncopation_score, pattern_length_beats, pattern_complexity) VALUES
    (2, 'four_on_floor', decode('aabbccdd', 'hex'), ARRAY[0, 480, 960, 1440, 1920], ARRAY[480, 480, 480, 480], 0.00, array_fill(0.8, ARRAY[16])::vector, 0.25, 4, 0.35);

-- File 5: DnB arp rhythm pattern
INSERT INTO rhythm_patterns (file_id, pattern_type, pattern_signature, onset_times, inter_onset_intervals, swing_factor, groove_template, syncopation_score, pattern_length_beats, pattern_complexity) VALUES
    (5, 'sixteenth_notes', decode('eeff1122', 'hex'), ARRAY[0, 120, 240, 360, 480, 600, 720, 840], ARRAY[120, 120, 120, 120, 120, 120, 120], 0.05, array_fill(0.9, ARRAY[16])::vector, 0.65, 2, 0.78);

-- =============================================================================
-- SECTION 12: HARMONIC PATTERNS
-- =============================================================================
-- Purpose: Chord progressions and harmonic analysis
-- =============================================================================

-- File 3: Piano chord progression (I-IV-V-I in C major)
INSERT INTO harmonic_patterns (file_id, chord_sequence, chord_types, chord_roots, roman_numerals, harmonic_rhythm, progression_length, harmonic_complexity, uses_seventh_chords, progression_hash) VALUES
    (3, ARRAY['C', 'F', 'G', 'C'], ARRAY['major', 'major', 'major', 'major'], ARRAY[0, 5, 7, 0], ARRAY['I', 'IV', 'V', 'I'], ARRAY[1920, 1920, 1920, 1920], 4, 0.35, false, decode('abcd1234', 'hex'));

-- File 4: House chord progression (i-VI-III-VII in A minor)
INSERT INTO harmonic_patterns (file_id, chord_sequence, chord_types, chord_roots, roman_numerals, harmonic_rhythm, progression_length, harmonic_complexity, uses_seventh_chords, progression_hash) VALUES
    (4, ARRAY['Am', 'F', 'C', 'G'], ARRAY['minor', 'major', 'major', 'major'], ARRAY[9, 5, 0, 7], ARRAY['i', 'VI', 'III', 'VII'], ARRAY[960, 960, 960, 960], 4, 0.42, false, decode('1234abcd', 'hex'));

-- =============================================================================
-- SECTION 13: MELODIC PATTERNS
-- =============================================================================
-- Purpose: Melodic analysis
-- =============================================================================

-- File 1: Bass melodic pattern
INSERT INTO melodic_patterns (file_id, pitch_sequence, interval_sequence, contour_direction, note_durations, motif_count, sequence_count, repetition_score, melodic_range, avg_interval_size, stepwise_motion_ratio, melodic_hash) VALUES
    (1, ARRAY[36, 38, 40, 38, 36], ARRAY[2, 2, -2, -2], ARRAY['up', 'up', 'down', 'down'], ARRAY[480, 480, 480, 480, 960], 2, 1, 0.65, 4, 2.0, 1.0, decode('aabb1122', 'hex'));

-- File 13: Vocal melody
INSERT INTO melodic_patterns (file_id, pitch_sequence, interval_sequence, contour_direction, note_durations, motif_count, sequence_count, repetition_score, melodic_range, avg_interval_size, stepwise_motion_ratio, melodic_hash) VALUES
    (13, ARRAY[60, 62, 64, 65, 67, 69, 71, 72], ARRAY[2, 2, 1, 2, 2, 2, 1], ARRAY['up', 'up', 'up', 'up', 'up', 'up', 'up'], ARRAY[480, 480, 480, 480, 480, 480, 480, 960], 1, 0, 0.25, 12, 1.71, 1.0, decode('ccdd3344', 'hex'));

-- =============================================================================
-- SECTION 14: PROCESSING JOBS
-- =============================================================================
-- Purpose: Track batch processing jobs
-- =============================================================================

-- Job 1: Completed import job
INSERT INTO processing_jobs (id, job_type, source_directory, total_files, processed_files, failed_files, skipped_files, status, started_at, completed_at, settings) VALUES
    ('a1b2c3d4-e5f6-4789-0123-456789abcdef', 'batch_import', '/library/bass/', 150, 145, 3, 2, 'completed', NOW() - INTERVAL '2 hours', NOW() - INTERVAL '1 hour', '{"verify_files": true, "skip_duplicates": true}'::jsonb);

-- Job 2: In-progress analysis job
INSERT INTO processing_jobs (id, job_type, source_directory, total_files, processed_files, failed_files, skipped_files, status, started_at, settings) VALUES
    ('b2c3d4e5-f678-4901-2345-6789abcdef12', 'bpm_analysis', '/library/melody/', 200, 120, 5, 0, 'running', NOW() - INTERVAL '30 minutes', '{"algorithm": "autocorrelation", "min_bpm": 60, "max_bpm": 200}'::jsonb);

-- Job 3: Failed job
INSERT INTO processing_jobs (id, job_type, source_directory, total_files, processed_files, failed_files, skipped_files, status, error_message, started_at, completed_at) VALUES
    ('c3d4e5f6-7890-4123-4567-89abcdef1234', 'duplicate_scan', '/library/', 1000, 850, 150, 0, 'failed', 'Database connection timeout', NOW() - INTERVAL '3 hours', NOW() - INTERVAL '2 hours');

-- =============================================================================
-- SECTION 15: PROCESSING ERRORS
-- =============================================================================
-- Purpose: Track errors during processing
-- =============================================================================

-- Errors from job 1
INSERT INTO processing_errors (job_id, filepath, error_type, error_message, occurred_at) VALUES
    ('a1b2c3d4-e5f6-4789-0123-456789abcdef', '/library/bass/corrupted_file.mid', 'ParseError', 'Invalid MIDI header: expected MThd', NOW() - INTERVAL '90 minutes'),
    ('a1b2c3d4-e5f6-4789-0123-456789abcdef', '/library/bass/invalid_format.mid', 'ValidationError', 'File size too small (< 14 bytes)', NOW() - INTERVAL '85 minutes'),
    ('a1b2c3d4-e5f6-4789-0123-456789abcdef', '/library/bass/permission_denied.mid', 'IOError', 'Permission denied', NOW() - INTERVAL '80 minutes');

-- Errors from job 2
INSERT INTO processing_errors (job_id, filepath, error_type, error_message, occurred_at) VALUES
    ('b2c3d4e5-f678-4901-2345-6789abcdef12', '/library/melody/no_tempo.mid', 'AnalysisError', 'No tempo events found', NOW() - INTERVAL '25 minutes'),
    ('b2c3d4e5-f678-4901-2345-6789abcdef12', '/library/melody/ambiguous_bpm.mid', 'AnalysisError', 'Multiple conflicting tempo markers', NOW() - INTERVAL '20 minutes');

-- =============================================================================
-- VERIFICATION QUERIES
-- =============================================================================
-- Purpose: Verify test data was inserted correctly
-- =============================================================================

DO $$
DECLARE
    files_count INTEGER;
    metadata_count INTEGER;
    tags_count INTEGER;
    favorites_count INTEGER;
BEGIN
    -- Count records
    SELECT COUNT(*) INTO files_count FROM files;
    SELECT COUNT(*) INTO metadata_count FROM musical_metadata;
    SELECT COUNT(*) INTO tags_count FROM tags;
    SELECT COUNT(*) INTO favorites_count FROM favorites;

    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'TEST DATA SUMMARY';
    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'Files: %', files_count;
    RAISE NOTICE 'Musical metadata: %', metadata_count;
    RAISE NOTICE 'Tags: %', tags_count;
    RAISE NOTICE 'Favorites: %', favorites_count;
    RAISE NOTICE '=============================================================================';
    RAISE NOTICE 'Test fixtures loaded successfully!';
    RAISE NOTICE '=============================================================================';
END $$;

COMMIT;

-- =============================================================================
-- USAGE EXAMPLES
-- =============================================================================
/*

-- Find all files with BPM between 120-140
SELECT f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
WHERE m.bpm BETWEEN 120 AND 140;

-- Get all files tagged as 'house'
SELECT f.filename, f.filepath
FROM files f
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name = 'house';

-- Find compatible files for file 3 (piano)
SELECT f.filename, fc.overall_score, fc.key_compatible, fc.bpm_compatible
FROM file_compatibility fc
JOIN files f ON (fc.file_id_a = f.id OR fc.file_id_b = f.id)
WHERE (fc.file_id_a = 3 OR fc.file_id_b = 3)
AND f.id != 3
ORDER BY fc.overall_score DESC;

-- Get all user favorites
SELECT f.filename, f.filepath, fav.created_at
FROM favorites fav
JOIN files f ON fav.file_id = f.id
ORDER BY fav.created_at DESC;

-- Find all split tracks from parent file 7
SELECT ts.track_number, ts.track_name, ts.instrument, f.filename
FROM track_splits ts
JOIN files f ON ts.split_file_id = f.id
WHERE ts.parent_file_id = 7
ORDER BY ts.track_number;

*/
